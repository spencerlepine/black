//! Spec file parser (grammar: tools/rom-diff/svflow_specs/README.md).

use crate::{num, Rng};

pub(crate) const DEFAULT_INPUTS: u32 = 200;
/// Fixed address of pointer argument 0 and the distance between pointer arguments.
pub(crate) const BUF_BASE: u32 = 0x02E1_0000;
/// Fixed address of the `work` image.
pub(crate) const WORK_BASE: u32 = 0x02E0_0000;
pub(crate) const BUF_STRIDE: u32 = 0x1_0000;

#[derive(Clone, Debug)]
pub(crate) enum Gen {
    Any,
    Range(u32, u32),
    Near(u32, u32),
    OneOf(Vec<u32>),
    Const(u32),
    Mask(u32),
    AddrOf(String, u32),
    Alt(Vec<Gen>),
    /// `sum16 OFS SIZE`: u16 halfword sum of the fix target's bytes `[OFS, OFS + SIZE)`; no draw.
    Sum16(usize, usize),
    /// Earlier argument K plus a draw (wrapping).
    Rel(usize, Box<Gen>),
    /// Earlier argument K xor a draw.
    XRel(usize, Box<Gen>),
    /// (Earlier argument K & MASK) plus a draw (wrapping).
    RelM(usize, u32, Box<Gen>),
    /// `ifeq K V THEN || ELSE`: earlier argument K == V draws THEN, else ELSE (only the chosen one draws).
    IfEq(usize, u32, Box<Gen>, Box<Gen>),
}

impl Gen {
    fn parse(t: &[&str]) -> Gen {
        match t[0] {
            "alt" => Gen::Alt(t[1..].split(|x| *x == "||").map(Gen::parse).collect()),
            "rel" => Gen::Rel(num(t[1]) as usize, Box::new(Gen::parse(&t[2..]))),
            "relm" => Gen::RelM(num(t[1]) as usize, num(t[2]), Box::new(Gen::parse(&t[3..]))),
            "ifeq" => {
                let i = t
                    .iter()
                    .position(|x| *x == "||")
                    .expect("fndiff spec: ifeq needs ||");
                Gen::IfEq(
                    num(t[1]) as usize,
                    num(t[2]),
                    Box::new(Gen::parse(&t[3..i])),
                    Box::new(Gen::parse(&t[i + 1..])),
                )
            }
            "xrel" => Gen::XRel(num(t[1]) as usize, Box::new(Gen::parse(&t[2..]))),
            "any" => Gen::Any,
            "range" => {
                range_check(t[1], t[2]);
                Gen::Range(num(t[1]), num(t[2]))
            }
            "near" => Gen::Near(num(t[1]), num(t[2])),
            "oneof" => Gen::OneOf(t[1..].iter().map(|x| num(x)).collect()),
            "const" => Gen::Const(num(t[1])),
            "mask" => Gen::Mask(num(t[1])),
            "addrof" => Gen::AddrOf(t[1].to_string(), t.get(2).map_or(0, |x| num(x))),
            "sum16" => Gen::Sum16(num(t[1]) as usize, num(t[2]) as usize),
            k => panic!("fndiff spec: bad generator {k}"),
        }
    }

