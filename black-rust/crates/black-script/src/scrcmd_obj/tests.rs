use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021B1668, "63cfde56f815f7bf"),
    (0x021B16DC, "70c3acd0b842fa0c"),
    (0x021B19E0, "3fe7d2bc945f6274"),
    (0x021B1A44, "3c97c1893f30fddc"),
    (0x021B1F60, "0c248ff0a649875f"),
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

impl ObjHooks for Rom<'_> {
    fn fun_02159ae8(&mut self, vm: u32, ctx: u32) -> u32 {
        self.s(0x0215_9AE8, &[vm, ctx])
    }
    fn fun_02159b10(&mut self, vm: u32, ctx: u32) -> u32 {
        self.s(0x0215_9B10, &[vm, ctx])
    }
    fn fun_0215a5ec(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A5EC, &[ctx])
    }
    fn fun_0215a5f4(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A5F4, &[ctx])
    }
    fn fun_0215a6fc(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A6FC, &[ctx])
    }
    fn fun_02158f58(&mut self, sw: u32) -> u32 {
        self.s(0x0215_8F58, &[sw])
    }
    fn fun_02158f90(&mut self, sw: u32) -> u32 {
        self.s(0x0215_8F90, &[sw])
    }
    fn fun_0216de94(&mut self, mmdls: u32, id: u32) -> u32 {
        self.s(0x0216_DE94, &[mmdls, id])
    }
    fn fun_0216de54(&mut self, mmdls: u32, id: u32) -> u32 {
        self.s(0x0216_DE54, &[mmdls, id])
    }
    fn fun_0216d63c(&mut self, mmdl: u32, n: u32) -> u32 {
        self.s(0x0216_D63C, &[mmdl, n])
    }
    fn fun_0216d454(&mut self, mmdl: u32) -> u32 {
        self.s(0x0216_D454, &[mmdl])
    }
    fn fun_0216da5c(&mut self, mmdl: u32) -> u32 {
        self.s(0x0216_DA5C, &[mmdl])
    }
    fn fun_0216daf8(&mut self, mmdl: u32) {
        self.s(0x0216_DAF8, &[mmdl]);
    }
    fn fun_0219bba4(&mut self, mmdl: u32) -> u32 {
        self.s(0x0219_BBA4, &[mmdl])
    }
    fn fun_02188cb4(&mut self, fm: u32) -> u32 {
        self.s(0x0218_8CB4, &[fm])
    }
    fn fun_021a3098(&mut self, player: u32) -> u32 {
        self.s(0x021A_3098, &[player])
    }
    fn fun_021b1e74(&mut self, ctx: u32) -> u32 {
        self.s(0x021B_1E74, &[ctx])
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
    let globals: &[u32] = if addr == 0x021B_1A44 {
        &[LOCK_FLAGS]
    } else {
        &[]
    };
    with_rom(c, globals, |h, v| match addr {
        0x021B_1668 => object_of(h, v[0], v[1], 0) as u64,
        0x021B_16DC => wait_movements(h, v[0], v[1]) as u64,
        0x021B_19E0 => cmd_object_prop(h, v[0], v[1]) as u64,
        0x021B_1A44 => wait_lock(h, v[0], v[1]) as u64,
        0x021B_1F60 => cmd_player_dir(h, v[0], v[1]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-script scrcmd_obj`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "scrcmd_obj",
            include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_obj.spec"),
            PORTED,
            RECORDED,
        ),
        &run,
    );
}
