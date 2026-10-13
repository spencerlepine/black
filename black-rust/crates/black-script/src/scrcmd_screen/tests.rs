use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021B2830, "7d712a6a2d399450"),
    (0x021B2860, "dbb77b7add5979c4"),
    (0x021B2910, "4d127f5aa10bc2cd"),
    (0x021B2E60, "79fc3ed6baea0579"),
    (0x021B2F18, "ef5c62eca926919d"),
];
// fndiff-recorded-end

/// The spec's stubs, with every argument buffer (and `global`) of the case in one memory window set (at the emulator's
/// addresses).
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

impl ScreenHooks for Rom<'_> {
    fn fun_02011330(&mut self, vm: u32) -> u32 {
        self.s(0x0201_1330, &[vm])
    }
    fn fun_020113d0(&mut self, vm: u32, cb: u32) {
        self.s(0x0201_13D0, &[vm, cb]);
    }
    fn fun_02159b10(&mut self, vm: u32, ctx: u32) -> u32 {
        self.s(0x0215_9B10, &[vm, ctx])
    }
    fn fun_0215a5dc(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A5DC, &[ctx])
    }
    fn fun_0215a5f4(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A5F4, &[ctx])
    }
    fn fun_0201210c(&mut self, gamesys: u32) -> u32 {
        self.s(0x0201_210C, &[gamesys])
    }
    fn fun_0204e720(&mut self, mode: u32, start: u32, end: u32, wait: u32) {
        self.s(0x0204_E720, &[mode, start, end, wait]);
    }
    fn fun_0204e804(&mut self) -> u32 {
        self.s(0x0204_E804, &[])
    }
    fn fun_021b2ebc(&mut self, gamesys: u32, fm: u32, kind: u32, color: u32, no_wait: u32) {
        self.s(0x021B_2EBC, &[gamesys, fm, kind, color, no_wait]);
    }
    fn fun_02043658(&mut self, x: u32) -> u32 {
        self.s(0x0204_3658, &[x])
    }
    fn fun_02043534(&mut self, x: u32) {
        self.s(0x0204_3534, &[x]);
    }
    fn fun_02188da0(&mut self, x: u32) {
        self.s(0x0218_8DA0, &[x]);
    }
    fn fun_02188db0(&mut self, fm: u32) {
        self.s(0x0218_8DB0, &[fm]);
    }
    fn fun_02189a1c(&mut self, fm: u32, on: u32) {
        self.s(0x0218_9A1C, &[fm, on]);
    }
    fn fun_02030eac(&mut self, task: u32) {
        self.s(0x0203_0EAC, &[task]);
    }
    fn fun_020307b0(&mut self, block: u32) {
        self.s(0x0203_07B0, &[block]);
    }
}

/// Run `f` on the case's memory: every non-empty argument buffer at its address and the function's globals at `globals`; the
/// buffers are put back afterwards.
fn with_rom(c: &mut Case, globals: &[u32], f: impl FnOnce(&mut Rom, &[u32]) -> u64) -> u64 {
    let vals = c.vals.clone();
    let mut mem = Mem::default();
    let mut keep = Vec::new();
    for (i, b) in c.bufs.iter_mut().enumerate() {
        if !b.is_empty() {
            mem.add_window(vals[i], std::mem::take(b));
            keep.push(Some(i));
        }
    }
    for (k, g) in c.globals.iter_mut().enumerate() {
        mem.add_window(globals[k], std::mem::take(g));
        keep.push(None);
    }
    let mut rom = Rom {
        stubs: &mut c.env,
        mem,
    };
    let r = f(&mut rom, &vals);
    let wins = std::mem::take(&mut rom.mem.extra);
    let mut k = 0;
    for (slot, (_, bytes)) in keep.into_iter().zip(wins) {
        match slot {
            Some(i) => c.bufs[i] = bytes,
            None => {
                c.globals[k] = bytes;
                k += 1;
            }
        }
    }
    r
}

fn run(addr: u32, c: &mut Case) -> u64 {
    let globals: &[u32] = if addr == 0x021B_2E60 {
        &[WAIT_OBJ_ID]
    } else {
        &[]
    };
    with_rom(c, globals, |h, v| match addr {
        0x021B_2830 => cmd_fade(h, v[0], v[1]) as u64,
        0x021B_2860 => wait_fade(h, v[0], v[1]) as u64,
        0x021B_2910 => cmd_field_fade_429(h, v[0], v[1]) as u64,
        0x021B_2E60 => cmd_298(h, v[0], v[1]) as u64,
        0x021B_2F18 => {
            field_fade_task(h, v[0], v[1]);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-script scrcmd_screen`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "scrcmd_screen",
            include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_screen.spec"),
            PORTED,
            RECORDED,
        ),
        &run,
    );
}
