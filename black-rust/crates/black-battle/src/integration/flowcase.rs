//! The flow areas (cmd_queue, core1-6, hem, handex, api) through their differential tests' own
//! `call` tables: a [`Case`] is built from the battle's state as the area's spec describes the
//! function's inputs (argument kinds, `region`s found through `fix work N u32 addrof region:X`,
//! `global`s), the table runs the port exactly as the differential test does, and the case's
//! images are written back. The argument mapping is therefore the verified one.
//!
//! The case's copies of objects (regions, buffers, globals) are registered as aliases of their
//! addresses while the port runs, so a port that lends them (`call_l`) lends the right objects,
//! and their blocks count as held.

use super::{Bufs, Env};
use crate::server_flow::env::{Arg, Lent};
use crate::server_flow::testkit::Case;
use crate::server_flow::SvFlowWork;
use std::collections::HashMap;

/// An argument kind of the spec.
#[derive(Clone, Debug)]
pub enum ArgKind {
    /// `arg work [OFS]`: the work's address (+ OFS).
    Work(u32),
    /// `arg buf N`.
    Buf(u32),
    /// `arg bpp`.
    Bpp,
    /// any scalar.
    Val,
}

/// One `fn` block.
#[derive(Clone, Debug, Default)]
pub struct FnSpec {
    pub ret: String,
    pub args: Vec<ArgKind>,
    /// `(size, work offset of its pointer)`.
    pub regions: Vec<(u32, Option<u32>)>,
    pub globals: Vec<(u32, u32)>,
}

/// Parses an svflow spec into its `fn` blocks.
pub fn parse(text: &str) -> HashMap<u32, FnSpec> {
    let mut out = HashMap::new();
    let mut cur: Option<(u32, FnSpec)> = None;
    let mut names: Vec<String> = Vec::new();
    for line in text.lines() {
        let l = line.split('#').next().unwrap().trim();
        let w: Vec<&str> = l.split_whitespace().collect();
        if w.is_empty() {
            continue;
        }
        let num = |s: &str| -> u32 {
            let s = s.trim_start_matches("0x").trim_start_matches("0X");
            u32::from_str_radix(s, 16).unwrap_or_else(|_| s.parse().unwrap())
        };
        match w[0] {
            "fn" => {
                if let Some((a, f)) = cur.take() {
                    out.insert(a, f);
                }
                let ret = w
                    .iter()
                    .find_map(|x| x.strip_prefix("ret="))
                    .unwrap_or("none")
                    .to_string();
                cur = Some((
                    u32::from_str_radix(w[1], 16).unwrap(),
                    FnSpec {
                        ret,
                        ..FnSpec::default()
                    },
                ));
                names.clear();
            }
            "arg" if cur.is_some() => {
                let f = &mut cur.as_mut().unwrap().1;
                f.args.push(match w[1] {
                    "work" => ArgKind::Work(w.get(2).map_or(0, |x| num(x))),
                    "buf" => ArgKind::Buf(num(w[2])),
                    "bpp" => ArgKind::Bpp,
                    _ => ArgKind::Val,
                });
            }
            "region" if cur.is_some() => {
                let f = &mut cur.as_mut().unwrap().1;
                names.push(w[1].to_string());
                f.regions.push((num(w[2]), None));
            }
            "global" if cur.is_some() => {
                let f = &mut cur.as_mut().unwrap().1;
                f.globals.push((num(w[1]), num(w[2])));
            }
            "fix" if cur.is_some() && w[1] == "work" && w.len() >= 6 && w[4] == "addrof" => {
                if let Some(r) = w[5].strip_prefix("region:") {
                    let f = &mut cur.as_mut().unwrap().1;
                    if let Some(k) = names.iter().position(|n| n == r) {
                        f.regions[k].1 = Some(num(w[2]));
                    }
                }
            }
            _ => {}
        }
    }
    if let Some((a, f)) = cur.take() {
        out.insert(a, f);
    }
    out
}

/// The address the work lives at in the integration (the differential tests' fixed work
/// address, which their `call` tables use to turn work pointers into offsets).
pub const WORK_ADDR: u32 = 0x02E0_0000;

