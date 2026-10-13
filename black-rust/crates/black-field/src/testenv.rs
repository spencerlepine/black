//! Test-only [`FieldEnv`] for the F2a areas: the spec's stubs answer every [`FieldEnv::call`], and the
//! case's work image, buffers, regions and globals form one memory window set at the addresses the
//! emulator uses (work 0x02E00000, argument i 0x02E10000 + i * 0x10000, region k 0x02F10000 + k * 0x10000,
//! globals at their spec addresses).

use crate::hooks::{CArg, FieldEnv, Mem, MemEnv};
use black_fndiff::{Arg, Case, StubEnv};

/// Address of the spec's `work` arena.
pub const WORK: u32 = 0x02E0_0000;
/// Address of pointer argument `i` (`arg buf`).
pub fn buf(i: u32) -> u32 {
    0x02E1_0000 + 0x1_0000 * i
}

/// The stubs plus the memory of one case.
pub struct Env<'a> {
    pub stubs: &'a mut StubEnv,
    pub mem: Mem,
    /// Stub addresses of the function's spec block: a thunk that is stubbed itself (F4a: `02008DF0` in
    /// `event_battle_head`) is not mapped to its target.
    pub stubbed: Vec<u32>,
}

impl MemEnv for Env<'_> {
    fn mem(&mut self) -> &mut Mem {
        &mut self.mem
    }
}

/// Thumb thunks (`ldr r3, =target; [movs rN, #k;] bx r3`) that the ports call by their own address: the
/// spec stubs their target (the thunk runs as real code on the ROM side and lands on the stub). The third
/// field is the constant arguments the thunk appends (when the target's stub reads them).
const THUNKS: &[(u32, u32, &[u32])] = &[
    (0x0200_84F8, 0x0201_F41C, &[]),  // -> 0201F41C (string copy)
    (0x0200_8854, 0x0200_71CC, &[]), // -> 020071CC(x, 0x1C) (save block 0x1C; the stub logs x only)
    (0x0200_D418, 0x0200_71CC, &[]), // -> 020071CC(x, 0x38)
    (0x0200_ED48, 0x0200_71CC, &[]), // -> 020071CC(x, 0x44)
    (0x0200_8DF0, 0x0200_8DAC, &[1]), // -> 02008DAC(x, y, 1)
    (0x0216_26A8, 0x0216_2B68, &[]),
    (0x0216_26B0, 0x0216_2BCC, &[]),
    (0x0218_AECC, 0x0216_26B8, &[]), // fieldmap_event's veneer
    (0x021A_A8F8, 0x021A_AF54, &[]),
    (0x0200_BE48, 0x0200_71CC, &[]), // -> 020071CC(x, 0x35)
    (0x0201_27C4, 0x0201_27D0, &[1]),
    (0x0200_BDB0, 0x0200_71CC, &[]), // -> 020071CC(x, 0x32)
    (0x0201_40B4, 0x0203_07B0, &[]), // Heap_Free veneers
    (0x021C_31FC, 0x0203_07B0, &[]),
    (0x021A_0138, 0x0203_07B0, &[]),
    (0x021D_31A4, 0x0203_07B0, &[]),
    (0x0201_DD60, 0x0201_DD68, &[]),
    (0x021A_BE04, 0x021A_BFDC, &[]),
    (0x021C_7FAC, 0x021C_8000, &[]),
    (0x021A_1BDC, 0x021A_1C34, &[]),
];

impl FieldEnv for Env<'_> {
    fn call(&mut self, addr: u32, args: &mut [CArg<'_>]) -> u32 {
        if let Some(&(_, target, extra)) = THUNKS
            .iter()
            .find(|t| t.0 == addr && !self.stubbed.contains(&addr))
        {
            let mut v: Vec<CArg<'_>> = args
                .iter_mut()
                .map(|a| std::mem::replace(a, CArg::V(0)))
                .collect();
            v.extend(extra.iter().map(|&x| CArg::V(x)));
            let r = self.call(target, &mut v);
            for (a, b) in args.iter_mut().zip(v) {
                *a = b;
            }
            return r;
        }
        // copy the pointed-to bytes out of the windows, run the stub, write the outs back
        let mut ins: Vec<Vec<u8>> = Vec::new();
        let mut outs: Vec<Vec<u8>> = Vec::new();
        for a in args.iter() {
            match a {
                CArg::In(p, n) => {
                    ins.push((0..*n).map(|i| self.mem.r8(p.wrapping_add(i))).collect())
                }
                CArg::Out(p, n) => {
                    outs.push((0..*n).map(|i| self.mem.r8(p.wrapping_add(i))).collect())
                }
                _ => {}
            }
        }
        let r;
        {
            let mut ii = ins.iter();
            let mut oi = outs.iter_mut();
            let mut sa: Vec<Arg<'_>> = Vec::new();
            for a in args.iter_mut() {
                if let CArg::X(_) = a {
                    continue;
                }
                sa.push(match a {
                    CArg::V(v) => Arg::V(*v),
                    CArg::P(_) => Arg::P,
                    CArg::In(..) => Arg::In(ii.next().unwrap()),
                    CArg::Out(..) => Arg::Out(oi.next().unwrap()),
                    CArg::InBuf(b) => Arg::In(b),
                    CArg::OutBuf(b) => Arg::Out(b),
                    CArg::X(_) => unreachable!(),
                });
            }
            r = self.stubs.call(addr, &mut sa);
        }
        let mut oi = outs.into_iter();
        for a in args.iter() {
            if let CArg::Out(p, _) = a {
                self.mem.write(*p, &oi.next().unwrap());
            }
        }
        r
    }
}

