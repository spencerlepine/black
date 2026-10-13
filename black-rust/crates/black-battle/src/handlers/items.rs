//! Held-item handlers (`hand_tok`): registry at 0x021F1EA4 (171 item IDs -> getter -> table in
//! the pool 0x021F136C-0x021F1EA4), code 0x021DD05C-0x021E02C0.
//!
//! Each handler is C `void handler(factor, flow_work, poke_id, work)`; the factor is a type-5
//! factor whose sub ID (`EventSystem::sub_id`) is the item ID. The doc comment gives the
//! original address, the items using it and the event. Event-variable labels are kept numeric
//! (see `event_var`).

use black_core::softfloat::{f2i, fadd, fsub, i2f, u2f};

use crate::event_factor::{EventSystem, FactorId};
use crate::flow::{
    str_param_add_arg, str_param_set_extra, str_param_setup, Battle, FlowHost, HandexBytes,
};
use crate::main_util::same_side;
use crate::poke_type::pair_has_type;
use crate::sick_cont::SickCont;

fn is_me<H: FlowHost>(b: &Battle<H>, label: u16, me: u8) -> bool {
    me as u32 == b.var(label) as u32
}

/// C: `FUN_overlay_d_93__021df4e0(factor, flow, poke_id, kind, work)`: when var 3 is the holder
/// and var 0x39 equals `kind`, sets var 0x24 to the item's parameter 2.
fn rewrite_24_from_item_param<H: FlowHost>(
    sys: &mut EventSystem,
    f: FactorId,
    b: &mut Battle<H>,
    me: u8,
    kind: i32,
) {
    if is_me(b, 3, me) && b.var(0x39) == kind {
        // C: `FUN_overlay_d_93__021dcff8(factor, 2)`: `item_param(sub_id, 2)`.
        let v = b.host.item_param(sys.sub_id(f) as u32, 2);
        b.rewrite_var(0x24, v as i32);
    }
}

/// C: `FUN_overlay_d_93__021dd044(factor, flow, poke_id)`: lets the factor run again and pushes
/// and pops a parameterless HANDEX work of type 0.
fn allow_reentry_and_pop<H: FlowHost>(
    sys: &mut EventSystem,
    f: FactorId,
    b: &mut Battle<H>,
    me: u32,
) {
    sys.allow_reentry(f);
    b.host.handex_push_pop(0, me);
}

/// C: `FUN_overlay_d_93__021d7e10(bpp)`: the first condition of the 6-entry table the Pokemon
/// has, or 0.
fn first_table_sick<H: FlowHost>(b: &mut Battle<H>, me: u32) -> u32 {
    let mut i = 0;
    loop {
        let e = b.host.sick_list_entry(i);
        i += 1;
        if e == 0 {
            return 0;
        }
        if b.mon(me).has_sick(e) {
            return e;
        }
    }
}

/// C: unlabeled routine at `0x021DD1D8(flow, poke_id, kind)`: whether the holder has the
/// condition `kind` (0x24: a major status, 0x25: a major status or sick 6, 0x26: any condition
/// of the table).
fn holder_has_condition<H: FlowHost>(b: &mut Battle<H>, me: u32, kind: u32) -> bool {
    match kind {
        0x24 => b.mon(me).major_status() != 0,
        0x25 => b.mon(me).major_status() != 0 || b.mon(me).has_sick(6),
        0x26 => first_table_sick(b, me) != 0,
        _ => b.mon(me).has_sick(kind),
    }
}

/// C: unlabeled routine at `0x021DD158(factor, flow, poke_id, kind)`: when var 2 is the holder
/// and var 0x2E is absent, 3 or 0, and the holder has condition `kind`, runs `021dd044`.
fn cure_if_condition<H: FlowHost>(
    sys: &mut EventSystem,
    f: FactorId,
    b: &mut Battle<H>,
    me: u8,
    kind: u32,
) {
    let v = b.vars_ref().get_value_if_exists(0x2E);
    if is_me(b, 2, me)
        && (v.is_none() || v == Some(3) || v == Some(0))
        && holder_has_condition(b, me as u32, kind)
    {
        allow_reentry_and_pop(sys, f, b, me as u32);
    }
}

/// C: unlabeled routine at `0x021DD19C(factor, flow, poke_id, kind)`: when var 2 is the holder
/// and it has condition `kind`, pushes HANDEX work 0xB (+4 `kind`, +8 holder, +0x14 = 1).
fn push_cure_work<H: FlowHost>(b: &mut Battle<H>, me: u8, kind: u32) {
    if is_me(b, 2, me) && holder_has_condition(b, me as u32, kind) {
        let w = b.host.handex_push(0xB, me as u32);
        let buf = b.host.handex(w);
        buf.put_u32(4, kind);
        buf.put_u8(0x14, 1);
        buf.put_u8(8, me);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021DD338(flow, poke_id)`: the slot of the holder's last move if
/// that move is out of PP, else 4.
fn last_move_empty_slot<H: FlowHost>(b: &mut Battle<H>, me: u32) -> u32 {
    let mon = b.mon(me);
    let slot = mon.move_slot_of(mon.last_move_orig() as u32) as u32;
    if slot != 4 && mon.move_pp(slot) == 0 {
        slot
    } else {
        4
    }
}

/// C: unlabeled routine at `0x021DD368(flow, poke_id)`: the first move slot out of PP, else 4.
fn first_empty_slot<H: FlowHost>(b: &mut Battle<H>, me: u32) -> u32 {
    let mon = b.mon(me);
    let n = mon.move_count() as u32;
    let mut i = 0u32;
    if n != 0 {
        loop {
            if mon.move_pp(i & 0xFF) == 0 {
                return i & 0xFF;
            }
            i = i.wrapping_add(1);
            if i >= n {
                break;
            }
        }
    }
    4
}

/// C: unlabeled routine at `0x021DD39C(factor, flow, poke_id, work, any_slot)`: restores PP of
/// an empty move slot (the last used one, else the first; with `any_slot` else the first slot
/// missing PP) through HANDEX work 9; returns whether a slot was found.
fn restore_pp<H: FlowHost>(
    sys: &mut EventSystem,
    f: FactorId,
    b: &mut Battle<H>,
    me: u32,
    any_slot: bool,
) -> bool {
    let mut slot = last_move_empty_slot(b, me);
    if slot == 4 {
        slot = first_empty_slot(b, me);
        if slot == 4 && any_slot {
            let mon = b.mon(me);
            let n = mon.move_count() as u32;
            let mut i = 0u32;
            if n != 0 {
                loop {
                    if mon.move_pp_missing(i & 0xFF) != 0 {
                        slot = i & 0xFF;
                        break;
                    }
                    i = i.wrapping_add(1);
                    if i >= n {
                        break;
                    }
                }
            }
        }
    }
    if slot == 4 {
        return false;
    }
    let mv = b.mon(me).move_id(slot) as u32;
    let item = sys.sub_id(f) as u32;
    let w = b.host.handex_push(9, me);
    let buf = b.host.handex(w);
    buf.put_u8(4, 10);
    buf.put_u8(5, me as u8);
    buf.put_u8(6, slot as u8);
    buf.put_u8(7, buf[7] & 0xFE | 1);
    str_param_setup(buf, 8, 2, 0x38F);
    str_param_add_arg(buf, 8, me);
    str_param_add_arg(buf, 8, item);
    str_param_add_arg(buf, 8, mv);
    b.host.handex_pop(w);
    true
}

/// C: `FUN_overlay_d_93__021dcff8(factor, param)`: `item_param(sub_id, param)`.
fn item_param<H: FlowHost>(
    sys: &mut EventSystem,
    f: FactorId,
    b: &mut Battle<H>,
    param: u32,
) -> u32 {
    b.host.item_param(sys.sub_id(f) as u32, param)
}

/// C: `FUN_overlay_d_93__021dd008(factor)`: `percent_fixed(item_param(factor, 2))`.
fn item_percent_fixed<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>) -> i32 {
    percent_fixed(item_param(sys, f, b, 2) as i32)
}

/// The arithmetic of `021dd008`: `(param + 100) * 4096` as a single (`i2f`), moved 0.5 away from
/// zero (`fadd(0.5, f)` when positive, else `fsub(f, 0.5)`), truncated (`f2i`) and divided by
/// 100 (`_s32_div_f`): an item's percentage boost as a 4096-based fixed-point multiplier.
/// Soft-float helpers: `black_core::softfloat`.
fn percent_fixed(param: i32) -> i32 {
    let v = param.wrapping_add(100);
    let x = i2f(v.wrapping_shl(12));
    let r = if v > 0 {
        fadd(0x3F00_0000, x)
    } else {
        fsub(x, 0x3F00_0000)
    };
    f2i(r).wrapping_div(100)
}

/// C: `FUN_overlay_d_93__021ddea4(flow, poke_id, n)`: `n` is halved when BattleMon get 0x11 is
/// 0x52 and `n` > 2, and becomes 1 when 0; true when BattleMon get 0xE is above 1 and get 0xD is
/// at most the holder's max HP / `n`. (Meaning of 0x11, 0xD, 0xE: UNVERIFIED.)
fn max_hp_check<H: FlowHost>(b: &mut Battle<H>, me: u32, n: u32) -> bool {
    let mut n = n;
    if b.mon_get(me, 0x11) == 0x52 && n > 2 {
        n >>= 1;
    }
    if n == 0 {
        n = 1;
    }
    if b.mon_get(me, 0xE) > 1 {
        let cur = b.mon_get(me, 0xD);
        return cur <= b.max_hp_div(me, n);
    }
    false
}

/// C: `FUN_overlay_d_93__021dde74(flow, poke_id, n)`: when var 2 is the holder and var 0x2E
/// (as a byte) is below 2, `max_hp_check` with `n`; else false.
fn max_hp_check_var2<H: FlowHost>(b: &mut Battle<H>, me: u32, n: u32) -> bool {
    me == b.var(2) as u32 && (b.var(0x2E) as u32 & 0xFF) < 2 && max_hp_check(b, me, n)
}

/// C: `FUN_overlay_d_93__021dde30(factor, flow, poke_id, n, gated)`: when var 2 is the holder,
/// runs `max_hp_check_var2` if `gated` else `max_hp_check`, and on true `allow_reentry_and_pop`.
fn max_hp_check_reentry<H: FlowHost>(
    sys: &mut EventSystem,
    f: FactorId,
    b: &mut Battle<H>,
    me: u32,
    n: u32,
    gated: bool,
) {
    if me != b.var(2) as u32 {
        return;
    }
    let ok = if gated {
        max_hp_check_var2(b, me, n)
    } else {
        max_hp_check(b, me, n)
    };
    if ok {
        allow_reentry_and_pop(sys, f, b, me);
    }
}

/// C: `FUN_021dd9e8(flow, poke_id, value, flag)`: when var 2 is the holder, pushes HANDEX work
/// 0xE (+4 `value` as u32, +0xC `flag`, +0xF = 1, +0x10 holder) and pops it.
fn push_work_0e<H: FlowHost>(b: &mut Battle<H>, me: u32, value: u32, flag: u8) {
    if me != b.var(2) as u32 {
        return;
    }
    let w = b.host.handex_push(0xE, me);
    let buf = b.host.handex(w);
    buf.put_u32(4, value);
    buf.put_u8(0xC, flag);
    buf.put_u8(0xF, 1);
    buf.put_u8(0x10, me as u8);
    b.host.handex_pop(w);
}

/// C: `FUN_overlay_d_93__021dd7b8(factor, flow, poke_id, work, kind)`: when `work[0]` is 0 and
/// var 2 is the holder: pushes HANDEX work 5 (+6 holder, +4 u16 = holder's max HP / item param
/// 2, at least 1, str 0x38C with holder and item ID); if `FUN_020189D0(pokemon, kind)` is -1,
/// pushes work 0xC (+0xF holder, +4 = 6, +8 `make_sick_cont(6)`); then sets `work[0]` to 1.
fn fraction_work_and_sick<H: FlowHost>(
    sys: &mut EventSystem,
    f: FactorId,
    b: &mut Battle<H>,
    me: u8,
    kind: u32,
) {
    if sys.factor_mut(f).work[0] != 0 || me as u32 != b.var(2) as u32 {
        return;
    }
    let w = b.host.handex_push(5, me as u32);
    b.host.handex(w).put_u8(6, me);
    let p = item_param(sys, f, b, 2);
    let div = b.max_hp_div_min1(me as u32, p);
    let buf = b.host.handex(w);
    buf.put_u16(4, div as u16);
    str_param_setup(buf, 8, 2, 0x38C);
    str_param_add_arg(buf, 8, me as u32);
    str_param_add_arg(buf, 8, sys.sub_id(f) as u32);
    b.host.handex_pop(w);
    let pokemon = b.mon(me as u32).get_pokemon();
    if b.host.fun_020189d0(pokemon, kind) == -1 {
        let w = b.host.handex_push(0xC, me as u32);
        let cont = b.make_sick_cont(6, me as u32).0;
        let buf = b.host.handex(w);
        buf.put_u8(0xF, me);
        buf.put_u32(4, 6);
        buf.put_u32(8, cont);
        b.host.handex_pop(w);
    }
    sys.factor_mut(f).work[0] = 1;
}

/// C: `FUN_overlay_d_93__021ddcd8(factor, flow, poke_id, work, kind, p)`: when var 4 is the
/// holder, var 0x16 equals `kind` and (`p` != 0 or var 0x38 > 3), and the holder has no
/// substitute: multiplies var 0x35 by 0x800 and sets `work[0]` to 1 unless `FUN_021c7ec4(flow)`.
fn kind_check_and_mark<H: FlowHost>(
    sys: &mut EventSystem,
    f: FactorId,
    b: &mut Battle<H>,
    me: u32,
    kind: u32,
    p: u32,
) {
    if me != b.var(4) as u32 || kind != b.var(0x16) as u32 {
        return;
    }
    if p == 0 && b.var(0x38) <= 3 {
        return;
    }
    if b.mon(me).has_substitute() {
        return;
    }
    b.mul_var(0x35, 0x800);
    if !b.host.fun_021c7ec4() {
        sys.factor_mut(f).work[0] = 1;
    }
}

/// C: unlabeled routine at `0x021DD05C`: `021dd158` with kind 1.
pub fn h_021dd05c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    cure_if_condition(sys, f, b, me, 1);
}

/// C: unlabeled routine at `0x021DD080`: `021dd158` with kind 2.
pub fn h_021dd080<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    cure_if_condition(sys, f, b, me, 2);
}

/// C: unlabeled routine at `0x021DD0A4`: `021dd158` with kind 4.
pub fn h_021dd0a4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    cure_if_condition(sys, f, b, me, 4);
}

/// C: unlabeled routine at `0x021DD0C8`: `021dd158` with kind 3.
pub fn h_021dd0c8<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    cure_if_condition(sys, f, b, me, 3);
}

/// C: unlabeled routine at `0x021DD0EC`: `021dd158` with kind 6.
pub fn h_021dd0ec<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    cure_if_condition(sys, f, b, me, 6);
}

/// C: unlabeled routine at `0x021DD110`: `021dd158` with kind 5.
pub fn h_021dd110<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    cure_if_condition(sys, f, b, me, 5);
}

/// C: unlabeled routine at `0x021DD134`: `021dd158` with kind 0x25.
pub fn h_021dd134<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    cure_if_condition(sys, f, b, me, 0x25);
}

/// C: unlabeled routine at `0x021DD068`: `021dd19c` with kind 1.
pub fn h_021dd068<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    push_cure_work(b, me, 1);
}

/// C: unlabeled routine at `0x021DD08C`: `021dd19c` with kind 2.
pub fn h_021dd08c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    push_cure_work(b, me, 2);
}

/// C: unlabeled routine at `0x021DD0B0`: `021dd19c` with kind 4.
pub fn h_021dd0b0<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    push_cure_work(b, me, 4);
}

/// C: unlabeled routine at `0x021DD0D4`: `021dd19c` with kind 3.
pub fn h_021dd0d4<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    push_cure_work(b, me, 3);
}

/// C: unlabeled routine at `0x021DD0F8`: `021dd19c` with kind 6.
pub fn h_021dd0f8<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    push_cure_work(b, me, 6);
}

/// C: unlabeled routine at `0x021DD11C`: `021dd19c` with kind 5.
pub fn h_021dd11c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    push_cure_work(b, me, 5);
}

/// C: unlabeled routine at `0x021DD140`: `021dd19c` with kind 0x25.
pub fn h_021dd140<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    push_cure_work(b, me, 0x25);
}

/// C: unlabeled routine at `0x021DD240`: when var 2 is the holder and its last move is out of
/// PP, `021dd044`.
pub fn h_021dd240<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && last_move_empty_slot(b, me as u32) != 4 {
        allow_reentry_and_pop(sys, f, b, me as u32);
    }
}

/// C: unlabeled routine at `0x021DD26C`: like `021dd2a4` when var 0x2E is 2 or 0.
pub fn h_021dd26c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let v = b.var(0x2E);
        if (v == 2 || v == 0) && first_empty_slot(b, me as u32) != 4 {
            allow_reentry_and_pop(sys, f, b, me as u32);
        }
    }
}

/// C: unlabeled routine at `0x021DD2A4`: `021dd044` when var 2 is the holder and a move slot
/// is out of PP.
pub fn h_021dd2a4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && first_empty_slot(b, me as u32) != 4 {
        allow_reentry_and_pop(sys, f, b, me as u32);
    }
}

/// C: unlabeled routine at `0x021DD2D0`: when var 2 is the holder and its last move is out of
/// PP, sets var 0x51 to 1.
pub fn h_021dd2d0<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && last_move_empty_slot(b, me as u32) != 4 {
        b.rewrite_var(0x51, 1);
    }
}

/// C: unlabeled routine at `0x021DD2F8`: restores PP (`021dd39c`, `any_slot` = 0) when var 2
/// is the holder.
pub fn h_021dd2f8<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        restore_pp(sys, f, b, me as u32, false);
    }
}

/// C: unlabeled routine at `0x021DD320`: restores PP (`021dd39c`, `any_slot` = 1) when work
/// word 6 is non-zero.
pub fn h_021dd320<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if sys.factor_mut(f).work[6] != 0 {
        restore_pp(sys, f, b, me as u32, true);
    }
}

/// C: unlabeled routine at `0x021DD480`: when the holder has no sick 0xF (`021d62c4`),
/// `021dde30` with n 2, gated.
pub fn h_021dd480<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if !b.mon(me as u32).has_sick(0xF) {
        max_hp_check_reentry(sys, f, b, me as u32, 2, true);
    }
}

/// C: unlabeled routine at `0x021DD4B0`: when the holder has no sick 0xF, `021dde30` with n 2,
/// ungated.
pub fn h_021dd4b0<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if !b.mon(me as u32).has_sick(0xF) {
        max_hp_check_reentry(sys, f, b, me as u32, 2, false);
    }
}

/// C: unlabeled routine at `0x021DD4E0`: when the holder has no sick 0xF and `021ddea4` with
/// n 2 holds, `021dd044`.
pub fn h_021dd4e0<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if !b.mon(me as u32).has_sick(0xF) && max_hp_check(b, me as u32, 2) {
        allow_reentry_and_pop(sys, f, b, me as u32);
    }
}

/// C: unlabeled routine at `0x021DD514`: when var 2 is the holder, pushes HANDEX work 5 (+6
/// holder, +4 u16 item param 2, str 0x38C with holder and item ID).
pub fn h_021dd514<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let w = b.host.handex_push(5, me as u32);
        b.host.handex(w).put_u8(6, me);
        let p = item_param(sys, f, b, 2);
        let buf = b.host.handex(w);
        buf.put_u16(4, p as u16);
        str_param_setup(buf, 8, 2, 0x38C);
        str_param_add_arg(buf, 8, me as u32);
        str_param_add_arg(buf, 8, sys.sub_id(f) as u32);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021DD614`: `021dd7b8` with kind 0.
pub fn h_021dd614<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    fraction_work_and_sick(sys, f, b, me, 0);
}

/// C: unlabeled routine at `0x021DD624`: `021dd7b8` with kind 0 when `work[6]` is non-zero.
pub fn h_021dd624<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if sys.factor_mut(f).work[6] != 0 {
        fraction_work_and_sick(sys, f, b, me, 0);
    }
}

