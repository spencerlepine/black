use crate::server_flow::testkit::{run_area_suite, Case};

// Written by tools/check-svflow.sh only, after the Rust hashes matched the ROM run.
// svflow-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021BF324, "fe5cf1365b24d216"),
    (0x021BF3D8, "ac7cde6a6a6d031d"),
    (0x021BF4AC, "94d46823ae1e995a"),
    (0x021BF6E8, "90698ace6bb56945"),
    (0x021BF78C, "ff991c0372d742cc"),
    (0x021BF83C, "831e995abf26ddb1"),
    (0x021BF8D4, "1a532d239c842a41"),
    (0x021BF9A4, "e43b0ae7f88f1174"),
    (0x021BFA00, "8d8fec1d58de8d56"),
    (0x021BFA60, "3110f70a3d88664f"),
    (0x021BFBD0, "66a39dacbb736dbd"),
    (0x021BFC08, "0329b2f811d272e5"),
    (0x021BFD24, "2548240159a7ec55"),
    (0x021BFE60, "93b9e169b41a0617"),
    (0x021BFEAC, "f2caa01c43d8f568"),
    (0x021BFF60, "9eb511a357d5c868"),
    (0x021C001C, "842119695dc41135"),
    (0x021C024C, "05f21c604dc7b9a9"),
    (0x021C0274, "95b850026004e090"),
    (0x021C02CC, "a29429fc5388fa67"),
    (0x021C031C, "bc5239f5124e6891"),
    (0x021C03AC, "51c96114ad968278"),
    (0x021C03D8, "1237c1e9a542af43"),
    (0x021C0404, "8cd67fcf049de9b3"),
    (0x021C0450, "cd39cf5bea9a19bf"),
    (0x021C0520, "137be1545eab2950"),
    (0x021C0780, "4b57ace52b4d482b"),
    (0x021C07AC, "b094da5e1b7179a6"),
    (0x021C07D4, "823eff02ef6653ef"),
    (0x021C0898, "29651c6a52062640"),
    (0x021C08CC, "1efccfec2fd158ca"),
    (0x021C091C, "445000e739cbd8f4"),
    (0x021C0A4C, "738d3637b99b4a6a"),
    (0x021C0C34, "7fcae47bda5789e8"),
    (0x021C0C38, "7435523cd055e7dd"),
    (0x021C0CB4, "92390cac44f228d6"),
    (0x021C0CE8, "ae7a29de166e3d4d"),
    (0x021C0D70, "1e0223c9bf3d3c67"),
    (0x021C0D90, "be520df3f5c2db2b"),
];
// svflow-recorded-end

/// The command queue context: region `$q` = the queue buffer, global 0 = the scratch words at
/// 0x021F3B44 (a macro so the borrows of `c.env` / `c.work` stay disjoint).
macro_rules! cx {
    ($c:ident, $q:expr) => {
        super::Cx {
            q: &mut $c.regions[$q],
            scratch: &mut $c.globals[0],
            w_addr: $c.vals[0],
        }
    };
}

