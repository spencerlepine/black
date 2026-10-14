use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021A1DB4, "6c4062ed625a66e2"),
    (0x021A1DF8, "2d025b6139b9ff68"),
    (0x021A1E08, "43afc5e004250b3f"),
    (0x021A1FEC, "40a26aea54dbcc96"),
    (0x021A2050, "fd94c6cefb83047e"),
    (0x021A20C8, "32686b4ac6bad3e7"),
    (0x021A2108, "0b363db2788786ae"),
    (0x021A2114, "8ebb57bf0ac6a2b6"),
    (0x021A2120, "64e020aefb2b3625"),
    (0x021A2130, "7a51dd451080b779"),
    (0x021A213C, "6e35a0fe42ac2621"),
    (0x021A2148, "33ffae61ff18b853"),
    (0x021A2154, "f754f6d12d38680e"),
    (0x021A2160, "4aafa0b107b3f38b"),
    (0x021A2170, "139798182f9bbe17"),
    (0x021A2180, "d4a8d8ab6be5c575"),
    (0x021A2190, "3bd20b6f94604080"),
    (0x021A21A0, "5728cdcc6eed88d7"),
    (0x021A21B0, "bfc2ab0ed03fa0f3"),
    (0x021A21B8, "0ea8604b057fe5ec"),
    (0x021A21F8, "de4ce5b907ee2e3b"),
    (0x021A2228, "e6b256816072fd9d"),
    (0x021A223C, "0480f94faf8fc311"),
    (0x021A2248, "b96d872c4d605461"),
    (0x021A22E0, "abf5420a2201a02f"),
    (0x021A231C, "6b82bf1bf8187cbd"),
    (0x021A232C, "3302eabb0584821f"),
    (0x021A2390, "efb153915c8f70d2"),
    (0x021A23B0, "91db7d7d4fc5a637"),
    (0x021A23D4, "b06cc0f25ac0e34d"),
    (0x021A23E8, "2d17b6ad574f8246"),
    (0x021A23F4, "05175394aee0a93c"),
    (0x021A2400, "662edd1ad0992998"),
    (0x021A240C, "76caf17e8723dd8f"),
    (0x021A2418, "d505ec1615a282da"),
    (0x021A25BC, "651acf9f3d449dc0"),
    (0x021A25CC, "e60dffaa5934bdf5"),
    (0x021A25DC, "60e8a5696b81b220"),
    (0x021A2604, "86034e86b4364c75"),
    (0x021A26D8, "76bd541e54ad2c15"),
    (0x021A2724, "eeee47433f9b1688"),
    (0x021A273C, "c72a5e221ea7970d"),
    (0x021A282C, "a26d60691a9e01b4"),
    (0x021A284C, "ad8832e161217411"),
    (0x021A28CC, "c08867a39c61bd4d"),
    (0x021A2924, "23c71eb0c6ab67a1"),
    (0x021A2968, "a61b05b2b23ec69c"),
    (0x021A2970, "4f803989a1e14729"),
    (0x021A2A84, "9609e634e3a0b23b"),
    (0x021A2AAC, "24f06cdc78137b14"),
    (0x021A2AB8, "4369a7430f9903d6"),
    (0x021A2B8C, "ccbfffa5dee431d5"),
    (0x021A2C2C, "768287f1c3093a5e"),
    (0x021A2C30, "59523136649163f5"),
    (0x021A2C6C, "2a7ae2cf3b4db5f0"),
    (0x021A2C98, "4d8d9cad24c013cb"),
    (0x021A2CC8, "217ec389589f1475"),
    (0x021A2CEC, "6d846fb9f56a8d55"),
    (0x021A2CF8, "bd5bd0b90ea92a35"),
    (0x021A2D04, "de8f1d57967a5bac"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/weather_task.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| {
        let r = match addr {
            0x021A_1DB4 => new(e, v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], v[8]),
            0x021A_1DF8 => {
                delete(e, v[0]);
                0
            }
            0x021A_1E08 => {
                frame(e, v[0], v[1]);
                0
            }
            0x021A_1FEC => {
                start(e, v[0], v[1], v[2], v[3], v[4], v[5]);
                0
            }
            0x021A_2050 => {
                stop(e, v[0], v[1], v[2]);
                0
            }
            0x021A_20C8 => {
                end(e, v[0]);
                0
            }
            0x021A_2108 => {
                light_change(e, v[0], v[1], v[2]);
                0
            }
            0x021A_2114 => {
                light_change_now(e, v[0], v[1], v[2]);
                0
            }
            0x021A_2120 => {
                light_restore(e, v[0], v[1]);
                0
            }
            0x021A_2130 => {
                light_fade(e, v[0], v[1], v[2], v[3]);
                0
            }
            0x021A_213C => light_fade_set(e, v[0]),
            0x021A_2148 => camera_target(e, v[0]),
            0x021A_2154 => {
                light_fade_once(e, v[0], v[1], v[2]);
                0
            }
            0x021A_2160 => {
                bg_set_8(e, v[0], v[1], v[2]);
                0
            }
            0x021A_2170 => {
                bg_set_14(e, v[0], v[1], v[2]);
                0
            }
            0x021A_2180 => {
                bg_set_18(e, v[0], v[1], v[2]);
                0
            }
            0x021A_2190 => {
                bg_set_c(e, v[0], v[1], v[2]);
                0
            }
            0x021A_21A0 => {
                bg_set_10(e, v[0], v[1], v[2]);
                0
            }
            0x021A_21B0 => status_thunk(e, v[0]),
            0x021A_21B8 => work(e, v[0]),
            0x021A_21F8 => {
                stepper_start(e, v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], v[8]);
                0
            }
            0x021A_2228 => {
                stepper_retarget(e, v[0], v[1], v[2], v[3], v[4]);
                0
            }
            0x021A_223C => stepper_frame(e, v[0], v[1]),
            0x021A_2248 => stepper_frame_plain(e, v[0], v[1]),
            0x021A_22E0 => {
                fog_on(e, v[0], v[1], v[2], v[3]);
                0
            }
            0x021A_231C => {
                fog_off(e, v[0], v[1]);
                0
            }
            0x021A_232C => {
                fog_fade(e, v[0], v[1], v[2], v[3], v[4]);
                0
            }
            0x021A_2390 => {
                fog_set(e, v[0], v[1], v[2], v[3]);
                0
            }
            0x021A_23B0 => fog_idle(e, v[0]),
            0x021A_23D4 => zonefog_on(e, v[0]),
            0x021A_23E8 => zonefog_get_0380(e, v[0]),
            0x021A_23F4 => zonefog_get_0390(e, v[0]),
            0x021A_2400 => zonefog_get_03a4(e, v[0]),
            0x021A_240C => zonefog_get_03b8(e, v[0]),
            0x021A_2418 => zonefog_get_03cc(e, v[0]),
            0x021A_25BC => {
                se_start(e, v[0], v[1]);
                0
            }
            0x021A_25CC => {
                se_stop(e, v[0]);
                0
            }
            0x021A_25DC => {
                se_play(e, v[0], v[1]);
                0
            }
            0x021A_2604 => {
                se_play2(e, v[0], v[1], v[2]);
                0
            }
            0x021A_26D8 => {
                reset(e, v[0]);
                0
            }
            0x021A_2724 => callback(e, v[0], v[1], v[2]),
            0x021A_273C => status(e, v[0]),
            0x021A_282C => {
                res_load(e, v[0], v[1]);
                0
            }
            0x021A_284C => {
                res_free(e, v[0]);
                0
            }
            0x021A_28CC => {
                work_new(e, v[0], v[1]);
                0
            }
            0x021A_2924 => {
                work_free(e, v[0]);
                0
            }
            0x021A_2968 => {
                set_ready(e, v[0]);
                0
            }
            0x021A_2970 => ready(e, v[0]),
            0x021A_2A84 => {
                stepper_init(
                    e, v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], v[8], v[9],
                );
                0
            }
            0x021A_2AAC => {
                stepper_targets(e, v[0], v[1], v[2], v[3], v[4]);
                0
            }
            0x021A_2AB8 => stepper_step(e, v[0], v[1]),
            0x021A_2B8C => stepper_tick(e, v[0], v[1]),
            0x021A_2C2C => {
                sound_set_mode(e, v[0], v[1]);
                0
            }
            0x021A_2C30 => {
                sound_start(e, v[0], v[1], v[2]);
                0
            }
            0x021A_2C6C => {
                sound_stop(e, v[0], v[1]);
                0
            }
            0x021A_2C98 => {
                sound_resume(e, v[0], v[1]);
                0
            }
            0x021A_2CC8 => {
                fog_fade_request(e, v[0], v[1], v[2], v[3]);
                0
            }
            0x021A_2CEC => {
                fog_fade_clear(e, v[0]);
                0
            }
            0x021A_2CF8 => fog_fade_pending(e, v[0]),
            0x021A_2D04 => {
                fog_fade_apply(e, v[0]);
                0
            }
            _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
        };
        r as u64
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field weather_task`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("weather_task", SPEC, PORTED, RECORDED), &call);
}

/// The embedded status table is the ROM's (`0x021D88E8`, overlay 21 at `0x02187F00`).
#[test]
fn rom_tables_match() {
    let Some(w) = crate::testenv::overlay21() else {
        return;
    };
    for (i, &s) in STATUS.iter().enumerate() {
        let a = 0x021D_88E8 + i as u32;
        assert_eq!((w(a & !3) >> (8 * (a & 3))) as u8, s, "state {i}");
    }
}
