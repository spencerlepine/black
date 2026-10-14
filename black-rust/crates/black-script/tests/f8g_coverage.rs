//! F8g: script-command coverage audit. Every non-null slot of the ROM command table (0x0217062C in overlay 10, 609 entries,
//! count word at 0x021705D8; the 15 words at 0x021705F0 that F1-SPLIT counted as table slots are the separate init-proc
//! table, see `scrcmd.rs`) is either ported (`scrcmd::commands()`), in [`EXCLUDED`] (out of scope, MIGRATION-AGENT.md §1 D9,
//! with the reason per id) or in [`PENDING`] (in scope, specced, not ported yet: follow-up port job).
//!
//! ROM-gated: skipped without `$BLACK_ROM_BUILD/OVY_10.sbin`, a failure with `REQUIRE_ROM=1`.

use black_script::scrcmd::commands;

const TABLE: u32 = 0x0217_062C;
const COUNT_WORD: u32 = 0x0217_05D8;
/// Load address of overlay 10.
const OVY10: u32 = 0x0215_5160;

const MUSICAL: &str = "X2 Musicals (scrcmd_musical.c, D9)";
const NETWORK: &str = "X1 scrcmd_network.c comm / Wi-Fi / Union Room (D9)";
const INTRUDE: &str = "X1 overlay 18 intrude_field.c / intrude_subdisp.c, Entralink (D9)";
const PALPARK: &str = "X1 scrcmd_palpark.c Poke Transfer download play (D9)";

/// Out-of-scope ids: (command id, handler address with Thumb bit as in the ROM table, reason). Sorted by id.
const EXCLUDED: &[(u16, u32, &str)] = &[
    (93, 0x0215_D885, MUSICAL),
    (313, 0x0215_F1C9, NETWORK),
    (315, 0x0215_F215, NETWORK),
    (316, 0x0215_F229, NETWORK),
    (317, 0x0215_F239, NETWORK),
    (318, 0x0215_F281, NETWORK),
    (359, 0x0215_D34D, MUSICAL),
    (360, 0x0215_D469, MUSICAL),
    (361, 0x0215_D505, MUSICAL),
    (362, 0x0215_D7D5, MUSICAL),
    (363, 0x0215_DB39, MUSICAL),
    (364, 0x0215_D865, MUSICAL),
    (365, 0x0215_DADD, MUSICAL),
    (366, 0x0215_DB01, MUSICAL),
    (367, 0x0217_CF69, INTRUDE),
    (368, 0x0217_FAE1, INTRUDE),
    (369, 0x0217_CF89, INTRUDE),
    (371, 0x0217_F649, INTRUDE),
    (372, 0x0217_FB2D, INTRUDE),
    (373, 0x0217_F8A1, INTRUDE),
    (374, 0x0217_F8D1, INTRUDE),
    (375, 0x0217_F915, INTRUDE),
    (
        382,
        0x021F_4651,
        "overlay 48 field gimmick (0x021F4650), outside the `field` set; parked under F10 (needs a tools/re set with ov 48)",
    ),
    (448, 0x021C_C571, PALPARK),
    (449, 0x021C_C5D9, PALPARK),
    (
        494,
        0x021B_6BDD,
        "X2 Dream World delivery (pdw_postman.c, D9); logic specced in scrcmd_pokemon_fld",
    ),
    (495, 0x0215_E1C1, MUSICAL),
    (496, 0x021C_C615, PALPARK),
    (497, 0x021C_C6A9, PALPARK),
    (498, 0x021C_C6FD, PALPARK),
    (499, 0x0215_F3A5, NETWORK),
    (500, 0x0215_F2F1, NETWORK),
    (501, 0x0215_F319, NETWORK),
    (502, 0x0215_F33D, NETWORK),
    (503, 0x0215_F425, NETWORK),
    (504, 0x0215_F4F5, NETWORK),
    (604, 0x0215_F52D, NETWORK),
    (605, 0x0215_F2B9, NETWORK),
    (606, 0x0215_F2DD, NETWORK),
];

/// In-scope ids that are specced but not ported: (command id, handler address, size in bytes, reason).
const PENDING: &[(u16, u32, u32, &str)] = &[(
    414,
    0x021D_29F1,
    0xEC,
    "overlay 21 scrcmd_misc21: draws / clears a text window (BMP window render only, behind Misc21Hooks); specced in scrcmd_misc21.spec",
)];

/// The null table slots (no handler).
const NULL_SLOTS: &[u16] = &[137, 143, 144, 145, 153, 154, 156, 157, 435];

fn overlay10() -> Option<Vec<u8>> {
    let got = std::env::var("BLACK_ROM_BUILD")
        .ok()
        .and_then(|b| std::fs::read(format!("{b}/OVY_10.sbin")).ok());
    if got.is_none() && std::env::var("REQUIRE_ROM").is_ok_and(|v| !v.is_empty() && v != "0") {
        panic!("REQUIRE_ROM is set but $BLACK_ROM_BUILD/OVY_10.sbin is missing");
    }
    got
}

#[test]
fn lists_are_consistent() {
    let ported = commands();
    for w in EXCLUDED.windows(2) {
        assert!(w[0].0 < w[1].0, "EXCLUDED sorted: {}", w[1].0);
    }
    for &(id, _, why) in EXCLUDED {
        assert!(!why.is_empty());
        assert!(
            !ported.iter().any(|c| c.0 == id),
            "{id} is ported and excluded"
        );
        assert!(!NULL_SLOTS.contains(&id));
    }
    for &(id, _, _, why) in PENDING {
        assert!(!why.is_empty());
        assert!(
            !ported.iter().any(|c| c.0 == id),
            "{id} is ported and pending"
        );
        assert!(
            !EXCLUDED.iter().any(|e| e.0 == id),
            "{id} is excluded and pending"
        );
    }
}

#[test]
fn every_rom_command_is_ported_or_documented() {
    let Some(ovy) = overlay10() else {
        return;
    };
    let rd = |a: u32| {
        let o = (a - OVY10) as usize;
        u32::from_le_bytes(ovy[o..o + 4].try_into().unwrap())
    };
    let n = rd(COUNT_WORD);
    assert_eq!(n, 609);
    let ported = commands();
    let (mut nonnull, mut np, mut ne, mut npend, mut nnull) = (0, 0, 0, 0, 0);
    for id in 0..n {
        let w = rd(TABLE + 4 * id);
        let id = id as u16;
        if w == 0 {
            assert!(NULL_SLOTS.contains(&id), "null slot {id} not listed");
            nnull += 1;
            continue;
        }
        nonnull += 1;
        if let Some(&(_, h)) = ported.iter().find(|c| c.0 == id) {
            assert_eq!(
                h,
                w & !1,
                "ported id {id}: handler differs from the ROM table"
            );
            np += 1;
        } else if let Some(e) = EXCLUDED.iter().find(|e| e.0 == id) {
            assert_eq!(e.1, w, "excluded id {id}: handler address");
            ne += 1;
        } else if let Some(p) = PENDING.iter().find(|p| p.0 == id) {
            assert_eq!(p.1, w, "pending id {id}: handler address");
            npend += 1;
        } else {
            panic!("command {id} (handler {w:#x}) is neither ported, excluded nor pending");
        }
    }
    assert_eq!(nnull, NULL_SLOTS.len());
    assert_eq!(np, ported.len(), "every ported id is a non-null slot");
    assert_eq!(ne, EXCLUDED.len());
    assert_eq!(npend, PENDING.len());
    assert_eq!(nonnull, np + ne + npend);
    assert_eq!((nonnull, np, ne, npend), (600, 560, 39, 1));
}