    pub(crate) fn draw(
        &self,
        r: &mut Rng,
        addr_of: &dyn Fn(&str) -> u32,
        tgt: Option<&[u8]>,
        vals: &[u32],
    ) -> u32 {
        match self {
            Gen::Sum16(ofs, size) => {
                let t = tgt.expect("fndiff spec: sum16 outside a fix");
                (0..size / 2).fold(0u16, |a, i| {
                    a.wrapping_add(u16::from_le_bytes([t[ofs + 2 * i], t[ofs + 2 * i + 1]]))
                }) as u32
            }
            Gen::Any => match r.next() % 8 {
                0 => 0,
                1 => 1,
                2 => 0xFF,
                3 => r.next() % 0x100,
                4 => r.next() % 0x10000,
                _ => r.next(),
            },
            Gen::Range(lo, hi) => {
                lo.wrapping_add((r.next() as u64 % (hi.wrapping_sub(*lo) as u64 + 1)) as u32)
            }
            Gen::Near(n, d) => (n.wrapping_sub(*d)).wrapping_add(r.next() % (2 * d + 1)),
            Gen::OneOf(v) => v[r.next() as usize % v.len()],
            Gen::Const(v) => *v,
            Gen::Mask(m) => r.next() & m,
            Gen::AddrOf(t, o) => addr_of(t) + o,
            Gen::Alt(v) => {
                let i = r.next() as usize % v.len();
                v[i].draw(r, addr_of, tgt, vals)
            }
            Gen::Rel(k, g) => vals[*k].wrapping_add(g.draw(r, addr_of, tgt, vals)),
            Gen::XRel(k, g) => vals[*k] ^ g.draw(r, addr_of, tgt, vals),
            Gen::RelM(k, m, g) => (vals[*k] & m).wrapping_add(g.draw(r, addr_of, tgt, vals)),
            Gen::IfEq(k, v, a, b) => {
                let g = if vals[*k] == *v { a } else { b };
                g.draw(r, addr_of, tgt, vals)
            }
        }
    }
}

pub(crate) fn conv(v: u32, ty: &str) -> u32 {
    match ty {
        "u8" => v & 0xFF,
        "u16" => v & 0xFFFF,
        "u32" | "s32" => v,
        "s8" => v as u8 as i8 as i32 as u32,
        "s16" => v as u16 as i16 as i32 as u32,
        _ => panic!("fndiff spec: bad type {ty}"),
    }
}

pub(crate) fn width(ty: &str) -> usize {
    match ty {
        "u8" | "s8" => 1,
        "u16" | "s16" => 2,
        _ => 4,
    }
}

pub(crate) fn ret_value(kind: &str, val: u32) -> Option<u32> {
    match kind {
        "none" => None,
        "bool" => Some(val & 1),
        "u8" => Some(val & 0xFF),
        "u16" => Some(val & 0xFFFF),
        "u32" | "i32" => Some(val),
        // B14-spec: a stub's 64-bit result; this is the low word (r0), `StubEnv::call64` adds r1
        "u64" => Some(val),
        k => {
            if let Some(n) = k.strip_prefix("mod:") {
                Some(val % num(n))
            } else if let Some(m) = k.strip_prefix("mask:") {
                Some(val & num(m))
            } else if let Some(p) = k.strip_prefix("nullable:") {
                // `nullable:Z:KIND` = 0 when mix(val, 0x1234) % Z == 0, else KIND
                let (z, inner) = p.split_once(':').unwrap();
                if crate::mix(val, 0x1234).is_multiple_of(num(z)) {
                    Some(0)
                } else {
                    ret_value(inner, val)
                }
            } else if let Some(p) = k.strip_prefix("pick:") {
                // one of V1/V2/... chosen by mix(val, 0xC0DE) % count
                let vs: Vec<u32> = p.split('/').map(num).collect();
                Some(vs[crate::mix(val, 0xC0DE) as usize % vs.len()])
            } else if let Some(p) = k.strip_prefix("pick+") {
                Some(pick_any(k, p, val))
            } else if let Some(p) = k.strip_prefix("ptr:work:") {
                // stub result: an address inside the work image, `ptr:work:OFS:N` = WORK + OFS + 4 * (mix(val, 0xC0DE) % N)
                let (ofs, n) = p.split_once(':').unwrap();
                Some(
                    WORK_BASE
                        .wrapping_add(num(ofs))
                        .wrapping_add(4 * (crate::mix(val, 0xC0DE) % num(n))),
                )
            } else if let Some(p) = k.strip_prefix("ptr:arg") {
                // stub result: an address inside pointer argument K, `ptr:argK:OFS:N` =
                // base(K) + OFS + 4 * (mix(val, 0xC0DE) % N)
                let mut it = p.split(':');
                let (k, ofs, n) = (
                    num(it.next().unwrap()),
                    num(it.next().unwrap()),
                    num(it.next().unwrap()),
                );
                Some(
                    BUF_BASE
                        .wrapping_add(k.wrapping_mul(BUF_STRIDE))
                        .wrapping_add(ofs)
                        .wrapping_add(4 * (crate::mix(val, 0xC0DE) % n)),
                )
            } else if k == "ofs:work" || k.starts_with("ofs:arg") {
                // pointer into a pointer argument (or, `ofs:work`, into the work image): the port returns the offset, 0 for null
                Some(val)
            } else {
                panic!("fndiff spec: bad ret kind {k}")
            }
        }
    }
}

