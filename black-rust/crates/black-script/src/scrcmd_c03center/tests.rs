use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021B5DD0, "ce8039f09390e2e4"),
    (0x021B5F24, "62c5d01797a17168"),
    (0x021B5FB4, "86b19dfc98aa8e45"),
    (0x021B5FB8, "7aa213794048d09d"),
    (0x021B6124, "c0f7e824bf166ff6"),
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

impl C03Hooks for Rom<'_> {
    fn fun_02159ae8(&mut self, vm: u32, ctx: u32) -> u32 {
        self.s(0x0215_9AE8, &[vm, ctx])
    }
    fn fun_02159b10(&mut self, vm: u32, ctx: u32) -> u32 {
        self.s(0x0215_9B10, &[vm, ctx])
    }
    fn fun_0215b4ac(&mut self, ctx: u32, slot: u32, mon: &mut [u8; 4]) -> u32 {
        self.stubs.call(
            0x0215_B4AC,
            &mut [Arg::V(ctx), Arg::V(slot), Arg::Out(&mut mon[..])],
        )
    }
    fn fun_0201891c(&mut self, mon: u32, k: u32) -> u32 {
        self.s(0x0201_891C, &[mon, k])
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
        0x021B_5DD0 => table_lookup(h, v[0], v[1]) as u64,
        0x021B_5F24 => tutor_move(h, v[0], v[1]) as u64,
        0x021B_5FB4 => tutor_max_level(v[0]) as u64,
        0x021B_5FB8 => tutor_can_learn(h, v[0], v[1]) as u64,
        0x021B_6124 => cmd_486(h, v[0], v[1]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-script scrcmd_c03center`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "scrcmd_c03center",
            include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_c03center.spec"),
            PORTED,
            RECORDED,
        ),
        &run,
    );
}