/// C: unlabeled routine at `0x021DD648`: `021dd7b8` with kind 1.
pub fn h_021dd648<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    fraction_work_and_sick(sys, f, b, me, 1);
}

/// C: unlabeled routine at `0x021DD658`: `021dd7b8` with kind 1 when `work[6]` is non-zero.
pub fn h_021dd658<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if sys.factor_mut(f).work[6] != 0 {
        fraction_work_and_sick(sys, f, b, me, 1);
    }
}

/// C: unlabeled routine at `0x021DD67C`: `021dd7b8` with kind 2.
pub fn h_021dd67c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    fraction_work_and_sick(sys, f, b, me, 2);
}

/// C: unlabeled routine at `0x021DD68C`: `021dd7b8` with kind 2 when `work[6]` is non-zero.
pub fn h_021dd68c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if sys.factor_mut(f).work[6] != 0 {
        fraction_work_and_sick(sys, f, b, me, 2);
    }
}

/// C: unlabeled routine at `0x021DD6B0`: `021dd7b8` with kind 3.
pub fn h_021dd6b0<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    fraction_work_and_sick(sys, f, b, me, 3);
}

/// C: unlabeled routine at `0x021DD6C0`: `021dd7b8` with kind 3 when `work[6]` is non-zero.
pub fn h_021dd6c0<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if sys.factor_mut(f).work[6] != 0 {
        fraction_work_and_sick(sys, f, b, me, 3);
    }
}

/// C: unlabeled routine at `0x021DD6E4`: `021dd7b8` with kind 4.
pub fn h_021dd6e4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    fraction_work_and_sick(sys, f, b, me, 4);
}

/// C: unlabeled routine at `0x021DD6F4`: `021dd7b8` with kind 4 when `work[6]` is non-zero.
pub fn h_021dd6f4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if sys.factor_mut(f).work[6] != 0 {
        fraction_work_and_sick(sys, f, b, me, 4);
    }
}

/// C: unlabeled routine at `0x021DD70C`: when `work[0]` is 0 and the holder has no sick 0xF,
/// `021dde30` with n 2, gated.
pub fn h_021dd70c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if sys.factor_mut(f).work[0] == 0 && !b.mon(me as u32).has_sick(0xF) {
        max_hp_check_reentry(sys, f, b, me as u32, 2, true);
    }
}

/// C: unlabeled routine at `0x021DD744`: when `work[0]` is 0 and the holder has no sick 0xF,
/// `021dde30` with n 2, ungated.
pub fn h_021dd744<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if sys.factor_mut(f).work[0] == 0 && !b.mon(me as u32).has_sick(0xF) {
        max_hp_check_reentry(sys, f, b, me as u32, 2, false);
    }
}

/// C: unlabeled routine at `0x021DD77C`: when `work[0]` is 0, the holder has no sick 0xF and
/// `021ddea4` with n 2 holds, `021dd044`.
pub fn h_021dd77c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if sys.factor_mut(f).work[0] == 0
        && !b.mon(me as u32).has_sick(0xF)
        && max_hp_check(b, me as u32, 2)
    {
        allow_reentry_and_pop(sys, f, b, me as u32);
    }
}

/// C: unlabeled routine at `0x021DD87C`: `021dd9e8` with value 1, flag 1.
pub fn h_021dd87c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    push_work_0e(b, me as u32, 1, 1);
}

/// C: unlabeled routine at `0x021DD898`: `021dd9e8` with value 2, flag 1.
pub fn h_021dd898<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    push_work_0e(b, me as u32, 2, 1);
}

/// C: unlabeled routine at `0x021DD8B4`: `021dd9e8` with value 5, flag 1.
pub fn h_021dd8b4<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    push_work_0e(b, me as u32, 5, 1);
}

/// C: unlabeled routine at `0x021DD8D0`: `021dd9e8` with value 3, flag 1.
pub fn h_021dd8d0<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    push_work_0e(b, me as u32, 3, 1);
}

/// C: unlabeled routine at `0x021DD8EC`: `021dd9e8` with value 4, flag 1.
pub fn h_021dd8ec<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    push_work_0e(b, me as u32, 4, 1);
}

/// C: unlabeled routine at `0x021DD908`: when var 2 is the holder and `flag_b(9)` is clear,
/// pushes HANDEX work 0x17 (+4 9, +8 holder) and then work 4 (str 0x3E9 with holder and item
/// ID).
pub fn h_021dd908<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && !b.mon(me as u32).flag_b(9) {
        let w = b.host.handex_push(0x17, me as u32);
        let buf = b.host.handex(w);
        buf.put_u32(4, 9);
        buf.put_u8(8, me);
        b.host.handex_pop(w);
        let w = b.host.handex_push(4, me as u32);
        let buf = b.host.handex(w);
        str_param_setup(buf, 4, 2, 0x3E9);
        str_param_add_arg(buf, 4, me as u32);
        str_param_add_arg(buf, 4, sys.sub_id(f) as u32);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021DDA28`: when var 4 is the holder, var 0x46 is 0 and var 0x38
/// is above 3, and the holder has no sick 0xF, `021dd044`.
pub fn h_021dda28<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if me as u32 == b.var(4) as u32
        && b.var(0x46) == 0
        && b.var(0x38) > 3
        && !b.mon(me as u32).has_sick(0xF)
    {
        allow_reentry_and_pop(sys, f, b, me as u32);
    }
}

/// C: unlabeled routine at `0x021DDA6C`: when var 2 is the holder, pushes HANDEX work 5 (+6
/// holder, +4 u16 = max HP / (item param 2 as a byte), at least 1).
pub fn h_021dda6c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let div = item_param(sys, f, b, 2) as u8 as u32;
        let w = b.host.handex_push(5, me as u32);
        b.host.handex(w).put_u8(6, me);
        let v = b.max_hp_div_min1(me as u32, div);
        b.host.handex(w).put_u16(4, v as u16);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021DDAC4`: `021ddcd8` with kind 9, p 0.
pub fn h_021ddac4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    kind_check_and_mark(sys, f, b, me as u32, 9, 0);
}

/// C: unlabeled routine at `0x021DDAE4`: `021ddcd8` with kind 0xA, p 0.
pub fn h_021ddae4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    kind_check_and_mark(sys, f, b, me as u32, 0xA, 0);
}

/// C: unlabeled routine at `0x021DDB04`: `021ddcd8` with kind 0xC, p 0.
pub fn h_021ddb04<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    kind_check_and_mark(sys, f, b, me as u32, 0xC, 0);
}

/// C: unlabeled routine at `0x021DDB24`: `021ddcd8` with kind 0xB, p 0.
pub fn h_021ddb24<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    kind_check_and_mark(sys, f, b, me as u32, 0xB, 0);
}

/// C: unlabeled routine at `0x021DDB44`: `021ddcd8` with kind 0xE, p 0.
pub fn h_021ddb44<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    kind_check_and_mark(sys, f, b, me as u32, 0xE, 0);
}

/// C: unlabeled routine at `0x021DDB64`: `021ddcd8` with kind 1, p 0.
pub fn h_021ddb64<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    kind_check_and_mark(sys, f, b, me as u32, 1, 0);
}

/// C: unlabeled routine at `0x021DDB84`: `021ddcd8` with kind 3, p 0.
pub fn h_021ddb84<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    kind_check_and_mark(sys, f, b, me as u32, 3, 0);
}

/// C: unlabeled routine at `0x021DDBA4`: `021ddcd8` with kind 4, p 0.
pub fn h_021ddba4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    kind_check_and_mark(sys, f, b, me as u32, 4, 0);
}

/// C: unlabeled routine at `0x021DDBC4`: `021ddcd8` with kind 2, p 0.
pub fn h_021ddbc4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    kind_check_and_mark(sys, f, b, me as u32, 2, 0);
}

/// C: unlabeled routine at `0x021DDBE4`: `021ddcd8` with kind 0xD, p 0.
pub fn h_021ddbe4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    kind_check_and_mark(sys, f, b, me as u32, 0xD, 0);
}

/// C: unlabeled routine at `0x021DDC04`: `021ddcd8` with kind 6, p 0.
pub fn h_021ddc04<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    kind_check_and_mark(sys, f, b, me as u32, 6, 0);
}

/// C: unlabeled routine at `0x021DDC24`: `021ddcd8` with kind 5, p 0.
pub fn h_021ddc24<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    kind_check_and_mark(sys, f, b, me as u32, 5, 0);
}

/// C: unlabeled routine at `0x021DDC44`: `021ddcd8` with kind 7, p 0.
pub fn h_021ddc44<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    kind_check_and_mark(sys, f, b, me as u32, 7, 0);
}

/// C: unlabeled routine at `0x021DDC64`: `021ddcd8` with kind 0xF, p 0.
pub fn h_021ddc64<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    kind_check_and_mark(sys, f, b, me as u32, 0xF, 0);
}

/// C: unlabeled routine at `0x021DDC84`: `021ddcd8` with kind 0x10, p 0.
pub fn h_021ddc84<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    kind_check_and_mark(sys, f, b, me as u32, 0x10, 0);
}

