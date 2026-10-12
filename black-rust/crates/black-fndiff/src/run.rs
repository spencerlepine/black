//! Input generation, hashing and the per-area suite (mirrors `Runner`/`run_fn`/`main` of svflow.py).

use crate::spec::{conv, parse_spec, ret_value, width, ArgSpec, FnSpec};
use crate::stub::{Halt, StubEnv};
use crate::{fold, Rng, FNV_OFFSET};

const WORK: u32 = 0x02E0_0000;
/// Size of the legacy `work` image (`SvFlowWork`).
const WORK_SIZE: usize = 0x212C;
use crate::spec::{BUF_BASE, BUF_STRIDE};
const REGION_BASE: u32 = 0x02F1_0000;

/// Hooks for argument and environment kinds owned by one crate (e.g. black-battle's `bpp`
/// BattleMon image and `env bpp` API stubs). A crate without any uses `()`.
pub trait Ext: Default {
    /// Called right after the per-input salt is drawn, when the function has `env` lines.
    fn begin(&mut self, _envs: &[String], _r: &mut Rng) {}
    /// Bytes behind an `arg <kind>` the generic kit does not know; hashed like a `buf`.
    fn custom_arg(&mut self, kind: &str, _r: &mut Rng) -> Vec<u8> {
        panic!("fndiff spec: unknown arg kind {kind:?}")
    }
    /// Words hashed in the `env` section (the driver adds the count in front).
    fn env_words(&self, _envs: &[String]) -> Vec<u32> {
        Vec::new()
    }
}

impl Ext for () {}

/// One generated input. `vals[i]` is argument `i` as passed to the original (a scalar, or the
/// fixed emulator address of a pointer argument); `bufs[i]` holds the bytes behind a `buf` or
/// custom argument (empty otherwise). The port runs on these and the suite hashes them afterwards.
pub struct Case<X: Ext = ()> {
    /// The legacy `work` image (0x212C bytes); empty when the function says `nowork`.
    pub work: Vec<u8>,
    pub vals: Vec<u32>,
    pub bufs: Vec<Vec<u8>>,
    /// `region` contents in declaration order.
    pub regions: Vec<Vec<u8>>,
    region_names: Vec<String>,
    pub globals: Vec<Vec<u8>>,
    pub env: StubEnv,
    pub ext: X,
}

impl<X: Ext> Case<X> {
    fn region_index(&self, name: &str) -> usize {
        self.region_names
            .iter()
            .position(|n| n == name)
            .unwrap_or_else(|| panic!("no region {name:?} in this function's spec"))
    }

    pub fn region(&self, name: &str) -> &[u8] {
        &self.regions[self.region_index(name)]
    }

    pub fn region_mut(&mut self, name: &str) -> &mut [u8] {
        let i = self.region_index(name);
        &mut self.regions[i]
    }
}

struct Sink {
    h: u64,
    keep: Option<Vec<(String, u32)>>,
}

impl Sink {
    fn sec(&mut self, name: &str, ws: &[u32]) {
        for &w in ws {
            self.h = fold(self.h, w);
            if let Some(k) = self.keep.as_mut() {
                k.push((name.to_string(), w));
            }
        }
    }
}

fn le(b: &[u8]) -> u32 {
    let mut a = [0u8; 4];
    a[..b.len()].copy_from_slice(b);
    u32::from_le_bytes(a)
}

/// `crypt` directive (see `spec::CryptFix`); mirrors `crypt_fix` in svflow.py.
fn crypt_fix(tgt: &mut [u8], ofs: usize, cf: &crate::spec::CryptFix) {
    if let Some((w, o, m)) = cf.skip {
        if le(&tgt[o..o + w]) & m != 0 {
            return;
        }
    }
    let mut state = le(&tgt[cf.seed_ofs..cf.seed_ofs + cf.seed_width]);
    for i in 0..cf.size / 2 {
        state = state.wrapping_mul(0x41C6_4E6D).wrapping_add(0x6073);
        let o = ofs + 2 * i;
        let v = u16::from_le_bytes([tgt[o], tgt[o + 1]]) ^ (state >> 16) as u16;
        tgt[o..o + 2].copy_from_slice(&v.to_le_bytes());
    }
}

