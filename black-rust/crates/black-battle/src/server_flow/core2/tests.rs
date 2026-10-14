use crate::server_flow::testkit::{run_area_suite, Case};

// Written by tools/check-svflow.sh only, after the Rust hashes matched the ROM run.
// svflow-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021BD1D0, "a76498ce5b87eb22"),
    (0x021BD1E0, "7ee292f45a0384fe"),
    (0x021BD200, "aa1423e72e14f15c"),
    (0x021BD2B4, "6b05a5fd8f99d118"),
    (0x021BD338, "c71ae22e058ad5b7"),
    (0x021BD34C, "6ba7a21cd165d5e1"),
    (0x021BD3E8, "feaa9d49f626f7fe"),
    (0x021BD43C, "85e283708f1b2be9"),
    (0x021BD4E4, "625c02b4ce0eea62"),
    (0x021BD518, "eba815011e8e9ff2"),
    (0x021BD554, "16f88e6892d035e6"),
    (0x021BD584, "54384778ba1ec4e9"),
    (0x021BD60C, "4ed3d9447e85bf31"),
    (0x021BD6B8, "a5638a9cf466c7d1"),
    (0x021BD708, "45acf9d1bb72b746"),
    (0x021BD850, "575780c5fa967a8e"),
    (0x021BD8AC, "e8c529ffc5232d19"),
    (0x021BD8F0, "003f22076c018442"),
    (0x021BD9D4, "c96d199870bcb04e"),
    (0x021BDA00, "9accb6045d4fae42"),
    (0x021BDB88, "e988acad5c65a58e"),
    (0x021BDBB8, "8da79425271246bd"),
    (0x021BDBEC, "52ce2d9a25902725"),
    (0x021BDC0C, "7445abd93e3598fd"),
    (0x021BDC2C, "8538b1b2a24a0cd3"),
    (0x021BDC58, "26ca5b818fad740e"),
    (0x021BDC78, "dd5ca67bf9b8f325"),
    (0x021BDC98, "370110359a7f55c0"),
    (0x021BDD00, "7517be9d0d718a2a"),
    (0x021BDD7C, "1f5d3b17ceec453b"),
    (0x021BDE18, "37f6f19e5cda10cb"),
    (0x021BDE48, "76534f0001d2033e"),
    (0x021BDE98, "6592168b38087e39"),
    (0x021BE3A8, "c5e5265eb843cc90"),
    (0x021BE400, "a089a2b1f95090bf"),
    (0x021BE518, "0f3dbd3610562d9b"),
    (0x021BE52C, "6af6c6de90b7811b"),
    (0x021BE550, "471c7e1c0e4bd459"),
    (0x021BE620, "27e57c15a3f69532"),
    (0x021BE66C, "45be133c517b00eb"),
    (0x021BE6A8, "8d5c04cc1b2669b6"),
    (0x021BE6EC, "5d9d1ea966ad9541"),
    (0x021BE7E4, "0da7fa307f0c4ae3"),
    (0x021BE878, "4e863408e7248c5b"),
    (0x021BE8C4, "d73efe1a06a0c5b4"),
    (0x021BE8F4, "09b462c08fc78f91"),
    (0x021BE924, "39b0b83464df3cd9"),
    (0x021BE95C, "4104765148be7b79"),
    (0x021BE994, "32fce93441c74a78"),
    (0x021BE9D0, "c0897d257f99617b"),
    (0x021BEA14, "5231ea887a167ab7"),
    (0x021BEA60, "a06a9720bdc62600"),
    (0x021BEAB4, "f831b3e98b648242"),
    (0x021BEB14, "8ce20b7dc4371096"),
    (0x021BEBD8, "f931bc69cdadb667"),
    (0x021BEC08, "64a1353470d38e94"),
    (0x021BEC2C, "fb7c099c4d4cff8a"),
    (0x021BEC58, "50b922b12580d66c"),
    (0x021BF04C, "c315ea02ee3756eb"),
    (0x021BF0E4, "6d0a743df54c8cea"),
    (0x021BF168, "32de834e94fb7b9b"),
    (0x021BF1B4, "9aa8661bcde07ae2"),
    (0x021BF244, "f1c82aa54b1e7b28"),
    (0x021BF27C, "10ed4bcc786e0658"),
    (0x021BF2B4, "fe4672b1315c2019"),
    (0x021BF2EC, "ba68f070235c16e4"),
];
// svflow-recorded-end

