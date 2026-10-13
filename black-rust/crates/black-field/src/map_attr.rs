//! Map attribute accessors and predicates of overlay 21 (`0x021AB0D4..0x021AB5F4`, the tail of F1 row 21.30;
//! MAPATTR of the original field code). An attribute word is `flags << 16 | value`: flags bit 0 = blocked, bit 1 =
//! water, bit 2 = wild encounters; the value is the tile kind (grass, long grass, water, sand, ice, ...).
//!
//! F5a-spec specced all 68 functions (`tools/rom-diff/specs/black-field/map_attr.spec`); this module holds the
//! smoke ports. The predicates compare the whole 32-bit argument, as the originals do.

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021A_B0F0,
    0x021A_B100,
    0x021A_B250,
    0x021A_B270,
    0x021A_B29C,
    0x021A_B504,
];

/// C: `FUN_overlay_d_21__021ab0f0` (unlabeled routine at `0x021AB0F0`): the attribute value (low half).
pub fn value(a: u32) -> u32 {
    a & 0xFFFF
}

/// C: `FUN_overlay_d_21__021ab100` (unlabeled routine at `0x021AB100`): a valid attribute: not 0xFFFFFFFF (no
/// map) and its [`value`] not 0xFF.
pub fn valid(a: u32) -> bool {
    a != u32::MAX && value(a) != 0xFF
}

/// C: `FUN_overlay_d_21__021ab250` (unlabeled routine at `0x021AB250`): value 6, 0x22 or 7.
pub fn is_long_grass_a(v: u32) -> bool {
    v == 6 || v == 0x22 || v == 7
}

/// C: `FUN_overlay_d_21__021ab270` (unlabeled routine at `0x021AB270`): value 9.
pub fn is_long_grass_b(v: u32) -> bool {
    v == 9
}

/// C: `FUN_overlay_d_21__021ab29c` (unlabeled routine at `0x021AB29C`): long grass (dark grass, the double-battle
/// encounter type 1): [`is_long_grass_a`] or [`is_long_grass_b`].
pub fn is_long_grass(v: u32) -> bool {
    is_long_grass_a(v) || is_long_grass_b(v)
}

/// C: `FUN_overlay_d_21__021ab504` (unlabeled routine at `0x021AB504`): the effect-encounter candidate kind of an
/// attribute word: value 0x20 -> 7 (whatever the flags); without flag bit 2 -> 8 (none); value 4 -> 0, 5 -> 1,
/// 0x21 -> 2, 8 -> 3, 10 -> 4, 0x3D -> 5, 0x3F / 0x43 -> 6, any other -> 8.
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

#[cfg(test)]
mod tests;
