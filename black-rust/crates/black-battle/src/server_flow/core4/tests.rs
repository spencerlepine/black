use crate::server_flow::testkit::{run_area_suite, Case};

// Written by tools/check-svflow.sh only, after the Rust hashes matched the ROM run.
// svflow-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021C0F24, "9fe75b074dba66a5"),
    (0x021C1184, "5a2342a60f79dd94"),
    (0x021C1194, "44e752a7b4435b17"),
    (0x021C1198, "8397da029daef106"),
    (0x021C11A4, "29bdfeb39c968851"),
    (0x021C11F0, "b33b6c162e3b023d"),
    (0x021C14E0, "fa7b2ee165620439"),
    (0x021C15AC, "6cdfdb2f0fd50a67"),
    (0x021C15D4, "20ec4174e968d506"),
    (0x021C15E8, "def65b3f487a03b4"),
    (0x021C1624, "378e80a68cb3a776"),
    (0x021C1678, "40841b9eaa499666"),
    (0x021C16F8, "aaf527430e1bdaa6"),
    (0x021C1758, "ac2ef2dc19c07917"),
    (0x021C1788, "1a37558937a84040"),
    (0x021C17CC, "e6e28083909698fa"),
    (0x021C1828, "6c26f66e5f7d71bb"),
    (0x021C1880, "4ca97f3e8d39cabb"),
    (0x021C18D4, "f08dab81d60bb243"),
    (0x021C1910, "58bbe6796cbb7759"),
    (0x021C195C, "0d67020102e1b7f7"),
    (0x021C199C, "1a341a9172a1cd73"),
    (0x021C19D8, "8e421db91b313341"),
    (0x021C1A10, "985e83ef6d64f436"),
    (0x021C1A54, "793477aee324580d"),
    (0x021C1B5C, "863bbecec6666e0d"),
    (0x021C1C88, "c939df9b4628ffe8"),
    (0x021C1CCC, "2656fe7e533eb8d7"),
    (0x021C1CE4, "6b3e0909df3b9c74"),
    (0x021C1D60, "5a2bb5732466eba2"),
    (0x021C1DCC, "49fcf3820148ff7a"),
    (0x021C1E14, "a4b2431e2546ecc0"),
    (0x021C1E74, "e3a59fe783135d29"),
    (0x021C2048, "f49232de4490f65f"),
    (0x021C20BC, "8f855ab43513cfb9"),
    (0x021C20D0, "898c2de0dd42558b"),
    (0x021C2128, "82f3b357f5724cd4"),
    (0x021C2204, "8b70d9af1f9a4bda"),
    (0x021C2250, "ef08a9387cf452e2"),
    (0x021C2314, "ce68ec4733a066c8"),
    (0x021C232C, "ebb0d08a6d741911"),
    (0x021C236C, "ff398c0b15178f10"),
    (0x021C2488, "dfaa99da03c0ce93"),
    (0x021C25A4, "b86493d3390ecfbf"),
    (0x021C264C, "61df927b8bfa6dcf"),
    (0x021C26A8, "84c9bf35a0217787"),
    (0x021C26FC, "be554202b6077dfa"),
    (0x021C2758, "a72c705965766caa"),
    (0x021C27A0, "0606ce2ea76432a2"),
    (0x021C282C, "6e98fb243bbafe47"),
    (0x021C2910, "437c206e4e98190f"),
    (0x021C2AB0, "bfbe38be278a842c"),
    (0x021C2AF0, "9e7dd30644e00256"),
    (0x021C2B3C, "2ffc18d1e147897e"),
    (0x021C2BA4, "b85572a0e5d7334a"),
    (0x021C2BF0, "5878efdc65c87e52"),
    (0x021C2C50, "fa765de8b70d9651"),
    (0x021C2CA8, "6b639f0fa702caf6"),
    (0x021C2CD8, "605879b156478806"),
    (0x021C2D10, "c7b7cc1eb90c73f6"),
    (0x021C2DC4, "5c30e082673f147a"),
    (0x021C2E4C, "3746b51c64a8eda9"),
    (0x021C2E64, "3e0facc08bd3d6e9"),
    (0x021C2F70, "182ddd5e170b85d0"),
    (0x021C3008, "cc0a75f74fe3d93e"),
    (0x021C312C, "9c738e500cd4fe28"),
    (0x021C3184, "309f679a6cdf3a7f"),
    (0x021C3274, "e92bfb9bb8c33685"),
    (0x021C334C, "258a45f77fca070a"),
    (0x021C3370, "2e6b0762a9723666"),
    (0x021C3394, "933dc941bdb4d38a"),
    (0x021C33E8, "60f71f9b95ff7a41"),
    (0x021C3438, "c5ef420a5a277ae8"),
    (0x021C36A0, "8edc88a05e099ae5"),
    (0x021C36E8, "be912df58699babc"),
    (0x021C3720, "6aa9196ceb73ef68"),
    (0x021C376C, "4e92a5d74e1427b9"),
    (0x021C3810, "5aa8846bf122c5f5"),
    (0x021C39CC, "2c0eabd1ff14c7a8"),
    (0x021C39F4, "2302ecff05a8ce91"),
    (0x021C3A4C, "106ea46cf66949b5"),
    (0x021C3AA8, "4b7ae93bcac54b74"),
    (0x021C3B68, "c190027c956c704e"),
    (0x021C3B8C, "6a75fb7c028b7b94"),
    (0x021C3BB8, "400c916cfe5d17e2"),
    (0x021C3BE0, "f1cd8ae0363df250"),
    (0x021C3C10, "129bf36bfe87cc4e"),
    (0x021C3C60, "70b3bbc8967facd2"),
    (0x021C3CCC, "8f4efccf2c8405db"),
    (0x021C3D08, "e2b67516ddd6bdbd"),
    (0x021C3E0C, "3ab40c9a9d3f8d6e"),
    (0x021C3FDC, "5906b52c6a7a3402"),
    (0x021C4034, "9c8227496e661204"),
    (0x021C4104, "da9640aa19a71033"),
    (0x021C41C0, "08b7c00cdb523cea"),
    (0x021C4218, "3e2f2eca9fe0b248"),
    (0x021C4268, "878f5ddb20b5dc06"),
    (0x021C4304, "5004f0068a13ee29"),
    (0x021C43F0, "c09c118e2c248aaf"),
    (0x021C446C, "fc73bea04380c9d5"),
    (0x021C4614, "f6a64056b4dfdaa8"),
    (0x021C4744, "e136f92c4b916e5d"),
];
// svflow-recorded-end

