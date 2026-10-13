//! Ability handlers (`hand_tok`): registry at battle_calc rodata 0x021F0E74.
//!
//! Each handler is C `void handler(factor, flow_work, poke_id, work)`; the doc comment gives
//! the original address, the abilities using it and the event. Event-variable labels are kept
//! numeric (see `event_var`).

use crate::battle_mon::value;
use crate::calc::{effectiveness_class, fx_mul, max_i32, u32_div};
use crate::event_factor::{EventSystem, FactorId};
use crate::flow::{
    str_param_add_arg, str_param_set_extra, str_param_setup, Battle, FlowHost, HandexBytes,
};
use crate::main_util::same_side;

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

/// Species 421 (Cherrim).
const SPECIES_CHERRIM: u16 = 0x1A5;

/// HANDEX work type 0x39: form change (+4 Pokemon, +5 form, +8 string parameter).
const HANDEX_FORM_CHANGE: u32 = 0x39;

/// C: `FUN_overlay_d_93__021d8e00`: the Pokemon is Cherrim.
fn is_cherrim<H: FlowHost>(b: &mut Battle<H>, me: u8) -> bool {
    b.mon(me as u32).species() == SPECIES_CHERRIM
}

/// C: `FUN_overlay_d_93__021d8e7c`: changes to `form` (if different) with message 0xDE;
/// `popup` goes to header bit 23.
fn change_form<H: FlowHost>(b: &mut Battle<H>, me: u8, form: u8, popup: u8) {
    if form as u32 == b.mon_get(me as u32, value::FORM) {
        return;
    }
    let id = b.host.handex_push(HANDEX_FORM_CHANGE, me as u32);
    let buf = b.host.handex(id);
    buf.put_u8(4, me);
    buf.put_u8(5, form);
    let h = buf.get_u32(0);
    buf.put_u32(0, (h & 0xFF7F_FFFF) | (((popup as u32) << 31) >> 8));
    str_param_setup(buf, 8, 2, 0xDE);
    str_param_add_arg(buf, 8, me as u32);
    b.host.handex_pop(id);
}

/// 0x021D8E1C: Flower Gift: takes the sunny form in sun (form 1), else form 0; marks the
/// factor active (work word 0).
pub fn h_021d8e1c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_cherrim(b, me) {
        return;
    }
    let sunny = (b.host.weather() == 1) as u8;
    change_form(b, me, sunny, 1);
    work(sys, f)[0] = 1;
}

/// 0x021D8E58: Flower Gift on entry (var 2).
pub fn h_021d8e58<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        h_021d8e1c(sys, f, b, me);
    }
}

/// 0x021D8EDC: Flower Gift weather change.
pub fn h_021d8edc<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if work(sys, f)[0] != 0 && is_cherrim(b, me) {
        let sunny = (b.host.weather() == 1) as u8;
        change_form(b, me, sunny, 1);
    }
}

/// 0x021D8F1C: Flower Gift: back to form 0 when var 2 is this Pokemon.
pub fn h_021d8f1c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if work(sys, f)[0] != 0 && is_me(b, 2, me) && is_cherrim(b, me) {
        change_form(b, me, 0, 0);
    }
}

/// 0x021D8F54: Flower Gift: back to form 0.
pub fn h_021d8f54<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if work(sys, f)[0] != 0 && is_cherrim(b, me) {
        change_form(b, me, 0, 0);
    }
}

/// 0x021D8F84: Flower Gift: back to form 0 when var 0x10 is no longer this factor's ability.
pub fn h_021d8f84<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if work(sys, f)[0] == 0 || !is_cherrim(b, me) || !is_me(b, 2, me) {
        return;
    }
    if b.var(0x10) as u32 != sys.sub_id(f) as u32 {
        change_form(b, me, 0, 0);
    }
}

/// 0x021D8FCC: Flower Gift: in sun, physical moves of allies x1.5.
pub fn h_021d8fcc<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_cherrim(b, me) || b.host.weather() != 1 {
        return;
    }
    let attacker = b.var(3) as u8 as u32;
    if same_side(me as u32, attacker) && is_physical(b) {
        b.mul_var(0x35, 0x1800);
    }
}

/// 0x021D901C: Flower Gift: in sun, allies take special hits (var 0x1A == 2) x1.5 better.
pub fn h_021d901c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_cherrim(b, me) || b.host.weather() != 1 {
        return;
    }
    let defender = b.var(4) as u8 as u32;
    if same_side(me as u32, defender) && b.var(0x1A) == 2 {
        b.mul_var(0x35, 0x1800);
    }
}

/// 0x021D9070: Rivalry: same gender x1.25, opposite x0.75 (genderless, 2, unaffected).
pub fn h_021d9070<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let target = b.var(4) as u8 as u32;
    let mine = b.mon_get(me as u32, value::FA_BYTE) as u8;
    let theirs = b.mon_get(target, value::FA_BYTE) as u8;
    if mine == 2 || theirs == 2 {
        return;
    }
    b.mul_var(0x31, if mine == theirs { 0x1400 } else { 0xC00 });
}

/// 0x021D90E8: Technician: power 60 or less x1.5.
pub fn h_021d90e8<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.var(0x30) as u32 <= 60 {
        b.mul_var(0x31, 0x1800);
    }
}

/// 0x021D9118: Iron Fist: moves with flag 7 (punching) x1.2.
pub fn h_021d9118<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let mv = b.var(0x12) as u16 as u32;
    if b.host.move_flag(mv, 7) {
        b.mul_var(0x31, 0x1333);
    }
}

/// 0x021D9154: Reckless: recoil moves (param 30) and Jump Kick / Hi Jump Kick x1.2.
pub fn h_021d9154<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let mv = b.var(0x12) as u16 as u32;
    if b.host.move_param(mv, 30) != 0 || mv == 26 || mv == 136 {
        b.mul_var(0x31, 0x1333);
    }
}

/// 0x021D919C: Marvel Scale: with a major status, physical hits (var 0x1A == 1) x1.5 better.
pub fn h_021d919c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 4, me) || b.mon(me as u32).major_status() == 0 {
        return;
    }
    let _ = b.var(0x12);
    if b.var(0x1A) == 1 {
        b.mul_var(0x35, 0x1800);
    }
}

/// 0x021D91E4: sets var 0x51 for the attacker.
pub fn h_021d91e4<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        b.rewrite_var(0x51, 1);
    }
}

/// Unlabeled helper at 0x021D928C: blocks condition `sick` (or any for 8) inflicted by
/// another Pokemon's stat-lowering effect (var 0x20 < 0); records the result in work word 0.
fn block_sick<H: FlowHost>(
    sys: &mut EventSystem,
    f: FactorId,
    b: &mut Battle<H>,
    me: u8,
    sick: u32,
) {
    if !is_me(b, 2, me) || is_me(b, 3, me) {
        return;
    }
    if sick != 8 && sick != b.var(0x1F) as u32 {
        return;
    }
    if b.var(0x20) < 0 {
        work(sys, f)[0] = b.rewrite_var(0x41, 1) as u32;
    }
}

/// Unlabeled helper at 0x021D92CC: after a block (work word 0), shows message `str_id` with
/// the ability pop-up unless var 0x19 repeats the last one (work word 1).
fn block_message<H: FlowHost>(
    sys: &mut EventSystem,
    f: FactorId,
    b: &mut Battle<H>,
    me: u8,
    str_id: u32,
) {
    if !is_me(b, 2, me) || work(sys, f)[0] == 0 {
        return;
    }
    let v = b.var(0x19) as u32;
    if v == 0 || work(sys, f)[1] != v {
        b.host.handex_push_pop(hx::ABILITY_POPUP_IN, me as u32);
        let id = b.host.handex_push(hx::MESSAGE, me as u32);
        let buf = b.host.handex(id);
        str_param_setup(buf, 4, 2, str_id);
        str_param_add_arg(buf, 4, me as u32);
        b.host.handex_pop(id);
        b.host.handex_push_pop(hx::ABILITY_POPUP_OUT, me as u32);
        work(sys, f)[1] = v;
    }
    work(sys, f)[0] = 0;
}

/// 0x021D9214: blocks condition 1.
pub fn h_021d9214<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    block_sick(sys, f, b, me, 1);
}
/// 0x021D9224: message 201 after a block.
pub fn h_021d9224<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    block_message(sys, f, b, me, 201);
}
/// 0x021D9240: blocks condition 6.
pub fn h_021d9240<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    block_sick(sys, f, b, me, 6);
}
/// 0x021D9250: message 207 after a block.
pub fn h_021d9250<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    block_message(sys, f, b, me, 207);
}
/// 0x021D926C: blocks every stat-lowering condition (8).
pub fn h_021d926c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    block_sick(sys, f, b, me, 8);
}
/// 0x021D927C: message 198 after a block.
pub fn h_021d927c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    block_message(sys, f, b, me, 198);
}

/// 0x021D9340: Simple: doubles stage changes (var 0x20).
pub fn h_021d9340<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let v = b.var(0x20);
        b.rewrite_var(0x20, v.wrapping_shl(1));
    }
}

/// 0x021D936C: Leaf Guard: in sun, blocks major statuses and condition 14 (yawn).
pub fn h_021d936c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 4, me) || b.host.weather() != 1 {
        return;
    }
    let sick = b.var(0x1D);
    if sick < 6 || sick == 14 {
        work(sys, f)[0] = b.rewrite_var(0x41, 1) as u32;
    }
}

/// 0x021D93A8: in sun, sets var 0x41 for the Pokemon in var 2.
pub fn h_021d93a8<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && b.host.weather() == 1 {
        b.rewrite_var(0x41, 1);
    }
}

/// Unlabeled helper at 0x021D9648: blocks condition `sick` (var 0x1D) on this Pokemon.
fn immune_to<H: FlowHost>(b: &mut Battle<H>, me: u8, sick: u32) -> bool {
    if is_me(b, 4, me) && sick == b.var(0x1D) as u32 {
        return b.rewrite_var(0x41, 1);
    }
    false
}

/// C: `FUN_overlay_d_93__021d9670`: after a block (work word 0 == 1), shows message `str_id`
/// with the ability pop-up.
fn immune_message<H: FlowHost>(
    sys: &mut EventSystem,
    f: FactorId,
    b: &mut Battle<H>,
    me: u8,
    str_id: u16,
) {
    if !is_me(b, 4, me) || work(sys, f)[0] != 1 {
        return;
    }
    b.host.handex_push_pop(hx::ABILITY_POPUP_IN, me as u32);
    let id = b.host.handex_push(hx::MESSAGE, me as u32);
    let buf = b.host.handex(id);
    str_param_setup(buf, 4, 2, str_id as u32);
    str_param_add_arg(buf, 4, me as u32);
    b.host.handex_pop(id);
    b.host.handex_push_pop(hx::ABILITY_POPUP_OUT, me as u32);
    work(sys, f)[0] = 0;
}

/// C: `FUN_overlay_d_93__021d96cc`: [`immune_message`] with message 210.
pub fn h_021d96cc<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    immune_message(sys, f, b, me, 210);
}

/// HANDEX work type 0xB: cure a condition (+4 condition, +8 Pokemon, +0x14 flag).
const HANDEX_CURE_SICK: u32 = 0xB;

/// C: `FUN_overlay_d_93__021d96fc`: cures `sick` if present, with the ability pop-up.
fn cure_own<H: FlowHost>(b: &mut Battle<H>, me: u8, sick: u32) {
    if !b.mon(me as u32).has_sick(sick) {
        return;
    }
    b.host.handex_push_pop(hx::ABILITY_POPUP_IN, me as u32);
    let id = b.host.handex_push(HANDEX_CURE_SICK, me as u32);
    let buf = b.host.handex(id);
    buf.put_u32(4, sick);
    buf.put_u8(0x14, 1);
    buf.put_u8(8, me);
    b.host.handex_pop(id);
    b.host.handex_push_pop(hx::ABILITY_POPUP_OUT, me as u32);
}

/// C: `FUN_overlay_d_93__021d96dc`: [`cure_own`] for the Pokemon in var 2.
fn cure_own_on_entry<H: FlowHost>(b: &mut Battle<H>, me: u8, sick: u32) {
    if is_me(b, 2, me) {
        cure_own(b, me, sick);
    }
}

/// 0x021D93D8: blocks condition 1 (paralysis): Limber.
pub fn h_021d93d8<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    work(sys, f)[0] = immune_to(b, me, 1) as u32;
}
/// 0x021D93EC: cures condition 1 on entry.
pub fn h_021d93ec<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    cure_own_on_entry(b, me, 1);
}
/// 0x021D93FC: cures condition 1.
pub fn h_021d93fc<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    cure_own(b, me, 1);
}
/// 0x021D9418: blocks sleep (2) and yawn (14): Insomnia / Vital Spirit.
pub fn h_021d9418<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    let r = immune_to(b, me, 2) as u32;
    work(sys, f)[0] = r;
    if r == 0 {
        work(sys, f)[0] = immune_to(b, me, 14) as u32;
    }
}
/// 0x021D9440: cures sleep on entry.
pub fn h_021d9440<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    cure_own_on_entry(b, me, 2);
}
/// 0x021D9450: cures sleep.
pub fn h_021d9450<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    cure_own(b, me, 2);
}
/// 0x021D9460: sets var 0x41 for the Pokemon in var 2.
pub fn h_021d9460<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        b.rewrite_var(0x41, 1);
    }
}
/// 0x021D9484: blocks freeze (3): Magma Armor.
pub fn h_021d9484<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    work(sys, f)[0] = immune_to(b, me, 3) as u32;
}
/// 0x021D9498: cures freeze on entry.
pub fn h_021d9498<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    cure_own_on_entry(b, me, 3);
}

/// 0x021D94C4: blocks condition 5 (poison): Immunity.
pub fn h_021d94c4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    work(sys, f)[0] = immune_to(b, me, 5) as u32;
}
/// 0x021D94D8: cures condition 5 on entry.
pub fn h_021d94d8<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    cure_own_on_entry(b, me, 5);
}
/// 0x021D94E8: cures condition 5.
pub fn h_021d94e8<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    cure_own(b, me, 5);
}
/// 0x021D9504: blocks condition 4 (burn): Water Veil.
pub fn h_021d9504<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    work(sys, f)[0] = immune_to(b, me, 4) as u32;
}
/// 0x021D9518: cures condition 4 on entry.
pub fn h_021d9518<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    cure_own_on_entry(b, me, 4);
}
/// 0x021D9528: cures condition 4.
pub fn h_021d9528<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    cure_own(b, me, 4);
}
/// 0x021D9544: blocks condition 6 (confusion): Own Tempo.
pub fn h_021d9544<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    work(sys, f)[0] = immune_to(b, me, 6) as u32;
}
/// C: `FUN_overlay_d_93__021d9558`: [`immune_message`] with message 0x165.
pub fn h_021d9558<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    immune_message(sys, f, b, me, 0x165);
}
/// 0x021D956C: cures condition 6 on entry.
pub fn h_021d956c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    cure_own_on_entry(b, me, 6);
}
/// 0x021D957C: cures condition 6.
pub fn h_021d957c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    cure_own(b, me, 6);
}
/// 0x021D9598: blocks condition 7 (attraction): Oblivious.
pub fn h_021d9598<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    work(sys, f)[0] = immune_to(b, me, 7) as u32;
}
/// 0x021D95AC: cures condition 7 on entry.
pub fn h_021d95ac<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    cure_own_on_entry(b, me, 7);
}
/// 0x021D95BC: cures condition 7.
pub fn h_021d95bc<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    cure_own(b, me, 7);
}