pub(crate) fn call<E: crate::server_flow::env::FlowEnv>(addr: u32, c: &mut Case<E>) -> u64 {
    match addr {
        0x021B_F324 => super::pair_blocked(&mut c.env, &c.work, c.vals[1], c.vals[2]) as u64,
        0x021C_08CC => {
            // region 0 = the command queue buffer, global 0 = the scratch words at 0x021F3B44
            super::apply_and_queue(
                &mut c.env,
                &c.work,
                &mut c.regions[0],
                &mut c.globals[0],
                c.vals[1],
                c.vals[2],
                c.vals[3],
            );
            0
        }
        0x021C_0CB4 => {
            let (rec, out) = c.bufs.split_at_mut(2);
            super::record_mons(&mut c.env, &c.work, &rec[1], &mut out[0]) as u64
        }
        0x021C_0D70 => super::record_total(&c.bufs[0]) as u64,
        0x021C_0C34 => super::record_len(&c.bufs[0]) as u64,
        0x021C_0780 => {
            let mut cx = cx!(c, 0);
            super::put_cmd_25(&mut c.env, &mut cx, c.vals[1], c.vals[2], c.vals[3]);
            0
        }
        0x021C_0898 => {
            let mut cx = cx!(c, 0);
            super::apply_up_to(
                &mut c.env, &c.work, &mut cx, c.vals[1], c.vals[2], c.vals[3],
            ) as u64
        }
        0x021B_FBD0 => {
            let mut cx = cx!(c, 0);
            super::rewrite_cmd_30(&mut c.env, &mut cx, c.vals[1], &mut c.bufs[2], c.vals[3]);
            0
        }
        0x021C_02CC => super::build_02cc(&mut c.env, c.vals[1], c.vals[2], c.vals[3]) as u64,
        0x021C_0274 => super::build_0274(&mut c.env, c.vals[1], c.vals[2], c.vals[3]) as u64,
        0x021B_F9A4 => super::build_f9a4(&mut c.env, c.vals[1], c.vals[2], &c.bufs[3]) as u64,
        0x021B_FA00 => {
            super::same_side_gate(&mut c.env, &c.work, c.vals[1], c.vals[2], c.vals[3]) as u64
        }
        0x021C_03D8 => {
            super::state2_notify(&mut c.env, c.vals[1]);
            0
        }
        0x021C_03AC => super::flag7_roll(&mut c.env, c.vals[1]) as u64,
        0x021C_0404 => super::state3_roll(&mut c.env, c.vals[1], c.vals[2]) as u64,
        0x021C_0C38 => {
            let rec = c.bufs[1].clone();
            let (_, rest) = c.bufs.split_at_mut(2);
            let (mons, rest) = rest.split_first_mut().unwrap();
            let (values, rest) = rest.split_first_mut().unwrap();
            let (tables, rest) = rest.split_first_mut().unwrap();
            super::record_copy_hit(
                &mut c.env,
                &c.work,
                &rec,
                mons,
                values,
                tables,
                &mut rest[0],
            ) as u64
        }
        0x021C_0CE8 => {
            let rec = c.bufs[1].clone();
            let (_, rest) = c.bufs.split_at_mut(2);
            let (mons, rest) = rest.split_first_mut().unwrap();
            let (values, rest) = rest.split_first_mut().unwrap();
            let (tables, rest) = rest.split_first_mut().unwrap();
            let (flags, rest) = rest.split_first_mut().unwrap();
            let mut out = super::MissOut {
                values,
                tables,
                flags,
                rolls: &mut rest[0],
            };
            super::record_copy_miss(&mut c.env, &c.work, &rec, mons, &mut out) as u64
        }
        0x021C_0520 => {
            let mut cx = cx!(c, 0);
            super::failure_message(&mut c.env, &mut cx, c.vals[1], c.vals[2], c.vals[3]);
            0
        }
        0x021C_0450 => {
            let mut cx = cx!(c, 0);
            super::failure_print(
                &mut c.env,
                &mut c.work,
                &mut cx,
                c.vals[1],
                c.vals[2],
                c.vals[3],
            );
            0
        }
        0x021C_024C => {
            let mut cx = cx!(c, 0);
            super::failure_check(&mut c.env, &mut c.work, &mut cx, c.vals[1], c.vals[2]) as u64
        }
        0x021B_FE60 => {
            super::clear_flag_1a(&mut c.env, c.vals[1]);
            0
        }
        0x021C_031C => {
            let mut cx = cx!(c, 0);
            super::flag6_check(&mut c.env, &c.work, &mut cx, c.vals[1]) as u64
        }
        0x021C_07D4 => {
            super::build_07d4(&mut c.env, c.vals[1], c.vals[2], c.vals[3], c.vals[4]) as u64
        }
        0x021C_07AC => {
            let mut cx = cx!(c, 0);
            super::apply_listed(
                &mut c.env, &c.work, &mut cx, c.vals[1], c.vals[2], c.vals[3], c.vals[4],
            ) as u64
        }
        0x021B_FA60 => super::can_hit(
            &mut c.env, &c.work, c.vals[1], c.vals[2], &c.bufs[3], c.vals[3],
        ) as u64,
        0x021B_FEAC => {
            let mut cx = cx!(c, 0);
            super::swap_effect(&mut c.env, &mut cx, c.vals[1], c.vals[2], &mut c.bufs[3]) as u64
        }
        0x021B_F83C => {
            super::filter_list_pass(&mut c.env, &c.work, c.vals[2], c.vals[3]);
            0
        }
        0x021B_F8D4 => {
            super::filter_list_hits(
                &mut c.env, &c.work, &c.bufs[1], c.vals[2], c.vals[3], c.vals[1],
            );
            0
        }
        0x021B_FF60 => {
            let mut cx = cx!(c, 0);
            super::release_partner(&mut c.env, &c.work, &mut cx, c.vals[1], c.vals[2]);
            0
        }
        0x021B_FD24 => {
            let mut cx = cx!(c, 0);
            super::run_effect(
                &mut c.env,
                &mut c.work,
                &mut cx,
                c.vals[1],
                c.vals[2],
                c.vals[3],
                c.vals[4],
            ) as u64
        }
        0x021B_FC08 => {
            // region 0 = the per-turn struct at work + 0x4A8, region 1 = the command queue
            let (p, q) = c.regions.split_at_mut(1);
            let mut cx = super::Cx {
                q: &mut q[0],
                scratch: &mut c.globals[0],
                w_addr: c.vals[0],
            };
            super::use_effect(
                &mut c.env,
                &mut c.work,
                &mut cx,
                &mut p[0],
                c.vals[1],
                c.vals[2],
                c.vals[3],
            ) as u64
        }
        0x021B_F3D8 => {
            super::list_apply(
                &mut c.env,
                &c.work,
                c.vals[2],
                c.vals[3],
                &mut c.bufs[4],
                c.vals[1],
                c.vals[0],
            );
            0
        }
        0x021B_F6E8 => {
            let mut cx = cx!(c, 0);
            super::run_event(
                &mut c.env,
                &mut c.work,
                &mut cx,
                (c.vals[2], c.vals[3]),
                c.vals[5],
                c.vals[1],
                c.vals[4],
            ) as u64
        }
        0x021B_F78C => {
            // bufs[1] = move record, bufs[5] = 24-word table, bufs[7] = the out word (stack arg 3)
            let (head, out) = c.bufs.split_at_mut(7);
            super::event_message(
                &mut c.env,
                &head[1],
                c.vals[2],
                (c.vals[3], c.vals[4]),
                &head[5],
                c.vals[6],
                &mut out[0],
            ) as u64
        }
        0x021B_F4AC => {
            let mut cx = cx!(c, 0);
            super::run_event_passes(
                &mut c.env,
                &mut c.work,
                &mut cx,
                &c.bufs[1],
                c.vals[2],
                c.vals[3],
                c.vals[1],
                c.vals[4],
            );
            0
        }
        0x021C_001C => {
            let mut cx = cx!(c, 0);
            super::ability_gate(
                &mut c.env,
                &mut c.work,
                &mut cx,
                c.vals[1],
                c.vals[2],
                c.vals[3],
            ) as u64
        }
        0x021C_0A4C => {
            let mut out = std::mem::take(&mut c.bufs[6]);
            let mv = super::Move {
                rec: &c.bufs[3],
                mon: c.vals[1],
                list: c.vals[2],
                table: &c.bufs[4],
                rec_addr: c.vals[3],
                table_addr: c.vals[4],
                waddr: c.vals[0],
            };
            super::fill_record(&mut c.env, &mv, c.vals[5], &mut out);
            c.bufs[6] = out;
            0
        }
        0x021C_0D90 => {
            // regions: 0 = the command queue, 1 / 2 = the damage records at work + 0x86C / 0x870
            let (q, recs) = c.regions.split_at_mut(1);
            let (r1, r2) = recs.split_at_mut(1);
            let mut cx = super::Cx {
                q: &mut q[0],
                scratch: &mut c.globals[0],
                w_addr: c.vals[0],
            };
            let mut records = super::Records {
                r1: &mut r1[0],
                r2: &mut r2[0],
            };
            let mut pos = std::mem::take(&mut c.bufs[5]);
            let mv = super::Move {
                rec: &c.bufs[1],
                mon: c.vals[2],
                list: c.vals[3],
                table: &c.bufs[4],
                rec_addr: c.vals[1],
                table_addr: c.vals[4],
                waddr: c.vals[0],
            };
            let r = super::build_records(
                &mut c.env,
                &mut c.work,
                &mut cx,
                &mut records,
                &mv,
                (&mut pos, c.vals[6]),
            );
            c.bufs[5] = pos;
            r as u64
        }
        0x021C_091C => {
            // regions: 0 = the per-turn struct at work + 0x4A8, 1 = the command queue, 2 / 3 =
            // the damage records at work + 0x86C / 0x870
            let (p, rest) = c.regions.split_at_mut(1);
            let (q, recs) = rest.split_at_mut(1);
            let (r1, r2) = recs.split_at_mut(1);
            let mut cx = super::Cx {
                q: &mut q[0],
                scratch: &mut c.globals[0],
                w_addr: c.vals[0],
            };
            let mut records = super::Records {
                r1: &mut r1[0],
                r2: &mut r2[0],
            };
            let mv = super::Move {
                rec: &c.bufs[1],
                mon: c.vals[2],
                list: c.vals[3],
                table: &c.bufs[4],
                rec_addr: c.vals[1],
                table_addr: c.vals[4],
                waddr: c.vals[0],
            };
            super::build_damage(
                &mut c.env,
                &mut c.work,
                &mut cx,
                &mut p[0],
                &mut records,
                &mv,
                c.vals[5],
            );
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/rom-diff/svflow.py core3`).
#[test]
fn svflow_match_rom() {
    run_area_suite(
        "core3",
        include_str!("../../../../../tools/rom-diff/svflow_specs/core3.spec"),
        super::PORTED,
        RECORDED,
        call,
    );
}