/// The area of a flow address, with its spec text and `call` table.
fn area(addr: u32) -> Option<(&'static str, &'static str)> {
    Some(match addr {
        0x021C_C8D8..=0x021C_D6EB => (
            "cmd_queue",
            include_str!("../../../../tools/rom-diff/svflow_specs/cmd_queue.spec"),
        ),
        0x021B_B968..=0x021B_D1CF => (
            "core1",
            include_str!("../../../../tools/rom-diff/svflow_specs/core1.spec"),
        ),
        0x021B_D1D0..=0x021B_F323 => (
            "core2",
            include_str!("../../../../tools/rom-diff/svflow_specs/core2.spec"),
        ),
        0x021B_F324..=0x021C_0F23 => (
            "core3",
            include_str!("../../../../tools/rom-diff/svflow_specs/core3.spec"),
        ),
        0x021C_0F24..=0x021C_4837 => (
            "core4",
            include_str!("../../../../tools/rom-diff/svflow_specs/core4.spec"),
        ),
        0x021C_4838..=0x021C_65A7 => (
            "core5",
            include_str!("../../../../tools/rom-diff/svflow_specs/core5.spec"),
        ),
        0x021C_65A8..=0x021C_7CBB => (
            "core6",
            include_str!("../../../../tools/rom-diff/svflow_specs/core6.spec"),
        ),
        0x021C_7CBC..=0x021C_880F => (
            "api",
            include_str!("../../../../tools/rom-diff/svflow_specs/api.spec"),
        ),
        0x021C_8810..=0x021C_8917 => (
            "hem",
            include_str!("../../../../tools/rom-diff/svflow_specs/hem.spec"),
        ),
        0x021C_8918..=0x021C_A8CB => (
            "handex",
            include_str!("../../../../tools/rom-diff/svflow_specs/handex.spec"),
        ),
        0x021C_A8CC..=0x021C_C8D7 => (
            "events",
            include_str!("../../../../tools/rom-diff/svflow_specs/events.spec"),
        ),
        _ => return None,
    })
}

/// The addresses whose arm in area `name`'s test `call` table does not touch `c.work` (nor
/// hands `c` to a helper): parsed from the table's source.
fn no_work_arms(name: &str) -> &'static std::collections::BTreeSet<u32> {
    use std::collections::{BTreeMap, BTreeSet};
    use std::sync::OnceLock;
    static CACHE: OnceLock<BTreeMap<&'static str, BTreeSet<u32>>> = OnceLock::new();
    let all = CACHE.get_or_init(|| {
        let srcs: [(&str, &str); 11] = [
            (
                "cmd_queue",
                include_str!("../server_flow/cmd_queue/tests.rs"),
            ),
            ("core1", include_str!("../server_flow/core1/tests.rs")),
            ("core2", include_str!("../server_flow/core2/tests.rs")),
            ("core3", include_str!("../server_flow/core3/tests.rs")),
            ("core4", include_str!("../server_flow/core4/tests.rs")),
            ("core5", include_str!("../server_flow/core5/tests.rs")),
            ("core6", include_str!("../server_flow/core6/tests.rs")),
            ("api", include_str!("../server_flow/api/tests.rs")),
            ("hem", include_str!("../server_flow/hem/tests.rs")),
            ("handex", include_str!("../server_flow/handex/tests.rs")),
            ("events", include_str!("../server_flow/events/tests.rs")),
        ];
        srcs.iter()
            .map(|&(n, src)| (n, parse_no_work(src)))
            .collect()
    });
    &all[name]
}