fn fill(r: &mut Rng, n: usize) -> Vec<u8> {
    let mut v = Vec::with_capacity(n);
    for _ in 0..n / 4 {
        v.extend(r.next().to_le_bytes());
    }
    v
}

fn words(b: &[u8]) -> Vec<u32> {
    b.chunks(4)
        .map(|c| u32::from_le_bytes(c.try_into().unwrap()))
        .collect()
}

fn buf_addr(i: usize) -> u32 {
    BUF_BASE + i as u32 * BUF_STRIDE
}

fn region_addr(k: usize) -> u32 {
    REGION_BASE + k as u32 * BUF_STRIDE
}

type Kept = Option<Vec<(String, u32)>>;

fn run_input<X: Ext>(
    fs: &FnSpec,
    seed: u32,
    call: &dyn Fn(u32, &mut Case<X>) -> u64,
    keep: bool,
) -> (String, Kept) {
    let mut r = Rng::new(fs.addr ^ seed.wrapping_mul(0x9E37));
    let salt = r.next();
    let mut ext = X::default();
    if !fs.envs.is_empty() {
        ext.begin(&fs.envs, &mut r);
    }
    let work = if fs.nowork {
        Vec::new()
    } else {
        fill(&mut r, WORK_SIZE)
    };
    let region_names: Vec<String> = fs.regions.iter().map(|(n, _)| n.clone()).collect();
    let addr_of = |t: &str| -> u32 {
        if t == "work" {
            WORK
        } else if let Some(n) = t.strip_prefix("region:") {
            region_addr(region_names.iter().position(|x| x == n).unwrap())
        } else if let Some(i) = t.strip_prefix("arg") {
            buf_addr(i.parse().unwrap())
        } else {
            fs.globals[t.strip_prefix("global").unwrap().parse::<usize>().unwrap()].0
        }
    };
    let mut vals = Vec::new();
    let mut bufs: Vec<Vec<u8>> = Vec::new();
    for (i, a) in fs.args.iter().enumerate() {
        match a {
            ArgSpec::Scalar(ty, g) => {
                let v = conv(g.draw(&mut r, &addr_of, None, &vals), ty);
                vals.push(v);
                bufs.push(Vec::new());
            }
            ArgSpec::Work(ofs) => {
                vals.push(WORK + ofs);
                bufs.push(Vec::new());
            }
            ArgSpec::Region(n, ofs) => {
                vals.push(addr_of(&format!("region:{n}")) + ofs);
                bufs.push(Vec::new());
            }
            ArgSpec::Buf(n) => {
                bufs.push(fill(&mut r, *n));
                vals.push(buf_addr(i));
            }
            ArgSpec::Custom(kind) => {
                bufs.push(ext.custom_arg(kind, &mut r));
                vals.push(buf_addr(i));
            }
        }
    }
    let mut globals: Vec<Vec<u8>> = fs.globals.iter().map(|&(_, n)| fill(&mut r, n)).collect();
    let mut regions: Vec<Vec<u8>> = fs.regions.iter().map(|&(_, n)| fill(&mut r, n)).collect();
    let mut c = Case {
        work,
        vals,
        bufs,
        regions: Vec::new(),
        region_names: region_names.clone(),
        globals: Vec::new(),
        env: StubEnv::new(fs.stubs.clone(), salt),
        ext,
    };
    for fx in &fs.fixes {
        let tgt: &mut [u8] = if fx.target == "work" {
            &mut c.work[..]
        } else if let Some(n) = fx.target.strip_prefix("region:") {
            let k = region_names.iter().position(|x| x == n).unwrap();
            &mut regions[k][..]
        } else if let Some(i) = fx.target.strip_prefix("arg") {
            &mut c.bufs[i.parse::<usize>().unwrap()][..]
        } else {
            &mut globals[fx
                .target
                .strip_prefix("global")
                .unwrap()
                .parse::<usize>()
                .unwrap()][..]
        };
        if let Some(cf) = &fx.crypt {
            crypt_fix(tgt, fx.ofs, cf);
            continue;
        }
        let w = width(&fx.ty);
        if let Some(n) = fx.ramp {
            let mut acc = 0u32;
            for i in 0..n {
                let d = conv(fx.gen.draw(&mut r, &addr_of, Some(&*tgt), &c.vals), &fx.ty);
                acc = acc.wrapping_add(d);
                let o = fx.ofs + i * w;
                tgt[o..o + w].copy_from_slice(&acc.to_le_bytes()[..w]);
            }
            continue;
        }
        let v = conv(fx.gen.draw(&mut r, &addr_of, Some(&*tgt), &c.vals), &fx.ty);
        let mut v = v;
        if let Some((lo, bw)) = fx.bits {
            let m = (((1u64 << bw) - 1) << lo) as u32;
            let mut cur = [0u8; 4];
            cur[..w].copy_from_slice(&tgt[fx.ofs..fx.ofs + w]);
            v = (u32::from_le_bytes(cur) & !m) | ((v << lo) & m);
        }
        tgt[fx.ofs..fx.ofs + w].copy_from_slice(&v.to_le_bytes()[..w]);
    }
    c.globals = globals;
    c.regions = regions;
    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| call(fs.addr, &mut c)));
    let ret = match res {
        Ok(r) => r,
        Err(e) if e.is::<Halt>() => 0,
        Err(e) => std::panic::resume_unwind(e),
    };
    let mut s = Sink {
        h: FNV_OFFSET,
        keep: keep.then(Vec::new),
    };
    if let Some(a) = c.env.halted() {
        s.sec("halt", &[1, a]);
    } else {
        s.sec("halt", &[0]);
        if fs.ret == "u64" {
            s.sec("ret", &[ret as u32, (ret >> 32) as u32]);
        } else if let Some(v) = ret_value(&fs.ret, ret as u32) {
            s.sec("ret", &[v]);
        }
    }
    if !fs.nowork {
        s.sec("work", &words(&c.work));
    }
    for (i, a) in fs.args.iter().enumerate() {
        if matches!(a, ArgSpec::Buf(_) | ArgSpec::Custom(_)) {
            s.sec(&format!("arg{i}"), &words(&c.bufs[i]));
        }
    }
    for (k, (n, _)) in fs.regions.iter().enumerate() {
        s.sec(&format!("region:{n}"), &words(&c.regions[k]));
    }
    for (k, g) in c.globals.iter().enumerate() {
        s.sec(&format!("global{k}"), &words(g));
    }
    s.sec("stray", &[0]);
    let mut log = vec![c.env.log.len() as u32];
    log.extend(&c.env.log);
    s.sec("stublog", &log);
    if !fs.envs.is_empty() {
        let flat = c.ext.env_words(&fs.envs);
        let mut v = vec![flat.len() as u32];
        v.extend(flat);
        s.sec("env", &v);
    }
    (format!("{:016x}", s.h), s.keep)
}