/// 0x021D95CC: Oblivious vs move 445 (Captivate): sets var 0x40 and shows message 210.
pub fn h_021d95cc<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 4, me) || b.var(0x12) as u16 != 0x1BD || !b.rewrite_var(0x40, 1) {
        return;
    }
    b.host.handex_push_pop(hx::ABILITY_POPUP_IN, me as u32);
    let id = b.host.handex_push(hx::MESSAGE, me as u32);
    let buf = b.host.handex(id);
    str_param_setup(buf, 4, 2, 210);
    str_param_add_arg(buf, 4, me as u32);
    b.host.handex_pop(id);
    b.host.handex_push_pop(hx::ABILITY_POPUP_OUT, me as u32);
}

/// HANDEX work type 0x1D: weather change (+4 weather, +5 turns, +6 flag, +8 string).
const HANDEX_WEATHER: u32 = 0x1D;

/// Unlabeled helper at 0x021D97B4: sets permanent weather (turns 0xFF) when this Pokemon
/// enters (var 2).
fn set_weather_on_entry<H: FlowHost>(b: &mut Battle<H>, me: u8, weather: u8) {
    if !is_me(b, 2, me) {
        return;
    }
    let id = b.host.handex_push(HANDEX_WEATHER, me as u32);
    let buf = b.host.handex(id);
    let h = buf.get_u32(0);
    buf.put_u32(0, h | HANDEX_HEADER_POPUP);
    buf.put_u8(4, weather);
    buf.put_u8(5, 0xFF);
    b.host.handex_pop(id);
}

/// 0x021D9744: Drizzle: sets weather 2 on entry.
pub fn h_021d9744<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    set_weather_on_entry(b, me, 2);
}
/// 0x021D9760: Drought: sets weather 1 on entry.
pub fn h_021d9760<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    set_weather_on_entry(b, me, 1);
}
/// 0x021D977C: Sand Stream: sets weather 4 on entry.
pub fn h_021d977c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    set_weather_on_entry(b, me, 4);
}
/// 0x021D9798: Snow Warning: sets weather 3 on entry.
pub fn h_021d9798<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    set_weather_on_entry(b, me, 3);
}

/// 0x021D97EC: Cloud Nine / Air Lock: clears the weather on entry (message 94).
pub fn h_021d97ec<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) {
        return;
    }
    let id = b.host.handex_push(HANDEX_WEATHER, me as u32);
    let buf = b.host.handex(id);
    let h = buf.get_u32(0);
    buf.put_u32(0, h | HANDEX_HEADER_POPUP);
    buf.put_u8(4, 0);
    buf.put_u8(6, 1);
    str_param_setup(buf, 8, 1, 94);
    b.host.handex_pop(id);
}

/// HANDEX work type 5: restore HP (+4 amount, +6 Pokemon).
const HANDEX_RECOVER_HP: u32 = 5;
/// HANDEX work type 7: damage (+4 amount, +6 Pokemon).
const HANDEX_DAMAGE: u32 = 7;
/// "Every major status" for [`HANDEX_CURE_SICK`].
const SICK_ALL_MAJOR: u32 = 36;

/// 0x021D9830: sets var 0x41.
pub fn h_021d9830<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, _: u8) {
    b.rewrite_var(0x41, 1);
}

/// Unlabeled helper at 0x021D9880: in weather `kind` (var 0x39), restores 1/16 HP instead
/// of taking weather damage (var 0x41).
fn weather_heal<H: FlowHost>(b: &mut Battle<H>, me: u8, kind: u32) {
    if kind != b.var(0x39) as u32 || !is_me(b, 2, me) {
        return;
    }
    let id = b.host.handex_push(HANDEX_RECOVER_HP, me as u32);
    let h = b.host.handex(id).get_u32(0);
    b.host.handex(id).put_u32(0, h | HANDEX_HEADER_POPUP);
    b.host.handex(id).put_u8(6, me);
    let amount = b.max_hp_div_min1(me as u32, 16);
    b.host.handex(id).put_u16(4, amount as u16);
    b.host.handex_pop(id);
    b.rewrite_var(0x41, 1);
}

/// 0x021D9848: Ice Body: heals in hail.
pub fn h_021d9848<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    weather_heal(b, me, 3);
}
/// 0x021D9864: Rain Dish: heals in rain.
pub fn h_021d9864<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    weather_heal(b, me, 2);
}

/// 0x021D98DC: Solar Power: loses 1/8 HP in sun (var 0x39 == 1).
pub fn h_021d98dc<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) || b.var(0x39) != 1 {
        return;
    }
    let amount = b.max_hp_div_min1(me as u32, 8);
    b.host.handex_push_pop(hx::ABILITY_POPUP_IN, me as u32);
    let id = b.host.handex_push(HANDEX_DAMAGE, me as u32);
    b.host.handex(id).put_u8(6, me);
    b.host.handex(id).put_u16(4, amount as u16);
    b.host.handex_pop(id);
    b.host.handex_push_pop(hx::ABILITY_POPUP_OUT, me as u32);
}

/// 0x021D9934: Solar Power: special moves x1.5 in sun.
pub fn h_021d9934<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) || b.host.weather() != 1 {
        return;
    }
    let mv = b.var(0x12) as u16 as u32;
    if b.host.move_category(mv) == 2 {
        b.mul_var(0x35, 0x1800);
    }
}

/// 0x021D9978: sets var 0x41 against other Pokemon's effects except condition 8.
pub fn h_021d9978<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me) && !is_me(b, 3, me) && b.var(0x1D) != 8 {
        b.rewrite_var(0x41, 1);
    }
}

/// 0x021D99A4: sets var 0x41 against other Pokemon's effects.
pub fn h_021d99a4<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me) && !is_me(b, 3, me) {
        b.rewrite_var(0x41, 1);
    }
}

/// 0x021D99C8: sets var 0x41 for the defender.
pub fn h_021d99c8<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me) {
        b.rewrite_var(0x41, 1);
    }
}

/// 0x021D99E0: sets var 0x47 for the defender.
pub fn h_021d99e0<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me) {
        b.rewrite_var(0x47, 1);
    }
}

/// 0x021D99F8: sets var 0x47 when this Pokemon is the event's only target.
pub fn h_021d99f8<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if b.is_event_target(me as u32) && b.var(5) == 1 {
        b.rewrite_var(0x47, 1);
    }
}

/// 0x021D9A24: doubles var 0x26 (truncated to 16 bits) for the attacker.
pub fn h_021d9a24<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let v = ((b.var(0x26) as u32) << 17) >> 16;
        b.rewrite_var(0x26, v as i32);
    }
}

/// 0x021D9A44: sets var 0x45 for the attacker.
pub fn h_021d9a44<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        b.rewrite_var(0x45, 1);
    }
}

/// 0x021D9A68: Hydration: cures major statuses in rain.
pub fn h_021d9a68<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) || b.host.weather() != 2 || b.mon(me as u32).major_status() == 0 {
        return;
    }
    b.host.handex_push_pop(hx::ABILITY_POPUP_IN, me as u32);
    let id = b.host.handex_push(HANDEX_CURE_SICK, me as u32);
    let buf = b.host.handex(id);
    buf.put_u32(4, SICK_ALL_MAJOR);
    buf.put_u8(8, me);
    buf.put_u8(0x14, 1);
    let h = buf.get_u32(0);
    buf.put_u32(0, h | (2 << 24));
    b.host.handex_pop(id);
    b.host.handex_push_pop(hx::ABILITY_POPUP_OUT, me as u32);
}

/// 0x021D9ADC: Shed Skin: 33% chance to cure major statuses.
pub fn h_021d9adc<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) || b.mon(me as u32).major_status() == 0 {
        return;
    }
    if !b.roll_percent_or_forced(33) {
        return;
    }
    let id = b.host.handex_push(HANDEX_CURE_SICK, me as u32);
    let buf = b.host.handex(id);
    let h = buf.get_u32(0);
    buf.put_u32(0, h | HANDEX_HEADER_POPUP | (2 << 24));
    buf.put_u32(4, SICK_ALL_MAJOR);
    buf.put_u8(8, me);
    buf.put_u8(0x14, 1);
    b.host.handex_pop(id);
}

/// 0x021D9B40: Poison Heal: poison (var 0x1D == 5) restores 1/8 HP instead.
pub fn h_021d9b40<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) || b.var(0x1D) != 5 {
        return;
    }
    b.rewrite_var(0x32, 0);
    let id = b.host.handex_push(HANDEX_RECOVER_HP, me as u32);
    let amount = b.max_hp_div_min1(me as u32, 8);
    let buf = b.host.handex(id);
    buf.put_u16(4, amount as u16);
    buf.put_u8(6, me);
    let h = buf.get_u32(0);
    buf.put_u32(0, h | (8 << 20));
    b.host.handex_pop(id);
}

/// HANDEX work type 0xC: inflict a condition (+4 condition, +8 cont, +0xC flag, +0xF target).
const HANDEX_ADD_SICK: u32 = 0xC;
/// HANDEX work type 0x14: change type (+4 type pair, +6 Pokemon).
const HANDEX_CHANGE_TYPE: u32 = 0x14;
/// HANDEX work type 0x1F: change ability (+4 ability, +6 Pokemon, +8 flag, +0xC string).
const HANDEX_CHANGE_ABILITY: u32 = 0x1F;

/// 0x021D9BA4: sets var 0x41 for the defender.
pub fn h_021d9ba4<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me) {
        b.rewrite_var(0x41, 1);
    }
}

/// 0x021D9BC8: Super Luck: critical-hit stage (var 0x2C, u8) +1.
pub fn h_021d9bc8<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let v = (b.var(0x2C) as u32).wrapping_add(1) as u8;
        b.rewrite_var(0x2C, v as i32);
    }
}

/// 0x021D9BF8: Anger Point: a critical hit (var 0x45) maxes Attack (message 0x1E1).
pub fn h_021d9bf8<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 4, me) || b.var(0x46) != 0 || b.var(0x45) == 0 {
        return;
    }
    if b.mon(me as u32).stage_headroom(1) <= 0 {
        return;
    }
    let id = b.host.handex_push(hx::RANK_EFFECT, me as u32);
    let room = b.mon(me as u32).stage_headroom(1);
    let buf = b.host.handex(id);
    buf.put_u32(4, 1);
    buf.put_u8(0xC, room as u8);
    buf.put_u8(0xE, 1);
    buf.put_u8(0xF, 1);
    buf.put_u8(0x10, me);
    buf.put_u8(0xD, 1);
    let h = buf.get_u32(0);
    buf.put_u32(0, h | (1 << 23));
    str_param_setup(buf, 0x18, 2, 0x1E1);
    str_param_add_arg(buf, 0x18, me as u32);
    b.host.handex_pop(id);
}

/// Unlabeled helper at 0x021D9E08: on a contact move (move flag 0) against this Pokemon,
/// `percent`% chance to inflict `sick` with `cont` on the attacker.
fn contact_status<H: FlowHost>(b: &mut Battle<H>, me: u8, sick: u32, cont: u32, percent: u8) {
    if !is_me(b, 4, me) || b.var(0x46) != 0 {
        return;
    }
    let mv = b.var(0x12) as u16 as u32;
    if !b.host.move_flag(mv, 0) || !b.roll_percent_or_forced(percent as u32) {
        return;
    }
    let id = b.host.handex_push(HANDEX_ADD_SICK, me as u32);
    let attacker = b.var(3) as u8;
    let buf = b.host.handex(id);
    let h = buf.get_u32(0);
    buf.put_u32(0, h | HANDEX_HEADER_POPUP_MSG);
    buf.put_u32(4, sick);
    buf.put_u32(8, cont);
    buf.put_u8(0xC, 0);
    buf.put_u8(0xF, attacker);
    b.host.handex_pop(id);
}

/// 0x021D9C8C: Poison Point.
pub fn h_021d9c8c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    let c = b.calc_mut().make_default_sick_cont(5).0;
    contact_status(b, me, 5, c, 30);
}
/// 0x021D9CBC: Static.
pub fn h_021d9cbc<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    let c = b.calc_mut().make_default_sick_cont(1).0;
    contact_status(b, me, 1, c, 30);
}
/// 0x021D9CEC: Flame Body.
pub fn h_021d9cec<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    let c = b.calc_mut().make_default_sick_cont(4).0;
    contact_status(b, me, 4, c, 30);
}

/// 0x021D9D1C: Cute Charm: contact from the opposite gender may attract (condition 7).
pub fn h_021d9d1c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 4, me) || b.var(0x46) != 0 || b.host.fun_021c83d4() {
        return;
    }
    let attacker = b.var(3) as u8 as u32;
    let mine = b.mon_get(me as u32, value::FA_BYTE) as u8;
    let theirs = b.mon_get(attacker, value::FA_BYTE) as u8;
    if mine == 2 || theirs == 2 || mine == theirs {
        return;
    }
    let c = b.make_sick_cont(7, me as u32).0;
    contact_status(b, me, 7, c, 30);
}

/// 0x021D9DB0: Effect Spore: 30% split as poison (rand > 20), paralysis (> 10), else sleep.
pub fn h_021d9db0<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 4, me) || b.var(0x46) != 0 {
        return;
    }
    let r = b.calc_mut().rand(30);
    let sick = if r > 20 {
        5
    } else if r > 10 {
        1
    } else {
        2
    };
    let c = b.calc_mut().make_default_sick_cont(sick).0;
    contact_status(b, me, sick, c, 30);
}

/// 0x021D9E80: Rough Skin: contact costs the attacker 1/8 HP (message 0x1AE).
pub fn h_021d9e80<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 4, me) || b.var(0x46) != 0 {
        return;
    }
    let mv = b.var(0x12) as u16 as u32;
    if !b.host.move_flag(mv, 0) {
        return;
    }
    let attacker = b.var(3) as u8;
    if b.mon_get(attacker as u32, value::HP) == 0 {
        return;
    }
    let id = b.host.handex_push(HANDEX_DAMAGE, me as u32);
    let h = b.host.handex(id).get_u32(0);
    b.host.handex(id).put_u32(0, h | HANDEX_HEADER_POPUP_MSG);
    b.host.handex(id).put_u8(6, attacker);
    let amount = b.max_hp_div_min1(attacker as u32, 8);
    let buf = b.host.handex(id);
    buf.put_u16(4, amount as u16);
    str_param_setup(buf, 0xC, 2, 0x1AE);
    str_param_add_arg(buf, 0xC, attacker as u32);
    b.host.handex_pop(id);
}

/// 0x021D9F1C: Aftermath: fainting to contact costs the attacker 1/4 HP (message 0x192).
pub fn h_021d9f1c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 4, me) || b.mon_get(me as u32, value::HP) != 0 {
        return;
    }
    let mv = b.var(0x12) as u16 as u32;
    if !b.host.move_flag(mv, 0) {
        return;
    }
    let attacker = b.var(3) as u8;
    let id = b.host.handex_push(HANDEX_DAMAGE, me as u32);
    let h = b.host.handex(id).get_u32(0);
    b.host.handex(id).put_u32(0, h | HANDEX_HEADER_POPUP_MSG);
    b.host.handex(id).put_u8(6, attacker);
    let amount = b.max_hp_div_min1(attacker as u32, 4);
    let buf = b.host.handex(id);
    buf.put_u16(4, amount as u16);
    let f7 = buf[7];
    buf.put_u8(7, (f7 & !1) | 1);
    str_param_setup(buf, 0xC, 2, 0x192);
    str_param_add_arg(buf, 0xC, attacker as u32);
    b.host.handex_pop(id);
}

/// 0x021D9FC4: Color Change: becomes the type of the move that hit it.
pub fn h_021d9fc4<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !b.is_event_target(me as u32) || b.host.fun_021c83d4() {
        return;
    }
    if b.mon_get(me as u32, value::HP) == 0 {
        return;
    }
    let ty = b.var(0x16) as u8;
    if ty == 17 || b.mon(me as u32).has_type(ty as u32) {
        return;
    }
    b.host.handex_push_pop(hx::ABILITY_POPUP_IN, me as u32);
    let id = b.host.handex_push(HANDEX_CHANGE_TYPE, me as u32);
    let buf = b.host.handex(id);
    buf.put_u16(4, crate::poke_type::make_mono_pair(ty));
    buf.put_u8(6, me);
    b.host.handex_pop(id);
    b.host.handex_push_pop(hx::ABILITY_POPUP_OUT, me as u32);
}

