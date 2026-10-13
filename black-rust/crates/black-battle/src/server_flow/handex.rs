//! HANDEX execution (0x021C8918-0x021CA8CC; B10b): the executor `021c8918` (switch over the
//! record id 0x00-0x3B), its 59 procs, the message helper `021c99a0` and `021ca7f4`. Specs and
//! the record layout: `tools/rom-diff/svflow_specs/handex.spec` (header + the case table on the
//! `021c8918` block).
//!
//! B10bc-spec smoke ports: procs 0x03 (`8d0c`), 0x25 (`9f0c`), 0x38 (`a738`), 0x3A (`a7c4`) and
//! 0x3B (`a7d4`). A proc gets the work, the record bytes and (for `q`) the command queue behind
//! `work + 0xC` plus the put_cmd scratch words at 0x021F3B44; callees outside the area go through
//! [`FlowEnv`], cmd_queue functions are called directly.

use super::cmd_queue;
use super::env::{Arg, FlowEnv};
use super::SvFlowWork;

/// pokeID field of a HANDEX record header (bits 8-12 of word 0).
fn rec_poke_id(rec: &[u8]) -> u32 {
    (u32::from_le_bytes(rec[0..4].try_into().unwrap()) >> 8) & 0x1F
}

/// `021b9934(work[8], id)`: the BattleMon of a pokeID.
fn mon_of<E: FlowEnv>(env: &mut E, w: &SvFlowWork, id: u32) -> u32 {
    env.call(0x021B_9934, &mut [Arg::V(w.u32_at(8)), Arg::V(id)])
}

/// C: `FUN_overlay_d_93__021c8d0c`: HANDEX proc 0x03: core5 `021c5a34(work, mon)` for the
/// record's pokeID; returns 1.
pub fn proc_03<E: FlowEnv>(env: &mut E, w: &SvFlowWork, rec: &[u8]) -> u32 {
    let mon = mon_of(env, w, rec_poke_id(rec));
    env.call(0x021C_5A34, &mut [Arg::P, Arg::V(mon)]);
    1
}

/// C: `FUN_overlay_d_93__021c9f0c`: HANDEX proc 0x25: puts command 0x1F (`rec[4]`, `rec[5]`,
/// `rec[6]`, `rec[7]`, u16 `rec[8]`) on the queue `q`, then `021d5734(mon(rec[4]), rec[5],
/// u16 rec[8], rec[6], rec[7])`; returns 1.
pub fn proc_25<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    rec: &[u8],
    q: &mut [u8],
    scratch: &mut [u8],
) -> u32 {
    let mon = mon_of(env, w, rec[4] as u32);
    let v8 = u16::from_le_bytes([rec[8], rec[9]]) as u32;
    let args = [
        rec[4] as u32,
        rec[5] as u32,
        rec[6] as u32,
        rec[7] as u32,
        v8,
    ];
    cmd_queue::put_cmd(env, q, 0x1F, &args, scratch);
    env.call(
        0x021D_5734,
        &mut [
            Arg::V(mon),
            Arg::V(rec[5] as u32),
            Arg::V(v8),
            Arg::V(rec[6] as u32),
            Arg::V(rec[7] as u32),
        ],
    );
    1
}

/// C: `FUN_overlay_d_93__021ca738`: HANDEX proc 0x38: puts command 0x56 (0) on the queue `q`;
/// returns 1 (the record is not read).
pub fn proc_38<E: FlowEnv>(env: &mut E, q: &mut [u8], scratch: &mut [u8]) -> u32 {
    cmd_queue::put_cmd(env, q, 0x56, &[0], scratch);
    1
}

/// C: `FUN_overlay_d_93__021ca7c4`: HANDEX proc 0x3A: byte +4 of the record `d` behind
/// `work + 0x4A8` := `rec[4]`; returns 1.
pub fn proc_3a(rec: &[u8], d: &mut [u8]) -> u32 {
    d[4] = rec[4];
    1
}

/// C: `FUN_overlay_d_93__021ca7d4`: HANDEX proc 0x3B: sets bit 0 of byte +5 of the record `d`
/// behind `work + 0x4A8`; returns 1 (the record is not read).
pub fn proc_3b(d: &mut [u8]) -> u32 {
    if d[5] & 1 == 0 {
        d[5] = (d[5] & !1) | 1;
    }
    1
}

/// Addresses of ported functions; stubs at these addresses become inactive.
pub const PORTED: &[u32] = &[0x021C8D0C, 0x021C9F0C, 0x021CA738, 0x021CA7C4, 0x021CA7D4];

#[cfg(test)]
mod tests;