fn parse_no_work(src: &str) -> std::collections::BTreeSet<u32> {
    let mut out = std::collections::BTreeSet::new();
    let Some(start) = src.find("fn call") else {
        return out;
    };
    let body = &src[start..];
    // arm heads: lines starting with `0x...` (alternatives with `|`) followed by `=>`
    let mut heads: Vec<(usize, Vec<u32>)> = Vec::new();
    let mut pos = 0;
    for line in body.split_inclusive('\n') {
        let t = line.trim_start();
        if t.starts_with("0x") {
            if let Some(k) = t.find("=>") {
                let addrs: Vec<u32> = t[..k]
                    .split('|')
                    .filter_map(|a| {
                        u32::from_str_radix(&a.trim().trim_start_matches("0x").replace('_', ""), 16)
                            .ok()
                    })
                    .collect();
                if !addrs.is_empty() {
                    heads.push((pos, addrs));
                }
            }
        }
        pos += line.len();
    }
    // `let w = &mut c.work;` before the match: `w` in an arm is the work
    let pre = &body[..heads.first().map_or(0, |h| h.0)];
    let w_alias = pre.contains("let w = &mut c.work") || pre.contains("let w = &c.work");
    let word_w = |a: &str| {
        let b = a.as_bytes();
        (0..b.len()).any(|i| {
            b[i] == b'w'
                && (i == 0
                    || !(b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'_' || b[i - 1] == b'.'))
                && (i + 1 == b.len() || !(b[i + 1].is_ascii_alphanumeric() || b[i + 1] == b'_'))
        })
    };
    for (i, (p, addrs)) in heads.iter().enumerate() {
        let end = heads.get(i + 1).map_or(body.len(), |h| h.0);
        // helpers that take the case but only read its buffers
        let arm = body[*p..end].replace("mon(c, ", "mon_of_buf(");
        let arm = arm.as_str();
        let uses = (w_alias && word_w(arm))
            || arm.contains("c.work")
            || arm.contains("(c,")
            || arm.contains("(c)")
            || arm.contains(", c)")
            || arm.contains(", c,")
            || arm.contains("!(");
        if !uses {
            out.extend(addrs.iter().copied());
        }
    }
    out
}

/// Runs the `call` table of area `name`.
fn run(name: &str, addr: u32, case: &mut Case<&mut Env>) -> u64 {
    use crate::server_flow as sf;
    match name {
        "cmd_queue" => sf::cmd_queue::tests::call(addr, case),
        "core1" => sf::core1::tests::call(addr, case),
        "core2" => sf::core2::tests::call(addr, case),
        "core3" => sf::core3::tests::call(addr, case),
        "core4" => sf::core4::tests::call(addr, case),
        "core5" => sf::core5::tests::call(addr, case),
        "core6" => sf::core6::tests::call(addr, case),
        "api" => sf::api::tests::call(addr, case),
        "hem" => sf::hem::tests::call(addr, case),
        "handex" => sf::handex::tests::call(addr, case),
        "events" => sf::events::tests::call(addr, case),
        _ => unreachable!(),
    }
}

/// Where a case image came from, to store it back.
enum Src {
    /// An address in the object table.
    Addr(u32),
    /// A battle Pokemon.
    Mon(u32),
    /// A part of the flow work the case took (offset): merged back into the work image.
    InWork(usize),
    /// A part of the flow work the case did not take (address, initial bytes): not held; the
    /// bytes the port changed are stored back (a callee may change the work meanwhile).
    WorkPart(u32, Vec<u8>),
    /// Nothing (a dummy image).
    None,
    /// The bytes of argument slice `i` (a caller-local buffer: `In` / `Out` / `PBuf` / `PHost`),
    /// copied into a temporary heap block at `.1` for the call so that a callee the port passes
    /// the pointer on to finds it (B18: 021c7e14 -> 021c1e74 -> 021c728c); initial bytes `.2`.
    Local(usize, u32, Vec<u8>),
}

