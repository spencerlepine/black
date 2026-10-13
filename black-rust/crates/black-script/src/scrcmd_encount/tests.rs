use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021B5190, "973bdfb11a1397e6"),
    (0x021B538C, "b62de9bb6a813ae6"),
    (0x021B55F0, "f009640c96059e7c"),
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

impl EncountHooks for Rom<'_> {
    fn fun_0215a5dc(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A5DC, &[ctx])
    }
    fn fun_0201210c(&mut self, gamesys: u32) -> u32 {
        self.s(0x0201_210C, &[gamesys])
    }
    fn fun_020124e4(&mut self, event: u32) -> u32 {
        self.s(0x0201_24E4, &[event])
    }
    fn fun_02188c58(&mut self, fm: u32) -> u32 {
        self.s(0x0218_8C58, &[fm])
    }
    fn fun_0218ec54(&mut self, e: u32) {
        self.s(0x0218_EC54, &[e]);
    }
    fn fun_0218fe50(&mut self, e: u32) -> u32 {
        self.s(0x0218_FE50, &[e])
    }
}

/// Run `f` on the case's memory: every non-empty argument buffer (and region) at its address; the buffers are put back
/// afterwards.
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
        0x021B_5190 => add_unique(h, v[0], v[1], v[2]) as u64,
        0x021B_538C => cmd_321(h, v[0], v[1]) as u64,
        0x021B_55F0 => wait_325(h, v[0]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-script scrcmd_encount`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "scrcmd_encount",
            include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_encount.spec"),
            PORTED,
            RECORDED,
        ),
        &run,
    );
}
