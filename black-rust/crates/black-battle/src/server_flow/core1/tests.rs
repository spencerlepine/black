use crate::server_flow::testkit::{run_area_suite, Case};

// Written by tools/check-svflow.sh only, after the Rust hashes matched the ROM run.
// svflow-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021BB968, "ced73076feb0e580"),
    (0x021BB9D8, "c651fdc78135119f"),
    (0x021BBB48, "72d83e32191173ff"),
    (0x021BBB60, "499046381e7b3ab0"),
    (0x021BBC34, "52d5adaabdbf9830"),
    (0x021BBC44, "df60fb0d5b843504"),
    (0x021BBCD4, "cd6da3418b04da3b"),
    (0x021BBD20, "46a3fb0fcd764069"),
    (0x021BBD2C, "7c2307887f7785c8"),
    (0x021BBD80, "96d450a6b72fb6be"),
    (0x021BBD8C, "1df510dcb69b0f30"),
    (0x021BBFA8, "063c52fc0807d084"),
    (0x021BC0E8, "cbaffc4d3e5dfc34"),
    (0x021BC220, "01d8292d20078c28"),
    (0x021BC2B0, "c8099f3ee28d1386"),
    (0x021BC310, "c521941c7d00ea70"),
    (0x021BC360, "ac5189e0f11adaa1"),
    (0x021BC390, "8a98a1e8fd475a98"),
    (0x021BC3B0, "9aacffd3e5479e5a"),
    (0x021BC418, "aacb2f6be91d1848"),
    (0x021BC4DC, "2b1d3ec8d24d3a68"),
    (0x021BC550, "0cf189fbd1ec392f"),
    (0x021BC55C, "ec36a8b9dce3e162"),
    (0x021BC568, "8a51cabb877197e0"),
    (0x021BC610, "78338eaa09e65263"),
    (0x021BC874, "321acfe3f1af0799"),
    (0x021BC8EC, "9449c714b4872032"),
    (0x021BC948, "9becf8e0d65dac4a"),
    (0x021BCA08, "8cd56461e3eb2234"),
    (0x021BCA50, "b254da49246fa337"),
    (0x021BCAAC, "0688f87e70c43ba9"),
    (0x021BCB14, "4a79d649394b4b4f"),
    (0x021BCB68, "02d134dfc087b93b"),
    (0x021BCB98, "8851d65e6480ed19"),
    (0x021BCC50, "e7fc959ff391050c"),
    (0x021BCCE0, "8ea1d289a5ce4f3d"),
    (0x021BCEFC, "60db63b8a8436d05"),
    (0x021BCF34, "9570fd8ffc50a0ed"),
    (0x021BCF70, "cb62854729c2253f"),
    (0x021BCFAC, "a8cd05ca7ae4fc0b"),
    (0x021BD080, "4fed464d9dca7570"),
    (0x021BD0C0, "df01c65e81550e81"),
    (0x021BD148, "35751308c6846082"),
    (0x021BD170, "d02357c15a81a5f2"),
    (0x021BD1B0, "111e56f8099f1568"),
];
// svflow-recorded-end

/// Fixed address of the work image in the cases (`Case::vals`).
const WORK: u32 = 0x02E0_0000;

