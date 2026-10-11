//! Ability handlers (`hand_tok`): registry at battle_calc rodata 0x021F0E74.
//!
//! Each handler is C `void handler(factor, flow_work, poke_id, work)`; the doc comment gives
//! the original address, the abilities using it and the event. Event-variable labels are kept
//! numeric (see `event_var`).

use crate::battle_mon::value;
use crate::calc::{effectiveness_class, fx_mul, u32_div};
use crate::event_factor::{EventSystem, FactorId};
use crate::flow::{str_param_add_arg, str_param_setup, Battle, FlowHost, HandexBytes};

/// HANDEX work types used below.
mod hx {
    /// Show the ability pop-up.
    pub const ABILITY_POPUP_IN: u32 = 2;
    /// Hide the ability pop-up.
    pub const ABILITY_POPUP_OUT: u32 = 3;
    /// Message: +4 string parameter.
    pub const MESSAGE: u32 = 4;
    /// Stat-stage change: +4 stat, +0xC delta (s8), +0xE flag, +0xF count, +0x10 targets.
    pub const RANK_EFFECT: u32 = 0xE;
}

/// Header bit asking the message work to show the ability pop-up (`4 << 21`, same bit as
/// [`HANDEX_HEADER_POPUP`]).
const HANDEX_HEADER_POPUP_MSG: u32 = 4 << 21;

fn work(sys: &mut EventSystem, f: FactorId) -> &mut [u32; 7] {
    &mut sys.factor_mut(f).work
}

fn is_physical<H: FlowHost>(b: &mut Battle<H>) -> bool {
    let mv = b.var(0x12) as u16 as u32;
    b.host.move_category(mv) == 1
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
    for (i, &t) in targets.iter().enumerate().take(n as usize) {
        buf.put_u8(0x10 + i, t);
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

/// 0x021D88E8: while work word 3 is set, var 0x35 x0.5 for the Pokemon in var 2.
pub fn h_021d88e8<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && work(sys, f)[3] != 0 {
        b.mul_var(0x35, 0x800);
    }
}

/// 0x021D890C: while work word 3 is set, physical moves get var 0x35 x0.5.
pub fn h_021d890c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && work(sys, f)[3] != 0 && is_physical(b) {
        b.mul_var(0x35, 0x800);
    }
}

/// C: `FUN_overlay_d_93__021d8974`: starts the countdown (work words 0 and 3 set, 1 and 2
/// cleared) and shows message 0x1F0 with the ability pop-up.
fn start_countdown<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    let w = work(sys, f);
    w[0] = 1;
    w[3] = 1;
    w[1] = 0;
    w[2] = 0;
    let id = b.host.handex_push(hx::MESSAGE, me as u32);
    let buf = b.host.handex(id);
    let h = buf.get_u32(0);
    buf.put_u32(0, h | HANDEX_HEADER_POPUP_MSG);
    str_param_setup(buf, 4, 2, 0x1F0);
    str_param_add_arg(buf, 4, me as u32);
    b.host.handex_pop(id);
}

/// 0x021D8940: starts the countdown for the Pokemon in var 2.
pub fn h_021d8940<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        start_countdown(sys, f, b, me);
    }
}

/// 0x021D8964: starts the countdown unless already started.
pub fn h_021d8964<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if work(sys, f)[0] == 0 {
        start_countdown(sys, f, b, me);
    }
}

/// 0x021D89B8: counts the Pokemon's turns (work word 1) while its queued action is not of
/// kind 3; after five, shows message 0x1F3 and ends (word 3 cleared, word 2 set). Slow Start.
pub fn h_021d89b8<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) || work(sys, f)[2] != 0 {
        return;
    }
    if (work(sys, f)[1] as i32) < 5 {
        let mut action = 0;
        if !b.host.find_action(me as u32, &mut action) || action & 0xF == 3 {
            return;
        }
        work(sys, f)[1] = work(sys, f)[1].wrapping_add(1);
    }
    if (work(sys, f)[1] as i32) < 5 {
        return;
    }
    let id = b.host.handex_push(hx::MESSAGE, me as u32);
    let buf = b.host.handex(id);
    str_param_setup(buf, 4, 2, 0x1F3);
    str_param_add_arg(buf, 4, me as u32);
    b.host.handex_pop(id);
    let w = work(sys, f);
    w[3] = 0;
    w[2] = 1;
}

