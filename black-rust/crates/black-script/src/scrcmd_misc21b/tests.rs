use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021867A8, "5e3185fff5704316"),
    (0x021867B4, "82198230fcb9242c"),
    (0x021869B0, "1c89e7f5d54ca891"),
    (0x021869BC, "9445a7a36f32e14d"),
    (0x021C8738, "39f392f34e6188bc"),
    (0x021C87D4, "67fdd23606d48447"),
    (0x021C8810, "9f9cf44875ac4af1"),
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
        0x0218_67A8 => wfbc_rec_hi4(h, v[0]) as u64,
        0x0218_67B4 => wfbc_rec_near(h, v[0]) as u64,
        0x0218_69B0 => {
            wfbc_count_a_inc(h, v[0]);
            0
        }
        0x0218_69BC => {
            wfbc_count_b_add10(h, v[0]);
            0
        }
        0x021C_8738 => boat_query(h, v[0], v[1]) as u64,
        0x021C_87D4 => {
            boat_set(h, v[0], v[1], v[2]);
            0
        }
        0x021C_8810 => {
            boat_flag(h, v[0]);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-script scrcmd_misc21b`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "scrcmd_misc21b",
            include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_misc21b.spec"),
            PORTED,
            RECORDED,
        ),
        &run,
    );
}