/// C: unlabeled routine at `0x021DDCA4`: `021ddcd8` with kind 8, p 0.
pub fn h_021ddca4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    kind_check_and_mark(sys, f, b, me as u32, 8, 0);
}

/// C: unlabeled routine at `0x021DDCC4`: `021ddcd8` with kind 0, p 1.
pub fn h_021ddcc4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    kind_check_and_mark(sys, f, b, me as u32, 0, 1);
}

/// C: unlabeled routine at `0x021DDD38`: when `work[0]` is non-zero and the holder is an event
/// target (`021e85f8`), pushes HANDEX work 0x23 (str 0xDB with holder and item ID).
pub fn h_021ddd38<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if sys.factor_mut(f).work[0] != 0 && b.is_event_target(me as u32) {
        let w = b.host.handex_push(0x23, me as u32);
        let buf = b.host.handex(w);
        str_param_setup(buf, 8, 2, 0xDB);
        str_param_add_arg(buf, 8, me as u32);
        str_param_add_arg(buf, 8, sys.sub_id(f) as u32);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021DDD8C`: when var 2 is the holder, `021dde30` with n = item
/// param 2, gated.
pub fn h_021ddd8c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let n = item_param(sys, f, b, 2);
        max_hp_check_reentry(sys, f, b, me as u32, n, true);
    }
}

/// C: unlabeled routine at `0x021DDDC4`: when var 2 is the holder, `021dde30` with n = item
/// param 2, ungated.
pub fn h_021dddc4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let n = item_param(sys, f, b, 2);
        max_hp_check_reentry(sys, f, b, me as u32, n, false);
    }
}

/// C: unlabeled routine at `0x021DDDFC`: `021ddea4` with n = item param 2 holds, `021dd044`.
pub fn h_021dddfc<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    let n = item_param(sys, f, b, 2);
    if max_hp_check(b, me as u32, n) {
        allow_reentry_and_pop(sys, f, b, me as u32);
    }
}

/// C: unlabeled routine at `0x021DDEFC`: when var 2 is the holder and `021ddea4` with n 4
/// holds, sets var 0x11 to 2; if that succeeds, `021dd044`.
pub fn h_021ddefc<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && max_hp_check(b, me as u32, 4) && b.rewrite_var(0x11, 2) {
        allow_reentry_and_pop(sys, f, b, me as u32);
    }
}

/// C: unlabeled routine at `0x021DDF38`: when var 2 is the holder, pushes HANDEX work 4 (str
/// 0x401 with holder and item ID) and sets `work[0]` to 0.
pub fn h_021ddf38<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let w = b.host.handex_push(4, me as u32);
        let buf = b.host.handex(w);
        str_param_setup(buf, 4, 2, 0x401);
        str_param_add_arg(buf, 4, me as u32);
        str_param_add_arg(buf, 4, sys.sub_id(f) as u32);
        b.host.handex_pop(w);
        sys.factor_mut(f).work[0] = 0;
    }
}

/// 0x021DF39C: item 228, event 11: when var 2 (masked to a byte) is on the holder's side, sets
/// var 0x51 to 1 (C: `FUN_021e8878`).
pub fn h_021df39c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if same_side(b.var(2) as u32 & 0xFF, me as u32) {
        b.rewrite_var(0x51, 1);
    }
}

/// 0x021DF48C: item 282, event 0x7C: `FUN_021df4e0` with kind 3 (the same helper serves
/// 0x021DF4A4, 0x021DF4BC and 0x021DF4D4 for items 283-285).
pub fn h_021df48c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    rewrite_24_from_item_param(sys, f, b, me, 3);
}

/// C: `FUN_overlay_d_93__021dcfd0(flow, percent)`: true when `BattleCalc::roll_percent(percent)`
/// holds, else `FUN_overlay_d_93__021c82b8(flow, 2)`.
fn roll_percent_or_forced_2<H: FlowHost>(b: &mut Battle<H>, percent: u32) -> bool {
    if b.calc_mut().roll_percent(percent) {
        return true;
    }
    b.host.fun_021c82b8(2)
}

/// C: unlabeled routine at `0x021DE108(factor, flow, poke_id, work, kind)`: when var 4 is the
/// holder and var 0x46 is 0, `move_fun_0689c8cc(var 0x12)` is false and var 0x1A equals `kind`:
/// allows re-entry, sets `work[0]` to var 3 + 1, and pushes HANDEX work 0 with `+4 |= 2` and pops it.
fn mark_turn_work<H: FlowHost>(
    sys: &mut EventSystem,
    f: FactorId,
    b: &mut Battle<H>,
    me: u8,
    kind: i32,
) {
    if me as i32 != b.var(4) || b.var(0x46) != 0 {
        return;
    }
    let mv = b.var(0x12) as u32 & 0xFFFF;
    if b.host.move_fun_0689c8cc(mv) {
        return;
    }
    if b.var(0x1A) != kind {
        return;
    }
    sys.allow_reentry(f);
    sys.factor_mut(f).work[0] = (b.var(3) as u32).wrapping_add(1);
    let w = b.host.handex_push(0, me as u32);
    let buf = b.host.handex(w);
    let v = buf.get_u32(4) | 2;
    buf.put_u32(4, v);
    b.host.handex_pop(w);
}

/// C: unlabeled routine at `0x021DD588`: when var 2 is the holder, pushes HANDEX work 5 (+6
/// holder, +4 u16 = BattleMon get 0xE times item param 2 (byte) / 100, str 0x38C with holder and
/// item ID). (Meaning of 0xE: UNVERIFIED.)
pub fn h_021dd588<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let n = b.mon_get(me as u32, 0xE);
        let p = item_param(sys, f, b, 2) & 0xFF;
        let w = b.host.handex_push(5, me as u32);
        let buf = b.host.handex(w);
        buf.put_u8(6, me);
        buf.put_u16(4, crate::calc::u32_div(p.wrapping_mul(n), 100) as u16);
        str_param_setup(buf, 8, 2, 0x38C);
        str_param_add_arg(buf, 8, me as u32);
        str_param_add_arg(buf, 8, sys.sub_id(f) as u32);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021DDF98`: when var 2 is the holder and `021dde30` with n = item
/// param 2 (byte) holds (gated), `021dd044`.
pub fn h_021ddf98<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let n = item_param(sys, f, b, 2) & 0xFF;
        max_hp_check_reentry(sys, f, b, me as u32, n, true);
    }
}

/// C: unlabeled routine at `0x021DDFDC`: when var 2 is the holder, `021de024`.
pub fn h_021ddfdc<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        h_021de024(sys, f, b, me);
    }
}

/// C: unlabeled routine at `0x021DE000`: when var 0xC is not 6, `021ddfdc`.
pub fn h_021de000<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if b.var(0xC) != 6 {
        h_021ddfdc(sys, f, b, me);
    }
}

/// C: unlabeled routine at `0x021DE024`: `021ddea4` with n = item param 2 (byte) holds, `021dd044`.
pub fn h_021de024<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    let n = item_param(sys, f, b, 2) & 0xFF;
    if max_hp_check(b, me as u32, n) {
        allow_reentry_and_pop(sys, f, b, me as u32);
    }
}

/// C: unlabeled routine at `0x021DE058`: when var 2 is the holder, pushes HANDEX work 0xC (+0xF
/// holder, +4 = 0x22, +8 `permanent_param(0x78)`, str 0x404 with holder and item ID).
pub fn h_021de058<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let w = b.host.handex_push(0xC, me as u32);
        let buf = b.host.handex(w);
        buf.put_u8(0xF, me);
        buf.put_u32(4, 0x22);
        buf.put_u32(8, SickCont::permanent_param(0x78).0);
        str_param_setup(buf, 0x14, 2, 0x404);
        str_param_add_arg(buf, 0x14, me as u32);
        str_param_add_arg(buf, 0x14, sys.sub_id(f) as u32);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021DE0C0`: when `work[6]` is non-zero, `021de058`.
pub fn h_021de0c0<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if sys.factor_mut(f).work[6] != 0 {
        h_021de058(sys, f, b, me);
    }
}

/// C: unlabeled routine at `0x021DE0DC`: `021de108` with kind 1.
pub fn h_021de0dc<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mark_turn_work(sys, f, b, me, 1);
}

/// C: unlabeled routine at `0x021DE0F8`: `021de108` with kind 2.
pub fn h_021de0f8<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mark_turn_work(sys, f, b, me, 2);
}

/// C: unlabeled routine at `0x021DE180`: when var 2 is the holder and `work[0]` is non-zero,
/// with slot = `work[0] - 1` (byte) not fainted: pushes HANDEX work 7 (+6 slot, +4 u16 =
/// max HP of the slot / item param 2 (byte), at least 1; str 0x407 with slot, holder and item ID).
pub fn h_021de180<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) || sys.factor_mut(f).work[0] == 0 {
        return;
    }
    let slot = sys.factor_mut(f).work[0].wrapping_sub(1) & 0xFF;
    if b.mon_is_fainted(slot) {
        return;
    }
    let p = item_param(sys, f, b, 2) & 0xFF;
    let w = b.host.handex_push(7, me as u32);
    b.host.handex(w).put_u8(6, slot as u8);
    let hp = b.max_hp_div_min1(slot, p);
    let buf = b.host.handex(w);
    buf.put_u16(4, hp as u16);
    str_param_setup(buf, 0xC, 2, 0x407);
    let s = buf[6] as u32;
    str_param_add_arg(buf, 0xC, s);
    str_param_add_arg(buf, 0xC, me as u32);
    str_param_add_arg(buf, 0xC, sys.sub_id(f) as u32);
    b.host.handex_pop(w);
}

/// C: unlabeled routine at `0x021DE228`: when `021d5d68` (BattleMon has a negative stage) holds
/// for the holder, `021dd044`.
pub fn h_021de228<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if b.mon(me as u32).has_negative_stage() {
        allow_reentry_and_pop(sys, f, b, me as u32);
    }
}

/// C: unlabeled routine at `0x021DE24C`: when var 0xC is not 6, `021de228`.
pub fn h_021de24c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if b.var(0xC) != 6 {
        h_021de228(sys, f, b, me);
    }
}

/// C: unlabeled routine at `0x021DE270`: when var 2 is the holder and the holder has a negative
/// stage, `021dd044`.
pub fn h_021de270<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && b.mon(me as u32).has_negative_stage() {
        allow_reentry_and_pop(sys, f, b, me as u32);
    }
}

/// C: unlabeled routine at `0x021DE2A0`: when var 2 is the holder, pushes HANDEX work 0x12 (+4
/// holder) and pops it, then pushes work 4 (str 0x3F2 with holder and item ID) and pops it.
pub fn h_021de2a0<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let w = b.host.handex_push(0x12, me as u32);
        b.host.handex(w).put_u8(4, me);
        b.host.handex_pop(w);
        let w = b.host.handex_push(4, me as u32);
        let buf = b.host.handex(w);
        str_param_setup(buf, 4, 2, 0x3F2);
        str_param_add_arg(buf, 4, me as u32);
        str_param_add_arg(buf, 4, sys.sub_id(f) as u32);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021DE300`: when `work[6]` is non-zero, `021de2a0`.
pub fn h_021de300<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if sys.factor_mut(f).work[6] != 0 {
        h_021de2a0(sys, f, b, me);
    }
}

/// C: unlabeled routine at `0x021DE31C`: when var 2 is the holder and the holder has a condition
/// from the 6-entry table (`021d7e10`), `021dd044`.
pub fn h_021de31c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && first_table_sick(b, me as u32) != 0 {
        allow_reentry_and_pop(sys, f, b, me as u32);
    }
}

/// C: unlabeled routine at `0x021DE34C`: `021dd19c` with kind 0x26.
pub fn h_021de34c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    push_cure_work(b, me, 0x26);
}

/// C: unlabeled routine at `0x021DE358`: when `work[6]` is non-zero, `021de34c`.
pub fn h_021de358<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if sys.factor_mut(f).work[6] != 0 {
        h_021de34c(sys, f, b, me);
    }
}

/// C: unlabeled routine at `0x021DE3D4`: when var 2 is the holder, `EventVar_Mul(0x35, 0x800)`.
pub fn h_021de3d4<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        b.mul_var(0x35, 0x800);
    }
}

/// C: unlabeled routine at `0x021DE3FC`: when var 2 is the holder and `roll_percent(item param 2
/// (byte))` or `021c82b8(flow, 2)` holds (`021dcfd0`), and `021d76bc(0x11, 2)` succeeds, `021dd044`.
pub fn h_021de3fc<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let p = item_param(sys, f, b, 2) & 0xFF;
        if roll_percent_or_forced_2(b, p) && b.rewrite_var(0x11, 2) {
            allow_reentry_and_pop(sys, f, b, me as u32);
        }
    }
}

/// C: unlabeled routine at `0x021DE440`: when var 2 is the holder, pushes HANDEX work 4 (str
/// 0x401 with holder and item ID) and sets `work[0]` to 0.
pub fn h_021de440<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let w = b.host.handex_push(4, me as u32);
        let buf = b.host.handex(w);
        str_param_setup(buf, 4, 2, 0x401);
        str_param_add_arg(buf, 4, me as u32);
        str_param_add_arg(buf, 4, sys.sub_id(f) as u32);
        b.host.handex_pop(w);
        sys.factor_mut(f).work[0] = 0;
    }
}

/// C: unlabeled routine at `0x021DE4A0`: when var 2 is the holder, `EventVar_Rewrite(0x11, 0)`.
pub fn h_021de4a0<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        b.rewrite_var(0x11, 0);
    }
}

/// C: unlabeled routine at `0x021DE4C4`: when var 3 is the holder and var 0x25 is 0,
/// `EventVar_Rewrite(0x26, item param 2 (byte))`.
pub fn h_021de4c4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.var(0x25) == 0 {
        let p = item_param(sys, f, b, 2) & 0xFF;
        b.rewrite_var(0x26, p as i32);
    }
}

