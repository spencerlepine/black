use crate::server_flow::testkit::{run_area_suite, Case};

// Written by tools/check-svflow.sh only, after the Rust hashes matched the ROM run.
// svflow-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021D2FD8, "94716790c9bde4c0"),
    (0x021D3138, "2701e24386cbc840"),
    (0x021D31A0, "2cdfc036f4635527"),
    (0x021D324C, "d91a177f0a4c34dc"),
    (0x021D32E0, "8d92c241dd6f8937"),
    (0x021D344C, "09fd388c1083af11"),
    (0x021D34B0, "39cd0dd648890f8b"),
    (0x021D34E8, "d67e4af04bc859f6"),
    (0x021D3568, "8233ec0c7206271a"),
    (0x021D35C8, "5308d83e90c189f6"),
    (0x021D3630, "43bc912e71f09213"),
    (0x021D3688, "1422c349a8d677cf"),
    (0x021D36CC, "0a2578c16dfebf20"),
    (0x021D36EC, "138b5727c9dff55c"),
    (0x021D370C, "55f4f50ca103e0d0"),
    (0x021D3734, "f07356e33d2c3d9e"),
    (0x021D375C, "56903e079736586a"),
    (0x021D377C, "4f2d95cc2609a8e3"),
    (0x021D3794, "dcc50cdd8d126d2d"),
    (0x021D37B8, "e102022d1cbb87cf"),
    (0x021D37DC, "394ed6c021698aed"),
    (0x021D37FC, "0d3c4ce46d17efef"),
    (0x021D381C, "638e9204208b2116"),
    (0x021D3894, "d663b5d14cb1076f"),
    (0x021D38AC, "e8f85b0b250dc218"),
    (0x021D38C4, "b7ca70ba959edb38"),
    (0x021D38E0, "6164eb138acb17e3"),
    (0x021D3900, "b10e49867ccd10f4"),
    (0x021D3918, "9b04b0a6c0487702"),
    (0x021D3934, "464e985d3ffc8d9e"),
    (0x021D3988, "7f8fd596063f5119"),
    (0x021D39A8, "b26d85a145be5590"),
    (0x021D3A20, "726663341b247f9c"),
    (0x021D3A40, "f72d4a644d244b7b"),
    (0x021D3A7C, "cc47935127dee45d"),
    (0x021D3A98, "a0c5344b5bc44222"),
    (0x021D3AB4, "bf5932207fb26abd"),
    (0x021D3AD0, "b96e26d506e43ca2"),
    (0x021D3AEC, "02fd5edc2d1ba6b2"),
    (0x021D3B08, "816c490e03123509"),
    (0x021D3B28, "5b04519b032722d6"),
    (0x021D3B5C, "6b80e6bb1cd7344c"),
    (0x021D3B74, "40abb1506c7f737b"),
    (0x021D3B88, "fc11df9fb5aebc6e"),
    (0x021D3B9C, "037c92444a947dcd"),
    (0x021D3BB0, "f93cf3bcb137abcc"),
    (0x021D3BC0, "0177df752bdb23bb"),
    (0x021D3BE0, "205a0349095e0262"),
    (0x021D3C0C, "f326e69a2a2aabe0"),
    (0x021D3C2C, "542f08df45d94f17"),
    (0x021D3C44, "fc809e1e05a351e1"),
    (0x021D3C70, "4c3590227f64408a"),
    (0x021D3C90, "0bc600535213b177"),
    (0x021D3CA8, "114a063c7d4300e6"),
    (0x021D3CC8, "2839ec220663ac18"),
    (0x021D3CFC, "bf916e50e167a4c6"),
    (0x021D3D1C, "98c754cfd269aed5"),
    (0x021D3D2C, "7e22e59ffe49f01f"),
    (0x021D3D50, "bd2f8a075c9c9bd4"),
    (0x021D3D70, "f500b43c53467755"),
    (0x021D3D7C, "d48e847a04729276"),
    (0x021D3D84, "584630d7e2ddf4ee"),
    (0x021D3D90, "d16521b8919d74b4"),
    (0x021D3D9C, "f0d8ddbd0349516e"),
    (0x021D3DC4, "f25b9bcdf149e803"),
    (0x021D3E70, "5c007074db884266"),
    (0x021D3EC8, "21b1c52f074ee6ec"),
];
// svflow-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    let env = &mut c.env;
    let mut b: Vec<&mut [u8]> = c.bufs.iter_mut().map(|b| b.as_mut_slice()).collect();
    // (c, unused, args) one-shot handlers
    macro_rules! h1 {
        ($f:path) => {
            $f(env, &*b[0], &*b[2]) as u64
        };
    }
    // (c, state, args) handlers
    macro_rules! h3 {
        ($f:path) => {{
            let (o, rest) = b.split_at_mut(1);
            let (s, a) = rest.split_at_mut(1);
            $f(env, &*o[0], &mut *s[0], &*a[0]) as u64
        }};
    }
    use super as m;
    match addr {
        0x021D_2FD8 => h3!(m::cmd_exp_gain),
        0x021D_3138 => h3!(m::cmd_gauge_message),
        0x021D_31A0 => h3!(m::cmd_party_slot),
        0x021D_324C => h3!(m::cmd_ability_change),
        0x021D_32E0 => {
            let (o, rest) = b.split_at_mut(1);
            let (s, a) = rest.split_at_mut(1);
            m::cmd_ability_swap(env, &*o[0], &mut *s[0], &*a[0], &mut c.globals[0]) as u64
        }
        0x021D_344C => h3!(m::cmd_illusion_break),
        0x021D_34B0 => {
            let (_, rest) = b.split_at_mut(1);
            let (s, a) = rest.split_at_mut(1);
            m::cmd_play_se(env, &mut *s[0], &*a[0]) as u64
        }
        0x021D_34E8 => h3!(m::cmd_position_anim),
        0x021D_3568 => h3!(m::cmd_pair_anim),
        0x021D_35C8 => h3!(m::cmd_form_change),
        0x021D_3630 => h3!(m::cmd_ability_window_open),
        0x021D_3688 => h3!(m::cmd_ability_window_close),
        0x021D_36CC => h1!(m::cmd_mon_damage),
        0x021D_36EC => h1!(m::cmd_mon_heal),
        0x021D_370C => h1!(m::cmd_mon_spend_pp),
        0x021D_3734 => h1!(m::cmd_mon_spend_orig_pp),
        0x021D_375C => h1!(m::cmd_mon_mark_used),
        0x021D_377C => h1!(m::cmd_mon_faint),
        0x021D_3794 => h1!(m::cmd_mon_restore_pp),
        0x021D_37B8 => h1!(m::cmd_mon_restore_orig_pp),
        0x021D_37DC => h1!(m::cmd_mon_raise_stage),
        0x021D_37FC => h1!(m::cmd_mon_lower_stage),
        0x021D_381C => h1!(m::cmd_mon_set_stages),
        0x021D_3894 => h1!(m::cmd_mon_clear_negative_stages),
        0x021D_38AC => h1!(m::cmd_mon_reset_stages),
        0x021D_38C4 => h1!(m::cmd_mon_add_crit_boost),
        0x021D_38E0 => h1!(m::cmd_mon_set_sick),
        0x021D_3900 => h1!(m::cmd_mon_cure_major_status),
        0x021D_3918 => h1!(m::cmd_mon_cure_sick),
        0x021D_3934 => h1!(m::cmd_switch_in),
        0x021D_3988 => h1!(m::cmd_mon_set_type_pair),
        0x021D_39A8 => h1!(m::cmd_tick_sicks),
        0x021D_3A20 => h1!(m::cmd_mon_set_u14),
        0x021D_3A40 => h1!(m::cmd_mon_record_move),
        0x021D_3A7C => h1!(m::cmd_mon_set_flag_b),
        0x021D_3A98 => h1!(m::cmd_mon_clear_flag_b),
        0x021D_3AB4 => h1!(m::cmd_mon_set_flag_a),
        0x021D_3AD0 => h1!(m::cmd_mon_clear_flag_a),
        0x021D_3AEC => h1!(m::cmd_mon_set_ability),
        0x021D_3B08 => h1!(m::cmd_mon_set_item),
        0x021D_3B28 => h1!(m::cmd_mon_move_replace),
        0x021D_3B5C => h1!(m::cmd_mon_reset_on_battle_end),
        0x021D_3B74 => h1!(m::cmd_view_side_ef9f4),
        0x021D_3B88 => h1!(m::cmd_view_side_efad8),
        0x021D_3B9C => h1!(m::cmd_view_side_efb34),
        0x021D_3BB0 => h1!(m::cmd_view_side_efa94),
        0x021D_3BC0 => h1!(m::cmd_mon_set_u8_157),
        0x021D_3BE0 => h1!(m::cmd_mon_copy_passable),
        0x021D_3C0C => h1!(m::cmd_mon_make_substitute),
        0x021D_3C2C => h1!(m::cmd_mon_clear_substitute),
        0x021D_3C44 => {
            let (o, rest) = b.split_at_mut(1);
            m::cmd_counter_add(env, &mut *o[0], &*rest[1]) as u64
        }
        0x021D_3C70 => h1!(m::cmd_party_9d10),
        0x021D_3C90 => h1!(m::cmd_mon_6738),
        0x021D_3CA8 => h1!(m::cmd_mon_cure_linked),
        0x021D_3CC8 => h1!(m::relay_mon_block),
        0x021D_3CFC => h1!(m::cmd_mon_turn_start),
        0x021D_3D1C => m::cmd_view_side_efc78(env, &*b[0]) as u64,
        0x021D_3D2C => h1!(m::cmd_mon_set_stat),
        0x021D_3D50 => h1!(m::cmd_mon_set_weight),
        0x021D_3D70 => m::client_id(&*b[0]) as u64,
        0x021D_3D7C => m::party(&*b[0]) as u64,
        0x021D_3D84 => m::view_side_ef9a8(env, &*b[0]) as u64,
        0x021D_3D90 => m::counter(&*b[0]) as u64,
        0x021D_3D9C => {
            m::load_trainer_items(env, &mut *b[0]);
            0
        }
        0x021D_3DC4 => m::pick_trainer_item(env, &mut *b[0], v[1], v[2]) as u64,
        0x021D_3E70 => m::item_boosts_stat(env, v[0], v[1]) as u64,
        0x021D_3EC8 => m::item_cures_status(env, v[0], v[1]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/rom-diff/svflow.py client5`).
#[test]
fn svflow_match_rom() {
    run_area_suite(
        "client5",
        include_str!("../../../../../tools/rom-diff/svflow_specs/client5.spec"),
        super::PORTED,
        RECORDED,
        call,
    );
}
