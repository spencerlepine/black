//! Map attribute accessors and predicates of overlay 21 (`0x021AB0D4..0x021AB5F4`, the tail of F1 row 21.30;
//! MAPATTR of the original field code). An attribute word is `flags << 16 | value`: flags bit 0 = blocked, bit 1 =
//! water, bit 2 = wild encounters; the value is the tile kind (grass, long grass, water, sand, ice, ...).
//!
//! F5a-spec specced all 68 functions (`tools/rom-diff/specs/black-field/map_attr.spec`); F5a ported all of them.
//! The predicates compare the whole 32-bit argument, as the originals do (`cmp r0, #k`), except the range checks
//! written as `(v - k) & 0xFFFF <= 1`, which look at the low half only. The meaning of most attribute values is
//! not known (UNVERIFIED): those predicates are named after the value they test.

use crate::hooks::{CArg, FieldEnv};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021A_B0D4,
    0x021A_B0F0,
    0x021A_B0F8,
    0x021A_B100,
    0x021A_B11C,
    0x021A_B140,
    0x021A_B158,
    0x021A_B164,
    0x021A_B170,
    0x021A_B17C,
    0x021A_B188,
    0x021A_B194,
    0x021A_B1A0,
    0x021A_B1AC,
    0x021A_B1B8,
    0x021A_B1C4,
    0x021A_B1D0,
    0x021A_B1DC,
    0x021A_B1E8,
    0x021A_B1F4,
    0x021A_B200,
    0x021A_B20C,
    0x021A_B218,
    0x021A_B224,
    0x021A_B230,
    0x021A_B23C,
    0x021A_B250,
    0x021A_B264,
    0x021A_B270,
    0x021A_B27C,
    0x021A_B29C,
    0x021A_B2BC,
    0x021A_B2DC,
    0x021A_B2E8,
    0x021A_B2F8,
    0x021A_B304,
    0x021A_B310,
    0x021A_B31C,
    0x021A_B328,
    0x021A_B334,
    0x021A_B348,
    0x021A_B358,
    0x021A_B364,
    0x021A_B370,
    0x021A_B37C,
    0x021A_B390,
    0x021A_B3B0,
    0x021A_B3D4,
    0x021A_B3E0,
    0x021A_B3EC,
    0x021A_B3F8,
    0x021A_B404,
    0x021A_B410,
    0x021A_B41C,
    0x021A_B428,
    0x021A_B434,
    0x021A_B440,
    0x021A_B44C,
    0x021A_B458,
    0x021A_B474,
    0x021A_B490,
    0x021A_B4A4,
    0x021A_B4B8,
    0x021A_B4EC,
    0x021A_B4F8,
    0x021A_B504,
    0x021A_B580,
    0x021A_B5AC,
];

/// `FUN_0218DAEC(fsys, &pos, &out)`: the map height / attribute query (returns 1 when `out` was filled; the
/// attribute word is `out + 8`).
const MAP_QUERY: u32 = 0x0218_DAEC;

/// C: `FUN_overlay_d_21__021ab0d4` (unlabeled routine at `0x021AB0D4`): the attribute word at the position `pos`
/// (a 12-byte VecFx32): `FUN_0218DAEC(fsys, pos, &out)` into a 16-byte local; when it returns 1, `out + 8`, else
/// 0xFFFFFFFF.
pub fn at<E: FieldEnv>(env: &mut E, fsys: u32, pos: u32) -> u32 {
    let out = env.mem().push_local(0x10);
    let r = env.call(
        MAP_QUERY,
        &mut [CArg::V(fsys), CArg::In(pos, 12), CArg::Out(out, 0x10)],
    );
    let a = if r == 1 {
        env.r32(out.wrapping_add(8))
    } else {
        u32::MAX
    };
    env.mem().pop_local(out);
    a
}

/// C: `FUN_overlay_d_21__021ab0f0` (unlabeled routine at `0x021AB0F0`): the attribute value (low half).
pub fn value(a: u32) -> u32 {
    a & 0xFFFF
}

/// C: `FUN_overlay_d_21__021ab0f8` (unlabeled routine at `0x021AB0F8`): the attribute flags (high half).
pub fn flags(a: u32) -> u32 {
    a >> 16
}

