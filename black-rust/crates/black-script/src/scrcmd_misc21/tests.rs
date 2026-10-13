use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021CFA1C, "97aeccb69611967c"),
    (0x021CFCF4, "eb812d1bb178947a"),
    (0x021CFD88, "2ed4e3a15afbdb38"),
    (0x021D1988, "d0d3bcecd4b621c3"),
    (0x021D21C0, "d67b0ade3b5c7b41"),
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

impl Misc21Hooks for Rom<'_> {
    fn fun_02011610(&mut self, anime: u32, t: &mut [u8; 12]) {
        self.stubs
            .call(0x0201_1610, &mut [Arg::V(anime), Arg::Out(&mut t[..])]);
    }
    fn fun_02011638(&mut self, anime: u32, r: &mut [u8; 12]) {
        self.stubs
            .call(0x0201_1638, &mut [Arg::V(anime), Arg::Out(&mut r[..])]);
    }
    fn fun_0207c9d0(&mut self, a: &[u8; 12], b: &[u8; 12], out: &mut [u8; 12]) {
        self.stubs.call(
            0x0207_C9D0,
            &mut [Arg::In(a), Arg::In(b), Arg::Out(&mut out[..])],
        );
    }
    fn fun_0216d89c(&mut self, mmdl: u32, pos: &[u8; 12]) {
        self.stubs
            .call(0x0216_D89C, &mut [Arg::V(mmdl), Arg::In(pos)]);
    }
    fn fun_0216d60c(&mut self, mmdl: u32, dir: u32) {
        self.s(0x0216_D60C, &[mmdl, dir]);
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
        0x021C_FA1C => ot_list_add(h, v[0], v[1]) as u64,
        0x021C_FCF4 => match_index(h, v[0], v[1]) as u64,
        0x021C_FD88 => message_id(v[0], v[1], v[2], v[3], v[4]) as u64,
        0x021D_1988 => {
            obj_anim_apply(h, v[0], v[1], v[2]);
            0
        }
        0x021D_21C0 => in_table_a(v[0]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-script scrcmd_misc21`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "scrcmd_misc21",
            include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_misc21.spec"),
            PORTED,
            RECORDED,
        ),
        &run,
    );
}
