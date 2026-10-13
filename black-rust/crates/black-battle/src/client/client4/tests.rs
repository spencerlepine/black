use crate::server_flow::testkit::{run_area_suite, Case};

// Written by tools/check-svflow.sh only, after the Rust hashes matched the ROM run.
// svflow-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021D10B8, "6f1a2694b3fc9adb"),
    (0x021D123C, "eb21c63e2a1cb6be"),
    (0x021D127C, "881b67156c36ff4a"),
    (0x021D1290, "aed7b7aace2bcdf3"),
    (0x021D13A4, "f2f42c399751774d"),
    (0x021D13FC, "4f572b33d9785be8"),
    (0x021D1410, "40bef08f45dcf91d"),
    (0x021D1708, "aa3dead5630b6ced"),
    (0x021D171C, "118e65460e38e9fd"),
    (0x021D185C, "0234b56eb76266c2"),
    (0x021D18EC, "9b4951aee1a90833"),
    (0x021D1960, "c3429144670f2403"),
    (0x021D1A18, "0eae94dbcce5328e"),
    (0x021D1A60, "b16308e1d2f88abe"),
    (0x021D1A74, "02dc127870777a0f"),
    (0x021D1BD0, "836a277453e4d178"),
    (0x021D1C40, "34669cf1403c74be"),
    (0x021D1CAC, "2c8daf3831ba7abb"),
    (0x021D1CF4, "6b424016acde7f91"),
    (0x021D1DF4, "cbae5d7abc0145d0"),
    (0x021D1E6C, "14195f4f534da471"),
    (0x021D1EB4, "ac9c9c983f59abad"),
    (0x021D1F10, "d5a2d4470dceb8f9"),
    (0x021D1F58, "fcb19c8164e91eca"),
    (0x021D1FB4, "206e82ec5a65d134"),
    (0x021D2000, "da63bfb006c000af"),
    (0x021D2030, "40c3a33cdfa7d3ea"),
    (0x021D20BC, "204b455325c8c719"),
    (0x021D210C, "09aabb178ad17887"),
    (0x021D215C, "7e27cd3484251c50"),
    (0x021D21D4, "00718cc84f6d2ff3"),
    (0x021D2234, "e3d41951baa3c203"),
    (0x021D2278, "300c1d24bce5e6db"),
    (0x021D2320, "c51407e53ee8acd3"),
    (0x021D2374, "0eada83ab0da5193"),
    (0x021D23B8, "a1d778021bb9bd98"),
    (0x021D2424, "ec8ac013252a5f83"),
    (0x021D2490, "aa8c03537bebd24d"),
    (0x021D2510, "04cc5a524647f335"),
    (0x021D2580, "426a5c4951c3cec0"),
    (0x021D25C4, "9054a564e73e2434"),
    (0x021D2628, "4f66474481bd4bf3"),
    (0x021D266C, "c8f310d8f3954759"),
    (0x021D2700, "2427ee27e16e002c"),
    (0x021D2814, "e176cb86fc873a7f"),
    (0x021D284C, "c7654842a51f0003"),
    (0x021D28AC, "26b45916f432b703"),
    (0x021D2930, "39cac444d2718ac8"),
    (0x021D2990, "3efc5f0cb6f6816f"),
    (0x021D299C, "5bd1973e1e6d105a"),
    (0x021D29CC, "6a7f1c2652fbc4bc"),
    (0x021D2A20, "0e41ffb59c008a05"),
    (0x021D2C7C, "b271ff9d79f401b9"),
    (0x021D2CA8, "042273250d684c53"),
];
// svflow-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    let env = &mut c.env;
    let mut b: Vec<&mut [u8]> = c.bufs.iter_mut().map(|b| b.as_mut_slice()).collect();
    // (c, state, args) handlers
    macro_rules! h3 {
        ($f:path) => {{
            let (o, rest) = b.split_at_mut(1);
            let (s, a) = rest.split_at_mut(1);
            $f(env, &*o[0], &mut *s[0], &*a[0]) as u64
        }};
    }
    // (c, state, data, data_base) request steps
    macro_rules! d3 {
        ($f:path) => {{
            let (o, rest) = b.split_at_mut(1);
            let (s, d) = rest.split_at_mut(1);
            $f(env, &*o[0], &mut *s[0], &*d[0], v[2]) as u64
        }};
    }
    match addr {
        0x021D_10B8 => {
            let (o, rest) = b.split_at_mut(1);
            let (s, d) = rest.split_at_mut(1);
            super::req_switch_prompt(env, &mut *o[0], v[0], &mut *s[0], &*d[0], v[2]) as u64
        }
        0x021D_123C => super::req_record_data(env, &mut *b[0]) as u64,
        0x021D_127C => d3!(super::req_battle_result_bgm_mode2),
        0x021D_1290 => d3!(super::req_battle_result_bgm),
        0x021D_13A4 => super::battle_result(env, &*b[0], &*b[1], v[1]) as u64,
        0x021D_13FC => d3!(super::req_battle_end_mode2),
        0x021D_1410 => d3!(super::req_battle_end),
        0x021D_1708 => d3!(super::req_trainer_message_mode2),
        0x021D_171C => d3!(super::req_trainer_message),
        0x021D_185C => {
            super::trainer_speech(env, &*b[0], v[1], v[2]);
            0
        }
        0x021D_18EC => {
            let (o, s) = b.split_at_mut(1);
            super::req_money_message(env, &*o[0], &mut *s[0]) as u64
        }
        0x021D_1960 => {
            let (o, s) = b.split_at_mut(1);
            super::req_lost_message(env, &*o[0], &mut *s[0]) as u64
        }
        0x021D_1A18 => {
            let (o, s) = b.split_at_mut(1);
            super::req_message_b7(env, &*o[0], &mut *s[0]) as u64
        }
        0x021D_1A60 | 0x021D_1A74 => {
            let (o, rest) = b.split_at_mut(1);
            let (s, rest) = rest.split_at_mut(1);
            let (q, d) = rest.split_at_mut(1);
            if addr == 0x021D_1A60 {
                super::play_commands_mode2(
                    env, &mut *o[0], &mut *s[0], &mut *q[0], v[2], &*d[0], v[3],
                ) as u64
            } else {
                super::play_commands(env, &mut *o[0], &mut *s[0], &mut *q[0], v[2], &*d[0], v[3])
                    as u64
            }
        }
        0x021D_1BD0 => h3!(super::cmd_hp_message),
        0x021D_1C40 => {
            let (o, out) = b.split_at_mut(1);
            super::hp_message_id(env, &*o[0], v[1], &mut *out[1]) as u64
        }
        0x021D_1CAC => h3!(super::cmd_hp_gauge),
        0x021D_1CF4 => h3!(super::cmd_send_out),
        0x021D_1DF4 => super::own_send_out_message(env, &*b[0]) as u64,
        0x021D_1E6C => h3!(super::cmd_message),
        0x021D_1EB4 => h3!(super::cmd_message_se),
        0x021D_1F10 => h3!(super::cmd_message_alt),
        0x021D_1F58 => h3!(super::cmd_message_alt_se),
        0x021D_1FB4 => h3!(super::cmd_message_6f4),
        0x021D_2000 => super::move_anim_param(env, &*b[0], v[1], v[2]) as u64,
        0x021D_2030 => h3!(super::cmd_move_anim),
        0x021D_20BC => h3!(super::cmd_status_anim),
        0x021D_210C => h3!(super::cmd_effect_anim),
        0x021D_215C => {
            let (o, rest) = b.split_at_mut(1);
            let (s, rest) = rest.split_at_mut(1);
            let (a, q) = rest.split_at_mut(1);
            super::cmd_targets_anim(env, &*o[0], &mut *s[0], &*a[0], &mut *q[0], v[3]) as u64
        }
        0x021D_21D4 => h3!(super::cmd_slot_anim_message),
        0x021D_2234 => h3!(super::cmd_fx_81d0),
        0x021D_2278 => h3!(super::cmd_hit_anim),
        0x021D_2320 => h3!(super::cmd_faint_anim),
        0x021D_2374 => h3!(super::cmd_slot_anim_438),
        0x021D_23B8 => h3!(super::cmd_position_anim_7c8),
        0x021D_2424 => h3!(super::cmd_position_anim_7d8),
        0x021D_2490 => h3!(super::cmd_field_effect),
        0x021D_2510 => h3!(super::cmd_field_end),
        0x021D_2580 => h3!(super::cmd_slot_anim_41),
        0x021D_25C4 => h3!(super::cmd_slot_anim_838),
        0x021D_2628 => h3!(super::cmd_slot_anim_43),
        0x021D_266C => h3!(super::cmd_swap_slots),
        0x021D_2700 => h3!(super::cmd_rotate),
        0x021D_2814 => h3!(super::cmd_fx_7e94),
        0x021D_284C => h3!(super::cmd_fx_7ea4),
        0x021D_28AC => h3!(super::cmd_transform),
        0x021D_2930 => h3!(super::cmd_effectiveness_anim),
        0x021D_2990 => super::play_sound(env, &*b[2]) as u64,
        0x021D_299C => {
            let (o, s) = b.split_at_mut(1);
            super::view_effect_step(env, &*o[0], &mut *s[0]) as u64
        }
        0x021D_29CC => super::cmd_add_evs(env, &*b[0], &*b[2]) as u64,
        0x021D_2A20 => {
            let (o, rest) = b.split_at_mut(1);
            let (s, a) = rest.split_at_mut(1);
            super::cmd_level_up(env, &*o[0], &mut *s[0], &*a[0], &mut c.globals[0]) as u64
        }
        0x021D_2C7C => super::window_sound_event(env, v[0]) as u64,
        0x021D_2CA8 => {
            let (o, s) = b.split_at_mut(1);
            super::learn_move_step(env, &*o[0], &mut *s[0], v[2], &mut c.globals[0]) as u64
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/rom-diff/svflow.py client4`).
#[test]
fn svflow_match_rom() {
    run_area_suite(
        "client4",
        include_str!("../../../../../tools/rom-diff/svflow_specs/client4.spec"),
        super::PORTED,
        RECORDED,
        call,
    );
}