/// C: `FUN_overlay_d_21__021ab100` (unlabeled routine at `0x021AB100`): a valid attribute: not 0xFFFFFFFF (no
/// map) and its [`value`] not 0xFF.
pub fn valid(a: u32) -> bool {
    a != u32::MAX && value(a) != 0xFF
}

/// C: `FUN_overlay_d_21__021ab11c` (unlabeled routine at `0x021AB11C`): blocked: not [`valid`], or flags bit 0.
pub fn blocked(a: u32) -> bool {
    let f = flags(a);
    !valid(a) || f & 1 != 0
}

/// C: `FUN_overlay_d_21__021ab140` (unlabeled routine at `0x021AB140`): the low half of `v` is 8 or 9
/// (`(v - 8) & 0xFFFF <= 1`).
pub fn is_8_or_9(v: u32) -> bool {
    v.wrapping_add(0xFFF8) & 0xFFFF <= 1
}

/// C: `FUN_overlay_d_21__021ab158` (unlabeled routine at `0x021AB158`): `v == 0x74`.
pub fn is_74(v: u32) -> bool {
    v == 0x74
}

/// C: `FUN_overlay_d_21__021ab164` (unlabeled routine at `0x021AB164`): `v == 0x75`.
pub fn is_75(v: u32) -> bool {
    v == 0x75
}

/// C: `FUN_overlay_d_21__021ab170` (unlabeled routine at `0x021AB170`): `v == 0x73`.
pub fn is_73(v: u32) -> bool {
    v == 0x73
}

/// C: `FUN_overlay_d_21__021ab17c` (unlabeled routine at `0x021AB17C`): `v == 0x72`.
pub fn is_72(v: u32) -> bool {
    v == 0x72
}

/// C: `FUN_overlay_d_21__021ab188` (unlabeled routine at `0x021AB188`): `v == 0x7C`.
pub fn is_7c(v: u32) -> bool {
    v == 0x7C
}

/// C: `FUN_overlay_d_21__021ab194` (unlabeled routine at `0x021AB194`): `v == 0xD4`.
pub fn is_d4(v: u32) -> bool {
    v == 0xD4
}

/// C: unlabeled routine at `0x021AB1A0`: `v == 0xD6`.
pub fn is_d6(v: u32) -> bool {
    v == 0xD6
}

/// C: unlabeled routine at `0x021AB1AC`: `v == 0xD7`.
pub fn is_d7(v: u32) -> bool {
    v == 0xD7
}

/// C: `FUN_overlay_d_21__021ab1b8` (unlabeled routine at `0x021AB1B8`): `v == 0xD8`.
pub fn is_d8(v: u32) -> bool {
    v == 0xD8
}

/// C: `FUN_overlay_d_21__021ab1c4` (unlabeled routine at `0x021AB1C4`): `v == 0xE2`.
pub fn is_e2(v: u32) -> bool {
    v == 0xE2
}

/// C: `FUN_overlay_d_21__021ab1d0` (unlabeled routine at `0x021AB1D0`): `v == 0xD9`.
pub fn is_d9(v: u32) -> bool {
    v == 0xD9
}

/// C: `FUN_overlay_d_21__021ab1dc` (unlabeled routine at `0x021AB1DC`): `v == 0xDA`.
pub fn is_da(v: u32) -> bool {
    v == 0xDA
}

/// C: `FUN_overlay_d_21__021ab1e8` (unlabeled routine at `0x021AB1E8`): `v == 0xDB`.
pub fn is_db(v: u32) -> bool {
    v == 0xDB
}

/// C: `FUN_overlay_d_21__021ab1f4` (unlabeled routine at `0x021AB1F4`): `v == 0xDC`.
pub fn is_dc(v: u32) -> bool {
    v == 0xDC
}

/// C: `FUN_overlay_d_21__021ab200` (unlabeled routine at `0x021AB200`): `v == 0xDD`.
pub fn is_dd(v: u32) -> bool {
    v == 0xDD
}

/// C: `FUN_overlay_d_21__021ab20c` (unlabeled routine at `0x021AB20C`): `v == 0xDE`.
pub fn is_de(v: u32) -> bool {
    v == 0xDE
}

/// C: `FUN_overlay_d_21__021ab218` (unlabeled routine at `0x021AB218`): `v == 0xDF`.
pub fn is_df(v: u32) -> bool {
    v == 0xDF
}

/// C: `FUN_overlay_d_21__021ab224` (unlabeled routine at `0x021AB224`): `v == 0xE0`.
pub fn is_e0(v: u32) -> bool {
    v == 0xE0
}