/// C: unlabeled routine at `0x021DE4F4`: when var 2 is the holder, pushes HANDEX work 0x2B (+4
/// holder, +5 = 100) and pops it.
pub fn h_021de4f4<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let w = b.host.handex_push(0x2B, me as u32);
        let buf = b.host.handex(w);
        buf.put_u8(4, me);
        buf.put_u8(5, 100);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021DE564`: when var 3 is the holder, var 0x2C = (var 0x2C as byte
/// + 1) as byte.
pub fn h_021de564<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let v = ((b.var(0x2C) as u32 & 0xFF) + 1) & 0xFF;
        b.rewrite_var(0x2C, v as i32);
    }
}

/// C: unlabeled routine at `0x021DE598`: when var 3 is the holder and its species is 0x71,
/// var 0x2C = (var 0x2C as byte + 2) as byte.
pub fn h_021de598<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.mon(me as u32).species() as u32 == 0x71 {
        let v = ((b.var(0x2C) as u32 & 0xFF) + 2) & 0xFF;
        b.rewrite_var(0x2C, v as i32);
    }
}

/// C: unlabeled routine at `0x021DE5DC`: when var 3 is the holder and its species is 0x53,
/// var 0x2C = (var 0x2C as byte + 2) as byte.
pub fn h_021de5dc<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.mon(me as u32).species() as u32 == 0x53 {
        let v = ((b.var(0x2C) as u32 & 0xFF) + 2) & 0xFF;
        b.rewrite_var(0x2C, v as i32);
    }
}

/// C: unlabeled routine at `0x021DE74C`: when var 3 is the holder, its species is 0x16E and
/// `move_category(var 0x12)` is 2, `EventVar_Mul(0x35, 0x2000)`.
pub fn h_021de74c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.mon(me as u32).species() as u32 == 0x16E {
        let mv = b.var(0x12) as u32 & 0xFFFF;
        if b.host.move_category(mv) == 2 {
            b.mul_var(0x35, 0x2000);
        }
    }
}

/// C: unlabeled routine at `0x021DE79C`: when var 4 is the holder, its species is 0x16E and
/// var 0x1A is 2, `EventVar_Mul(0x35, 0x2000)`.
pub fn h_021de79c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me) && b.mon(me as u32).species() as u32 == 0x16E && b.var(0x1A) == 2 {
        b.mul_var(0x35, 0x2000);
    }
}

/// C: unlabeled routine at `0x021DE7E4`: when var 4 is the holder, its species is 0x84, it is not
/// transformed (`021d6d54`) and var 0x1A is 1, `EventVar_Mul(0x35, 0x2000)`.
pub fn h_021de7e4<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me)
        && b.mon(me as u32).species() as u32 == 0x84
        && !b.mon(me as u32).is_transformed()
        && b.var(0x1A) == 1
    {
        b.mul_var(0x35, 0x2000);
    }
}

/// C: unlabeled routine at `0x021DE834`: when var 2 is the holder, its species is 0x84 and it is
/// not transformed (`021d6d54`), `EventVar_Mul(0x35, 0x2000)`.
pub fn h_021de834<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me)
        && b.mon(me as u32).species() as u32 == 0x84
        && !b.mon(me as u32).is_transformed()
    {
        b.mul_var(0x35, 0x2000);
    }
}

/// C: unlabeled routine at `0x021DE878`: when var 3 is the holder, its species is 0x17C or 0x17D
/// and var 0x1A is 2, `EventVar_Mul(0x35, 0x1800)`.
pub fn h_021de878<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me)
        && (b.mon(me as u32).species() as u32).wrapping_sub(0x17C) < 2
        && b.var(0x1A) == 2
    {
        b.mul_var(0x35, 0x1800);
    }
}

/// C: unlabeled routine at `0x021DE8B4`: when var 4 is the holder, its species is 0x17C or 0x17D
/// and var 0x1A is 2, `EventVar_Mul(0x35, 0x1800)`.
pub fn h_021de8b4<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me)
        && (b.mon(me as u32).species() as u32).wrapping_sub(0x17C) < 2
        && b.var(0x1A) == 2
    {
        b.mul_var(0x35, 0x1800);
    }
}

/// C: unlabeled routine at `0x021DE8FC`: when var 3 is the holder, its species is 0x68 or 0x69
/// and `move_category(var 0x12)` is 1, `EventVar_Mul(0x35, 0x2000)`.
pub fn h_021de8fc<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && (b.mon(me as u32).species() as u32).wrapping_sub(0x68) < 2 {
        let mv = b.var(0x12) as u32 & 0xFFFF;
        if b.host.move_category(mv) == 1 {
            b.mul_var(0x35, 0x2000);
        }
    }
}

/// C: unlabeled routine at `0x021DE948`: when var 3 is the holder and `move_category(var 0x12)`
/// is 1, `EventVar_Mul(0x35, 0x1800)`.
pub fn h_021de948<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let mv = b.var(0x12) as u32 & 0xFFFF;
        if b.host.move_category(mv) == 1 {
            b.mul_var(0x35, 0x1800);
        }
    }
}

/// C: unlabeled routine at `0x021DE980`: when var 3 is the holder and `move_category(var 0x12)`
/// is 2, `EventVar_Mul(0x35, 0x1800)`.
pub fn h_021de980<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let mv = b.var(0x12) as u32 & 0xFFFF;
        if b.host.move_category(mv) == 2 {
            b.mul_var(0x35, 0x1800);
        }
    }
}

/// C: unlabeled routine at `0x021DE9B8`: when var 2 is the holder, `EventVar_Mul(0x35, 0x1800)`.
pub fn h_021de9b8<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        b.mul_var(0x35, 0x1800);
    }
}

/// C: unlabeled routine at `0x021DE9D4`: when var 3 is the holder and var 0x12 (masked) is not
/// 0xA5, pushes HANDEX work 0xC (+0xF holder, +4 = 0x1B, +8 `permanent_param(var 0x12)`, +0x10 = 2).
pub fn h_021de9d4<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let mv = b.var(0x12) as u32 & 0xFFFF;
        if mv != 0xA5 {
            let w = b.host.handex_push(0xC, me as u32);
            let buf = b.host.handex(w);
            buf.put_u8(0xF, me);
            buf.put_u32(4, 0x1B);
            buf.put_u32(8, SickCont::permanent_param(mv).0);
            buf.put_u8(0x10, 2);
            b.host.handex_pop(w);
        }
    }
}

/// C: unlabeled routine at `0x021DEA1C`: when var 2 is the holder, pushes HANDEX work 0xB (+4 =
/// 0x1B, +8 holder, +0x14 = 1, +0x15 = 1) and pops it.
pub fn h_021dea1c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let w = b.host.handex_push(0xB, me as u32);
        let buf = b.host.handex(w);
        buf.put_u32(4, 0x1B);
        buf.put_u8(8, me);
        buf.put_u8(0x14, 1);
        buf.put_u8(0x15, 1);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021DEA58`: when var 4 is the holder: if the holder is at full HP,
/// `EventVar_Rewrite(0x3A, 3)` and `work[0]` becomes its result; else `work[0]` is 0.
pub fn h_021dea58<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me) {
        if b.mon_is_full_hp(me as u32) {
            let ok = b.rewrite_var(0x3A, 3);
            sys.factor_mut(f).work[0] = ok as u32;
            return;
        }
        sys.factor_mut(f).work[0] = 0;
    }
}

/// C: unlabeled routine at `0x021DEA8C`: when var 2 is the holder and `work[0]` is non-zero,
/// pushes HANDEX work 0x23 (str 0x3EF with holder and item ID) and pops it; `work[0]` becomes 0.
pub fn h_021dea8c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && sys.factor_mut(f).work[0] != 0 {
        let w = b.host.handex_push(0x23, me as u32);
        let buf = b.host.handex(w);
        str_param_setup(buf, 8, 2, 0x3EF);
        str_param_add_arg(buf, 8, me as u32);
        str_param_add_arg(buf, 8, sys.sub_id(f) as u32);
        b.host.handex_pop(w);
        sys.factor_mut(f).work[0] = 0;
    }
}

/// C: unlabeled routine at `0x021DEAF4`: when var 4 is the holder: if `roll_percent(item param 2
/// (byte))` or `021c82b8(flow, 2)` holds (`021dcfd0`), `EventVar_Rewrite(0x3A, 3)` and `work[0]`
/// becomes its result; else `work[0]` is 0.
pub fn h_021deaf4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me) {
        let p = item_param(sys, f, b, 2) & 0xFF;
        if roll_percent_or_forced_2(b, p) {
            let ok = b.rewrite_var(0x3A, 3);
            sys.factor_mut(f).work[0] = ok as u32;
            return;
        }
        sys.factor_mut(f).work[0] = 0;
    }
}

/// C: unlabeled routine at `0x021DEB30`: when var 2 is the holder and `work[0]` is non-zero,
/// `021dd044` and `work[0]` becomes 0.
pub fn h_021deb30<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && sys.factor_mut(f).work[0] != 0 {
        allow_reentry_and_pop(sys, f, b, me as u32);
        sys.factor_mut(f).work[0] = 0;
    }
}

/// C: unlabeled routine at `0x021DEB5C`: when var 2 is the holder, pushes HANDEX work 4 (str
/// 0x3EF with holder and item ID) and pops it.
pub fn h_021deb5c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let w = b.host.handex_push(4, me as u32);
        let buf = b.host.handex(w);
        str_param_setup(buf, 4, 2, 0x3EF);
        str_param_add_arg(buf, 4, me as u32);
        str_param_add_arg(buf, 4, sys.sub_id(f) as u32);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021DEBEC`: when var 3 is the holder, pushes HANDEX work 7 (+6
/// holder, +4 u16 = max HP of the holder / 10, at least 1; str 0x3F5 with holder) and pops it.
pub fn h_021debec<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let w = b.host.handex_push(7, me as u32);
        b.host.handex(w).put_u8(6, me);
        let hp = b.max_hp_div_min1(me as u32, 10);
        let buf = b.host.handex(w);
        buf.put_u16(4, hp as u16);
        str_param_setup(buf, 0xC, 2, 0x3F5);
        str_param_add_arg(buf, 0xC, me as u32);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021DED28`: when var 3 is the holder and var 0x1D is 8,
/// `SickCont::set_turns(var 0x1E, 8)` written back to var 0x1E.
pub fn h_021ded28<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.var(0x1D) == 8 {
        let mut sc = SickCont(b.var(0x1E) as u32);
        sc.set_turns(8);
        b.rewrite_var(0x1E, sc.0 as i32);
    }
}

/// C: `FUN_021e889c(factor, flow, poke_id)`: when var 2 is on the holder's side, sets var 0x51 to
/// 1 and returns true.
fn ally_sets_51<H: FlowHost>(b: &mut Battle<H>, me: u8) -> bool {
    let who = b.var(2) as u8 as u32;
    same_side(who, me as u32) && b.rewrite_var(0x51, 1)
}

/// C: shape shared by `0x021DF548`, `0x021DF6AC` and `0x021DF7F4`: pushes HANDEX work 0xC for
/// Pokemon `target` (+0xF holder, +4 = `kind`, +8 = `make_default_sick_cont(kind)`) and pops it.
fn push_default_sick_work<H: FlowHost>(b: &mut Battle<H>, me: u8, target: u32, kind: u32) {
    let w = b.host.handex_push(0xC, target);
    b.host.handex(w).put_u8(0xF, me);
    b.host.handex(w).put_u32(4, kind);
    let c = b.calc_mut().make_default_sick_cont(kind).0;
    b.host.handex(w).put_u32(8, c);
    b.host.handex_pop(w);
}

/// C: unlabeled routine at `0x021DED6C`: when var 3 is the holder, var 0x4D is 0 and var 0x32 is
/// non-zero, pushes HANDEX work 5 (+6 holder, +4 u16 = var 0x32 / item param 2, or 1 if that is
/// 0; str 0x392 with the holder and item ID).
pub fn h_021ded6c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.var(0x4D) == 0 && b.var(0x32) != 0 {
        let p = item_param(sys, f, b, 2);
        let mut v = crate::calc::u32_div(b.var(0x32) as u32, p);
        if v == 0 {
            v = 1;
        }
        let item = sys.sub_id(f) as u32;
        let w = b.host.handex_push(5, me as u32);
        let buf = b.host.handex(w);
        buf.put_u8(6, me);
        buf.put_u16(4, v as u16);
        str_param_setup(buf, 8, 2, 0x392);
        str_param_add_arg(buf, 8, me as u32);
        str_param_add_arg(buf, 8, item);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021DEDF8`: when var 2 is the holder and var 0x53 is below 2, var
/// 0x1E gets `SickCont::add_turns` with item param 2 (as a byte).
pub fn h_021dedf8<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && (b.var(0x53) as u32) < 2 {
        let mut sc = SickCont(b.var(0x1E) as u32);
        let n = item_param(sys, f, b, 2) as u8 as u32;
        sc.add_turns(n);
        b.rewrite_var(0x1E, sc.0 as i32);
    }
}

/// C: unlabeled routine at `0x021DEE48`: when var 3 is the holder, var 0x12 is not 0x1FB and
/// var 0x51 is rewritten to 1, sets `work[0]` to 1.
pub fn h_021dee48<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.var(0x12) != 0x1FB && b.rewrite_var(0x51, 1) {
        sys.factor_mut(f).work[0] = 1;
    }
}

/// C: unlabeled routine at `0x021DEE7C`: when var 3 is the holder and `work[0]` is non-zero,
/// `021dd044`.
pub fn h_021dee7c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && sys.factor_mut(f).work[0] != 0 {
        allow_reentry_and_pop(sys, f, b, me as u32);
    }
}

/// C: unlabeled routine at `0x021DEEA4`: when var 2 is the holder, pushes HANDEX work 4 (str
/// 0x3FE with the holder and item ID) and pops it.
pub fn h_021deea4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let item = sys.sub_id(f) as u32;
        let w = b.host.handex_push(4, me as u32);
        let buf = b.host.handex(w);
        str_param_setup(buf, 4, 2, 0x3FE);
        str_param_add_arg(buf, 4, me as u32);
        str_param_add_arg(buf, 4, item);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021DEEFC`: when var 2 is the holder and the holder is not at full
/// HP, pushes HANDEX work 0 (+4 bit 0 set) and `021d70c8` (`allow_reentry`).
pub fn h_021deefc<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        b.mon(me as u32);
        if !b.mon_is_full_hp(me as u32) {
            let w = b.host.handex_push(0, me as u32);
            let buf = b.host.handex(w);
            let v = buf.get_u32(4);
            buf.put_u32(4, v & 0xFFFF_FFFE | 1);
            sys.allow_reentry(f);
            b.host.handex_pop(w);
        }
    }
}

/// C: unlabeled routine at `0x021DEF48`: when var 2 is the holder and the holder is not at full
/// HP, pushes HANDEX work 5 (+6 holder, +4 u16 = max HP / 0x10, at least 1; str 0x392 with the
/// holder and item ID) and pops it.
pub fn h_021def48<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        b.mon(me as u32);
        if !b.mon_is_full_hp(me as u32) {
            let w = b.host.handex_push(5, me as u32);
            b.host.handex(w).put_u8(6, me);
            let div = b.max_hp_div_min1(me as u32, 0x10);
            let item = sys.sub_id(f) as u32;
            let buf = b.host.handex(w);
            buf.put_u16(4, div as u16);
            str_param_setup(buf, 8, 2, 0x392);
            str_param_add_arg(buf, 8, me as u32);
            str_param_add_arg(buf, 8, item);
            b.host.handex_pop(w);
        }
    }
}