/// What to run and what to compare it with.
pub struct Suite<'a> {
    pub area: &'a str,
    pub spec: &'a str,
    /// Functions to run (the area's `PORTED`).
    pub ported: &'a [u32],
    /// `(address, hash)` rows written by the check script after a full ROM match.
    pub recorded: &'a [(u32, &'a str)],
    /// Prefix of the harness env vars (`FNDIFF` by default; the server-flow kit keeps `SVFLOW`).
    pub env_prefix: &'a str,
    /// Check script named in failure messages.
    pub tool: &'a str,
}

impl<'a> Suite<'a> {
    pub fn new(
        area: &'a str,
        spec: &'a str,
        ported: &'a [u32],
        recorded: &'a [(u32, &'a str)],
    ) -> Self {
        Suite {
            area,
            spec,
            ported,
            recorded,
            env_prefix: "FNDIFF",
            tool: "tools/check-fn.sh <crate>",
        }
    }
}

fn write_json(path: &str, area: &str, total: &str, funcs: &[(u32, String, Vec<String>)]) {
    let mut out = format!("{{\"module\": \"{area}\", \"hash\": \"{total}\", \"funcs\": {{");
    for (i, (addr, h, seeds)) in funcs.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        let s: Vec<String> = seeds.iter().map(|x| format!("\"{x}\"")).collect();
        out.push_str(&format!(
            "\"{addr:08x}\": {{\"hash\": \"{h}\", \"n_inputs\": {}, \"seeds\": [{}]}}",
            seeds.len(),
            s.join(", ")
        ));
    }
    out.push_str("}}\n");
    std::fs::write(path, out).unwrap();
}