/// C: `FUN_overlay_d_21__021ab230` (unlabeled routine at `0x021AB230`): `v == 0xE1`.
pub fn is_e1(v: u32) -> bool {
    v == 0xE1
}

/// C: `FUN_overlay_d_21__021ab23c` (unlabeled routine at `0x021AB23C`): `v` is 4, 0x21 or 5 (the short-grass
/// values; UNVERIFIED name).
pub fn is_grass_a(v: u32) -> bool {
    v == 4 || v == 0x21 || v == 5
}

/// C: `FUN_overlay_d_21__021ab250` (unlabeled routine at `0x021AB250`): value 6, 0x22 or 7.
pub fn is_long_grass_a(v: u32) -> bool {
    v == 6 || v == 0x22 || v == 7
}

/// C: `FUN_overlay_d_21__021ab264` (unlabeled routine at `0x021AB264`): `v == 8`.
pub fn is_grass_b(v: u32) -> bool {
    v == 8
}

/// C: `FUN_overlay_d_21__021ab270` (unlabeled routine at `0x021AB270`): value 9.
pub fn is_long_grass_b(v: u32) -> bool {
    v == 9
}

/// C: `FUN_overlay_d_21__021ab27c` (unlabeled routine at `0x021AB27C`): [`is_grass_a`] or [`is_grass_b`].
pub fn is_short_grass(v: u32) -> bool {
    is_grass_a(v) || is_grass_b(v)
}

/// C: `FUN_overlay_d_21__021ab29c` (unlabeled routine at `0x021AB29C`): long grass (dark grass, the double-battle
/// encounter type 1): [`is_long_grass_a`] or [`is_long_grass_b`].
pub fn is_long_grass(v: u32) -> bool {
    is_long_grass_a(v) || is_long_grass_b(v)
}

/// C: `FUN_overlay_d_21__021ab2bc` (unlabeled routine at `0x021AB2BC`): grass: [`is_short_grass`] or
/// [`is_long_grass`].
pub fn is_grass(v: u32) -> bool {
    is_short_grass(v) || is_long_grass(v)
}

/// C: `FUN_overlay_d_21__021ab2dc` (unlabeled routine at `0x021AB2DC`): `v == 0x1D`.
pub fn is_1d(v: u32) -> bool {
    v == 0x1D
}

/// C: `FUN_overlay_d_21__021ab2e8` (unlabeled routine at `0x021AB2E8`): `v` is 0x41 or 0x44.
pub fn is_41_or_44(v: u32) -> bool {
    v == 0x41 || v == 0x44
}

/// C: unlabeled routine at `0x021AB2F8`: `v == 0x14`.
pub fn is_14(v: u32) -> bool {
    v == 0x14
}

/// C: `FUN_overlay_d_21__021ab304` (unlabeled routine at `0x021AB304`): `v == 0x17`.
pub fn is_17(v: u32) -> bool {
    v == 0x17
}

/// C: `FUN_overlay_d_21__021ab310` (unlabeled routine at `0x021AB310`): `v == 0x40`.
pub fn is_40(v: u32) -> bool {
    v == 0x40
}

/// C: `FUN_overlay_d_21__021ab31c` (unlabeled routine at `0x021AB31C`): `v == 0xE`.
pub fn is_0e(v: u32) -> bool {
    v == 0xE
}

/// C: `FUN_overlay_d_21__021ab328` (unlabeled routine at `0x021AB328`): `v == 0xF`.
pub fn is_0f(v: u32) -> bool {
    v == 0xF
}

/// C: `FUN_overlay_d_21__021ab334` (unlabeled routine at `0x021AB334`): [`is_0e`].
pub fn is_0e_b(v: u32) -> bool {
    is_0e(v)
}

/// C: `FUN_overlay_d_21__021ab348` (unlabeled routine at `0x021AB348`): `v` is 0x18 or 0x10.
pub fn is_18_or_10(v: u32) -> bool {
    v == 0x18 || v == 0x10
}

/// C: unlabeled routine at `0x021AB358`: `v == 0x10`.
pub fn is_10(v: u32) -> bool {
    v == 0x10
}

/// C: `FUN_overlay_d_21__021ab364` (unlabeled routine at `0x021AB364`): `v == 0x1C`.
pub fn is_1c(v: u32) -> bool {
    v == 0x1C
}