/// C: unlabeled routine at `0x021DEFC4`: when var 2 is the holder and `021c83d4` is false: if the
/// holder's type pair has type 3, pushes HANDEX work 5 (+6 holder, +4 u16 = max HP / item param 2,
/// at least 1; str 0x392 with the holder and item ID). Otherwise pushes work 7 (+6 holder, +4 u16
/// = max HP / 8, at least 1, +7 |= 2, +8 = 0x25F, +10 = `poke_pos_in_battle`, +0xB = 6; str 0x40E
/// with the holder and item ID).
pub fn h_021defc4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) || b.host.fun_021c83d4() {
        return;
    }
    b.mon(me as u32);
    let pair = b.mon(me as u32).type_pair();
    if pair_has_type(pair, 3) {
        let w = b.host.handex_push(5, me as u32);
        b.host.handex(w).put_u8(6, me);
        let p = item_param(sys, f, b, 2);
        let div = b.max_hp_div_min1(me as u32, p);
        let item = sys.sub_id(f) as u32;
        let buf = b.host.handex(w);
        buf.put_u16(4, div as u16);
        str_param_setup(buf, 8, 2, 0x392);
        str_param_add_arg(buf, 8, me as u32);
        str_param_add_arg(buf, 8, item);
        b.host.handex_pop(w);
        return;
    }
    let w = b.host.handex_push(7, me as u32);
    b.host.handex(w).put_u8(6, me);
    let div = b.max_hp_div_min1(me as u32, 8);
    let pos = b.host.poke_pos_in_battle(me as u32);
    let item = sys.sub_id(f) as u32;
    let buf = b.host.handex(w);
    buf.put_u16(4, div as u16);
    buf[7] |= 2;
    buf.put_u16(8, 0x25F);
    buf.put_u8(10, pos as u8);
    buf.put_u8(0xB, 6);
    str_param_setup(buf, 0xC, 2, 0x40E);
    str_param_add_arg(buf, 0xC, me as u32);
    str_param_add_arg(buf, 0xC, item);
    b.host.handex_pop(w);
}

/// C: unlabeled routine at `0x021DF0C4`: when var 4 is the holder, var 0x1D is 7 and var 3 (as a
/// byte) is not 0x1F, pushes HANDEX work 0xC (+0xF = var 3 as a byte, +4 = 7, +8 =
/// `SickCont::poke(holder)`, +0xD = 1; str 0x14A with var 3 as a byte and the item ID) and pops it.
pub fn h_021df0c4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me) && b.var(0x1D) == 7 {
        let v3 = b.var(3) as u32;
        if v3 & 0xFF != 0x1F {
            let w = b.host.handex_push(0xC, me as u32);
            b.host.handex(w).put_u8(0xF, v3 as u8);
            b.host.handex(w).put_u32(4, 7);
            let poke = SickCont::poke(me as u32).0;
            let item = sys.sub_id(f) as u32;
            let buf = b.host.handex(w);
            buf.put_u32(8, poke);
            buf.put_u8(0xD, 1);
            str_param_setup(buf, 0x14, 2, 0x14A);
            str_param_add_arg(buf, 0x14, v3 & 0xFF);
            str_param_add_arg(buf, 0x14, item);
            b.host.handex_pop(w);
        }
    }
}

/// C: unlabeled routine at `0x021DF150`: when var 4 is the holder, var 0x46 is 0, `move_flag`
/// of var 0x12 (as a u16) is true and the held item of Pokemon (var 3 as a byte) is 0, pushes
/// HANDEX work 0x24 (+4 = var 3 as a byte) and pops it.
pub fn h_021df150<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me) && b.var(0x46) == 0 {
        let mv = b.var(0x12) as u16 as u32;
        if b.host.move_flag(mv, 0) {
            let v3 = b.var(3) as u32;
            if b.mon(v3 & 0xFF).held_item() == 0 {
                let w = b.host.handex_push(0x24, me as u32);
                b.host.handex(w).put_u8(4, v3 as u8);
                b.host.handex_pop(w);
            }
        }
    }
}

/// C: unlabeled routine at `0x021DF1B0`: when var 2 is the holder, pushes HANDEX work 7 (+6
/// holder, +4 u16 = max HP / item param 2, +7 |= 2, +8 = 0x25F, +10 = `poke_pos_in_battle`,
/// +0xB = 6; str 0x40E with the holder and item ID) and pops it.
pub fn h_021df1b0<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        b.mon(me as u32);
        let w = b.host.handex_push(7, me as u32);
        b.host.handex(w).put_u8(6, me);
        let p = item_param(sys, f, b, 2);
        let div = b.max_hp_div_min1(me as u32, p);
        let pos = b.host.poke_pos_in_battle(me as u32);
        let item = sys.sub_id(f) as u32;
        let buf = b.host.handex(w);
        buf.put_u16(4, div as u16);
        buf[7] |= 2;
        buf.put_u16(8, 0x25F);
        buf.put_u8(10, pos as u8);
        buf.put_u8(0xB, 6);
        str_param_setup(buf, 0xC, 2, 0x40E);
        str_param_add_arg(buf, 0xC, me as u32);
        str_param_add_arg(buf, 0xC, item);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021DF288`: when var 2 is the holder, `EventVar_Mul(0x35, 0x800)`.
pub fn h_021df288<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        b.mul_var(0x35, 0x800);
    }
}

/// C: unlabeled routine at `0x021DF3A4`: when `ally_sets_51` (`021e889c`) is true, pushes HANDEX
/// work 4 (str 0x3EC with the holder and item ID; extra value 0x56A) and pops it.
pub fn h_021df3a4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if ally_sets_51(b, me) {
        let w = b.host.handex_push(4, me as u32);
        let item = sys.sub_id(f) as u32;
        let buf = b.host.handex(w);
        str_param_setup(buf, 4, 2, 0x3EC);
        str_param_add_arg(buf, 4, me as u32);
        str_param_add_arg(buf, 4, item);
        str_param_set_extra(buf, 4, 0x56A);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021DF404`: when var 2 is the holder, `021c8408(holder)`.
pub fn h_021df404<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        b.host.fun_021c8408(me as u32);
    }
}

/// C: unlabeled routine at `0x021DF420`: tail call to `021c8408(wk, poke_id)` (the literal at
/// `0x021DF428` is `0x021C8409`).
pub fn h_021df420<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    b.host.fun_021c8408(me as u32);
}

/// C: unlabeled routine at `0x021DF438`: when var 3 is the holder and its species is 0x1E7, and
/// var 0x16 (as a byte) is 0xF or 7, `EventVar_Mul(0x31, 0x1333)`.
pub fn h_021df438<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.mon(me as u32).species() as u32 == 0x1E7 {
        let v = b.var(0x16) as u32 & 0xFF;
        if v == 0xF || v == 7 {
            b.mul_var(0x31, 0x1333);
        }
    }
}

/// C: unlabeled routine at `0x021DF4A4`: `021df4e0` with kind 4.
pub fn h_021df4a4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    rewrite_24_from_item_param(sys, f, b, me, 4);
}

/// C: unlabeled routine at `0x021DF4BC`: `021df4e0` with kind 1.
pub fn h_021df4bc<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    rewrite_24_from_item_param(sys, f, b, me, 1);
}

/// C: unlabeled routine at `0x021DF4D4`: `021df4e0` with kind 2.
pub fn h_021df4d4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    rewrite_24_from_item_param(sys, f, b, me, 2);
}

/// C: unlabeled routine at `0x021DF51C`: when var 3 is the holder and its species is 0x19,
/// `EventVar_Mul(0x35, 0x2000)`.
pub fn h_021df51c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.mon(me as u32).species() as u32 == 0x19 {
        b.mul_var(0x35, 0x2000);
    }
}

/// C: unlabeled routine at `0x021DF548`: when var 2 is the holder, `push_default_sick_work` with
/// target var 3 (as a byte) and kind 1.
pub fn h_021df548<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let target = b.var(3) as u32 & 0xFF;
        push_default_sick_work(b, me, target, 1);
    }
}

/// C: unlabeled routine at `0x021DF594`: when var 2 is the holder, pushes HANDEX work 0xC (+4 = 5,
/// +8 = `SickCont::badly_poisoned`, +0xC = 0, +0xF holder; str 0xF0 with the holder and item ID)
/// and pops it.
pub fn h_021df594<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let w = b.host.handex_push(0xC, me as u32);
        let item = sys.sub_id(f) as u32;
        let buf = b.host.handex(w);
        buf.put_u32(4, 5);
        buf.put_u32(8, SickCont::badly_poisoned().0);
        buf.put_u8(0xC, 0);
        buf.put_u8(0xF, me);
        str_param_setup(buf, 0x14, 2, 0xF0);
        str_param_add_arg(buf, 0x14, me as u32);
        str_param_add_arg(buf, 0x14, item);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021DF5F4`: when var 2 is the holder, pushes HANDEX work 0xC for
/// Pokemon var 3 (as a byte) (+0xF holder, +4 = 5, +8 = `SickCont::badly_poisoned`) and pops it.
pub fn h_021df5f4<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let target = b.var(3) as u32 & 0xFF;
        let w = b.host.handex_push(0xC, target);
        let buf = b.host.handex(w);
        buf.put_u8(0xF, me);
        buf.put_u32(4, 5);
        buf.put_u32(8, SickCont::badly_poisoned().0);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021DF640`: when var 2 is the holder, pushes HANDEX work 0xC (+4 = 4,
/// +8 = `make_default_sick_cont(4)`, +0xC = 0, +0xF holder, +0x1C = item ID; str 0x102 with the
/// holder and item ID) and pops it.
pub fn h_021df640<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let w = b.host.handex_push(0xC, me as u32);
        b.host.handex(w).put_u32(4, 4);
        let c = b.calc_mut().make_default_sick_cont(4).0;
        let item = sys.sub_id(f) as u32;
        let buf = b.host.handex(w);
        buf.put_u32(8, c);
        buf.put_u8(0xC, 0);
        buf.put_u8(0xF, me);
        buf.put_u32(0x1C, item);
        str_param_setup(buf, 0x14, 2, 0x102);
        str_param_add_arg(buf, 0x14, me as u32);
        str_param_add_arg(buf, 0x14, item);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021DF6AC`: when var 2 is the holder, `push_default_sick_work` with
/// target var 3 (as a byte) and kind 4.
pub fn h_021df6ac<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let target = b.var(3) as u32 & 0xFF;
        push_default_sick_work(b, me, target, 4);
    }
}

/// C: unlabeled routine at `0x021DF7F4`: when var 2 is the holder, `push_default_sick_work` with
/// target var 3 (as a byte) and kind 5.
pub fn h_021df7f4<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let target = b.var(3) as u32 & 0xFF;
        push_default_sick_work(b, me, target, 5);
    }
}

/// C: unlabeled routine at `0x021DFA20`: when var 2 is the holder, `EventVar_Mul(0x35, 0x800)`.
pub fn h_021dfa20<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        b.mul_var(0x35, 0x800);
    }
}

/// C: unlabeled routine at `0x021DFA3C`: when var 2 is the holder, `021d76bc(0x41, 1)`.
pub fn h_021dfa3c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        b.rewrite_var(0x41, 1);
    }
}

/// C: unlabeled routine at `0x021DFA60`: when var 2 is the holder, `EventVar_Mul(0x35, 0x800)`.
pub fn h_021dfa60<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        b.mul_var(0x35, 0x800);
    }
}

/// C: unlabeled routine at `0x021DFA88`: when var 4 is the holder and `021c818c(holder)` is true,
/// `EventVar_Mul(0x35, 0x1800)`.
pub fn h_021dfa88<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me) && b.host.fun_021c818c(me as u32) {
        b.mul_var(0x35, 0x1800);
    }
}

/// C: unlabeled routine at `0x021DFABC`: when var 4 is the holder, var 0x46 is 0 and `move_flag`
/// of var 0x12 (as a u16) is true, pushes HANDEX work 7 for Pokemon var 3 (as a byte) (+6 = that
/// Pokemon, +4 u16 = its max HP / item param 2 as a byte, at least 1; str 0x1A8 with that
/// Pokemon) and pops it.
pub fn h_021dfabc<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me) && b.var(0x46) == 0 {
        let mv = b.var(0x12) as u16 as u32;
        if b.host.move_flag(mv, 0) {
            let w = b.host.handex_push(7, me as u32);
            let target = b.var(3) as u8;
            b.host.handex(w).put_u8(6, target);
            b.mon(target as u32);
            let p = item_param(sys, f, b, 2) as u8 as u32;
            let div = b.max_hp_div_min1(target as u32, p);
            let buf = b.host.handex(w);
            buf.put_u16(4, div as u16);
            str_param_setup(buf, 0xC, 2, 0x1A8);
            str_param_add_arg(buf, 0xC, target as u32);
            b.host.handex_pop(w);
        }
    }
}

/// C: unlabeled routine at `0x021DFB4C`: when var 2 is the holder and `field_effect_active(2)` is
/// false, pushes HANDEX work 4 (str 0x198 with the holder) and pops it.
pub fn h_021dfb4c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && !b.host.field_effect_active(2) {
        let w = b.host.handex_push(4, me as u32);
        let buf = b.host.handex(w);
        str_param_setup(buf, 4, 2, 0x198);
        str_param_add_arg(buf, 4, me as u32);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021DFB94`: when var 2 is the holder, `021d76bc(0x51, 1)`.
pub fn h_021dfb94<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        b.rewrite_var(0x51, 1);
    }
}