/// 0x021D8A3C: Compound Eyes: var 0x35 x1.3 for the attacker.
pub fn h_021d8a3c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        b.mul_var(0x35, 0x14CD);
    }
}

/// 0x021D8A64: Sand Veil: var 0x35 x0.8 in weather 4 (sandstorm).
pub fn h_021d8a64<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me) && b.host.weather() == 4 {
        b.mul_var(0x35, 0xCCD);
    }
}

/// 0x021D8AA8: Snow Cloak: var 0x35 x0.8 in weather 3 (hail).
pub fn h_021d8aa8<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me) && b.host.weather() == 3 {
        b.mul_var(0x35, 0xCCD);
    }
}

/// Unlabeled helper at 0x021D8AEC: no weather damage (var 0x41) when var 0x39 is `weather`
/// and var 0x32 is positive.
fn weather_immunity<H: FlowHost>(b: &mut Battle<H>, me: u8, weather: u32) {
    if is_me(b, 2, me) && b.var(0x32) > 0 && b.var(0x39) as u32 == weather as u8 as u32 {
        b.rewrite_var(0x41, 1);
    }
}

/// 0x021D8A8C: immunity to weather 4 (sandstorm) damage.
pub fn h_021d8a8c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    weather_immunity(b, me, 4);
}

/// 0x021D8AD0: immunity to weather 3 (hail) damage.
pub fn h_021d8ad0<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    weather_immunity(b, me, 3);
}

/// 0x021D8B1C: Tinted Lens (110): resisted hits (class 3) do double.
pub fn h_021d8b1c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && effectiveness_class(b.var(0x38)) == 3 {
        b.mul_var(0x35, 0x2000);
    }
}

/// 0x021D8B50: super-effective hits (class 2) on the defender x0.75 (Solid Rock / Filter).
pub fn h_021d8b50<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me) && effectiveness_class(b.var(0x38)) == 2 {
        b.mul_var(0x35, 0xC00);
    }
}

/// 0x021D8B84: var 0x35 x1.5 for the attacker when var 0x45 is set.
pub fn h_021d8b84<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.var(0x45) != 0 {
        b.mul_var(0x35, 0x1800);
    }
}

/// 0x021D8BB4: Speed +1 when flag-B bit 0 is set (Speed Boost).
pub fn h_021d8bb4<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) || !b.mon(me as u32).flag_b(0) {
        return;
    }
    let id = b.host.handex_push(hx::RANK_EFFECT, me as u32);
    let buf = b.host.handex(id);
    let h = buf.get_u32(0);
    buf.put_u32(0, h | HANDEX_HEADER_POPUP);
    buf.put_u32(4, 5);
    buf.put_u8(0xF, 1);
    buf.put_u8(0x10, me);
    buf.put_u8(0xC, 1);
    b.host.handex_pop(id);
}

/// 0x021D8C0C: rewrites var 0x35 to 2.0 when var 0x44 is set.
pub fn h_021d8c0c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && b.var(0x44) != 0 {
        b.rewrite_var(0x35, 0x2000);
    }
}

/// Unlabeled helper at 0x021D8CAC: at 1/3 HP or less, moves of `ty` get var 0x35 x1.5
/// (Overgrow, Blaze, Torrent, Swarm).
fn pinch_boost<H: FlowHost>(b: &mut Battle<H>, me: u8, ty: u32) {
    if !is_me(b, 3, me) {
        return;
    }
    // FUN_overlay_d_93__021d7b60: max HP / 3.
    let third = u32_div(b.mon_get(me as u32, value::MAX_HP), 3);
    let hp = b.mon_get(me as u32, value::HP);
    if hp <= third && ty == b.var(0x16) as u32 {
        b.mul_var(0x35, 0x1800);
    }
}

/// 0x021D8C3C: pinch boost for type 9 (Fire).
pub fn h_021d8c3c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    pinch_boost(b, me, 9);
}
/// 0x021D8C58: pinch boost for type 10 (Water).
pub fn h_021d8c58<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    pinch_boost(b, me, 10);
}
/// 0x021D8C74: pinch boost for type 11 (Grass).
pub fn h_021d8c74<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    pinch_boost(b, me, 11);
}
/// 0x021D8C90: pinch boost for type 6 (Bug).
pub fn h_021d8c90<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    pinch_boost(b, me, 6);
}