/// C: `FUN_overlay_d_21__021ab370` (unlabeled routine at `0x021AB370`): `v == 0x19`.
pub fn is_19(v: u32) -> bool {
    v == 0x19
}

/// C: `FUN_overlay_d_21__021ab37c` (unlabeled routine at `0x021AB37C`): `v` is 0xC, 0xB or 0x23.
pub fn is_0c_0b_23(v: u32) -> bool {
    v == 0xC || v == 0xB || v == 0x23
}

/// C: `FUN_overlay_d_21__021ab390` (unlabeled routine at `0x021AB390`): [`is_19`] or [`is_0c_0b_23`].
pub fn is_19_0c_0b_23(v: u32) -> bool {
    is_19(v) || is_0c_0b_23(v)
}

/// C: `FUN_overlay_d_21__021ab3b0` (unlabeled routine at `0x021AB3B0`): `v` is 0x3D, 0x3E or 0x3F, or its low half
/// is 0x42 or 0x43 (`(v - 0x42) & 0xFFFF <= 1`).
pub fn is_3d_to_43(v: u32) -> bool {
    v == 0x3D || v == 0x3E || v == 0x3F || v.wrapping_add(0xFFBE) & 0xFFFF <= 1
}

/// C: `FUN_overlay_d_21__021ab3d4` (unlabeled routine at `0x021AB3D4`): `v == 2`.
pub fn is_02(v: u32) -> bool {
    v == 2
}

/// C: `FUN_overlay_d_21__021ab3e0` (unlabeled routine at `0x021AB3E0`): `v == 0x1E`.
pub fn is_1e(v: u32) -> bool {
    v == 0x1E
}

/// C: `FUN_overlay_d_21__021ab3ec` (unlabeled routine at `0x021AB3EC`): `v == 0x30`.
pub fn is_30(v: u32) -> bool {
    v == 0x30
}

/// C: unlabeled routine at `0x021AB3F8` (no caller in the field set): `v == 0x32`.
pub fn is_32(v: u32) -> bool {
    v == 0x32
}

/// C: `FUN_overlay_d_21__021ab404` (unlabeled routine at `0x021AB404`): `v == 0xBE`.
pub fn is_be(v: u32) -> bool {
    v == 0xBE
}

/// C: `FUN_overlay_d_21__021ab410` (unlabeled routine at `0x021AB410`): `v == 0xBF`.
pub fn is_bf(v: u32) -> bool {
    v == 0xBF
}

/// C: `FUN_overlay_d_21__021ab41c` (unlabeled routine at `0x021AB41C`): `v == 0xC`.
pub fn is_0c(v: u32) -> bool {
    v == 0xC
}

/// C: unlabeled routine at `0x021AB428`: `v == 5`.
pub fn is_05(v: u32) -> bool {
    v == 5
}

/// C: `FUN_overlay_d_21__021ab434` (unlabeled routine at `0x021AB434`): `v == 7`.
pub fn is_07(v: u32) -> bool {
    v == 7
}

/// C: `FUN_overlay_d_21__021ab440` (unlabeled routine at `0x021AB440`): `v == 0x1B`.
pub fn is_1b(v: u32) -> bool {
    v == 0x1B
}

/// C: `FUN_overlay_d_21__021ab44c` (unlabeled routine at `0x021AB44C`): `v == 0x33`.
pub fn is_33(v: u32) -> bool {
    v == 0x33
}

/// C: unlabeled routine at `0x021AB458`: `v == 0x53`, or its low half is 0x55 or 0x56 (`(v - 0x55) & 0xFFFF <= 1`).
pub fn is_53_55_56(v: u32) -> bool {
    v == 0x53 || v.wrapping_add(0xFFAB) & 0xFFFF <= 1
}

/// C: `FUN_overlay_d_21__021ab474` (unlabeled routine at `0x021AB474`): `v == 0x54`, or its low half is 0x57 or
/// 0x58 (`(v - 0x57) & 0xFFFF <= 1`).
pub fn is_54_57_58(v: u32) -> bool {
    v == 0x54 || v.wrapping_add(0xFFA9) & 0xFFFF <= 1
}

/// C: `FUN_overlay_d_21__021ab490` (unlabeled routine at `0x021AB490`): `v` is 0x52, 0x56 or 0x58.
pub fn is_52_56_58(v: u32) -> bool {
    v == 0x52 || v == 0x56 || v == 0x58
}