/// C: unlabeled routine at `0x021DFBAC`: when var 4 is the holder, pushes HANDEX work 0x20 (+6
/// holder, +4 u16 = 0; str 0x19B with the holder) and pops it.
pub fn h_021dfbac<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me) {
        let w = b.host.handex_push(0x20, me as u32);
        let buf = b.host.handex(w);
        buf.put_u8(6, me);
        buf.put_u16(4, 0);
        str_param_setup(buf, 0xC, 2, 0x19B);
        str_param_add_arg(buf, 0xC, me as u32);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021DFC00`: when the holder is an event target (`021e85f8`) and
/// `poke_pos(var 3 as a byte)` is not 6, var 0x4D is 0, `021c7f24(var 3 as a byte)` is true and
/// `021c83d4` is false: pushes HANDEX work 0x23 (str 0x1A1 with the holder and var 3 as a byte)
/// and pops it, then work 0x2E (+6 = var 3 as a byte, +7 = byte 7 & 0xF | 0x10, +4 u16 = 0x1A1 +
/// 0xEB) and pops it.
pub fn h_021dfc00<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if !b.is_event_target(me as u32) {
        return;
    }
    let v3 = b.var(3) as u32;
    if b.host.poke_pos(v3 & 0xFF) != 6
        && b.var(0x4D) == 0
        && b.host.fun_021c7f24(v3 & 0xFF)
        && !b.host.fun_021c83d4()
    {
        let w = b.host.handex_push(0x23, me as u32);
        let buf = b.host.handex(w);
        str_param_setup(buf, 8, 2, 0x1A1);
        str_param_add_arg(buf, 8, me as u32);
        str_param_add_arg(buf, 8, v3 & 0xFF);
        b.host.handex_pop(w);
        let w = b.host.handex_push(0x2E, me as u32);
        let v3 = b.var(3);
        let buf = b.host.handex(w);
        buf.put_u8(6, v3 as u8);
        buf[7] = buf[7] & 0xF | 0x10;
        buf.put_u16(4, (0x1A1u16).wrapping_add(0xEB));
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E0284(factor, flow, poke_id, work, kind)`: when var 3 is the
/// holder, var 0x16 equals `kind`, and `work[0]` is 1 or `021c7ec4` is true: `021d7704(0x31,
/// 0x1800)` (multiplies var 0x31).
fn mul_0x31_on_kind<H: FlowHost>(
    sys: &mut EventSystem,
    f: FactorId,
    b: &mut Battle<H>,
    me: u8,
    kind: u32,
) {
    if is_me(b, 3, me)
        && b.var(0x16) as u32 == kind
        && (sys.factor_mut(f).work[0] == 1 || b.host.fun_021c7ec4())
    {
        b.mul_var(0x31, 0x1800);
    }
}

/// C: unlabeled routine at `0x021E01D4(factor, flow, poke_id, work, kind)`: when var 3 is the
/// holder and var 0x16 equals `kind`, then for the item in var 0x12 (low 16 bits): `0201c314`
/// (trait `move_fun_0201c314`) is true, `021e0274` (trait `move_unredirectable`) is false, the
/// factor is not temporary and `021c7ec4` is false: makes the factor temporary, pushes HANDEX
/// work 0x23 (str 0xB6 with the factor's sub ID and var 0x12) and pops it, then sets `work[0]`
/// to 1. The item ID is passed to both trait methods, which name their argument `mv`.
fn push_work_0x23_on_kind<H: FlowHost>(
    sys: &mut EventSystem,
    f: FactorId,
    b: &mut Battle<H>,
    me: u8,
    kind: u32,
) {
    if !(is_me(b, 3, me) && b.var(0x16) as u32 == kind) {
        return;
    }
    let item = (b.var(0x12) as u32) & 0xFFFF;
    if !b.host.move_fun_0201c314(item)
        || b.host.move_unredirectable(item)
        || sys.is_temporary(f)
        || b.host.fun_021c7ec4()
    {
        return;
    }
    sys.make_temporary(f);
    let sub = sys.sub_id(f) as u32;
    let var12 = b.var(0x12) as u32;
    let w = b.host.handex_push(0x23, me as u32);
    let buf = b.host.handex(w);
    str_param_setup(buf, 8, 1, 0xB6);
    str_param_add_arg(buf, 8, sub);
    str_param_add_arg(buf, 8, var12);
    b.host.handex_pop(w);
    sys.factor_mut(f).work[0] = 1;
}

/// C: unlabeled routine at `0x021DFCB8`: when `021c7ec4` is false and var 4 is the holder,
/// `021d76bc(0x4B, 1)`.
pub fn h_021dfcb8<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    let busy = b.host.fun_021c7ec4();
    if !busy && is_me(b, 4, me) {
        b.rewrite_var(0x4B, 1);
    }
}

/// C: unlabeled routine at `0x021DFCE8`: when var 3 is the holder, var 0x1E goes through
/// `021e8bd0(&v, 1)` (`SickCont::set_flag`) and is written back.
pub fn h_021dfce8<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let mut sc = SickCont(b.var(0x1E) as u32);
        sc.set_flag(1);
        b.rewrite_var(0x1E, sc.0 as i32);
    }
}

/// C: unlabeled routine at `0x021DFD20`: when var 4 is the holder, var 0x46 is 0, var 0x16 is
/// 10 and `can_change_stage(3, 1)` holds for the holder: `allow_reentry_and_pop`.
pub fn h_021dfd20<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me)
        && b.var(0x46) == 0
        && b.var(0x16) == 10
        && b.mon(me as u32).can_change_stage(3, 1)
    {
        allow_reentry_and_pop(sys, f, b, me as u32);
    }
}

/// C: unlabeled routine at `0x021DFD68`: when var 2 is the holder, `021dd9e8(3, 1)`: HANDEX work
/// 0xE with value 3 and flag 1.
pub fn h_021dfd68<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    push_work_0e(b, me as u32, 3, 1);
}

/// C: unlabeled routine at `0x021DFDA4`: when var 4 is the holder, var 0x46 is 0, var 0x16 is
/// 0xC and `can_change_stage(1, 1)` holds for the holder: `allow_reentry_and_pop`.
pub fn h_021dfda4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me)
        && b.var(0x46) == 0
        && b.var(0x16) == 0xC
        && b.mon(me as u32).can_change_stage(1, 1)
    {
        allow_reentry_and_pop(sys, f, b, me as u32);
    }
}

/// C: unlabeled routine at `0x021DFDEC`: when var 2 is the holder, `021dd9e8(1, 1)`: HANDEX
/// work 0xE with value 1 and flag 1.
pub fn h_021dfdec<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    push_work_0e(b, me as u32, 1, 1);
}

/// C: unlabeled routine at `0x021DD98C`: collects the stats 1..=5 (table at 0x021F1364) whose
/// stage can still be raised for the holder into a scratch buffer (`021c8120(flow, 5)` =
/// `flow + 0x1fec`, modelled as a local), then, if any, picks one with `calc.rand` and runs
/// `push_work_0e` with it and flag 2.
pub fn h_021dd98c<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    let mut list = [0u8; 5];
    let mut n = 0usize;
    for stat in 1u8..=5 {
        if b.mon(me as u32).can_change_stage(stat as u32, 1) {
            list[n] = stat;
            n += 1;
        }
    }
    if n != 0 {
        let i = b.calc_mut().rand(n as u32) as usize;
        push_work_0e(b, me as u32, list[i] as u32, 2);
    }
}

/// C: unlabeled routine at `0x021DFE28`: when the holder is an event target, var 0x4D is 0 and
/// `021c8374`, `021c83d4` and `021c830c` are false while `021c7f24(holder)` and the once-per-battle
/// flag test-and-set `021c7f4c` are true: `allow_reentry_and_pop`.
pub fn h_021dfe28<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if b.is_event_target(me as u32)
        && b.var(0x4D) == 0
        && !b.host.fun_021c8374(me as u32)
        && !b.host.fun_021c83d4()
        && !b.host.fun_021c830c()
        && b.host.fun_021c7f24(me as u32)
        && b.host.fun_021c7f4c()
    {
        allow_reentry_and_pop(sys, f, b, me as u32);
    }
}

/// C: unlabeled routine at `0x021DFE88`: when var 2 is the holder and `021c7f24(holder)` is
/// true: pushes HANDEX work 4 (str 0x19E with the holder) and pops it, then work 0x29 (byte
/// +0x4C = holder) and pops it.
pub fn h_021dfe88<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && b.host.fun_021c7f24(me as u32) {
        let w = b.host.handex_push(4, me as u32);
        let buf = b.host.handex(w);
        str_param_setup(buf, 4, 2, 0x19E);
        str_param_add_arg(buf, 4, me as u32);
        b.host.handex_pop(w);
        let w = b.host.handex_push(0x29, me as u32);
        b.host.handex(w).put_u8(0x4C, me);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E02C0`: when var 3 is the holder and `work[0]` is 1, sets
/// `work[0]` to 2.
pub fn h_021e02c0<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && sys.factor_mut(f).work[0] == 1 {
        sys.factor_mut(f).work[0] = 2;
    }
}

/// C: unlabeled routine at `0x021DFEF4`: `021e0284` with kind 9.
pub fn h_021dfef4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_on_kind(sys, f, b, me, 9);
}

/// C: unlabeled routine at `0x021DFF04`: `021e01d4` with kind 9.
pub fn h_021dff04<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    push_work_0x23_on_kind(sys, f, b, me, 9);
}

/// C: unlabeled routine at `0x021DFF20`: `021e0284` with kind 0xA.
pub fn h_021dff20<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_on_kind(sys, f, b, me, 0xA);
}

/// C: unlabeled routine at `0x021DFF30`: `021e01d4` with kind 0xA.
pub fn h_021dff30<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    push_work_0x23_on_kind(sys, f, b, me, 0xA);
}

/// C: unlabeled routine at `0x021DFF4C`: `021e0284` with kind 0xC.
pub fn h_021dff4c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_on_kind(sys, f, b, me, 0xC);
}

/// C: unlabeled routine at `0x021DFF5C`: `021e01d4` with kind 0xC.
pub fn h_021dff5c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    push_work_0x23_on_kind(sys, f, b, me, 0xC);
}

/// C: unlabeled routine at `0x021DFF78`: `021e0284` with kind 0xB.
pub fn h_021dff78<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_on_kind(sys, f, b, me, 0xB);
}

/// C: unlabeled routine at `0x021DFF88`: `021e01d4` with kind 0xB.
pub fn h_021dff88<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    push_work_0x23_on_kind(sys, f, b, me, 0xB);
}

/// C: unlabeled routine at `0x021DFFA4`: `021e0284` with kind 0xE.
pub fn h_021dffa4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_on_kind(sys, f, b, me, 0xE);
}

/// C: unlabeled routine at `0x021DFFB4`: `021e01d4` with kind 0xE.
pub fn h_021dffb4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    push_work_0x23_on_kind(sys, f, b, me, 0xE);
}

/// C: unlabeled routine at `0x021DFFD0`: `021e0284` with kind 1.
pub fn h_021dffd0<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_on_kind(sys, f, b, me, 1);
}

/// C: unlabeled routine at `0x021DFFE0`: `021e01d4` with kind 1.
pub fn h_021dffe0<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    push_work_0x23_on_kind(sys, f, b, me, 1);
}

/// C: unlabeled routine at `0x021DFFFC`: `021e0284` with kind 3.
pub fn h_021dfffc<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_on_kind(sys, f, b, me, 3);
}

/// C: unlabeled routine at `0x021E000C`: `021e01d4` with kind 3.
pub fn h_021e000c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    push_work_0x23_on_kind(sys, f, b, me, 3);
}

/// C: unlabeled routine at `0x021E0028`: `021e0284` with kind 4.
pub fn h_021e0028<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_on_kind(sys, f, b, me, 4);
}

/// C: unlabeled routine at `0x021E0038`: `021e01d4` with kind 4.
pub fn h_021e0038<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    push_work_0x23_on_kind(sys, f, b, me, 4);
}

/// C: unlabeled routine at `0x021E0054`: `021e0284` with kind 2.
pub fn h_021e0054<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_on_kind(sys, f, b, me, 2);
}

/// C: unlabeled routine at `0x021E0064`: `021e01d4` with kind 2.
pub fn h_021e0064<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    push_work_0x23_on_kind(sys, f, b, me, 2);
}

/// C: unlabeled routine at `0x021E0080`: `021e0284` with kind 0xD.
pub fn h_021e0080<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_on_kind(sys, f, b, me, 0xD);
}

/// C: unlabeled routine at `0x021E0090`: `021e01d4` with kind 0xD.
pub fn h_021e0090<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    push_work_0x23_on_kind(sys, f, b, me, 0xD);
}

/// C: unlabeled routine at `0x021E00AC`: `021e0284` with kind 6.
pub fn h_021e00ac<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_on_kind(sys, f, b, me, 6);
}

/// C: unlabeled routine at `0x021E00BC`: `021e01d4` with kind 6.
pub fn h_021e00bc<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    push_work_0x23_on_kind(sys, f, b, me, 6);
}

/// C: unlabeled routine at `0x021E00D8`: `021e0284` with kind 5.
pub fn h_021e00d8<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_on_kind(sys, f, b, me, 5);
}

/// C: unlabeled routine at `0x021E00E8`: `021e01d4` with kind 5.
pub fn h_021e00e8<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    push_work_0x23_on_kind(sys, f, b, me, 5);
}

/// C: unlabeled routine at `0x021E0104`: `021e0284` with kind 7.
pub fn h_021e0104<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_on_kind(sys, f, b, me, 7);
}

/// C: unlabeled routine at `0x021E0114`: `021e01d4` with kind 7.
pub fn h_021e0114<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    push_work_0x23_on_kind(sys, f, b, me, 7);
}

/// C: unlabeled routine at `0x021E0130`: `021e0284` with kind 0xF.
pub fn h_021e0130<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_on_kind(sys, f, b, me, 0xF);
}

/// C: unlabeled routine at `0x021E0140`: `021e01d4` with kind 0xF.
pub fn h_021e0140<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    push_work_0x23_on_kind(sys, f, b, me, 0xF);
}

/// C: unlabeled routine at `0x021E015C`: `021e0284` with kind 0x10.
pub fn h_021e015c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_on_kind(sys, f, b, me, 0x10);
}

/// C: unlabeled routine at `0x021E016C`: `021e01d4` with kind 0x10.
pub fn h_021e016c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    push_work_0x23_on_kind(sys, f, b, me, 0x10);
}

/// C: unlabeled routine at `0x021E0188`: `021e0284` with kind 8.
pub fn h_021e0188<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_on_kind(sys, f, b, me, 8);
}

/// C: unlabeled routine at `0x021E0198`: `021e01d4` with kind 8.
pub fn h_021e0198<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    push_work_0x23_on_kind(sys, f, b, me, 8);
}

/// C: unlabeled routine at `0x021E01B4`: `021e01d4` with kind 0.
pub fn h_021e01b4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    push_work_0x23_on_kind(sys, f, b, me, 0);
}

/// C: unlabeled routine at `0x021E01C4`: `021e0284` with kind 0.
pub fn h_021e01c4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_on_kind(sys, f, b, me, 0);
}

/// C: unlabeled routine at `0x021DE538`: when var 3 is the holder, `EventVar_Mul(0x35,
/// item_percent_fixed)` (`021dd008`).
pub fn h_021de538<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let v = item_percent_fixed(sys, f, b);
        b.mul_var(0x35, v);
    }
}

/// C: unlabeled routine at `0x021DEBB4`: when var 3 is the holder and var 0x38 is above 3,
/// `EventVar_Mul(0x35, item_percent_fixed)` (`021dd008`).
pub fn h_021debb4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.var(0x38) > 3 {
        let v = item_percent_fixed(sys, f, b);
        b.mul_var(0x35, v);
    }
}