/// 0x021DA04C: Synchronize: passes poison, paralysis or burn back to its source.
pub fn h_021da04c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 4, me) {
        return;
    }
    let sick = b.var(0x1D) as u32;
    if sick != 5 && sick != 1 && sick != 4 {
        return;
    }
    let src = b.var(3) as u8;
    if src == 31 || src == me {
        return;
    }
    let cont = b.var(0x1E) as u32;
    b.host.handex_push_pop(hx::ABILITY_POPUP_IN, me as u32);
    let id = b.host.handex_push(HANDEX_ADD_SICK, me as u32);
    b.host.handex(id).put_u8(0xF, src);
    b.host.handex(id).put_u32(4, sick);
    let c = if sick == 5 && crate::sick_cont::SickCont(cont).is_badly_poisoned() {
        cont
    } else {
        b.make_sick_cont(sick, me as u32).0
    };
    let buf = b.host.handex(id);
    buf.put_u32(8, c);
    buf.put_u8(0xC, 1);
    b.host.handex_pop(id);
    b.host.handex_push_pop(hx::ABILITY_POPUP_OUT, me as u32);
}

/// 0x021DA0FC: sets var 0x41 for the attacker.
pub fn h_021da0fc<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        b.rewrite_var(0x41, 1);
    }
}

/// 0x021DA120: sets var 0x16 (move type) to 0 (Normal).
pub fn h_021da120<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        b.rewrite_var(0x16, 0);
    }
}

/// 0x021DA150: Trace: copies a random opponent's ability on entry, or later the ability of
/// an adjacent opponent that enters (work word 0 = nothing was traceable).
pub fn h_021da150<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    let entering = b.var(2) as u8;
    let mut ability = 0u32;
    let mut src = 31u32;
    if entering == me {
        let pos = b.host.poke_pos(me as u32);
        let mut ids = [0u8; 8];
        let n = b.host.expand_poke_ids((0x100 | pos) & 0xFFFF, &mut ids);
        let mut cands = [0u8; 8];
        let mut count = 0u8;
        let mut i = 0u8;
        while (i as u32) < n {
            let a = b.mon_get(ids[i as usize] as u32, value::ABILITY_RAW) as u16;
            if !b.host.ability_uncopyable(a as u32) {
                cands[count as usize] = ids[i as usize];
                count = count.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }
        if count == 0 {
            work(sys, f)[0] = 1;
        } else {
            let k = if count == 1 {
                0
            } else {
                b.calc_mut().rand(count as u32)
            };
            let id = cands[k as u8 as usize] as u32;
            ability = b.mon_get(id, value::ABILITY_RAW);
            src = b.mon(id).poke_id() as u32;
        }
    } else if !same_side(entering as u32, me as u32) && work(sys, f)[0] == 1 {
        let other = b.host.poke_pos_in_battle(entering as u32);
        let mine = b.host.poke_pos(me as u32);
        if other != 6 && (b.host.rule() != 2 || b.host.positions_adjacent(mine, other)) {
            let a = b.mon_get(entering as u32, value::ABILITY_RAW);
            if !b.host.ability_uncopyable(a & 0xFFFF) {
                ability = a;
                src = entering as u32;
            }
        }
    }
    if ability == 0 || me == 31 {
        return;
    }
    let id = b.host.handex_push(HANDEX_CHANGE_ABILITY, me as u32);
    let buf = b.host.handex(id);
    buf.put_u8(6, me);
    buf.put_u16(4, ability as u16);
    buf.put_u8(8, 1);
    let h = buf.get_u32(0);
    buf.put_u32(0, h | (1 << 23));
    str_param_setup(buf, 0xC, 2, 0x17D);
    str_param_add_arg(buf, 0xC, src);
    let a = u16::from_le_bytes([buf[4], buf[5]]) as u32;
    str_param_add_arg(buf, 0xC, a);
    b.host.handex_pop(id);
}

/// 0x021DA2F0: Natural Cure: cures major statuses when switching out.
pub fn h_021da2f0<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) {
        return;
    }
    let id = b.host.handex_push(HANDEX_CURE_SICK, me as u32);
    let buf = b.host.handex(id);
    buf.put_u32(4, SICK_ALL_MAJOR);
    buf.put_u8(0x14, 1);
    buf.put_u8(8, me);
    buf.put_u8(0x15, 1);
    b.host.handex_pop(id);
}

fn popup_message<H: FlowHost>(b: &mut Battle<H>, me: u8, str_id: u32, args: &[u32]) {
    b.host.handex_push_pop(hx::ABILITY_POPUP_IN, me as u32);
    let id = b.host.handex_push(hx::MESSAGE, me as u32);
    let buf = b.host.handex(id);
    str_param_setup(buf, 4, 2, str_id);
    for &a in args {
        str_param_add_arg(buf, 4, a);
    }
    b.host.handex_pop(id);
    b.host.handex_push_pop(hx::ABILITY_POPUP_OUT, me as u32);
}

/// 0x021DA32C: Download: raises Sp. Atk if the foes' total Defense is at least their total
/// Sp. Def, else Attack.
pub fn h_021da32c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) {
        return;
    }
    let mut ids = [0u8; 8];
    let n = b.host.opponent_ids(me as u32, &mut ids);
    if n == 0 {
        return;
    }
    let (mut def, mut spd) = (0u16, 0u16);
    let mut i = 0u8;
    while (i as u32) < n {
        let id = ids[i as usize] as u32;
        def = def.wrapping_add(b.mon_get(id, value::DEFENSE) as u16);
        spd = spd.wrapping_add(b.mon_get(id, value::SP_DEFENSE) as u16);
        i = i.wrapping_add(1);
    }
    let stat = if def < spd { 1 } else { 3 };
    let id = b.host.handex_push(hx::RANK_EFFECT, me as u32);
    let buf = b.host.handex(id);
    let h = buf.get_u32(0);
    buf.put_u32(0, h | HANDEX_HEADER_POPUP);
    buf.put_u8(0xF, 1);
    buf.put_u8(0x10, me);
    buf.put_u32(4, stat);
    buf.put_u8(0xC, 1);
    b.host.handex_pop(id);
}

/// 0x021DA3D8: Forewarn: announces the foes' strongest move (one-hit KOs count as 150,
/// Counter / Mirror Coat / Metal Burst 120, other variable power 80, status moves 1),
/// choosing randomly among ties.
pub fn h_021da3d8<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) {
        return;
    }
    let mut ids = [0u8; 8];
    let n = b.host.opponent_ids(me as u32, &mut ids);
    let mut best = 0u32;
    let mut found: Vec<(u8, u16)> = Vec::new();
    let mut i = 0u8;
    while (i as u32) < n {
        let id = ids[i as usize] as u32;
        let moves = b.mon(id).move_count();
        let mut j = 0u8;
        while j < moves {
            let mv = b.mon(id).move_id(j as u32) as u32;
            let power = if b.host.move_category(mv) == 0 {
                1
            } else {
                let p = b.host.move_power(mv) as u8 as u32;
                if p != 1 {
                    p
                } else if b.host.move_effect_class(mv) == 9 {
                    150
                } else if mv == 68 || mv == 243 || mv == 0x170 {
                    120
                } else {
                    80
                }
            };
            if power >= best {
                if power > best {
                    best = power;
                    found.clear();
                }
                let pid = b.mon(id).poke_id();
                found.push((pid, mv as u16));
            }
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    if found.is_empty() {
        return;
    }
    let k = b.calc_mut().rand(found.len() as u32) as u8 as usize;
    let (pid, mv) = found[k];
    popup_message(b, me, 0x1B1, &[pid as u32, mv as u32]);
}

/// C: `FUN_overlay_d_93__021da5bc`: whether `foe` knows a one-hit KO move or a damaging
/// move that is super effective against `me`.
fn has_dangerous_move<H: FlowHost>(b: &mut Battle<H>, me: u32, foe: u32) -> bool {
    let pair = b.mon(me).type_pair();
    let moves = b.mon(foe).move_count();
    let mut i = 0u8;
    while i < moves {
        let mv = b.mon(foe).move_id(i as u32) as u32;
        if b.host.move_effect_class(mv) == 9 {
            return true;
        }
        if b.host.move_fun_0201c314(mv) {
            let ty = b.host.move_type(mv) as u8;
            if crate::calc::type_effectiveness_vs_pair(ty, pair) > 3 {
                return true;
            }
        }
        i = i.wrapping_add(1);
    }
    false
}

/// 0x021DA528: Anticipation: shudders if a foe has a dangerous move (message 0x1B4).
pub fn h_021da528<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) {
        return;
    }
    let mut ids = [0u8; 8];
    let n = b.host.opponent_ids(me as u32, &mut ids);
    let mut i = 0u8;
    while (i as u32) < n {
        if has_dangerous_move(b, me as u32, ids[i as usize] as u32) {
            break;
        }
        i = i.wrapping_add(1);
    }
    if i as u32 == n {
        return;
    }
    popup_message(b, me, 0x1B4, &[me as u32]);
}

/// 0x021DA624: Frisk: reveals a random foe's held item (message 0x1B7).
pub fn h_021da624<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) {
        return;
    }
    let mut ids = [0u8; 8];
    let n = b.host.opponent_ids(me as u32, &mut ids);
    let mut items = [0u16; 8];
    let mut count = 0u8;
    let mut i = 0u8;
    while (i as u32) < n {
        items[count as usize] = b.mon(ids[i as usize] as u32).held_item();
        if items[count as usize] != 0 {
            count = count.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    if count == 0 {
        return;
    }
    let k = b.calc_mut().rand(count as u32) as u8 as usize;
    popup_message(b, me, 0x1B7, &[me as u32, items[k] as u32]);
}

/// 0x021DA6EC: sets var 0x41 and shows message 210 with the ability pop-up.
pub fn h_021da6ec<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me) && b.rewrite_var(0x41, 1) {
        popup_message(b, me, 210, &[me as u32]);
    }
}

/// 0x021DA748: Sturdy: at full HP, var 0x3A = 4 (survive); records the result in work 0.
pub fn h_021da748<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 4, me) {
        return;
    }
    let full = b.mon_get(me as u32, value::HP) == b.mon_get(me as u32, value::MAX_HP);
    work(sys, f)[0] = if full {
        b.rewrite_var(0x3A, 4) as u32
    } else {
        0
    };
}

/// 0x021DA780: Sturdy message (0x202) after it triggered.
pub fn h_021da780<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) || work(sys, f)[0] == 0 {
        return;
    }
    popup_message(b, me, 0x202, &[me as u32]);
    work(sys, f)[0] = 0;
}

/// 0x021DA7DC: Unaware: ignores the foe's stages (vars 0x28 / 0x27 = 6).
pub fn h_021da7dc<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        b.rewrite_var(0x28, 6);
    } else if is_me(b, 4, me) {
        b.rewrite_var(0x27, 6);
    }
}

/// 0x021DA808: sets var 0x51 for the defender.
pub fn h_021da808<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me) {
        b.rewrite_var(0x51, 1);
    }
}

/// 0x021DA820: sets var 0x51 for the attacker.
pub fn h_021da820<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        b.rewrite_var(0x51, 1);
    }
}

/// 0x021DA844: Heatproof: Fire moves x0.5.
pub fn h_021da844<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me) && b.var(0x16) == 9 {
        b.mul_var(0x31, 0x800);
    }
}

/// 0x021DA868: Heatproof: halves burn damage (at least 1).
pub fn h_021da868<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && b.var(0x1D) == 4 {
        let v = b.var(0x32) / 2;
        b.rewrite_var(0x32, max_i32(v, 1));
    }
}

/// 0x021DA9E0: Dry Skin: loses 1/8 HP in sun, restores 1/8 in rain (var 0x39).
pub fn h_021da9e0<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) {
        return;
    }
    let w = b.var(0x39) as u8;
    if w == 1 {
        b.host.handex_push_pop(hx::ABILITY_POPUP_IN, me as u32);
        let id = b.host.handex_push(HANDEX_DAMAGE, me as u32);
        b.host.handex(id).put_u8(6, me);
        let a = b.max_hp_div_min1(me as u32, 8);
        b.host.handex(id).put_u16(4, a as u16);
        b.host.handex_pop(id);
        b.host.handex_push_pop(hx::ABILITY_POPUP_OUT, me as u32);
    } else if w == 2 {
        let id = b.host.handex_push(HANDEX_RECOVER_HP, me as u32);
        let h = b.host.handex(id).get_u32(0);
        b.host.handex(id).put_u32(0, h | HANDEX_HEADER_POPUP);
        b.host.handex(id).put_u8(6, me);
        let a = b.max_hp_div_min1(me as u32, 8);
        b.host.handex(id).put_u16(4, a as u16);
        b.host.handex_pop(id);
    }
}

/// 0x021DAA70: Dry Skin: Fire moves x1.25.
pub fn h_021daa70<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me) && b.var(0x16) == 9 {
        b.mul_var(0x31, 0x1400);
    }
}

/// C: `FUN_overlay_d_93__021da8a8`: absorbs another Pokemon's move of type `ty` (var 0x40).
fn absorbs<H: FlowHost>(b: &mut Battle<H>, me: u8, ty: u32) -> bool {
    if is_me(b, 4, me) && !is_me(b, 3, me) && ty == b.var(0x16) as u32 {
        return b.rewrite_var(0x40, 1);
    }
    false
}

/// C: `FUN_overlay_d_93__021da8dc`: restores 1/`div` HP (message 0x183), or reports no
/// effect at full HP (210); sets var 0x51.
fn absorb_heal<H: FlowHost>(b: &mut Battle<H>, me: u8, div: u32) {
    let full = b.mon_get(me as u32, value::HP) == b.mon_get(me as u32, value::MAX_HP);
    if !full {
        let id = b.host.handex_push(HANDEX_RECOVER_HP, me as u32);
        b.host.handex(id).put_u8(6, me);
        let a = b.max_hp_div_min1(me as u32, div);
        let buf = b.host.handex(id);
        buf.put_u16(4, a as u16);
        let h = buf.get_u32(0);
        buf.put_u32(0, h | HANDEX_HEADER_POPUP);
        str_param_setup(buf, 8, 2, 0x183);
        str_param_add_arg(buf, 8, me as u32);
        b.host.handex_pop(id);
    } else {
        let id = b.host.handex_push(hx::MESSAGE, me as u32);
        let buf = b.host.handex(id);
        str_param_setup(buf, 4, 2, 210);
        str_param_add_arg(buf, 4, me as u32);
        let h = buf.get_u32(0);
        buf.put_u32(0, h | HANDEX_HEADER_POPUP);
        b.host.handex_pop(id);
    }
    b.rewrite_var(0x51, 1);
}

/// 0x021DAA94: Water Absorb (and Dry Skin): Water moves restore 1/4 HP.
pub fn h_021daa94<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if absorbs(b, me, 10) {
        absorb_heal(b, me, 4);
    }
}

/// 0x021DAAC0: same as 0x021DAA94 for another event.
pub fn h_021daac0<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if absorbs(b, me, 10) {
        absorb_heal(b, me, 4);
    }
}