/// Runs every function in `suite.ported` over its spec inputs through `call` (which maps the
/// address to the port and returns its result), and checks the per-function hashes against
/// `suite.recorded`.
///
/// Harness env vars (`<P>` = `suite.env_prefix`; used by tools/check-fn.sh):
/// - `<P>_JSON=<path>`: write the per-function and per-input hashes there and skip the
///   comparison with `recorded`.
/// - `<P>_ONLY=<hex addr>`: run only that function.
/// - `<P>_DETAIL=<hex addr>:<seed>` + `<P>_DETAIL_OUT=<path>`: write that input's
///   section-labelled word stream (`[["work", 1], ...]`) and stop.
pub fn run_suite<X: Ext>(suite: &Suite<'_>, call: &dyn Fn(u32, &mut Case<X>) -> u64) {
    let area = suite.area;
    let var = |k: &str| std::env::var(format!("{}_{k}", suite.env_prefix)).ok();
    let spec = parse_spec(suite.spec);
    let only: Option<u32> =
        var("ONLY").map(|v| u32::from_str_radix(v.trim_start_matches("0x"), 16).unwrap());
    let detail: Option<(u32, u32)> = var("DETAIL").map(|v| {
        let (a, s) = v.split_once(':').unwrap();
        (
            u32::from_str_radix(a.trim_start_matches("0x"), 16).unwrap(),
            s.parse().unwrap(),
        )
    });
    let mut total = FNV_OFFSET;
    let mut funcs: Vec<(u32, String, Vec<String>)> = Vec::new();
    for &addr in suite.ported {
        if only.is_some_and(|a| a != addr) || detail.is_some_and(|(a, _)| a != addr) {
            continue;
        }
        let fs = spec.iter().find(|f| f.addr == addr).unwrap_or_else(|| {
            panic!("{addr:#010x} is PORTED but has no spec in {area}: request one")
        });
        let mut seeds = Vec::new();
        for seed in 0..fs.inputs {
            let want = detail == Some((addr, seed));
            let (h, st) = run_input(fs, seed, call, want);
            if let (true, Some(st)) = (want, st) {
                let items: Vec<String> =
                    st.iter().map(|(n, w)| format!("[\"{n}\", {w}]")).collect();
                std::fs::write(
                    var("DETAIL_OUT").unwrap(),
                    format!("[{}]\n", items.join(", ")),
                )
                .unwrap();
                return;
            }
            seeds.push(h);
        }
        let mut fh = FNV_OFFSET;
        for s in &seeds {
            let v = u64::from_str_radix(s, 16).unwrap();
            fh = fold(fold(fh, v as u32), (v >> 32) as u32);
        }
        total = fold(total, addr);
        for s in &seeds {
            let v = u64::from_str_radix(s, 16).unwrap();
            total = fold(fold(total, v as u32), (v >> 32) as u32);
        }
        funcs.push((addr, format!("{fh:016x}"), seeds));
    }
    if let Some(path) = var("JSON") {
        write_json(&path, area, &format!("{total:016x}"), &funcs);
        return;
    }
    if only.is_some() {
        for (a, h, _) in &funcs {
            println!("fndiff {a:08x} {h}");
        }
        return;
    }
    for (addr, h, _) in &funcs {
        let rec = suite
            .recorded
            .iter()
            .find(|(a, _)| a == addr)
            .unwrap_or_else(|| {
                panic!(
                    "{addr:#010x} has no recorded hash: run {} {area}",
                    suite.tool
                )
            });
        assert_eq!(
            h, rec.1,
            "{area} {addr:#010x}: hash differs from the ROM-verified record"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn region_arg_and_fix_round_trip() {
        let spec = "fn 02000000 ret=u32\n  region r 8\n  arg u8 const 3\n  arg region r 4\n  fix region:r 0 u32 const 0x11223344\n  nowork\n";
        let fs = &parse_spec(spec)[0];
        let (h1, st) = run_input::<()>(
            fs,
            0,
            &|_, c| {
                assert_eq!(c.vals[0], 3);
                assert_eq!(c.vals[1], REGION_BASE + 4);
                assert_eq!(&c.region("r")[..4], &[0x44, 0x33, 0x22, 0x11]);
                c.region_mut("r")[7] = 9;
                7
            },
            true,
        );
        let names: Vec<String> = st.unwrap().into_iter().map(|(n, _)| n).collect();
        assert!(names.contains(&"region:r".to_string()));
        assert!(!names.contains(&"work".to_string()));
        let (h2, _) = run_input::<()>(fs, 0, &|_, _| 7, false);
        assert_ne!(h1, h2, "a region write must change the hash");
    }

    #[test]
    fn ramp_is_a_running_sum_of_draws() {
        let spec = "fn 02000000 ret=u32\n  region r 16\n  arg region r\n  nowork\n  ramp region:r 0 4 u32 range 1 5\n";
        let fs = &parse_spec(spec)[0];
        run_input::<()>(
            fs,
            0,
            &|_, c| {
                let v: Vec<u32> = c
                    .region("r")
                    .chunks(4)
                    .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
                    .collect();
                assert!(v[0] >= 1 && v[0] <= 5);
                assert!(v.windows(2).all(|w| w[1] > w[0] && w[1] - w[0] <= 5));
                0
            },
            false,
        );
    }

    #[test]
    fn sum16_and_crypt_fixes() {
        // checksum of the plain bytes, then encrypt with it as seed (the poke_tool record recipe)
        let spec = "fn 02000000 ret=u32\n  region r 16\n  arg region r\n  nowork\n  fix region:r 0 u32 const 0x00020001\n  fix region:r 4 u32 const 0x00040003\n  fix region:r 6 u16 sum16 0 6\n  fix region:r 8 u16 const 0\n  crypt region:r 0 6 u16@6 skipif=u16@8&1\n";
        let fs = &parse_spec(spec)[0];
        run_input::<()>(
            fs,
            0,
            &|_, c| {
                let r = c.region("r");
                // sum16 over [0, 6) = 1 + 2 + 3 = 6 (written at 6, after the sum was taken)
                assert_eq!(&r[6..8], &[6, 0]);
                let mut state = 6u32;
                for i in 0..3 {
                    state = state.wrapping_mul(0x41C6_4E6D).wrapping_add(0x6073);
                    let plain = [1u16, 2, 3][i];
                    let got = u16::from_le_bytes([r[2 * i], r[2 * i + 1]]);
                    assert_eq!(got, plain ^ (state >> 16) as u16);
                }
                0
            },
            false,
        );
        // skipif: the field at 8 is set before the crypt line -> the bytes stay plain
        let spec2 = spec.replace("u16 const 0\n", "u16 const 1\n");
        let fs2 = &parse_spec(&spec2)[0];
        run_input::<()>(
            fs2,
            0,
            &|_, c| {
                assert_eq!(&c.region("r")[..6], &[1, 0, 2, 0, 3, 0]);
                0
            },
            false,
        );
    }
}
