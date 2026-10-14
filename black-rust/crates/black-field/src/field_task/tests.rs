use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021C6C70, "edec4d857dad3094"),
    (0x021C6CA8, "fb5594b03545ec81"),
    (0x021C6CCC, "ed06f2103a8bfba1"),
    (0x021C6CD0, "721520006ad3a004"),
    (0x021C6CE0, "955de5cd6703d916"),
    (0x021C6CE4, "691691cebcb354b2"),
    (0x021C6CEC, "7eb870e9e4baf3f5"),
    (0x021C6CF4, "0380d73768adcbe9"),
    (0x021C6D10, "c95cf840871b62ec"),
    (0x021C6D5C, "90b123c5f46bc6cd"),
    (0x021C6D90, "def84a5c7b7e06a2"),
    (0x021C6DE0, "93b190847eff7922"),
    (0x021C6E20, "0c866f219da25f5f"),
    (0x021C6E48, "79042bc5a1558b00"),
    (0x021C6E5C, "20f30f8548073b62"),
    (0x021C6E74, "d0e09a0ae276b064"),
    (0x021C6E94, "0059bb3c12d9fd1a"),
    (0x021C6EA8, "3acf31a4723571e7"),
    (0x021C6EDC, "4d4eda78c9019adf"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/field_task.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x021C_6C70 => u64::from(task_new(e, v[0], v[1], v[2])),
        0x021C_6CA8 => {
            task_free(e, v[0]);
            0
        }
        0x021C_6CCC => u64::from(task_work(e, v[0])),
        0x021C_6CD0 => u64::from(task_is_done(e, v[0])),
        0x021C_6CE0 => u64::from(task_state(e, v[0])),
        0x021C_6CE4 => {
            task_start(e, v[0]);
            0
        }
        0x021C_6CEC => {
            task_end(e, v[0]);
            0
        }
        0x021C_6CF4 => {
            task_run(e, v[0]);
            0
        }
        0x021C_6D10 => u64::from(mgr_new(e, v[0], v[1])),
        0x021C_6D5C => {
            mgr_free(e, v[0]);
            0
        }
        0x021C_6D90 => {
            mgr_run(e, v[0]);
            0
        }
        0x021C_6DE0 => {
            mgr_add(e, v[0], v[1], v[2]);
            0
        }
        0x021C_6E20 => u64::from(mgr_active(e, v[0])),
        0x021C_6E48 => u64::from(mgr_is_empty(e, v[0])),
        0x021C_6E5C => {
            mgr_set(e, v[0], v[1], v[2], v[3]);
            0
        }
        0x021C_6E74 => {
            mgr_remove(e, v[0], v[1]);
            0
        }
        0x021C_6E94 => {
            mgr_run_one(e, v[0], v[1]);
            0
        }
        0x021C_6EA8 => {
            mgr_start_waiting(e, v[0], v[1]);
            0
        }
        0x021C_6EDC => {
            mgr_cleanup(e, v[0], v[1]);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field field_task`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("field_task", SPEC, PORTED, RECORDED), &call);
}