/// Unlabeled helper at 0x021DA96C: raises `stat` by `amount` if possible (with the ability
/// pop-up), else reports no effect (message 210).
fn boost_own_stat<H: FlowHost>(b: &mut Battle<H>, me: u8, stat: u32, amount: u32) {
    if b.mon(me as u32).can_change_stage(stat, amount as i32) {
        let id = b.host.handex_push(hx::RANK_EFFECT, me as u32);
        let buf = b.host.handex(id);
        buf.put_u8(0xF, 1);
        buf.put_u8(0x10, me);
        buf.put_u8(0xE, 1);
        buf.put_u32(4, stat);
        buf.put_u8(0xC, amount as u8);
        let h = buf.get_u32(0);
        buf.put_u32(0, h | (1 << 23));
        b.host.handex_pop(id);
    } else {
        let id = b.host.handex_push(hx::MESSAGE, me as u32);
        let buf = b.host.handex(id);
        str_param_setup(buf, 4, 2, 210);
        str_param_add_arg(buf, 4, me as u32);
        let h = buf.get_u32(0);
        buf.put_u32(0, h | HANDEX_HEADER_POPUP);
        b.host.handex_pop(id);
    }
}

fn message_with_popup_flag<H: FlowHost>(b: &mut Battle<H>, me: u8, str_id: u32, args: &[u32]) {
    let id = b.host.handex_push(hx::MESSAGE, me as u32);
    let buf = b.host.handex(id);
    let h = buf.get_u32(0);
    buf.put_u32(0, h | HANDEX_HEADER_POPUP_MSG);
    str_param_setup(buf, 4, 2, str_id);
    for &a in args {
        str_param_add_arg(buf, 4, a);
    }
    b.host.handex_pop(id);
}

/// 0x021DAAEC: Volt Absorb: Electric moves restore 1/4 HP.
pub fn h_021daaec<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if absorbs(b, me, 12) {
        absorb_heal(b, me, 4);
    }
}

/// 0x021DAB24: Motor Drive: Electric moves raise Speed.
pub fn h_021dab24<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if absorbs(b, me, 12) {
        boost_own_stat(b, me, 5, 1);
    }
}

/// 0x021DAB48: sets var 0x4B when var 0x15 is 7.
pub fn h_021dab48<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.var(0x15) == 7 {
        b.rewrite_var(0x4B, 1);
    }
}

/// 0x021DAB78: Soundproof: blocks sound moves (move flag 8).
pub fn h_021dab78<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 4, me) {
        return;
    }
    let mv = b.var(0x12) as u16 as u32;
    if b.host.move_flag(mv, 8) && b.rewrite_var(0x40, 1) {
        popup_message(b, me, 210, &[me as u32]);
    }
}

/// 0x021DABF4: sets var 0x51 if unset, recording success in work word 0.
pub fn h_021dabf4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && b.var(0x51) == 0 {
        work(sys, f)[0] = b.rewrite_var(0x51, 1) as u32;
    }
}

/// 0x021DAC1C: after 0x021DABF4 triggered, shows message 210.
pub fn h_021dac1c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && work(sys, f)[0] != 0 {
        message_with_popup_flag(b, me, 210, &[me as u32]);
        work(sys, f)[0] = 0;
    }
}

/// 0x021DAC6C: clears work word 0.
pub fn h_021dac6c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        work(sys, f)[0] = 0;
    }
}

/// 0x021DAC90: Wonder Guard: only super-effective damaging moves (except Struggle) hit.
pub fn h_021dac90<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 4, me) || is_me(b, 3, me) {
        return;
    }
    let mv = b.var(0x12) as u16 as u32;
    if !b.host.move_fun_0201c314(mv) || mv == 165 {
        return;
    }
    if b.var(0x38) <= 3 && b.rewrite_var(0x40, 1) {
        popup_message(b, me, 210, &[me as u32]);
    }
}

/// 0x021DAD2C: two-step toggle: on the second call sets var 0x22 to 19 (work 1 = result).
pub fn h_021dad2c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) {
        return;
    }
    if work(sys, f)[0] != 0 {
        work(sys, f)[1] = b.rewrite_var(0x22, 19) as u32;
        work(sys, f)[0] = 0;
    } else {
        work(sys, f)[0] = 1;
    }
}

/// 0x021DAD58: sets work word 0 when flag-A bit 0 is set.
pub fn h_021dad58<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && b.mon(me as u32).flag_a(0) {
        work(sys, f)[0] = 1;
    }
}

/// 0x021DAD84: shows message 0x1BD after 0x021DAD2C succeeded (work word 1).
pub fn h_021dad84<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && work(sys, f)[1] != 0 {
        message_with_popup_flag(b, me, 0x1BD, &[me as u32]);
        work(sys, f)[1] = 0;
    }
}

/// 0x021DADD8: clears work word 0 when var 0x0C is 7.
pub fn h_021dadd8<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && b.var(0x0C) == 7 {
        work(sys, f)[0] = 0;
    }
}

/// 0x021DAE04: Damp: blocks Explosion (153) and Self-Destruct (120) via var 0x22 = 19.
pub fn h_021dae04<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, _: u8) {
    let mv = b.var(0x12) as u16 as u32;
    work(sys, f)[0] = 0;
    if (mv == 153 || mv == 120) && b.var(0x22) == 0 {
        work(sys, f)[0] = b.rewrite_var(0x22, 19) as u32;
        work(sys, f)[1] = mv;
    }
}

/// 0x021DAE38: Damp message (0x389) naming the blocked Pokemon and move.
pub fn h_021dae38<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if work(sys, f)[0] == 0 {
        return;
    }
    let who = b.var(2) as u8 as u32;
    let mv = work(sys, f)[1];
    message_with_popup_flag(b, me, 0x389, &[who, mv]);
    work(sys, f)[0] = 0;
}

/// Skip-check callback of [`h_021dae94`] (address of the original routine).
pub const SKIP_CHECK_DAMP: u32 = 0x021D_AEC5;

/// Unlabeled skip-check callback at 0x021DAEC4: disables ability 106 (Aftermath).
pub fn skip_021daec4<H: FlowHost>(
    _: &EventSystem,
    _: FactorId,
    _: &mut Battle<H>,
    ftype: u32,
    _: u32,
    sub_id: u16,
    _: u8,
) -> bool {
    ftype == 4 && sub_id == 106
}

/// 0x021DAE94: Damp: installs the Aftermath-blocking skip check.
pub fn h_021dae94<H: FlowHost>(sys: &mut EventSystem, f: FactorId, _: &mut Battle<H>, _: u8) {
    sys.set_skip_check(f, SKIP_CHECK_DAMP);
}

/// 0x021DAEA4: removes the skip check.
pub fn h_021daea4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, _: &mut Battle<H>, _: u8) {
    sys.clear_skip_check(f);
}

/// 0x021DAEAC: removes the skip check when this Pokemon leaves (var 2).
pub fn h_021daeac<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        sys.clear_skip_check(f);
    }
}

/// 0x021DAEE4: Flash Fire: Fire moves x1.5 once activated (flag-B bit 13).
pub fn h_021daee4<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.mon(me as u32).flag_b(13) && b.var(0x16) == 9 {
        b.mul_var(0x35, 0x1800);
    }
}

/// HANDEX work type 0x17: set a flag-B bit (+4 bit, +8 Pokemon).
const HANDEX_SET_FLAG_B: u32 = 0x17;
/// HANDEX work type 0x18: clear a flag-B bit (+4 bit, +8 Pokemon).
const HANDEX_CLEAR_FLAG_B: u32 = 0x18;
/// Flag-B bit: Flash Fire is active.
const FLAG_B_FLASH_FIRE: u32 = 13;

/// 0x021DAF1C: Flash Fire: clears the boost when leaving.
pub fn h_021daf1c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) || !b.mon(me as u32).flag_b(FLAG_B_FLASH_FIRE) {
        return;
    }
    let id = b.host.handex_push(HANDEX_CLEAR_FLAG_B, me as u32);
    b.host.handex(id).put_u8(8, me);
    b.host.handex(id).put_u32(4, FLAG_B_FLASH_FIRE);
    b.host.handex_pop(id);
}

fn plain_message<H: FlowHost>(b: &mut Battle<H>, me: u8, str_id: u32) {
    let id = b.host.handex_push(hx::MESSAGE, me as u32);
    let buf = b.host.handex(id);
    str_param_setup(buf, 4, 2, str_id);
    str_param_add_arg(buf, 4, me as u32);
    b.host.handex_pop(id);
}

/// 0x021DAF58: Flash Fire: absorbs Fire moves and activates (message 0x1AB), or reports no
/// effect if already active.
pub fn h_021daf58<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !absorbs(b, me, 9) {
        return;
    }
    b.host.handex_push_pop(hx::ABILITY_POPUP_IN, me as u32);
    if !b.mon(me as u32).flag_b(FLAG_B_FLASH_FIRE) {
        plain_message(b, me, 0x1AB);
        let id = b.host.handex_push(HANDEX_SET_FLAG_B, me as u32);
        b.host.handex(id).put_u8(8, me);
        b.host.handex(id).put_u32(4, FLAG_B_FLASH_FIRE);
        b.host.handex_pop(id);
    } else {
        plain_message(b, me, 210);
    }
    b.host.handex_push_pop(hx::ABILITY_POPUP_OUT, me as u32);
}

/// 0x021DB00C: Bad Dreams: sleeping foes lose 1/8 HP (message 0x1E4).
pub fn h_021db00c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) {
        return;
    }
    let mut shown = false;
    let pos = b.host.poke_pos(me as u32);
    let mut ids = [0u8; 8];
    let n = b.host.expand_poke_ids((0x100 | pos) & 0xFFFF, &mut ids);
    let mut i = 0u8;
    while (i as u32) < n {
        let id = ids[i as usize];
        if b.mon(id as u32).has_sick(2) {
            if !shown {
                shown = true;
                b.host.handex_push_pop(hx::ABILITY_POPUP_IN, me as u32);
            }
            let w = b.host.handex_push(HANDEX_DAMAGE, me as u32);
            b.host.handex(w).put_u8(6, id);
            let a = b.max_hp_div_min1(id as u32, 8);
            let buf = b.host.handex(w);
            buf.put_u16(4, a as u16);
            str_param_setup(buf, 0xC, 2, 0x1E4);
            str_param_add_arg(buf, 0xC, id as u32);
            b.host.handex_pop(w);
        }
        i = i.wrapping_add(1);
    }
    if shown {
        b.host.handex_push_pop(hx::ABILITY_POPUP_OUT, me as u32);
    }
}

/// C: `FUN_overlay_d_93__021e889c`: when var 2 is an ally, sets var 0x51; returns whether it
/// was written. (`021e8878` is the same without the result.)
fn ally_sets_51<H: FlowHost>(b: &mut Battle<H>, me: u8) -> bool {
    let who = b.var(2) as u8 as u32;
    same_side(who, me as u32) && b.rewrite_var(0x51, 1)
}

/// 0x021DB0DC: sets var 0x51 when var 2 is an ally (`FUN_overlay_d_93__021e8878`).
pub fn h_021db0dc<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    ally_sets_51(b, me);
}

/// 0x021DB0E4: like 0x021DB0DC, then shows message 72 (type 1) with extra value 0x56A.
pub fn h_021db0e4<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !ally_sets_51(b, me) {
        return;
    }
    let id = b.host.handex_push(hx::MESSAGE, me as u32);
    let buf = b.host.handex(id);
    let h = buf.get_u32(0);
    buf.put_u32(0, h | HANDEX_HEADER_POPUP_MSG);
    str_param_setup(buf, 4, 1, 72);
    str_param_set_extra(buf, 4, 0x56A);
    b.host.handex_pop(id);
}

/// 0x021DB134: announces Turboblaze (163, 0x1F9), Teravolt (164, 0x1F6) or Mold Breaker
/// (0x1BA) on entry.
pub fn h_021db134<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) {
        return;
    }
    let str_id = match sys.sub_id(f) {
        163 => 0x1F9,
        164 => 0x1F6,
        _ => 0x1BA,
    };
    popup_message(b, me, str_id, &[me as u32]);
}

/// Skip-check callback of [`h_021db1d4`] (address of the original routine).
pub const SKIP_CHECK_MOLD_BREAKER: u32 = 0x021D_B1A9;

/// Unlabeled skip-check callback at 0x021DB1A8: disables breakable abilities.
pub fn skip_021db1a8<H: FlowHost>(
    _: &EventSystem,
    _: FactorId,
    b: &mut Battle<H>,
    ftype: u32,
    _: u32,
    sub_id: u16,
    _: u8,
) -> bool {
    ftype == 4 && b.host.ability_breakable(sub_id as u32)
}

/// 0x021DB1D4: Mold Breaker: while attacking, installs the skip check (work word 0).
pub fn h_021db1d4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && work(sys, f)[0] == 0 {
        sys.set_skip_check(f, SKIP_CHECK_MOLD_BREAKER);
        work(sys, f)[0] = 1;
    }
}

/// 0x021DB200: Mold Breaker: removes the skip check.
pub fn h_021db200<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && work(sys, f)[0] == 1 {
        sys.clear_skip_check(f);
        work(sys, f)[0] = 0;
    }
}

/// 0x021DB224: same as 0x021DB200 for another event.
pub fn h_021db224<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    h_021db200(sys, f, b, me);
}

/// Species 351 (Castform).
const SPECIES_CASTFORM: u16 = 0x15F;

/// Unlabeled helper at 0x021DB318: Castform back to its normal form.
fn forecast_reset<H: FlowHost>(b: &mut Battle<H>, me: u8) {
    if b.mon(me as u32).species() != SPECIES_CASTFORM || b.mon_get(me as u32, value::FORM) == 0 {
        return;
    }
    let id = b.host.handex_push(HANDEX_FORM_CHANGE, me as u32);
    let buf = b.host.handex(id);
    buf.put_u8(4, me);
    buf.put_u8(5, 0);
    str_param_setup(buf, 8, 2, 222);
    str_param_add_arg(buf, 8, me as u32);
    b.host.handex_pop(id);
}

/// Unlabeled helper at 0x021DB374: Castform takes the form of `weather` (sun 1, rain 2,
/// hail 3, else 0).
fn forecast_form<H: FlowHost>(b: &mut Battle<H>, me: u8, weather: u32) {
    if b.mon(me as u32).species() != SPECIES_CASTFORM {
        return;
    }
    let cur = b.mon_get(me as u32, value::FORM) as u8;
    let form = if weather <= 3 { weather as u8 } else { 0 };
    if form == cur {
        return;
    }
    let id = b.host.handex_push(HANDEX_FORM_CHANGE, me as u32);
    let buf = b.host.handex(id);
    let h = buf.get_u32(0);
    buf.put_u32(0, h | HANDEX_HEADER_POPUP);
    buf.put_u8(4, me);
    buf.put_u8(5, form);
    str_param_setup(buf, 8, 2, 222);
    str_param_add_arg(buf, 8, me as u32);
    b.host.handex_pop(id);
}

/// 0x021DB254: Forecast: adopts the weather form and marks the factor active.
pub fn h_021db254<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    let w = b.host.weather();
    forecast_form(b, me, w);
    work(sys, f)[0] = 1;
}

/// 0x021DB274: Forecast on entry.
pub fn h_021db274<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        h_021db254(sys, f, b, me);
    }
}

/// 0x021DB298: Forecast on weather change.
pub fn h_021db298<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if work(sys, f)[0] != 0 {
        let w = b.host.weather();
        forecast_form(b, me, w);
    }
}

/// 0x021DB2B8: Forecast reset while active.
pub fn h_021db2b8<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if work(sys, f)[0] != 0 {
        forecast_reset(b, me);
    }
}

/// 0x021DB2C8: Forecast reset when the ability changes away (var 0x10).
pub fn h_021db2c8<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && b.var(0x10) as u32 != sys.sub_id(f) as u32 {
        forecast_reset(b, me);
    }
}

/// 0x021DB2F8: Forecast reset for the Pokemon in var 2.
pub fn h_021db2f8<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        forecast_reset(b, me);
    }
}

