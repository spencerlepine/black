use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021AE9CC, "f216e18e4c8ee070"),
    (0x021AEBB0, "98c632fa69d7b9a3"),
    (0x021AECC0, "f97daa2de9a27361"),
    (0x021AEEE0, "88b810d018b8d54f"),
    (0x021AF1E8, "7ab6ebe94e5ef410"),
    (0x021AF21C, "717ec5548adca135"),
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

impl FldHooks for Rom<'_> {
    fn fun_02159ae8(&mut self, vm: u32, ctx: u32) -> u32 {
        self.s(0x0215_9AE8, &[vm, ctx])
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
    fn fun_02159124(&mut self, sw: u32, i: u32) -> u32 {
        self.s(0x0215_9124, &[sw, i])
    }
    fn fun_02158f8c(&mut self, sw: u32) -> u32 {
        self.s(0x0215_8F8C, &[sw])
    }
    fn fun_02158f88(&mut self, sw: u32) -> u32 {
        self.s(0x0215_8F88, &[sw])
    }
    fn fun_0215a460(&mut self, id: u32) -> u32 {
        self.s(0x0215_A460, &[id])
    }
    fn fun_0215a4a0(&mut self, id: u32) -> u32 {
        self.s(0x0215_A4A0, &[id])
    }
    fn fun_0215a4b4(&mut self, id: u32) -> u32 {
        self.s(0x0215_A4B4, &[id])
    }
    fn fun_0202a230(&mut self, v: u32, k: u32) -> u32 {
        self.s(0x0202_A230, &[v, k])
    }
    fn fun_02005f38(&mut self) -> u32 {
        self.s(0x0200_5F38, &[])
    }
    fn fun_02006224(&mut self, i: u32) -> u32 {
        self.s(0x0200_6224, &[i])
    }
    fn fun_02006148(&mut self, x: u32) -> u32 {
        self.s(0x0200_6148, &[x])
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
        0x021A_E9CC => cmd_trainer_rec_14(h, v[0], v[1]) as u64,
        0x021A_EBB0 => cmd_three_vars(h, v[0], v[1]) as u64,
        0x021A_ECC0 => cmd_classify_146(h, v[0], v[1]) as u64,
        0x021A_EEE0 => wait_160(h, v[0], v[1]) as u64,
        0x021A_F1E8 => {
            sound_flags_update(h, v[0]);
            0
        }
        0x021A_F21C => {
            sound_flags_add(h, v[0], v[1]);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-script scrcmd_fld`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "scrcmd_fld",
            include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_fld.spec"),
            PORTED,
            RECORDED,
        ),
        &run,
    );
}
