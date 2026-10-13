use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021B3350, "4f196f685167693d"),
    (0x021B3568, "6fc15635dfb5f1db"),
    (0x021B49EC, "341e901db65f8b22"),
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

impl ShopHooks for Rom<'_> {
    fn fun_0200c274(&mut self, money: u32) -> u32 {
        self.s(0x0200_C274, &[money])
    }
    fn fun_0200dabc(&mut self, bp: u32) -> u32 {
        self.s(0x0200_DABC, &[bp])
    }
    fn fun_0200c29c(&mut self, money: u32, n: u32) {
        self.s(0x0200_C29C, &[money, n]);
    }
    fn fun_0200da98(&mut self, bp: u32, n: u32) {
        self.s(0x0200_DA98, &[bp, n]);
    }
    fn fun_02008dac(&mut self, records: u32, id: u32, n: u32) {
        self.s(0x0200_8DAC, &[records, id, n]);
    }
    fn fun_02008df0(&mut self, records: u32, id: u32) {
        self.s(0x0200_8DF0, &[records, id]);
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
        0x021B_3350 => currency(h, v[0]) as u64,
        0x021B_3568 => {
            pay(h, v[0], v[1]);
            0
        }
        0x021B_49EC => count_step(h, v[0], v[1], v[2]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-script scrcmd_shop`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "scrcmd_shop",
            include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_shop.spec"),
            PORTED,
            RECORDED,
        ),
        &run,
    );
}
