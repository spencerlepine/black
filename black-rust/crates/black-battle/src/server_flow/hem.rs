//! HANDEX manager (0x021C8810-0x021C8918; B10b): the message ("str") record helpers and the
//! push / pop of HANDEX records in the manager at `work + 0x1D74` (layouts in the header of
//! `tools/rom-diff/svflow_specs/hem.spec`).
//!
//! A str record (0x24 bytes): u16 message id at +0; u16 at +2 = bits 0-7 kind, bits 8-14
//! argument count (max 8), bit 15 "the extra word at +0x20 is set"; u32 arguments from +4.

use super::cmd_queue::{alloc_record, free_last_record, word_flag29, word_flag30};
use super::env::{Arg, FlowEnv};
use super::SvFlowWork;

/// Offset of the HANDEX manager (packed header word, records from +4) in the work.
pub const HANDEX_MGR: usize = 0x1D74;

fn flags(rec: &[u8]) -> u16 {
    u16::from_le_bytes([rec[2], rec[3]])
}

fn set_flags(rec: &mut [u8], h: u16) {
    rec[2..4].copy_from_slice(&h.to_le_bytes());
}

/// C: `FUN_overlay_d_93__021c8810`: clears the str record `rec` (`MI_CpuFill8(rec, 0, 0x24)`,
/// then the kind byte again).
pub fn str_clear(rec: &mut [u8]) {
    rec[..0x24].fill(0);
    let h = flags(rec) & 0xFF00;
    set_flags(rec, h);
}

/// C: `FUN_overlay_d_93__021c8828`: 1 when the str record's kind byte is nonzero, else 0.
pub fn str_is_set(rec: &[u8]) -> u32 {
    (flags(rec) & 0xFF != 0) as u32
}

/// C: `sub_021c8838`: initialises the str record `rec`: kind byte `kind & 0xFF`, message id
/// `id`, argument count 0 (bit 15 of the flags halfword is kept).
pub fn str_init(rec: &mut [u8], kind: u32, id: u32) {
    let h = ((flags(rec) & 0xFF00) | (kind as u16 & 0xFF)) & 0x80FF;
    set_flags(rec, h);
    rec[0..2].copy_from_slice(&(id as u16).to_le_bytes());
}

/// C: `FUN_overlay_d_93__021c8860`: appends the argument `v` to the str record `rec`; no-op
/// when it already holds 8.
pub fn str_push(rec: &mut [u8], v: u32) {
    let h = flags(rec);
    let n = (h as u32 >> 8) & 0x7F;
    if n < 8 {
        set_flags(rec, (h & 0x80FF) | (((n + 1) & 0x7F) << 8) as u16);
        let o = 4 + 4 * n as usize;
        rec[o..o + 4].copy_from_slice(&v.to_le_bytes());
    }
}

/// C: unlabeled routine at `0x021C888C`: stores the extra word `v` at +0x20 of the str record
/// and sets flag bit 15, unless the record already holds 8 arguments.
pub fn str_set_extra(rec: &mut [u8], v: u32) {
    let h = flags(rec);
    if (h as u32 >> 8) & 0x7F < 8 {
        rec[0x20..0x24].copy_from_slice(&v.to_le_bytes());
        set_flags(rec, h | 0x8000);
    }
}

/// C: `FUN_overlay_d_93__021c88a4`: allocates a HANDEX record of kind `id` (pokeID `v`) in the
/// manager (`alloc_record(work + 0x1D74, id, v)`). Returns the record's work offset, 0 when the
/// id is unknown or the record does not fit.
pub fn handex_push(w: &mut SvFlowWork, id: u32, v: u32) -> u32 {
    match alloc_record(&mut w.mem[HANDEX_MGR..], id, v) {
        0 => 0,
        ofs => HANDEX_MGR as u32 + ofs,
    }
}

/// C: `FUN_overlay_d_93__021c88b4`: allocates a HANDEX record (`handex_push`) and executes and
/// frees it at once (`handex_pop`). `waddr` is the address of the work (the executor receives the
/// record's address).
pub fn handex_push_exec<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, waddr: u32, id: u32, v: u32) {
    let rec = handex_push(w, id, v);
    handex_pop(env, w, waddr, rec);
}

/// C: `FUN_overlay_d_93__021c88c8`: executes the HANDEX record at work offset `rec` (handex
/// `021c8918`, reached through `env`), then releases it from the manager when it is the last one
/// (`free_last_record(work + 0x1D74, rec)`). `rec` = 0 passes a null record (the original then
/// frees relative to address 0; callers never do that).
pub fn handex_pop<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, waddr: u32, rec: u32) {
    let addr = if rec == 0 { 0 } else { waddr.wrapping_add(rec) };
    env.call(0x021C_8918, &mut [Arg::P, Arg::V(addr)]);
    free_last_record(
        &mut w.mem[HANDEX_MGR..],
        rec.wrapping_sub(HANDEX_MGR as u32),
    );
}

/// C: `FUN_overlay_d_93__021c88e4`: the manager state: 0 when header bit 30 is clear, else 2
/// when bit 29 is set and 1 when it is not.
pub fn handex_state(w: &SvFlowWork) -> u32 {
    let m = &w.mem[HANDEX_MGR..];
    if word_flag30(m) == 0 {
        0
    } else if word_flag29(m) != 0 {
        2
    } else {
        1
    }
}

/// C: unlabeled routine at `0x021C8910`: `ldr r3,[pc]; bx r3` thunk to `handex_state`.
pub fn handex_state_thunk(w: &SvFlowWork) -> u32 {
    handex_state(w)
}

/// Addresses of ported functions; stubs at these addresses become inactive.
pub const PORTED: &[u32] = &[
    0x021C8810, 0x021C8828, 0x021C8838, 0x021C8860, 0x021C888C, 0x021C88A4, 0x021C88B4, 0x021C88C8,
    0x021C88E4, 0x021C8910,
];

#[cfg(test)]
mod tests;