/// C: `FUN_overlay_d_93__021db4e8`: draws another Pokemon's move of `ty` to this Pokemon
/// (var 4 rewritten); true if the target changed.
fn redirect<H: FlowHost>(b: &mut Battle<H>, me: u8, ty: u32) -> bool {
    let attacker = b.var(3) as u8;
    if attacker == me || ty != b.var(0x16) as u32 {
        return false;
    }
    let mv = b.var(0x12) as u16 as u32;
    if b.host.fun_021c8290(attacker as u32, me as u32, mv) || b.host.move_unredirectable(mv) {
        return false;
    }
    let old = b.var(4) as u8;
    b.rewrite_var(4, me as i32) && old != me
}

/// 0x021DB41C: Storm Drain: redirects Water moves (work word 0 = redirected).
pub fn h_021db41c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    work(sys, f)[0] = redirect(b, me, 10) as u32;
}

/// 0x021DB430: Storm Drain: Water moves raise Sp. Atk.
pub fn h_021db430<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if absorbs(b, me, 10) {
        boost_own_stat(b, me, 3, 1);
    }
}

/// 0x021DB460: Lightning Rod: redirects Electric moves (work word 0 = redirected).
pub fn h_021db460<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    work(sys, f)[0] = redirect(b, me, 12) as u32;
}

/// 0x021DB474: after a redirect, shows message 0x1C0 if this Pokemon is a target.
pub fn h_021db474<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if work(sys, f)[0] == 0 {
        return;
    }
    if b.is_event_target(me as u32) {
        message_with_popup_flag(b, me, 0x1C0, &[me as u32]);
    }
    work(sys, f)[0] = 0;
}

/// 0x021DB4C4: Lightning Rod: Electric moves raise Sp. Atk.
pub fn h_021db4c4<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if absorbs(b, me, 12) {
        boost_own_stat(b, me, 3, 1);
    }
}

/// 0x021DB560: sets var 0x41 and shows message 0x1C6.
pub fn h_021db560<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 4, me) || !b.rewrite_var(0x41, 1) {
        return;
    }
    let id = b.host.handex_push(hx::MESSAGE, me as u32);
    let buf = b.host.handex(id);
    let h = buf.get_u32(0);
    buf.put_u32(0, h | (1 << 23));
    str_param_setup(buf, 4, 2, 0x1C6);
    str_param_add_arg(buf, 4, me as u32);
    b.host.handex_pop(id);
}

/// 0x021DB5C0: Liquid Ooze: drained HP (var 0x20) damages the drainer instead (0x1C9).
pub fn h_021db5c0<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 4, me) {
        return;
    }
    let amount = b.var(0x20) as u16;
    b.rewrite_var(0x20, 0);
    if amount == 0 {
        return;
    }
    let id = b.host.handex_push(HANDEX_DAMAGE, me as u32);
    let h = b.host.handex(id).get_u32(0);
    b.host.handex(id).put_u32(0, h | (32 << 18));
    let drainer = b.var(3) as u8;
    let buf = b.host.handex(id);
    buf.put_u8(6, drainer);
    buf.put_u16(4, amount);
    str_param_setup(buf, 0xC, 2, 0x1C9);
    let who = buf[6] as u32;
    str_param_add_arg(buf, 0xC, who);
    b.host.handex_pop(id);
}

/// 0x021DB62C: makes this factor temporary (type 6) when var 2 is this Pokemon.
pub fn h_021db62c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        sys.make_temporary(f);
    }
}

/// Skip-check callback of [`h_021db6a4`] (address of the original routine).
pub const SKIP_CHECK_KLUTZ: u32 = 0x021D_B651;

/// Items Klutz does not disable: battle_calc rodata 0x021F0B78 (13 `u16`s).
fn klutz_exempt(item: u16) -> bool {
    (0..13).any(|i| {
        let o = 0x0B78 - 0x041C + 2 * i;
        u16::from_le_bytes([
            crate::calc::BATTLE_CALC_RODATA[o],
            crate::calc::BATTLE_CALC_RODATA[o + 1],
        ]) == item
    })
}

/// Unlabeled skip-check callback at 0x021DB650: Klutz: disables this Pokemon's held item
/// (factor type 5) except exempt items, unless its ability is suppressed (condition 0x10).
pub fn skip_021db650<H: FlowHost>(
    sys: &EventSystem,
    g: FactorId,
    b: &mut Battle<H>,
    ftype: u32,
    _: u32,
    sub_id: u16,
    owner: u8,
) -> bool {
    if b.mon(owner as u32).has_sick(0x10) {
        return false;
    }
    if ftype != 5 || owner != sys.poke_id(g) {
        return false;
    }
    !klutz_exempt(sub_id)
}

/// 0x021DB6A4: Klutz: installs the skip check once (work word 0).
pub fn h_021db6a4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, _: &mut Battle<H>, _: u8) {
    if work(sys, f)[0] == 0 {
        sys.set_skip_check(f, SKIP_CHECK_KLUTZ);
        work(sys, f)[0] = 1;
    }
}

/// 0x021DB6C0: Klutz on entry.
pub fn h_021db6c0<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        h_021db6a4(sys, f, b, me);
    }
}

/// 0x021DB6E4: removes the skip check when this Pokemon leaves.
pub fn h_021db6e4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        sys.clear_skip_check(f);
    }
}

/// HANDEX work type 0x21 (+4 Pokemon).
const HANDEX_21: u32 = 0x21;

/// 0x021DB6FC: pushes work 0x21 for this Pokemon when it holds an item.
pub fn h_021db6fc<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && b.mon(me as u32).held_item() != 0 {
        let id = b.host.handex_push(HANDEX_21, me as u32);
        b.host.handex(id).put_u8(4, me);
        b.host.handex_pop(id);
    }
}

/// 0x021DB734: blocks move 363 by this Pokemon via var 0x22 = 19 (work word 2).
pub fn h_021db734<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) {
        return;
    }
    let mv = b.var(0x12) as u16;
    work(sys, f)[2] = 0;
    if mv == 0x16B && is_me(b, 2, me) {
        work(sys, f)[2] = b.rewrite_var(0x22, 19) as u32;
    }
}

/// 0x021DB774: message 0x389 for the block of 0x021DB734.
pub fn h_021db774<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) || work(sys, f)[2] == 0 {
        return;
    }
    let _ = b.var(2);
    let id = b.host.handex_push(hx::MESSAGE, me as u32);
    let buf = b.host.handex(id);
    let h = buf.get_u32(0);
    buf.put_u32(0, h | HANDEX_HEADER_POPUP_MSG);
    str_param_setup(buf, 4, 2, 0x389);
    str_param_add_arg(buf, 4, me as u32);
    let mv = b.var(0x12) as u32;
    str_param_add_arg(b.host.handex(id), 4, mv);
    b.host.handex_pop(id);
    work(sys, f)[2] = 0;
}

/// 0x021DB7E8: blocks moves 271 and 415 (var 0x40) with message 210.
pub fn h_021db7e8<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 4, me) {
        return;
    }
    let mv = b.var(0x12) as u16;
    if (mv == 0x10F || mv == 0x19F) && b.rewrite_var(0x40, 1) {
        message_with_popup_flag(b, me, 210, &[me as u32]);
    }
}

/// 0x021DB854: sets var 0x41 when var 0x2D is clear (work word 0 = result).
pub fn h_021db854<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && b.var(0x2D) == 0 {
        work(sys, f)[0] = b.rewrite_var(0x41, 1) as u32;
    }
}

/// 0x021DB87C: message 0x1ED after 0x021DB854 triggered.
pub fn h_021db87c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && work(sys, f)[0] != 0 {
        message_with_popup_flag(b, me, 0x1ED, &[me as u32]);
        work(sys, f)[0] = 0;
    }
}

// ---- handlers 0x021D88B8, 0x021D94A8 and 0x021DB8E8 onward ------------------------------------

/// 0x021D88B8: Stall (100), event 0xF: clears var 0x11 for the Pokemon in var 2.
pub fn h_021d88b8<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        b.rewrite_var(0x11, 0);
    }
}

/// 0x021D94A8: Magma Armor (40), event 2: cures freeze (condition 3).
pub fn h_021d94a8<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    cure_own(b, me, 3);
}

/// 0x021DB8E8: Pressure (46), events 0x55 / 0x8A: announces the ability (message 0x1E7).
pub fn h_021db8e8<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        popup_message(b, me, 0x1E7, &[me as u32]);
    }
}

/// 0x021DB93C: Pressure (46), event 0x4E: one extra PP (var 0x20) when this Pokemon is
/// targeted by a foe's move.
pub fn h_021db93c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if same_side(b.var(3) as u8 as u32, me as u32) {
        return;
    }
    let mut hit = b.is_event_target(me as u32);
    if !hit {
        let mv = b.var(0x12) as u16 as u32;
        hit = b.host.move_effect_class(mv) == 10 || b.host.move_fun_0689c8dc(mv);
    }
    if hit {
        let v = (b.var(0x20) as u32).wrapping_add(1) as u8;
        b.rewrite_var(0x20, v as i32);
    }
}

/// 0x021DB9B8: sets var 0x51 to 0 for the defender (event 0x80).
pub fn h_021db9b8<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me) {
        b.rewrite_var(0x51, 0);
    }
}

/// 0x021DB9DC: Stench (1), event 0x6C: sets var 0x26 to 10 when var 0x25 is 0.
pub fn h_021db9dc<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.var(0x25) as u8 == 0 {
        b.rewrite_var(0x26, 10);
    }
}

/// Foes of `me` (position code 0x100 | position), as the original expands them.
fn foe_ids<H: FlowHost>(b: &mut Battle<H>, me: u8, ids: &mut [u8]) -> u32 {
    let pos = b.host.poke_pos(me as u32);
    b.host.expand_poke_ids((pos | 0x100) & 0xFFFF, ids)
}

/// 0x021DBA0C: Shadow Tag (23), event 0xC: traps foes (var 0x41) unless one has Shadow Tag.
pub fn h_021dba0c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if same_side(b.var(2) as u8 as u32, me as u32) {
        return;
    }
    let mut ids = [0u8; 8];
    let n = foe_ids(b, me, &mut ids);
    let mut i = 0u8;
    while n != 0 && (i as u32) < n {
        if b.mon_get(ids[i as usize] as u32, value::ABILITY) == 23 {
            return;
        }
        i = i.wrapping_add(1);
    }
    b.rewrite_var(0x41, 1);
}

/// 0x021DBA80: Arena Trap (71), event 0xC: traps foes (var 0x41) until one passes
/// `FUN_021c8234`.
pub fn h_021dba80<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if same_side(b.var(2) as u8 as u32, me as u32) {
        return;
    }
    let mut ids = [0u8; 8];
    let n = foe_ids(b, me, &mut ids);
    let mut i = 0u8;
    while n != 0 && (i as u32) < n {
        if !b.host.fun_021c8234(ids[i as usize] as u32) {
            b.rewrite_var(0x41, 1);
            return;
        }
        i = i.wrapping_add(1);
    }
}

/// 0x021DBAF0: Magnet Pull (42), event 0xC: traps Steel-type (8) foes (var 0x41).
pub fn h_021dbaf0<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if same_side(b.var(2) as u8 as u32, me as u32) {
        return;
    }
    let mut ids = [0u8; 8];
    let n = foe_ids(b, me, &mut ids);
    let mut i = 0u8;
    while n != 0 && (i as u32) < n {
        if b.mon(ids[i as usize] as u32).has_type(8) {
            b.rewrite_var(0x41, 1);
        }
        i = i.wrapping_add(1);
    }
}

/// 0x021DBB64: Unburden (84), event 0x9C: remembers (work word 0) that an item is held.
pub fn h_021dbb64<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && b.var(0x2D) == 0 && b.mon(me as u32).held_item() != 0 {
        work(sys, f)[0] = 1;
    }
}

/// 0x021DBB98: Unburden (84), event 0x13: doubles var 0x35 once the held item is gone.
pub fn h_021dbb98<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && work(sys, f)[0] == 1 && b.mon(me as u32).held_item() == 0 {
        b.mul_var(0x35, 0x2000);
    }
}

/// HANDEX work type 0x20: give a held item (+4 item, +6 Pokemon, +7 flag, +0xC string).
const HANDEX_GIVE_ITEM: u32 = 0x20;

/// 0x021DBBD8: Pickup (53), event 0x77: without an item, recovers the used item (+0x14) of a
/// random Pokemon that has one.
pub fn h_021dbbd8<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) || b.mon(me as u32).held_item() != 0 {
        return;
    }
    let pos = b.host.poke_pos(me as u32);
    // The original keeps ids at +0, candidates at +6, their count at +0xC in a scratch block.
    let mut buf = [0u8; 14];
    let n = b.host.expand_poke_ids((0x200 | pos) & 0xFFFF, &mut buf);
    buf[12] = n as u8;
    buf[13] = 0;
    let mut found = 0u8;
    let mut i = 0u8;
    while i < buf[12] {
        let id = buf[i as usize] as u32;
        if b.mon(id).flag_a(8) && b.mon(id).u14() != 0 {
            buf[6 + found as usize] = b.mon(id).poke_id();
            found = found.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    if found == 0 {
        return;
    }
    let pick = b.calc_mut().rand(found as u32) as u8;
    let target = buf[6 + pick as usize] as u32;
    let item = b.mon(target).u14();
    if item == 0 {
        return;
    }
    let id = b.host.handex_push(HANDEX_GIVE_ITEM, me as u32);
    let w = b.host.handex(id);
    let h = w.get_u32(0);
    w.put_u32(0, h | HANDEX_HEADER_POPUP_MSG);
    w.put_u8(6, me);
    w.put_u16(4, item);
    w.put_u8(8, 1);
    w.put_u8(9, target as u8);
    str_param_setup(w, 12, 2, 0x1EA);
    str_param_add_arg(w, 12, me as u32);
    str_param_add_arg(w, 12, item as u32);
    b.host.handex_pop(id);
}

/// Unlabeled routine at 0x021E8638: `species` is locked to its form-changing item.
fn form_item_locked<H: FlowHost>(b: &mut Battle<H>, species: u32, item: u32) -> bool {
    match species {
        0x1E7 => item == 0x70,
        0x1ED => b.host.item_fun_0689c980(item),
        0x289 => b.host.item_fun_0689c990(item),
        _ => false,
    }
}

/// Unlabeled routine at 0x021E869C: [`form_item_locked`] for Pokemon `poke_id`'s own item.
fn form_item_locked_own<H: FlowHost>(b: &mut Battle<H>, poke_id: u32) -> bool {
    let species = b.mon(poke_id).species() as u32;
    let item = b.mon(poke_id).held_item() as u32;
    form_item_locked(b, species, item)
}

/// Unlabeled routine at 0x021E86C8.
fn fun_021e86c8<H: FlowHost>(b: &mut Battle<H>, poke_id: u32) -> bool {
    !b.host.fun_021c8168() && b.host.fun_021b8ddc(poke_id) == 1
}

/// Unlabeled routine at 0x021E86E8: `me` cannot take `attacker`'s item (form-locked items).
fn item_theft_blocked<H: FlowHost>(b: &mut Battle<H>, me: u32, attacker: u32) -> bool {
    if fun_021e86c8(b, me) {
        return true;
    }
    if form_item_locked_own(b, attacker) {
        return true;
    }
    let species = b.mon(me).species() as u32;
    let item = b.mon(attacker).held_item() as u32;
    form_item_locked(b, species, item)
}

/// HANDEX work type 0x24: take the attacker's held item (+4 Pokemon, +8 string).
const HANDEX_STEAL_ITEM: u32 = 0x24;

/// 0x021DBCEC: Magician (124), event 0x87: takes the attacker's item on contact when this
/// Pokemon holds none.
pub fn h_021dbcec<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !b.is_event_target(me as u32) {
        return;
    }
    let att = b.var(3) as u8 as u32;
    if item_theft_blocked(b, me as u32, att) {
        return;
    }
    let mv = b.var(0x12) as u16 as u32;
    if !b.host.move_flag(mv, 0) {
        return;
    }
    if b.mon(me as u32).held_item() != 0 {
        return;
    }
    let item = b.mon(att).held_item();
    if item == 0 {
        return;
    }
    let id = b.host.handex_push(HANDEX_STEAL_ITEM, me as u32);
    let w = b.host.handex(id);
    let h = w.get_u32(0);
    w.put_u32(0, h | HANDEX_HEADER_POPUP);
    w.put_u8(4, att as u8);
    str_param_setup(w, 8, 2, 0x1CC);
    str_param_add_arg(w, 8, att);
    str_param_add_arg(w, 8, item as u32);
    b.host.handex_pop(id);
}

