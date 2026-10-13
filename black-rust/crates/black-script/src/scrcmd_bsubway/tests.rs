use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021CA234, "5cb7681bcaf7fd8d"),
    (0x021CA238, "4f7af0b644b020d1"),
    (0x021CA23C, "f64d065ba0f72fd8"),
    (0x021CA250, "c8a79800acefeaa1"),
    (0x021CA258, "d9659586872315f3"),
    (0x021CA270, "2195266678bb00b1"),
    (0x021CBEFC, "c20fd52b1d423e61"),
];
// fndiff-recorded-end

/// Every argument buffer of the case in one memory window set (at the emulator's addresses).
struct Rom {
    mem: Mem,
}

impl MemEnv for Rom {
    fn mem(&mut self) -> &mut Mem {
        &mut self.mem
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
    let mut rom = Rom { mem };
    let r = f(&mut rom, &vals);
    let wins = std::mem::take(&mut rom.mem.extra);
    for (i, (_, bytes)) in keep.into_iter().zip(wins) {
        c.bufs[i] = bytes;
    }
    r
}

fn run(addr: u32, c: &mut Case) -> u64 {
    with_rom(c, |h, v| match addr {
        0x021C_A234 => streak(h, v[0]) as u64,
        0x021C_A238 => {
            set_streak(h, v[0], v[1] as u16);
            0
        }
        0x021C_A23C => {
            inc_streak(h, v[0]);
            0
        }
        0x021C_A250 => {
            zero_streak(h, v[0]);
            0
        }
        0x021C_A258 => {
            streak_from(h, v[0], v[1], v[2]);
            0
        }
        0x021C_A270 => party_size(v[0]) as u64,
        0x021C_BEFC => trainer_iv(v[0]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-script scrcmd_bsubway`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "scrcmd_bsubway",
            include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_bsubway.spec"),
            PORTED,
            RECORDED,
        ),
        &run,
    );
}