/// C: unlabeled routine at `0x021DEC44`: when var 3 is the holder, `EventVar_Mul(0x35,
/// item_percent_fixed)` (`021dd008`).
pub fn h_021dec44<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let v = item_percent_fixed(sys, f, b);
        b.mul_var(0x35, v);
    }
}

/// C: unlabeled routine at `0x021DF370`: when var 3 is the holder, `EventVar_Mul(0x35,
/// item_percent_fixed)` (`021dd008`).
pub fn h_021df370<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let v = item_percent_fixed(sys, f, b);
        b.mul_var(0x35, v);
    }
}

/// C: `FUN_overlay_d_93__021df93c(factor, flow, poke_id, kind)`: when var 3 is the holder and
/// var 0x16 equals `kind`, `EventVar_Mul(0x31, item_percent_fixed)` (`021dd008`).
fn mul_0x31_percent_on_kind<H: FlowHost>(
    sys: &mut EventSystem,
    f: FactorId,
    b: &mut Battle<H>,
    me: u8,
    kind: u32,
) {
    if is_me(b, 3, me) && b.var(0x16) as u32 == kind {
        let v = item_percent_fixed(sys, f, b);
        b.mul_var(0x31, v);
    }
}

/// The arithmetic shared by `021de374`, `021de668` and `021dec70`: the single `fl` is moved 0.5
/// away from zero (`fadd(0.5, fl)` when `positive`, else `fsub(fl, 0.5)`), truncated (`f2i`) and
/// divided by 100 (`_s32_div_f`).
fn round_div_100(fl: u32, positive: bool) -> i32 {
    let r = if positive {
        fadd(0x3F00_0000, fl)
    } else {
        fsub(fl, 0x3F00_0000)
    };
    f2i(r).wrapping_div(100)
}

/// C: unlabeled routine at `0x021DE374`: when var 4 is the holder, `EventVar_Mul(0x35,
/// ((100 - param 2) << 12 as unsigned single, rounded) / 100)`. (Item 213.)
pub fn h_021de374<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me) {
        let x = 100u32.wrapping_sub(item_param(sys, f, b, 2));
        let v = round_div_100(u2f(x.wrapping_shl(12)), x != 0);
        b.mul_var(0x35, v);
    }
}

/// C: unlabeled routine at `0x021DE668`: like `021de374` (var 4) but `100 - param 2` is a 16-bit
/// value converted as a signed single. (Item 255.)
pub fn h_021de668<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me) {
        let x = 100u32.wrapping_sub(item_param(sys, f, b, 2)) & 0xFFFF;
        let v = round_div_100(i2f((x << 12) as i32), x != 0);
        b.mul_var(0x35, v);
    }
}

/// C: unlabeled routine at `0x021DE620`: when var 3 is the holder and the Pokemon with the ID in
/// var 4 has flag-A bit 1 set, `EventVar_Mul(0x35, item_percent_fixed)` (`021dd008`). (Item 276.)
pub fn h_021de620<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let id = b.var(4) as u8 as u32;
        if b.mon(id).flag_a(1) {
            let v = item_percent_fixed(sys, f, b);
            b.mul_var(0x35, v);
        }
    }
}

/// C: unlabeled routine at `0x021DE6CC`: when var 3 is the holder and the category of the move
/// in var 0x12 (`0201C1E8`) is 1 (physical), `EventVar_Mul(0x31, item_percent_fixed)`. (Item 266.)
pub fn h_021de6cc<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_on_category(sys, f, b, me, 1);
}

/// C: unlabeled routine at `0x021DE70C`: like `021de6cc` with category 2 (special). (Item 267.)
pub fn h_021de70c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_on_category(sys, f, b, me, 2);
}

/// C: shape shared by `0x021DE6CC` and `0x021DE70C`.
fn mul_0x31_on_category<H: FlowHost>(
    sys: &mut EventSystem,
    f: FactorId,
    b: &mut Battle<H>,
    me: u8,
    category: u32,
) {
    if is_me(b, 3, me) {
        let mv = b.var(0x12) as u16 as u32;
        if b.host.move_category(mv) == category {
            let v = item_percent_fixed(sys, f, b);
            b.mul_var(0x31, v);
        }
    }
}

/// C: unlabeled routine at `0x021DEC70`: when var 3 is the holder, `move_fun_0689c8cc(var 0x12)`
/// (via the ARM thunk `021ded0c`) is false, the holder's consecutive-use count is at least 1 and
/// var 0x12 is its last move, `EventVar_Mul(0x35, (min(count * param 2 + 100, 200) << 12 as
/// single, rounded) / 100)`. (Item 277.)
pub fn h_021dec70<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let mv = b.var(0x12) as u16 as u32;
        if b.host.move_fun_0689c8cc(mv) {
            return;
        }
        let streak = b.mon(me as u32).move_streak() as u32;
        if streak < 1 {
            return;
        }
        let mv = b.var(0x12) as u16;
        if mv != b.mon(me as u32).last_move() {
            return;
        }
        let p = item_param(sys, f, b, 2);
        let mut x = streak.wrapping_mul(p).wrapping_add(100) & 0xFFFF;
        if x > 200 {
            x = 200;
        }
        let v = round_div_100(i2f((x << 12) as i32), x != 0);
        b.mul_var(0x35, v);
    }
}

/// C: unlabeled routine at `0x021DF974`: when var 3 is the holder, its species is 0x1E4 and var
/// 0x16 (low byte) is 0xF or 0xA, `EventVar_Mul(0x31, item_percent_fixed)`. (Item 136.)
pub fn h_021df974<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_on_species_kind(sys, f, b, me, 0x1E4, 0xA);
}

/// C: unlabeled routine at `0x021DF9C8`: like `021df974` with species 0x1E3 and kind 8 (or 0xF).
/// (Item 135.)
pub fn h_021df9c8<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_on_species_kind(sys, f, b, me, 0x1E3, 8);
}

/// C: shape shared by `0x021DF974` and `0x021DF9C8`.
fn mul_0x31_on_species_kind<H: FlowHost>(
    sys: &mut EventSystem,
    f: FactorId,
    b: &mut Battle<H>,
    me: u8,
    species: u32,
    kind: u32,
) {
    if is_me(b, 3, me) && b.mon(me as u32).species() as u32 == species {
        let k = b.var(0x16) as u32 & 0xFF;
        if k == 0xF || k == kind {
            let v = item_percent_fixed(sys, f, b);
            b.mul_var(0x31, v);
        }
    }
}

/// C: unlabeled routine at `0x021DF6F8`: `021df93c` with kind 6. (Items 222,308.)
pub fn h_021df6f8<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_percent_on_kind(sys, f, b, me, 6);
}

/// C: unlabeled routine at `0x021DF710`: `021df93c` with kind 4. (Items 237,305.)
pub fn h_021df710<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_percent_on_kind(sys, f, b, me, 4);
}

/// C: unlabeled routine at `0x021DF728`: `021df93c` with kind 5. (Items 238,309.)
pub fn h_021df728<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_percent_on_kind(sys, f, b, me, 5);
}

/// C: unlabeled routine at `0x021DF740`: `021df93c` with kind 11. (Items 239,301.)
pub fn h_021df740<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_percent_on_kind(sys, f, b, me, 11);
}

/// C: unlabeled routine at `0x021DF758`: `021df93c` with kind 16. (Items 240,312.)
pub fn h_021df758<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_percent_on_kind(sys, f, b, me, 16);
}

/// C: unlabeled routine at `0x021DF770`: `021df93c` with kind 1. (Items 241,303.)
pub fn h_021df770<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_percent_on_kind(sys, f, b, me, 1);
}

/// C: unlabeled routine at `0x021DF788`: `021df93c` with kind 12. (Items 242,300.)
pub fn h_021df788<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_percent_on_kind(sys, f, b, me, 12);
}

/// C: unlabeled routine at `0x021DF7A0`: `021df93c` with kind 8. (Items 233,313.)
pub fn h_021df7a0<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_percent_on_kind(sys, f, b, me, 8);
}

/// C: unlabeled routine at `0x021DF7B8`: `021df93c` with kind 10. (Items 243,299.)
pub fn h_021df7b8<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_percent_on_kind(sys, f, b, me, 10);
}

/// C: unlabeled routine at `0x021DF7D0`: `021df93c` with kind 2. (Items 244,306.)
pub fn h_021df7d0<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_percent_on_kind(sys, f, b, me, 2);
}

/// C: unlabeled routine at `0x021DF7E8`: `021df93c` with kind 3. (Items 245,304.)
pub fn h_021df7e8<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_percent_on_kind(sys, f, b, me, 3);
}

/// C: unlabeled routine at `0x021DF840`: `021df93c` with kind 14. (Items 246,302.)
pub fn h_021df840<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_percent_on_kind(sys, f, b, me, 14);
}

/// C: unlabeled routine at `0x021DF858`: `021df93c` with kind 7. (Items 247,310.)
pub fn h_021df858<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_percent_on_kind(sys, f, b, me, 7);
}

/// C: unlabeled routine at `0x021DF870`: `021df93c` with kind 13. (Items 248,307.)
pub fn h_021df870<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_percent_on_kind(sys, f, b, me, 13);
}

/// C: unlabeled routine at `0x021DF888`: `021df93c` with kind 9. (Items 249,298.)
pub fn h_021df888<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_percent_on_kind(sys, f, b, me, 9);
}

/// C: unlabeled routine at `0x021DF8A0`: `021df93c` with kind 15. (Items 250,311.)
pub fn h_021df8a0<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_percent_on_kind(sys, f, b, me, 15);
}

/// C: unlabeled routine at `0x021DF8B8`: `021df93c` with kind 0. (Items 251.)
pub fn h_021df8b8<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_percent_on_kind(sys, f, b, me, 0);
}

/// C: unlabeled routine at `0x021DF8D0`: `021df93c` with kind 13. (Items 314.)
pub fn h_021df8d0<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_percent_on_kind(sys, f, b, me, 13);
}

/// C: unlabeled routine at `0x021DF8E8`: `021df93c` with kind 5. (Items 315.)
pub fn h_021df8e8<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_percent_on_kind(sys, f, b, me, 5);
}

/// C: unlabeled routine at `0x021DF900`: `021df93c` with kind 10. (Items 317.)
pub fn h_021df900<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_percent_on_kind(sys, f, b, me, 10);
}

/// C: unlabeled routine at `0x021DF918`: `021df93c` with kind 10. (Items 254.)
pub fn h_021df918<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_percent_on_kind(sys, f, b, me, 10);
}

/// C: unlabeled routine at `0x021DF930`: `021df93c` with kind 11. (Items 318.)
pub fn h_021df930<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    mul_0x31_percent_on_kind(sys, f, b, me, 11);
}

/// Ported item handlers (regenerated by tools/sync-handlers.py).
pub const PORTED: &[u32] = &[
    0x021DD05C, 0x021DD068, 0x021DD080, 0x021DD08C, 0x021DD0A4, 0x021DD0B0, 0x021DD0C8, 0x021DD0D4,
    0x021DD0EC, 0x021DD0F8, 0x021DD110, 0x021DD11C, 0x021DD134, 0x021DD140, 0x021DD240, 0x021DD26C,
    0x021DD2A4, 0x021DD2D0, 0x021DD2F8, 0x021DD320, 0x021DD480, 0x021DD4B0, 0x021DD4E0, 0x021DD514,
    0x021DD588, 0x021DD614, 0x021DD624, 0x021DD648, 0x021DD658, 0x021DD67C, 0x021DD68C, 0x021DD6B0,
    0x021DD6C0, 0x021DD6E4, 0x021DD6F4, 0x021DD70C, 0x021DD744, 0x021DD77C, 0x021DD87C, 0x021DD898,
    0x021DD8B4, 0x021DD8D0, 0x021DD8EC, 0x021DD908, 0x021DD98C, 0x021DDA28, 0x021DDA6C, 0x021DDAC4,
    0x021DDAE4, 0x021DDB04, 0x021DDB24, 0x021DDB44, 0x021DDB64, 0x021DDB84, 0x021DDBA4, 0x021DDBC4,
    0x021DDBE4, 0x021DDC04, 0x021DDC24, 0x021DDC44, 0x021DDC64, 0x021DDC84, 0x021DDCA4, 0x021DDCC4,
    0x021DDD38, 0x021DDD8C, 0x021DDDC4, 0x021DDDFC, 0x021DDEFC, 0x021DDF38, 0x021DDF98, 0x021DDFDC,
    0x021DE000, 0x021DE024, 0x021DE058, 0x021DE0C0, 0x021DE0DC, 0x021DE0F8, 0x021DE180, 0x021DE228,
    0x021DE24C, 0x021DE270, 0x021DE2A0, 0x021DE300, 0x021DE31C, 0x021DE34C, 0x021DE358, 0x021DE374,
    0x021DE3D4, 0x021DE3FC, 0x021DE440, 0x021DE4A0, 0x021DE4C4, 0x021DE4F4, 0x021DE538, 0x021DE564,
    0x021DE598, 0x021DE5DC, 0x021DE620, 0x021DE668, 0x021DE6CC, 0x021DE70C, 0x021DE74C, 0x021DE79C,
    0x021DE7E4, 0x021DE834, 0x021DE878, 0x021DE8B4, 0x021DE8FC, 0x021DE948, 0x021DE980, 0x021DE9B8,
    0x021DE9D4, 0x021DEA1C, 0x021DEA58, 0x021DEA8C, 0x021DEAF4, 0x021DEB30, 0x021DEB5C, 0x021DEBB4,
    0x021DEBEC, 0x021DEC44, 0x021DEC70, 0x021DED28, 0x021DED6C, 0x021DEDF8, 0x021DEE48, 0x021DEE7C,
    0x021DEEA4, 0x021DEEFC, 0x021DEF48, 0x021DEFC4, 0x021DF0C4, 0x021DF150, 0x021DF1B0, 0x021DF288,
    0x021DF370, 0x021DF39C, 0x021DF3A4, 0x021DF404, 0x021DF420, 0x021DF438, 0x021DF48C, 0x021DF4A4,
    0x021DF4BC, 0x021DF4D4, 0x021DF51C, 0x021DF548, 0x021DF594, 0x021DF5F4, 0x021DF640, 0x021DF6AC,
    0x021DF6F8, 0x021DF710, 0x021DF728, 0x021DF740, 0x021DF758, 0x021DF770, 0x021DF788, 0x021DF7A0,
    0x021DF7B8, 0x021DF7D0, 0x021DF7E8, 0x021DF7F4, 0x021DF840, 0x021DF858, 0x021DF870, 0x021DF888,
    0x021DF8A0, 0x021DF8B8, 0x021DF8D0, 0x021DF8E8, 0x021DF900, 0x021DF918, 0x021DF930, 0x021DF974,
    0x021DF9C8, 0x021DFA20, 0x021DFA3C, 0x021DFA60, 0x021DFA88, 0x021DFABC, 0x021DFB4C, 0x021DFB94,
    0x021DFBAC, 0x021DFC00, 0x021DFCB8, 0x021DFCE8, 0x021DFD20, 0x021DFD68, 0x021DFDA4, 0x021DFDEC,
    0x021DFE28, 0x021DFE88, 0x021DFEF4, 0x021DFF04, 0x021DFF20, 0x021DFF30, 0x021DFF4C, 0x021DFF5C,
    0x021DFF78, 0x021DFF88, 0x021DFFA4, 0x021DFFB4, 0x021DFFD0, 0x021DFFE0, 0x021DFFFC, 0x021E000C,
    0x021E0028, 0x021E0038, 0x021E0054, 0x021E0064, 0x021E0080, 0x021E0090, 0x021E00AC, 0x021E00BC,
    0x021E00D8, 0x021E00E8, 0x021E0104, 0x021E0114, 0x021E0130, 0x021E0140, 0x021E015C, 0x021E016C,
    0x021E0188, 0x021E0198, 0x021E01B4, 0x021E01C4, 0x021E02C0,
];