/// 0x021DBD9C: Cursed Body (130), event 0x4B: 30% to disable (condition 13) the attacker's
/// move for 4 turns (5 if it already acted).
pub fn h_021dbd9c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 4, me) || b.var(0x46) != 0 || b.host.fun_021c83d4() {
        return;
    }
    let att = b.var(3) as u8 as u32;
    if b.mon(att).has_sick(13) {
        return;
    }
    let mv = b.var(0x14) as u16 as u32;
    if mv == 0 || mv == 165 || b.host.move_fun_0689c8cc(mv) {
        return;
    }
    if !b.roll_percent_or_forced(30) {
        return;
    }
    let mut turns = 4u8;
    if b.mon(att).flag_a(1) {
        turns = turns.wrapping_add(1);
    }
    let id = b.host.handex_push(HANDEX_ADD_SICK, me as u32);
    let cont = crate::sick_cont::SickCont::turn_param(turns as u32, mv).0;
    let w = b.host.handex(id);
    w.put_u32(4, 13);
    w.put_u32(8, cont);
    w.put_u8(0xF, att as u8);
    let h = w.get_u32(0);
    w.put_u32(0, h | HANDEX_HEADER_POPUP);
    b.host.handex_pop(id);
}

/// 0x021DBE6C: Weak Armor (133), event 0x4B: a physical hit lowers Defense and raises Speed.
pub fn h_021dbe6c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 4, me) || b.var(0x1A) != 1 || b.var(0x46) != 0 {
        return;
    }
    let m = b.mon(me as u32);
    if !(m.can_change_stage(2, -1) || m.can_change_stage(5, 1)) || b.mon_is_fainted(me as u32) {
        return;
    }
    b.host.handex_push_pop(hx::ABILITY_POPUP_IN, me as u32);
    for (stat, delta) in [(2u32, 0xFFu8), (5, 1)] {
        let id = b.host.handex_push(hx::RANK_EFFECT, me as u32);
        let w = b.host.handex(id);
        w.put_u8(0xF, 1);
        w.put_u8(0x10, me);
        w.put_u8(0xE, 1);
        w.put_u32(4, stat);
        w.put_u8(0xC, delta);
        b.host.handex_pop(id);
    }
    b.host.handex_push_pop(hx::ABILITY_POPUP_OUT, me as u32);
}

/// Unlabeled routine at 0x021DBFD4: the move has a secondary effect (Sheer Force).
fn secondary_effect_move<H: FlowHost>(b: &mut Battle<H>, mv: u32) -> bool {
    let class = b.host.move_effect_class(mv);
    if b.host.move_param(mv, 10) != 0 {
        return true;
    }
    match class {
        4 => b.host.move_param(mv, 11) != 8,
        6 => true,
        7 => {
            let n = b.host.move_fun_0201c020(mv);
            let mut i = 0u32;
            while i < n {
                if b.host.move_fun_0201c0bc(mv, i) < 0 {
                    return false;
                }
                i = i.wrapping_add(1);
            }
            true
        }
        _ => mv == 0x122,
    }
}

/// 0x021DBF28: Sheer Force (125), event 0x38: x1.3 power (var 0x31) for such moves.
pub fn h_021dbf28<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let mv = b.var(0x12) as u16 as u32;
        if secondary_effect_move(b, mv) {
            b.mul_var(0x31, 0x14CD);
        }
    }
}

/// 0x021DBF58: Sheer Force (125), events 0x51 / 0x64: sets var 0x41 for such moves.
pub fn h_021dbf58<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let mv = b.var(0x12) as u16 as u32;
        if secondary_effect_move(b, mv) {
            b.rewrite_var(0x41, 1);
        }
    }
}

/// 0x021DBF84: Sheer Force (125), event 0x6C: sets var 0x41.
pub fn h_021dbf84<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        b.rewrite_var(0x41, 1);
    }
}

/// 0x021DBF9C: Sheer Force (125), event 0x82: sets var 0x48 for such moves.
pub fn h_021dbf9c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let mv = b.var(0x12) as u16 as u32;
        if secondary_effect_move(b, mv) {
            b.rewrite_var(0x48, 1);
        }
    }
}

/// 0x021DC054: Defiant (128), event 0x5D: a foe's stat drop (var 0x20 < 0) raises Attack by 2.
pub fn h_021dc054<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) || same_side(me as u32, b.var(3) as u8 as u32) || b.var(0x20) >= 0 {
        return;
    }
    let id = b.host.handex_push(hx::RANK_EFFECT, me as u32);
    let w = b.host.handex(id);
    let h = w.get_u32(0);
    w.put_u32(0, h | HANDEX_HEADER_POPUP_MSG);
    w.put_u32(4, 1);
    w.put_u8(0xC, 2);
    w.put_u8(0xE, 1);
    w.put_u8(0xF, 1);
    w.put_u8(0x10, me);
    b.host.handex_pop(id);
}

/// 0x021DC0BC: Defeatist (129), event 0x3B: at half HP or less, var 0x35 x0.5.
pub fn h_021dc0bc<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let hp = b.mon_get(me as u32, value::HP);
    if hp <= b.max_hp_div(me as u32, 2) {
        b.mul_var(0x35, 0x800);
    }
}

/// 0x021DC104: Multiscale (136), event 0x47: at full HP, var 0x35 x0.5.
pub fn h_021dc104<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me) && b.mon_is_full_hp(me as u32) {
        b.mul_var(0x35, 0x800);
    }
}

/// 0x021DC13C: Friend Guard (132), event 0x47: ally hits (var 4) get var 0x35 x0.75.
pub fn h_021dc13c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    let defender = b.var(4) as u8 as u32;
    if defender != me as u32 && same_side(me as u32, defender) {
        b.mul_var(0x35, 0xC00);
    }
}

/// 0x021DC170: Healer (131), event 0x76: 30% per ally to cure its major status.
pub fn h_021dc170<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) {
        return;
    }
    let pos = b.host.poke_pos_in_battle(me as u32);
    let mut ids = [0u8; 8];
    let n = b.host.expand_poke_ids((0x400 | pos) & 0xFFFF, &mut ids);
    let mut i = 0u8;
    while (i as u32) < n {
        let ally = ids[i as usize];
        if ally != me && b.mon(ally as u32).major_status() != 0 && b.roll_percent_or_forced(30) {
            let id = b.host.handex_push(HANDEX_CURE_SICK, me as u32);
            let w = b.host.handex(id);
            let h = w.get_u32(0);
            w.put_u32(0, h | HANDEX_HEADER_POPUP | (2 << 24));
            w.put_u8(0x14, 1);
            w.put_u8(8, ally);
            w.put_u32(4, SICK_ALL_MAJOR);
            b.host.handex_pop(id);
        }
        i = i.wrapping_add(1);
    }
}

/// 0x021DC218: Toxic Boost (137), event 0x38: poisoned physical attacker, var 0x31 x1.5.
pub fn h_021dc218<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.mon(me as u32).has_sick(5) && b.var(0x1A) == 1 {
        b.mul_var(0x31, 0x1800);
    }
}

/// 0x021DC25C: Flare Boost (138), event 0x38: burned special attacker, var 0x31 x1.5.
pub fn h_021dc25c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.mon(me as u32).has_sick(4) && b.var(0x1A) == 2 {
        b.mul_var(0x31, 0x1800);
    }
}

/// 0x021DC2A0: Telepathy (140), event 0x2D: ignores an ally's damaging move (message 0x1D5).
pub fn h_021dc2a0<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 4, me) {
        return;
    }
    let att = b.var(3) as u8 as u32;
    if !same_side(me as u32, att) || me as u32 == att {
        return;
    }
    let mv = b.var(0x12) as u16 as u32;
    if !b.host.move_fun_0201c314(mv) || !b.rewrite_var(0x40, 1) {
        return;
    }
    message_with_popup_flag(b, me, 0x1D5, &[me as u32]);
}

/// 0x021DC32C: Moody (141), event 0x77: raises a random stat by 2 and lowers another by 1.
pub fn h_021dc32c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) {
        return;
    }
    // Scratch block: stat pairs from +0, raisable stats at +0xA8, lowerable stats at +0xAF.
    let mut buf = [0u8; 182];
    let (mut ups, mut downs) = (0u8, 0u8);
    for s in 0..7u32 {
        if b.mon(me as u32).can_change_stage(s + 1, 1) {
            buf[0xA8 + ups as usize] = (s + 1) as u8;
            ups = ups.wrapping_add(1);
        }
        if b.mon(me as u32).can_change_stage(s + 1, -1) {
            buf[0xAF + downs as usize] = (s + 1) as u8;
            downs = downs.wrapping_add(1);
        }
    }
    let (mut up, mut down) = (0u8, 0u8);
    if ups == 0 && downs != 0 {
        down = buf[0xAF + b.calc_mut().rand(downs as u32) as usize];
    } else if ups != 0 && downs == 0 {
        up = buf[0xA8 + b.calc_mut().rand(ups as u32) as usize];
    } else if ups != 0 && downs != 0 {
        let mut n = 0u8;
        let mut i = 0u8;
        while i < ups {
            let mut j = 0u8;
            while j < downs {
                let d = buf[0xAF + j as usize];
                let u = buf[0xA8 + i as usize];
                if u != d {
                    let at = n as usize * 2;
                    n = n.wrapping_add(1);
                    buf[at..at + 2].copy_from_slice(&(((u as u16) << 8) | d as u16).to_le_bytes());
                    if n >= 84 {
                        break;
                    }
                }
                j = j.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }
        let at = b.calc_mut().rand(n as u32) as usize * 2;
        let pair = u16::from_le_bytes([buf[at], buf[at + 1]]);
        up = (pair >> 8) as u8;
        down = pair as u8;
    }
    b.host.handex_push_pop(hx::ABILITY_POPUP_IN, me as u32);
    for (stat, amount) in [(up, 2u8), (down, 0xFF)] {
        if stat != 0 {
            let id = b.host.handex_push(hx::RANK_EFFECT, me as u32);
            let w = b.host.handex(id);
            w.put_u8(0xF, 1);
            w.put_u8(0x10, me);
            w.put_u32(4, stat as u32);
            w.put_u8(0xC, amount);
            b.host.handex_pop(id);
        }
    }
    b.host.handex_push_pop(hx::ABILITY_POPUP_OUT, me as u32);
}

/// 0x021DC4A4: Overcoat (142), event 0x7F: sets var 0x41 while var 0x32 is positive.
pub fn h_021dc4a4<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && b.var(0x32) > 0 {
        b.rewrite_var(0x41, 1);
    }
}

/// 0x021DC4D4: Poison Touch (143), event 0x4B: 30% to poison the target on contact.
pub fn h_021dc4d4<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) || b.var(0x46) != 0 || b.var(0x47) != 0 {
        return;
    }
    let mv = b.var(0x12) as u16 as u32;
    if !b.host.move_flag(mv, 0) || !b.roll_percent_or_forced(30) {
        return;
    }
    let id = b.host.handex_push(HANDEX_ADD_SICK, me as u32);
    let target = b.var(4) as u8;
    let cont = b.calc_mut().make_default_sick_cont(5).0;
    let w = b.host.handex(id);
    let h = w.get_u32(0);
    w.put_u32(0, h | HANDEX_HEADER_POPUP);
    w.put_u8(0xF, target);
    w.put_u32(4, 5);
    w.put_u32(8, cont);
    str_param_setup(w, 20, 2, 0x1D8);
    str_param_add_arg(w, 20, target as u32);
    b.host.handex_pop(id);
}

/// HANDEX work type 8: restore HP (+4 / +5 flags, +7 Pokemon, +0x10 amount).
const HANDEX_HEAL: u32 = 8;

/// 0x021DC57C: Regenerator (144), event 0x54: restores 1/3 of max HP.
pub fn h_021dc57c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) || b.mon_is_fainted(me as u32) || b.mon_is_full_hp(me as u32) {
        return;
    }
    let mut amount = b.max_hp_div_min1(me as u32, 3);
    let max_hp = b.mon_get(me as u32, value::MAX_HP);
    let missing = max_hp.wrapping_sub(b.mon_get(me as u32, value::HP));
    if amount > missing {
        amount = missing;
    }
    let id = b.host.handex_push(HANDEX_HEAL, me as u32);
    let w = b.host.handex(id);
    w.put_u8(7, me);
    w.put_u32(16, amount);
    w.put_u8(4, 1);
    w.put_u8(5, 1);
    b.host.handex_pop(id);
}

/// 0x021DC604: blocks condition 2 (see [`block_sick`]).
pub fn h_021dc604<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    block_sick(sys, f, b, me, 2);
}
/// 0x021DC614: message 204 after a block.
pub fn h_021dc614<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    block_message(sys, f, b, me, 0xCC);
}

/// 0x021DC630: Sand Rush (146), event 0x13: doubles var 0x35 in a sandstorm (weather 4).
pub fn h_021dc630<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && b.host.weather() == 4 {
        b.mul_var(0x35, 0x2000);
    }
}

/// 0x021DC664: Wonder Skin (147), event 0x34: caps the accuracy (var 0x2B) of status moves
/// at 50.
pub fn h_021dc664<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me) && b.var(0x1A) == 0 && b.var(0x2B) as u32 > 50 {
        b.rewrite_var(0x2B, 50);
    }
}

/// 0x021DC6A0: Analytic (148), event 0x38: x1.3 power (var 0x31) unless `FUN_0689C8CC`.
pub fn h_021dc6a0<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let mv = b.var(0x12) as u16 as u32;
    if !b.host.move_fun_0689c8cc(mv) && b.host.fun_021c80ac(me as u32) {
        b.mul_var(0x31, 0x14CD);
    }
}

/// 0x021DC6F8: Sand Force (159), event 0x38: x1.3 power for Ground (4), Rock (5) and Steel
/// (8) moves in a sandstorm.
pub fn h_021dc6f8<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) || b.host.weather() != 4 {
        return;
    }
    let ty = b.var(0x16) as u8;
    if ty == 4 || ty == 5 || ty == 8 {
        b.mul_var(0x31, 0x14CD);
    }
}

/// Species 555 (Darmanitan).
const SPECIES_DARMANITAN: u16 = 0x22B;

/// 0x021DC738: Zen Mode (161), event 0x78: changes form at half HP or less.
pub fn h_021dc738<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if b.mon(me as u32).species() != SPECIES_DARMANITAN {
        return;
    }
    let hp = b.mon_get(me as u32, value::HP);
    let low = (hp <= b.max_hp_div(me as u32, 2)) as u32;
    if low == b.mon_get(me as u32, value::FORM) {
        return;
    }
    let id = b.host.handex_push(HANDEX_FORM_CHANGE, me as u32);
    let w = b.host.handex(id);
    let h = w.get_u32(0);
    w.put_u32(0, h | HANDEX_HEADER_POPUP);
    w.put_u8(4, me);
    w.put_u8(5, low as u8);
    str_param_setup(w, 8, 1, if low == 1 { 0xB9 } else { 0xBA });
    b.host.handex_pop(id);
}

