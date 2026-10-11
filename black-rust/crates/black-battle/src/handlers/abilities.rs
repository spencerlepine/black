//! Ability handlers (`hand_tok`): registry at battle_calc rodata 0x021F0E74.
//!
//! Each handler is C `void handler(factor, flow_work, poke_id, work)`; the doc comment gives
//! the original address, the abilities using it and the event. Event-variable labels are kept
//! numeric (see `event_var`).

use crate::calc::fx_mul;
use crate::event_factor::{EventSystem, FactorId};
use crate::flow::{Battle, FlowHost, HandexBytes};

/// HANDEX work types used below.
mod hx {
    /// Show the ability pop-up.
    pub const ABILITY_POPUP_IN: u32 = 2;
    /// Hide the ability pop-up.
    pub const ABILITY_POPUP_OUT: u32 = 3;
    /// Stat-stage change: +4 stat, +0xC delta (s8), +0xE flag, +0xF count, +0x10 targets.
    pub const RANK_EFFECT: u32 = 0xE;
}

/// Header bit set by handlers that show their own ability pop-up (`2 << 22`).
const HANDEX_HEADER_POPUP: u32 = 2 << 22;

fn is_me<H: FlowHost>(b: &Battle<H>, label: u16, me: u8) -> bool {
    me as u32 == b.var(label) as u32
}

/// 0x021D85F4: Intimidate (22), events 0x55/0x8A: lowers the Attack of every opponent.
pub fn h_021d85f4<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) {
        return;
    }
    let pos = b.host.poke_pos(me as u32);
    let mut targets = [0u8; 8];
    let n = b.host.expand_poke_ids((pos | 0x100) & 0xFFFF, &mut targets);
    if n == 0 {
        return;
    }
    b.host.handex_push_pop(hx::ABILITY_POPUP_IN, me as u32);
    let w = b.host.handex_push(hx::RANK_EFFECT, me as u32);
    let buf = b.host.handex(w);
    buf.put_u32(4, 1);
    buf.put_u8(0xC, 0xFF);
    buf.put_u8(0xE, 1);
    buf.put_u8(0xF, n as u8);
    for i in 0..n as usize {
        buf.put_u8(0x10 + i, targets[i]);
    }
    b.host.handex_pop(w);
    b.host.handex_push_pop(hx::ABILITY_POPUP_OUT, me as u32);
}

/// 0x021D8684: Inner Focus (39), event 0x6D: sets var 0x41 (no flinch).
pub fn h_021d8684<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me) {
        b.rewrite_var(0x41, 1);
    }
}

/// 0x021D86A8: Steadfast (80), event 0x21: when var 0x22 is 6 (flinch), raises Speed.
pub fn h_021d86a8<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if b.var(0x22) != 6 || !is_me(b, 2, me) {
        return;
    }
    let w = b.host.handex_push(hx::RANK_EFFECT, me as u32);
    let buf = b.host.handex(w);
    let h = buf.get_u32(0);
    buf.put_u32(0, h | HANDEX_HEADER_POPUP);
    buf.put_u32(4, 5);
    buf.put_u8(0xC, 1);
    buf.put_u8(0xE, 0);
    buf.put_u8(0xF, 1);
    buf.put_u8(0x10, me);
    b.host.handex_pop(w);
}

/// 0x021D86FC: Thick Fat (47), event 0x3B: halves Ice (14) and Fire (9) move power.
pub fn h_021d86fc<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 4, me) {
        return;
    }
    let ty = b.var(0x16) as u8;
    if ty == 14 || ty == 9 {
        b.mul_var(0x35, 0x800);
    }
}

/// 0x021D8734: Huge Power / Pure Power (37, 74), event 0x3B: doubles physical attack.
pub fn h_021d8734<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let mv = b.var(0x12) as u16 as u32;
    if b.host.move_category(mv) == 1 {
        b.mul_var(0x35, 0x2000);
    }
}

/// 0x021D8760: doubles var 0x35 for the Pokemon in var 2 while the weather is 2 (rain).
pub fn h_021d8760<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && b.host.weather() == 2 {
        b.mul_var(0x35, 0x2000);
    }
}

/// 0x021D8794: doubles var 0x35 for the Pokemon in var 2 while the weather is 1 (sun).
pub fn h_021d8794<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && b.host.weather() == 1 {
        b.mul_var(0x35, 0x2000);
    }
}

/// 0x021D87C8: with a major status, var 0x35 x1.5 and var 0x51 cleared (Quick Feet).
pub fn h_021d87c8<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) {
        return;
    }
    if b.mon(me as u32).major_status() != 0 {
        b.mul_var(0x35, 0x1800);
        b.rewrite_var(0x51, 0);
    }
}

/// 0x021D8808: while condition 6 (confusion) is active, var 0x35 x0.5 (Tangled Feet).
pub fn h_021d8808<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me) && b.mon(me as u32).has_sick(6) {
        b.mul_var(0x35, 0x800);
    }
}

/// 0x021D8844: physical moves get var 0x35 x0.8 (Hustle accuracy).
pub fn h_021d8844<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let mv = b.var(0x12) as u16 as u32;
    if b.host.move_category(mv) == 1 {
        b.mul_var(0x35, 0xCCD);
    }
}

/// 0x021D8874: physical moves get var 0x33 x1.5, rounded (Hustle attack).
pub fn h_021d8874<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let mv = b.var(0x12) as u16 as u32;
    if b.host.move_category(mv) == 1 {
        let v = fx_mul(b.var(0x33) as u32, 0x1800);
        b.rewrite_var(0x33, v as i32);
    }
}

/// Original addresses of the ported handlers (also the differential test order).
pub const PORTED: &[u32] = &[
    0x021D85F4, 0x021D8684, 0x021D86A8, 0x021D86FC, 0x021D8734, 0x021D8760, 0x021D8794, 0x021D87C8,
    0x021D8808, 0x021D8844, 0x021D8874,
];

/// Ported ability handlers by original address.
pub fn handler<H: FlowHost>(addr: u32) -> Option<crate::flow::Handler<H>> {
    Some(match addr & !1 {
        0x021D85F4 => h_021d85f4::<H>,
        0x021D8684 => h_021d8684::<H>,
        0x021D86A8 => h_021d86a8::<H>,
        0x021D86FC => h_021d86fc::<H>,
        0x021D8734 => h_021d8734::<H>,
        0x021D8760 => h_021d8760::<H>,
        0x021D8794 => h_021d8794::<H>,
        0x021D87C8 => h_021d87c8::<H>,
        0x021D8808 => h_021d8808::<H>,
        0x021D8844 => h_021d8844::<H>,
        0x021D8874 => h_021d8874::<H>,
        _ => return None,
    })
}

#[cfg(test)]
mod tests;