/// Selector salt of `pick+W:` (independent of the 0xC0DE list index; `PICK_ANY_SALT` in svflow.py).
pub(crate) const PICK_ANY_SALT: u32 = 0xA11;
/// 1 in `PICK_ANY_FULL` answers of `pick+W:` is full-width (`PICK_ANY_FULL` in svflow.py).
pub(crate) const PICK_ANY_FULL: u32 = 4;

/// `pick+W:V1/V2/..` (Z23; W = u8 | u16 | u32 | i32): a weighted `pick:` that keeps the whole width W reachable.
/// When `mix(mix(val, 0xA11), 0x5EED) % 4 == 0` the answer is the plain W kind (`val` & the mask of W), otherwise
/// `V[mix(val, 0xC0DE) % count]` (the value `pick:V1/V2/..` gives for the same `val`). Every V must fit W
/// (`pick_any` in svflow.py). The selector is mixed twice: `mix(val, a) ^ mix(val, b)` is a constant, so a single
/// mix would decide the low bits of the list index (a 4-value list would never draw one of its values).
fn pick_any(kind: &str, p: &str, val: u32) -> u32 {
    let (w, list) = p
        .split_once(':')
        .unwrap_or_else(|| panic!("fndiff spec: bad ret kind {kind}"));
    let mask = match w {
        "u8" => 0xFF,
        "u16" => 0xFFFF,
        "u32" | "i32" => u32::MAX,
        _ => panic!("fndiff spec: bad ret kind {kind}"),
    };
    assert!(!list.is_empty(), "fndiff spec: bad ret kind {kind}");
    let vs: Vec<u64> = list.split('/').map(num64).collect();
    assert!(
        vs.iter().all(|&v| v <= u64::from(mask)),
        "fndiff spec: ret kind {kind}: value wider than {w}"
    );
    if crate::mix(crate::mix(val, PICK_ANY_SALT), 0x5EED).is_multiple_of(PICK_ANY_FULL) {
        return val & mask;
    }
    vs[crate::mix(val, 0xC0DE) as usize % vs.len()] as u32
}

/// Z25: `range LO HI` needs LO <= HI as integers (a leading `-` is a negative integer that wraps into the word, as in
/// svflow.py) and HI - LO < 2^32; svflow.py and this parser used to draw differently for HI < LO, so both reject it.
fn range_check(lo: &str, hi: &str) {
    let signed = |s: &str| -> i64 {
        let (neg, t) = match s.trim().strip_prefix('-') {
            Some(t) => (true, t),
            None => (false, s.trim()),
        };
        let v = num(t) as i64;
        if neg {
            -v
        } else {
            v
        }
    };
    let (l, h) = (signed(lo), signed(hi));
    assert!(
        h >= l && h - l < 1 << 32,
        "fndiff spec: bad generator: `range LO HI` needs LO <= HI and HI - LO < 2^32 (got {lo} {hi})"
    );
}

/// `int(x, 0)` for a list value that may not fit 32 bits (`pick+W:` rejects it instead of wrapping).
fn num64(s: &str) -> u64 {
    let s = s.trim();
    let r = if let Some(h) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u64::from_str_radix(h, 16)
    } else {
        s.parse()
    };
    r.unwrap_or_else(|_| panic!("fndiff spec: bad number {s}"))
}