/// 0x021DC7BC: Zen Mode (161), events 0x6A / 0x89: Darmanitan returns to its normal form.
pub fn h_021dc7bc<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) || b.mon(me as u32).species() != SPECIES_DARMANITAN {
        return;
    }
    let id = b.host.handex_push(HANDEX_FORM_CHANGE, me as u32);
    let w = b.host.handex(id);
    w.put_u8(4, me);
    w.put_u8(5, 0);
    str_param_setup(w, 8, 1, 0xBA);
    str_param_add_arg(w, 8, me as u32);
    b.host.handex_pop(id);
}

/// Skip-check callback of [`h_021dc844`] (address of the original routine).
pub const SKIP_CHECK_021DC820: u32 = 0x021D_C821;

/// Unlabeled skip-check callback at 0x021DC820: skips type-2 factors of the other side whose
/// ID is at most 3.
pub fn skip_021dc820<H: FlowHost>(
    sys: &EventSystem,
    f: FactorId,
    _: &mut Battle<H>,
    ftype: u32,
    _: u32,
    sub_id: u16,
    owner: u8,
) -> bool {
    ftype == 2 && crate::main_util::poke_side(sys.poke_id(f) as u32) != owner && sub_id <= 3
}

/// 0x021DC844: (151), event 3: installs [`skip_021dc820`].
pub fn h_021dc844<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        sys.set_skip_check(f, SKIP_CHECK_021DC820);
    }
}

/// 0x021DC864: (151), event 4: removes the skip check.
pub fn h_021dc864<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        sys.clear_skip_check(f);
    }
}

/// 0x021DC894: Moxie-style (153), event 0x83: raises Attack by 1 for each fainted target.
pub fn h_021dc894<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let n = b.var(5) as u32;
    let mut i = 0u32;
    while i < n {
        let target = b.var((i + 6) as u16) as u8 as u32;
        if b.mon_is_fainted(target) {
            let id = b.host.handex_push(hx::RANK_EFFECT, me as u32);
            let w = b.host.handex(id);
            let h = w.get_u32(0);
            w.put_u32(0, h | HANDEX_HEADER_POPUP);
            w.put_u8(0xF, 1);
            w.put_u8(0x10, me);
            w.put_u32(4, 1);
            w.put_u8(0xC, 1);
            b.host.handex_pop(id);
        }
        i = i.wrapping_add(1);
    }
}

/// 0x021DC908: (154), event 0x4B: a hit of type 16 raises Attack by 1.
pub fn h_021dc908<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 4, me) || b.var(0x46) != 0 || b.var(0x16) != 16 {
        return;
    }
    let id = b.host.handex_push(hx::RANK_EFFECT, me as u32);
    let w = b.host.handex(id);
    let h = w.get_u32(0);
    w.put_u32(0, h | HANDEX_HEADER_POPUP_MSG);
    w.put_u8(0xF, 1);
    w.put_u8(0x10, me);
    w.put_u32(4, 1);
    w.put_u8(0xC, 1);
    b.host.handex_pop(id);
}

/// 0x021DC960: Rattled (155), event 0x4B: a hit of type 16, 6 or 7 raises Speed by 1.
pub fn h_021dc960<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 4, me) || b.var(0x46) != 0 {
        return;
    }
    let ty = b.var(0x16) as u8;
    if ty != 16 && ty.wrapping_add(250) > 1 {
        return;
    }
    let id = b.host.handex_push(hx::RANK_EFFECT, me as u32);
    let w = b.host.handex(id);
    let h = w.get_u32(0);
    w.put_u32(0, h | HANDEX_HEADER_POPUP);
    w.put_u8(0xF, 1);
    w.put_u8(0x10, me);
    w.put_u32(4, 5);
    w.put_u8(0xC, 1);
    b.host.handex_pop(id);
}

/// 0x021DC9C8: Mummy (152), event 0x4B: a contact move changes the attacker's ability to
/// ability 152 (message 0x1CF).
pub fn h_021dc9c8<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 4, me) || b.var(0x46) != 0 || b.host.fun_021c83d4() {
        return;
    }
    let mv = b.var(0x12) as u16 as u32;
    if !b.host.move_flag(mv, 0) {
        return;
    }
    let att = b.var(3) as u8 as u32;
    if b.mon_get(att, value::ABILITY_RAW) == 152 {
        return;
    }
    let id = b.host.handex_push(HANDEX_CHANGE_ABILITY, me as u32);
    let w = b.host.handex(id);
    w.put_u16(4, 152);
    w.put_u8(6, att as u8);
    str_param_setup(w, 12, 2, 0x1CF);
    str_param_add_arg(w, 12, att);
    if !same_side(me as u32, att) {
        let h = w.get_u32(0);
        w.put_u32(0, h | HANDEX_HEADER_POPUP);
    }
    b.host.handex_pop(id);
}

/// 0x021DCA68: Sap Sipper-style (157), event 0x2D: absorbs type 11 moves and raises Attack.
pub fn h_021dca68<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if absorbs(b, me, 11) {
        boost_own_stat(b, me, 1, 1);
    }
}

/// 0x021DCA98: (158), event 0x11: status moves get var 0x18 + 1.
pub fn h_021dca98<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let mv = b.var(0x12) as u16 as u32;
    if b.host.move_category(mv) == 0 {
        let v = b.var(0x18) as u8 as i32 + 1;
        b.rewrite_var(0x18, v);
    }
}

/// Unlabeled routine at 0x021E8744 (Magic Bounce): bounces a reflectable move aimed at
/// `me` from the other side.
pub(crate) fn magic_bounce_move<H: FlowHost>(b: &mut Battle<H>, me: u8) {
    let user = b.var(2) as u8 as u32;
    if same_side(user, me as u32) {
        return;
    }
    let mv = b.var(0x12) as u16 as u32;
    if b.host.move_effect_class(mv) != 11 || !b.host.move_flag(mv, 4) {
        return;
    }
    if b.rewrite_var(0x22, 25) {
        b.host.fun_021c8434(me as u32, user);
    }
}

/// Unlabeled routine at 0x021E8798 (Magic Bounce): bounces a reflectable move aimed at `me`.
pub(crate) fn magic_bounce_hit<H: FlowHost>(b: &mut Battle<H>, me: u8) {
    if !is_me(b, 4, me) || b.var(0x4E) != 0 || b.mon(me as u32).any_flag_b_3546() {
        return;
    }
    let mv = b.var(0x12) as u16 as u32;
    if !b.host.move_flag(mv, 4) || !b.rewrite_var(0x40, 1) {
        return;
    }
    let user = b.var(3) as u8 as u32;
    b.host.fun_021c8434(me as u32, user);
    b.rewrite_var(0x51, 1);
}

/// Unlabeled routine at 0x021E8804 (Magic Bounce): message 0x2FC naming the bounced move.
pub(crate) fn magic_bounce_message<H: FlowHost>(b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) || b.mon(me as u32).any_flag_b_3546() {
        return;
    }
    let id = b.host.handex_push(hx::MESSAGE, me as u32);
    let mv = b.var(0x12) as u32;
    let w = b.host.handex(id);
    str_param_setup(w, 4, 2, 0x2FC);
    str_param_add_arg(w, 4, me as u32);
    str_param_add_arg(w, 4, mv);
    b.host.handex_pop(id);
}

/// 0x021DCAD8: Magic Bounce (156), event 0x1F.
pub fn h_021dcad8<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    magic_bounce_move(b, me);
}
/// 0x021DCAE0: Magic Bounce (156), event 0x2D.
pub fn h_021dcae0<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    magic_bounce_hit(b, me);
}
/// 0x021DCAE8: Magic Bounce (156), event 9: announces the bounce.
pub fn h_021dcae8<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && b.rewrite_var(0x51, 1) {
        b.host.handex_push_pop(hx::ABILITY_POPUP_IN, me as u32);
        magic_bounce_message(b, me);
        b.host.handex_push_pop(hx::ABILITY_POPUP_OUT, me as u32);
    }
}

/// 0x021DCB38: Harvest (139), event 0x77: brings back the used berry (always in sun, else
/// 50%).
pub fn h_021dcb38<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) {
        return;
    }
    let item = b.mon(me as u32).u14();
    if item == 0 || !b.host.item_fun_020213c0(item as u32) || b.mon(me as u32).held_item() != 0 {
        return;
    }
    if b.host.weather() != 1 && !b.roll_percent_or_forced(50) {
        return;
    }
    let id = b.host.handex_push(HANDEX_GIVE_ITEM, me as u32);
    let w = b.host.handex(id);
    let h = w.get_u32(0);
    w.put_u32(0, h | HANDEX_HEADER_POPUP_MSG);
    w.put_u16(4, item);
    w.put_u8(6, me);
    w.put_u8(7, 1);
    str_param_setup(w, 12, 2, 0x1DB);
    str_param_add_arg(w, 12, me as u32);
    str_param_add_arg(w, 12, item as u32);
    b.host.handex_pop(id);
}

/// 0x021DCBE8: Heavy Metal (134), event 0x7B: doubles var 0x35.
pub fn h_021dcbe8<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        b.mul_var(0x35, 0x2000);
    }
}

/// 0x021DCC10: Light Metal (135), event 0x7B: halves var 0x35.
pub fn h_021dcc10<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        b.mul_var(0x35, 0x800);
    }
}

/// 0x021DCC2C: Contrary (126), event 0x5A: negates stat changes (var 0x20); work word 0
/// records whether the rewrite applied.
pub fn h_021dcc2c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let v = b.var(0x20);
        work(sys, f)[0] = b.rewrite_var(0x20, v.wrapping_neg()) as u32;
    }
}

/// Skip-check callback of [`h_021dcc80`] (address of the original routine).
pub const SKIP_CHECK_UNNERVE: u32 = 0x021D_CCED;

/// Unlabeled skip-check callback at 0x021DCCEC: Unnerve skips an opposing berry (type-5
/// factor) that `FUN_020213C0` accepts.
pub fn skip_021dccec<H: FlowHost>(
    sys: &EventSystem,
    f: FactorId,
    b: &mut Battle<H>,
    ftype: u32,
    _: u32,
    sub_id: u16,
    owner: u8,
) -> bool {
    ftype == 5
        && !same_side(sys.poke_id(f) as u32, owner as u32)
        && b.host.item_fun_020213c0(sub_id as u32)
}

/// 0x021DCC5C: Unnerve (127), event 0x55: see [`h_021dcc80`].
pub fn h_021dcc5c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        h_021dcc80(sys, f, b, me);
    }
}

/// 0x021DCC80: Unnerve (127), events 0x56 / 0x58: announces the ability once (message 0xB0)
/// and installs [`skip_021dccec`].
pub fn h_021dcc80<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if work(sys, f)[0] != 0 {
        return;
    }
    let other = crate::main_util::other_side(crate::main_util::poke_side(me as u32) as u32);
    b.host.handex_push_pop(hx::ABILITY_POPUP_IN, me as u32);
    let id = b.host.handex_push(hx::MESSAGE, me as u32);
    let w = b.host.handex(id);
    str_param_setup(w, 4, 1, 0xB0);
    str_param_add_arg(w, 4, other as u32);
    b.host.handex_pop(id);
    b.host.handex_push_pop(hx::ABILITY_POPUP_OUT, me as u32);
    sys.set_skip_check(f, SKIP_CHECK_UNNERVE);
    work(sys, f)[0] = 1;
}

/// HANDEX work type 0x33: transform into another Pokemon (+4 target, +8 string).
const HANDEX_TRANSFORM: u32 = 0x33;

/// 0x021DCD20: Imposter (150), event 0x55: transforms into the Pokemon opposite.
pub fn h_021dcd20<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) {
        return;
    }
    let pos = b.host.poke_pos_in_battle(me as u32);
    let rule = b.host.rule();
    let opposite = b.host.fun_021b8c5c(rule, pos);
    let target = b.host.poke_id_at_pos(opposite);
    if b.mon_is_fainted(me as u32) || b.mon_is_fainted(target) {
        return;
    }
    let id = b.host.handex_push(HANDEX_TRANSFORM, me as u32);
    let w = b.host.handex(id);
    let h = w.get_u32(0);
    w.put_u32(0, h | HANDEX_HEADER_POPUP);
    w.put_u8(4, target as u8);
    str_param_setup(w, 8, 2, 0x284);
    str_param_add_arg(w, 8, me as u32);
    str_param_add_arg(w, 8, target as u8 as u32);
    b.host.handex_pop(id);
}

/// HANDEX work type 0x34: end an illusion (+4 Pokemon, +8 string).
const HANDEX_ILLUSION_END: u32 = 0x34;

/// Unlabeled routine at 0x021DCE48: ends this Pokemon's Illusion (message 0x1DE).
fn illusion_end<H: FlowHost>(b: &mut Battle<H>, me: u8) {
    if !b.mon(me as u32).is_illusion_active() {
        return;
    }
    let id = b.host.handex_push(HANDEX_ILLUSION_END, me as u32);
    let w = b.host.handex(id);
    w.put_u8(4, me);
    str_param_setup(w, 8, 2, 0x1DE);
    str_param_add_arg(w, 8, me as u32);
    b.host.handex_pop(id);
}

/// 0x021DCDD0: Illusion (149), event 0x4B.
pub fn h_021dcdd0<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me) && b.var(0x46) == 0 {
        illusion_end(b, me);
    }
}
/// 0x021DCDF8: Illusion (149), event 0x6A.
pub fn h_021dcdf8<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        illusion_end(b, me);
    }
}
/// 0x021DCE18: Illusion (149), event 0x89: ends it unless var 0x10 is this factor's ability.
pub fn h_021dce18<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if b.var(0x10) as u32 != sys.sub_id(f) as u32 && is_me(b, 2, me) {
        illusion_end(b, me);
    }
}

/// 0x021DCE9C: (162), event 0x34: var 0x35 x1.1 when the attacker is on this side.
pub fn h_021dce9c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if same_side(me as u32, b.var(3) as u8 as u32) {
        b.mul_var(0x35, 0x119A);
    }
}