/// The command queue context: region `$q` = the queue buffer, global 0 = the put_cmd scratch
/// (a macro so the borrows of `c.env` / `c.work` stay disjoint). `$gb` = offset of the scratch
/// words (0x021F3B44) inside global 0.
macro_rules! cx {
    ($c:ident, $q:expr, $gb:expr) => {
        super::Cx {
            q: &mut $c.regions[$q],
            g: &mut $c.globals[0],
            gb: $gb,
            w_addr: $c.vals[0],
        }
    };
}

pub(crate) fn call<E: crate::server_flow::env::FlowEnv>(addr: u32, c: &mut Case<E>) -> u64 {
    let v = c.vals.clone();
    match addr {
        0x021C_1184 => super::byte1_is_zero(&c.bufs[0]) as u64,
        0x021C_2E4C => super::counter_next(&mut c.work) as u64,
        0x021C_15AC => {
            // region 0 = the command queue buffer (work + 0xC)
            super::level_message(&mut c.env, &mut c.regions[0], v[1] as i32);
            0
        }
        0x021C_15E8 => {
            super::record_bracket_1624(&mut c.env, &mut c.work, v[1], v[2], &c.bufs[3]);
            0
        }
        0x021C_20BC => super::effect_applies(&mut c.env, v[1]) as u64,
        0x021C_1194 => super::byte3(&c.bufs[0]) as u64,
        0x021C_1198 => {
            super::set_byte5_if_3(&mut c.bufs[0], v[1]);
            0
        }
        0x021C_14E0 => {
            super::list_apply_6b5c(&mut c.env, &mut c.work, &mut c.regions[0], v[1], v[3], v[2]);
            0
        }
        0x021C_15D4 => super::min_with_hp(&mut c.env, v[0], v[1]) as u64,
        0x021C_1624 => {
            super::msg_1624(&mut c.env, v[1], v[2], &c.bufs[3]);
            0
        }
        0x021C_1678 => super::hp_gate(&mut c.env, v[1], v[2], v[3], &mut c.bufs[4]) as u64,
        0x021C_16F8 => {
            super::msg_16f8(&mut c.env, &mut c.work, &mut c.regions[0], v[1], v[2]);
            0
        }
        0x021C_1758 => {
            super::msg_1758(&mut c.env, v[1]);
            0
        }
        0x021C_1788 => {
            super::record_bracket_17cc(&mut c.env, &mut c.work, v[2], v[3], &c.bufs[4]);
            0
        }
        0x021C_17CC => {
            super::msg_17cc(&mut c.env, v[2], v[3], &c.bufs[4]);
            0
        }
        0x021C_1828 => {
            let mut cx = cx!(c, 0, 0);
            super::put_move_used(&mut c.env, &mut cx, v[1], v[2], v[3], &c.bufs[4], v[5]);
            0
        }
        0x021C_18D4 => {
            super::record_bracket_1910(&mut c.env, &mut c.work, v[1], &c.bufs[2], v[3]);
            0
        }
        0x021C_1910 => {
            super::msg_1910(&mut c.env, v[1], &c.bufs[2], v[3]);
            0
        }
        0x021C_195C => {
            super::record_bracket_199c(&mut c.env, &mut c.work, v[1], v[2]);
            0
        }
        0x021C_199C => {
            super::msg_199c(&mut c.env, v[1], v[2]);
            0
        }
        0x021C_19D8 => {
            super::record_bracket_1a10(&mut c.env, &mut c.work, v[1], &c.bufs[2]);
            0
        }
        0x021C_1A10 => {
            super::msg_1a10(&mut c.env, v[1], &c.bufs[2]);
            0
        }
        0x021C_1A54 => {
            super::post_hit_messages(&mut c.env, &mut c.work, &c.bufs[1], v[2], v[3], v[5]);
            0
        }
        0x021C_1B5C => {
            super::msg_1b5c(&mut c.env, v[1], v[2], &c.bufs[3], v[4], v[5], v[6]);
            0
        }
        0x021C_1C88 => {
            super::msg_1c88(&mut c.env, v[1], &c.bufs[3], v[4]);
            0
        }
        0x021C_1CCC => {
            super::move_chance_gate(&mut c.env, &mut c.work, &c.bufs[1], v[2], v[3]) as u64
        }
        0x021C_1CE4 => {
            super::list_apply_557c(&mut c.env, &c.bufs[1], v[3]);
            0
        }
        0x021C_1D60 => super::chance_gate(&mut c.env, &mut c.work, v[1], v[2]) as u64,
        0x021C_1DCC => {
            let mut cx = cx!(c, 0, 0);
            super::category8_followup(
                &mut c.env,
                &mut c.work,
                &mut cx,
                &c.bufs[1],
                v[2],
                v[3],
                v[4],
            );
            0
        }
        0x021C_1E14 => {
            let mut cx = cx!(c, 0, 0);
            super::pick_and_apply(&mut c.env, &mut c.work, &mut cx, v[1], v[2], v[3]) as u64
        }
        0x021C_1E74 => {
            let (a, b) = c.bufs.split_at_mut(8);
            super::base_damage(
                &mut c.env, &c.work, v[1], v[2], &a[3], v[3], v[4], v[5], v[6], v[7], &mut b[0],
            ) as u64
        }
        0x021C_2048 => {
            super::try_damage_effect(&mut c.env, &mut c.work, v[1], v[2], v[3]);
            0
        }
        0x021C_20D0 => super::apply_amount(&mut c.env, &mut c.work, v[1], v[2], v[3] != 0) as u64,
        0x021C_2128 => {
            let mut cx = cx!(c, 0, 0);
            super::placeholder_effect(&mut c.env, &mut c.work, &mut cx, v[1]) as u64
        }
        0x021C_2204 => super::event_90(&mut c.env, v[1], v[2]) as u64,
        0x021C_2250 => {
            let mut cx = cx!(c, 0, 0);
            super::mon_state_change(&mut c.env, &mut c.work, &mut cx, v[1], v[2], v[3]);
            0
        }
        0x021C_2314 => {
            super::run_5be4_4f64(&mut c.env, v[1]);
            0
        }
        0x021C_2AB0 => super::event_7a(&mut c.env) as u64,
        0x021C_2AF0 => super::msg_2af0(&mut c.env, v[1]) as u64,
        0x021C_2B3C => super::msg_2b3c(&mut c.env, v[1], v[2], v[3]) as u64,
        0x021C_2BA4 => {
            super::msg_2ba4(&mut c.env, v[1], v[2], v[3]);
            0
        }
        0x021C_2BF0 => {
            super::msg_2bf0(&mut c.env, v[1], v[2], v[3], v[4]);
            0
        }
        0x021C_2C50 => {
            super::msg_2c50(&mut c.env, v[1], v[2], v[3]);
            0
        }
        0x021C_2CA8 => {
            super::msg_2ca8(&mut c.env, v[1]);
            0
        }
        0x021C_2D10 => super::event_51_roll(&mut c.env, &c.work, &c.bufs[1], v[2], v[3]) as u64,
        0x021C_236C => {
            super::effect_roll(&mut c.env, &c.work, v[1], v[2], v[3], &mut c.bufs[4]) as u64
        }
        0x021C_2488 => {
            let (r0, r1) = c.regions.split_at_mut(1);
            let mut cx = super::Cx {
                q: &mut r0[0],
                g: &mut c.globals[0],
                gb: 0,
                w_addr: v[0],
            };
            super::effect_each(
                &mut c.env,
                &mut c.work,
                &mut cx,
                &mut r1[0],
                v[1],
                v[2],
                v[3],
            );
            0
        }
        0x021C_25A4 => {
            let mut cx = cx!(c, 0, 0);
            super::effect_round(
                &mut c.env,
                &mut c.work,
                &mut cx,
                v[1],
                v[2],
                v[3],
                v[4],
                v[5],
                v[6],
            ) as u64
        }
        0x021C_264C => super::msg_264c(&mut c.env, v[1], v[2], v[3]) as u64,
        0x021C_26A8 => {
            super::msg_26a8(&mut c.env, v[1], v[2], v[3], v[4]);
            0
        }
        0x021C_26FC => {
            super::msg_26fc(&mut c.env, v[1], v[2], v[3], &mut c.bufs[4]);
            0
        }
        0x021C_2758 => {
            let mut cx = cx!(c, 0, 0);
            super::check_then_fail(
                &mut c.env,
                &mut c.work,
                &mut cx,
                v[1],
                v[2],
                v[3],
                v[4],
                v[5],
                v[6],
            ) as u64
        }
        0x021C_27A0 => {
            super::immunity_or_message(&mut c.env, &mut c.work, v[1], v[2], v[3], v[4], v[5], v[6])
                as u64
        }
        0x021C_282C => super::immunity_check(&mut c.env, v[1], v[2], v[3], v[4]) as u64,
        0x021C_2910 => {
            let mut cx = cx!(c, 0, 0);
            super::failure_message(
                &mut c.env,
                &mut c.work,
                &mut cx,
                v[1],
                v[2],
                v[3],
                v[4],
                v[5],
                v[6],
                v[7],
            );
            0
        }
        0x021C_2DC4 => {
            let (r0, r1) = c.regions.split_at_mut(1);
            let mut cx = super::Cx {
                q: &mut r0[0],
                g: &mut c.globals[0],
                gb: 0,
                w_addr: v[0],
            };
            super::multi_hit_each(
                &mut c.env,
                &mut c.work,
                &mut cx,
                &mut r1[0],
                &c.bufs[1],
                v[2],
                v[3],
            );
            0
        }
        0x021C_2E64 => {
            let mut cx = cx!(c, 0, 0);
            super::multi_hit(
                &mut c.env,
                &mut c.work,
                &mut cx,
                &c.bufs[1],
                v[2],
                v[3],
                v[4],
            ) as u64
        }
        0x021C_2F70 => {
            let (a, b) = c.bufs.split_at_mut(6);
            super::hit_event(&mut c.env, v[1], v[2], v[3], v[4], &mut a[5], &mut b[0]);
            0
        }
        0x021C_3008 => {
            let mut cx = cx!(c, 0, 0);
            super::hit_gate(
                &mut c.env,
                &mut c.work,
                &mut cx,
                v[1],
                v[2],
                v[3],
                v[4],
                v[5],
                v[6],
                v[7],
                v[8],
                v[9],
            ) as u64
        }
        0x021C_312C => super::msg_312c(&mut c.env, v[1], v[2], v[3], v[5]) as u64,
        0x021C_3FDC => {
            super::notify_others(&mut c.env, &mut c.work, v[1]);
            0
        }
        0x021C_232C => {
            let mut cx = cx!(c, 0, 0);
            super::effect_gate(&mut c.env, &mut c.work, &mut cx, &c.bufs[1], v[2], v[3]);
            0
        }
        0x021C_2CD8 => {
            let mut cx = cx!(c, 0, 0);
            super::multi_hit_if_rolled(&mut c.env, &mut c.work, &mut cx, &c.bufs[1], v[2], v[3]);
            0
        }
        0x021C_1880 => {
            let mut cx = cx!(c, 0, 0);
            super::category_dispatch(
                &mut c.env,
                &mut c.work,
                &mut cx,
                &c.bufs[1],
                v[2],
                v[3],
                v[5],
            );
            0
        }
        0x021C_0F24 => {
            let (r0, rest) = c.regions.split_at_mut(1);
            let (r1, r2) = rest.split_at_mut(1);
            let mut cx = super::Cx {
                q: &mut r0[0],
                g: &mut c.globals[0],
                gb: 0x24,
                w_addr: v[0],
            };
            super::apply_move_to_targets(
                &mut c.env,
                &mut c.work,
                &mut cx,
                &mut r1[0],
                &mut r2[0],
                &c.bufs[1],
                v[2],
                v[3],
                v[1],
                v[4],
            ) as u64
        }
        0x021C_11A4 => {
            let mut cx = cx!(c, 0, 0x24);
            let (a, b) = c.bufs.split_at_mut(5);
            super::damage_with_log(
                &mut c.env,
                &mut c.work,
                &mut cx,
                &a[1],
                v[2],
                v[4],
                &mut b[0],
                v[7],
                v[1],
            ) as u64
        }
        0x021C_11F0 => {
            let (a, b) = c.bufs.split_at_mut(5);
            let mut cx = cx!(c, 0, 0x24);
            super::per_target_damage(
                &mut c.env,
                &mut c.work,
                &mut cx,
                v[1],
                &a[4],
                &mut b[0],
                v[7],
                v[3],
                v[4],
            ) as u64
        }
        0x021C_3184 => {
            let (r0, r1) = c.regions.split_at_mut(1);
            let mut cx = super::Cx {
                q: &mut r0[0],
                g: &mut c.globals[0],
                gb: 0,
                w_addr: v[0],
            };
            super::effect_each_with_hits(
                &mut c.env,
                &mut c.work,
                &mut cx,
                &mut r1[0],
                &c.bufs[1],
                v[2],
                v[3],
            );
            0
        }
        0x021C_3274 => {
            let (r0, r1) = c.regions.split_at_mut(1);
            let mut cx = super::Cx {
                q: &mut r0[0],
                g: &mut c.globals[0],
                gb: 0,
                w_addr: v[0],
            };
            super::each_apply_79e0(&mut c.env, &mut c.work, &mut cx, &mut r1[0], v[1], v[3]);
            0
        }
        0x021C_334C => {
            let mut cx = cx!(c, 0, 0);
            super::gate_then_apply(&mut c.env, &mut c.work, &mut cx, v[1], v[2], v[3]) as u64
        }
        0x021C_3370 => super::mon_unavailable(&mut c.env, v[1]) as u64,
        0x021C_3394 => {
            let mut cx = cx!(c, 0, 0);
            super::flag_f_gate(&mut c.env, &mut c.work, &mut cx, v[1], v[2]) as u64
        }
        0x021C_33E8 => {
            let mut cx = cx!(c, 0, 0);
            super::cmd_4d_then_5b74(&mut c.env, &mut cx, v[1], v[2]);
            0
        }
        0x021C_3438 => {
            let (r0, r1) = c.regions.split_at_mut(1);
            let mut cx = super::Cx {
                q: &mut r0[0],
                g: &mut c.globals[0],
                gb: 0,
                w_addr: v[0],
            };
            super::damage_each(
                &mut c.env,
                &mut c.work,
                &mut cx,
                &mut r1[0],
                &c.bufs[1],
                v[2],
                v[3],
            );
            0
        }
        0x021C_36A0 => {
            let mut cx = cx!(c, 0, 0);
            super::cmds_3_32_34(&mut c.env, &mut cx, v[1], &c.bufs[2], v[3]);
            0
        }
        0x021C_36E8 => {
            let mut cx = cx!(c, 0, 0);
            super::cmd_32_then_16f8(
                &mut c.env,
                &mut c.work,
                &mut cx,
                v[1],
                &c.bufs[2],
                v[3],
                v[4],
                v[5],
            );
            0
        }
        0x021C_3720 => {
            let mut cx = cx!(c, 0, 0);
            super::cmds_34_32_41c0(&mut c.env, &mut cx, v[1], &c.bufs[2], v[3]) as u64
        }
        0x021C_376C => {
            let (r0, r1) = c.regions.split_at_mut(1);
            let mut cx = super::Cx {
                q: &mut r0[0],
                g: &mut c.globals[0],
                gb: 0,
                w_addr: v[0],
            };
            super::each_single_mon_effect(
                &mut c.env,
                &mut c.work,
                &mut cx,
                &mut r1[0],
                v[1],
                v[2],
                v[3],
            );
            0
        }
        0x021C_3810 => {
            let mut cx = cx!(c, 0, 0);
            super::single_mon_effect(
                &mut c.env,
                &mut c.work,
                &mut cx,
                v[1],
                v[2],
                v[3],
                &mut c.bufs[4],
                v[5],
                v[6],
                v[7],
            ) as u64
        }
        0x021C_39CC => super::gate_39cc(&mut c.env, &c.work) as u64,
        0x021C_39F4 => super::random_party_slot(&mut c.env, &c.work, &c.bufs[1]) as u64,
        0x021C_3A4C => super::event_8b(&mut c.env, v[0], v[2]) as u64,
        0x021C_3AA8 => {
            let (r0, r1) = c.regions.split_at_mut(1);
            let mut cx = super::Cx {
                q: &mut r0[0],
                g: &mut c.globals[0],
                gb: 0,
                w_addr: v[0],
            };
            super::effect_by_move(
                &mut c.env,
                &mut c.work,
                &mut cx,
                &mut r1[0],
                &c.bufs[1],
                v[2],
            );
            0
        }
        0x021C_3B68 => {
            let mut cx = cx!(c, 0, 0);
            super::slot_gate_apply(&mut c.env, &mut c.work, &mut cx, v[1], v[2]) as u64
        }
        0x021C_3B8C => super::slot_gate(&mut c.env, v[1], v[2]) as u64,
        0x021C_3BB8 => {
            let mut cx = cx!(c, 0, 0);
            super::cmd_3f(&mut c.env, &mut c.work, &mut cx, v[1], v[2]);
            0
        }
        0x021C_3BE0 => {
            super::record_bracket_7998(&mut c.env, &mut c.work, v[1]);
            0
        }
        0x021C_3C10 => super::event_7c(&mut c.env, v[1], v[2]) as u64,
        0x021C_3C60 => {
            let mut cx = cx!(c, 0, 0);
            super::cmd_21_22(&mut c.env, &mut cx, v[1], v[2], v[3]) as u64
        }
        0x021C_3CCC => {
            super::event_9e(&mut c.env, v[1], v[2]);
            0
        }
        0x021C_3D08 => {
            let (r0, r1) = c.regions.split_at_mut(1);
            let mut cx = super::Cx {
                q: &mut r0[0],
                g: &mut c.globals[0],
                gb: 0,
                w_addr: v[0],
            };
            super::effect_by_id(
                &mut c.env,
                &mut c.work,
                &mut cx,
                &mut r1[0],
                &c.bufs[1],
                v[2],
                v[3],
            );
            0
        }
        0x021C_3E0C => {
            let (r0, r1) = c.regions.split_at_mut(1);
            let mut cx = super::Cx {
                q: &mut r0[0],
                g: &mut c.globals[0],
                gb: 0,
                w_addr: v[0],
            };
            super::swap_fields(&mut c.env, &mut c.work, &mut cx, &mut r1[0], v[1], v[2]);
            0
        }
        0x021C_4034 => {
            let mut cx = cx!(c, 0, 0);
            super::pay_quarter_hp(&mut c.env, &mut c.work, &mut cx, v[1]) as u64
        }
        0x021C_4104 => {
            let mut cx = cx!(c, 0, 0);
            super::apply_target_effects(
                &mut c.env,
                &mut c.work,
                &mut cx,
                v[1],
                v[2],
                v[3],
                v[4],
                v[5],
                &c.bufs[6],
            ) as u64
        }
        0x021C_41C0 => {
            let mut cx = cx!(c, 0, 0);
            super::cmds_28_52(&mut c.env, &mut cx, v[1]);
            0
        }
        0x021C_4218 => {
            super::record_bracket_4268(
                &mut c.env,
                &mut c.work,
                v[1],
                v[2],
                &c.bufs[3],
                v[4],
                v[5],
                v[6],
                v[7],
            );
            0
        }
        0x021C_4268 => {
            super::msg_4268(&mut c.env, v[1], v[2], &c.bufs[3], v[4], v[5], v[6], v[7]);
            0
        }
        0x021C_4304 => {
            let (a, b) = c.bufs.split_at_mut(4);
            super::list_message(&mut c.env, &a[1], v[2], v[3], &mut b[0]) as u64
        }
        0x021C_43F0 => {
            super::collect_mons(&mut c.env, v[0], v[1]);
            0
        }
        0x021C_446C => {
            let mut cx = cx!(c, 0, 0);
            super::turn_end(&mut c.env, &mut c.work, &mut cx) as u64
        }
        0x021C_4614 => {
            let mut cx = cx!(c, 0, 0);
            super::pair_sides(&mut c.env, &c.work, &mut cx);
            0
        }
        0x021C_4744 => {
            let mut cx = cx!(c, 0, 0);
            super::random_side_effect(&mut c.env, &mut c.work, &mut cx);
            0
        }
        // WIRE-INSERT
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/rom-diff/svflow.py core4`).
#[test]
fn svflow_match_rom() {
    run_area_suite(
        "core4",
        include_str!("../../../../../tools/rom-diff/svflow_specs/core4.spec"),
        super::PORTED,
        RECORDED,
        call,
    );
}
