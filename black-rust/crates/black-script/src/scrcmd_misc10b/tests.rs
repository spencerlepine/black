use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02166658, "b0fbb028a6893ff2"),
    (0x0216670C, "4d631892010c14a5"),
    (0x02166714, "c2c32965306900c1"),
    (0x02166824, "a6390818ca50fa24"),
    (0x02166834, "69d653f3bb76cf76"),
    (0x0216B2E4, "b22c394b1871c2fa"),
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
        0x0216_6658 => boat_slot_word(h, v[0], v[1]) as u64,
        0x0216_670C => boat_time(h, v[0]) as u64,
        0x0216_6714 => {
            boat_time_set(h, v[0], v[1]);
            0
        }
        0x0216_6824 => rec20_byte(h, v[0], v[1]) as u64,
        0x0216_6834 => rec20_field(h, v[0], v[1]) as u64,
        0x0216_B2E4 => {
            symbol_warp_pos(h, v[0], v[1], v[2]);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-script scrcmd_misc10b`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "scrcmd_misc10b",
            include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_misc10b.spec"),
            PORTED,
            RECORDED,
        ),
        &run,
    );
}
