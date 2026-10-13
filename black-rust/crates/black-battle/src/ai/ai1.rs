//! Overlay 96 trainer-AI functions (0x02187F00-0x0218A574; B14). Specs:
//! `tools/rom-diff/svflow_specs/ai1.spec` (`nowork`, `set battle+96`). Smoke ports only (B14-spec); the B14
//! port job ports the rest. Each handler takes `(vm, ai)` and returns the AI object's status word `ai[0xC4]`.

use super::{rd16, rd32, wr32};

/// Status word of the AI object (the handler return value: 0 continue, 1 yield).
const STATUS: usize = 0xC4;
/// Scratch result word most leaf commands write before `0218a0f4` compares it.
const RESULT: usize = 0x38;

/// Functions of this area that are ported and checked by `tests::svflow_match_rom`.
pub const PORTED: &[u32] = &[
    0x0218_8F08,
    0x0218_8F50,
    0x0218_93B4,
    0x0218_94B4,
    0x0218_94C4,
];

/// C: `FUN_overlay_d_96__02188f08`: command "load the slot score"; `ai[0x38] = ai[2]` (u16), return `ai[0xC4]`.
pub fn cmd_result_from_score(ai: &mut [u8]) -> u32 {
    wr32(ai, RESULT, u32::from(rd16(ai, 2)));
    rd32(ai, STATUS)
}

/// C: `FUN_overlay_d_96__02188f50`: command that only yields its status; return `ai[0xC4]`.
pub fn cmd_status(ai: &[u8]) -> u32 {
    rd32(ai, STATUS)
}

/// C: `FUN_overlay_d_96__021893b4`: command "set the abort flag"; `ai[0x44] |= 2`, return `ai[0xC4]`.
pub fn cmd_set_abort(ai: &mut [u8]) -> u32 {
    ai[0x44] |= 2;
    rd32(ai, STATUS)
}

/// C: `FUN_overlay_d_96__021894b4`: command "copy field +0x98 to the result"; `ai[0x38] = ai[0x98]`, return
/// `ai[0xC4]`.
pub fn cmd_result_from_98(ai: &mut [u8]) -> u32 {
    let v = rd32(ai, 0x98);
    wr32(ai, RESULT, v);
    rd32(ai, STATUS)
}

/// C: `FUN_overlay_d_96__021894c4`: command "copy field +0x9C to the result"; `ai[0x38] = ai[0x9C]`, return
/// `ai[0xC4]`.
pub fn cmd_result_from_9c(ai: &mut [u8]) -> u32 {
    let v = rd32(ai, 0x9C);
    wr32(ai, RESULT, v);
    rd32(ai, STATUS)
}

#[cfg(test)]
mod tests;
