use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02160A94, "c35507c8228c0183"),
    (0x02160C88, "ac103a138f86a2de"),
    (0x02160D6C, "3674587ce57e2b69"),
    (0x02160DDC, "5c3d6907045aa674"),
    (0x02160E08, "760a6e38f1fe598b"),
    (0x02160F88, "e66953e70f17c38d"),
    (0x02160FB4, "fd58a23576bf845a"),
    (0x02160FE0, "c9fddbcbe6e67b4b"),
    (0x02161070, "9fd7d87a28a693d2"),
    (0x0216117C, "3fb8284b77833495"),
    (0x021611E0, "02eac9bdaa08cda9"),
    (0x02161214, "3f75d322100b59df"),
    (0x02161280, "ab1002fa3648fe8a"),
    (0x02161308, "b3129944606ad43e"),
    (0x02161340, "38224193460200ac"),
    (0x02161350, "8a51c311d99ddee6"),
    (0x02161378, "489f22cc98883ad5"),
    (0x021613A4, "9eb40dcb6628b712"),
    (0x021613CC, "154232b76a7c1a8d"),
    (0x02161484, "e35c12866e7ece91"),
    (0x02161534, "dbd0773387f80120"),
    (0x02161554, "9f83bb887e7edb33"),
    (0x02161610, "f289eebc35851e21"),
    (0x0216163C, "9f0b80ea30be010f"),
    (0x02161668, "6a8c05413162fd66"),
    (0x021616A4, "a4b3ac76b4ede3ce"),
    (0x021616F0, "a8696536e322a9e2"),
    (0x02161724, "662de8e5ec9b4944"),
    (0x02161824, "8f275fc645b7d742"),
    (0x02161850, "0de97ef38dc3baca"),
    (0x021618A0, "b0f8a7cd3e676bd5"),
    (0x021618E0, "99ed3e8db8c5011d"),
    (0x02161938, "01e6299ad3cc74f4"),
    (0x02161950, "d8165ded3b2762d7"),
    (0x02161964, "995230d48ee28d4a"),
    (0x02161984, "8f21fbb9de139429"),
    (0x021619B8, "ac9ebd10e58759cc"),
    (0x02161AC0, "f95f358f290ce617"),
    (0x02161B1C, "5fe180a618b43185"),
    (0x02161B74, "50d6e0da81b53817"),
    (0x02161B90, "0c2473a5c499bb28"),
    (0x02161BF8, "a18e91bf1d80f9c2"),
    (0x02161C84, "f775c7b8fb27c7dd"),
    (0x02161CDC, "b7efc6ff75ae6de8"),
    (0x02161D18, "b5da7fa917efda25"),
    (0x02161D64, "7c89a06833d518e9"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/proclink.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| {
        let r = match addr {
            0x0216_0A94 => party_init(e, v[0], v[1], v[2], v[3]),
            0x0216_0C88 => party_return(e, v[0], v[1]),
            0x0216_0D6C => d4_init(e, v[0], v[1]),
            0x0216_0DDC => d4_return(e, v[0], v[1]),
            0x0216_0E08 => app83_init(e, v[0], v[1], v[2], v[3]),
            0x0216_0F88 => app83_return(e, v[0], v[1]),
            0x0216_0FB4 => zone_mode(e, v[0]),
            0x0216_0FE0 => bag_init(e, v[0], v[1], v[2], v[3]),
            0x0216_1070 => bag_return(e, v[0], v[1]),
            0x0216_117C => app6f_init(e, v[0], v[1]),
            0x0216_11E0 => app6f_return(e, v[0], v[1]),
            0x0216_1214 => app48_init(e, v[0]),
            0x0216_1280 => app48_return(e, v[0], v[1]),
            0x0216_1308 => app44_init(e, v[0]),
            0x0216_1340 => app44_return(e, v[1]),
            0x0216_1350 => {
                report_special(e, v[0]);
                0
            }
            0x0216_1378 => app80_init(e, v[0]),
            0x0216_13A4 => app80_return(e, v[0], v[1]),
            0x0216_13CC => item_init(e, v[0], v[2], v[3]),
            0x0216_1484 => item_return(e, v[0], v[1]),
            0x0216_1534 => {
                item_free(e, v[0]);
                0
            }
            0x0216_1554 => appc3_init(e, v[0], v[2], v[3]),
            0x0216_1610 => appc3_return(e, v[0]),
            0x0216_163C => appbb_init(e, v[0]),
            0x0216_1668 => appbb_return(e, v[0], v[1]),
            0x0216_16A4 => event_new(e, v[0], v[1], v[2], v[3], v[4]),
            0x0216_16F0 => event_proc(e, v[2]),
            0x0216_1724 => pause_proc(e, v[1], v[2]),
            0x0216_1824 => pause_new(e, v[0], v[1]),
            0x0216_1850 => acmd_proc(e, v[1], v[2]),
            0x0216_18A0 => acmd_new(e, v[0], v[1], v[2], v[3]),
            0x0216_18E0 => acmd_new_player(e, v[0], v[1]),
            0x0216_1938 => obj_of(e, v[0], v[1]),
            0x0216_1950 => {
                slots_clear(e, v[0]);
                0
            }
            0x0216_1964 => {
                slot_put(e, v[0], v[1]);
                0
            }
            0x0216_1984 => slots_update(e, v[0]),
            0x0216_19B8 => jump_proc(e, v[1], v[2]),
            0x0216_1AC0 => jump_new(e, v[0], v[1], v[2], v[3]),
            0x0216_1B1C => acmd_start_player(e, v[0]),
            0x0216_1B74 => acmd_end(e, v[0]),
            0x0216_1B90 => fade_new(e, v[0], v[1], v[2], v[3], v[4]),
            0x0216_1BF8 => fade_proc(e, v[1], v[2]),
            0x0216_1C84 => param_fill(e, v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]),
            0x0216_1CDC => param_a(e, v[0], v[1], v[2], v[3]),
            0x0216_1D18 => param_b(e, v[0], v[1], v[2], v[3]),
            0x0216_1D64 => {
                display_set(e, v[0]);
                0
            }
            _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
        };
        u64::from(r)
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field proclink`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("proclink", SPEC, PORTED, RECORDED), &call);
}

/// [`ARC`] equals the ROM words at 0x0217140C (overlay 10, loaded at 0x02155160).
#[test]
fn arc_matches_rom() {
    let dir = std::env::var("BLACK_ROM_BUILD").ok();
    let Some(ov) = dir.and_then(|d| std::fs::read(format!("{d}/OVY_10.sbin")).ok()) else {
        assert!(std::env::var("REQUIRE_ROM").is_err(), "OVY_10.sbin missing");
        return;
    };
    for (k, &w) in ARC.iter().enumerate() {
        let o = 0x0217_140C - 0x0215_5160 + 4 * k;
        assert_eq!(
            u32::from_le_bytes(ov[o..o + 4].try_into().unwrap()),
            w,
            "ARC[{k}]"
        );
    }
}
