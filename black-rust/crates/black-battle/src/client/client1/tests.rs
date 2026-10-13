use crate::server_flow::testkit::{run_area_suite, Case};

// Written by tools/check-svflow.sh only, after the Rust hashes matched the ROM run.
// svflow-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021CD6EC, "9280e7dec0439b85"),
    (0x021CD6F0, "fcb727adb86cfb8e"),
    (0x021CD8E0, "c89c8d6fe83e23a8"),
    (0x021CD914, "27c9785c3334a69e"),
    (0x021CD924, "87be406bd080e685"),
    (0x021CD938, "9e82c25820226e3d"),
    (0x021CD95C, "1b5ae7c4d1d9d851"),
    (0x021CD960, "15362e1a0ec6f736"),
    (0x021CD978, "ccb2ef3c895bb18e"),
    (0x021CD97C, "cab10420fe56c95b"),
    (0x021CD984, "03e5c05e2d70a78e"),
    (0x021CD9B0, "3d7b4da0490f5f4f"),
    (0x021CD9C8, "6871017aac935429"),
    (0x021CD9DC, "838af3b2feff34da"),
    (0x021CD9E0, "14b516d20fc91736"),
    (0x021CD9F4, "db9988908c6de10a"),
    (0x021CDA0C, "f6a7f241a173baf7"),
    (0x021CDA18, "f73be3d3530a0c9e"),
    (0x021CDC0C, "d85c2fe6c488f072"),
    (0x021CDD34, "331f18650f7f4e7a"),
    (0x021CDD50, "e99a26d9a0ae6e59"),
    (0x021CDD94, "11eced3ff4a7e976"),
    (0x021CDDA0, "490e13b5a23f56ca"),
    (0x021CDDCC, "faf54f414155b1f6"),
];
// svflow-recorded-end

/// `ret=ofs:argK`: the port's pointer as an offset into argument `k` (0 stays 0).
fn ofs(c: &Case, k: usize, p: u32) -> u64 {
    if p == 0 {
        0
    } else {
        p.wrapping_sub(c.vals[k]) as u64
    }
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let a0 = c.vals.first().copied().unwrap_or(0);
    match addr {
        0x021C_D6EC => {
            super::nop();
            0
        }
        0x021C_D6F0 => {
            let a = super::CreateArgs {
                id: c.vals[4],
                b11c: c.vals[5],
                mode: c.vals[6],
                b1aa: c.vals[7],
                no_ai: c.vals[8],
                heap: c.vals[10],
            };
            let (lo, hi) = c.bufs.split_at_mut(11);
            let p = super::create(
                &mut c.env, c.vals[0], c.vals[1], c.vals[2], c.vals[3], &a, &lo[9], &mut hi[0],
            );
            ofs(c, 11, p)
        }
        0x021C_D8E0 => {
            super::free(&mut c.env, &c.bufs[0], a0);
            0
        }
        0x021C_D914 => {
            super::set_done(&mut c.bufs[0]);
            0
        }
        0x021C_D924 => super::record_data(&mut c.env, &c.bufs[0]) as u64,
        0x021C_D938 => {
            super::set_replay(&mut c.env, &mut c.bufs[0], c.vals[1]);
            0
        }
        0x021C_D95C => {
            super::set_view(&mut c.bufs[0], c.vals[1]);
            0
        }
        0x021C_D960 => {
            super::set_a0(&mut c.bufs[0], c.vals[1]);
            0
        }
        0x021C_D978 => super::adapter(&c.bufs[0]) as u64,
        0x021C_D97C => super::run_step(&mut c.env, &mut c.bufs[0]) as u64,
        0x021C_D984 => {
            super::replay_restart(&mut c.env, &mut c.bufs[0], c.vals[1]);
            0
        }
        0x021C_D9B0 => {
            super::restart_main(&mut c.env, &mut c.bufs[0]);
            0
        }
        0x021C_D9C8 => super::is_replay(&c.bufs[0]) as u64,
        0x021C_D9DC => super::cursor_u16(&c.bufs[0]) as u64,
        0x021C_D9E0 => super::is_waiting(&c.bufs[0]) as u64,
        0x021C_D9F4 => {
            super::counter_add(&mut c.bufs[0], c.vals[1]);
            0
        }
        0x021C_DA0C => {
            super::set_step(&mut c.bufs[0], c.vals[1]);
            0
        }
        0x021C_DA18 => super::main_step(&mut c.env, &mut c.bufs[0], a0) as u64,
        0x021C_DC0C => super::wait_step(&mut c.env, &mut c.bufs[0], a0) as u64,
        0x021C_DD34 => {
            super::result_clear(&mut c.bufs[0], a0);
            0
        }
        0x021C_DD50 => {
            // arg 2: null or the 4-byte region `out` (region 0)
            let mut out = u32::from_le_bytes(c.regions[0][..4].try_into().unwrap());
            let has_out = c.vals[2] != 0;
            let r = super::request_handler(
                &c.bufs[0],
                c.vals[1],
                if has_out { Some(&mut out) } else { None },
            );
            if has_out {
                c.regions[0][..4].copy_from_slice(&out.to_le_bytes());
            }
            r as u64
        }
        0x021C_DD94 => {
            let (obj, out) = c.bufs.split_at_mut(1);
            super::request_pair(&obj[0], &mut out[0]);
            0
        }
        0x021C_DDA0 => super::effect_pending(&mut c.env, &c.bufs[0]) as u64,
        0x021C_DDCC => super::flag_20(&c.bufs[0]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/rom-diff/svflow.py client1`).
#[test]
fn svflow_match_rom() {
    run_area_suite(
        "client1",
        include_str!("../../../../../tools/rom-diff/svflow_specs/client1.spec"),
        super::PORTED,
        RECORDED,
        call,
    );
}