#[derive(Clone, Debug)]
pub(crate) enum ArgSpec {
    Scalar(String, Gen),
    /// Pointer into the legacy 0x212C-byte `work` image.
    Work(u32),
    Buf(usize),
    /// Pointer into the named `region` (offset added to its base).
    Region(String, u32),
    /// Argument kind owned by the calling crate (e.g. `bpp`); see [`crate::Ext::custom_arg`].
    Custom(String),
}

#[derive(Clone, Debug)]
pub(crate) struct Fix {
    pub target: String,
    pub ofs: usize,
    pub ty: String,
    pub gen: Gen,
    pub bits: Option<(u32, u32)>,
    /// `crypt` directive instead of a generated value.
    pub crypt: Option<CryptFix>,
    /// `ramp TARGET OFS COUNT TYPE GEN` (P-spec): COUNT values, each the previous one (0 before the
    /// first) plus a draw; `Some(COUNT)`.
    pub ramp: Option<usize>,
}

/// `crypt TARGET OFS SIZE u16@SEEDOFS|u32@SEEDOFS [skipif=uN@OFS&MASK]` (P1): XOR the halfwords of
/// `[OFS, OFS + SIZE)` with the poke_tool LCG32 stream (`state = state * 0x41C64E6D + 0x6073`,
/// key = `state >> 16`), seeded by the little-endian field at SEEDOFS; skipped when the
/// `skipif` field `& MASK` is nonzero. No random draw.
#[derive(Clone, Debug)]
pub(crate) struct CryptFix {
    pub size: usize,
    pub seed_width: usize,
    pub seed_ofs: usize,
    /// (width, offset, mask)
    pub skip: Option<(usize, usize, u32)>,
}