/// The command queue context: region 0 = the queue buffer, global 0 = the scratch words at
/// 0x021F3B44 (a macro so the borrows of `c.env` / `c.work` stay disjoint).
macro_rules! cx {
    ($c:ident) => {
        super::Cx {
            q: &mut $c.regions[0],
            scratch: &mut $c.globals[0],
            w_addr: $c.vals[0],
        }
    };
}

pub(crate) fn call<E: crate::server_flow::env::FlowEnv>(addr: u32, c: &mut Case<E>) -> u64 {
    match addr {
        0x021B_EC08 => super::mon_is_ours(&mut c.env, &c.work, c.vals[1]) as u64,
        0x021B_EC2C => {
            super::notify_special_id(&mut c.env, &c.work, c.vals[1], c.vals[2]);
            0
        }
        0x021B_DC78 => {
            // region 0 = the command queue buffer, global 0 = the scratch words at 0x021F3B44
            super::put_mon_client_cmd(&mut c.env, &mut c.regions[0], &mut c.globals[0], c.vals[1]);
            0
        }
        0x021B_E8C4 => {
            super::queue_wrap_mon_builder(&mut c.env, &mut c.work, c.vals[1]);
            0
        }
        0x021B_D1D0 => {
            super::mark_done(&mut c.env, &mut c.work, c.vals[1]);
            0
        }
        0x021B_D1E0 => {
            super::reset_record(&mut c.bufs[0]);
            0
        }
        0x021B_E518 => {
            super::list_clear(&mut c.bufs[0]);
            0
        }
        0x021B_E52C => {
            let (a, b, d) = (c.vals[1] as u8, c.vals[2] as u8, c.vals[3] as u8);
            super::list_push(&mut c.bufs[0], a, b, d);
            0
        }
        0x021B_D338 => {
            let main = c.bufs[1].clone();
            super::set_flag_if_type3(&mut c.env, &mut c.bufs[0], &main);
            0
        }
        0x021B_DBEC => {
            super::build_bdbec(&mut c.env);
            0
        }
        0x021B_DC0C => {
            super::build_bdc0c(&mut c.env);
            0
        }
        0x021B_DC58 => {
            super::build_bdc58(&mut c.env);
            0
        }
        0x021B_D554 => {
            super::build_bd554(&mut c.env, c.vals[1]);
            0
        }
        0x021B_DB88 => {
            super::build_bdb88(&mut c.env, c.vals[1]);
            0
        }
        0x021B_DE18 => {
            super::build_bde18(&mut c.env, c.vals[1]);
            0
        }
        0x021B_DBB8 => {
            super::build_bdbb8(&mut c.env, c.vals[1]);
            0
        }
        0x021B_E8F4 => {
            super::build_be8f4(&mut c.env, c.vals[1]);
            0
        }
        0x021B_F2EC => {
            super::build_bf2ec(&mut c.env, c.vals[1], c.vals[2], c.vals[3]);
            0
        }
        0x021B_E95C => {
            super::build_be95c(&mut c.env, c.vals[1], c.vals[2]);
            0
        }
        0x021B_D8AC => super::build_bd8ac(&mut c.env, c.vals[1]) as u64,
        0x021B_E6A8 => {
            super::build_be6a8(&mut c.env, c.vals[1], &c.bufs[2], c.vals[3]);
            0
        }
        0x021B_E9D0 => {
            super::build_be9d0(&mut c.env, c.vals[1], c.vals[2], c.vals[3]);
            0
        }
        0x021B_E620 => {
            super::build_be620(&mut c.env, c.vals[1], &mut c.bufs[2]);
            0
        }
        0x021B_EA14 => {
            super::build_bea14(&mut c.env, c.vals[1], c.vals[2], c.vals[3]);
            0
        }
        0x021B_E878 => super::build_be878(&mut c.env, c.vals[1], c.vals[2]) as u64,
        0x021B_F168 => super::build_bf168(&mut c.env, &c.bufs[1], c.vals[2]) as u64,
        0x021B_D6B8 => super::build_bd6b8(&mut c.env, c.vals[1]) as u64,
        0x021B_EA60 => {
            super::build_bea60(&mut c.env, c.vals[1], c.vals[2], c.vals[3]);
            0
        }
        0x021B_D850 => super::build_bd850(&mut c.env, c.vals[1]) as u64,
        0x021B_EB14 => {
            let (lo, hi) = c.bufs.split_at_mut(5);
            super::build_beb14(
                &mut c.env,
                c.vals[1],
                c.vals[2],
                c.vals[3],
                &mut lo[4][0],
                &mut hi[0][0],
            ) as u64
        }
        0x021B_E924 => {
            super::queue_wrap_be95c(&mut c.env, &mut c.work, c.vals[1], c.vals[2]);
            0
        }
        0x021B_DC2C => {
            super::queue_wrap_bdc58(&mut c.env, &mut c.work);
            0
        }
        0x021B_D518 => {
            super::queue_wrap_bd554(&mut c.env, &mut c.work, c.vals[1]);
            0
        }
        0x021B_E66C => {
            super::queue_wrap_be6a8(&mut c.env, &mut c.work, c.vals[1], &c.bufs[2], c.vals[3]);
            0
        }
        0x021B_E994 => {
            super::queue_wrap_be9d0(&mut c.env, &mut c.work, c.vals[1], c.vals[2], c.vals[3]);
            0
        }
        0x021B_F244 => {
            super::queue_wrap_bf244(&mut c.env, &mut c.work, c.vals[1], c.vals[2]);
            0
        }
        0x021B_F27C => {
            super::queue_wrap_bf27c(&mut c.env, &mut c.work, c.vals[1], c.vals[2]);
            0
        }
        0x021B_F2B4 => {
            super::queue_wrap_bf2b4(&mut c.env, &mut c.work, c.vals[1], c.vals[2]);
            0
        }
        0x021B_D2B4 => {
            super::iter_first(&mut c.env, &mut c.bufs[0], &c.regions[0]);
            0
        }
        0x021B_D34C => {
            let (lo, hi) = c.bufs.split_at_mut(2);
            super::iter_next(&mut c.env, &mut lo[0], &c.regions[0], &mut hi[0]) as u64
        }
        0x021B_D200 => {
            super::fill_switch_record(&mut c.env, &mut c.bufs[0], c.vals[1], c.vals[2], c.vals[3]);
            0
        }
        0x021B_D43C => {
            let mut cx = cx!(c);
            super::swap_party_slots(
                &mut c.env,
                &c.work,
                &mut cx,
                c.vals[1],
                (c.vals[2], c.vals[3]),
                c.vals[4],
            );
            0
        }
        0x021B_D4E4 => {
            super::announce_pair(&mut c.env, &mut c.work, c.vals[1], c.vals[2], c.vals[3]);
            0
        }
        0x021B_D3E8 => {
            super::switch_slot_front(&mut c.env, &mut c.work, c.vals[1]);
            0
        }
        0x021B_D8F0 => {
            let mut cx = cx!(c);
            super::switch_in(
                &mut c.env,
                &mut c.work,
                &mut cx,
                c.vals[1],
                c.vals[2],
                c.vals[3],
            );
            0
        }
        0x021B_D9D4 => {
            let mut cx = cx!(c);
            super::switch_in_cmd(
                &mut c.env,
                &mut c.work,
                &mut cx,
                c.vals[1],
                c.vals[2],
                c.vals[3],
                c.vals[4],
            );
            0
        }
        0x021B_EBD8 => {
            let mut cx = cx!(c);
            super::put_cmd_30(
                &mut c.env, &c.work, &mut cx, c.vals[1], c.vals[2], c.vals[3],
            );
            0
        }
        0x021B_DA00 => super::send_client_messages(&mut c.env, &mut c.work, c.vals[0]) as u64,
        0x021B_DD7C => {
            let mut cx = cx!(c);
            super::faint_message(&mut c.env, &mut c.work, &mut cx, c.vals[1], c.vals[2]);
            0
        }
        0x021B_DD00 => super::faint_if_alive(&mut c.env, &mut c.work, c.vals[1], c.vals[2]) as u64,
        0x021B_DC98 => {
            super::faint_notify(&mut c.env, &mut c.work, c.vals[1], c.vals[2]);
            0
        }
        0x021B_D708 => {
            // no scratch words in this spec: the function only puts variable-length commands
            let mut cx = super::Cx {
                q: &mut c.regions[0],
                scratch: &mut [],
                w_addr: c.vals[0],
            };
            super::switch_messages(
                &mut c.env,
                &mut c.work,
                &mut cx,
                c.vals[1],
                c.vals[2],
                c.vals[3],
            ) as u64
        }
        0x021B_D60C => super::can_switch_out(&mut c.env, &mut c.work, c.vals[1], c.vals[2]) as u64,
        0x021B_D584 => {
            super::switch_out_notice(&mut c.env, &mut c.work, &mut c.regions[0], c.vals[1]) as u64
        }
        0x021B_DE48 => {
            let word = u32::from_le_bytes(c.bufs[1][..4].try_into().unwrap());
            super::replacement_move(&mut c.env, c.vals[0], word) as u64
        }
        0x021B_E3A8 => super::side_effect(&mut c.env, c.vals[1], &c.bufs[2]) as u64,
        0x021B_E400 => {
            let (l, z) = c.regions.split_at_mut(1);
            super::run_pending_list(
                &mut c.env,
                &mut c.work,
                c.vals[0],
                &l[0],
                &mut z[0],
                c.vals[1],
            );
            0
        }
        0x021B_E550 => {
            super::run_pending_one(
                &mut c.env,
                &mut c.work,
                c.vals[0],
                &c.regions[0],
                c.vals[1],
                c.vals[2],
            );
            0
        }
        0x021B_EAB4 => super::list_mons_reply(
            &mut c.env,
            c.vals[0],
            c.vals[1],
            c.vals[2],
            c.vals[3],
            &mut c.bufs[4],
        ) as u64,
        0x021B_E6EC => super::swap_with_pending(
            &mut c.env,
            &mut c.work,
            c.vals[0],
            &mut c.regions[0],
            c.vals[1],
            c.vals[2],
        ) as u64,
        0x021B_E7E4 => {
            let mut cx = cx!(c);
            super::flag_mon_effect(
                &mut c.env,
                &mut c.work,
                &mut cx,
                c.vals[1],
                c.vals[2],
                c.vals[3],
                &mut c.bufs[4][0],
            ) as u64
        }
        0x021B_F04C => {
            super::apply_to_targets(&mut c.env, &mut c.work, &c.bufs[1], c.vals[2], c.vals[3]);
            0
        }
        0x021B_F0E4 => {
            super::add_first_target(&mut c.env, c.vals[1], c.vals[2], &c.bufs[3], c.vals[4]);
            0
        }
        0x021B_F1B4 => {
            super::add_other_targets(&mut c.env, &c.bufs[1], c.vals[2], c.vals[3], c.vals[4]);
            0
        }
        0x021B_EC58 => {
            // region 0 = queue, region 1 = the flag struct at work + 0x4A8; the spec declares no
            // scratch words (the placeholder command only writes zeros there)
            let (q, p) = c.regions.split_at_mut(1);
            let mut scratch = [0u8; 0x40];
            let mut cx = super::Cx {
                q: &mut q[0],
                scratch: &mut scratch,
                w_addr: c.vals[0],
            };
            super::apply_move_category(
                &mut c.env,
                &mut c.work,
                &mut cx,
                &mut p[0],
                c.vals[1],
                c.vals[2],
                c.vals[3],
            ) as u64
        }
        0x021B_DE98 => {
            // regions: 0 = queue, 1 = record D (work + 0x1AB0), 2 = record E (work + 0x1AB4)
            let (q, rest) = c.regions.split_at_mut(1);
            let (d, e) = rest.split_at_mut(1);
            let mut cx = super::Cx {
                q: &mut q[0],
                scratch: &mut c.globals[0],
                w_addr: c.vals[0],
            };
            let mut m = super::MoveCtx {
                d: &mut d[0],
                e: &mut e[0],
                word: &mut c.bufs[2],
                word_addr: c.vals[2],
            };
            super::use_move(
                &mut c.env,
                &mut c.work,
                &mut cx,
                &mut m,
                c.vals[1],
                c.vals[3],
            );
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/rom-diff/svflow.py core2`).
#[test]
fn svflow_match_rom() {
    run_area_suite(
        "core2",
        include_str!("../../../../../tools/rom-diff/svflow_specs/core2.spec"),
        super::PORTED,
        RECORDED,
        call,
    );
}
