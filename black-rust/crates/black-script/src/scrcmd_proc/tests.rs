use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0215C63C, "12f00e7615ebb409"),
    (0x0215C6F0, "c92956ddfc1e3ef8"),
    (0x0215C700, "619f7bf9888630ea"),
    (0x0215C7C4, "d47a2e75d75d78cb"),
    (0x0215C870, "7c9f20a4e15b7551"),
    (0x0215CCE8, "a1b5d6eadf0c2fc4"),
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

impl ProcHooks for Rom<'_> {
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
    fn fun_02158fb4(&mut self, sw: u32) -> u32 {
        self.s(0x0215_8FB4, &[sw])
    }
    fn fun_02158fbc(&mut self, sw: u32) {
        self.s(0x0215_8FBC, &[sw]);
    }
    fn fun_0215897c(&mut self, sw: u32, child: u32) {
        self.s(0x0215_897C, &[sw, child]);
    }
    fn fun_02030734(&mut self, heap: u32, size: u32, from_end: u32, file: u32, line: u32) -> u32 {
        self.s(0x0203_0734, &[heap, size, from_end, file, line])
    }
    fn fun_020307b0(&mut self, block: u32) {
        self.s(0x0203_07B0, &[block]);
    }
    fn fun_0216f940(&mut self, block: u32) {
        self.s(0x0216_F940, &[block]);
    }
    fn fun_020120b4(&mut self, gamesys: u32, ovl: u32, proc_table: u32, param: u32) {
        self.s(0x0201_20B4, &[gamesys, ovl, proc_table, param]);
    }
    fn fun_0201210c(&mut self, gamesys: u32) -> u32 {
        self.s(0x0201_210C, &[gamesys])
    }
    fn fun_020124ac(&mut self, gamesys: u32, kind: u32, proc_table: u32, rec: &[u8; 8]) -> u32 {
        self.stubs.call(
            0x0201_24AC,
            &mut [
                Arg::V(gamesys),
                Arg::V(kind),
                Arg::V(proc_table),
                Arg::In(rec),
            ],
        )
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
    with_rom(c, &[], |h, v| match addr {
        0x0215_C63C => {
            proc_call_wait(h, v[0], v[1], v[2], v[3], v[4], v[5], v[6]);
            0
        }
        0x0215_C6F0 => cmd_332(h, v[0], v[1]) as u64,
        0x0215_C700 => {
            cb_334(h, v[0]);
            0
        }
        0x0215_C7C4 => {
            cb_336(h, v[0]);
            0
        }
        0x0215_C870 => {
            cb_489(h, v[0]);
            0
        }
        0x0215_CCE8 => cmd_357(h, v[0], v[1]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-script scrcmd_proc`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "scrcmd_proc",
            include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_proc.spec"),
            PORTED,
            RECORDED,
        ),
        &run,
    );
}
