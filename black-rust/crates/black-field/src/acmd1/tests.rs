use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02197D3C, "81c0627be958c4b4"),
    (0x02197DA0, "74ac20f7825920d3"),
    (0x02197DAC, "f4030fc59424027d"),
    (0x02197DCC, "7c8e1aa519bb3c83"),
    (0x02197DE8, "444601f6d35221ae"),
    (0x02197DF4, "cc06f43850d75f16"),
    (0x02197E10, "48657f36c706d5cc"),
    (0x02197E28, "53b94df83833d37b"),
    (0x02197E44, "0affd666e073898a"),
    (0x02197E78, "6e8879ea2a17ace5"),
    (0x02197E7C, "8374968a8a72d04f"),
    (0x02197EB4, "2122f5a920e5ae0d"),
    (0x02197EDC, "483e59abb16bc12a"),
    (0x02197F20, "d57a6f2924fe57b1"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/acmd1.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x0219_7D3C => list_task_new(e, v[0], v[1]) as u64,
        0x0219_7DA0 => list_ended(e, v[0]) as u64,
        0x0219_7DAC => {
            list_delete(e, v[0]);
            0
        }
        0x0219_7DCC => {
            list_proc(e, v[0], v[1]);
            0
        }
        0x0219_7DE8 => list_start(&mut e.mem, v[0]) as u64,
        0x0219_7DF4 => list_wait_ready(e, v[0]) as u64,
        0x0219_7E10 => list_set(e, v[0]) as u64,
        0x0219_7E28 => list_wait_done(e, v[0]) as u64,
        0x0219_7E44 => list_next(&mut e.mem, v[0]) as u64,
        0x0219_7E78 => list_stop(v[0]) as u64,
        0x0219_7E7C => code_for_dir(v[0], v[1]) as u64,
        0x0219_7EB4 => {
            super::run(e, v[0]);
            0
        }
        0x0219_7EDC => action(e, v[0]) as u64,
        0x0219_7F20 => step(e, v[0], v[1], v[2]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field acmd1`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("acmd1", SPEC, PORTED, RECORDED), &call);
}

#[test]
fn code_for_dir_maps_within_a_kind() {
    assert_eq!(code_for_dir(2, 0x0D), 0x0E);
    assert_eq!(code_for_dir(3, 0x5C), 0x5D);
    assert_eq!(code_for_dir(1, 0x99), 0x99);
    assert_eq!(code_for_dir(0, 0x1_0000), 0x1_0000);
}

/// The embedded ROM tables ([`DIR_CODES`], [`LIST_SEQS`], [`GRID_STEPS`], [`RAIL_STEPS`]) are the build's (`OVY_21.sbin`).
#[test]
fn tables_match_rom() {
    let Some(w) = crate::testenv::overlay21() else {
        return;
    };
    for (i, t) in DIR_CODES.iter().enumerate() {
        let p = w(0x021D_5D6C + 4 * i as u32);
        for (k, &code) in t.iter().enumerate() {
            assert_eq!(w(p + 4 * k as u32), code, "dir table {i}");
        }
    }
    assert_eq!(w(0x021D_5D6C + 4 * DIR_CODES.len() as u32), 0);
    for (k, &f) in LIST_SEQS.iter().enumerate() {
        assert_eq!(w(0x021D_4A50 + 4 * k as u32), f);
    }
    // literal pools of 02197D3C / 02197DCC / 02197E7C / 02197F20
    assert_eq!(w(0x0219_7D98), FILE_FLDMMDL_ACMD);
    assert_eq!(w(0x0219_7D9C), LIST_PROC);
    assert_eq!(w(0x0219_7DE4), 0x021D_4A50);
    assert_eq!(w(0x0219_7EB0), 0x021D_5D6C);
    assert_eq!(w(0x0219_7F54), 0x021D_624C);
    assert_eq!(w(0x0219_7F58), 0x021D_5F58);
    for (table, steps) in [(0x021D_5F58, &GRID_STEPS), (0x021D_624C, &RAIL_STEPS)] {
        for (code, (list, fs)) in steps.iter().enumerate() {
            assert_eq!(
                w(table + 4 * code as u32),
                *list,
                "{table:#x} code {code:#x}"
            );
            for (k, &f) in fs.iter().enumerate() {
                assert_eq!(
                    w(list + 4 * k as u32),
                    f,
                    "{table:#x} code {code:#x} step {k}"
                );
            }
        }
    }
    // every grid list ends with the end step
    for (_, fs) in GRID_STEPS.iter() {
        assert_eq!(*fs.last().unwrap(), 0x0219_7F5D);
    }
}