/// Original addresses of the ported handlers (also the differential test order).
pub const PORTED: &[u32] = &[
    0x021D85F4, 0x021D8684, 0x021D86A8, 0x021D86FC, 0x021D8734, 0x021D8760, 0x021D8794, 0x021D87C8,
    0x021D8808, 0x021D8844, 0x021D8874, 0x021D88B8, 0x021D88E8, 0x021D890C, 0x021D8940, 0x021D8964,
    0x021D89B8, 0x021D8A3C, 0x021D8A64, 0x021D8A8C, 0x021D8AA8, 0x021D8AD0, 0x021D8B1C, 0x021D8B50,
    0x021D8B84, 0x021D8BB4, 0x021D8C0C, 0x021D8C3C, 0x021D8C58, 0x021D8C74, 0x021D8C90, 0x021D8CF4,
    0x021D8D40, 0x021D8E1C, 0x021D8E58, 0x021D8EDC, 0x021D8F1C, 0x021D8F54, 0x021D8F84, 0x021D8FCC,
    0x021D901C, 0x021D9070, 0x021D90E8, 0x021D9118, 0x021D9154, 0x021D919C, 0x021D91E4, 0x021D9214,
    0x021D9224, 0x021D9240, 0x021D9250, 0x021D926C, 0x021D927C, 0x021D9340, 0x021D936C, 0x021D93A8,
    0x021D93D8, 0x021D93EC, 0x021D93FC, 0x021D9418, 0x021D9440, 0x021D9450, 0x021D9460, 0x021D9484,
    0x021D9498, 0x021D94A8, 0x021D94C4, 0x021D94D8, 0x021D94E8, 0x021D9504, 0x021D9518, 0x021D9528,
    0x021D9544, 0x021D9558, 0x021D956C, 0x021D957C, 0x021D9598, 0x021D95AC, 0x021D95BC, 0x021D95CC,
    0x021D96CC, 0x021D9744, 0x021D9760, 0x021D977C, 0x021D9798, 0x021D97EC, 0x021D9830, 0x021D9848,
    0x021D9864, 0x021D98DC, 0x021D9934, 0x021D9978, 0x021D99A4, 0x021D99C8, 0x021D99E0, 0x021D99F8,
    0x021D9A24, 0x021D9A44, 0x021D9A68, 0x021D9ADC, 0x021D9B40, 0x021D9BA4, 0x021D9BC8, 0x021D9BF8,
    0x021D9C8C, 0x021D9CBC, 0x021D9CEC, 0x021D9D1C, 0x021D9DB0, 0x021D9E80, 0x021D9F1C, 0x021D9FC4,
    0x021DA04C, 0x021DA0FC, 0x021DA120, 0x021DA150, 0x021DA2F0, 0x021DA32C, 0x021DA3D8, 0x021DA528,
    0x021DA624, 0x021DA6EC, 0x021DA748, 0x021DA780, 0x021DA7DC, 0x021DA808, 0x021DA820, 0x021DA844,
    0x021DA868, 0x021DA9E0, 0x021DAA70, 0x021DAA94, 0x021DAAC0, 0x021DAAEC, 0x021DAB24, 0x021DAB48,
    0x021DAB78, 0x021DABF4, 0x021DAC1C, 0x021DAC6C, 0x021DAC90, 0x021DAD2C, 0x021DAD58, 0x021DAD84,
    0x021DADD8, 0x021DAE04, 0x021DAE38, 0x021DAE94, 0x021DAEA4, 0x021DAEAC, 0x021DAEE4, 0x021DAF1C,
    0x021DAF58, 0x021DB00C, 0x021DB0DC, 0x021DB0E4, 0x021DB134, 0x021DB1D4, 0x021DB200, 0x021DB224,
    0x021DB254, 0x021DB274, 0x021DB298, 0x021DB2B8, 0x021DB2C8, 0x021DB2F8, 0x021DB41C, 0x021DB430,
    0x021DB460, 0x021DB474, 0x021DB4C4, 0x021DB560, 0x021DB5C0, 0x021DB62C, 0x021DB6A4, 0x021DB6C0,
    0x021DB6E4, 0x021DB6FC, 0x021DB734, 0x021DB774, 0x021DB7E8, 0x021DB854, 0x021DB87C, 0x021DB8E8,
    0x021DB93C, 0x021DB9B8, 0x021DB9DC, 0x021DBA0C, 0x021DBA80, 0x021DBAF0, 0x021DBB64, 0x021DBB98,
    0x021DBBD8, 0x021DBCEC, 0x021DBD9C, 0x021DBE6C, 0x021DBF28, 0x021DBF58, 0x021DBF84, 0x021DBF9C,
    0x021DC054, 0x021DC0BC, 0x021DC104, 0x021DC13C, 0x021DC170, 0x021DC218, 0x021DC25C, 0x021DC2A0,
    0x021DC32C, 0x021DC4A4, 0x021DC4D4, 0x021DC57C, 0x021DC604, 0x021DC614, 0x021DC630, 0x021DC664,
    0x021DC6A0, 0x021DC6F8, 0x021DC738, 0x021DC7BC, 0x021DC844, 0x021DC864, 0x021DC894, 0x021DC908,
    0x021DC960, 0x021DC9C8, 0x021DCA68, 0x021DCA98, 0x021DCAD8, 0x021DCAE0, 0x021DCAE8, 0x021DCB38,
    0x021DCBE8, 0x021DCC10, 0x021DCC2C, 0x021DCC5C, 0x021DCC80, 0x021DCD20, 0x021DCDD0, 0x021DCDF8,
    0x021DCE18, 0x021DCE9C,
];

/// Ported skip-check callbacks by the address stored in the factor.
pub fn skip_check<H: FlowHost>(addr: u32) -> Option<crate::flow::SkipCheck<H>> {
    Some(match addr & !1 {
        0x021DAEC4 => skip_021daec4::<H>,
        0x021DB1A8 => skip_021db1a8::<H>,
        0x021DB650 => skip_021db650::<H>,
        0x021DC820 => skip_021dc820::<H>,
        0x021DCCEC => skip_021dccec::<H>,
        _ => return None,
    })
}

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
        0x021D88B8 => h_021d88b8::<H>,
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
        0x021D8E1C => h_021d8e1c::<H>,
        0x021D8E58 => h_021d8e58::<H>,
        0x021D8EDC => h_021d8edc::<H>,
        0x021D8F1C => h_021d8f1c::<H>,
        0x021D8F54 => h_021d8f54::<H>,
        0x021D8F84 => h_021d8f84::<H>,
        0x021D8FCC => h_021d8fcc::<H>,
        0x021D901C => h_021d901c::<H>,
        0x021D9070 => h_021d9070::<H>,
        0x021D90E8 => h_021d90e8::<H>,
        0x021D9118 => h_021d9118::<H>,
        0x021D9154 => h_021d9154::<H>,
        0x021D919C => h_021d919c::<H>,
        0x021D91E4 => h_021d91e4::<H>,
        0x021D9214 => h_021d9214::<H>,
        0x021D9224 => h_021d9224::<H>,
        0x021D9240 => h_021d9240::<H>,
        0x021D9250 => h_021d9250::<H>,
        0x021D926C => h_021d926c::<H>,
        0x021D927C => h_021d927c::<H>,
        0x021D9340 => h_021d9340::<H>,
        0x021D936C => h_021d936c::<H>,
        0x021D93A8 => h_021d93a8::<H>,
        0x021D93D8 => h_021d93d8::<H>,
        0x021D93EC => h_021d93ec::<H>,
        0x021D93FC => h_021d93fc::<H>,
        0x021D9418 => h_021d9418::<H>,
        0x021D9440 => h_021d9440::<H>,
        0x021D9450 => h_021d9450::<H>,
        0x021D9460 => h_021d9460::<H>,
        0x021D9484 => h_021d9484::<H>,
        0x021D9498 => h_021d9498::<H>,
        0x021D94A8 => h_021d94a8::<H>,
        0x021D94C4 => h_021d94c4::<H>,
        0x021D94D8 => h_021d94d8::<H>,
        0x021D94E8 => h_021d94e8::<H>,
        0x021D9504 => h_021d9504::<H>,
        0x021D9518 => h_021d9518::<H>,
        0x021D9528 => h_021d9528::<H>,
        0x021D9544 => h_021d9544::<H>,
        0x021D9558 => h_021d9558::<H>,
        0x021D956C => h_021d956c::<H>,
        0x021D957C => h_021d957c::<H>,
        0x021D9598 => h_021d9598::<H>,
        0x021D95AC => h_021d95ac::<H>,
        0x021D95BC => h_021d95bc::<H>,
        0x021D95CC => h_021d95cc::<H>,
        0x021D96CC => h_021d96cc::<H>,
        0x021D9744 => h_021d9744::<H>,
        0x021D9760 => h_021d9760::<H>,
        0x021D977C => h_021d977c::<H>,
        0x021D9798 => h_021d9798::<H>,
        0x021D97EC => h_021d97ec::<H>,
        0x021D9830 => h_021d9830::<H>,
        0x021D9848 => h_021d9848::<H>,
        0x021D9864 => h_021d9864::<H>,
        0x021D98DC => h_021d98dc::<H>,
        0x021D9934 => h_021d9934::<H>,
        0x021D9978 => h_021d9978::<H>,
        0x021D99A4 => h_021d99a4::<H>,
        0x021D99C8 => h_021d99c8::<H>,
        0x021D99E0 => h_021d99e0::<H>,
        0x021D99F8 => h_021d99f8::<H>,
        0x021D9A24 => h_021d9a24::<H>,
        0x021D9A44 => h_021d9a44::<H>,
        0x021D9A68 => h_021d9a68::<H>,
        0x021D9ADC => h_021d9adc::<H>,
        0x021D9B40 => h_021d9b40::<H>,
        0x021D9BA4 => h_021d9ba4::<H>,
        0x021D9BC8 => h_021d9bc8::<H>,
        0x021D9BF8 => h_021d9bf8::<H>,
        0x021D9C8C => h_021d9c8c::<H>,
        0x021D9CBC => h_021d9cbc::<H>,
        0x021D9CEC => h_021d9cec::<H>,
        0x021D9D1C => h_021d9d1c::<H>,
        0x021D9DB0 => h_021d9db0::<H>,
        0x021D9E80 => h_021d9e80::<H>,
        0x021D9F1C => h_021d9f1c::<H>,
        0x021D9FC4 => h_021d9fc4::<H>,
        0x021DA04C => h_021da04c::<H>,
        0x021DA0FC => h_021da0fc::<H>,
        0x021DA120 => h_021da120::<H>,
        0x021DA150 => h_021da150::<H>,
        0x021DA2F0 => h_021da2f0::<H>,
        0x021DA32C => h_021da32c::<H>,
        0x021DA3D8 => h_021da3d8::<H>,
        0x021DA528 => h_021da528::<H>,
        0x021DA624 => h_021da624::<H>,
        0x021DA6EC => h_021da6ec::<H>,
        0x021DA748 => h_021da748::<H>,
        0x021DA780 => h_021da780::<H>,
        0x021DA7DC => h_021da7dc::<H>,
        0x021DA808 => h_021da808::<H>,
        0x021DA820 => h_021da820::<H>,
        0x021DA844 => h_021da844::<H>,
        0x021DA868 => h_021da868::<H>,
        0x021DA9E0 => h_021da9e0::<H>,
        0x021DAA70 => h_021daa70::<H>,
        0x021DAA94 => h_021daa94::<H>,
        0x021DAAC0 => h_021daac0::<H>,
        0x021DAAEC => h_021daaec::<H>,
        0x021DAB24 => h_021dab24::<H>,
        0x021DAB48 => h_021dab48::<H>,
        0x021DAB78 => h_021dab78::<H>,
        0x021DABF4 => h_021dabf4::<H>,
        0x021DAC1C => h_021dac1c::<H>,
        0x021DAC6C => h_021dac6c::<H>,
        0x021DAC90 => h_021dac90::<H>,
        0x021DAD2C => h_021dad2c::<H>,
        0x021DAD58 => h_021dad58::<H>,
        0x021DAD84 => h_021dad84::<H>,
        0x021DADD8 => h_021dadd8::<H>,
        0x021DAE04 => h_021dae04::<H>,
        0x021DAE38 => h_021dae38::<H>,
        0x021DAE94 => h_021dae94::<H>,
        0x021DAEA4 => h_021daea4::<H>,
        0x021DAEAC => h_021daeac::<H>,
        0x021DAEE4 => h_021daee4::<H>,
        0x021DAF1C => h_021daf1c::<H>,
        0x021DAF58 => h_021daf58::<H>,
        0x021DB00C => h_021db00c::<H>,
        0x021DB0DC => h_021db0dc::<H>,
        0x021DB0E4 => h_021db0e4::<H>,
        0x021DB134 => h_021db134::<H>,
        0x021DB1D4 => h_021db1d4::<H>,
        0x021DB200 => h_021db200::<H>,
        0x021DB224 => h_021db224::<H>,
        0x021DB254 => h_021db254::<H>,
        0x021DB274 => h_021db274::<H>,
        0x021DB298 => h_021db298::<H>,
        0x021DB2B8 => h_021db2b8::<H>,
        0x021DB2C8 => h_021db2c8::<H>,
        0x021DB2F8 => h_021db2f8::<H>,
        0x021DB41C => h_021db41c::<H>,
        0x021DB430 => h_021db430::<H>,
        0x021DB460 => h_021db460::<H>,
        0x021DB474 => h_021db474::<H>,
        0x021DB4C4 => h_021db4c4::<H>,
        0x021DB560 => h_021db560::<H>,
        0x021DB5C0 => h_021db5c0::<H>,
        0x021DB62C => h_021db62c::<H>,
        0x021DB6A4 => h_021db6a4::<H>,
        0x021DB6C0 => h_021db6c0::<H>,
        0x021DB6E4 => h_021db6e4::<H>,
        0x021DB6FC => h_021db6fc::<H>,
        0x021DB734 => h_021db734::<H>,
        0x021DB774 => h_021db774::<H>,
        0x021DB7E8 => h_021db7e8::<H>,
        0x021DB854 => h_021db854::<H>,
        0x021DB87C => h_021db87c::<H>,
        0x021DB8E8 => h_021db8e8::<H>,
        0x021DB93C => h_021db93c::<H>,
        0x021DB9B8 => h_021db9b8::<H>,
        0x021DB9DC => h_021db9dc::<H>,
        0x021DBA0C => h_021dba0c::<H>,
        0x021DBA80 => h_021dba80::<H>,
        0x021DBAF0 => h_021dbaf0::<H>,
        0x021DBB64 => h_021dbb64::<H>,
        0x021DBB98 => h_021dbb98::<H>,
        0x021DBBD8 => h_021dbbd8::<H>,
        0x021DBCEC => h_021dbcec::<H>,
        0x021DBD9C => h_021dbd9c::<H>,
        0x021DBE6C => h_021dbe6c::<H>,
        0x021DBF28 => h_021dbf28::<H>,
        0x021DBF58 => h_021dbf58::<H>,
        0x021DBF84 => h_021dbf84::<H>,
        0x021DBF9C => h_021dbf9c::<H>,
        0x021DC054 => h_021dc054::<H>,
        0x021DC0BC => h_021dc0bc::<H>,
        0x021DC104 => h_021dc104::<H>,
        0x021DC13C => h_021dc13c::<H>,
        0x021DC170 => h_021dc170::<H>,
        0x021DC218 => h_021dc218::<H>,
        0x021DC25C => h_021dc25c::<H>,
        0x021DC2A0 => h_021dc2a0::<H>,
        0x021DC32C => h_021dc32c::<H>,
        0x021DC4A4 => h_021dc4a4::<H>,
        0x021DC4D4 => h_021dc4d4::<H>,
        0x021DC57C => h_021dc57c::<H>,
        0x021DC604 => h_021dc604::<H>,
        0x021DC614 => h_021dc614::<H>,
        0x021DC630 => h_021dc630::<H>,
        0x021DC664 => h_021dc664::<H>,
        0x021DC6A0 => h_021dc6a0::<H>,
        0x021DC6F8 => h_021dc6f8::<H>,
        0x021DC738 => h_021dc738::<H>,
        0x021DC7BC => h_021dc7bc::<H>,
        0x021DC844 => h_021dc844::<H>,
        0x021DC864 => h_021dc864::<H>,
        0x021DC894 => h_021dc894::<H>,
        0x021DC908 => h_021dc908::<H>,
        0x021DC960 => h_021dc960::<H>,
        0x021DC9C8 => h_021dc9c8::<H>,
        0x021DCA68 => h_021dca68::<H>,
        0x021DCA98 => h_021dca98::<H>,
        0x021DCAD8 => h_021dcad8::<H>,
        0x021DCAE0 => h_021dcae0::<H>,
        0x021DCAE8 => h_021dcae8::<H>,
        0x021DCB38 => h_021dcb38::<H>,
        0x021DCBE8 => h_021dcbe8::<H>,
        0x021DCC10 => h_021dcc10::<H>,
        0x021DCC2C => h_021dcc2c::<H>,
        0x021DCC5C => h_021dcc5c::<H>,
        0x021DCC80 => h_021dcc80::<H>,
        0x021DCD20 => h_021dcd20::<H>,
        0x021DCDD0 => h_021dcdd0::<H>,
        0x021DCDF8 => h_021dcdf8::<H>,
        0x021DCE18 => h_021dce18::<H>,
        0x021DCE9C => h_021dce9c::<H>,
        _ => return None,
    })
}

#[cfg(test)]
mod tests;
