use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0218B754, "e9c089bc8ca2c850"),
    (0x0218B7F8, "dce68afcea42f11a"),
    (0x0218B83C, "e0eea9e9a2bd5a99"),
    (0x0218B88C, "41f5c025fc02d83e"),
    (0x0218B8AC, "09b5aba509aff5f1"),
    (0x0218B910, "cc88de07417c00be"),
    (0x0218B924, "0a3263481873295f"),
    (0x0218B930, "f5fcea35b2e1c1a4"),
    (0x0218B974, "3d0e3e232c39498e"),
    (0x0218B9FC, "b32ba34a5a31cc13"),
    (0x0218BA04, "30559e3e06f3906e"),
    (0x0218BA48, "0d6f8ffc02dd9e39"),
    (0x0218BA70, "ae7ea6fa04b8074c"),
    (0x0218BA98, "3c268ef906ac6786"),
    (0x0218BACC, "94ab9b55009fa7c0"),
    (0x0218BB04, "64835917f4dffdea"),
    (0x0218BB0C, "f2a0071f2e8a91a8"),
    (0x0218BB64, "8a9e8f4764be8ea5"),
    (0x0218BB78, "d270d5b323df5c7e"),
    (0x0218BBD0, "b10e9c901a8c471a"),
    (0x0218BBF4, "a6fa8d47726bd3b6"),
    (0x0218BC00, "6e36305a3f42c9a6"),
    (0x0218BC10, "4a674d0ca910ee32"),
    (0x0218BC1C, "5d6bef24b1a54222"),
    (0x0218BC40, "c3e9ddb348094535"),
    (0x0218BC6C, "5339afee12f2e581"),
    (0x0218BC8C, "9727d8af1ba5c24d"),
    (0x0218BCAC, "8783357a2a2e0b18"),
    (0x0218BCB0, "b71b896059a04dbd"),
    (0x0218BCE4, "86acf755882de43c"),
    (0x0218BCF4, "98cbf2780c364536"),
    (0x0218BD20, "5be90951ae9b8f63"),
    (0x0218BD24, "bf47377543f5c425"),
    (0x0218BD28, "27f1d400db3e6014"),
    (0x0218BD94, "e6e0615849f28d36"),
    (0x0218BDD8, "3ba0563a3bf86972"),
    (0x0218BE84, "0ca168b639d45338"),
    (0x0218BEE8, "61dac3860dde1eae"),
    (0x0218BF84, "a275a52705537bc7"),
    (0x0218BFB4, "7478e01095dac471"),
    (0x0218C02C, "95e6177aca8ec828"),
    (0x0218C040, "7d3239d312ed7125"),
    (0x0218C08C, "0b8fb2b5eb478691"),
    (0x0218C0D8, "85755b5d2d1f3fb6"),
    (0x0218C124, "b2f63824e2d0862c"),
    (0x0218C150, "c99542a0deb47d2d"),
    (0x0218C154, "f396d3285bd7bdd4"),
    (0x0218C1A4, "f3196ca1aab823e8"),
    (0x0218C1C4, "cdb0a9f6e8b84e27"),
    (0x0218C23C, "411bc1513844aa40"),
    (0x0218C290, "9b766170ffc3b6c9"),
    (0x0218C2E4, "987a646ac654ef23"),
    (0x0218C350, "f0e7aa68e0dbdc6e"),
    (0x0218C388, "2d987ab1c5b47caf"),
    (0x0218C3B0, "7d147bc18eec3d74"),
    (0x0218C3E0, "2728ed81054667f3"),
    (0x0218C438, "c99542a0deb47d2d"),
    (0x0218C43C, "82028691b3dc7b49"),
    (0x0218C47C, "cb74d6f71b272c0d"),
    (0x0218C490, "3079d684f77f78d5"),
    (0x0218C4A0, "9a0cbb4a5d77291e"),
    (0x0218C4EC, "5b58b44354e59338"),
    (0x0218C518, "1a44ecf11b403b6a"),
    (0x0218C530, "c1d73bde53871285"),
    (0x0218C54C, "d4c067dc732245a5"),
    (0x0218C574, "344e3807cb029946"),
    (0x0218C584, "db9a598e5e814641"),
    (0x0218C638, "a3e36f0c5459042e"),
    (0x0218C654, "5aa5ec9b6f9e68d8"),
    (0x0218C674, "649d5d21e9fbc8cd"),
    (0x0218C69C, "d765ba2007ceee55"),
    (0x0218C6D0, "00d3753d7af6ffe2"),
    (0x0218C6F4, "c0e7046b949d6af9"),
    (0x0218C724, "0c394f254f306678"),
    (0x0218C7D8, "cf38218f9162191d"),
    (0x0218C7F0, "518673f208869590"),
    (0x0218C860, "4c333869012e18fb"),
    (0x0218C88C, "880da31341ef550e"),
    (0x0218C89C, "642f12e1ef0ead04"),
    (0x0218C8B0, "c6af71eab8f72d44"),
    (0x0218C8D8, "093b24aaecd2d638"),
    (0x0218C8FC, "6bb8faeffe85b601"),
    (0x0218C910, "06cfb40c6639e995"),
    (0x0218C924, "a142c82c393e314e"),
    (0x0218C934, "f08251a9226a4d71"),
    (0x0218C944, "2be772a66640b596"),
    (0x0218C990, "edb3814f0f690eac"),
    (0x0218C9A4, "568ca7f4a4dfbd0a"),
    (0x0218C9C4, "9214159c84142a1c"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/buildmodel.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| {
        let r: u32 = match addr {
            0x0218_B754 => new(e, v[0], v[1], v[2]),
            0x0218_B7F8 => {
                delete(e, v[0]);
                0
            }
            0x0218_B83C => {
                update(e, v[0]);
                0
            }
            0x0218_B88C => {
                draw(e, v[0]);
                0
            }
            0x0218_B8AC => {
                load(e, v[0], v[1], v[2], v[3]);
                0
            }
            0x0218_B910 => record_of(e, v[0], v[1]),
            0x0218_B924 => record(e, v[0], v[1]),
            0x0218_B930 => linked(e, v[0], v[1], v[2], v[3]),
            0x0218_B974 => place_block(e, v[0], v[1], v[2], v[3]),
            0x0218_B9FC => {
                remove_thunk(e, v[0], v[1]);
                0
            }
            0x0218_BA04 => place_one(e, v[0], v[1], v[2], v[3]),
            0x0218_BA48 => {
                door_slot_clear(e, v[0], v[1]);
                0
            }
            0x0218_BA70 => {
                door_slot_put(e, v[0], v[1]);
                0
            }
            0x0218_BA98 => door_by_code(e, v[0], v[1]),
            0x0218_BACC => door_code(e, v[0], v[1]),
            0x0218_BB04 => s_ptr(v[0]),
            0x0218_BB0C => {
                load_list(e, v[0], v[1], v[2]);
                0
            }
            0x0218_BB64 => {
                free_list(e, v[0]);
                0
            }
            0x0218_BB78 => {
                build_map(e, v[0], v[1]);
                0
            }
            0x0218_BBD0 => index_of(e, v[0], v[1]),
            0x0218_BBF4 => list_entry(e, v[0], v[1]),
            0x0218_BC00 => list_entry2(e, v[0], v[1]),
            0x0218_BC10 => hdr_entry(e, v[0], v[1]),
            0x0218_BC1C => member_of(v[0]),
            0x0218_BC40 => load_textures(e, v[0], v[1], v[2], v[3]),
            0x0218_BC6C => {
                free_textures(e, v[0]);
                0
            }
            0x0218_BC8C => hdr_count(e, v[0]),
            0x0218_BCAC => hdr_of(v[0]),
            0x0218_BCB0 => door_kind(e, v[0]),
            0x0218_BCE4 => {
                tod_init(e, v[0], v[1]);
                0
            }
            0x0218_BCF4 => {
                tod_update(e, v[0]);
                0
            }
            0x0218_BD20 => tod_changed(e, v[0]),
            0x0218_BD24 => tod_anim(e, v[0]),
            0x0218_BD28 => {
                load_sets(e, v[0], v[1]);
                0
            }
            0x0218_BD94 => {
                free_sets(e, v[0]);
                0
            }
            0x0218_BDD8 => {
                make_objs(e, v[0], v[1]);
                0
            }
            0x0218_BE84 => {
                free_objs(e, v[0], v[1]);
                0
            }
            0x0218_BEE8 => {
                load_set(e, v[0], v[1], v[2]);
                0
            }
            0x0218_BF84 => {
                free_set(e, v[0]);
                0
            }
            0x0218_BFB4 => {
                make_obj(e, v[0], v[1], v[2]);
                0
            }
            0x0218_C02C => {
                init_off(e, v[1]);
                0
            }
            0x0218_C040 => {
                init_all(e, v[1]);
                0
            }
            0x0218_C08C => {
                init_tod(e, v[0], v[1]);
                0
            }
            0x0218_C0D8 => {
                obj_delete(e, v[0]);
                0
            }
            0x0218_C124 => {
                obj_frame(e, v[0], v[1]);
                0
            }
            0x0218_C150 => {
                frame_none();
                0
            }
            0x0218_C154 => {
                frame_tod(e, v[0], v[1]);
                0
            }
            0x0218_C1A4 => {
                frame_all(e, v[1]);
                0
            }
            0x0218_C1C4 => {
                frame_states(e, v[1]);
                0
            }
            0x0218_C23C => {
                play_forward(e, v[0], v[1]);
                0
            }
            0x0218_C290 => {
                play_loop(e, v[0], v[1]);
                0
            }
            0x0218_C2E4 => {
                play_backward(e, v[0], v[1]);
                0
            }
            0x0218_C350 => {
                stop(e, v[0], v[1]);
                0
            }
            0x0218_C388 => {
                all_off(e, v[0]);
                0
            }
            0x0218_C3B0 => {
                set_state(e, v[0], v[1], v[2]);
                0
            }
            0x0218_C3E0 => {
                state_proc(e, v[0], v[1], v[2]);
                0
            }
            0x0218_C438 => {
                state_none();
                0
            }
            0x0218_C43C => stopped(e, v[0], v[1]),
            0x0218_C47C => frame_of(e, v[0], v[1]),
            0x0218_C490 => {
                entry_clear(e, v[1]);
                0
            }
            0x0218_C4A0 => place(e, v[0], v[1], v[2], v[3]),
            0x0218_C4EC => {
                remove(e, v[0], v[1]);
                0
            }
            0x0218_C518 => {
                entry_remove(e, v[0], v[1]);
                0
            }
            0x0218_C530 => entry_state(e, v[0]),
            0x0218_C54C => {
                entry_swap(e, v[0], v[1]);
                0
            }
            0x0218_C574 => {
                entry_set_state(e, v[0], v[1], v[2]);
                0
            }
            0x0218_C584 => find_in_box(e, v[0], v[1], v[2], v[3]),
            0x0218_C638 => first_in_box(e, v[0], v[1], v[2]),
            0x0218_C654 => first_near(e, v[0], v[1], v[2]),
            0x0218_C674 => entry_door_kind(e, v[0], v[1]),
            0x0218_C69C => {
                entry_set_anim(e, v[0], v[1], v[2], v[3]);
                0
            }
            0x0218_C6D0 => {
                entry_position(e, v[0], v[1]);
                0
            }
            0x0218_C6F4 => entry_frame(e, v[0], v[1], v[2]),
            0x0218_C724 => door_new(e, v[0], v[1]),
            0x0218_C7D8 => door_new_near(e, v[0], v[1], v[2]),
            0x0218_C7F0 => door_new_at(e, v[0], v[1], v[2]),
            0x0218_C860 => {
                door_delete(e, v[0]);
                0
            }
            0x0218_C88C => {
                door_set(e, v[0], v[1], v[2]);
                0
            }
            0x0218_C89C => {
                door_mode(e, v[0], v[1]);
                0
            }
            0x0218_C8B0 => door_start(e, v[0], v[1]),
            0x0218_C8D8 => door_restart(e, v[0]),
            0x0218_C8FC => door_stopped_s(e, v[0], v[1]),
            0x0218_C910 => door_stopped(e, v[0]),
            0x0218_C924 => door_model_kind(e, v[0]),
            0x0218_C934 => {
                door_draw(e, v[0]);
                0
            }
            0x0218_C944 => door_se_of(e, v[0], v[1], v[2]),
            0x0218_C990 => door_se(e, v[0], v[1]),
            0x0218_C9A4 => door_play_se(e, v[0]),
            0x0218_C9C4 => {
                box_around(e, v[0], v[1]);
                0
            }
            _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
        };
        r as u64
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field buildmodel`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("buildmodel", SPEC, PORTED, RECORDED), &call);
}

/// [`TOD_ANIM`], [`DOOR_KIND`] and [`DOOR_SE`] are the ROM's (overlay 21 `0x021D3D50`, `0x021D3DC4`, `0x021D3D88`), and the
/// animation proc tables `0x021D3D58` hold the procs the ports dispatch to.
#[test]
fn tables_match_rom() {
    let Some(w) = crate::testenv::overlay21() else {
        return;
    };
    let byte = |a: u32| (w(a & !3) >> (8 * (a & 3))) as u8;
    let half = |a: u32| (w(a & !3) >> (8 * (a & 2))) as u16;
    for (i, &b) in TOD_ANIM.iter().enumerate() {
        assert_eq!(byte(0x021D_3D50 + i as u32), b, "TOD_ANIM {i}");
    }
    for (i, &b) in DOOR_KIND.iter().enumerate() {
        assert_eq!(byte(0x021D_3DC4 + i as u32), b, "DOOR_KIND {i}");
    }
    for (i, (key, se)) in DOOR_SE.iter().enumerate() {
        let a = 0x021D_3D88 + 10 * i as u32;
        assert_eq!(half(a), *key, "DOOR_SE {i}");
        for (k, &s) in se.iter().enumerate() {
            assert_eq!(half(a + 2 + 2 * k as u32), s, "DOOR_SE {i} {k}");
        }
    }
    // {init, frame, state} per animation kind 0..3 (Thumb addresses)
    let procs = [
        [0x0218_C02D, 0x0218_C151, 0x0218_C439],
        [0x0218_C041, 0x0218_C1A5, 0x0218_C439],
        [0x0218_C02D, 0x0218_C1C5, 0x0218_C3E1],
        [0x0218_C08D, 0x0218_C155, 0x0218_C439],
    ];
    for (k, p) in procs.iter().enumerate() {
        for (j, &f) in p.iter().enumerate() {
            assert_eq!(
                w(0x021D_3D58 + 12 * k as u32 + 4 * j as u32),
                f,
                "proc table {k} {j}"
            );
        }
    }
}