pub(crate) fn call<E: crate::server_flow::env::FlowEnv>(addr: u32, c: &mut Case<E>) -> u64 {
    use super::*;
    match addr {
        0x021B_C550 => super::struct_0x4b8(c.vals[0]) as u64,
        0x021B_CB68 => super::record_index(&c.work, c.vals[1].wrapping_sub(WORK)) as u64,
        0x021B_CA08 => super::record_by_mon(&mut c.env, &c.work, c.vals[1]).map_or(0, |o| o as u64),
        0x021B_C874 => {
            let base = (c.vals[0] - WORK) as usize;
            super::sort_records(&mut c.env, &mut c.work.mem[base..], c.vals[1]);
            0
        }
        0x021B_C4DC => {
            let list = (c.vals[1] - WORK) as usize;
            super::collect_list(&mut c.env, &mut c.work, &mut c.regions[0], list) as u64
        }
        0x021B_C55C => super::byte_784(&c.work) as u64,
        0x021B_BD20 => {
            super::clear_77c_a(&mut c.work);
            0
        }
        0x021B_BD80 => {
            super::clear_77c_b(&mut c.work);
            0
        }
        0x021B_BC34 => {
            super::clear_77c_783(&mut c.work);
            0
        }
        0x021B_C8EC => super::priority_bits(&mut c.env, c.vals[1], c.vals[2]) as u64,
        0x021B_C948 => super::speed_value(&mut c.env, c.vals[1], c.vals[2]) as u64,
        0x021B_CF34 => {
            super::event_1(&mut c.env, c.vals[1], c.vals[2]);
            0
        }
        0x021B_CEFC => {
            super::event_1_bracketed(&mut c.env, &mut c.work, c.vals[1], c.vals[2]);
            0
        }
        0x021B_CF70 => {
            super::event_2(&mut c.env, c.vals[1], c.vals[2]);
            0
        }
        0x021B_D080 => super::event_0e(&mut c.env, c.vals[1]) as u64,
        0x021B_C2B0 => super::some_team_unusable(&mut c.env, &c.work) as u64,
        0x021B_C310 => super::own_team_usable(&mut c.env, &c.work) as u64,
        0x021B_CAAC => find_pending(&mut c.env, &c.work, c.vals[0], c.vals[1], c.vals[2])
            .map_or(0, |o| o as u64),
        0x021B_CA50 => {
            next_record_by_mon(&mut c.env, &c.work, c.vals[1].wrapping_sub(WORK), c.vals[2])
                .map_or(0, |o| o as u64)
        }
        0x021B_CB14 => find_pending_alive(&mut c.env, &c.work, c.vals[0], c.vals[1], c.vals[2])
            .map_or(0, |o| o as u64),
        0x021B_CB98 => {
            super::move_record_up(&mut c.work, c.vals[1].wrapping_sub(WORK), c.vals[2] as i32)
                as u32 as u64
        }
        0x021B_CC50 => {
            super::move_record_last(&mut c.work, c.vals[1].wrapping_sub(WORK));
            0
        }
        0x021B_D148 => super::promote_record(&mut c.env, &mut c.work, c.vals[1]) as u64,
        0x021B_D170 => super::promote_pending(&mut c.env, &mut c.work, c.vals[0], c.vals[1]) as u64,
        0x021B_D1B0 => super::demote_record(&mut c.env, &mut c.work, c.vals[1]) as u64,
        0x021B_C0E8 => {
            super::refresh_priorities(
                &mut c.env,
                &mut c.work,
                c.vals[0],
                c.vals[1].wrapping_sub(WORK) as usize,
                c.vals[2],
            );
            0
        }
        0x021B_C220 => super::queue_switch_ins(&mut c.env, &c.regions[0], &c.bufs[1]) as u64,
        0x021B_C418 => {
            let (q, g) = (&mut c.regions[0], &mut c.globals[0]);
            super::announce_switch(&mut c.env, &c.work, q, g, c.vals[1], c.vals[2]);
            0
        }
        0x021B_D0C0 => {
            super::replace_action(&mut c.env, &mut c.work, WORK, c.vals[1], c.vals[2]) as u64
        }
        0x021B_C3B0 => {
            let (s, q) = c.regions.split_at_mut(1);
            super::push_client_counts(&mut c.env, &c.work, &s[0], &mut q[0], &mut c.globals[0]);
            0
        }
        0x021B_C360 => super::start_queue(&mut c.env, &c.work, &mut c.regions[0]) as u64,
        0x021B_C390 => {
            // the spec has the queue region only (work + 0x18 is never 1 there): no S object
            super::count_commands(&mut c.env, &c.work, &[], &mut c.regions[0], &mut [0; 0x40])
                as u64
        }
        0x021B_C568 => {
            super::apply_move_hooks(&mut c.env, &mut c.work, c.vals[0]);
            0
        }
        0x021B_CFAC => super::item_use_result(&mut c.env, &c.work, c.vals[1]) as u64,
        0x021B_CCE0 => super::execute_record(
            &mut c.env,
            &mut c.work,
            c.vals[0],
            c.vals[1].wrapping_sub(WORK) as usize,
        ) as u64,
        0x021B_C610 => super::build_action_list(
            &mut c.env,
            &mut c.work,
            c.vals[0],
            &c.regions[0],
            &c.bufs[1],
            c.vals[2].wrapping_sub(WORK) as usize,
        ) as u64,
        0x021B_BFA8 => {
            let mut none: [u8; 0] = [];
            let s = c.regions.get_mut(0).map_or(&mut none[..], |r| &mut r[..]);
            super::run_actions(&mut c.env, &mut c.work, c.vals[0], s, c.vals[1]) as u64
        }
        0x021B_BCD4 => {
            super::run_mon_list(&mut c.env);
            0
        }
        0x021B_BC44 => {
            let (s, q) = c.regions.split_at_mut(1);
            super::turn_start(&mut c.env, &mut c.work, c.vals[1], &mut s[0], &mut q[0]) as u64
        }
        0x021B_BD2C => {
            let (s, q) = c.regions.split_at_mut(1);
            super::turn_resume(&mut c.env, &mut c.work, &mut s[0], &mut q[0], c.vals[1]) as u64
        }
        0x021B_BD8C => {
            let (s, q) = c.regions.split_at_mut(1);
            super::build_and_run(
                &mut c.env,
                &mut c.work,
                c.vals[0],
                c.vals[1],
                &mut s[0],
                &mut q[0],
            ) as u64
        }
        0x021B_B968 => super::create(
            &mut c.env,
            &mut c.work,
            WORK,
            c.vals[0],
            c.vals[1],
            c.vals[2],
            c.vals[3],
            c.vals[4],
            c.vals[5] as u16,
        ) as u64,
        0x021B_B9D8 => {
            super::init_work(&mut c.env, &mut c.work);
            0
        }
        0x021B_BB48 => {
            super::destroy(&mut c.env, &c.work);
            0
        }
        0x021B_BB60 => {
            let (s, q) = c.regions.split_at_mut(1);
            super::queue_party_announce(&mut c.env, &c.work, c.vals[0], &s[0], &mut q[0]) as u64
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/rom-diff/svflow.py core1`).
#[test]
fn svflow_match_rom() {
    run_area_suite(
        "core1",
        include_str!("../../../../../tools/rom-diff/svflow_specs/core1.spec"),
        super::PORTED,
        RECORDED,
        call,
    );
}