/// Ported item handlers by original address.
pub fn handler<H: FlowHost>(addr: u32) -> Option<crate::flow::Handler<H>> {
    Some(match addr & !1 {
        0x021DD05C => h_021dd05c::<H>,
        0x021DD068 => h_021dd068::<H>,
        0x021DD080 => h_021dd080::<H>,
        0x021DD08C => h_021dd08c::<H>,
        0x021DD0A4 => h_021dd0a4::<H>,
        0x021DD0B0 => h_021dd0b0::<H>,
        0x021DD0C8 => h_021dd0c8::<H>,
        0x021DD0D4 => h_021dd0d4::<H>,
        0x021DD0EC => h_021dd0ec::<H>,
        0x021DD0F8 => h_021dd0f8::<H>,
        0x021DD110 => h_021dd110::<H>,
        0x021DD11C => h_021dd11c::<H>,
        0x021DD134 => h_021dd134::<H>,
        0x021DD140 => h_021dd140::<H>,
        0x021DD240 => h_021dd240::<H>,
        0x021DD26C => h_021dd26c::<H>,
        0x021DD2A4 => h_021dd2a4::<H>,
        0x021DD2D0 => h_021dd2d0::<H>,
        0x021DD2F8 => h_021dd2f8::<H>,
        0x021DD320 => h_021dd320::<H>,
        0x021DD480 => h_021dd480::<H>,
        0x021DD4B0 => h_021dd4b0::<H>,
        0x021DD4E0 => h_021dd4e0::<H>,
        0x021DD514 => h_021dd514::<H>,
        0x021DD588 => h_021dd588::<H>,
        0x021DD614 => h_021dd614::<H>,
        0x021DD624 => h_021dd624::<H>,
        0x021DD648 => h_021dd648::<H>,
        0x021DD658 => h_021dd658::<H>,
        0x021DD67C => h_021dd67c::<H>,
        0x021DD68C => h_021dd68c::<H>,
        0x021DD6B0 => h_021dd6b0::<H>,
        0x021DD6C0 => h_021dd6c0::<H>,
        0x021DD6E4 => h_021dd6e4::<H>,
        0x021DD6F4 => h_021dd6f4::<H>,
        0x021DD70C => h_021dd70c::<H>,
        0x021DD744 => h_021dd744::<H>,
        0x021DD77C => h_021dd77c::<H>,
        0x021DD87C => h_021dd87c::<H>,
        0x021DD898 => h_021dd898::<H>,
        0x021DD8B4 => h_021dd8b4::<H>,
        0x021DD8D0 => h_021dd8d0::<H>,
        0x021DD8EC => h_021dd8ec::<H>,
        0x021DD908 => h_021dd908::<H>,
        0x021DD98C => h_021dd98c::<H>,
        0x021DDA28 => h_021dda28::<H>,
        0x021DDA6C => h_021dda6c::<H>,
        0x021DDAC4 => h_021ddac4::<H>,
        0x021DDAE4 => h_021ddae4::<H>,
        0x021DDB04 => h_021ddb04::<H>,
        0x021DDB24 => h_021ddb24::<H>,
        0x021DDB44 => h_021ddb44::<H>,
        0x021DDB64 => h_021ddb64::<H>,
        0x021DDB84 => h_021ddb84::<H>,
        0x021DDBA4 => h_021ddba4::<H>,
        0x021DDBC4 => h_021ddbc4::<H>,
        0x021DDBE4 => h_021ddbe4::<H>,
        0x021DDC04 => h_021ddc04::<H>,
        0x021DDC24 => h_021ddc24::<H>,
        0x021DDC44 => h_021ddc44::<H>,
        0x021DDC64 => h_021ddc64::<H>,
        0x021DDC84 => h_021ddc84::<H>,
        0x021DDCA4 => h_021ddca4::<H>,
        0x021DDCC4 => h_021ddcc4::<H>,
        0x021DDD38 => h_021ddd38::<H>,
        0x021DDD8C => h_021ddd8c::<H>,
        0x021DDDC4 => h_021dddc4::<H>,
        0x021DDDFC => h_021dddfc::<H>,
        0x021DDEFC => h_021ddefc::<H>,
        0x021DDF38 => h_021ddf38::<H>,
        0x021DDF98 => h_021ddf98::<H>,
        0x021DDFDC => h_021ddfdc::<H>,
        0x021DE000 => h_021de000::<H>,
        0x021DE024 => h_021de024::<H>,
        0x021DE058 => h_021de058::<H>,
        0x021DE0C0 => h_021de0c0::<H>,
        0x021DE0DC => h_021de0dc::<H>,
        0x021DE0F8 => h_021de0f8::<H>,
        0x021DE180 => h_021de180::<H>,
        0x021DE228 => h_021de228::<H>,
        0x021DE24C => h_021de24c::<H>,
        0x021DE270 => h_021de270::<H>,
        0x021DE2A0 => h_021de2a0::<H>,
        0x021DE300 => h_021de300::<H>,
        0x021DE31C => h_021de31c::<H>,
        0x021DE34C => h_021de34c::<H>,
        0x021DE358 => h_021de358::<H>,
        0x021DE374 => h_021de374::<H>,
        0x021DE3D4 => h_021de3d4::<H>,
        0x021DE3FC => h_021de3fc::<H>,
        0x021DE440 => h_021de440::<H>,
        0x021DE4A0 => h_021de4a0::<H>,
        0x021DE4C4 => h_021de4c4::<H>,
        0x021DE4F4 => h_021de4f4::<H>,
        0x021DE538 => h_021de538::<H>,
        0x021DE564 => h_021de564::<H>,
        0x021DE598 => h_021de598::<H>,
        0x021DE5DC => h_021de5dc::<H>,
        0x021DE620 => h_021de620::<H>,
        0x021DE668 => h_021de668::<H>,
        0x021DE6CC => h_021de6cc::<H>,
        0x021DE70C => h_021de70c::<H>,
        0x021DE74C => h_021de74c::<H>,
        0x021DE79C => h_021de79c::<H>,
        0x021DE7E4 => h_021de7e4::<H>,
        0x021DE834 => h_021de834::<H>,
        0x021DE878 => h_021de878::<H>,
        0x021DE8B4 => h_021de8b4::<H>,
        0x021DE8FC => h_021de8fc::<H>,
        0x021DE948 => h_021de948::<H>,
        0x021DE980 => h_021de980::<H>,
        0x021DE9B8 => h_021de9b8::<H>,
        0x021DE9D4 => h_021de9d4::<H>,
        0x021DEA1C => h_021dea1c::<H>,
        0x021DEA58 => h_021dea58::<H>,
        0x021DEA8C => h_021dea8c::<H>,
        0x021DEAF4 => h_021deaf4::<H>,
        0x021DEB30 => h_021deb30::<H>,
        0x021DEB5C => h_021deb5c::<H>,
        0x021DEBB4 => h_021debb4::<H>,
        0x021DEBEC => h_021debec::<H>,
        0x021DEC44 => h_021dec44::<H>,
        0x021DEC70 => h_021dec70::<H>,
        0x021DED28 => h_021ded28::<H>,
        0x021DED6C => h_021ded6c::<H>,
        0x021DEDF8 => h_021dedf8::<H>,
        0x021DEE48 => h_021dee48::<H>,
        0x021DEE7C => h_021dee7c::<H>,
        0x021DEEA4 => h_021deea4::<H>,
        0x021DEEFC => h_021deefc::<H>,
        0x021DEF48 => h_021def48::<H>,
        0x021DEFC4 => h_021defc4::<H>,
        0x021DF0C4 => h_021df0c4::<H>,
        0x021DF150 => h_021df150::<H>,
        0x021DF1B0 => h_021df1b0::<H>,
        0x021DF288 => h_021df288::<H>,
        0x021DF370 => h_021df370::<H>,
        0x021DF39C => h_021df39c::<H>,
        0x021DF3A4 => h_021df3a4::<H>,
        0x021DF404 => h_021df404::<H>,
        0x021DF420 => h_021df420::<H>,
        0x021DF438 => h_021df438::<H>,
        0x021DF48C => h_021df48c::<H>,
        0x021DF4A4 => h_021df4a4::<H>,
        0x021DF4BC => h_021df4bc::<H>,
        0x021DF4D4 => h_021df4d4::<H>,
        0x021DF51C => h_021df51c::<H>,
        0x021DF548 => h_021df548::<H>,
        0x021DF594 => h_021df594::<H>,
        0x021DF5F4 => h_021df5f4::<H>,
        0x021DF640 => h_021df640::<H>,
        0x021DF6AC => h_021df6ac::<H>,
        0x021DF6F8 => h_021df6f8::<H>,
        0x021DF710 => h_021df710::<H>,
        0x021DF728 => h_021df728::<H>,
        0x021DF740 => h_021df740::<H>,
        0x021DF758 => h_021df758::<H>,
        0x021DF770 => h_021df770::<H>,
        0x021DF788 => h_021df788::<H>,
        0x021DF7A0 => h_021df7a0::<H>,
        0x021DF7B8 => h_021df7b8::<H>,
        0x021DF7D0 => h_021df7d0::<H>,
        0x021DF7E8 => h_021df7e8::<H>,
        0x021DF7F4 => h_021df7f4::<H>,
        0x021DF840 => h_021df840::<H>,
        0x021DF858 => h_021df858::<H>,
        0x021DF870 => h_021df870::<H>,
        0x021DF888 => h_021df888::<H>,
        0x021DF8A0 => h_021df8a0::<H>,
        0x021DF8B8 => h_021df8b8::<H>,
        0x021DF8D0 => h_021df8d0::<H>,
        0x021DF8E8 => h_021df8e8::<H>,
        0x021DF900 => h_021df900::<H>,
        0x021DF918 => h_021df918::<H>,
        0x021DF930 => h_021df930::<H>,
        0x021DF974 => h_021df974::<H>,
        0x021DF9C8 => h_021df9c8::<H>,
        0x021DFA20 => h_021dfa20::<H>,
        0x021DFA3C => h_021dfa3c::<H>,
        0x021DFA60 => h_021dfa60::<H>,
        0x021DFA88 => h_021dfa88::<H>,
        0x021DFABC => h_021dfabc::<H>,
        0x021DFB4C => h_021dfb4c::<H>,
        0x021DFB94 => h_021dfb94::<H>,
        0x021DFBAC => h_021dfbac::<H>,
        0x021DFC00 => h_021dfc00::<H>,
        0x021DFCB8 => h_021dfcb8::<H>,
        0x021DFCE8 => h_021dfce8::<H>,
        0x021DFD20 => h_021dfd20::<H>,
        0x021DFD68 => h_021dfd68::<H>,
        0x021DFDA4 => h_021dfda4::<H>,
        0x021DFDEC => h_021dfdec::<H>,
        0x021DFE28 => h_021dfe28::<H>,
        0x021DFE88 => h_021dfe88::<H>,
        0x021DFEF4 => h_021dfef4::<H>,
        0x021DFF04 => h_021dff04::<H>,
        0x021DFF20 => h_021dff20::<H>,
        0x021DFF30 => h_021dff30::<H>,
        0x021DFF4C => h_021dff4c::<H>,
        0x021DFF5C => h_021dff5c::<H>,
        0x021DFF78 => h_021dff78::<H>,
        0x021DFF88 => h_021dff88::<H>,
        0x021DFFA4 => h_021dffa4::<H>,
        0x021DFFB4 => h_021dffb4::<H>,
        0x021DFFD0 => h_021dffd0::<H>,
        0x021DFFE0 => h_021dffe0::<H>,
        0x021DFFFC => h_021dfffc::<H>,
        0x021E000C => h_021e000c::<H>,
        0x021E0028 => h_021e0028::<H>,
        0x021E0038 => h_021e0038::<H>,
        0x021E0054 => h_021e0054::<H>,
        0x021E0064 => h_021e0064::<H>,
        0x021E0080 => h_021e0080::<H>,
        0x021E0090 => h_021e0090::<H>,
        0x021E00AC => h_021e00ac::<H>,
        0x021E00BC => h_021e00bc::<H>,
        0x021E00D8 => h_021e00d8::<H>,
        0x021E00E8 => h_021e00e8::<H>,
        0x021E0104 => h_021e0104::<H>,
        0x021E0114 => h_021e0114::<H>,
        0x021E0130 => h_021e0130::<H>,
        0x021E0140 => h_021e0140::<H>,
        0x021E015C => h_021e015c::<H>,
        0x021E016C => h_021e016c::<H>,
        0x021E0188 => h_021e0188::<H>,
        0x021E0198 => h_021e0198::<H>,
        0x021E01B4 => h_021e01b4::<H>,
        0x021E01C4 => h_021e01c4::<H>,
        0x021E02C0 => h_021e02c0::<H>,
        _ => return None,
    })
}

#[cfg(test)]
mod percent_tests {
    use super::percent_fixed;

    /// Host-float reference for ordinary parameters (the ROM check only reaches 0..=255).
    #[test]
    fn percent_fixed_matches_host_float() {
        for p in -400..400i32 {
            let v = p + 100;
            let x = (v << 12) as f32;
            let r = if v > 0 { 0.5 + x } else { x - 0.5 };
            assert_eq!(percent_fixed(p), (r as i32) / 100, "param {p}");
        }
        assert_eq!(percent_fixed(20), 4915);
        assert_eq!(percent_fixed(0), 4096);
    }
}

#[cfg(test)]
mod tests;