/// C: `FUN_overlay_d_21__021ab4a4` (unlabeled routine at `0x021AB4A4`): `v` is 0x51, 0x55 or 0x57.
pub fn is_51_55_57(v: u32) -> bool {
    v == 0x51 || v == 0x55 || v == 0x57
}

/// C: `FUN_overlay_d_21__021ab4b8` (unlabeled routine at `0x021AB4B8`): [`is_8_or_9`], [`is_be`], [`is_bf`] or
/// [`is_0f`] (in that order).
pub fn is_8_9_be_bf_0f(v: u32) -> bool {
    is_8_or_9(v) || is_be(v) || is_bf(v) || is_0f(v)
}

/// C: `FUN_overlay_d_21__021ab4ec` (unlabeled routine at `0x021AB4EC`): `v == 0x21`.
pub fn is_21(v: u32) -> bool {
    v == 0x21
}

/// C: `FUN_overlay_d_21__021ab4f8` (unlabeled routine at `0x021AB4F8`): `v == 0x22`.
pub fn is_22(v: u32) -> bool {
    v == 0x22
}

/// C: `FUN_overlay_d_21__021ab504` (unlabeled routine at `0x021AB504`): the effect-encounter candidate kind of an
/// attribute word: value 0x20 -> 7 (whatever the flags); without flag bit 2 -> 8 (none); value 4 -> 0, 5 -> 1,
/// 0x21 -> 2, 8 -> 3, 10 -> 4, 0x3D -> 5, 0x3F / 0x43 -> 6, any other -> 8 (jump table for 0..10).
pub fn effect_kind(a: u32) -> u32 {
    let v = a & 0xFFFF;
    let flags = a >> 16;
    if v == 0x20 {
        return 7;
    }
    if flags & 4 == 0 {
        return 8;
    }
    match v {
        4 => 0,
        5 => 1,
        0x21 => 2,
        8 => 3,
        10 => 4,
        0x3D => 5,
        0x3F | 0x43 => 6,
        _ => 8,
    }
}

/// ROM `0x021D8EA0`: 37 pairs (attribute value, battle ground kind).
pub const GROUND_KINDS: [(u8, u8); 37] = [
    (0x00, 4),
    (0x02, 2),
    (0x03, 1),
    (0x04, 5),
    (0x05, 5),
    (0x06, 5),
    (0x07, 5),
    (0x08, 5),
    (0x09, 5),
    (0x0A, 10),
    (0x0B, 8),
    (0x0C, 8),
    (0x0E, 7),
    (0x0F, 7),
    (0x10, 13),
    (0x11, 4),
    (0x13, 4),
    (0x14, 11),
    (0x16, 14),
    (0x17, 12),
    (0x18, 13),
    (0x19, 8),
    (0x1B, 4),
    (0x1C, 9),
    (0x1E, 3),
    (0x1F, 0),
    (0x20, 4),
    (0x21, 5),
    (0x22, 5),
    (0x23, 8),
    (0x30, 10),
    (0x3D, 6),
    (0x3F, 6),
    (0x43, 6),
    (0x7C, 8),
    (0xA1, 1),
    (0xBE, 1),
];

/// C: `FUN_overlay_d_21__021ab580` (unlabeled routine at `0x021AB580`): the battle ground kind of the attribute
/// value `v`: the kind of the first [`GROUND_KINDS`] pair whose value byte equals `v` (the whole register is
/// compared), else 0.
pub fn ground_kind(v: u32) -> u32 {
    GROUND_KINDS
        .iter()
        .find(|p| p.0 as u32 == v)
        .map_or(0, |p| p.1 as u32)
}

/// C: `FUN_overlay_d_21__021ab5ac` (unlabeled routine at `0x021AB5AC`): can the player move from the attribute
/// `a` onto `b` (water entry; UNVERIFIED name): 0 when `a` is [`blocked`]; else 1 when `b` has the water flag
/// (bit 1) and is not [`blocked`], or when `b`'s value is 0x41 / 0x44 ([`is_41_or_44`]).
pub fn enters(a: u32, b: u32) -> bool {
    let fb = flags(b);
    let vb = value(b);
    if blocked(a) {
        return false;
    }
    (fb & 2 != 0 && !blocked(b)) || is_41_or_44(vb)
}

#[cfg(test)]
mod tests;