/// 0x021D8CF4: Guts: physical moves x1.5 with a major status.
pub fn h_021d8cf4<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.mon(me as u32).major_status() != 0 && is_physical(b) {
        b.mul_var(0x35, 0x1800);
    }
}

/// Unlabeled helper at 0x021D8D9C: whether an ally of `me` has `ability`. The ally IDs are
/// expanded into the factor's work area, as in the original.
fn ally_has_ability<H: FlowHost>(
    sys: &mut EventSystem,
    f: FactorId,
    b: &mut Battle<H>,
    me: u8,
    ability: u32,
) -> bool {
    let pos = b.host.poke_pos_in_battle(me as u32);
    if pos == 6 {
        return false;
    }
    let mut bytes = [0u8; 28];
    for (i, w) in work(sys, f).iter().enumerate() {
        bytes[4 * i..4 * i + 4].copy_from_slice(&w.to_le_bytes());
    }
    let n = b
        .host
        .expand_poke_ids(((7 << 8) | pos) & 0xFFFF, &mut bytes);
    for (i, w) in work(sys, f).iter_mut().enumerate() {
        *w = u32::from_le_bytes(bytes[4 * i..4 * i + 4].try_into().unwrap());
    }
    let mut i = 0u8;
    while (i as u32) < n {
        let id = bytes[i as usize];
        if me != id && ability == b.mon_get(id as u32, value::ABILITY) {
            return true;
        }
        i = i.wrapping_add(1);
    }
    false
}

/// 0x021D8D40: Plus / Minus: special moves x1.5 when an ally has Plus (57) or Minus (58).
pub fn h_021d8d40<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    if !ally_has_ability(sys, f, b, me, 57) && !ally_has_ability(sys, f, b, me, 58) {
        return;
    }
    let mv = b.var(0x12) as u16 as u32;
    if b.host.move_category(mv) == 2 {
        b.mul_var(0x35, 0x1800);
    }
}

/// Original addresses of the ported handlers (also the differential test order).
pub const PORTED: &[u32] = &[
    0x021D85F4, 0x021D8684, 0x021D86A8, 0x021D86FC, 0x021D8734, 0x021D8760, 0x021D8794, 0x021D87C8,
    0x021D8808, 0x021D8844, 0x021D8874, 0x021D88E8, 0x021D890C, 0x021D8940, 0x021D8964, 0x021D89B8,
    0x021D8A3C, 0x021D8A64, 0x021D8A8C, 0x021D8AA8, 0x021D8AD0, 0x021D8B1C, 0x021D8B50, 0x021D8B84,
    0x021D8BB4, 0x021D8C0C, 0x021D8C3C, 0x021D8C58, 0x021D8C74, 0x021D8C90, 0x021D8CF4, 0x021D8D40,
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
        0x021D88E8 => h_021d88e8::<H>,
        0x021D890C => h_021d890c::<H>,
        0x021D8940 => h_021d8940::<H>,
        0x021D8964 => h_021d8964::<H>,
        0x021D89B8 => h_021d89b8::<H>,
        0x021D8A3C => h_021d8a3c::<H>,
        0x021D8A64 => h_021d8a64::<H>,
        0x021D8A8C => h_021d8a8c::<H>,
        0x021D8AA8 => h_021d8aa8::<H>,
        0x021D8AD0 => h_021d8ad0::<H>,
        0x021D8B1C => h_021d8b1c::<H>,
        0x021D8B50 => h_021d8b50::<H>,
        0x021D8B84 => h_021d8b84::<H>,
        0x021D8BB4 => h_021d8bb4::<H>,
        0x021D8C0C => h_021d8c0c::<H>,
        0x021D8C3C => h_021d8c3c::<H>,
        0x021D8C58 => h_021d8c58::<H>,
        0x021D8C74 => h_021d8c74::<H>,
        0x021D8C90 => h_021d8c90::<H>,
        0x021D8CF4 => h_021d8cf4::<H>,
        0x021D8D40 => h_021d8d40::<H>,
        _ => return None,
    })
}

#[cfg(test)]
mod tests;
