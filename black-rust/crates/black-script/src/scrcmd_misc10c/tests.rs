use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0216B970, "a6738f3ebaf313f2"),
    (0x0216BB70, "3f67f062d99e4b29"),
    (0x0216BB80, "65da52ef591a7548"),
    (0x0216BB90, "e2d8a51677efd8d3"),
    (0x0216BBB4, "584971c4771d2f9f"),
    (0x0216BBC4, "d0d179ed6c872155"),
];
// fndiff-recorded-end

/// The spec's stubs, with every argument buffer of the case in one memory window set (at the emulator's addresses).
struct Rom<'a> {
    stubs: &'a mut StubEnv,
    mem: Mem,
}

impl Rom<'_> {
    fn s(&mut self, addr: u32, args: &[u32]) -> u32 {
        let mut a: Vec<Arg> = args.iter().map(|&v| Arg::V(v)).collect();
        self.stubs.call(addr, &mut a)
    }
}

impl MemEnv for Rom<'_> {
    fn mem(&mut self) -> &mut Mem {
        &mut self.mem
    }
}

impl Misc10cHooks for Rom<'_> {
    fn fun_0200e458(&mut self, t: u32, id: u32) -> u32 {
        self.s(0x0200_E458, &[t, id])
    }
    fn fun_0200e47c(&mut self, t: u32, id: u32) -> u32 {
        self.s(0x0200_E47C, &[t, id])
    }
}

/// Run `f` on the case's memory: every non-empty argument buffer at its address; the buffers are put back afterwards.
fn with_rom(c: &mut Case, f: impl FnOnce(&mut Rom, &[u32]) -> u64) -> u64 {
    let vals = c.vals.clone();
    let mut mem = Mem::default();
    let mut keep = Vec::new();
    for (i, b) in c.bufs.iter_mut().enumerate() {
        if !b.is_empty() {
            mem.add_window(vals[i], std::mem::take(b));
            keep.push(i);
        }
    }
    let mut rom = Rom {
        stubs: &mut c.env,
        mem,
    };
    let r = f(&mut rom, &vals);
    let wins = std::mem::take(&mut rom.mem.extra);
    for (i, (_, bytes)) in keep.into_iter().zip(wins) {
        c.bufs[i] = bytes;
    }
    r
}

fn run(addr: u32, c: &mut Case) -> u64 {
    with_rom(c, |h, v| match addr {
        0x0216_B970 => record_sum(h, v[0], v[1]) as u64,
        0x0216_BB70 => chal_kind(v[0]) as u64,
        0x0216_BB80 => chal_count(v[0]) as u64,
        0x0216_BB90 => {
            let mut ids = [0u8; 3];
            chal_ids(v[0], &mut ids);
            for (k, b) in ids.into_iter().enumerate() {
                h.w8(v[1] + k as u32, b);
            }
            0
        }
        0x0216_BBB4 => chal_a(v[0]) as u64,
        0x0216_BBC4 => chal_b(v[0]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-script scrcmd_misc10c`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "scrcmd_misc10c",
            include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_misc10c.spec"),
            PORTED,
            RECORDED,
        ),
        &run,
    );
}