pub fn call(env: &mut Env, lent: &mut Lent<'_>, addr: u32, args: &mut [Arg<'_>]) -> Option<u32> {
    let (name, text) = area(addr)?;
    if !env.specs.contains_key(name) {
        env.specs.insert(name, parse(text));
    }
    let Some(spec) = env.specs[name].get(&addr).cloned() else {
        // a veneer or an address without a spec block: not a port
        return None;
    };
    let ported = crate::integration::dispatch::is_ported(addr);
    if !ported {
        return None;
    }
    // ---- the case ----
    let wa = env.waddr;
    // the frame depth the port's callees report as their caller's (`Env::dispatch`)
    let depth = env.stack.len();
    env.work_sync.remove(&depth);
    // a port whose `call` arm does not use the case work (it gets the work address only) runs
    // without taking it, so its callees can (the original passes the pointer on)
    let takes_work = spec.args.iter().any(|a| matches!(a, ArgKind::Work(_)))
        && !no_work_arms(name).contains(&addr);
    let mut base = Vec::new();
    let work = if takes_work {
        let v = env.objs.borrow_mut().take(wa);
        base = v.clone();
        SvFlowWork {
            mem: v.into_boxed_slice().try_into().expect("work size"),
        }
    } else {
        SvFlowWork::new()
    };
    let mut vals = Vec::new();
    let mut bufs = Vec::new();
    let mut srcs = Vec::new();
    let mut inwork_bufs: Vec<(usize, usize, Vec<u8>)> = Vec::new();
    // the initial images of the buffers read from memory: only the bytes the port changed are
    // stored back (buffer arguments may overlap, e.g. the arrays of 0x021F3B20)
    let mut addr_init: std::collections::HashMap<usize, Vec<u8>> = std::collections::HashMap::new();
    for (i, k) in spec.args.iter().enumerate() {
        let a = args.get(i);
        let value = match a {
            Some(Arg::V(x)) | Some(Arg::PAt(x)) => Some(*x),
            Some(Arg::P) => Some(wa),
            _ => None,
        };
        match k {
            ArgKind::Work(o) => {
                vals.push(wa + o);
                bufs.push(Vec::new());
                srcs.push(Src::None);
            }
            ArgKind::Val => {
                vals.push(value.unwrap_or(0));
                bufs.push(Vec::new());
                srcs.push(Src::None);
            }
            ArgKind::Bpp if a.is_none() => {
                // a `bpp` image the caller does not pass (the events specs' party image the `mons`
                // lookups of the test arms read): every battle Pokemon, at their addresses
                let lo = *env.mons.keys().next().expect("no battle Pokemon");
                let hi = *env.mons.keys().last().unwrap() + crate::battle_mon::BPP_SIZE as u32;
                let mut img = vec![0u8; (hi - lo) as usize];
                for (&ma, b) in &env.mons {
                    let o = (ma - lo) as usize;
                    img[o..o + crate::battle_mon::BPP_SIZE].copy_from_slice(&b.raw[..]);
                }
                vals.push(lo);
                bufs.push(img);
                srcs.push(Src::None);
            }
            ArgKind::Bpp => {
                let m = value.expect("bpp argument by value");
                vals.push(m);
                match env.mons.get(&m).map(|b| b.raw.to_vec()) {
                    Some(b) => {
                        bufs.push(b);
                        srcs.push(Src::Mon(m));
                    }
                    None => {
                        // the spec's `bpp` is a guess for a value the caller passes on (e.g. the
                        // stack words of 021ca7f4 that hold a slot index): a zero image
                        env.note(format!(
                            "{addr:#x}: bpp argument {i} = {m:#x} is not a battle Pokemon"
                        ));
                        bufs.push(vec![0; crate::battle_mon::BPP_SIZE]);
                        srcs.push(Src::None);
                    }
                }
            }
            ArgKind::Buf(_) if matches!(a, Some(Arg::P)) => panic!(
                "B16 gap: {addr:#x} gets a pass-through pointer for its buffer argument {i} ({})",
                env.backtrace()
            ),
            ArgKind::Buf(n) => match value {
                Some(p) if takes_work && p >= wa && p < wa + SvFlowWork::SIZE as u32 => {
                    let o = (p - wa) as usize;
                    let k = (*n as usize).min(SvFlowWork::SIZE - o);
                    let mut b = work.mem[o..o + k].to_vec();
                    b.resize(*n as usize, 0);
                    vals.push(p);
                    inwork_bufs.push((bufs.len(), o, b.clone()));
                    bufs.push(b);
                    srcs.push(Src::InWork(o));
                }
                Some(p)
                    if !takes_work && wa != 0 && p >= wa && p < wa + SvFlowWork::SIZE as u32 =>
                {
                    vals.push(p);
                    let b = env.read_vec(lent, p, *n as usize);
                    srcs.push(Src::WorkPart(p, b.clone()));
                    bufs.push(b);
                }
                Some(p) if p != 0 => {
                    vals.push(p);
                    bufs.push(env.read_vec(lent, p, *n as usize));
                    srcs.push(Src::Addr(p));
                    addr_init.insert(bufs.len() - 1, bufs[bufs.len() - 1].clone());
                }
                Some(_) => {
                    vals.push(0);
                    bufs.push(vec![0; *n as usize]);
                    srcs.push(Src::None);
                }
                None => {
                    let b = Bufs::new(args).get(env, lent, i, *n as usize);
                    let la = env
                        .objs
                        .borrow_mut()
                        .alloc(*n, "caller-local buffer (flow case)");
                    env.write_bytes(lent, la, &b);
                    vals.push(la);
                    srcs.push(Src::Local(i, la, b.clone()));
                    bufs.push(b);
                }
            },
        }
    }
    let mut regions = Vec::new();
    let mut raddrs = Vec::new();
    // regions that lie in the flow work itself (`*(work + 0x1AB0)` = work + 0xD04, ...): a copy
    // of the work image, merged back into it after the call
    let mut inwork: Vec<(usize, usize, Vec<u8>)> = Vec::new();
    let mut rwork: Vec<(usize, u32, Vec<u8>)> = Vec::new();
    for &(size, wofs) in &spec.regions {
        let p = match wofs {
            Some(o) if takes_work => work.u32_at(o as usize),
            Some(o) => env.r32(wa + o),
            None => 0,
        };
        if takes_work && p >= wa && p < wa + SvFlowWork::SIZE as u32 {
            let o = (p - wa) as usize;
            let n = (size as usize).min(SvFlowWork::SIZE - o);
            let mut r = work.mem[o..o + n].to_vec();
            r.resize(size as usize, 0);
            inwork.push((regions.len(), o, r.clone()));
            regions.push(r);
            raddrs.push(0);
            continue;
        }
        if !takes_work && wa != 0 && p >= wa && p < wa + SvFlowWork::SIZE as u32 {
            // in the work the case did not take: not held, the changed bytes are stored back
            let r = env.read_vec(lent, p, size as usize);
            rwork.push((regions.len(), p, r.clone()));
            regions.push(r);
            raddrs.push(0);
            continue;
        }
        if p == 0 || env.objs.borrow().find_block(p, 1).is_none() {
            regions.push(vec![0; size as usize]);
            raddrs.push(0);
        } else {
            regions.push(env.read_vec(lent, p, size as usize));
            raddrs.push(p);
        }
    }
    let mut globals = Vec::new();
    for &(g, size) in &spec.globals {
        globals.push(env.read_vec(lent, g, size as usize));
    }
    // aliases + holds
    let mark = env.aliases.len();
    let mut held_blocks = Vec::new();
    {
        let mut o = env.objs.borrow_mut();
        let mut reg = |ptr: usize, len: usize, a: u32, o: &mut super::objects::Objects| {
            if a == 0 || len == 0 {
                return;
            }
            env.aliases.push((ptr, len, a));
            if let Some(base) = o.base_of(a) {
                o.blocks.get_mut(&base).unwrap().alias_holds += 1;
                held_blocks.push(base);
            }
        };
        for (k, s) in srcs.iter().enumerate() {
            if let Src::Addr(a) = s {
                reg(bufs[k].as_ptr() as usize, bufs[k].len(), *a, &mut o);
            }
        }
        for (k, &a) in raddrs.iter().enumerate() {
            reg(regions[k].as_ptr() as usize, regions[k].len(), a, &mut o);
        }
        for (k, &(g, _)) in spec.globals.iter().enumerate() {
            reg(globals[k].as_ptr() as usize, globals[k].len(), g, &mut o);
        }
    }
    // work parts (not held): aliases only, for `PHost` pointers into them
    for (k, s) in srcs.iter().enumerate() {
        if let Src::WorkPart(a, _) | Src::Local(_, a, _) = s {
            env.aliases
                .push((bufs[k].as_ptr() as usize, bufs[k].len(), *a));
        }
    }
    for &(k, a, _) in &rwork {
        env.aliases
            .push((regions[k].as_ptr() as usize, regions[k].len(), a));
    }
    let mut case = Case {
        work,
        vals,
        bufs,
        globals,
        regions,
        env: &mut *env,
    };
    let r = run(name, addr, &mut case);
    let Case {
        work,
        vals,
        bufs,
        globals,
        regions,
        ..
    } = case;
    // ---- write back ----
    {
        let mut o = env.objs.borrow_mut();
        for base in held_blocks {
            o.blocks.get_mut(&base).unwrap().alias_holds -= 1;
        }
    }
    env.aliases.truncate(mark);
    if !takes_work && work.mem.iter().any(|&b| b != 0) {
        panic!("B16: {addr:#x} wrote the dummy work (its `call` arm uses the work after all)");
    }
    // the port freed the work (`021bbb48`, the flow's destructor)
    let takes_work = takes_work && env.objs.borrow().blocks.contains_key(&wa);
    if takes_work {
        // three-way merge: the bytes the port changed (ours vs the image it was given) over what
        // nested callees left in the table (a port that holds the work read-only does not see
        // their writes, see `Env::note`)
        let mut ours: Box<[u8]> = work.mem;
        for (k, o, init) in &inwork {
            for (i, (&now, &was)) in regions[*k].iter().zip(init.iter()).enumerate() {
                if now != was && o + i < ours.len() {
                    ours[o + i] = now;
                }
            }
        }
        for (k, o, init) in &inwork_bufs {
            for (i, (&now, &was)) in bufs[*k].iter().zip(init.iter()).enumerate() {
                if now != was && o + i < ours.len() {
                    ours[o + i] = now;
                }
            }
        }
        // B19: against the image the port's last writable lend handed back (`Env::work_sync`;
        // the port's callees see `depth` = this frame), else the one it was given
        let sync = env.work_sync.remove(&depth).unwrap_or(base);
        let mut o = env.objs.borrow_mut();
        let theirs = o.blocks[&wa].data.clone();
        let merged: Vec<u8> = (0..ours.len())
            .map(|i| {
                if ours[i] != sync[i] {
                    ours[i]
                } else {
                    theirs[i]
                }
            })
            .collect();
        o.put(wa, merged);
    }
    for (k, &a) in raddrs.iter().enumerate() {
        if a != 0 {
            env.write_bytes(lent, a, &regions[k]);
        }
    }
    for (k, a, init) in &rwork {
        let now = env.read_vec(lent, *a, init.len());
        let mut out = now.clone();
        let mut conflict = false;
        for i in 0..init.len() {
            if regions[*k][i] != init[i] {
                conflict |= now[i] != init[i];
                out[i] = regions[*k][i];
            }
        }
        if conflict {
            env.note(format!(
                "{addr:#x}: a work region and a callee both changed work+{:#x}",
                a - wa
            ));
        }
        env.write_bytes(lent, *a, &out);
    }
    for (k, &(g, _)) in spec.globals.iter().enumerate() {
        env.write_bytes(lent, g, &globals[k]);
    }
    for (k, s) in srcs.iter().enumerate() {
        match s {
            Src::Addr(a) => {
                let init = &addr_init[&k];
                let mut out = env.read_vec(lent, *a, init.len());
                for i in 0..out.len() {
                    if bufs[k][i] != init[i] {
                        out[i] = bufs[k][i];
                    }
                }
                env.write_bytes(lent, *a, &out);
            }
            Src::WorkPart(a, init) => {
                let now = env.read_vec(lent, *a, init.len());
                let mut out = now.clone();
                let mut conflict = false;
                for i in 0..init.len().min(bufs[k].len()) {
                    if bufs[k][i] != init[i] {
                        conflict |= now[i] != init[i];
                        out[i] = bufs[k][i];
                    }
                }
                if conflict {
                    env.note(format!(
                        "{addr:#x}: a work buffer and a callee both changed work+{:#x}",
                        a - wa
                    ));
                }
                env.write_bytes(lent, *a, &out);
            }
            Src::Local(i, la, init) => {
                // the bytes the port changed win; a callee's changes to the block are kept
                let mut out = env.read_vec(lent, *la, init.len());
                for j in 0..out.len().min(bufs[k].len()) {
                    if bufs[k][j] != init[j] {
                        out[j] = bufs[k][j];
                    }
                }
                Bufs::new(args).put(env, lent, *i, &out);
                env.objs.borrow_mut().free(*la);
            }
            Src::Mon(m) => {
                let b = env.mons.get_mut(m).unwrap();
                b.raw
                    .copy_from_slice(&bufs[k][..crate::battle_mon::BPP_SIZE]);
            }
            Src::None | Src::InWork(_) => {}
        }
    }
    // ---- the return value (the tables return offsets for `ret=ofs:..`) ----
    let r = r as u32;
    Some(match spec.ret.as_str() {
        "ofs:work" => {
            if r == 0 {
                0
            } else {
                wa + r
            }
        }
        k if k.starts_with("ofs:arg") => {
            let i: usize = k[7..].parse().unwrap();
            if r == 0 {
                0
            } else {
                vals[i] + r
            }
        }
        _ => r,
    })
}