#[derive(Clone, Debug)]
pub(crate) struct StubSpec {
    pub addr: u32,
    pub ret: String,
    pub args: Vec<String>,
    pub noreturn: bool,
    /// `seq`: the 0-based index of the call joins the result (a polled callee changes its answer).
    pub seq: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct FnSpec {
    pub addr: u32,
    pub ret: String,
    pub inputs: u32,
    pub envs: Vec<String>,
    pub nowork: bool,
    pub args: Vec<ArgSpec>,
    pub fixes: Vec<Fix>,
    pub globals: Vec<(u32, usize)>,
    pub regions: Vec<(String, usize)>,
    pub stubs: Vec<StubSpec>,
}

pub(crate) fn parse_spec(text: &str) -> Vec<FnSpec> {
    assert!(
        !text.contains("NEEDS-HAND"),
        "fndiff spec still has NEEDS-HAND lines (genspec.py draft)"
    );
    let mut out: Vec<FnSpec> = Vec::new();
    for line in text.lines() {
        let line = line.split('#').next().unwrap().trim();
        if line.is_empty() {
            continue;
        }
        let t: Vec<&str> = line.split_whitespace().collect();
        let opt = |k: &str| {
            t.iter()
                .skip(2)
                .find_map(|x| x.strip_prefix(k).and_then(|y| y.strip_prefix('=')))
        };
        match t[0] {
            "area" | "set" => {}
            "fn" => out.push(FnSpec {
                addr: u32::from_str_radix(t[1], 16).unwrap(),
                ret: opt("ret").unwrap_or("none").to_string(),
                inputs: opt("inputs").map_or(DEFAULT_INPUTS, num),
                envs: vec![],
                nowork: false,
                args: vec![],
                fixes: vec![],
                globals: vec![],
                regions: vec![],
                stubs: vec![],
            }),
            d => {
                let f = out.last_mut().expect("fndiff spec: directive before fn");
                match d {
                    "arg" => f.args.push(match t[1] {
                        "work" => ArgSpec::Work(t.get(2).map_or(0, |x| num(x))),
                        "buf" => ArgSpec::Buf(num(t[2]) as usize),
                        "region" => {
                            ArgSpec::Region(t[2].to_string(), t.get(3).map_or(0, |x| num(x)))
                        }
                        "u8" | "u16" | "u32" | "s8" | "s16" | "s32" => {
                            ArgSpec::Scalar(t[1].to_string(), Gen::parse(&t[2..]))
                        }
                        kind => ArgSpec::Custom(kind.to_string()),
                    }),
                    "fix" => {
                        let mut rest = &t[4..];
                        let (mut bits, mut reps) = (None, (1, 0));
                        while let Some(o) = rest
                            .last()
                            .filter(|o| o.starts_with("bits=") || o.starts_with("rep="))
                        {
                            let (a, b) =
                                o.split_once('=').unwrap().1.split_once([':', ',']).unwrap();
                            if o.starts_with("bits=") {
                                bits = Some((num(a), num(b)));
                            } else {
                                reps = (num(a), num(b));
                            }
                            rest = &rest[..rest.len() - 1];
                        }
                        for i in 0..reps.0 {
                            f.fixes.push(Fix {
                                target: t[1].to_string(),
                                ofs: num(t[2]) as usize + (i * reps.1) as usize,
                                ty: t[3].to_string(),
                                gen: Gen::parse(rest),
                                bits,
                                crypt: None,
                                ramp: None,
                            });
                        }
                    }
                    "crypt" => {
                        let (sw, so) = t[4].split_once('@').unwrap();
                        let skip = t.get(5).map(|o| {
                            let (kw, km) = o.split_once('=').unwrap().1.split_once('&').unwrap();
                            let (kw, ko) = kw.split_once('@').unwrap();
                            (width(kw), num(ko) as usize, num(km))
                        });
                        f.fixes.push(Fix {
                            target: t[1].to_string(),
                            ofs: num(t[2]) as usize,
                            ty: "u32".to_string(),
                            gen: Gen::Const(0),
                            bits: None,
                            ramp: None,
                            crypt: Some(CryptFix {
                                size: num(t[3]) as usize,
                                seed_width: width(sw),
                                seed_ofs: num(so) as usize,
                                skip,
                            }),
                        });
                    }
                    "ramp" => f.fixes.push(Fix {
                        target: t[1].to_string(),
                        ofs: num(t[2]) as usize,
                        ty: t[4].to_string(),
                        gen: Gen::parse(&t[5..]),
                        bits: None,
                        crypt: None,
                        ramp: Some(num(t[3]) as usize),
                    }),
                    "global" => f
                        .globals
                        .push((u32::from_str_radix(t[1], 16).unwrap(), num(t[2]) as usize)),
                    "region" => f.regions.push((t[1].to_string(), num(t[2]) as usize)),
                    "nowork" => f.nowork = true,
                    "env" => f.envs.push(t[1].to_string()),
                    "stub" => f.stubs.push(StubSpec {
                        addr: u32::from_str_radix(t[1], 16).unwrap(),
                        ret: opt("ret").unwrap_or("none").to_string(),
                        args: opt("args")
                            .map(|a| {
                                a.split(',')
                                    .filter(|x| !x.is_empty())
                                    .map(str::to_string)
                                    .collect()
                            })
                            .unwrap_or_default(),
                        noreturn: t[2..].contains(&"noreturn"),
                        seq: t[2..].contains(&"seq"),
                    }),
                    _ => panic!("fndiff spec: unknown directive {line:?}"),
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(t: &[&str]) -> bool {
        std::panic::catch_unwind(|| Gen::parse(t)).is_ok()
    }

    /// Z25: the same vectors as `range_lint_selftest` in tools/rom-diff/svflow.py (run by `svflow.py --selftest`).
    #[test]
    fn range_lint_vectors() {
        assert!(ok(&["range", "5", "5"]));
        assert!(ok(&["range", "0", "0xFFFFFFFF"]));
        assert!(ok(&["range", "-5", "5"]));
        assert!(!ok(&["range", "10", "5"]));
        assert!(!ok(&["range", "1", "0"]));
        assert!(!ok(&["range", "0xFFFFFFFF", "0"]));
        assert!(!ok(&["range", "-1", "0xFFFFFFFF"]));
    }
}