/// Addresses of the `global` lines (`key = "global "`) or `stub` lines (`"stub "`) of function `addr` in `spec`,
/// in declaration order.
fn lines_of(spec: &str, addr: u32, key: &str) -> Vec<u32> {
    let mut inside = false;
    let mut out = Vec::new();
    for l in spec.lines() {
        let t = l.trim();
        if let Some(rest) = t.strip_prefix("fn ") {
            let a = rest.split_whitespace().next().unwrap();
            inside = u32::from_str_radix(a, 16).ok() == Some(addr);
            continue;
        }
        if inside {
            if let Some(rest) = t.strip_prefix(key) {
                let a = rest.split_whitespace().next().unwrap();
                out.push(u32::from_str_radix(a, 16).unwrap());
            }
        }
    }
    out
}

/// Runs `f` on the case's memory (`f` gets the env and the argument values); moves the memory back
/// into the case afterwards, also when a `noreturn` stub unwinds (the payload is passed on).
pub fn run(c: &mut Case, spec: &str, addr: u32, f: impl FnOnce(&mut Env, &[u32]) -> u64) -> u64 {
    let globals = lines_of(spec, addr, "global ");
    let stubbed = lines_of(spec, addr, "stub ");
    let vals = c.vals.clone();
    let mut mem = Mem::new(WORK, std::mem::take(&mut c.work));
    let mut keep = Vec::new();
    for (i, b) in c.bufs.iter_mut().enumerate() {
        if !b.is_empty() {
            mem.add_window(vals[i], std::mem::take(b));
            keep.push((0, i));
        }
    }
    for (k, r) in c.regions.iter_mut().enumerate() {
        mem.add_window(0x02F1_0000 + 0x1_0000 * k as u32, std::mem::take(r));
        keep.push((1, k));
    }
    for (k, g) in c.globals.iter_mut().enumerate() {
        mem.add_window(globals[k], std::mem::take(g));
        keep.push((2, k));
    }
    let mut env = Env {
        stubs: &mut c.env,
        mem,
        stubbed,
    };
    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(&mut env, &vals)));
    let mem = std::mem::take(&mut env.mem);
    c.work = mem.bytes;
    for ((kind, i), (_, bytes)) in keep.into_iter().zip(mem.extra) {
        match kind {
            0 => c.bufs[i] = bytes,
            1 => c.regions[i] = bytes,
            _ => c.globals[i] = bytes,
        }
    }
    // a `noreturn` stub's `Halt` (or a real panic) goes on to the suite, with the state restored
    res.unwrap_or_else(|e| std::panic::resume_unwind(e))
}

/// A pointer result as the spec's `ofs:work` hashes it (offset into the work image, 0 = null).
pub fn ofs_work(p: u32) -> u64 {
    if p == 0 {
        0
    } else {
        p.wrapping_sub(WORK) as u64
    }
}

/// A pointer result as the spec's `ofs:argK` hashes it (offset into argument `k`, 0 = null).
pub fn ofs_arg(p: u32, k: u32) -> u64 {
    if p == 0 {
        0
    } else {
        p.wrapping_sub(buf(k)) as u64
    }
}

/// Overlay 21 of the ROM build (`$BLACK_ROM_BUILD/OVY_21.sbin`, loaded at `0x02187F00`) as a word reader, or `None` (skip)
/// when it is missing and `REQUIRE_ROM` is not set (F4b-port1: the acmd ROM tables are compared with it).
pub fn overlay21() -> Option<impl Fn(u32) -> u32> {
    let got = std::env::var("BLACK_ROM_BUILD")
        .ok()
        .and_then(|b| std::fs::read(format!("{b}/OVY_21.sbin")).ok());
    if got.is_none() && std::env::var("REQUIRE_ROM").is_ok_and(|v| !v.is_empty() && v != "0") {
        panic!("REQUIRE_ROM is set but $BLACK_ROM_BUILD/OVY_21.sbin is missing");
    }
    let ovy = got?;
    Some(move |a: u32| {
        let o = (a - 0x0218_7F00) as usize;
        u32::from_le_bytes(ovy[o..o + 4].try_into().unwrap())
    })
}
