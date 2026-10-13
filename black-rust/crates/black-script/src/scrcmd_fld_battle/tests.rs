use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021B5824, "e4b9cdee3902c9fd"),
    (0x021B59B0, "7a0a3aea7e3a6e6a"),
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

impl FldBattleHooks for Rom<'_> {
    fn fun_0200ca1c(&mut self, gamedata: u32) -> u32 {
        self.s(0x0200_CA1C, &[gamedata])
    }
    fn fun_0200ced4(&mut self, dex: u32, id: u32) -> u32 {
        self.s(0x0200_CED4, &[dex, id])
    }
    fn fun_02030734(&mut self, heap: u32, size: u32, from_end: u32, file: u32, line: u32) -> u32 {
        self.s(0x0203_0734, &[heap, size, from_end, file, line])
    }
    fn fun_020307b0(&mut self, block: u32) {
        self.s(0x0203_07B0, &[block]);
    }
    fn fun_021cc720(&mut self, fm: u32, party: u32, bx: u32, a: u32, b: u32, heap: u32) -> u32 {
        self.s(0x021C_C720, &[fm, party, bx, a, b, heap])
    }
    fn fun_021cc810(&mut self, sel: u32) -> u32 {
        self.s(0x021C_C810, &[sel])
    }
    fn fun_021cc7b4(&mut self, sel: u32, cancel: &mut [u8; 4]) -> u32 {
        self.stubs
            .call(0x021C_C7B4, &mut [Arg::V(sel), Arg::Out(&mut cancel[..])])
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
        0x021B_5824 => collect(h, v[0], v[1], v[2], v[3]) as u64,
        0x021B_59B0 => select_event(h, v[0], v[1], v[2]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-script scrcmd_fld_battle`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "scrcmd_fld_battle",
            include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_fld_battle.spec"),
            PORTED,
            RECORDED,
        ),
        &run,
    );
}
