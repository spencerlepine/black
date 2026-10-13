//! Move handlers: registry at 0x021F3030 (258 move IDs -> getter -> table in the pools
//! 0x021F240C-0x021F3030), code 0x021E02DC-0x021E8940.
//!
//! Each handler is C `void handler(factor, flow_work, poke_id, work)`; the factor is a type-0
//! factor whose sub ID (`EventSystem::sub_id`) is the move ID. The doc comment gives the
//! original address, the moves using it and the event. Event-variable labels are kept numeric
//! (see `event_var`).

use crate::battle_mon::{value, BattleMon, BppEnv};
use crate::calc::{fx_mul, fx_mul_min1, u32_div};
use crate::event_factor::{EventSystem, FactorId};
use crate::flow::{str_param_add_arg, str_param_setup, Battle, FlowHost, HandexBytes};
use crate::main_util::{opposing_side, other_side, poke_side, same_side};
use crate::poke_type::{make_mono_pair, types};
use crate::sick_cont::SickCont;

fn is_me<H: FlowHost>(b: &Battle<H>, label: u16, me: u8) -> bool {
    me as u32 == b.var(label) as u32
}

/// 0x021E0838: move 280, event 72: when var 3 is the holder, clears the factor's skip check
/// (C: `sub_021d7324`).
pub fn h_021e0838<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        sys.clear_skip_check(f);
    }
}

/// 0x021E25DC: move 253, event 14: when `work[6]` is set, sets var 0x41 to 1.
pub fn h_021e25dc<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, _me: u8) {
    if sys.factor(f).work[6] != 0 {
        b.rewrite_var(0x41, 1);
    }
}

/// C: unlabeled routine at `0x021E03D8(bpp, sub_id)`: `fun_021e03f8` for the Pokemon's ID.
pub fn fun_021e03d8(sys: &mut EventSystem, mon: &BattleMon, sub_id: u32) {
    fun_021e03f8(sys, mon.poke_id() as u32, sub_id);
}

/// C: unlabeled routine at `0x021E03F8(poke_id, sub_id)`: removes every type-0 factor owned by
/// `poke_id` whose sub ID (move ID) equals `sub_id`, restarting the search after each removal.
pub fn fun_021e03f8(sys: &mut EventSystem, poke_id: u32, sub_id: u32) {
    let mut cur = sys.find(0, poke_id);
    while let Some(c) = cur {
        if sub_id == sys.sub_id(c) as u32 {
            sys.remove(c);
            cur = sys.find(0, poke_id);
        } else {
            cur = sys.find_next(c);
        }
    }
}

/// C: unlabeled routine at `0x021E0944(factor, flow, poke_id)`: pushes HANDEX work 7 (+4 u16
/// max HP / 2 of the holder, at least 1; +6 holder; str 0x386 at +0xC with the holder), pops it.
pub fn fun_021e0944<H: FlowHost>(b: &mut Battle<H>, me: u32) {
    let hp = b.max_hp_div_min1(me, 2);
    let w = b.host.handex_push(7, me);
    let buf = b.host.handex(w);
    buf.put_u8(6, me as u8);
    buf.put_u16(4, hp as u16);
    str_param_setup(buf, 0xC, 2, 0x386);
    str_param_add_arg(buf, 0xC, me);
    b.host.handex_pop(w);
}

/// Common prefix the handlers at `0x021E072C` and `0x021E0850` inline for HANDEX work 0x1A:
/// `+7` = `side`, `+4` = 3, `+5` = `+6` = 0 (the callers then set bits in `+5`/`+6`, each gated
/// on `+4` being above 1 or 2, which always holds here).
fn work_1a_prefix(buf: &mut [u8], side: u8) {
    buf[7] = side;
    buf[4] = 3;
    buf[5] = 0;
    buf[6] = 0;
}

/// C: unlabeled routine at `0x021E0468`: move 160, event 160: when var 3 is the holder, collects
/// the types of the holder's moves other than this move that the holder does not have; if
/// there are any, pushes HANDEX work 0x14 (+4 u16 mono type pair of a random one of them, +6
/// holder).
pub fn h_021e0468<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let me = me as u32;
    let n = b.mon(me).move_count();
    let mut list = [0u8; 4];
    let mut count = 0u32;
    let mut i = 0u32;
    if n != 0 {
        loop {
            let mv = b.mon(me).move_id(i & 0xFF);
            if mv != sys.sub_id(f) {
                let ty = b.host.move_type(mv as u32);
                if !b.mon(me).has_type(ty) {
                    list[count as usize] = ty as u8;
                    count = count.wrapping_add(1) & 0xFFFF;
                }
            }
            i = i.wrapping_add(1) & 0xFFFF;
            if i >= n as u32 {
                break;
            }
        }
    }
    if count != 0 {
        let w = b.host.handex_push(0x14, me);
        let idx = b.calc.rand(count);
        let pair = make_mono_pair(list[(idx & 0xFFFF) as usize]);
        let buf = b.host.handex(w);
        buf.put_u16(4, pair);
        buf.put_u8(6, me as u8);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E0518`: move 293, event 160: when var 3 is the holder, picks a
/// type from the ground kind (`fun_021c8174`) and, if the holder's type pair differs from that
/// mono pair, pushes HANDEX work 0x14 (+4 u16 pair, +6 holder). Kinds 0,5 -> 11; 1,2,3,8,9,15
/// -> 4; 6,11,12 -> 10; 7,13 -> 14; 10 -> 5; others -> 0.
pub fn h_021e0518<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let ty: u8 = match b.host.fun_021c8174() {
        0 | 5 => 11,
        1 | 2 | 3 | 8 | 9 | 0xF => 4,
        6 | 0xB | 0xC => 10,
        7 | 0xD => 14,
        10 => 5,
        _ => 0,
    };
    let pair = make_mono_pair(ty);
    if pair != b.mon(me as u32).type_pair() {
        let w = b.host.handex_push(0x14, me as u32);
        let buf = b.host.handex(w);
        buf.put_u16(4, pair);
        buf.put_u8(6, me);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E05B0`: move 433, event 158: when var 3 is the holder: if
/// field effect 1 is not active, pushes HANDEX work 0x1B (+4 = 1, +8 `SickCont::turn(5)`, str
/// 0x359 at +0x10 with the holder); else pushes HANDEX work 0x1C (+4 = 1).
pub fn h_021e05b0<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    if !b.host.field_effect_active(1) {
        let w = b.host.handex_push(0x1B, me as u32);
        let buf = b.host.handex(w);
        buf.put_u32(4, 1);
        buf.put_u32(8, SickCont::turn(5).0);
        str_param_setup(buf, 0x10, 2, 0x359);
        str_param_add_arg(buf, 0x10, me as u32);
        b.host.handex_pop(w);
    } else {
        let w = b.host.handex_push(0x1C, me as u32);
        b.host.handex(w).put_u32(4, 1);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E0628`: move 356, event 158: when var 3 is the holder and
/// field effect 2 is not active, pushes HANDEX work 0x1B (+4 = 2, +8 `SickCont::turn(5)`, u16
/// 0x75 at +0x10, low byte of the u16 at +0x12 = 1), pops it, then pushes and pops a
/// parameterless work 0x35.
pub fn h_021e0628<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && !b.host.field_effect_active(2) {
        let w = b.host.handex_push(0x1B, me as u32);
        let buf = b.host.handex(w);
        buf.put_u32(4, 2);
        buf.put_u32(8, SickCont::turn(5).0);
        let h = u16::from_le_bytes([buf[0x12], buf[0x13]]);
        buf.put_u16(0x12, (h & 0xFF00) | 1);
        buf.put_u16(0x10, 0x75);
        b.host.handex_pop(w);
        b.host.handex_push_pop(0x35, me as u32);
    }
}

/// C: unlabeled routine at `0x021E068C`: move 346, event 158: when var 3 is the holder, pushes
/// HANDEX work 0x1B (+4 = 4, +8 `SickCont::poke(holder)`, str 0x72 type 1 at +0x10).
pub fn h_021e068c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let w = b.host.handex_push(0x1B, me as u32);
        let buf = b.host.handex(w);
        buf.put_u32(4, 4);
        buf.put_u32(8, SickCont::poke(me as u32).0);
        str_param_setup(buf, 0x10, 1, 0x72);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E06DC`: move 300, event 158: like `0x021E068C` with +4 = 5 and
/// str 0x73.
pub fn h_021e06dc<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let w = b.host.handex_push(0x1B, me as u32);
        let buf = b.host.handex(w);
        buf.put_u32(4, 5);
        buf.put_u32(8, SickCont::poke(me as u32).0);
        str_param_setup(buf, 0x10, 1, 0x73);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E072C`: move 432, event 160: when var 3 is the holder, with
/// `t` = var 6 as a byte: if `t` has no substitute, pushes HANDEX work 0xE (+4 = 7, +0xC =
/// 0xFF, +0xE = 1, +0xF = 1, +0x10 = `t`); then pushes HANDEX work 0x1A (`work_1a_prefix` with
/// the side of `t`; +5 = 0xCF, +6 = 1).
pub fn h_021e072c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let t = b.var(6) as u32 & 0xFF;
    if !b.mon(t).has_substitute() {
        let w = b.host.handex_push(0xE, me as u32);
        let buf = b.host.handex(w);
        buf.put_u32(4, 7);
        buf.put_u8(0xC, 0xFF);
        buf.put_u8(0xE, 1);
        buf.put_u8(0xF, 1);
        buf.put_u8(0x10, t as u8);
        b.host.handex_pop(w);
    }
    let w = b.host.handex_push(0x1A, me as u32);
    let buf = b.host.handex(w);
    work_1a_prefix(buf, poke_side(t));
    buf[5] |= 1 | 2 | 8 | 4 | 0x40 | 0x80;
    buf[6] |= 1;
    b.host.handex_pop(w);
}

/// C: unlabeled routine at `0x021E0850`: move 280, event 69: when var 3 is the holder, with
/// `side` = side of var 4 (as a byte): if `fun_06898cf4(side, 0)` or `fun_06898cf4(side, 1)`,
/// pushes HANDEX work 0x1A (`work_1a_prefix`; +5 = 3) and then HANDEX work 0x3A (+4 = 1).
pub fn h_021e0850<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let side = poke_side(b.var(4) as u32 & 0xFF);
    if b.host.fun_06898cf4(side as u32, 0) || b.host.fun_06898cf4(side as u32, 1) {
        let w = b.host.handex_push(0x1A, me as u32);
        let buf = b.host.handex(w);
        work_1a_prefix(buf, side);
        buf[5] |= 1 | 2;
        b.host.handex_pop(w);
        let w = b.host.handex_push(0x3A, me as u32);
        b.host.handex(w)[4] = 1;
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E0920`: move 26 and 136, event 38: when var 2 is the holder,
/// runs `fun_021e0944`.
pub fn h_021e0920<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        fun_021e0944(b, me as u32);
    }
}

/// The move the Pokemon in var 6 used last: `last_move_orig` when it differs from `last_move`.
fn target_last_move<H: FlowHost>(b: &mut Battle<H>) -> u32 {
    let t = b.var(6) as u32 & 0xFF;
    let a = b.mon(t).last_move() as u32;
    let o = b.mon(t).last_move_orig() as u32;
    if o != a {
        o
    } else {
        a
    }
}

/// Pushes the HANDEX work 0x25 (+4 holder, +5 `slot`, +6 = 0, +7 `flag`, +8 u16 `mv`), pops it,
/// then pushes work 4 with str 0x2B0 at +4 (holder, `mv`) and pops it, as `0x021E0998` and
/// `0x021E0A80` do.
fn push_move_swap_work<H: FlowHost>(b: &mut Battle<H>, me: u8, slot: u8, mv: u32, flag: u8) {
    let w = b.host.handex_push(0x25, me as u32);
    let buf = b.host.handex(w);
    buf.put_u8(4, me);
    buf.put_u8(5, slot);
    buf.put_u16(8, mv as u16);
    buf.put_u8(6, 0);
    buf.put_u8(7, flag);
    b.host.handex_pop(w);
    let w = b.host.handex_push(4, me as u32);
    let buf = b.host.handex(w);
    str_param_setup(buf, 4, 2, 0x2B0);
    str_param_add_arg(buf, 4, me as u32);
    str_param_add_arg(buf, 4, mv);
    b.host.handex_pop(w);
}

/// C: unlabeled routine at `0x021E0998` (verified under the Z18 harness hints): move 102, event 160: when var 3 is the holder and it
/// is not transformed: with `mv` = the move var 6's Pokemon used last, if `mv` is not 0, not
/// `fun_0689ca7c`, and the holder does not have it in a slot (slot 4), and the holder has this
/// move's ID in a slot other than 4, runs `push_move_swap_work` with flag 0.
#[allow(dead_code)]
pub fn h_021e0998<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) || b.mon(me as u32).is_transformed() {
        return;
    }
    let mv = target_last_move(b);
    if mv != 0 && !b.host.fun_0689ca7c(mv) && b.mon(me as u32).move_slot_of(mv) == 4 {
        let slot = b.mon(me as u32).move_slot_of(sys.sub_id(f) as u32);
        if slot != 4 {
            push_move_swap_work(b, me, slot, mv, 0);
        }
    }
}

/// C: unlabeled routine at `0x021E0A80` (verified under the Z18 harness hints): move 166, event 160: when var 3 is the holder: with
/// `mv` = the move var 6's Pokemon used last, if `mv` is not 0, 0xA6, 0xA5 or 0x1C0, the holder
/// does not know it and is not transformed, and the holder has this move's ID in a slot other
/// than 4, runs `push_move_swap_work` with flag 1.
#[allow(dead_code)]
pub fn h_021e0a80<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let mv = target_last_move(b);
    if mv != 0
        && mv != 0xA6
        && mv != 0xA5
        && mv != 0x1C0
        && !b.mon(me as u32).knows_move(mv)
        && !b.mon(me as u32).is_transformed()
    {
        let slot = b.mon(me as u32).move_slot_of(sys.sub_id(f) as u32);
        if slot != 4 {
            push_move_swap_work(b, me, slot, mv, 1);
        }
    }
}

/// C: unlabeled routine at `0x021E0818`: move 280, event 70: when var 3 is the holder, sets the
/// factor's skip-check callback to the thumb routine at `0x021E0900`.
pub fn h_021e0818<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        sys.set_skip_check(f, 0x021E_0901);
    }
}

/// C: unlabeled routine at `0x021B8C20(_, side, n)`: `n * 2 + 1` when bit 0 of `side` is clear,
/// else `n * 2` (the low byte; the first argument is unused).
pub fn fun_021b8c20(_unused: u32, side: u32, n: u32) -> u8 {
    if side & 1 == 0 {
        (n.wrapping_mul(2).wrapping_add(1)) as u8
    } else {
        (n.wrapping_mul(2)) as u8
    }
}

/// C: unlabeled routine at `0x021B8C38(side, n)`: `n * 2` when bit 0 of `side` is clear, else
/// `n * 2 + 1` (the low byte).
pub fn fun_021b8c38(side: u32, n: u32) -> u8 {
    if side & 1 == 0 {
        (n.wrapping_mul(2)) as u8
    } else {
        (n.wrapping_mul(2).wrapping_add(1)) as u8
    }
}

/// Bytes at 0x021EFCF0 read by `fun_021b8d08` (4 entries).
const TABLE_021EFCF0: [u8; 4] = [0x00, 0x0C, 0x06, 0x12];

/// C: unlabeled routine at `0x021B8D08(x)`: the index `i` (0..3) of the first entry `t` of the
/// byte table at 0x021EFCF0 with `t <= x <= (t + 5) & 0xFF` (unsigned), else 0.
pub fn fun_021b8d08(x: u32) -> u32 {
    for (i, &t) in TABLE_021EFCF0.iter().enumerate() {
        if !(x < t as u32 || ((t as u32 + 5) & 0xFF) < x) {
            return i as u32;
        }
    }
    0
}

/// C: unlabeled routine at `0x021B99F0(x)`: whether `x` is 2 or 3.
pub fn fun_021b99f0(x: i32) -> bool {
    x == 2 || x == 3
}

/// C: unlabeled routine at `0x021D7D6C(sick)`: whether `sick` is below 6 (signed).
pub fn fun_021d7d6c(sick: i32) -> bool {
    sick < 6
}

/// C: unlabeled routine at `0x021D7E90(list, count, value)`: whether `value` equals one of the
/// first `count` `u16` entries of `list`.
pub fn fun_021d7e90(list: &[u16], count: u32, value: u32) -> bool {
    let mut i = 0u32;
    if count != 0 {
        loop {
            if value == list[i as usize] as u32 {
                return true;
            }
            i = i.wrapping_add(1);
            if i >= count {
                break;
            }
        }
    }
    false
}

/// C: `BattleMon_GetPokemon` (`0x021D5844`, `ldr r0, [r0]`): the party Pokemon handle.
pub fn fun_021d5844(mon: &BattleMon) -> u32 {
    mon.get_pokemon()
}

/// Type for each Hidden Power index (the bytes at 0x0209E288: 1..=16).
const HIDDEN_POWER_TYPES: [u8; 16] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

/// C: unlabeled routine at `0x02018790(pp)`: Hidden Power type. Sums `1 << k` over `k` in 0..6
/// for which party parameter `0x46 + k` has bit 0 set, takes `15 * sum / 63` (reduced by 16
/// while above 16, which cannot happen) and indexes the type table at 0x0209E288.
pub fn fun_02018790<E: BppEnv>(pp: u32, env: &mut E) -> u8 {
    let mut sum = 0u32;
    for k in 0..6u32 {
        if env.pp_get(pp, 0x46 + k) & 1 != 0 {
            sum += 1 << k;
        }
    }
    let mut t = u32_div(15 * sum, 0x3F);
    while t > 16 {
        t -= 16;
    }
    HIDDEN_POWER_TYPES[t as usize]
}

/// C: unlabeled routine at `0x02018820(pp)`: Hidden Power base power. Sums `1 << k` over `k` in
/// 0..6 for which `pp_get(pp, 0x46 + k) & 3` is above 1, and returns `40 * sum / 63 + 30`.
pub fn fun_02018820<E: BppEnv>(pp: u32, env: &mut E) -> u32 {
    let mut sum = 0u32;
    for k in 0..6u32 {
        if env.pp_get(pp, 0x46 + k) & 3 > 1 {
            sum += 1 << k;
        }
    }
    u32_div(40 * sum, 0x3F) + 30
}

/// C: unlabeled routine at `0x0202136C(x)`: whether `x` equals one of the 12 `u16` entries of
/// the table at 0x0209E8EC (0x89..=0x94).
pub fn fun_0202136c(x: u32) -> bool {
    (0x89..=0x94).any(|v| v == x)
}

/// C: unlabeled routine at `0x02082D44(src, dst, n)`: copies `n` bytes from `src` to `dst`
/// front to back using 16-bit loads and stores only (safe for VRAM); the code special-cases
/// odd addresses and 32-bit alignment, which only changes the access width here. Overlap
/// behaviour is not modelled.
pub fn fun_02082d44(src: &[u8], dst: &mut [u8], n: usize) {
    dst[..n].copy_from_slice(&src[..n]);
}

/// C: unlabeled routine at `0x021D7D9C(poke_id)`: `(poke_id & 0x3F) << 3 | 3`, the sick-cont
/// value for kind 3 linked to a Pokemon (the same value as `SickCont::poke`).
pub fn fun_021d7d9c(poke_id: u32) -> u32 {
    ((poke_id & 0x3F) << 3) | 3
}

/// C: unlabeled routine at `0x021E0B68`: move 266 and 476, event 31: when the battle rule is 0
/// or 3, var 2 is the holder and var 0x22 is 0, sets var 0x22 to 0x1A.
pub fn h_021e0b68<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if (b.host.rule() == 0 || b.host.rule() == 3) && is_me(b, 2, me) && b.var(0x22) == 0 {
        b.rewrite_var(0x22, 0x1A);
    }
}

/// C: unlabeled routine at `0x021E0BA4`: move 266 and 476, event 160: when var 3 is the holder,
/// pushes HANDEX work 4 (str 0x29E at +4 with the holder) and sets `work[6]` to 1.
pub fn h_021e0ba4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let w = b.host.handex_push(4, me as u32);
        let buf = b.host.handex(w);
        str_param_setup(buf, 4, 2, 0x29E);
        str_param_add_arg(buf, 4, me as u32);
        b.host.handex_pop(w);
        sys.set_work(f, 6, 1);
    }
}

/// C: unlabeled routine at `0x021E0BE8`: move 266 and 476, event 42: when the battle rule is
/// neither 0 nor 3, with `t` = var 3 as a byte: if the holder and `t` are on different sides,
/// var 0x12 is not 507, the holder does not have condition 0x21, `fun_021c830c` is false and
/// `fun_021c8290(t, holder, var 0x12 as u16)` is false, sets var 4 to the holder.
pub fn h_021e0be8<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if b.host.rule() == 0 || b.host.rule() == 3 {
        return;
    }
    let t = b.var(3) as u32 & 0xFF;
    let me = me as u32;
    if !same_side(me, t)
        && b.var(0x12) != 507
        && !b.mon(me).has_sick(0x21)
        && !b.host.fun_021c830c()
    {
        let mv = b.var(0x12) as u32 & 0xFFFF;
        if !b.host.fun_021c8290(t, me, mv) {
            b.rewrite_var(4, me as i32);
        }
    }
}

/// C: unlabeled routine at `0x021E0C6C`: move 266 and 476, event 118: `fun_021e03f8(holder,
/// this move's ID)`.
pub fn h_021e0c6c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, _b: &mut Battle<H>, me: u8) {
    let mv = sys.sub_id(f) as u32;
    fun_021e03f8(sys, me as u32, mv);
}

/// C: unlabeled routine at `0x021E0C8C`: move 287, event 160: when var 3 is the holder and its
/// major status is 1, 4 or 5, pushes HANDEX work 0xB (+4 the status, +8 holder, +0x14 = 1).
pub fn h_021e0c8c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let st = b.mon(me as u32).major_status();
    if st == 5 || st == 1 || st == 4 {
        let w = b.host.handex_push(0xB, me as u32);
        let buf = b.host.handex(w);
        buf.put_u32(4, st);
        buf.put_u8(8, me);
        buf.put_u8(0x14, 1);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E0CE0`: move 169, 212 and 335, event 160: when var 3 is the
/// holder, with `t` = var 6: if `t` (as a byte) does not have condition 0x16, pushes HANDEX
/// work 0xC (+0xF `t` as a byte, +4 = 0x16, +8 `fun_021d7d9c(holder)`, str 0x368 at +0x14 with
/// `t` as a byte).
pub fn h_021e0ce0<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let t = b.var(6) as u32;
    if !b.mon(t & 0xFF).has_sick(0x16) {
        let w = b.host.handex_push(0xC, me as u32);
        let buf = b.host.handex(w);
        buf.put_u8(0xF, t as u8);
        buf.put_u32(4, 0x16);
        buf.put_u32(8, fun_021d7d9c(me as u32));
        str_param_setup(buf, 0x14, 2, 0x368);
        str_param_add_arg(buf, 0x14, t & 0xFF);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E0D54`: move 114, event 158: when var 3 is the holder, pushes
/// HANDEX work 0x10 (+4 the count of Pokemon IDs `expand_poke_ids` returns for code `0x800 |
/// position of the holder`, +5.. those IDs) and pops it, then pushes HANDEX work 4 (str 0x65
/// type 1 at +4) and pops it.
pub fn h_021e0d54<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let w = b.host.handex_push(0x10, me as u32);
    let pos = b.host.poke_pos(me as u32);
    let mut ids = [0u8; 8];
    let n = b.host.expand_poke_ids((0x800 | pos) & 0xFFFF, &mut ids);
    let buf = b.host.handex(w);
    buf[5..5 + (n as usize).min(8)].copy_from_slice(&ids[..(n as usize).min(8)]);
    buf[4] = n as u8;
    b.host.handex_pop(w);
    let w = b.host.handex_push(4, me as u32);
    str_param_setup(b.host.handex(w), 4, 1, 0x65);
    b.host.handex_pop(w);
}

/// C: unlabeled routine at `0x021E0DC4`: move 150, event 160: when var 3 is the holder, pushes
/// HANDEX work 4 (str 0x66 type 1 at +4) and pops it.
pub fn h_021e0dc4<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let w = b.host.handex_push(4, me as u32);
        str_param_setup(b.host.handex(w), 4, 1, 0x66);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E0E00`: move 174, event 40: when var 2 is the holder, sets var
/// 0x1B to 0 if the holder has the Ghost type, else 7 (C: `fun_021d80a8`).
pub fn h_021e0e00<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let v = if b.mon(me as u32).has_type(types::GHOST as u32) {
            0
        } else {
            7
        };
        b.rewrite_var(0x1B, v);
    }
}

/// C: unlabeled routine at `0x021E0E28` (verified under the Z18 harness hints): move 174, event 160: when var 3 is the holder: if the
/// holder does not have type 7, pushes three HANDEX work 0xE (+4 = 5 with +0xC = 0xFF, +4 = 1
/// with +0xC = 1, +4 = 2 with +0xC = 1; each +0xF = 1, +0x10 holder) and calls
/// `fun_021c83dc(0)`. If it has type 7: when the holder's BattleMon get 0xD is above 0 (signed)
/// and var 6 as a byte is not 0x1F (`t`), pushes HANDEX work 0xC (+0xF `t`, +4 = 10, +8
/// `SickCont::permanent`, str 0x428 at +0x14 with holder and `t`), then HANDEX work 8 (+7
/// holder, +0x10 = minus max HP / 2 (at least 1), +4 = 1, bit 24 of the u32 at +0 set); then
/// calls `fun_021c83dc(1)`. (Get 0xD is `value::HP`, a name inferred from its use; the meaning
/// of type 7 and of the HANDEX work 0xE/0xC/8 fields is UNVERIFIED.)
#[allow(dead_code)]
pub fn h_021e0e28<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let m = me as u32;
    let flag = if !b.mon(m).has_type(7) {
        for (kind, amount) in [(5u32, 0xFFu8), (1, 1), (2, 1)] {
            let w = b.host.handex_push(0xE, m);
            let buf = b.host.handex(w);
            buf.put_u8(0xF, 1);
            buf.put_u8(0x10, me);
            buf.put_u32(4, kind);
            buf.put_u8(0xC, amount);
            b.host.handex_pop(w);
        }
        0
    } else {
        if b.mon_get(m, 0xD) as i32 > 0 {
            let t = b.var(6) as u32 & 0xFF;
            if t != 0x1F {
                let w = b.host.handex_push(0xC, m);
                let buf = b.host.handex(w);
                buf.put_u8(0xF, t as u8);
                buf.put_u32(4, 10);
                buf.put_u32(8, SickCont::permanent().0);
                str_param_setup(buf, 0x14, 2, 0x428);
                str_param_add_arg(buf, 0x14, m);
                str_param_add_arg(buf, 0x14, t);
                b.host.handex_pop(w);
                let w = b.host.handex_push(8, m);
                let hp = b.max_hp_div_min1(m, 2);
                let buf = b.host.handex(w);
                buf.put_u8(7, me);
                buf.put_u32(0x10, hp.wrapping_neg());
                buf.put_u8(4, 1);
                let v = buf.get_u32(0) | 0x100_0000;
                buf.put_u32(0, v);
                b.host.handex_pop(w);
            }
        }
        1
    };
    b.host.fun_021c83dc(flag);
}

/// C: unlabeled routine at `0x021E0F50`: move 86, event 61: when var 3 is the holder, sets var
/// 0x51 to 1.
pub fn h_021e0f50<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        b.rewrite_var(0x51, 1);
    }
}

/// C: unlabeled routine at `0x021E0F74`: move 388, event 44: when var 3 is the holder and the
/// BattleMon get 0x11 (`value::ABILITY`, name inferred) of var 4 (as a byte) is 0x36 or 0xF,
/// sets var 0x40 to 1.
pub fn h_021e0f74<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let t = b.var(4) as u32 & 0xFF;
    if b.mon_get(t, 0x11) == 0x36 || b.mon_get(t, 0x11) == 0xF {
        b.rewrite_var(0x40, 1);
    }
}

/// C: unlabeled routine at `0x021E0FB8`: move 388, event 160: when var 3 is the holder, pushes
/// HANDEX work 0x1F (+6 var 6 as a byte, +4 u16 = 0xF, str 0x195 at +0xC with the byte at +6
/// and the u16 at +4) and pops it.
pub fn h_021e0fb8<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let t = b.var(6);
    let w = b.host.handex_push(0x1F, me as u32);
    let buf = b.host.handex(w);
    buf.put_u8(6, t as u8);
    buf.put_u16(4, 0xF);
    str_param_setup(buf, 0xC, 2, 405);
    let (a0, a1) = (buf[6] as u32, 0xF);
    str_param_add_arg(buf, 0xC, a0);
    str_param_add_arg(buf, 0xC, a1);
    b.host.handex_pop(w);
}

/// C: unlabeled routine at `0x021E101C`: move 138, event 44: when var 3 is the holder and var 4
/// (as a byte) does not have condition 2, sets var 0x40 to 1.
pub fn h_021e101c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let t = b.var(4) as u32 & 0xFF;
    if !b.mon(t).has_sick(2) {
        b.rewrite_var(0x40, 1);
    }
}

/// C: unlabeled routine at `0x021E1050`: move 138, event 5: when var 3 is the holder, sets var
/// 0x40 to 1.
pub fn h_021e1050<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        b.rewrite_var(0x40, 1);
    }
}

/// C: unlabeled routine at `0x021E1074`: move 445, event 44: when var 3 is the holder, with `t`
/// = var 4 as a byte: sets var 0x40 to 1 if, taking the low bytes of BattleMon get 0x12
/// (`value::FA_BYTE`, meaning UNVERIFIED) of the holder (`a`) and of `t` (`c`), `a` is 2, `c`
/// is 2 or `a == c`.
pub fn h_021e1074<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let t = b.var(4) as u32 & 0xFF;
    let a = b.mon_get(me as u32, 0x12) as u8;
    let c = b.mon_get(t, 0x12) as u8;
    if a == 2 || c == 2 || a == c {
        b.rewrite_var(0x40, 1);
    }
}

/// C: unlabeled routine at `0x021E112C(flow, out1, out2)`: maps the ground kind
/// (`fun_021c8174`) to `(result, *out1, *out2)` (the callers pass null for outputs they do not
/// need). Kinds 0,5 -> (7, 0, 2); 1,2,3,8,15 -> (1, 1, 6); 6,11,12 -> (5, 1, 1); 7 -> (3, 0, 3);
/// 13 -> (4, 0, 3); 9 -> (2, 1, 5); 10 -> (6, 2, 0); others -> (0, 0, 1).
pub fn fun_021e112c<H: FlowHost>(b: &mut Battle<H>) -> (u32, u8, u8) {
    match b.host.fun_021c8174() {
        0 | 5 => (7, 0, 2),
        1 | 2 | 3 | 8 | 0xF => (1, 1, 6),
        6 | 0xB | 0xC => (5, 1, 1),
        7 => (3, 0, 3),
        0xD => (4, 0, 3),
        9 => (2, 1, 5),
        10 => (6, 2, 0),
        _ => (0, 0, 1),
    }
}

/// C: unlabeled routine at `0x021E10D8`: move 161, event 100: when var 3 is the holder, picks
/// `v` = `[1, 4, 3][rand(3)]` (table at 0x021F286C), then sets var 0x1D to `v`, var 0x1E to
/// `make_default_sick_cont(v)` and var 0x26 to 0x14.
pub fn h_021e10d8<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let r = b.calc.rand(3);
    let v = [1u32, 4, 3][(r & 0xFF) as usize];
    let cont = b.calc.make_default_sick_cont(v);
    b.rewrite_var(0x1D, v as i32);
    b.rewrite_var(0x1E, cont.0 as i32);
    b.rewrite_var(0x26, 0x14);
}

/// C: unlabeled routine at `0x021E11B8`: move 290, event 36: when var 2 is the holder, calls
/// `fun_021c83dc` with the result of `fun_021e112c`.
pub fn h_021e11b8<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let (v, _, _) = fun_021e112c(b);
        b.host.fun_021c83dc(v);
    }
}

/// C: unlabeled routine at `0x021E11DC` (verified under the Z18 harness hints): move 290, event 131: when var 3 is the holder, var 0x47
/// is 0 and `rand(100)` is below 30: with `t` = var 6 as a byte and `(_, kind, x)` =
/// `fun_021e112c`: kind 0 pushes HANDEX work 0xC (+0xF `t`, +4 `x`, +8 `make_sick_cont(x)` of
/// the holder); kind 1 pushes HANDEX work 0xE (+0xF = 1, +0x10 `t`, +4 `x`, +0xC = 0xFF);
/// kind 2 pushes HANDEX work 0x2B (+4 `t`, +5 = 100).
#[allow(dead_code)]
pub fn h_021e11dc<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !(is_me(b, 3, me) && b.var(0x47) == 0 && b.calc.rand(100) < 0x1E) {
        return;
    }
    let m = me as u32;
    let t = b.var(6) as u8;
    let (_, kind, x) = fun_021e112c(b);
    let x = x as u32;
    match kind {
        0 => {
            let w = b.host.handex_push(0xC, m);
            b.host.handex(w).put_u8(0xF, t);
            b.host.handex(w).put_u32(4, x);
            let cont = b.make_sick_cont(x, m);
            b.host.handex(w).put_u32(8, cont.0);
            b.host.handex_pop(w);
        }
        1 => {
            let w = b.host.handex_push(0xE, m);
            let buf = b.host.handex(w);
            buf.put_u8(0xF, 1);
            buf.put_u8(0x10, t);
            buf.put_u32(4, x);
            buf.put_u8(0xC, 0xFF);
            b.host.handex_pop(w);
        }
        2 => {
            let w = b.host.handex_push(0x2B, m);
            let buf = b.host.handex(w);
            buf.put_u8(4, t);
            buf.put_u8(5, 100);
            b.host.handex_pop(w);
        }
        _ => {}
    }
}

/// C: unlabeled routine at `0x021E12A4` (verified under the Z18 harness hints): move 448, event 100: when var 3 is the holder, its
/// species is 441 and `n` = `fun_021c7ed8(holder)` is not 0: sets var 0x1D to 6, var 0x1E to
/// `make_sick_cont(6)` of the holder and var 0x26 to 10 (`n` below 3) or 30.
#[allow(dead_code)]
pub fn h_021e12a4<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let m = me as u32;
    if b.mon(m).species() as u32 != 441 {
        return;
    }
    let n = b.host.fun_021c7ed8(m);
    if n == 0 {
        return;
    }
    let v26 = if n > 2 { 0x1E } else { 10 };
    let cont = b.make_sick_cont(6, m);
    b.rewrite_var(0x1D, 6);
    b.rewrite_var(0x1E, cont.0 as i32);
    b.rewrite_var(0x26, v26);
}

/// C: unlabeled routine at `0x021E1320`: moves 35, 20, 83, 128, 328, 463 and 250, event 100:
/// when var 3 is the holder, sets the parameter (`SickCont::set_param_any`) of the sick cont in
/// var 0x1E to this move's ID.
pub fn h_021e1320<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let mut cont = SickCont(b.var(0x1E) as u32);
        cont.set_param_any(sys.sub_id(f) as u32);
        b.rewrite_var(0x1E, cont.0 as i32);
    }
}

/// C: unlabeled routine at `0x021E1354` (verified under the Z18 harness hints): moves 35, 20, 83, 128, 328, 463 and 250, event 97:
/// when var 3 is the holder and var 0x1D is 8: with `t` = var 4 as a byte, sets up the string
/// parameter var 0x3F points to (`msg_param`) with type 2, the holder and `t` as arguments and
/// the string by move ID: 0x14 -> 0x323, 0x23 -> 0x32A, 0x53 -> 0x33B, 0x80 -> 0x331, 0xFA ->
/// 0x338, 0x148 -> 0x341, 0x1CF -> 0x33E; any other move does nothing.
#[allow(dead_code)]
pub fn h_021e1354<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if !(is_me(b, 3, me) && b.var(0x1D) == 8) {
        return;
    }
    let t = b.var(4) as u32 & 0xFF;
    let sid = match sys.sub_id(f) {
        0x14 => 0x323,
        0x23 => 0x32A,
        0x53 => 0x33B,
        0x80 => 0x331,
        0xFA => 0x338,
        0x148 => 0x341,
        0x1CF => 0x33E,
        _ => return,
    };
    let buf = b.host.msg_param();
    str_param_setup(buf, 0, 2, sid);
    str_param_add_arg(buf, 0, t);
    str_param_add_arg(buf, 0, me as u32);
}

/// C: unlabeled routine at `0x021E1410`: move 250, event 153: when var 3 is the holder and var
/// 0x21 is 4, sets var 0x42 to 0.
pub fn h_021e1410<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.var(0x21) == 4 {
        b.rewrite_var(0x42, 0);
    }
}

/// C: unlabeled routine at `0x021E1434`: move 250, event 71: when var 3 is the holder and var 4
/// (as a byte) has flag B 4 set, multiplies var 0x35 by 0x2000.
pub fn h_021e1434<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let t = b.var(4) as u32 & 0xFF;
    if b.mon(t).flag_b(4) {
        b.mul_var(0x35, 0x2000);
    }
}

/// C: unlabeled routine at `0x021E1478`: move 162, event 70: when var 3 is the holder, sets var
/// 0x37 to the current HP of var 4 (as a byte) halved by the original's sign-corrected 17-bit
/// shift, at least 1.
pub fn h_021e1478<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let t = b.var(4) as u32 & 0xFF;
    let hp = b.mon_get(t, value::HP) as i32;
    let mut v = ((hp.wrapping_sub(hp >> 31)) as u32 & 0x1_FFFF) >> 1;
    if v == 0 {
        v = 1;
    }
    b.rewrite_var(0x37, v as i32);
}

/// C: unlabeled routine at `0x021E14C0`: move 82, event 70: when var 3 is the holder, sets var
/// 0x37 to 40.
pub fn h_021e14c0<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        b.rewrite_var(0x37, 0x28);
    }
}

/// C: unlabeled routine at `0x021E14E4`: move 49, event 70: when var 3 is the holder, sets var
/// 0x37 to 20.
pub fn h_021e14e4<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        b.rewrite_var(0x37, 0x14);
    }
}

/// C: unlabeled routine at `0x021E1508`: move 283, event 44: when var 3 is the holder and the
/// current HP of var 4 (as a byte) minus the holder's current HP is below 1 (signed), sets var
/// 0x40 to 1.
pub fn h_021e1508<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let t = b.var(4) as u32 & 0xFF;
    let a = b.mon_get(t, value::HP) as i32;
    let c = b.mon_get(me as u32, value::HP) as i32;
    if a.wrapping_sub(c) < 1 {
        b.rewrite_var(0x40, 1);
    }
}

/// C: unlabeled routine at `0x021E1558`: move 283, event 70: when var 3 is the holder, sets var
/// 0x37 to the current HP of var 4 (as a byte) minus the holder's current HP, or 1 when that is
/// negative.
pub fn h_021e1558<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let t = b.var(4) as u32 & 0xFF;
    let a = b.mon_get(t, value::HP) as i32;
    let c = b.mon_get(me as u32, value::HP) as i32;
    let mut d = a.wrapping_sub(c);
    if d < 0 {
        d = 1;
    }
    b.rewrite_var(0x37, d);
}

/// C: unlabeled routine at `0x021E15B0`: moves 69 and 101, event 70: when var 3 is the holder,
/// sets var 0x37 to the holder's level (low byte of BattleMon get 0xF).
pub fn h_021e15b0<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let lv = b.mon_get(me as u32, value::LEVEL) & 0xFF;
        b.rewrite_var(0x37, lv as i32);
    }
}

/// C: unlabeled routine at `0x021E15E8`: move 149, event 70: when var 3 is the holder, sets var
/// 0x37 to `level * ((rand(101) + 50) & 0xFFFF) / 100` (as an int division), truncated to 16
/// bits, at least 1.
pub fn h_021e15e8<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let lv = b.mon_get(me as u32, value::LEVEL) & 0xFF;
    let r = b.calc.rand(0x65);
    let x = lv.wrapping_mul(r.wrapping_add(0x32) & 0xFFFF);
    let mut v = ((x as i32).wrapping_div(100) as u32) & 0xFFFF;
    if v == 0 {
        v = 1;
    }
    b.rewrite_var(0x37, v as i32);
}

/// C: unlabeled routine at `0x021E163C`: move 254, event 31: when var 2 is the holder and its
/// byte at `+0x157` (`u8_157(0)`) is above 2, sets var 0x22 to 0x1A.
pub fn h_021e163c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && b.mon(me as u32).u8_157(0) > 2 {
        b.rewrite_var(0x22, 0x1A);
    }
}

/// C: unlabeled routine at `0x021E1AE4(bpp, kind, out)`: searches the Pokemon's most recent
/// action records (`history_record(0, n)`, n = 0, 1, 2, ... until none) for the first one whose
/// attacker (byte `+5`) is on the other side from the Pokemon and, unless `kind` is 0, whose
/// move (`u16` at +0) has the damage category `kind`; returns it (the original writes it to
/// `out`).
pub fn fun_021e1ae4<H: FlowHost>(b: &mut Battle<H>, me: u32, kind: u32) -> Option<[u16; 4]> {
    let a = b.mon(me).poke_id() as u32;
    let mut n = 1u8;
    let mut rec = b.mon(me).history_record(0, 0);
    while let Some(r) = rec {
        if !same_side(a, (r[2] >> 8) as u32)
            && (kind == 0 || kind == b.host.move_category(r[0] as u32))
        {
            return Some(r);
        }
        rec = b.mon(me).history_record(0, n as u32);
        n = n.wrapping_add(1);
    }
    None
}

/// C: unlabeled routine at `0x021E19F8(flow, poke_id, work, kind)`: when var 2 is the holder and
/// `fun_021e1ae4(holder, kind)` finds no record, sets var 0x22 to 0x1A.
pub fn fun_021e19f8<H: FlowHost>(b: &mut Battle<H>, me: u8, kind: u32) {
    if is_me(b, 2, me) && fun_021e1ae4(b, me as u32, kind).is_none() {
        b.rewrite_var(0x22, 0x1A);
    }
}

/// C: unlabeled routine at `0x021E1A2C(flow, poke_id, work, kind)`: when var 3 is the holder
/// and `fun_021e1ae4(holder, kind)` finds a record, with `atk` = its attacker (byte `+5`): sets
/// var 4 to `atk`, except that when `atk`'s position (`poke_pos`) is 6 the value is 0x1F, or
/// `fun_021c7d44()` unless the record's byte `+6` is 6, and if that is 0x1F it is replaced by
/// `fun_021c7d44()` unless `fun_021c8030(holder, 1)` is 6, and if still 0x1F by `atk`.
pub fn fun_021e1a2c<H: FlowHost>(b: &mut Battle<H>, me: u8, kind: u32) {
    if !is_me(b, 3, me) {
        return;
    }
    let Some(r) = fun_021e1ae4(b, me as u32, kind) else {
        return;
    };
    let atk = (r[2] >> 8) as u32;
    let mut value = atk;
    if b.host.poke_pos(atk) == 6 {
        value = 0x1F;
        if r[3] as u8 != 6 {
            value = b.host.fun_021c7d44();
        }
        if value == 0x1F {
            if b.host.fun_021c8030(me as u32, 1) != 6 {
                value = b.host.fun_021c7d44();
            }
            if value == 0x1F {
                value = atk;
            }
        }
    }
    b.rewrite_var(4, value as i32);
}

/// C: unlabeled routine at `0x021E1AA4(flow, poke_id, work, kind, fx)`: when var 3 is the
/// holder and `fun_021e1ae4(holder, kind)` finds a record, sets var 0x37 to `fx_mul` of the
/// record's `u16` at +2 and `fx`.
pub fn fun_021e1aa4<H: FlowHost>(b: &mut Battle<H>, me: u8, kind: u32, fx: u32) {
    if !is_me(b, 3, me) {
        return;
    }
    if let Some(r) = fun_021e1ae4(b, me as u32, kind) {
        let v = fx_mul(r[1] as u32, fx);
        b.rewrite_var(0x37, v as i32);
    }
}

/// C: unlabeled routine at `0x021E1668` (verified under the Z18 harness hints): move 254, event 160: when var 3 is the holder and its
/// byte `+0x157` (`n` = `u8_157(0)`) is below 3: pushes HANDEX work 0x26 (+4 holder, +5 = 0, +6
/// `n + 1`), work 4 (str 0x2D1 at +4 with holder and `n + 1`), work 0xE (+0xF = 1, +0x10
/// holder, +4 = 2, +0xC = 1, +0xE = 1), work 0x26 (bit 24 of the u32 at +0, +4 holder, +5 = 1,
/// +6 `u8_157(1) + 1`), work 0xE (as before with +4 = 4), and work 0x26 (+5 = 2, +6
/// `u8_157(2) + 1`).
#[allow(dead_code)]
pub fn h_021e1668<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let m = me as u32;
    let n = b.mon(m).u8_157(0);
    if n >= 3 {
        return;
    }
    let w = b.host.handex_push(0x26, m);
    let buf = b.host.handex(w);
    buf.put_u8(4, me);
    buf.put_u8(5, 0);
    buf.put_u8(6, n.wrapping_add(1));
    b.host.handex_pop(w);
    let w = b.host.handex_push(4, m);
    let buf = b.host.handex(w);
    str_param_setup(buf, 4, 2, 0x2D1);
    str_param_add_arg(buf, 4, m);
    str_param_add_arg(buf, 4, n as u32 + 1);
    b.host.handex_pop(w);
    for (kind, slot) in [(2u32, 1u32), (4, 2)] {
        let w = b.host.handex_push(0xE, m);
        let buf = b.host.handex(w);
        buf.put_u8(0xF, 1);
        buf.put_u8(0x10, me);
        buf.put_u32(4, kind);
        buf.put_u8(0xC, 1);
        buf.put_u8(0xE, 1);
        b.host.handex_pop(w);
        let w = b.host.handex_push(0x26, m);
        let v = b.mon(m).u8_157(slot);
        let buf = b.host.handex(w);
        let h = buf.get_u32(0) | 0x100_0000;
        buf.put_u32(0, h);
        buf.put_u8(4, me);
        buf.put_u8(5, slot as u8);
        buf.put_u8(6, v.wrapping_add(1));
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E1784`: moves 255 and 256, event 31: when var 2 is the holder
/// and its byte `+0x157` is 0, sets var 0x22 to 0x1A.
pub fn h_021e1784<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && b.mon(me as u32).u8_157(0) == 0 {
        b.rewrite_var(0x22, 0x1A);
    }
}

/// C: unlabeled routine at `0x021E17B0`: move 255, event 55: when var 3 is the holder, sets var
/// 0x30 to `u8_157(0) * 100`.
pub fn h_021e17b0<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let n = b.mon(me as u32).u8_157(0) as i32;
        b.rewrite_var(0x30, n * 100);
    }
}

/// C: unlabeled routine at `0x021E17DC`: moves 255 and 256, event 39: when var 2 is the holder,
/// with `a`, `c`, `d` = `u8_157(1)`, `u8_157(2)`, `u8_157(0)` of the holder: if `a` is not 0
/// pushes HANDEX work 0xE (+0xF = 1, +0x10 holder, +4 = 2, +0xC = `-a`) and work 0x26 (+4
/// holder, +5 = 1, +6 = 0); if `c` is not 0 the same with +4 = 4, `-c` and +5 = 2; if `d` is
/// not 0 pushes HANDEX work 0x26 (+4 holder, +5 = 0, +6 = 0) and work 4 (str 0x2D4 at +4 with
/// the holder).
pub fn h_021e17dc<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) {
        return;
    }
    let m = me as u32;
    let a = b.mon(m).u8_157(1);
    let c = b.mon(m).u8_157(2);
    let d = b.mon(m).u8_157(0);
    for (kind, v, slot) in [(2u32, a, 1u8), (4, c, 2)] {
        if v != 0 {
            let w = b.host.handex_push(0xE, m);
            let buf = b.host.handex(w);
            buf.put_u8(0xF, 1);
            buf.put_u8(0x10, me);
            buf.put_u32(4, kind);
            buf.put_u8(0xC, v.wrapping_neg());
            b.host.handex_pop(w);
            let w = b.host.handex_push(0x26, m);
            let buf = b.host.handex(w);
            buf.put_u8(4, me);
            buf.put_u8(5, slot);
            buf.put_u8(6, 0);
            b.host.handex_pop(w);
        }
    }
    if d != 0 {
        let w = b.host.handex_push(0x26, m);
        let buf = b.host.handex(w);
        buf.put_u8(4, me);
        buf.put_u8(5, 0);
        buf.put_u8(6, 0);
        b.host.handex_pop(w);
        let w = b.host.handex_push(4, m);
        let buf = b.host.handex(w);
        str_param_setup(buf, 4, 2, 0x2D4);
        str_param_add_arg(buf, 4, m);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E18EC`: move 256, event 143: when var 2 is the holder, sets var
/// 0x35 from its `u8_157(0)`: 2 -> 0x800, 3 -> 0x1000, anything else (0x400 for 1) -> 0x400.
pub fn h_021e18ec<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) {
        return;
    }
    let v = match b.mon(me as u32).u8_157(0) {
        2 => 0x800,
        3 => 0x1000,
        _ => 0x400,
    };
    b.rewrite_var(0x35, v);
}

/// C: unlabeled routine at `0x021E1938`: move 68, event 31: `fun_021e19f8` with kind 1.
pub fn h_021e1938<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    fun_021e19f8(b, me, 1);
}

/// C: unlabeled routine at `0x021E1948`: move 68, event 41: `fun_021e1a2c` with kind 1.
pub fn h_021e1948<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    fun_021e1a2c(b, me, 1);
}

/// C: unlabeled routine at `0x021E1958`: move 68, event 70: `fun_021e1aa4` with kind 1 and `fx`
/// 0x2000.
pub fn h_021e1958<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    fun_021e1aa4(b, me, 1, 0x2000);
}

/// C: unlabeled routine at `0x021E197C`: move 243, event 31: `fun_021e19f8` with kind 2.
pub fn h_021e197c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    fun_021e19f8(b, me, 2);
}

/// C: unlabeled routine at `0x021E198C`: move 243, event 41: `fun_021e1a2c` with kind 2.
pub fn h_021e198c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    fun_021e1a2c(b, me, 2);
}

/// C: unlabeled routine at `0x021E199C`: move 243, event 70: `fun_021e1aa4` with kind 2 and `fx`
/// 0x2000.
pub fn h_021e199c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    fun_021e1aa4(b, me, 2, 0x2000);
}

/// C: unlabeled routine at `0x021E19C0`: move 368, event 31: `fun_021e19f8` with kind 0.
pub fn h_021e19c0<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    fun_021e19f8(b, me, 0);
}

/// C: unlabeled routine at `0x021E19D0`: move 368, event 41: `fun_021e1a2c` with kind 0.
pub fn h_021e19d0<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    fun_021e1a2c(b, me, 0);
}

/// C: unlabeled routine at `0x021E19E0`: move 368, event 70: `fun_021e1aa4` with kind 0 and `fx`
/// 0x1800.
pub fn h_021e19e0<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    fun_021e1aa4(b, me, 0, 0x1800);
}

/// C: unlabeled routine at `0x021E1B4C`: move 387, event 31: when var 2 is the holder, with `n`
/// its move count and `used` the number of its moves other than move 387 whose used flag is set:
/// sets var 0x22 to 0x1A unless it knows move 387, `n` is at least 2 and `used` is at least
/// `n - 1` (signed).
pub fn h_021e1b4c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) {
        return;
    }
    let m = me as u32;
    let n = b.mon(m).move_count() as u32;
    let mut used = 0u32;
    let mut found = false;
    let mut slot = 0u32;
    if n != 0 {
        loop {
            if b.mon(m).move_id(slot) == 387 {
                found = true;
            } else if b.mon(m).move_used_flag(slot) != 0 {
                used = used.wrapping_add(1) & 0xFF;
            }
            slot = slot.wrapping_add(1) & 0xFF;
            if slot >= n {
                break;
            }
        }
    }
    if !found || n < 2 || (used as i32) < (n.wrapping_sub(1)) as i32 {
        b.rewrite_var(0x22, 0x1A);
    }
}

/// C: unlabeled routine at `0x021E1BD4`: moves 173 and 214, event 29: when var 2 is the holder
/// and var 0x22 is 2, sets var 0x41 to 1.
pub fn h_021e1bd4<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && b.var(0x22) == 2 {
        b.rewrite_var(0x41, 1);
    }
}

/// C: `sub_021E6210`: empty routine (a bare return); move 19, event 152.
pub fn h_021e6210<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, _b: &mut Battle<H>, _me: u8) {}

/// C: `sub_021E6274`: empty routine; move 467, event 152.
pub fn h_021e6274<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, _b: &mut Battle<H>, _me: u8) {}

/// C: `sub_021E6348`: empty routine; move 340, event 152.
pub fn h_021e6348<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, _b: &mut Battle<H>, _me: u8) {}

/// C: `sub_021E63AC`: empty routine; move 291, event 152.
pub fn h_021e63ac<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, _b: &mut Battle<H>, _me: u8) {}

/// C: `sub_021E6410`: empty routine; move 91, event 152.
pub fn h_021e6410<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, _b: &mut Battle<H>, _me: u8) {}

/// C: `sub_021E278C`: move 167, event 53: when var 3 is the holder, sets var 0x42 to 1.
pub fn h_021e278c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        b.rewrite_var(0x42, 1);
    }
}

/// C: `FUN_overlay_d_93__021E357C`: move 277, event 118: when var 2 is the holder, removes the
/// factor.
pub fn h_021e357c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        sys.remove(f);
    }
}

/// C: `sub_021E36A4`: move 289, event 118: when var 2 is the holder, removes the factor.
pub fn h_021e36a4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        sys.remove(f);
    }
}

/// C: `FUN_overlay_d_93__021E3AF0`: move 203, event 118: when var 2 is the holder, removes the
/// factor.
pub fn h_021e3af0<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        sys.remove(f);
    }
}

/// C: `sub_021E4220`: moves 364 and 467, event 46: when var 3 is the holder, sets var 0x51 to 1.
pub fn h_021e4220<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        b.rewrite_var(0x51, 1);
    }
}

/// C: `sub_021E6654`: move 165, event 80: when var 3 is the holder, sets var 0x51 to 1.
pub fn h_021e6654<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        b.rewrite_var(0x51, 1);
    }
}

/// C: `sub_021E66A8`: move 165, event 40: when var 2 is the holder, sets var 0x4B to 1.
pub fn h_021e66a8<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        b.rewrite_var(0x4B, 1);
    }
}

/// C: `FUN_overlay_d_93__021E6710`: moves 194 and 288, event 1: when var 2 is the holder,
/// removes the factor.
pub fn h_021e6710<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        sys.remove(f);
    }
}

/// C: `sub_021E795C`: moves 498 and 533, event 58: when var 3 is the holder, sets var 0x51 to 1.
pub fn h_021e795c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        b.rewrite_var(0x51, 1);
    }
}

/// C: `sub_021E7974`: moves 498 and 533, event 51: when var 3 is the holder, sets var 0x4B to 1.
pub fn h_021e7974<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        b.rewrite_var(0x4B, 1);
    }
}

/// C: `sub_021E6F14`: move 382, event 56: when var 3 is the holder, multiplies var 0x31 by 0x1800
/// (`EventVar_Mul` ratio).
pub fn h_021e6f14<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        b.mul_var(0x31, 0x1800);
    }
}

/// C: `FUN_overlay_d_93__021E5FD0`: move 270, event 56: when the factor's work[0] equals var 3,
/// multiplies var 0x31 by 0x1800.
pub fn h_021e5fd0<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, _me: u8) {
    if sys.factor(f).work[0] as i32 == b.var(3) {
        b.mul_var(0x31, 0x1800);
    }
}

/// C: `FUN_overlay_d_93__021E1F70`: move 268, event 4: when var 2 is the holder and work[0] is
/// 2, removes the factor.
pub fn h_021e1f70<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && sys.factor(f).work[0] == 2 {
        sys.remove(f);
    }
}

/// C: `sub_021E7300`: move 492, event 57: when var 3 is the holder, sets var 0x3B to var 4 as a
/// byte.
pub fn h_021e7300<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let v = b.var(4) as u32 & 0xFF;
        b.rewrite_var(0x3B, v as i32);
    }
}

/// C: `sub_021E8478`: moves 518, 519 and 520, event 64: when var 2 is the holder and work[0] is
/// not 0, sets var 0x51 to 1.
pub fn h_021e8478<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && sys.factor(f).work[0] != 0 {
        b.rewrite_var(0x51, 1);
    }
}

/// C: `sub_021E8498`: moves 518, 519 and 520, event 55: when var 3 is the holder and work[0] is
/// not 0, sets var 0x30 to 0x96.
pub fn h_021e8498<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && sys.factor(f).work[0] != 0 {
        b.rewrite_var(0x30, 0x96);
    }
}

/// C: `sub_021E2768`: move 167, event 55: when var 3 is the holder, increments work[0] and sets
/// var 0x30 to `(work[0] + 1) * 10`.
pub fn h_021e2768<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let v = sys.factor(f).work[0].wrapping_add(1);
        sys.factor_mut(f).work[0] = v;
        b.rewrite_var(0x30, (v as i32).wrapping_mul(10));
    }
}

/// C: `sub_021E2FE8`: moves 239 and 16, event 153: when var 3 is the holder and var 0x21 is 3,
/// sets var 0x42 to 0.
pub fn h_021e2fe8<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.var(0x21) == 3 {
        b.rewrite_var(0x42, 0);
    }
}

/// C: `sub_021E3050`: move 327, event 153: when var 3 is the holder and var 0x21 is 3, sets
/// var 0x42 to 0.
pub fn h_021e3050<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.var(0x21) == 3 {
        b.rewrite_var(0x42, 0);
    }
}

/// C: `sub_021E3080`: moves 87, 479 and 542, event 153: when var 3 is the holder and var 0x21 is
/// 3, sets var 0x42 to 0.
pub fn h_021e3080<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.var(0x21) == 3 {
        b.rewrite_var(0x42, 0);
    }
}

/// C: `sub_021E3128`: moves 89 and 222, event 153: when var 3 is the holder and var 0x21 is 5,
/// sets var 0x42 to 0.
pub fn h_021e3128<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.var(0x21) == 5 {
        b.rewrite_var(0x42, 0);
    }
}

/// C: `sub_021E399C`: move 57, event 153: when var 3 is the holder and var 0x21 is 4, sets var
/// 0x42 to 0.
pub fn h_021e399c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.var(0x21) == 4 {
        b.rewrite_var(0x42, 0);
    }
}

/// C: `sub_021E3A48`: move 206, event 116: when var 3 is the holder and var 0x51 is not 0, sets
/// var 0x3A to 2.
pub fn h_021e3a48<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.var(0x51) != 0 {
        b.rewrite_var(0x3A, 2);
    }
}

/// C: `sub_021E6F98`: moves 246, 466 and 318, event 89: when var 4 and var 3 are both the holder,
/// sets var 0x1F to 10.
pub fn h_021e6f98<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me) && is_me(b, 3, me) {
        b.rewrite_var(0x1F, 10);
    }
}

/// C: `sub_021E1DF4`: moves 153 and 120, event 36: when var 2 is the holder, runs
/// `handex_push_pop(0x3B, holder)`.
pub fn h_021e1df4<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        b.host.handex_push_pop(0x3B, me as u32);
    }
}

/// C: `sub_021E1D78`: move 228, event 50: when var 3 is the holder and `fun_021c830c` holds,
/// sets var 0x51 to 1.
pub fn h_021e1d78<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.host.fun_021c830c() {
        b.rewrite_var(0x51, 1);
    }
}

/// C: `sub_021E30A4`: moves 87 and 542, event 50: when `me` is var 3 and the weather is 2, sets
/// var 0x51 to 1.
pub fn h_021e30a4<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.host.weather() == 2 {
        b.rewrite_var(0x51, 1);
    }
}

/// C: `sub_021E30C8`: moves 87 and 542, event 52: when `me` is var 3 and the weather is 1, sets
/// var 0x2B to 0x32.
pub fn h_021e30c8<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.host.weather() == 1 {
        b.rewrite_var(0x2B, 0x32);
    }
}

/// C: `sub_021E30F8`: move 59, event 50: when `me` is var 3 and the weather is 3, sets var 0x51
/// to 1.
pub fn h_021e30f8<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.host.weather() == 3 {
        b.rewrite_var(0x51, 1);
    }
}

/// C: `sub_021E396C`: move 222, event 55: when var 3 is the holder, sets work[0] to 10 if it is 0,
/// then sets var 0x30 to work[0].
pub fn h_021e396c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        if sys.factor(f).work[0] == 0 {
            sys.factor_mut(f).work[0] = 10;
        }
        b.rewrite_var(0x30, sys.factor(f).work[0] as i32);
    }
}

/// C: `FUN_overlay_d_93__021E1F18`: move 268, event 56: when work[0] is 2, var 3 is the holder and
/// var 0x16 is 0xC, multiplies var 0x31 by 0x2000.
pub fn h_021e1f18<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if sys.factor(f).work[0] == 2 && is_me(b, 3, me) && b.var(0x16) == 0xC {
        b.mul_var(0x31, 0x2000);
    }
}

/// C: `FUN_overlay_d_93__021E1F44`: move 268, event 3: when var 3 is the holder and var 0x12 differs
/// from the factor's sub ID, sets work[0] to 2.
pub fn h_021e1f44<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.var(0x12) != sys.sub_id(f) as i32 {
        sys.factor_mut(f).work[0] = 2;
    }
}

/// C: `sub_021E32CC`: move 237, event 40: when var 2 is the holder, sets var 0x16 to the Hidden
/// Power type of its Pokemon (`fun_02018790`).
pub fn h_021e32cc<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let pp = b.mon(me as u32).get_pokemon();
        let t = fun_02018790(pp, &mut b.host);
        b.rewrite_var(0x16, t as i32);
    }
}

/// C: `sub_021E32F8`: move 237, event 55: when var 3 is the holder, sets var 0x30 to the Hidden
/// Power base power of its Pokemon (`fun_02018820`).
pub fn h_021e32f8<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let pp = b.mon(me as u32).get_pokemon();
        let t = fun_02018820(pp, &mut b.host);
        b.rewrite_var(0x30, t as i32);
    }
}

/// C: `sub_021E2D20`: move 217, event 55: when var 3 is the holder, sets var 0x30 to 0x28, 0x50 or
/// 0x78 by a roll of `rand(0x50)` (below 0x28 -> 0x28, above 0x45 -> 0x78, else 0x50).
pub fn h_021e2d20<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let r = b.calc.rand(0x50);
        let v = if r < 0x28 {
            0x28
        } else if r > 0x45 {
            0x78
        } else {
            0x50
        };
        b.rewrite_var(0x30, v);
    }
}

/// C: `FUN_overlay_d_93__021E5F18`: moves 312, 215 and 270, event 28: when var 3 is the holder,
/// sets var 0x51 to 1.
pub fn h_021e5f18<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        b.rewrite_var(0x51, 1);
    }
}

/// C: `FUN_overlay_d_93__021E5F30`: moves 312, 215 and 270, event 153: when var 3 is the holder,
/// sets var 0x42 to 0.
pub fn h_021e5f30<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        b.rewrite_var(0x42, 0);
    }
}

/// C: `FUN_overlay_d_93__021E5FEC`: move 270, event 118: when var 2 is the holder, removes the
/// factor.
pub fn h_021e5fec<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        sys.remove(f);
    }
}

/// C: `FUN_overlay_d_93__021E6420`: move 76, event 148: when var 3 is the holder and the weather
/// is 1, sets var 0x51 to 1.
pub fn h_021e6420<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.host.weather() == 1 {
        b.rewrite_var(0x51, 1);
    }
}

/// C: `FUN_overlay_d_93__021E3AC4`: move 203, event 116: when var 4 is the holder, work[6] is set
/// and var 0x51 is not 0, sets var 0x3A to 1.
pub fn h_021e3ac4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me) && sys.factor(f).work[6] != 0 && b.var(0x51) != 0 {
        b.rewrite_var(0x3A, 1);
    }
}

/// C: `FUN_overlay_d_93__021E7924`: moves 473, 540 and 548, event 58: when var 3 is the holder and
/// var 0x3C is 11, increments var 0x3D.
pub fn h_021e7924<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.var(0x3C) == 11 {
        let v = b.var(0x3D);
        b.rewrite_var(0x3D, v.wrapping_add(1));
    }
}

/// C: `FUN_overlay_d_93__021E1FCC`: moves 259, 432, 195, 193, 316 and 357, event 5: when var 3 is
/// the holder and var 0x12 equals the factor's sub ID (move ID), sets var 0x40 to 0.
pub fn h_021e1fcc<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.var(0x12) == sys.sub_id(f) as i32 {
        b.rewrite_var(0x40, 0);
    }
}

/// C: `FUN_overlay_d_93__021E36C8`: moves 168 and 343, event 129: when var 3 is the holder, sets
/// work[0] to 1 if its Pokemon holds an item, else 0.
pub fn h_021e36c8<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let held = b.mon(me as u32).held_item();
        sys.factor_mut(f).work[0] = (held != 0) as u32;
    }
}

/// C: `FUN_overlay_d_93__021E612C`: move 252, event 31: when var 2 is the holder and the Pokemon's
/// flag b bit 0 is set, sets var 0x22 to 0x1A.
pub fn h_021e612c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && b.mon(me as u32).flag_b(0) {
        b.rewrite_var(0x22, 0x1A);
    }
}

/// C: `FUN_overlay_d_93__021E8200`: moves 193 and 316, event 98: when var 3 is the holder, sets the
/// parameter (`SickCont::set_param_any`, 7) of the sick cont in var 0x1E.
pub fn h_021e8200<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let mut cont = SickCont(b.var(0x1E) as u32);
        cont.set_param_any(7);
        b.rewrite_var(0x1E, cont.0 as i32);
    }
}

/// C: `FUN_overlay_d_93__021E8238`: move 357, event 98: when var 3 is the holder, sets the
/// parameter (`SickCont::set_param_any`, 0x10) of the sick cont in var 0x1E.
pub fn h_021e8238<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let mut cont = SickCont(b.var(0x1E) as u32);
        cont.set_param_any(0x10);
        b.rewrite_var(0x1E, cont.0 as i32);
    }
}

/// C: `FUN_overlay_d_93__021E338C`: move 363, event 40: when var 2 is the holder, sets var 0x16 to
/// the item parameter 0xC of the held item (`item_param`).
pub fn h_021e338c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let item = b.mon(me as u32).held_item() as u32;
        let v = b.host.item_param(item, 0xC);
        b.rewrite_var(0x16, v as i32);
    }
}

/// C: `FUN_overlay_d_93__021E33BC`: move 363, event 55: when var 3 is the holder, sets var 0x30 to
/// the item parameter 0xB of the held item (`item_param`).
pub fn h_021e33bc<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let item = b.mon(me as u32).held_item() as u32;
        let v = b.host.item_param(item, 0xB);
        b.rewrite_var(0x30, v as i32);
    }
}

/// C: `FUN_overlay_d_93__021E5D24`: move 374, event 55: when var 3 is the holder, sets var 0x30 to
/// the item parameter 10 of the held item (`item_param`).
pub fn h_021e5d24<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let item = b.mon(me as u32).held_item() as u32;
        let v = b.host.item_param(item, 10);
        b.rewrite_var(0x30, v as i32);
    }
}

/// C: `FUN_overlay_d_93__021E7FB4`: move 507, event 147: when var 3 is the holder, var 4 is not
/// 0x1F (byte) and that Pokemon is on the holder's side, sets var 0x41 to 1.
pub fn h_021e7fb4<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let t = b.var(4) as u32 & 0xFF;
        if t != 0x1F && same_side(me as u32, t) {
            b.rewrite_var(0x41, 1);
        }
    }
}

/// C: `FUN_overlay_d_93__021E8270`: move 74, event 89: when var 3 is the holder and the weather is
/// 1, sets var 0x20 to 2 if it is 1 (otherwise rewrites it unchanged).
pub fn h_021e8270<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.host.weather() == 1 {
        let v = b.var(0x20);
        let v = if v == 1 { 2 } else { v };
        b.rewrite_var(0x20, v);
    }
}

/// C: `FUN_overlay_d_93__021E3D98`: move 117, event 31: when var 2 is the holder, work[0] is above
/// 1: sets var 0x22 to 0x1A if work[1] is 0 or work[2] is 0x1F, then sets work[0] to 3.
pub fn h_021e3d98<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && (sys.factor(f).work[0] as i32) > 1 {
        let w = sys.factor(f).work;
        if w[1] == 0 || w[2] == 0x1F {
            b.rewrite_var(0x22, 0x1A);
        }
        sys.factor_mut(f).work[0] = 3;
    }
}

/// C: `FUN_overlay_d_93__021E3E00`: move 117, event 75: when var 4 is the holder, var 0x46 is 0 and
/// work[6] is set, adds var 0x32 to work[1].
pub fn h_021e3e00<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me) && b.var(0x46) == 0 && sys.factor(f).work[6] != 0 {
        let add = b.var(0x32) as u32;
        let w = &mut sys.factor_mut(f).work;
        w[1] = w[1].wrapping_add(add);
    }
}

/// C: `FUN_overlay_d_93__021E3E30`: move 117, event 70: when var 3 is the holder and work[0] is
/// above 1, sets var 0x37 to work[1] * 2 (0 if work[1] is 0) and sets work[0] to 3.
pub fn h_021e3e30<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && (sys.factor(f).work[0] as i32) > 1 {
        b.mon(me as u32);
        let w1 = sys.factor(f).work[1];
        let v = if w1 != 0 { w1 << 1 } else { 0 };
        b.rewrite_var(0x37, v as i32);
        sys.factor_mut(f).work[0] = 3;
    }
}

/// C: `FUN_overlay_d_93__021E1C4C`: move 171, event 44: when var 3 is the holder and the Pokemon
/// named by var 4 does not have sick 2, sets var 0x40 to 1.
pub fn h_021e1c4c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let t = b.var(4) as u32 & 0xFF;
        if !b.mon(t).has_sick(2) {
            b.rewrite_var(0x40, 1);
        }
    }
}

/// C: `FUN_overlay_d_93__021E814C`: move 507, event 44: when var 3 is the holder and the Pokemon
/// named by var 4 has type 2, sets var 0x40 to 1.
pub fn h_021e814c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let t = b.var(4) as u32 & 0xFF;
        if b.mon(t).has_type(2) {
            b.rewrite_var(0x40, 1);
        }
    }
}

/// C: `FUN_overlay_d_93__021E2148`: move 99, event 1: when var 2 is the holder and work[6] is set,
/// removes this move's type-0 factors from the holder (`fun_021e03d8`).
pub fn h_021e2148<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && sys.factor(f).work[6] != 0 {
        let sid = sys.sub_id(f) as u32;
        fun_021e03d8(sys, b.mon(me as u32), sid);
    }
}

/// C: `FUN_overlay_d_93__021E2908`: move 263, event 56: when var 3 is the holder and the Pokemon's
/// major status is 1, 4 or 5, multiplies var 0x31 by 0x2000.
pub fn h_021e2908<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let s = b.mon(me as u32).major_status();
        if s == 5 || s == 1 || s == 4 {
            b.mul_var(0x31, 0x2000);
        }
    }
}

/// C: `FUN_overlay_d_93__021E1DCC`: moves 153, 120 and 515, event 69: when var 3 is the holder,
/// pushes HANDEX work 0x13 with the holder at +4 and pops it.
pub fn h_021e1dcc<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let w = b.host.handex_push(0x13, me as u32);
        b.host.handex(w).put_u8(4, me);
        b.host.handex_pop(w);
    }
}

/// C: `sub_021E1E10`: moves 153 and 120, event 39: when var 2 is the holder, pushes HANDEX work
/// 0x13 with the holder at +4 and pops it.
pub fn h_021e1e10<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let w = b.host.handex_push(0x13, me as u32);
        b.host.handex(w).put_u8(4, me);
        b.host.handex_pop(w);
    }
}

/// C: `sub_021E33EC`: move 363, event 39: when var 2 is the holder, pushes HANDEX work 0x23 with 1
/// at +4 and pops it.
pub fn h_021e33ec<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let w = b.host.handex_push(0x23, me as u32);
        b.host.handex(w).put_u32(4, 1);
        b.host.handex_pop(w);
    }
}

/// C: `FUN_overlay_d_93__021E249C`: move 253, event 4: when var 2 is the holder and var 0x51 is 0,
/// pushes HANDEX work 0xB (0x19 at +4, 1 at +0x14, the holder at +8) and pops it.
pub fn h_021e249c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && b.var(0x51) == 0 {
        let w = b.host.handex_push(0xB, me as u32);
        let buf = b.host.handex(w);
        buf.put_u32(4, 0x19);
        buf.put_u8(0x14, 1);
        buf.put_u8(8, me);
        b.host.handex_pop(w);
    }
}

/// C: `FUN_overlay_d_93__021E3BBC`: moves 203, 182, 197, 469 and 501, event 33: when var 2 is the
/// holder, pushes HANDEX work 0x26 (holder at +4, 3 at +5, 0 at +6), pops it and removes the
/// factor.
pub fn h_021e3bbc<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let w = b.host.handex_push(0x26, me as u32);
        let buf = b.host.handex(w);
        buf.put_u8(4, me);
        buf.put_u8(5, 3);
        buf.put_u8(6, 0);
        b.host.handex_pop(w);
        sys.remove(f);
    }
}

/// C: `FUN_overlay_d_93__021E56E0`: move 273, event 160: when var 3 is the holder, pushes HANDEX
/// work 0x1E (+4 = 0 as u32, +8 = the holder's battle position as a byte) and pops it.
pub fn h_021e56e0<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let w = b.host.handex_push(0x1E, me as u32);
        b.host.handex(w).put_u32(4, 0);
        let pos = b.host.poke_pos_in_battle(me as u32);
        b.host.handex(w).put_u8(8, pos as u8);
        b.host.handex_pop(w);
    }
}

/// C: `FUN_overlay_d_93__021E1F9C`: move 195, event 37: when var 2 is the holder, pushes HANDEX
/// work 4 and sets up its string parameter (type 1, string 0x77) at +4, then pops it.
pub fn h_021e1f9c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let w = b.host.handex_push(4, me as u32);
        str_param_setup(b.host.handex(w), 4, 1, 0x77);
        b.host.handex_pop(w);
    }
}

/// C: `FUN_overlay_d_93__021E2FB0`: move 311, event 55: when var 3 is the holder and the weather
/// is not 0, doubles var 0x30.
pub fn h_021e2fb0<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.host.weather() != 0 {
        let v = b.var(0x30);
        b.rewrite_var(0x30, v << 1);
    }
}

/// C: `FUN_overlay_d_93__021E6480`: move 76, event 56: when var 3 is the holder and the weather is
/// 2, 3 or 4, multiplies var 0x31 by 0x800.
pub fn h_021e6480<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && (b.host.weather().wrapping_add(0xFE) & 0xFF) < 3 {
        b.mul_var(0x31, 0x800);
    }
}

/// C: `FUN_overlay_d_93__021E6924`: move 107, event 36: when var 2 is the holder, pushes HANDEX
/// work 0x17 with 8 at +4 and the holder at +8, and pops it.
pub fn h_021e6924<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let w = b.host.handex_push(0x17, me as u32);
        let buf = b.host.handex(w);
        buf.put_u32(4, 8);
        buf.put_u8(8, me);
        b.host.handex_pop(w);
    }
}

/// C: `FUN_overlay_d_93__021E695C`: move 111, event 36: when var 2 is the holder, pushes HANDEX
/// work 0x17 with 7 at +4 and the holder at +8, and pops it.
pub fn h_021e695c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let w = b.host.handex_push(0x17, me as u32);
        let buf = b.host.handex(w);
        buf.put_u32(4, 7);
        buf.put_u8(8, me);
        b.host.handex_pop(w);
    }
}

/// C: `FUN_overlay_d_93__021E7C78`: move 515, event 70: when var 3 is the holder, sets var 0x37 to
/// the Pokemon's `BattleMon::get` value for id 0xD (`mon_get`).
pub fn h_021e7c78<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let v = b.mon_get(me as u32, 0xD);
        b.rewrite_var(0x37, v as i32);
    }
}

/// C: `FUN_overlay_d_93__021E5C64`: move 100, event 160: when var 3 is the holder, lets the factor run again
/// (`allow_reentry`) and pushes and pops HANDEX work 0x28 (`handex_push_pop`).
pub fn h_021e5c64<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        sys.allow_reentry(f);
        b.host.handex_push_pop(0x28, me as u32);
    }
}

/// C: shared shape of the handlers that push HANDEX work `ty` for `poke_id`, with a string
/// parameter at +4 (type 2, string `sid`) and one argument `arg`.
fn push_str_arg<H: FlowHost>(b: &mut Battle<H>, ty: u32, poke_id: u32, sid: u32, arg: u32) {
    let w = b.host.handex_push(ty, poke_id);
    let buf = b.host.handex(w);
    str_param_setup(buf, 4, 2, sid);
    str_param_add_arg(buf, 4, arg);
    b.host.handex_pop(w);
}

/// C: unlabeled routine at `0x021E2098`: when var 2 is the holder: if var 0x12 equals this
/// factor's sub ID it sets `work[6]`, else removes the holder's factors with that sub ID
/// (`021e03d8`).
pub fn h_021e2098<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let sub = sys.sub_id(f) as u32;
        if b.var(0x12) == sub as i32 {
            sys.set_work(f, 6, 1);
            return;
        }
        fun_021e03d8(sys, b.mon(me as u32), sub);
    }
}

/// 0x021E1BF8 (C: `sub_021e1bf8`): when var 2 is the holder: if var 0x22 is 2 it is cleared;
/// otherwise, unless the holder has sick 2, var 0x22 is set to 0x1A.
pub fn h_021e1bf8<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        if b.var(0x22) == 2 {
            b.rewrite_var(0x22, 0);
            return;
        }
        if !b.mon(me as u32).has_sick(2) {
            b.rewrite_var(0x22, 0x1A);
        }
    }
}

/// C: unlabeled routine at `0x021E2B5C`: when var 3 is the holder and the Pokemon in var 6 has
/// sick 2, pushes HANDEX work 0xB (+8 = var 6, +0x14 = 1, +4 = 2).
pub fn h_021e2b5c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let v6 = b.var(6) as u32;
        if b.mon(v6 & 0xFF).has_sick(2) {
            let w = b.host.handex_push(0xB, me as u32);
            let buf = b.host.handex(w);
            buf.put_u8(8, v6 as u8);
            buf.put_u8(0x14, 1);
            buf.put_u32(4, 2);
            b.host.handex_pop(w);
        }
    }
}

/// C: unlabeled routine at `0x021E2BF4`: when var 3 is the holder and var 6 is not 0x1F, and the
/// Pokemon in var 6 has sick 1, pushes HANDEX work 0xB (+8 = var 6, +0x14 = 1, +4 = 1).
pub fn h_021e2bf4<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let v6 = b.var(6) as u32;
        if v6 & 0xFF != 0x1F && b.mon(v6 & 0xFF).has_sick(1) {
            let w = b.host.handex_push(0xB, me as u32);
            let buf = b.host.handex(w);
            buf.put_u8(8, v6 as u8);
            buf.put_u8(0x14, 1);
            buf.put_u32(4, 1);
            b.host.handex_pop(w);
        }
    }
}

/// C: unlabeled routine at `0x021E314C`: when var 3 is the holder, multiplies var 0x35 by 0x2000
/// if the Pokemon in var 4 has flag B 5.
pub fn h_021e314c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let t = b.var(4) as u32 & 0xFF;
        if b.mon(t).flag_b(5) {
            b.mul_var(0x35, 0x2000);
        }
    }
}

/// C: unlabeled routine at `0x021E39C0`: as `h_021e314c` with flag B 4.
pub fn h_021e39c0<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let t = b.var(4) as u32 & 0xFF;
        if b.mon(t).flag_b(4) {
            b.mul_var(0x35, 0x2000);
        }
    }
}

/// C: unlabeled routine at `0x021E3A04`: as `h_021e314c` with flag B 8.
pub fn h_021e3a04<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let t = b.var(4) as u32 & 0xFF;
        if b.mon(t).flag_b(8) {
            b.mul_var(0x35, 0x2000);
        }
    }
}

/// C: unlabeled routine at `0x021E5D54`: when var 3 is the holder and the Pokemon has a held
/// item, pushes HANDEX work 4 (str type 2 at +4 with the holder and the item as arguments).
pub fn h_021e5d54<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let item = b.mon(me as u32).held_item() as u32;
        if item != 0 {
            let w = b.host.handex_push(4, me as u32);
            let buf = b.host.handex(w);
            str_param_setup(buf, 4, 2, 0x30B);
            str_param_add_arg(buf, 4, me as u32);
            str_param_add_arg(buf, 4, item);
            b.host.handex_pop(w);
        }
    }
}

/// C: unlabeled routine at `0x021E5E2C`: when var 2 is the holder and `work[0]` is 0, and the
/// holder has a held item, pushes HANDEX work 0x23 (+4 = 1).
pub fn h_021e5e2c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && sys.work(f, 0) == 0 && b.mon(me as u32).held_item() != 0 {
        let w = b.host.handex_push(0x23, me as u32);
        b.host.handex(w).put_u32(4, 1);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E5EB8`: when var 3 is the holder, pushes HANDEX work 0xC
/// (+0xF = holder, +4 = 0x1E, +8 = `SickCont::turn(5)`, str 0x292 at +0x14 with the holder).
pub fn h_021e5eb8<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let w = b.host.handex_push(0xC, me as u32);
        let buf = b.host.handex(w);
        buf.put_u8(0xF, me);
        buf.put_u32(4, 0x1E);
        buf.put_u32(8, SickCont::turn(5).0);
        str_param_setup(buf, 0x14, 2, 0x292);
        str_param_add_arg(buf, 0x14, me as u32);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E47CC`: when var 3 is the holder and the Pokemon in var 6 does
/// not have sick 0xC, pushes HANDEX work 0xC (+4 = 0xC, +8 = permanent sick cont, +0xF = var 6).
pub fn h_021e47cc<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let v6 = b.var(6) as u32;
        if !b.mon(v6 & 0xFF).has_sick(0xC) {
            let w = b.host.handex_push(0xC, me as u32);
            let buf = b.host.handex(w);
            buf.put_u32(4, 0xC);
            buf.put_u32(8, SickCont::permanent().0);
            buf.put_u8(0xF, v6 as u8);
            b.host.handex_pop(w);
        }
    }
}

/// C: unlabeled routine at `0x021E6164`: when var 2 is the holder, var 0x35 becomes 0xAAC for
/// weather 1 (sun), 0x400 for weather 2-4, else 0x800.
pub fn h_021e6164<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let v = match b.host.weather() {
            1 => 0xAAC,
            2..=4 => 0x400,
            _ => 0x800,
        };
        b.rewrite_var(0x35, v);
    }
}

/// C: unlabeled routine at `0x021E61BC`: when var 3 is the holder, pushes HANDEX work 0x17
/// (+8 = holder, +4 = 3) and then HANDEX work 4 (str 0x211 at +4 with the holder).
pub fn h_021e61bc<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let w = b.host.handex_push(0x17, me as u32);
        let buf = b.host.handex(w);
        buf.put_u8(8, me);
        buf.put_u32(4, 3);
        b.host.handex_pop(w);
        push_str_arg(b, 4, me as u32, 0x211, me as u32);
    }
}

/// C: unlabeled routine at `0x021E6220`: as `h_021e61bc` with HANDEX work 0x17 +4 = 6 and str
/// 0x21D.
pub fn h_021e6220<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let w = b.host.handex_push(0x17, me as u32);
        let buf = b.host.handex(w);
        buf.put_u8(8, me);
        buf.put_u32(4, 6);
        b.host.handex_pop(w);
        push_str_arg(b, 4, me as u32, 0x21D, me as u32);
    }
}

/// C: unlabeled routine at `0x021E6358`: as `h_021e61bc` with HANDEX work 0x17 +4 = 4 and str
/// 0x217.
pub fn h_021e6358<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let w = b.host.handex_push(0x17, me as u32);
        let buf = b.host.handex(w);
        buf.put_u8(8, me);
        buf.put_u32(4, 4);
        b.host.handex_pop(w);
        push_str_arg(b, 4, me as u32, 0x217, me as u32);
    }
}

/// C: unlabeled routine at `0x021E62F4`: as `h_021e61bc` with HANDEX work 0x17 +4 = 3 and str
/// 0x220.
pub fn h_021e62f4<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let w = b.host.handex_push(0x17, me as u32);
        let buf = b.host.handex(w);
        buf.put_u8(8, me);
        buf.put_u32(4, 3);
        b.host.handex_pop(w);
        push_str_arg(b, 4, me as u32, 0x220, me as u32);
    }
}

/// C: unlabeled routine at `0x021E6444`: when var 3 is the holder, pushes HANDEX work 4 (str
/// 0x229 at +4 with the holder).
pub fn h_021e6444<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        push_str_arg(b, 4, me as u32, 0x229, me as u32);
    }
}

/// C: unlabeled routine at `0x021E64B8`: when var 3 is the holder, pushes HANDEX work 4 (str
/// 0x223 at +4 with the holder).
pub fn h_021e64b8<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        push_str_arg(b, 4, me as u32, 0x223, me as u32);
    }
}

/// C: unlabeled routine at `0x021E6500`: when var 3 is the holder, pushes HANDEX work 4 (str
/// 0x226 at +4 with the holder).
pub fn h_021e6500<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        push_str_arg(b, 4, me as u32, 0x226, me as u32);
    }
}

/// C: unlabeled routine at `0x021E66CC`: when var 3 is the holder, pushes HANDEX work 4 (str
/// 0x272 at +4 with the holder) and sets `work[6]`.
pub fn h_021e66cc<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        push_str_arg(b, 4, me as u32, 0x272, me as u32);
        sys.set_work(f, 6, 1);
    }
}

/// C: unlabeled routine at `0x021E6800`: when var 3 is the holder, pushes HANDEX work 4 (str
/// 0x278 at +4 with the holder) and sets `work[6]`.
pub fn h_021e6800<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        push_str_arg(b, 4, me as u32, 0x278, me as u32);
        sys.set_work(f, 6, 1);
    }
}

/// C: unlabeled routine at `0x021E666C`: when var 3 is the holder, pushes HANDEX work 4 (str
/// 0x344 at +4 with the holder).
pub fn h_021e666c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        push_str_arg(b, 4, me as u32, 0x344, me as u32);
    }
}

/// C: unlabeled routine at `0x021E34FC`: when var 3 is the holder and `work[6]` is 0, pushes
/// HANDEX work 4 (str 0x2F9 at +4 with the holder) and sets `work[6]`.
pub fn h_021e34fc<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && sys.work(f, 6) == 0 {
        push_str_arg(b, 4, me as u32, 0x2F9, me as u32);
        sys.set_work(f, 6, 1);
    }
}

/// C: unlabeled routine at `0x021E55C4`: when var 3 is the holder, pushes HANDEX work 0x33 (+4
/// = var 6, str 0x284 at +8 with the holder and then the var 6 byte as arguments).
pub fn h_021e55c4<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let t = b.var(6) as u8;
        let w = b.host.handex_push(0x33, me as u32);
        let buf = b.host.handex(w);
        buf.put_u8(4, t);
        str_param_setup(buf, 8, 2, 0x284);
        str_param_add_arg(buf, 8, me as u32);
        str_param_add_arg(buf, 8, t as u32);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E6994`: when var 2 is the holder, pushes HANDEX work 0xC
/// (+0xF = holder, +0xD = 1, +4 = 0x18, +8 = `SickCont::turn_param(1, 2)` with flag 1 set).
pub fn h_021e6994<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let mut cont = SickCont::turn_param(1, 2);
        cont.set_flag(1);
        let w = b.host.handex_push(0xC, me as u32);
        let buf = b.host.handex(w);
        buf.put_u8(0xF, me);
        buf.put_u8(0xD, 1);
        buf.put_u32(4, 0x18);
        buf.put_u32(8, cont.0);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E6BAC`: when var 2 is the holder, sets the message parameter
/// (var 0x3F) to string 0x79 type 1 with the low 16 bits of var 0x12 as argument, and var 0x51
/// to 1.
pub fn h_021e6bac<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let v = b.var(0x12) as u32 & 0xFFFF;
        let buf = b.host.msg_param();
        str_param_setup(buf, 0, 1, 0x79);
        str_param_add_arg(buf, 0, v);
        b.rewrite_var(0x51, 1);
    }
}

/// C: unlabeled routine at `0x021E6AD8`: as `h_021e6bac` with string 0x78.
pub fn h_021e6ad8<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let v = b.var(0x12) as u32 & 0xFFFF;
        let buf = b.host.msg_param();
        str_param_setup(buf, 0, 1, 0x78);
        str_param_add_arg(buf, 0, v);
        b.rewrite_var(0x51, 1);
    }
}

/// C: unlabeled routine at `0x021E6FC8`: when var 3 is the holder and the Pokemon in var 4 has
/// sick 5, multiplies var 0x31 by 0x2000.
pub fn h_021e6fc8<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let t = b.var(4) as u32 & 0xFF;
        if b.mon(t).has_sick(5) {
            b.mul_var(0x31, 0x2000);
        }
    }
}

/// C: unlabeled routine at `0x021E75E0`: when var 3 is the holder, pushes HANDEX work 0x2E (+6
/// = var 6, +4 = 0x1FD + 0x8E if this factor's sub ID is 0x1FD else + 0x8F).
pub fn h_021e75e0<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let v6 = b.var(6) as u8;
        let sub = sys.sub_id(f);
        let v: u16 = if sub == 0x1FD {
            0x1FD + 0x8E
        } else {
            0x1FD + 0x8F
        };
        let w = b.host.handex_push(0x2E, me as u32);
        let buf = b.host.handex(w);
        buf.put_u8(6, v6);
        buf.put_u16(4, v);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E7CAC`: when var 3 is the holder, pushes HANDEX work 0x2F (+4 =
/// var 6, str 0x46E at +8 with the var 6 byte as argument).
pub fn h_021e7cac<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let t = b.var(6) as u8;
        let w = b.host.handex_push(0x2F, me as u32);
        let buf = b.host.handex(w);
        buf.put_u8(4, t);
        str_param_setup(buf, 8, 2, 0x46E);
        str_param_add_arg(buf, 8, t as u32);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E7D00`: as `h_021e7cac` with work 0x31 and str 0x46B.
pub fn h_021e7d00<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let t = b.var(6) as u8;
        let w = b.host.handex_push(0x31, me as u32);
        let buf = b.host.handex(w);
        buf.put_u8(4, t);
        str_param_setup(buf, 8, 2, 0x46B);
        str_param_add_arg(buf, 8, t as u32);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E82AC`: when var 3 is the holder, pushes HANDEX work 4 (str
/// 0x35F at +4 with the holder if this factor's sub ID is 0x229, else 0x362).
pub fn h_021e82ac<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let sid = if sys.sub_id(f) as u32 == 0x229 {
            0x35F
        } else {
            0x362
        };
        push_str_arg(b, 4, me as u32, sid, me as u32);
    }
}

/// C: unlabeled routine at `0x021E8440`: when var 3 is the holder and `work[0]` is nonzero,
/// pushes HANDEX work 4 (str 0xBB type 1 at +4).
pub fn h_021e8440<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && sys.work(f, 0) != 0 {
        let w = b.host.handex_push(4, me as u32);
        str_param_setup(b.host.handex(w), 4, 1, 0xBB);
        b.host.handex_pop(w);
    }
}

// ---------------------------------------------------------------------------------------------
// B6 batch 6
// ---------------------------------------------------------------------------------------------

/// C: `_s32_div_f` (`FUN_0209C0A4`): signed quotient truncating toward zero; a zero divisor
/// returns the dividend (the routine skips the loop and restores the sign).
fn s32_div(n: i32, d: i32) -> i32 {
    if d == 0 {
        n
    } else {
        n.wrapping_div(d)
    }
}

/// The Pokemon ID a stub that is passed a battle Pokemon pointer derives from it: the index
/// into the battle Pokemon array, 0xFF for the stand-in of an unknown ID.
fn mon_key<H: FlowHost>(b: &Battle<H>, id: u32) -> u32 {
    if (id as usize) < b.mons.len() {
        id
    } else {
        0xFF
    }
}

/// C: unlabeled routine at `0x021E3C5C(flow, poke_id, flag)`: pushes HANDEX work 0x26 (+4
/// `poke_id`, +5 = 3, +6 `u8_157(3) + 1`, bit 24 of the u32 at +0 = `flag & 1`).
pub fn fun_021e3c5c<H: FlowHost>(b: &mut Battle<H>, me: u32, flag: u32) {
    let n = b.mon(me).u8_157(3);
    let w = b.host.handex_push(0x26, me);
    let buf = b.host.handex(w);
    buf.put_u8(6, n.wrapping_add(1));
    buf.put_u8(4, me as u8);
    buf.put_u8(5, 3);
    let v = (flag & 1) << 24 | (buf.get_u32(0) & 0xFEFF_FFFF);
    buf.put_u32(0, v);
    b.host.handex_pop(w);
}

/// C: unlabeled routine at `0x021E3F08(flow, poke_id, work)`: when `work[6]` is set, pushes
/// HANDEX work 0xB (+4 = 0x19, +8 `poke_id`, +0x14 = +0x15 = 1) and clears `work[6]`.
pub fn fun_021e3f08<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u32) {
    if sys.work(f, 6) != 0 {
        let w = b.host.handex_push(0xB, me);
        let buf = b.host.handex(w);
        buf.put_u32(4, 0x19);
        buf.put_u8(0x14, 1);
        buf.put_u8(8, me as u8);
        buf.put_u8(0x15, 1);
        b.host.handex_pop(w);
        sys.set_work(f, 6, 0);
    }
}

/// C: unlabeled routine at `0x021E26C8(factor, flow, poke_id, work)`: pushes HANDEX work 0xB
/// (+8 `poke_id`, +0x14 = 1, +4 = 0x19), clears `work[6]` and removes the holder's factors with
/// this factor's sub ID (`fun_021e03f8`).
pub fn fun_021e26c8<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u32) {
    let w = b.host.handex_push(0xB, me);
    let buf = b.host.handex(w);
    buf.put_u8(8, me as u8);
    buf.put_u8(0x14, 1);
    buf.put_u32(4, 0x19);
    b.host.handex_pop(w);
    sys.set_work(f, 6, 0);
    let sub = sys.sub_id(f) as u32;
    fun_021e03f8(sys, me, sub);
}

/// C: unlabeled routine at `0x021E8860(x)`: var 0x30 = var 0x30 * `x` (plain multiply).
pub fn fun_021e8860<H: FlowHost>(b: &mut Battle<H>, x: i32) {
    let v = b.var(0x30);
    b.rewrite_var(0x30, v.wrapping_mul(x));
}

/// C: unlabeled routine at `0x021E556C(factor, flow, poke_id, _, p5, p6, p7, p8)`: when var 3
/// is the holder, pushes HANDEX work 0x19 (+4 `p6`, +0xC `p5`, +8 `p7`, str `p8` type 1 at
/// +0x10 with `p5` as its argument).
pub fn fun_021e556c<H: FlowHost>(b: &mut Battle<H>, me: u8, p5: u8, p6: u32, p7: u32, p8: u16) {
    if is_me(b, 3, me) {
        let w = b.host.handex_push(0x19, me as u32);
        let buf = b.host.handex(w);
        buf.put_u32(4, p6);
        buf.put_u8(0xC, p5);
        buf.put_u32(8, p7);
        str_param_setup(buf, 0x10, 1, p8 as u32);
        str_param_add_arg(buf, 0x10, p5 as u32);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E5784(factor, flow, poke_id, pos)`: when var 3 is the holder
/// and rewriting var 0x51 to 1 succeeds, pushes HANDEX work 0x1E (+4 = 3, +8 `pos` as a byte,
/// +0xC turn + 2, +0x10 sub ID, +0x1C = 2); a `pos` of 6 is first replaced by
/// `fun_021b8c20(rule, position of the holder, 0)`.
pub fn fun_021e5784<H: FlowHost>(
    sys: &mut EventSystem,
    f: FactorId,
    b: &mut Battle<H>,
    me: u8,
    pos: u32,
) {
    if is_me(b, 3, me) && b.rewrite_var(0x51, 1) {
        let mut pos = pos;
        if pos == 6 {
            let p = b.host.poke_pos_in_battle(me as u32);
            let rule = b.host.rule();
            pos = fun_021b8c20(rule, p, 0) as u32;
        }
        let w = b.host.handex_push(0x1E, me as u32);
        let turn = b.host.fun_021c8138();
        let buf = b.host.handex(w);
        buf.put_u32(4, 3);
        buf.put_u8(8, pos as u8);
        buf.put_u32(0xC, turn.wrapping_add(2));
        buf.put_u32(0x10, sys.sub_id(f) as u32);
        buf.put_u8(0x1C, 2);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E4A08(party, start, limit, out)`: appends the Pokemon IDs of
/// the party members with a major status (`021d62a8`) to `out` from index `start` while
/// `start < limit`; returns how many were added.
fn fun_021e4a08<H: FlowHost>(
    b: &mut Battle<H>,
    ids: &[u8],
    start: u32,
    limit: u32,
    out: &mut [u8],
) -> u8 {
    let mut n = 0u8;
    let mut at = start;
    for &id in ids {
        if limit <= at {
            return n;
        }
        if b.mon(id as u32).major_status() != 0 {
            out[at as usize] = b.mon(id as u32).poke_id();
            at = (at + 1) & 0xFF;
            n = n.wrapping_add(1);
        }
    }
    n
}

/// C: unlabeled routine at `0x021E4958(factor, flow, poke_id, _, str)`: when var 3 is the
/// holder, pushes HANDEX work 4 (str `str` type 1) and then, if any member of the holder's
/// party (and of the partner party) has a major status, HANDEX work 0xB (+0x14 = count (at most
/// 12), +8.. their Pokemon IDs, +4 = 0x24).
pub fn fun_021e4958<H: FlowHost>(b: &mut Battle<H>, me: u8, str_id: u16) {
    if !is_me(b, 3, me) {
        return;
    }
    let m = me as u32;
    let mut p1 = [0u8; 8];
    let n1 = b.host.party_ids(m, &mut p1) as usize;
    let mut p2 = [0u8; 8];
    let n2 = b.host.partner_party_ids(m, &mut p2);
    let mut scratch = [0u8; 12];
    let w = b.host.handex_push(4, m);
    str_param_setup(b.host.handex(w), 4, 1, str_id as u32);
    b.host.handex_pop(w);
    let mut total = fun_021e4a08(b, &p1[..n1.min(8)], 0, 12, &mut scratch) as u32;
    if let Some(n2) = n2 {
        let k = fun_021e4a08(b, &p2[..(n2 as usize).min(8)], total, 12, &mut scratch) as u32;
        total = (total + k) & 0xFF;
    }
    if total != 0 {
        let total = total.min(12) as usize;
        let w = b.host.handex_push(0xB, m);
        let buf = b.host.handex(w);
        buf.put_u8(0x14, total as u8);
        buf[8..8 + total].copy_from_slice(&scratch[..total]);
        buf.put_u32(4, 0x24);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E1C8C`: when var 3 is the holder, `t` = var 4 as a byte: if
/// `t` has no flag A 3 and its queued action is a fight whose move passes
/// `move_fun_0201c314`, nothing happens, else when rewriting var 0x40 to 1 succeeds sets the
/// message parameter (var 0x3F) to string 0x47 type 1.
pub fn h_021e1c8c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    use crate::calc_action::ActionRecord;
    if !is_me(b, 3, me) {
        return;
    }
    let t = b.var(4) as u32 & 0xFF;
    let mut ok = false;
    let mut act = 0u32;
    if !b.mon(t).flag_a(3) && b.host.find_action(t, &mut act) {
        let w = ActionRecord(act).fight_waza();
        if w != 0 && b.host.move_fun_0201c314(w) {
            ok = true;
        }
    }
    if ok {
        return;
    }
    let w = ActionRecord(act).fight_waza();
    if (w == 0 || !b.host.move_fun_0201c314(w)) && b.rewrite_var(0x40, 1) {
        let buf = b.host.msg_param();
        str_param_setup(buf, 0, 1, 0x47);
    }
}

/// C: unlabeled routine at `0x021E1D20`: when the holder has a queued action: `t` = var 6 as a
/// byte; if `t` is not on the holder's side and `fun_021c8290(holder, t, 0)` is false, appends
/// the holder to the flow's list (`fun_021c831c`).
pub fn h_021e1d20<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    let m = me as u32;
    let mut act = 0u32;
    if b.host.find_action(m, &mut act) {
        let t = b.var(6) as u32 & 0xFF;
        if !same_side(t, m) {
            b.host.poke_pos_in_battle(t);
            b.host.poke_pos_in_battle(m);
            if !b.host.fun_021c8290(m, t, 0) {
                b.host.fun_021c831c(m);
            }
        }
    }
}

/// C: unlabeled routine at `0x021E1E44`: when var 3 is the holder and its flag B 9 is clear,
/// pushes HANDEX work 0x17 (+8 holder, +4 = 9) and work 4 (str 0x411 at +4 with the holder).
pub fn h_021e1e44<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && !b.mon(me as u32).flag_b(9) {
        let w = b.host.handex_push(0x17, me as u32);
        let buf = b.host.handex(w);
        buf.put_u8(8, me);
        buf.put_u32(4, 9);
        b.host.handex_pop(w);
        push_str_arg(b, 4, me as u32, 0x411, me as u32);
    }
}

/// C: unlabeled routine at `0x021E1EB8`: when var 2 is the holder and var 0x12 is this
/// factor's sub ID: pushes HANDEX work 4 (str 0x298 at +4 with the holder), sets var 0x3E to 1
/// and `work[6]`, `work[0]` to 1.
pub fn h_021e1eb8<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && b.var(0x12) == sys.sub_id(f) as i32 {
        push_str_arg(b, 4, me as u32, 0x298, me as u32);
        b.rewrite_var(0x3E, 1);
        sys.set_work(f, 6, 1);
        sys.set_work(f, 0, 1);
    }
}

/// C: unlabeled routine at `0x021E203C`: when var 3 is the holder, with `v` = `get(0xF)` of the
/// holder: if `fun_021c83e8(v * 5)` holds pushes HANDEX work 4 (str 0x7A type 1 at +4).
pub fn h_021e203c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let v = b.mon_get(me as u32, 0xF);
        if b.host.fun_021c83e8(v.wrapping_mul(5)) {
            let w = b.host.handex_push(4, me as u32);
            str_param_setup(b.host.handex(w), 4, 1, 0x7A);
            b.host.handex_pop(w);
        }
    }
}

/// C: unlabeled routine at `0x021E20E0`: when var 4 is the holder, var 0x46 is 0 and the
/// holder has not fainted: pushes HANDEX work 0xE (+4 = 1, +0xC = 1, +0xE = 0, +0xF = 1,
/// +0x10 holder, +0xD = 1, str 0x214 at +0x18 with the holder).
pub fn h_021e20e0<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me) && b.var(0x46) == 0 && !b.mon_is_fainted(me as u32) {
        let w = b.host.handex_push(0xE, me as u32);
        let buf = b.host.handex(w);
        buf.put_u32(4, 1);
        buf.put_u8(0xC, 1);
        buf.put_u8(0xE, 0);
        buf.put_u8(0xF, 1);
        buf.put_u8(0x10, me);
        buf.put_u8(0xD, 1);
        str_param_setup(buf, 0x18, 2, 0x214);
        str_param_add_arg(buf, 0x18, me as u32);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E2188`: when var 3 is the holder and it lacks condition 0x23:
/// pushes HANDEX work 0xC (+4 = 0x23, +8 permanent, +0xF holder, str 0x259 at +0x14 with the
/// holder).
pub fn h_021e2188<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && !b.mon(me as u32).has_sick(0x23) {
        let w = b.host.handex_push(0xC, me as u32);
        let buf = b.host.handex(w);
        buf.put_u32(4, 0x23);
        buf.put_u32(8, SickCont::permanent().0);
        buf.put_u8(0xF, me);
        str_param_setup(buf, 0x14, 2, 0x259);
        str_param_add_arg(buf, 0x14, me as u32);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E21F4`: when var 2 is the holder, it lacks condition 0x19 and
/// `work[6]` is 0: `n` = `rand(2) + 2`; pushes HANDEX work 0xC (+4 = 0x19, +8
/// `turn_param(n, sub ID)`, +0xC = 0, +0xF holder), sets `work[6]` = 1 and `work[0]` = `n`.
pub fn h_021e21f4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && !b.mon(me as u32).has_sick(0x19) && sys.work(f, 6) == 0 {
        let n = (b.calc.rand(2) + 2) & 0xFF;
        let w = b.host.handex_push(0xC, me as u32);
        let buf = b.host.handex(w);
        buf.put_u32(4, 0x19);
        buf.put_u32(8, SickCont::turn_param(n, sys.sub_id(f) as u32).0);
        buf.put_u8(0xC, 0);
        buf.put_u8(0xF, me);
        b.host.handex_pop(w);
        sys.set_work(f, 6, 1);
        sys.set_work(f, 0, n);
    }
}

/// C: unlabeled routine at `0x021E2268`: when var 2 is the holder and `work[6]` is set:
/// decrements `work[0]` if nonzero; when var 0x51 is 0 pushes HANDEX work 0xB (+4 = 0x19, +0x14
/// = 1, +8 holder); when `work[0]` is now 0 pushes work 0xC (+4 = 6, +8 `make_sick_cont(6)`,
/// +0xD = 1, +0xF holder, str 0x168 at +0x14 with the holder); then if either held, removes the
/// holder's factors with this sub ID.
pub fn h_021e2268<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) || sys.work(f, 6) == 0 {
        return;
    }
    let m = me as u32;
    let w0 = sys.work(f, 0);
    if w0 != 0 {
        sys.set_work(f, 0, w0 - 1);
    }
    let v51 = b.var(0x51);
    if v51 == 0 {
        let w = b.host.handex_push(0xB, m);
        let buf = b.host.handex(w);
        buf.put_u32(4, 0x19);
        buf.put_u8(0x14, 1);
        buf.put_u8(8, me);
        b.host.handex_pop(w);
    }
    let n = sys.work(f, 0);
    if n == 0 {
        let w = b.host.handex_push(0xC, m);
        let cont = b.make_sick_cont(6, m);
        let buf = b.host.handex(w);
        buf.put_u32(4, 6);
        buf.put_u32(8, cont.0);
        buf.put_u8(0xD, 1);
        buf.put_u8(0xF, me);
        str_param_setup(buf, 0x14, 2, 0x168);
        str_param_add_arg(buf, 0x14, m);
        b.host.handex_pop(w);
    }
    if n == 0 || v51 == 0 {
        let sub = sys.sub_id(f) as u32;
        fun_021e03f8(sys, m, sub);
    }
}

/// C: unlabeled routine at `0x021E231C`: when var 2 is the holder and it has condition 2:
/// pushes HANDEX work 0xB (+4 = 0x19, +0x14 = 1, +8 holder) and removes the holder's factors
/// with this sub ID.
pub fn h_021e231c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && b.mon(me as u32).has_sick(2) {
        let w = b.host.handex_push(0xB, me as u32);
        let buf = b.host.handex(w);
        buf.put_u32(4, 0x19);
        buf.put_u8(0x14, 1);
        buf.put_u8(8, me);
        b.host.handex_pop(w);
        let sub = sys.sub_id(f) as u32;
        fun_021e03f8(sys, me as u32, sub);
    }
}

/// C: unlabeled routine at `0x021E24D4`: when var 2 is the holder, `work[6]` is set and the
/// holder has not fainted: if it lacks condition 0x19 pushes HANDEX work 4 (str 0x2CE at +4
/// with the holder) and removes the holder's factors with this sub ID, else pushes work 4 (str
/// 0x2CB with the holder).
pub fn h_021e24d4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) || sys.work(f, 6) == 0 {
        return;
    }
    let m = me as u32;
    if b.mon_is_fainted(m) {
        return;
    }
    if !b.mon(m).has_sick(0x19) {
        push_str_arg(b, 4, m, 0x2CE, m);
        let sub = sys.sub_id(f) as u32;
        fun_021e03f8(sys, m, sub);
    } else {
        push_str_arg(b, 4, m, 0x2CB, m);
    }
}

/// C: unlabeled routine at `0x021E2574`: when `work[6]` is set, var 0x1D is 2 and rewriting
/// var 0x41 to 1 succeeds: `t` = var 4 as a byte; pushes HANDEX work 4 (str 0x2C8 if `t` is the
/// holder else 0x2C5, at +4 with `t`).
pub fn h_021e2574<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if sys.work(f, 6) != 0 && b.var(0x1D) == 2 && b.rewrite_var(0x41, 1) {
        let t = b.var(4) as u32 & 0xFF;
        let sid = if t == me as u32 { 0x2C8 } else { 0x2C5 };
        push_str_arg(b, 4, me as u32, sid, t);
    }
}

/// C: unlabeled routine at `0x021E25FC`: when var 2 is the holder, it has neither condition
/// 0x19 nor 2 and `work[6]` is 0: pushes HANDEX work 0xC (+4 = 0x19, +8 `turn_param(5, sub
/// ID)`, +0xC = 0, +0xF holder) and sets `work[0]` = 0, `work[1]` = 5, `work[6]` = 1.
pub fn h_021e25fc<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let mon = b.mon(me as u32);
        if !mon.has_sick(0x19) && !mon.has_sick(2) && sys.work(f, 6) == 0 {
            let w = b.host.handex_push(0xC, me as u32);
            let buf = b.host.handex(w);
            buf.put_u32(4, 0x19);
            buf.put_u32(8, SickCont::turn_param(5, sys.sub_id(f) as u32).0);
            buf.put_u8(0xC, 0);
            buf.put_u8(0xF, me);
            b.host.handex_pop(w);
            sys.set_work(f, 0, 0);
            sys.set_work(f, 1, 5);
            sys.set_work(f, 6, 1);
        }
    }
}

/// C: unlabeled routine at `0x021E27B0`: when var 3 is the holder, with `a` / `c` the
/// `fun_021c81c8` values of the holder and of var 4: sets var 0x30 to `min(c * 25 / a + 1, 150)`.
pub fn h_021e27b0<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let t = b.var(4) as u32 & 0xFF;
        let ia = mon_key(b, me as u32);
        let ic = mon_key(b, t);
        let a = b.host.fun_021c81c8(ia, 0);
        let c = b.host.fun_021c81c8(ic, 0);
        let q = s32_div((c as i32).wrapping_mul(0x19), a as i32);
        let v = (q as u32).wrapping_add(1).min(150);
        b.rewrite_var(0x30, v as i32);
    }
}

/// C: unlabeled routine at `0x021E2898`: when var 3 is the holder: `q` = `get(0xD) * 0x30 /
/// get(0xE)` (at least 1), var 0x30 = the power from the table at 0x021F2CA8
/// (`(limit, power)`: 1/200, 4/150, 9/100, 16/80, 32/40, else 20).
pub fn h_021e2898<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let m = me as u32;
        let a = b.mon_get(m, 0xD) as i32;
        let c = b.mon_get(m, 0xE) as i32;
        let mut q = s32_div(a.wrapping_mul(0x30), c) as u32;
        if q == 0 {
            q = 1;
        }
        const T: [(u32, i32); 6] = [(1, 200), (4, 150), (9, 100), (16, 80), (32, 40), (0, 20)];
        let mut i = 0;
        while i < 5 && q > T[i].0 {
            i += 1;
        }
        b.rewrite_var(0x30, T[i].1);
    }
}

/// C: unlabeled routine at `0x021E2988`: when var 3 is the holder: var 0x30 =
/// `max(get(0xD) * power(sub ID) / get(0xE), 1)`.
pub fn h_021e2988<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let m = me as u32;
        let a = b.mon_get(m, 0xD);
        let c = b.mon_get(m, 0xE);
        let p = b.host.move_power(sys.sub_id(f) as u32);
        let mut v = u32_div(a.wrapping_mul(p), c);
        if v == 0 {
            v = 1;
        }
        b.rewrite_var(0x30, v as i32);
    }
}

/// C: unlabeled routine at `0x021E2C4C`: when var 3 is the holder: on a 20% roll rewrites var
/// 0x51 to 1 and stores the result in `work[0]`; then `fun_021c83dc(work[0] != 0)`.
pub fn h_021e2c4c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        if b.calc.roll_percent(0x14) {
            let r = b.rewrite_var(0x51, 1);
            sys.set_work(f, 0, r as u32);
        }
        let v = sys.work(f, 0) != 0;
        b.host.fun_021c83dc(v as u32);
    }
}

/// C: unlabeled routine at `0x021E2C88`: when var 3 is the holder and `work[0]` is set: `t` =
/// var 4: if `t` is not at full HP pushes HANDEX work 5 (+6 var 4, +4 max HP / 4 (at least 1)
/// of `t`, str 0x183 at +8 with `t`), else pushes work 4 (str 0xD2 at +4 with `t`).
pub fn h_021e2c88<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) || sys.work(f, 0) == 0 {
        return;
    }
    let v4 = b.var(4) as u32;
    let t = v4 & 0xFF;
    if !b.mon_is_full_hp(t) {
        let hp = b.max_hp_div_min1(t, 4);
        let w = b.host.handex_push(5, me as u32);
        let buf = b.host.handex(w);
        buf.put_u8(6, v4 as u8);
        buf.put_u16(4, hp as u16);
        str_param_setup(buf, 8, 2, 0x183);
        str_param_add_arg(buf, 8, t);
        b.host.handex_pop(w);
    } else {
        push_str_arg(b, 4, me as u32, 0xD2, t);
    }
}

/// C: unlabeled routine at `0x021E2D5C`: when var 3 is the holder: var 0x30 = 0x28 if the
/// holder does not know this factor's move, else the power from the table at 0x021F2860
/// (200, 80, 60, 50, 40) indexed by the move's remaining PP (at most 4).
pub fn h_021e2d5c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let sub = sys.sub_id(f) as u32;
        let slot = b.mon(me as u32).move_slot_of(sub);
        let v = if slot == 4 {
            0x28
        } else {
            const T: [i32; 5] = [200, 80, 60, 50, 40];
            let pp = b.mon(me as u32).move_pp(slot as u32).min(4);
            T[pp as usize]
        };
        b.rewrite_var(0x30, v);
    }
}

/// C: unlabeled routine at `0x021E2DC0`: when var 3 is the holder: `s` = the sum over values
/// 1..=7 of the Pokemon in var 4 (`get`) of `v - 6` where `v > 6`; var 0x30 = `min(s * 20 +
/// 60, 200)`.
pub fn h_021e2dc0<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let t = b.var(4) as u32 & 0xFF;
        let mut s = 0i32;
        for id in 1..=7u32 {
            let v = b.mon_get(t, id) as i32;
            if v > 6 {
                s += v - 6;
            }
        }
        let v = ((s as u32).wrapping_mul(20).wrapping_add(60)).min(200);
        b.rewrite_var(0x30, v as i32);
    }
}

/// C: unlabeled routine at `0x021E2E28`: when var 3 is the holder and its last move is this
/// factor's sub ID: var 0x30 is doubled once per use in its move streak, capped at 0xA0.
pub fn h_021e2e28<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let mon = b.mon(me as u32);
        if mon.last_move() as u32 == sys.sub_id(f) as u32 {
            let streak = mon.move_streak() as u32;
            let mut v = b.var(0x30) as u32;
            let mut i = 0u32;
            if streak != 0 {
                loop {
                    v = v.wrapping_mul(2);
                    if v > 0xA0 {
                        v = 0xA0;
                        break;
                    }
                    i += 1;
                    if i >= streak {
                        break;
                    }
                }
            }
            b.rewrite_var(0x30, v as i32);
        }
    }
}

/// C: unlabeled routine at `0x021E2ED4`: when var 3 is the holder: var 0x30 from the weight
/// `w` = `fun_021c83a0` of the Pokemon in var 4: 120 (>= 2000), 100 (>= 1000), 80 (>= 500), 60
/// (>= 250), 40 (>= 100), else 20.
pub fn h_021e2ed4<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let t = b.var(4) as u32 & 0xFF;
        let id = b.mon(t).poke_id() as u32;
        let w = b.host.fun_021c83a0(id);
        let v = if w >= 2000 {
            0x78
        } else if w >= 1000 {
            100
        } else if w >= 500 {
            0x50
        } else if w >= 0xFA {
            0x3C
        } else if w >= 100 {
            0x28
        } else {
            0x14
        };
        b.rewrite_var(0x30, v);
    }
}

/// C: unlabeled routine at `0x021E3190`: when var 2 is the holder and `fun_021c824c` holds for
/// it: var 0x16 = a type chosen by its held item (items 0x12A..=0x139 in table order: 9, 10,
/// 12, 11, 14, 1, 3, 4, 2, 13, 6, 5, 7, 15, 16, 8), 0 for other items.
pub fn h_021e3190<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && b.host.fun_021c824c(me as u32) {
        let item = b.mon(me as u32).held_item() as u32;
        const T: [i32; 16] = [9, 10, 12, 11, 14, 1, 3, 4, 2, 13, 6, 5, 7, 15, 16, 8];
        let i = item.wrapping_sub(0x12A);
        let v = if i < 16 { T[i as usize] } else { 0 };
        b.rewrite_var(0x16, v);
    }
}

/// C: unlabeled routine at `0x021E324C`: when var 2 is the holder and `fun_021c824c` holds for
/// it: by held item 0x74 / 0x75 / 0x76 / 0x77 sets var 0x16 to 10 / 12 / 9 / 14 (else 0) and
/// calls `fun_021c83dc` with 1 / 2 / 3 / 4 (else 0).
pub fn h_021e324c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && b.host.fun_021c824c(me as u32) {
        let (v, k) = match b.mon(me as u32).held_item() {
            0x74 => (10, 1),
            0x75 => (12, 2),
            0x76 => (9, 3),
            0x77 => (14, 4),
            _ => (0, 0),
        };
        b.rewrite_var(0x16, v);
        b.host.fun_021c83dc(k);
    }
}

/// C: unlabeled routine at `0x021E3330`: when var 2 is the holder: if `fun_021c824c` fails for
/// it sets var 0x22 to 0x1A and stops; otherwise, with the held item `i` (whose item parameter
/// 0xB is read once and discarded), sets var 0x22 to 0x1A if `i` is 0 or its parameter 0xB is
/// 0.
pub fn h_021e3330<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        if !b.host.fun_021c824c(me as u32) {
            b.rewrite_var(0x22, 0x1A);
            return;
        }
        let item = b.mon(me as u32).held_item() as u32;
        b.host.item_param(item, 0xB);
        if item == 0 || b.host.item_param(item, 0xB) == 0 {
            b.rewrite_var(0x22, 0x1A);
        }
    }
}

/// C: unlabeled routine at `0x021E35F0`: when var 3 is not the holder and the move in var 0x12
/// has flag 5: if the holder lacks condition 0x21 and rewriting var 2 to the holder succeeds,
/// rewrites var 4 to the holder and sets `work[0]` to 1.
pub fn h_021e35f0<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if (b.var(3) as u32 & 0xFF) != me as u32 {
        let mv = b.var(0x12) as u32 & 0xFFFF;
        if b.host.move_flag(mv, 5)
            && !b.mon(me as u32).has_sick(0x21)
            && b.rewrite_var(2, me as i32)
        {
            b.rewrite_var(4, me as i32);
            sys.set_work(f, 0, 1);
        }
    }
}

/// C: unlabeled routine at `0x021E3648`: when var 2 is the holder and `work[0]` is set: pushes
/// HANDEX work 4 (str 0x2F2 at +4 with the holder and var 3) and removes this factor.
pub fn h_021e3648<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && sys.work(f, 0) != 0 {
        let v3 = b.var(3) as u32;
        let w = b.host.handex_push(4, me as u32);
        let buf = b.host.handex(w);
        str_param_setup(buf, 4, 2, 0x2F2);
        str_param_add_arg(buf, 4, me as u32);
        str_param_add_arg(buf, 4, v3);
        b.host.handex_pop(w);
        sys.remove(f);
    }
}

/// C: unlabeled routine at `0x021E3904`: when var 2 is the holder: `r` = `rand(100)`; `i` = the
/// first index of the thresholds 5, 15, 35, 65, 85, 95, 100 above `r`; `work[0]` = the value
/// 10, 30, 50, 70, 90, 110, 150 (0 past the end) and HANDEX work 4 (str `0x68 + i` type 1 at
/// +4) is pushed.
pub fn h_021e3904<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let r = b.calc.rand(100) as u8;
        const T: [(u8, u8); 8] = [
            (5, 10),
            (15, 30),
            (35, 50),
            (65, 70),
            (85, 90),
            (95, 110),
            (100, 150),
            (0, 0),
        ];
        let mut i = 0usize;
        loop {
            if r < T[i].0 {
                break;
            }
            i += 1;
            if i >= 7 {
                break;
            }
        }
        sys.set_work(f, 0, T[i].1 as u32);
        let w = b.host.handex_push(4, me as u32);
        str_param_setup(b.host.handex(w), 4, 1, 0x68 + i as u32);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E3B14`: when var 3 is the holder and its last move is none of
/// 0xB6, 0xC5, 0xCB, 0x1D5, 0x1F5: pushes HANDEX work 0x26 (+4 holder, +5 = 3, +6 = 0).
pub fn h_021e3b14<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let mv = b.mon(me as u32).last_move();
        if [0xB6u16, 0xC5, 0xCB, 0x1D5, 0x1F5].contains(&mv) {
            return;
        }
        let w = b.host.handex_push(0x26, me as u32);
        let buf = b.host.handex(w);
        buf.put_u8(4, me);
        buf.put_u8(5, 3);
        buf.put_u8(6, 0);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E3D38`: when var 2 is the holder: with `work[0]` 1 (2) sets
/// the message parameter (var 0x3F) to string 0x2E9 (0x2EC) type 2 with the holder as argument
/// and var 0x51 to 1.
pub fn h_021e3d38<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let sid = match sys.work(f, 0) {
            1 => 0x2E9,
            2 => 0x2EC,
            _ => return,
        };
        let buf = b.host.msg_param();
        str_param_setup(buf, 0, 2, sid);
        str_param_add_arg(buf, 0, me as u32);
        b.rewrite_var(0x51, 1);
    }
}

/// C: unlabeled routine at `0x021E3F44`: when var 3 is the holder with no held item and a
/// nonzero `u14`: pushes HANDEX work 0x20 (+4 `u14`, +6 holder, +7 = 1, str 0x2DD at +0xC with
/// the holder and `u14`).
pub fn h_021e3f44<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let mon = b.mon(me as u32);
        if mon.held_item() == 0 {
            let v = mon.u14();
            if v != 0 {
                let w = b.host.handex_push(0x20, me as u32);
                let buf = b.host.handex(w);
                buf.put_u16(4, v);
                buf.put_u8(6, me);
                buf.put_u8(7, 1);
                str_param_setup(buf, 0xC, 2, 0x2DD);
                str_param_add_arg(buf, 0xC, me as u32);
                str_param_add_arg(buf, 0xC, v as u32);
                b.host.handex_pop(w);
            }
        }
    }
}

/// C: unlabeled routine at `0x021E3FC0`: when var 3 is the holder with a major status `s` and
/// the Pokemon `t` in var 6 has none: pushes HANDEX work 0xC (+4 `s`, +8 badly-poisoned if `s`
/// is 5 and the holder's condition is, else the default duration, +0xF `t`, +0xC = 1), work
/// 0xB (+4 `s`, +0x14 = 1, +8 holder, bit 24 set) and sets var 0x51 to 0.
pub fn h_021e3fc0<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let s = b.mon(me as u32).major_status();
    if s == 0 {
        return;
    }
    let v6 = b.var(6) as u32;
    if b.mon(v6 & 0xFF).major_status() != 0 {
        return;
    }
    let held = b.mon(me as u32).get_sick_cont(s);
    let cont = if s == 5 && held.is_badly_poisoned() {
        SickCont::badly_poisoned()
    } else {
        b.calc.make_default_sick_cont(s)
    };
    let w = b.host.handex_push(0xC, me as u32);
    let buf = b.host.handex(w);
    buf.put_u32(4, s);
    buf.put_u32(8, cont.0);
    buf.put_u8(0xF, v6 as u8);
    buf.put_u8(0xC, 1);
    b.host.handex_pop(w);
    let w = b.host.handex_push(0xB, me as u32);
    let buf = b.host.handex(w);
    buf.put_u32(4, s);
    buf.put_u8(0x14, 1);
    buf.put_u8(8, me);
    let h = buf.get_u32(0) | 0x100_0000;
    buf.put_u32(0, h);
    b.host.handex_pop(w);
    b.rewrite_var(0x51, 0);
}

/// C: unlabeled routine at `0x021E4080`: when var 3 is the holder: `t` = var 6, `a` / `c` =
/// `get(0xD)` of the holder / `t`, `m` = `(a + c) / 2`: pushes HANDEX work 8 (+4 = 2, +7
/// holder, +0x10 `m - a`, +8 `t`, +0x14 `m - c`, +6 = 1), work 4 (str 0x71 type 1), and work
/// 0x21 for the holder and for `t` (+4 the Pokemon, +8 = 1).
pub fn h_021e4080<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let m = me as u32;
    let v6 = b.var(6) as u32;
    let a = b.mon_get(m, 0xD);
    let c = b.mon_get(v6 & 0xFF, 0xD);
    let avg = a.wrapping_add(c) >> 1;
    let w = b.host.handex_push(8, m);
    let buf = b.host.handex(w);
    buf.put_u8(4, 2);
    buf.put_u8(7, me);
    buf.put_u32(0x10, avg.wrapping_sub(a));
    buf.put_u8(8, v6 as u8);
    buf.put_u32(0x14, avg.wrapping_sub(c));
    buf.put_u8(6, 1);
    b.host.handex_pop(w);
    let w = b.host.handex_push(4, m);
    str_param_setup(b.host.handex(w), 4, 1, 0x71);
    b.host.handex_pop(w);
    for who in [me, v6 as u8] {
        let w = b.host.handex_push(0x21, m);
        let buf = b.host.handex(w);
        buf.put_u8(4, who);
        buf.put_u32(8, 1);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E4154`: when var 3 is the holder, with `h` = max HP / 2 (at
/// least 1) and `r` = `stage_headroom(1)`: if `h < get(0xD)` and `r != 0`: pushes HANDEX work 8
/// (+4 = 1, +6 = 1, +7 holder, +0x10 `-h`), work 0xE (+4 = 1, +0xC `r`, +0xF = 1, +0x10
/// holder, +0xD = 1, +0xE = 1), work 4 (str 0x265 at +4 with the holder) and work 0x21 (+4
/// holder, +8 = 1).
pub fn h_021e4154<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let m = me as u32;
    let h = b.max_hp_div_min1(m, 2);
    let r = b.mon(m).stage_headroom(1);
    let cur = b.mon_get(m, 0xD);
    if h < cur && r != 0 {
        let w = b.host.handex_push(8, m);
        let buf = b.host.handex(w);
        buf.put_u8(4, 1);
        buf.put_u8(6, 1);
        buf.put_u8(7, me);
        buf.put_u32(0x10, h.wrapping_neg());
        b.host.handex_pop(w);
        let w = b.host.handex_push(0xE, m);
        let buf = b.host.handex(w);
        buf.put_u32(4, 1);
        buf.put_u8(0xC, r as u8);
        buf.put_u8(0xF, 1);
        buf.put_u8(0x10, me);
        buf.put_u8(0xD, 1);
        buf.put_u8(0xE, 1);
        b.host.handex_pop(w);
        push_str_arg(b, 4, m, 0x265, m);
        let w = b.host.handex_push(0x21, m);
        let buf = b.host.handex(w);
        buf.put_u8(4, me);
        buf.put_u32(8, 1);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E4238`: when var 3 is the holder and either the Pokemon `t`
/// in var 4 has flag A 7 or `work[0]` is set: pushes HANDEX work 0x16 (+8 `t`, +4 = 7) and
/// work 4 (str 0x20E at +4 with `t`).
pub fn h_021e4238<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let v4 = b.var(4) as u32;
        let t = v4 & 0xFF;
        if b.mon(t).flag_a(7) || sys.work(f, 0) != 0 {
            let w = b.host.handex_push(0x16, me as u32);
            let buf = b.host.handex(w);
            buf.put_u8(8, v4 as u8);
            buf.put_u32(4, 7);
            b.host.handex_pop(w);
            push_str_arg(b, 4, me as u32, 0x20E, t);
        }
    }
}

/// C: unlabeled routine at `0x021E42B0`: when var 3 is the holder and `fun_021c82c4(o, 9)` or
/// `(o, 10)` holds for the opposing side `o`: pushes HANDEX work 0x1A (side `o`, +4 = 3, +6 =
/// 6) and sets `work[0]` to 1.
pub fn h_021e42b0<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let o = other_side(poke_side(me as u32) as u32) as u32;
        if b.host.fun_021c82c4(o, 9) || b.host.fun_021c82c4(o, 10) {
            let w = b.host.handex_push(0x1A, me as u32);
            let buf = b.host.handex(w);
            work_1a_prefix(buf, o as u8);
            buf[6] |= 2;
            buf[6] |= 4;
            b.host.handex_pop(w);
            sys.set_work(f, 0, 1);
        }
    }
}

/// Stat IDs at 0x021F2D20 (Pokemon value order used by `h_021e4340`).
const STATS_021F2D20: [u32; 7] = [1, 2, 5, 3, 4, 6, 7];

/// C: unlabeled routine at `0x021E4340`: when var 3 is the holder: counts the stats of the
/// Pokemon `t` in var 6 that can rise by 2 (table 0x021F2D20); if any, picks one with `rand`
/// and pushes HANDEX work 0xE (+0x10 `t`, +0xF = 1, +4 the stat, +0xC = 2, +0xE = 0).
pub fn h_021e4340<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let v6 = b.var(6) as u32;
    let t = v6 & 0xFF;
    let mut n = 0u8;
    for s in STATS_021F2D20 {
        if b.mon(t).can_change_stage(s, 2) {
            n = n.wrapping_add(1);
        }
    }
    if n != 0 {
        let mut r = b.calc.rand(n as u32) as u8;
        for s in STATS_021F2D20 {
            if b.mon(t).can_change_stage(s, 2) {
                if r == 0 {
                    let w = b.host.handex_push(0xE, me as u32);
                    let buf = b.host.handex(w);
                    buf.put_u8(0x10, v6 as u8);
                    buf.put_u8(0xF, 1);
                    buf.put_u32(4, s);
                    buf.put_u8(0xC, 2);
                    buf.put_u8(0xE, 0);
                    b.host.handex_pop(w);
                }
                r = r.wrapping_sub(1);
            }
        }
    }
}

/// C: unlabeled routine at `0x021E4408`: when var 2 is the holder and var 0x22 is 0: sets var
/// 0x22 to 0x1A if the holder has condition 2, else to 0xD if it has condition 0xF, else to 0xE
/// if it is at full HP, else to 0xF if its value 0x11 is 0xF or 0x48 (the original returns after
/// the first match; Z18c fixed this, the port let later checks override).
pub fn h_021e4408<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && b.var(0x22) == 0 {
        let m = me as u32;
        if b.mon(m).has_sick(2) {
            b.rewrite_var(0x22, 0x1A);
            return;
        }
        if b.mon(m).has_sick(0xF) {
            b.rewrite_var(0x22, 0xD);
            return;
        }
        if b.mon_is_full_hp(m) {
            b.rewrite_var(0x22, 0xE);
            return;
        }
        let v = b.mon_get(m, 0x11) as u16;
        if v == 0xF || v == 0x48 {
            b.rewrite_var(0x22, 0xF);
        }
    }
}

/// C: unlabeled routine at `0x021E448C`: when var 3 is the holder and it is not at full HP:
/// pushes HANDEX work 0xC (+0xF holder, +4 = 2, +8 `turn(3)`, +0xC = 1, +0x10 = 1, +0xE = 1,
/// str 0x27E at +0x14 with the holder), work 5 (+6 holder, +4 `get(0xE) - get(0xD)` as a
/// `u16`, bit 24 set), work 0x21 (+4 holder, +8 = 3, bit 24 set) and sets var 0x51 to 0.
pub fn h_021e448c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) || b.mon_is_full_hp(me as u32) {
        return;
    }
    let m = me as u32;
    let w = b.host.handex_push(0xC, m);
    let buf = b.host.handex(w);
    buf.put_u8(0xF, me);
    buf.put_u32(4, 2);
    buf.put_u32(8, SickCont::turn(3).0);
    buf.put_u8(0xC, 1);
    buf.put_u8(0x10, 1);
    buf.put_u8(0xE, 1);
    str_param_setup(buf, 0x14, 2, 0x27E);
    str_param_add_arg(buf, 0x14, m);
    b.host.handex_pop(w);
    let w = b.host.handex_push(5, m);
    let max = b.mon_get(m, 0xE);
    let cur = b.mon_get(m, 0xD);
    let buf = b.host.handex(w);
    buf.put_u8(6, me);
    buf.put_u16(4, max.wrapping_sub(cur) as u16);
    let h = buf.get_u32(0) | 0x100_0000;
    buf.put_u32(0, h);
    b.host.handex_pop(w);
    let w = b.host.handex_push(0x21, m);
    let buf = b.host.handex(w);
    buf.put_u8(4, me);
    buf.put_u32(8, 3);
    let h = buf.get_u32(0) | 0x100_0000;
    buf.put_u32(0, h);
    b.host.handex_pop(w);
    b.rewrite_var(0x51, 0);
}

/// C: unlabeled routine at `0x021E4564`: when var 3 is the holder, with `a` / `c` the low bytes
/// of `get(0x12)` of the holder / of var 4: if either is 2 or they are equal, sets var 0x40 to
/// 1.
pub fn h_021e4564<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let t = b.var(4) as u32 & 0xFF;
        let a = b.mon_get(me as u32, 0x12) as u8;
        let c = b.mon_get(t, 0x12) as u8;
        if a == 2 || c == 2 || a == c {
            b.rewrite_var(0x40, 1);
        }
    }
}

/// C: unlabeled routine at `0x021E4688`: when var 3 is the holder: `t` = var 6, `mv` = the
/// original last move of `t`: if `t` lacks condition 0x17, `mv` fails `fun_0689c8ac` and `t`
/// has PP for it: pushes HANDEX work 0xC (+4 = 0x17, +8 `turn_param(3 (4 if `t` has flag A 1),
/// mv)`, +0xF `t`, +0xC = 0, str 0x22F at +0x14 with `t`).
pub fn h_021e4688<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let v6 = b.var(6) as u32;
    let t = v6 & 0xFF;
    let mv = b.mon(t).last_move_orig() as u32;
    if b.mon(t).has_sick(0x17) || b.host.fun_0689c8ac(mv) || b.mon(t).move_pp_of(mv) == 0 {
        return;
    }
    let n = if b.mon(t).flag_a(1) { 4 } else { 3 };
    let w = b.host.handex_push(0xC, me as u32);
    let buf = b.host.handex(w);
    buf.put_u32(4, 0x17);
    buf.put_u32(8, SickCont::turn_param(n, mv).0);
    buf.put_u8(0xF, v6 as u8);
    buf.put_u8(0xC, 0);
    str_param_setup(buf, 0x14, 2, 0x22F);
    str_param_add_arg(buf, 0x14, t);
    b.host.handex_pop(w);
}

/// C: unlabeled routine at `0x021E4754`: when var 3 is the holder and the Pokemon `t` in var 6
/// lacks condition 0xB: pushes HANDEX work 0xC (+4 = 0xB, +8 `turn(3)` (`turn(4)` if `t` has
/// flag A 1), +0xF `t`).
pub fn h_021e4754<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let v6 = b.var(6) as u32;
    let t = v6 & 0xFF;
    if !b.mon(t).has_sick(0xB) {
        let n = if b.mon(t).flag_a(1) { 4 } else { 3 };
        let w = b.host.handex_push(0xC, me as u32);
        let buf = b.host.handex(w);
        buf.put_u32(4, 0xB);
        buf.put_u32(8, SickCont::turn(n).0);
        buf.put_u8(0xF, v6 as u8);
        b.host.handex_pop(w);
    }
}

/// `BattleMon::get_unstaged` on Pokemon `poke_id`.
fn mon_get_unstaged<H: FlowHost>(b: &mut Battle<H>, poke_id: u32, id: u32) -> u32 {
    let mon = match b.mons.get(poke_id as usize) {
        Some(m) => m,
        None => &b.null_mon,
    };
    mon.get_unstaged(id, &mut b.host)
}

/// Sets bits `mask` of the byte at `o` of a HANDEX work.
fn or_u8(buf: &mut [u8], o: usize, mask: u8) {
    buf[o] |= mask;
}

/// C: unlabeled routine at `0x021E48C0`: when var 3 is the holder, pushes HANDEX work 0x1B (+4
/// = 3, +8 `poke(holder)`, +0xC = 1, +0x10 `u16` 0x24A, +0x12 `u16` 0x102, +0x14 holder).
pub fn h_021e48c0<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let w = b.host.handex_push(0x1B, me as u32);
        let buf = b.host.handex(w);
        buf.put_u32(4, 3);
        buf.put_u32(8, SickCont::poke(me as u32).0);
        buf.put_u8(0xC, 1);
        let v = (u16::from_le_bytes([buf[0x12], buf[0x13]]) & 0xFF00) | 2;
        buf.put_u16(0x12, v);
        buf.put_u16(0x10, 0x24A);
        let v = (0x80FF & u16::from_le_bytes([buf[0x12], buf[0x13]])) | 0x100;
        buf.put_u16(0x12, v);
        buf.put_u32(0x14, me as u32);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E4A70`: when var 3 is the holder: `t` = var 6, `x` =
/// `fun_021c2e4c()`: pushes HANDEX work 0xE twice (+0xF = 1, +0x10 `t`, +4 = 1 then 3, +0xC =
/// 0xFE, +8 `x`, +0xE = 1) and work 0x13 (+4 holder).
pub fn h_021e4a70<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let v6 = b.var(6) as u8;
        let x = b.host.fun_021c2e4c();
        for kind in [1u32, 3] {
            let w = b.host.handex_push(0xE, me as u32);
            let buf = b.host.handex(w);
            buf.put_u8(0xF, 1);
            buf.put_u8(0x10, v6);
            buf.put_u32(4, kind);
            buf.put_u8(0xC, 0xFE);
            buf.put_u32(8, x);
            buf.put_u8(0xE, 1);
            b.host.handex_pop(w);
        }
        let w = b.host.handex_push(0x13, me as u32);
        b.host.handex(w).put_u8(4, me);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E4B00`: when var 3 is the holder: `t` = var 6, `mv` = the
/// original last move of `t`, `slot` its slot, `pp` = its PP (at most 4): if `slot` is not 4
/// and `pp` is nonzero pushes HANDEX work 10 (+5 var 6, +4 `pp`, +6 `slot`, str 0x281 at +8
/// with `t`, `mv`, `pp`).
pub fn h_021e4b00<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let v6 = b.var(6) as u32;
    let t = v6 & 0xFF;
    let mv = b.mon(t).last_move_orig() as u32;
    let slot = b.mon(t).move_slot_of(mv);
    if slot == 4 {
        return;
    }
    let pp = b.mon(t).move_pp(slot as u32).min(4);
    if pp != 0 {
        let w = b.host.handex_push(10, me as u32);
        let buf = b.host.handex(w);
        buf.put_u8(5, v6 as u8);
        buf.put_u8(4, pp);
        buf.put_u8(6, slot);
        str_param_setup(buf, 8, 2, 0x281);
        str_param_add_arg(buf, 8, t);
        str_param_add_arg(buf, 8, mv);
        str_param_add_arg(buf, 8, pp as u32);
        b.host.handex_pop(w);
    }
}

/// Stores `get(src, k)` of Pokemon `src` as a byte at `+4 + k` of the pushed work `w` for each
/// `k` in `ks` (the order of the original's reads).
fn store_stats<H: FlowHost>(b: &mut Battle<H>, w: usize, src: u32, ks: &[u32]) {
    for &k in ks {
        let v = b.mon_get(src, k) as u8;
        b.host.handex(w).put_u8(4 + k as usize, v);
    }
}

/// C: unlabeled routine at `0x021E4BB0`: when var 3 is the holder: `t` = var 6: pushes HANDEX
/// work 0xF (+4 holder, +4+k the value k (1..=7) of `t`), work 4 (str 0x417 at +4 with the
/// holder and `t`).
pub fn h_021e4bb0<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let t = b.var(6) as u32 & 0xFF;
        let w = b.host.handex_push(0xF, me as u32);
        b.host.handex(w).put_u8(4, me);
        store_stats(b, w, t, &[1, 2, 3, 4, 5, 6, 7]);
        b.host.handex_pop(w);
        push_str_arg2(b, 4, me as u32, 0x417, me as u32, t);
    }
}

/// As `push_str_arg` with two arguments.
fn push_str_arg2<H: FlowHost>(b: &mut Battle<H>, ty: u32, poke_id: u32, sid: u32, a: u32, c: u32) {
    let w = b.host.handex_push(ty, poke_id);
    let buf = b.host.handex(w);
    str_param_setup(buf, 4, 2, sid);
    str_param_add_arg(buf, 4, a);
    str_param_add_arg(buf, 4, c);
    b.host.handex_pop(w);
}

/// C: unlabeled routine at `0x021E4D98`: when var 3 is the holder: `t` = var 6: swaps the
/// holder's and `t`'s attack and Sp. Atk (values 1 and 3) through two HANDEX work 0xF and pushes
/// work 4 (str 0x2A4 at +4 with the holder).
pub fn h_021e4d98<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let m = me as u32;
    let v6 = b.var(6) as u32;
    let t = v6 & 0xFF;
    let a1 = b.mon_get(m, 1) as u8;
    let a3 = b.mon_get(m, 3) as u8;
    let w = b.host.handex_push(0xF, m);
    b.host.handex(w).put_u8(4, me);
    store_stats(b, w, t, &[1, 3]);
    store_stats(b, w, m, &[2, 4, 5, 6, 7]);
    b.host.handex_pop(w);
    let w = b.host.handex_push(0xF, m);
    let buf = b.host.handex(w);
    buf.put_u8(4, v6 as u8);
    buf.put_u8(5, a1);
    buf.put_u8(7, a3);
    store_stats(b, w, t, &[2, 4, 5, 6, 7]);
    b.host.handex_pop(w);
    push_str_arg(b, 4, m, 0x2A4, m);
}

/// C: unlabeled routine at `0x021E4EC4`: as `h_021e4d98` swapping values 2 and 4, string
/// 0x2A7.
pub fn h_021e4ec4<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let m = me as u32;
    let v6 = b.var(6) as u32;
    let t = v6 & 0xFF;
    let a2 = b.mon_get(m, 2) as u8;
    let a4 = b.mon_get(m, 4) as u8;
    let w = b.host.handex_push(0xF, m);
    b.host.handex(w).put_u8(4, me);
    store_stats(b, w, m, &[1, 3]);
    store_stats(b, w, t, &[2, 4]);
    store_stats(b, w, m, &[5, 6, 7]);
    b.host.handex_pop(w);
    let w = b.host.handex_push(0xF, m);
    b.host.handex(w).put_u8(4, v6 as u8);
    store_stats(b, w, t, &[1, 3]);
    let buf = b.host.handex(w);
    buf.put_u8(6, a2);
    buf.put_u8(8, a4);
    store_stats(b, w, t, &[5, 6, 7]);
    b.host.handex_pop(w);
    push_str_arg(b, 4, m, 0x2A7, m);
}

/// C: unlabeled routine at `0x021E4FF4`: when var 3 is the holder: pushes HANDEX work 0x11
/// (+0xE holder, +4 `u16` unstaged value 9, +6 `u16` unstaged value 8, +0xF = 3, str 0x305 at
/// +0x10 with the holder) and work 0x17 (+8 holder, +4 = 10).
pub fn h_021e4ff4<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let m = me as u32;
        let w = b.host.handex_push(0x11, m);
        b.host.handex(w).put_u8(0xE, me);
        let v9 = mon_get_unstaged(b, m, 9) as u16;
        b.host.handex(w).put_u16(4, v9);
        let v8 = mon_get_unstaged(b, m, 8) as u16;
        let buf = b.host.handex(w);
        buf.put_u16(6, v8);
        buf[0xF] = (buf[0xF] & 0xFE) | 1;
        or_u8(buf, 0xF, 2);
        str_param_setup(buf, 0x10, 2, 0x305);
        str_param_add_arg(buf, 0x10, m);
        b.host.handex_pop(w);
        let w = b.host.handex_push(0x17, m);
        let buf = b.host.handex(w);
        buf.put_u8(8, me);
        buf.put_u32(4, 10);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E508C`: when var 3 is the holder: `t` = var 6: averages the
/// unstaged values 8 and 10 (`(a + c) / 2`) of the holder and `t` and pushes HANDEX work 0x11
/// for `t` (+0xE, +4 `u16` average of 8, +8 `u16` average of 10, +0xF = 5) and for the holder
/// (same, str 0x448 at +0x10 with the holder).
pub fn h_021e508c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        average_stats(b, me, 8, 10, 4, 8, 5, 0x448);
    }
}

/// C: unlabeled routine at `0x021E517C`: as `h_021e508c` with the unstaged values 9 and 0xB at
/// +6 / +10, flags |= 2 | 8 (no clearing of bit 0), string 0x44B.
pub fn h_021e517c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        average_stats(b, me, 9, 0xB, 6, 10, 0xA, 0x44B);
    }
}

/// Shared body of `h_021e508c` / `h_021e517c`. `flags` has bit 0 set to also apply the
/// `(x & 0xFE) | 1` step of the first handler.
#[allow(clippy::too_many_arguments)]
fn average_stats<H: FlowHost>(
    b: &mut Battle<H>,
    me: u8,
    id_a: u32,
    id_b: u32,
    off_a: usize,
    off_b: usize,
    flags: u8,
    sid: u32,
) {
    let m = me as u32;
    let v6 = b.var(6) as u32;
    let t = v6 & 0xFF;
    let a1 = mon_get_unstaged(b, m, id_a) as i32;
    let a2 = mon_get_unstaged(b, t, id_a) as i32;
    let b1 = mon_get_unstaged(b, m, id_b) as i32;
    let b2 = mon_get_unstaged(b, t, id_b) as i32;
    let va = ((a1 + a2) / 2) as u16;
    let vb = ((b1 + b2) / 2) as u16;
    for (who, with_str) in [(v6 as u8, false), (me, true)] {
        let w = b.host.handex_push(0x11, m);
        let buf = b.host.handex(w);
        buf.put_u8(0xE, who);
        buf.put_u16(off_a, va);
        buf.put_u16(off_b, vb);
        if flags & 1 != 0 {
            buf[0xF] = (buf[0xF] & 0xFE) | 1;
        }
        or_u8(buf, 0xF, flags & !1);
        if with_str {
            str_param_setup(buf, 0x10, 2, sid);
            str_param_add_arg(buf, 0x10, m);
        }
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E5264`: when var 3 is the holder: `t` = var 6: pushes HANDEX
/// work 0xC (+0xF holder, +4 = 0x1D, +8 `poke_turn(t, 2)`, str 0x28B at +0x14 with the holder
/// and `t`).
pub fn h_021e5264<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let t = b.var(6) as u32 & 0xFF;
        let w = b.host.handex_push(0xC, me as u32);
        let buf = b.host.handex(w);
        buf.put_u8(0xF, me);
        buf.put_u32(4, 0x1D);
        buf.put_u32(8, SickCont::poke_turn(t, 2).0);
        str_param_setup(buf, 0x14, 2, 0x28B);
        str_param_add_arg(buf, 0x14, me as u32);
        str_param_add_arg(buf, 0x14, t);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E5318`: as `fun_021e556c` with the holder's side, kind 1,
/// `turn(5)` and string 0x80.
pub fn h_021e5318<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    fun_021e556c(b, me, poke_side(me as u32), 1, SickCont::turn(5).0, 0x80);
}

/// C: unlabeled routine at `0x021E535C`: as `h_021e5318` with kind 2 and string 0x84.
pub fn h_021e535c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    fun_021e556c(b, me, poke_side(me as u32), 2, SickCont::turn(5).0, 0x84);
}

/// C: unlabeled routine at `0x021E53A0`: as `h_021e5318` with kind 3 and string 0x88.
pub fn h_021e53a0<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    fun_021e556c(b, me, poke_side(me as u32), 3, SickCont::turn(5).0, 0x88);
}

/// C: unlabeled routine at `0x021E53E4`: as `h_021e5318` with kind 4, `turn(4)` and string
/// 0x8C.
pub fn h_021e53e4<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    fun_021e556c(b, me, poke_side(me as u32), 4, SickCont::turn(4).0, 0x8C);
}

/// C: unlabeled routine at `0x021E5428`: as `h_021e5318` with kind 5 and string 0x90.
pub fn h_021e5428<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    fun_021e556c(b, me, poke_side(me as u32), 5, SickCont::turn(5).0, 0x90);
}

/// C: unlabeled routine at `0x021E546C`: `fun_021e556c` with `opposing_side(holder)`, kind 6,
/// a permanent duration and string 0x94.
pub fn h_021e546c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    fun_021e556c(
        b,
        me,
        opposing_side(me as u32),
        6,
        SickCont::permanent().0,
        0x94,
    );
}

/// C: unlabeled routine at `0x021E54AC`: as `h_021e546c` with kind 7 and string 0x98.
pub fn h_021e54ac<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    fun_021e556c(
        b,
        me,
        opposing_side(me as u32),
        7,
        SickCont::permanent().0,
        0x98,
    );
}

/// C: unlabeled routine at `0x021E54EC`: as `h_021e546c` with kind 8 and string 0x9C.
pub fn h_021e54ec<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    fun_021e556c(
        b,
        me,
        opposing_side(me as u32),
        8,
        SickCont::permanent().0,
        0x9C,
    );
}

/// Shared body of `h_021e5620` / `h_021e5680`: HANDEX work 0x1E (+4 `kind`, +8 the holder's
/// battle position) and work 0x13 (+4 holder).
fn push_1e_13<H: FlowHost>(b: &mut Battle<H>, me: u8, kind: u32) {
    let w = b.host.handex_push(0x1E, me as u32);
    b.host.handex(w).put_u32(4, kind);
    let pos = b.host.poke_pos_in_battle(me as u32);
    b.host.handex(w).put_u8(8, pos as u8);
    b.host.handex_pop(w);
    let w = b.host.handex_push(0x13, me as u32);
    b.host.handex(w).put_u8(4, me);
    b.host.handex_pop(w);
}

/// C: unlabeled routine at `0x021E5620`: when var 3 is the holder and `fun_021c7f24` holds for
/// it: pushes HANDEX work 0x1E (+4 = 1, +8 its position) and work 0x13 (+4 holder).
pub fn h_021e5620<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.host.fun_021c7f24(me as u32) {
        push_1e_13(b, me, 1);
    }
}

/// C: unlabeled routine at `0x021E5680`: as `h_021e5620` with +4 = 2.
pub fn h_021e5680<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.host.fun_021c7f24(me as u32) {
        push_1e_13(b, me, 2);
    }
}

/// C: unlabeled routine at `0x021E5748`: when var 3 is the holder, pushes HANDEX work 4 (str
/// 0x432 at +4 with the holder).
pub fn h_021e5748<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        push_str_arg(b, 4, me as u32, 0x432, me as u32);
    }
}

/// C: unlabeled routine at `0x021E5828`: as `h_021e5748` with string 0x435.
pub fn h_021e5828<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        push_str_arg(b, 4, me as u32, 0x435, me as u32);
    }
}

/// C: unlabeled routine at `0x021E5870`: when var 3 is the holder: `t` = var 6: pushes HANDEX
/// work 0xC (+0xF `t`, +4 = 0x10, +8 permanent, str 0x235 at +0x14 with `t`).
pub fn h_021e5870<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let v6 = b.var(6) as u32;
        let w = b.host.handex_push(0xC, me as u32);
        let buf = b.host.handex(w);
        buf.put_u8(0xF, v6 as u8);
        buf.put_u32(4, 0x10);
        buf.put_u32(8, SickCont::permanent().0);
        str_param_setup(buf, 0x14, 2, 0x235);
        str_param_add_arg(buf, 0x14, v6 & 0xFF);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E58D8`: when var 3 is the holder: `t` = var 6, `a` = `get(0x10)`
/// of `t` (its ability); if `fun_0689c91c(a)` fails pushes HANDEX work 0x1F (+6 holder, +4 `a`,
/// str 0x26B at +0xC with the holder, `t` and `a`).
pub fn h_021e58d8<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let t = b.var(6) as u32 & 0xFF;
        let a = b.mon_get(t, 0x10) as u16;
        if !b.host.fun_0689c91c(a as u32) {
            let w = b.host.handex_push(0x1F, me as u32);
            let buf = b.host.handex(w);
            buf.put_u8(6, me);
            buf.put_u16(4, a);
            str_param_setup(buf, 0xC, 2, 0x26B);
            str_param_add_arg(buf, 0xC, me as u32);
            str_param_add_arg(buf, 0xC, t);
            str_param_add_arg(buf, 0xC, a as u32);
            b.host.handex_pop(w);
        }
    }
}

/// C: unlabeled routine at `0x021E5974`: when var 3 is the holder, `fun_021c7f24` holds for it
/// and `fun_021c7f4c` (a once-per-battle flag) is newly set: `n` = `fun_021b8ddc(holder)`:
/// pushes HANDEX work 0x29 (+0x4C holder, str 0x302 at +4 with the holder and `n`) and work
/// 0x37 (+4 `u16` 0x302 - 0x7F if this factor's sub ID is 0x171 else 0x302 - 0x7E, +6 the
/// holder's position, +7 = 6, bit 24 set).
pub fn h_021e5974<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.host.fun_021c7f24(me as u32) && b.host.fun_021c7f4c() {
        let n = b.host.fun_021b8ddc(me as u32);
        let w = b.host.handex_push(0x29, me as u32);
        let buf = b.host.handex(w);
        buf.put_u8(0x4C, me);
        str_param_setup(buf, 4, 2, 0x302);
        str_param_add_arg(buf, 4, me as u32);
        str_param_add_arg(buf, 4, n);
        b.host.handex_pop(w);
        let w = b.host.handex_push(0x37, me as u32);
        let v: u16 = if sys.sub_id(f) == 0x171 {
            0x302 - 0x7F
        } else {
            0x302 - 0x7E
        };
        let pos = b.host.poke_pos_in_battle(me as u32);
        let buf = b.host.handex(w);
        buf.put_u16(4, v);
        buf.put_u8(6, pos as u8);
        buf.put_u8(7, 6);
        let h = buf.get_u32(0) | 0x100_0000;
        buf.put_u32(0, h);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E5A34`: when var 3 is the holder: for each of its conditions
/// 0x12 and 8 that it has, pushes HANDEX work 0xB (+0x14 = 1, +8 holder, +4 the condition);
/// then with `s2`, `s3`, `s4` = `fun_06898cf4(side, 6 / 7 / 8)` for its side, if any holds
/// pushes work 0x1A (side, +4 = 3, +5 |= 0x40 / 0x80 for `s2` / `s3`, +6 |= 1 for `s4`).
pub fn h_021e5a34<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let m = me as u32;
    for sick in [0x12u32, 8] {
        if b.mon(m).has_sick(sick) {
            let w = b.host.handex_push(0xB, m);
            let buf = b.host.handex(w);
            buf.put_u8(0x14, 1);
            buf.put_u8(8, me);
            buf.put_u32(4, sick);
            b.host.handex_pop(w);
        }
    }
    let side = poke_side(m) as u32;
    let s2 = b.host.fun_06898cf4(side, 6);
    let s3 = b.host.fun_06898cf4(side, 7);
    let s4 = b.host.fun_06898cf4(side, 8);
    if s2 || s3 || s4 {
        let w = b.host.handex_push(0x1A, m);
        let buf = b.host.handex(w);
        work_1a_prefix(buf, side as u8);
        if s2 {
            buf[5] |= 0x40;
        }
        if s3 {
            buf[5] |= 0x80;
        }
        if s4 {
            buf[6] |= 1;
        }
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E5B78`: when var 3 is the holder, `fun_021c7f24` holds for it
/// and `fun_021c7f4c` is newly set: pushes HANDEX work 0x1E (+4 = 4, +8 its position, +0xC
/// holder, +0x1C = 1), work 0x17 (+8 holder, +4 = 0xE, bit 24 set) and work 0x29 (+0x4C holder,
/// +0x4D = 1, bit 24 set).
pub fn h_021e5b78<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.host.fun_021c7f24(me as u32) && b.host.fun_021c7f4c() {
        let m = me as u32;
        let w = b.host.handex_push(0x1E, m);
        b.host.handex(w).put_u32(4, 4);
        let pos = b.host.poke_pos_in_battle(m);
        let buf = b.host.handex(w);
        buf.put_u8(8, pos as u8);
        buf.put_u32(0xC, m);
        buf.put_u8(0x1C, 1);
        b.host.handex_pop(w);
        let w = b.host.handex_push(0x17, m);
        let buf = b.host.handex(w);
        buf.put_u8(8, me);
        buf.put_u32(4, 0xE);
        let h = buf.get_u32(0) | 0x100_0000;
        buf.put_u32(0, h);
        b.host.handex_pop(w);
        let w = b.host.handex_push(0x29, m);
        let buf = b.host.handex(w);
        buf.put_u8(0x4C, me);
        buf.put_u8(0x4D, 1);
        let h = buf.get_u32(0) | 0x100_0000;
        buf.put_u32(0, h);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E5C18` (event with the target in `poke_id`): if
/// `fun_021c8168` holds or the battle rule is nonzero sets var 0x22 to 0x1A; then if the
/// Pokemon has both conditions 8 and 0x16 sets var 0x22 to 0x1A.
pub fn h_021e5c18<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if b.host.fun_021c8168() || b.host.rule() != 0 {
        b.rewrite_var(0x22, 0x1A);
    }
    let mon = b.mon(me as u32);
    if mon.has_sick(8) && mon.has_sick(0x16) {
        b.rewrite_var(0x22, 0x1A);
    }
}

/// C: unlabeled routine at `0x021E5C88`: if rewriting var 0x51 to 1 succeeds pushes HANDEX
/// work 4 (str 0x2FF at +4 with the Pokemon).
pub fn h_021e5c88<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if b.rewrite_var(0x51, 1) {
        push_str_arg(b, 4, me as u32, 0x2FF, me as u32);
    }
}

/// C: unlabeled routine at `0x021E5DAC`: when var 3 is the holder and holds an item `i`:
/// pushes HANDEX work 0x23 (+4 = 1), sets `work[0]` = 1 and, if vars 0x46 and 0x47 are 0 and
/// the item parameter 9 of `i` is nonzero, pushes work 0x22 (+4 var 4, +6 `i`).
pub fn h_021e5dac<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let item = b.mon(me as u32).held_item();
    if item == 0 {
        return;
    }
    let w = b.host.handex_push(0x23, me as u32);
    b.host.handex(w).put_u32(4, 1);
    b.host.handex_pop(w);
    sys.set_work(f, 0, 1);
    if b.var(0x46) == 0 && b.var(0x47) == 0 && b.host.item_param(item as u32, 9) != 0 {
        let w = b.host.handex_push(0x22, me as u32);
        let v4 = b.var(4) as u8;
        let buf = b.host.handex(w);
        buf.put_u8(4, v4);
        buf.put_u16(6, item);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E5E7C`: when var 2 is the holder and it has condition 0x15 or
/// 0x1F, sets var 0x22 to 0x1A.
pub fn h_021e5e7c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let mon = b.mon(me as u32);
        if mon.has_sick(0x15) || mon.has_sick(0x1F) {
            b.rewrite_var(0x22, 0x1A);
        }
    }
}

/// C: unlabeled routine at `0x021E5F48`: when `fun_021c8180()` is above 1 and var 3 is the
/// holder: `t` = var 6 as a byte: if `t` has not fainted and lacks flag A 1 pushes HANDEX work
/// 4 (str 0x414 at +4 with the holder and `t`), sets `work[0]` = `t` and makes the factor
/// temporary.
pub fn h_021e5f48<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if b.host.fun_021c8180() > 1 && is_me(b, 3, me) {
        let t = b.var(6) as u32 & 0xFF;
        if !b.mon_is_fainted(t) && !b.mon(t).flag_a(1) {
            push_str_arg2(b, 4, me as u32, 0x414, me as u32, t);
            sys.set_work(f, 0, t);
            sys.make_temporary(f);
        }
    }
}

/// C: unlabeled routine at `0x021E6278`: when var 3 is the holder and the Pokemon `t` in var 6
/// has flag A 7: pushes HANDEX work 0x16 (+8 `t`, +4 = 7) and work 4 (str 0x208 at +4 with
/// `t`).
pub fn h_021e6278<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let v6 = b.var(6) as u32;
        let t = v6 & 0xFF;
        if b.mon(t).flag_a(7) {
            let w = b.host.handex_push(0x16, me as u32);
            let buf = b.host.handex(w);
            buf.put_u8(8, v6 as u8);
            buf.put_u32(4, 7);
            b.host.handex_pop(w);
            push_str_arg(b, 4, me as u32, 0x208, t);
        }
    }
}

/// C: unlabeled routine at `0x021E63BC`: when var 3 is the holder, pushes HANDEX work 0x17 (+8
/// holder, +4 = 5) and work 4 (str 0x21A at +4 with the holder).
pub fn h_021e63bc<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let w = b.host.handex_push(0x17, me as u32);
        let buf = b.host.handex(w);
        buf.put_u8(8, me);
        buf.put_u32(4, 5);
        b.host.handex_pop(w);
        push_str_arg(b, 4, me as u32, 0x21A, me as u32);
    }
}

/// C: unlabeled routine at `0x021E6548`: when var 3 is the holder, pushes HANDEX work 4 (str
/// 0x22C at +4 with the holder) and work 0xE (+0xF = 1, +0x10 holder, +4 = 2, +0xC = 1, +0xE
/// = 1).
pub fn h_021e6548<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        push_str_arg(b, 4, me as u32, 0x22C, me as u32);
        let w = b.host.handex_push(0xE, me as u32);
        let buf = b.host.handex(w);
        buf.put_u8(0xF, 1);
        buf.put_u8(0x10, me);
        buf.put_u32(4, 2);
        buf.put_u8(0xC, 1);
        buf.put_u8(0xE, 1);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E65B0`: when var 3 is the holder: `t` = var 6, `i` = the item
/// held by `t`: if `fun_020213c0(i)` holds pushes HANDEX work 0x20 (+6 `t`, +4 = 0, str 0x308
/// at +0xC with the holder and `i`) and work 0x22 (bit 24 set, +6 `i`, +4 holder).
pub fn h_021e65b0<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let v6 = b.var(6) as u32;
        let item = b.mon(v6 & 0xFF).held_item();
        if b.host.item_fun_020213c0(item as u32) {
            let w = b.host.handex_push(0x20, me as u32);
            let buf = b.host.handex(w);
            buf.put_u8(6, v6 as u8);
            buf.put_u16(4, 0);
            str_param_setup(buf, 0xC, 2, 0x308);
            str_param_add_arg(buf, 0xC, me as u32);
            str_param_add_arg(buf, 0xC, item as u32);
            b.host.handex_pop(w);
            let w = b.host.handex_push(0x22, me as u32);
            let buf = b.host.handex(w);
            let h = buf.get_u32(0) | 0x100_0000;
            buf.put_u32(0, h);
            buf.put_u16(6, item);
            buf.put_u8(4, me);
            b.host.handex_pop(w);
        }
    }
}

/// C: unlabeled routine at `0x021E6728`: when var 4 is the holder and it has fainted: if the
/// move in var 0x12 fails `move_fun_0689c8cc`, and the attacker `t` (var 3) is on the other
/// side and has not fainted: makes the factor temporary and pushes HANDEX work 0x13 (+4
/// holder, +5 = 1) and work 0x13 (+4 var 3, str 0x275 at +8 with the holder); then removes
/// the factor.
pub fn h_021e6728<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 4, me) && b.mon_is_fainted(me as u32) {
        let mv = b.var(0x12) as u32 & 0xFFFF;
        if !b.host.move_fun_0689c8cc(mv) {
            let v3 = b.var(3) as u32;
            let t = v3 & 0xFF;
            if !same_side(me as u32, t) && !b.mon_is_fainted(t) {
                sys.make_temporary(f);
                let w = b.host.handex_push(0x13, me as u32);
                let buf = b.host.handex(w);
                buf.put_u8(4, me);
                buf.put_u8(5, 1);
                b.host.handex_pop(w);
                let w = b.host.handex_push(0x13, me as u32);
                let v3b = b.var(3) as u8;
                let buf = b.host.handex(w);
                buf.put_u8(4, v3b);
                str_param_setup(buf, 8, 2, 0x275);
                str_param_add_arg(buf, 8, me as u32);
                b.host.handex_pop(w);
            }
        }
        sys.remove(f);
    }
}

/// C: unlabeled routine at `0x021E6840`: when var 4 is the holder and the move in var 0x12
/// fails `move_fun_0689c8cc`, and the holder has fainted: with `t` = var 3 and `slot` the slot
/// of the move in var 0x14 for `t`: if known pushes HANDEX work 10 (+5 var 3, +6 `slot`, +4
/// minus its PP, +7 |= 2, str 0x27B at +8 with `t` and the move); then removes the factor.
pub fn h_021e6840<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 4, me) {
        return;
    }
    let mv = b.var(0x12) as u32 & 0xFFFF;
    if b.host.move_fun_0689c8cc(mv) || !b.mon_is_fainted(me as u32) {
        return;
    }
    let v3 = b.var(3) as u32;
    let t = v3 & 0xFF;
    let v14 = b.var(0x14) as u32 & 0xFFFF;
    let slot = b.mon(t).move_slot_of(v14);
    if slot != 4 {
        let w = b.host.handex_push(10, me as u32);
        let pp = b.mon(t).move_pp(slot as u32);
        let buf = b.host.handex(w);
        buf.put_u8(5, v3 as u8);
        buf.put_u8(6, slot);
        buf.put_u8(4, pp.wrapping_neg());
        or_u8(buf, 7, 2);
        str_param_setup(buf, 8, 2, 0x27B);
        str_param_add_arg(buf, 8, t);
        str_param_add_arg(buf, 8, v14);
        b.host.handex_pop(w);
    }
    sys.remove(f);
}

/// C: unlabeled routine at `0x021E69E8`: when var 2 is the holder: pushes HANDEX work 0x15 (+8
/// holder, +4 = 5); if the holder has none of its flag B 3, 5, 4, 6 bits set pushes work 0x37
/// (+4 `u16` 0x272, +6 its position, +7 = 6, +0xB = 1); then pushes work 4 (str 0x268 at +4
/// with the holder).
pub fn h_021e69e8<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let m = me as u32;
        let w = b.host.handex_push(0x15, m);
        let buf = b.host.handex(w);
        buf.put_u8(8, me);
        buf.put_u32(4, 5);
        b.host.handex_pop(w);
        if !b.mon(m).any_flag_b_3546() {
            let w = b.host.handex_push(0x37, m);
            b.host.handex(w).put_u16(4, 0x272);
            let pos = b.host.poke_pos_in_battle(m);
            let buf = b.host.handex(w);
            buf.put_u8(6, pos as u8);
            buf.put_u8(7, 6);
            buf.put_u8(0xB, 1);
            b.host.handex_pop(w);
        }
        push_str_arg(b, 4, m, 0x268, m);
    }
}

/// C: unlabeled routine at `0x021E6B20` (Nature Power): when var 2 is the holder: picks a move
/// from the terrain `fun_021c8174()` (0/5: 0x192, 1/2/3/8/0xF: 0x59, 6/0xB/0xC: 0x38, 7: 0x3B,
/// 9: 0x1AA, 10: 0x9D, 0xD: 0x3A, else 0xA1), sets var 0x12 to it and var 0xD to
/// `fun_021c8030(holder, move)`.
pub fn h_021e6b20<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let mv = match b.host.fun_021c8174() {
            0 | 5 => 0x192,
            1 | 2 | 3 | 8 | 0xF => 0x59,
            6 | 0xB | 0xC => 0x38,
            7 => 0x3B,
            9 => 0x1AA,
            10 => 0x9D,
            0xD => 0x3A,
            _ => 0xA1,
        };
        let x = b.host.fun_021c8030(me as u32, mv);
        b.rewrite_var(0x12, mv as i32);
        b.rewrite_var(0xD, x as i32);
    }
}

/// C: unlabeled routine at `0x021E6CDC` (Sleep Talk): when var 2 is the holder and it has
/// condition 2: collects into `work[]` the moves of the holder that fail `fun_0689ca1c` and
/// have no move flag 1; if any, picks one at random, sets var 0x12 to it and var 0xD to
/// `fun_021c8030(holder, move)`; otherwise sets var 0x41 to 1.
pub fn h_021e6cdc<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) || !b.mon(me as u32).has_sick(2) {
        return;
    }
    let m = me as u32;
    let n = b.mon(m).move_count() as u32;
    let mut cnt = 0u32;
    let mut slot = 0u32;
    while slot < n {
        let id = b.mon(m).move_id(slot) as u32;
        if !b.host.fun_0689ca1c(id) && !b.host.move_flag(id, 1) {
            sys.set_work(f, cnt, id);
            cnt = (cnt + 1) & 0xFF;
        }
        slot = (slot + 1) & 0xFF;
    }
    if cnt != 0 {
        let r = b.calc.rand(cnt) & 0xFF;
        let mv = sys.work(f, r) & 0xFFFF;
        let x = b.host.fun_021c8030(m, mv);
        b.rewrite_var(0x12, mv as i32);
        b.rewrite_var(0xD, x as i32);
    } else {
        b.rewrite_var(0x41, 1);
    }
}

/// Expands position code `code` into the factor's work area (as bytes) with `expand_poke_ids`;
/// returns how many IDs were written.
fn expand_into_work<H: FlowHost>(
    sys: &mut EventSystem,
    f: FactorId,
    b: &mut Battle<H>,
    code: u32,
) -> u32 {
    let mut bytes = [0u8; 28];
    for i in 0..7 {
        bytes[i * 4..i * 4 + 4].copy_from_slice(&sys.work(f, i as u32).to_le_bytes());
    }
    let n = b.host.expand_poke_ids(code, &mut bytes);
    for i in 0..7 {
        let w = u32::from_le_bytes(bytes[i * 4..i * 4 + 4].try_into().unwrap());
        sys.set_work(f, i as u32, w);
    }
    n
}

/// The position code var 0xE selects for `h_021e6dac` / `h_021e6e38`: a value of 6 is replaced
/// by `fun_021b8c20(rule, position of the holder, 0)`.
fn target_code<H: FlowHost>(b: &mut Battle<H>, me: u8) -> u32 {
    let v = b.var(0xE) as u32 & 0xFF;
    if v == 6 {
        let pos = b.host.poke_pos(me as u32);
        let rule = b.host.rule();
        fun_021b8c20(rule, pos, 0) as u32
    } else {
        v
    }
}

/// C: unlabeled routine at `0x021E6DAC` (Mirror Move): when var 2 is the holder: expands
/// the target code (var 0xE) into `work`; for the first Pokemon found with last move `mv`:
/// if `mv` is 0 or has no move flag 6 sets var 0x41 to 1, else sets var 0x12 to `mv` and var
/// 0xD to the code.
pub fn h_021e6dac<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let code = target_code(b, me);
        if expand_into_work(sys, f, b, code) != 0 {
            let first = sys.work(f, 0) as u8 as i8 as i32 as u32;
            let mv = b.mon(first).last_move() as u32;
            if mv == 0 || !b.host.move_flag(mv, 6) {
                b.rewrite_var(0x41, 1);
                return;
            }
            b.rewrite_var(0x12, mv as i32);
            b.rewrite_var(0xD, code as i32);
        }
    }
}

/// C: unlabeled routine at `0x021E6E38` (Me First): when var 2 is the holder: expands the
/// target code (var 0xE) into `work`; for the first Pokemon found, if it lacks flag A 3, has a
/// queued action, passes `fun_021c814c`, fails `fun_021c8374` and the action is a fight with a
/// move that passes `move_fun_0201c314` and fails `fun_0689c8bc`: sets var 0x12 to that move
/// and var 0xD to the code; then always sets var 0x41 to 1.
pub fn h_021e6e38<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    use crate::calc_action::ActionRecord;
    if !is_me(b, 2, me) {
        return;
    }
    let code = target_code(b, me);
    if expand_into_work(sys, f, b, code) != 0 {
        let first = sys.work(f, 0) as u8;
        let id = first as u32;
        let mut ok = false;
        let mut waza = 0u32;
        let mut act = 0u32;
        if !b.mon(first as i8 as i32 as u32).flag_a(3)
            && b.host.find_action(id, &mut act)
            && b.host.fun_021c814c(id)
            && !b.host.fun_021c8374(id)
        {
            waza = ActionRecord(act).fight_waza();
            if waza != 0 && b.host.move_fun_0201c314(waza) && !b.host.fun_0689c8bc(waza) {
                ok = true;
            }
        }
        if ok {
            b.rewrite_var(0x12, waza as i32);
            b.rewrite_var(0xD, code as i32);
        }
    }
    b.rewrite_var(0x41, 1);
}

/// C: unlabeled routine at `0x021E6F3C` (Copycat): when var 2 is the holder and
/// `mv` = `fun_021c8114()` is nonzero and fails `fun_0689ca5c`: sets var 0x12 to `mv` and var
/// 0xD to `fun_021c8030(holder, mv)`.
pub fn h_021e6f3c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let mv = b.host.fun_021c8114();
        if mv != 0 && !b.host.fun_0689ca5c(mv) {
            let x = b.host.fun_021c8030(me as u32, mv);
            b.rewrite_var(0x12, mv as i32);
            b.rewrite_var(0xD, x as i32);
        }
    }
}

/// C: unlabeled routine at `0x021E7080`: when var 3 is the holder: `s` = the sum over values
/// 1, 2, 5, 3, 4, 6, 7 of the holder (`get`) of `v - 6` where `v > 6`; if `s` is nonzero adds
/// `s * 20` to var 0x30.
pub fn h_021e7080<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let mut s = 0i32;
        for id in STATS_021F2D20 {
            let v = b.mon_get(me as u32, id) as i32;
            if 0 < v - 6 {
                s += v - 6;
            }
        }
        if s != 0 {
            let v = b.var(0x30);
            b.rewrite_var(0x30, v.wrapping_add(s.wrapping_mul(0x14)));
        }
    }
}

/// C: unlabeled routine at `0x021E70E4`: when var 3 is the holder: `q` = `fun_021c83a0(holder)
/// / fun_021c83a0(Pokemon in var 4)`; var 0x30 = 120 (`q` >= 5), 100 (4), 80 (3), 60 (2),
/// else 40.
pub fn h_021e70e4<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let t = b.var(4) as u32 & 0xFF;
        let a = b.host.fun_021c83a0(me as u32);
        let tid = b.mon(t).poke_id() as u32;
        let c = b.host.fun_021c83a0(tid);
        let q = s32_div(a as i32, c as i32);
        let v = if q >= 5 {
            0x78
        } else if q == 4 {
            0x64
        } else if q == 3 {
            0x50
        } else if q == 2 {
            0x3C
        } else {
            0x28
        };
        b.rewrite_var(0x30, v);
    }
}

/// C: unlabeled routine at `0x021E7164`: when var 3 is the holder: `q` = `fun_021c81c8(holder)
/// / fun_021c81c8(Pokemon in var 4)`; var 0x30 = 150 (`q` >= 4), 120 (3), 80 (2), 60 (1), else
/// 40.
pub fn h_021e7164<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let t = b.var(4) as u32 & 0xFF;
        let ia = mon_key(b, me as u32);
        let ic = mon_key(b, t);
        let a = b.host.fun_021c81c8(ia, 0);
        let c = b.host.fun_021c81c8(ic, 0);
        let q = s32_div(a as i32, c as i32);
        let v = if q >= 4 {
            0x96
        } else if q >= 3 {
            0x78
        } else if q >= 2 {
            0x50
        } else if q >= 1 {
            0x3C
        } else {
            0x28
        };
        b.rewrite_var(0x30, v);
    }
}

/// Verified with a hint capping var 5 to 1..3 (Z18c; the target IDs are vars 6..11).
/// C: unlabeled routine at `0x021E732C`: when var 3 is the holder: for each of the `var 5`
/// targets (vars 6..) pushes HANDEX work 0x14 (+4 mono pair of type 10, +6 the target).
pub fn h_021e732c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let n = b.var(5) as u32;
        let mut i = 0u32;
        while i < n {
            let w = b.host.handex_push(0x14, me as u32);
            let t = b.var((i + 6) as u16);
            let buf = b.host.handex(w);
            buf.put_u16(4, make_mono_pair(10));
            buf.put_u8(6, t as u8);
            b.host.handex_pop(w);
            i += 1;
        }
    }
}

/// Verified with a hint capping var 5 to 1..3 (Z18c; the target IDs are vars 6..11).
/// C: unlabeled routine at `0x021E7388`: when var 3 is the holder: for each of the `var 5`
/// targets (vars 6..) whose ability (`get(0x10)`) is not 0x36 pushes HANDEX work 0x1F (+6 the
/// target, +4 `u16` 0x56, str 0x195 at +0xC with the target and 0x56).
pub fn h_021e7388<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let n = b.var(5) as u32;
        let mut i = 0u32;
        while i < n {
            let t = b.var((i + 6) as u16) as u32;
            if b.mon_get(t & 0xFF, 0x10) != 0x36 {
                let w = b.host.handex_push(0x1F, me as u32);
                let t2 = b.var((i + 6) as u16) as u8;
                let buf = b.host.handex(w);
                buf.put_u8(6, t2);
                buf.put_u16(4, 0x56);
                str_param_setup(buf, 0xC, 2, 0x195);
                str_param_add_arg(buf, 0xC, t2 as u32);
                str_param_add_arg(buf, 0xC, 0x56);
                b.host.handex_pop(w);
            }
            i += 1;
        }
    }
}

/// Verified with a hint capping var 5 to 1..3 (Z18c; the target IDs are vars 6..11).
/// C: unlabeled routine at `0x021E7420`: when var 3 is the holder: `a` = its ability; if
/// `ability_uncopyable(a)` (called via `0689c93c`) fails, for each of the `var 5` targets whose
/// ability is not 0x36 pushes HANDEX work 0x1F (+6 the target, +4 `a`, str 0x195 at +0xC with the
/// target and `a`).
pub fn h_021e7420<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let a = b.mon_get(me as u32, 0x10) as u16;
    if b.host.ability_uncopyable(a as u32) {
        return;
    }
    let n = b.var(5) as u32;
    let mut i = 0u32;
    while i < n {
        let t = b.var((i + 6) as u16) as u32;
        if b.mon_get(t & 0xFF, 0x10) != 0x36 {
            let w = b.host.handex_push(0x1F, me as u32);
            let t2 = b.var((i + 6) as u16) as u8;
            let buf = b.host.handex(w);
            buf.put_u8(6, t2);
            buf.put_u16(4, a);
            str_param_setup(buf, 0xC, 2, 0x195);
            str_param_add_arg(buf, 0xC, t2 as u32);
            str_param_add_arg(buf, 0xC, a as u32);
            b.host.handex_pop(w);
        }
        i += 1;
    }
}

/// C: unlabeled routine at `0x021E74E4`: when var 3 is the holder and var 0x46 is 0: pushes
/// HANDEX work 0x10 (+4 = 1, +5 var 4) and work 4 (str 0xC3 at +4 with var 4 as a byte).
pub fn h_021e74e4<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.var(0x46) == 0 {
        let v4 = b.var(4) as u32;
        let w = b.host.handex_push(0x10, me as u32);
        let buf = b.host.handex(w);
        buf.put_u8(4, 1);
        buf.put_u8(5, v4 as u8);
        b.host.handex_pop(w);
        push_str_arg(b, 4, me as u32, 0xC3, v4 & 0xFF);
    }
}

/// C: unlabeled routine at `0x021E7554`: when var 3 is the holder and var 0x46 is 0: `t` =
/// var 4, `i` = the item held by `t`: if `fun_020213c0(i)` holds pushes HANDEX work 0x20 (+6
/// `t`, +4 = 0, str 0x454 at +0xC with `t` and `i`).
pub fn h_021e7554<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.var(0x46) == 0 {
        let v4 = b.var(4) as u32;
        let item = b.mon(v4 & 0xFF).held_item();
        if b.host.item_fun_020213c0(item as u32) {
            let w = b.host.handex_push(0x20, me as u32);
            let buf = b.host.handex(w);
            buf.put_u8(6, v4 as u8);
            buf.put_u16(4, 0);
            str_param_setup(buf, 0xC, 2, 0x454);
            str_param_add_arg(buf, 0xC, v4 & 0xFF);
            str_param_add_arg(buf, 0xC, item as u32);
            b.host.handex_pop(w);
        }
    }
}

/// C: unlabeled routine at `0x021E7634`: when var 3 is the holder: `t` = var 6: unless
/// `fun_021c8374(t)`: if `fun_021c8234(t)` and `t` lacks condition 0x21 pushes HANDEX work 0xC
/// (+0xF `t`, +4 = 0x1F, +8 permanent, str 0x468 at +0x14 with `t`) and for each of its
/// conditions 0x1E and 0x20 work 0xB (+0x14 = 1, +8 `t`, +0x15 = 1, +4 the condition); then if
/// `t` has flag B 3 pushes work 0x36 (+4 `t`, +8 = 3, and str 0x468 at +0xC with `t` unless the
/// first work was pushed).
pub fn h_021e7634<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let v6 = b.var(6) as u32;
    let t = v6 & 0xFF;
    if b.host.fun_021c8374(t) {
        return;
    }
    let mut pushed = false;
    if b.host.fun_021c8234(t) && !b.mon(t).has_sick(0x21) {
        let w = b.host.handex_push(0xC, me as u32);
        let buf = b.host.handex(w);
        buf.put_u8(0xF, v6 as u8);
        buf.put_u32(4, 0x1F);
        buf.put_u32(8, SickCont::permanent().0);
        str_param_setup(buf, 0x14, 2, 0x468);
        str_param_add_arg(buf, 0x14, t);
        b.host.handex_pop(w);
        pushed = true;
        for sick in [0x1Eu32, 0x20] {
            if b.mon(t).has_sick(sick) {
                let w = b.host.handex_push(0xB, me as u32);
                let buf = b.host.handex(w);
                buf.put_u8(0x14, 1);
                buf.put_u8(8, v6 as u8);
                buf.put_u8(0x15, 1);
                buf.put_u32(4, sick);
                b.host.handex_pop(w);
            }
        }
    }
    if b.mon(t).flag_b(3) {
        let w = b.host.handex_push(0x36, me as u32);
        let buf = b.host.handex(w);
        buf.put_u8(4, v6 as u8);
        buf.put_u32(8, 3);
        if !pushed {
            str_param_setup(buf, 0xC, 2, 0x468);
            str_param_add_arg(buf, 0xC, t);
        }
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E7760`: when var 3 is the holder, pushes five HANDEX work 0xE
/// (+0xF = 1, +0x10 holder, +0xE = 1) with (+4, +0xC) = (2, 0xFF), (4, 0xFF), (1, 2), (3, 2),
/// (5, 2).
pub fn h_021e7760<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        for (kind, amount) in [(2u32, 0xFFu8), (4, 0xFF), (1, 2), (3, 2), (5, 2)] {
            let w = b.host.handex_push(0xE, me as u32);
            let buf = b.host.handex(w);
            buf.put_u32(4, kind);
            buf.put_u8(0xC, amount);
            buf.put_u8(0xF, 1);
            buf.put_u8(0x10, me);
            buf.put_u8(0xE, 1);
            b.host.handex_pop(w);
        }
    }
}

/// C: unlabeled routine at `0x021E781C`: when var 3 is the holder: `t` = var 6: pushes HANDEX
/// work 0x14 (+6 holder, +4 the type pair of `t`, +7 = 1) and work 4 (bit 24 set, str 0x441 at
/// +4 with the holder and `t`).
pub fn h_021e781c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let t = b.var(6) as u32 & 0xFF;
        let w = b.host.handex_push(0x14, me as u32);
        let pair = b.mon(t).type_pair();
        let buf = b.host.handex(w);
        buf.put_u8(6, me);
        buf.put_u16(4, pair);
        buf.put_u8(7, 1);
        b.host.handex_pop(w);
        let w = b.host.handex_push(4, me as u32);
        let buf = b.host.handex(w);
        let h = buf.get_u32(0) | 0x100_0000;
        buf.put_u32(0, h);
        str_param_setup(buf, 4, 2, 0x441);
        str_param_add_arg(buf, 4, me as u32);
        str_param_add_arg(buf, 4, t);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E78B0`: when var 3 is the holder and its weight `w` is above
/// 1: pushes HANDEX work 0x2D (+4 holder, +6 `w - 1000` (1 unless `w` is above 1001), str
/// 0x44E at +8 with the holder).
pub fn h_021e78b0<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let wt = b.mon(me as u32).weight();
        if wt > 1 {
            let w = b.host.handex_push(0x2D, me as u32);
            let v: u16 = if 0x3E9 < wt { wt - 0x3E8 } else { 1 };
            let buf = b.host.handex(w);
            buf.put_u8(4, me);
            buf.put_u16(6, v);
            str_param_setup(buf, 8, 2, 0x44E);
            str_param_add_arg(buf, 8, me as u32);
            b.host.handex_pop(w);
        }
    }
}

/// Shared body of `h_021e7998` / `h_021e7a08`: when var 3 is the holder: if field effect
/// `kind` is not active pushes HANDEX work 0x1B (+4 `kind`, +8 `turn(5)`, str `sid` type 1 at
/// +0x10), else pushes work 0x1C (+4 `kind`).
fn field_effect_start<H: FlowHost>(b: &mut Battle<H>, me: u8, kind: u32, sid: u32) {
    if is_me(b, 3, me) {
        if !b.host.field_effect_active(kind) {
            let w = b.host.handex_push(0x1B, me as u32);
            let buf = b.host.handex(w);
            buf.put_u32(4, kind);
            buf.put_u32(8, SickCont::turn(5).0);
            str_param_setup(buf, 0x10, 1, sid);
            b.host.handex_pop(w);
        } else {
            let w = b.host.handex_push(0x1C, me as u32);
            b.host.handex(w).put_u32(4, kind);
            b.host.handex_pop(w);
        }
    }
}

/// C: unlabeled routine at `0x021E7998`: `field_effect_start` for effect 6, string 0xB2.
pub fn h_021e7998<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    field_effect_start(b, me, 6, 0xB2);
}

/// C: unlabeled routine at `0x021E7A08`: `field_effect_start` for effect 7, string 0xB4.
pub fn h_021e7a08<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    field_effect_start(b, me, 7, 0xB4);
}

/// C: unlabeled routine at `0x021E1D9C`: when var 3 is the holder and `fun_021c830c` holds,
/// `fun_021e8860(2)`.
pub fn h_021e1d9c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.host.fun_021c830c() {
        fun_021e8860(b, 2);
    }
}

/// C: unlabeled routine at `0x021E2678`: when var 2 is the holder, `fun_021e26c8`.
pub fn h_021e2678<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        fun_021e26c8(sys, f, b, me as u32);
    }
}

/// C: unlabeled routine at `0x021E269C`: when var 2 is the holder and var 0x51 is 0,
/// `fun_021e26c8`.
pub fn h_021e269c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && b.var(0x51) == 0 {
        fun_021e26c8(sys, f, b, me as u32);
    }
}

/// Verified with a hint capping `work[0]` / `work[1]` to small values (Z18c).
/// C: unlabeled routine at `0x021E2700`: when var 3 is the holder: `m` = `1 << work[0]` (one
/// more shift if its flag B 7 is set); `fun_021e8860(m)`; `work[0]` += 1 and when `work[1] <=
/// work[0]` removes the holder's factors with this sub ID.
pub fn h_021e2700<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let w0 = sys.work(f, 0);
    let mut m = 1i32;
    let mut k = 0u32;
    if w0 != 0 {
        loop {
            k += 1;
            m = m.wrapping_shl(1);
            if k >= w0 {
                break;
            }
        }
    }
    if b.mon(me as u32).flag_b(7) {
        m = m.wrapping_shl(1);
    }
    fun_021e8860(b, m);
    let n = w0.wrapping_add(1);
    sys.set_work(f, 0, n);
    if (sys.work(f, 1) as i32) <= n as i32 {
        let sub = sys.sub_id(f) as u32;
        fun_021e03f8(sys, me as u32, sub);
    }
}

/// C: unlabeled routine at `0x021E281C`: when var 3 is the holder: for each history record of
/// the holder (most recent first, `history_record(0, n)`) whose attacker byte equals var 4 and
/// whose second `u16` is nonzero, `fun_021e8860(2)`.
pub fn h_021e281c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let t = b.var(4) as u32 & 0xFF;
        let mut n = 1u32;
        let mut rec = b.mon(me as u32).history_record(0, 0);
        while let Some(r) = rec {
            if (r[2] >> 8) as u32 == t && r[1] != 0 {
                fun_021e8860(b, 2);
            }
            rec = b.mon(me as u32).history_record(0, n & 0xFF);
            n += 1;
        }
    }
}

/// C: unlabeled routine at `0x021E2948`: when var 3 is the holder and the Pokemon in var 4 has
/// flag A 1, `fun_021e8860(2)`.
pub fn h_021e2948<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let t = b.var(4) as u32 & 0xFF;
        if b.mon(t).flag_a(1) {
            fun_021e8860(b, 2);
        }
    }
}

/// C: unlabeled routine at `0x021E2B18`: when var 3 is the holder and the Pokemon in var 4 has
/// no substitute and has condition 2, `fun_021e8860(2)`.
pub fn h_021e2b18<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let t = b.var(4) as u32 & 0xFF;
        if !b.mon(t).has_substitute() && b.mon(t).has_sick(2) {
            fun_021e8860(b, 2);
        }
    }
}

/// C: unlabeled routine at `0x021E2BB4`: as `h_021e2b18` with condition 1.
pub fn h_021e2bb4<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let t = b.var(4) as u32 & 0xFF;
        if !b.mon(t).has_substitute() && b.mon(t).has_sick(1) {
            fun_021e8860(b, 2);
        }
    }
}

/// C: unlabeled routine at `0x021E2E90`: when var 3 is the holder and the Pokemon in var 4 has
/// flag A 2, `fun_021e8860(2)`.
pub fn h_021e2e90<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let t = b.var(4) as u32 & 0xFF;
        if b.mon(t).flag_a(2) {
            fun_021e8860(b, 2);
        }
    }
}

/// C: unlabeled routine at `0x021E300C`: when var 3 is the holder and the Pokemon in var 4 has
/// flag B 3, `fun_021e8860(2)`.
pub fn h_021e300c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let t = b.var(4) as u32 & 0xFF;
        if b.mon(t).flag_b(3) {
            fun_021e8860(b, 2);
        }
    }
}

/// C: unlabeled routine at `0x021E700C`: when var 3 is the holder and the Pokemon in var 4 has
/// a major status, `fun_021e8860(2)`.
pub fn h_021e700c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let t = b.var(4) as u32 & 0xFF;
        if b.mon(t).major_status() != 0 {
            fun_021e8860(b, 2);
        }
    }
}

/// C: unlabeled routine at `0x021E704C`: when var 3 is the holder and it holds no item,
/// `fun_021e8860(2)`.
pub fn h_021e704c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && b.mon(me as u32).held_item() == 0 {
        fun_021e8860(b, 2);
    }
}

/// C: unlabeled routine at `0x021E3A78`: when var 3 is the holder, pushes HANDEX work 4 (str
/// 0x1FF at +4 with the holder), sets `work[6]` and calls `fun_021e3c5c(holder, 0)`.
pub fn h_021e3a78<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        push_str_arg(b, 4, me as u32, 0x1FF, me as u32);
        sys.set_work(f, 6, 1);
        fun_021e3c5c(b, me as u32, 0);
    }
}

/// C: unlabeled routine at `0x021E3BF4`: when var 3 is the holder, pushes HANDEX work 0x15 (+8
/// holder, +4 = 7), calls `fun_021e3c5c(holder, 0)` and pushes work 4 (str 0x205 at +4 with
/// the holder).
pub fn h_021e3bf4<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let w = b.host.handex_push(0x15, me as u32);
        let buf = b.host.handex(w);
        buf.put_u8(8, me);
        buf.put_u32(4, 7);
        b.host.handex_pop(w);
        fun_021e3c5c(b, me as u32, 0);
        push_str_arg(b, 4, me as u32, 0x205, me as u32);
    }
}

/// C: unlabeled routine at `0x021E3CB0`: when var 2 is the holder: if `work[0]` is 0 pushes
/// HANDEX work 0xC (+4 = 0x19, +8 `permanent_param(sub ID)`, +0xF holder), sets var 0x3E to 2,
/// `work[6]` = 1, `work[0]` = 1, `work[1]` = 0; if `work[0]` is 1 sets var 0x3E to 2 and
/// `work[0]` = 2; otherwise `fun_021c83dc(1)` and `fun_021e3f08`.
pub fn h_021e3cb0<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) {
        return;
    }
    match sys.work(f, 0) {
        0 => {
            let w = b.host.handex_push(0xC, me as u32);
            let buf = b.host.handex(w);
            buf.put_u32(4, 0x19);
            buf.put_u32(8, SickCont::permanent_param(sys.sub_id(f) as u32).0);
            buf.put_u8(0xF, me);
            b.host.handex_pop(w);
            b.rewrite_var(0x3E, 2);
            sys.set_work(f, 6, 1);
            sys.set_work(f, 0, 1);
            sys.set_work(f, 1, 0);
        }
        1 => {
            b.rewrite_var(0x3E, 2);
            sys.set_work(f, 0, 2);
        }
        _ => {
            b.host.fun_021c83dc(1);
            fun_021e3f08(sys, f, b, me as u32);
        }
    }
}

/// C: unlabeled routine at `0x021E3E64`: when var 2 is the holder, `fun_021e3f08`.
pub fn h_021e3e64<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        fun_021e3f08(sys, f, b, me as u32);
    }
}

/// C: unlabeled routine at `0x021E492C`: `fun_021e4958` with string 0x70.
pub fn h_021e492c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    fun_021e4958(b, me, 0x70);
}

/// C: unlabeled routine at `0x021E4948`: `fun_021e4958` with string 0x6F.
pub fn h_021e4948<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    fun_021e4958(b, me, 0x6F);
}

/// C: unlabeled routine at `0x021E52D4`: `fun_021e556c` with the holder's side, kind 0,
/// `turn(5)` and string 0x7C.
pub fn h_021e52d4<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    fun_021e556c(b, me, poke_side(me as u32), 0, SickCont::turn(5).0, 0x7C);
}

/// C: unlabeled routine at `0x021E552C`: `fun_021e556c` with the holder's side, kind 9,
/// `turn(1)` and string 0xA0, then `fun_021e3c5c(holder, 1)`.
pub fn h_021e552c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    fun_021e556c(b, me, poke_side(me as u32), 9, SickCont::turn(1).0, 0xA0);
    fun_021e3c5c(b, me as u32, 1);
}

/// C: unlabeled routine at `0x021E7E0C`: as `h_021e552c` with kind 10 and string 0xA2.
pub fn h_021e7e0c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    fun_021e556c(b, me, poke_side(me as u32), 10, SickCont::turn(1).0, 0xA2);
    fun_021e3c5c(b, me as u32, 1);
}

/// C: unlabeled routine at `0x021E5720`: when var 3 is the holder, `fun_021e5784(var 0xD as
/// a byte)`.
pub fn h_021e5720<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let p = b.var(0xD) as u32 & 0xFF;
        fun_021e5784(sys, f, b, me, p);
    }
}

/// C: unlabeled routine at `0x021E5800`: as `h_021e5720`.
pub fn h_021e5800<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let p = b.var(0xD) as u32 & 0xFF;
        fun_021e5784(sys, f, b, me, p);
    }
}

/// C: unlabeled routine at `0x021E7A78`: when var 3 is the holder: `t` = var 4 as a byte, `p`
/// = `fun_021c7d34(t)`; unless `p` is 6, expands code `p | 0x900` into IDs and for each other
/// than `t` pushes HANDEX work 7 (+6 the ID, +4 max HP / 16 (at least 1) of the ID, +7 |=
/// 1, str 0x451 at +0xC with the ID).
pub fn h_021e7a78<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let t = b.var(4) as u32 & 0xFF;
    let p = b.host.fun_021c7d34(t);
    if p == 6 {
        return;
    }
    let mut ids = [0u8; 8];
    let n = b.host.expand_poke_ids((p & 0xFFFF) | 0x900, &mut ids) as usize;
    for &id in ids.iter().take(n.min(8)) {
        if id as u32 != t {
            let hp = b.max_hp_div_min1(id as u32, 0x10);
            let w = b.host.handex_push(7, me as u32);
            let buf = b.host.handex(w);
            buf.put_u8(6, id);
            buf.put_u16(4, hp as u16);
            buf[7] = (buf[7] & 0xFE) | 1;
            str_param_setup(buf, 0xC, 2, 0x451);
            str_param_add_arg(buf, 0xC, id as u32);
            b.host.handex_pop(w);
        }
    }
}

/// C: unlabeled routine at `0x021E7E58`: `pos` = the holder's position; with the battle rule 1
/// (2) and `fun_021c8300(fun_021b8ddc(holder))` = 4 (`fun_021b99f0(pos)` false) the ally slot
/// is looked up (`fun_021b8c38(pos, 0)`, or with 1 (2) if equal to `pos`) and its Pokemon
/// (`poke_id_at_pos`) becomes `t`; if `t` is not the holder pushes HANDEX work 0x32 (+4 holder,
/// +5 `t`, str 0x471 at +8 with the holder and `t`).
pub fn h_021e7e58<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    let m = me as u32;
    let pos = b.host.poke_pos_in_battle(m);
    let rule = b.host.rule();
    let mut t = m;
    if rule == 1 {
        let n = b.host.fun_021b8ddc(m);
        if b.host.fun_021c8300(n) == 4 {
            let mut p = fun_021b8c38(pos, 0) as u32;
            if p == pos {
                p = fun_021b8c38(pos, 1) as u32;
            }
            t = b.host.poke_id_at_pos(p);
        }
    } else if rule == 2 && !fun_021b99f0(pos as i32) {
        let mut p = fun_021b8c38(pos, 0) as u32;
        if p == pos {
            p = fun_021b8c38(pos, 2) as u32;
        }
        t = b.host.poke_id_at_pos(p);
    }
    if t != m {
        let w = b.host.handex_push(0x32, m);
        let buf = b.host.handex(w);
        buf.put_u8(4, me);
        buf.put_u8(5, t as u8);
        str_param_setup(buf, 8, 2, 0x471);
        str_param_add_arg(buf, 8, m);
        str_param_add_arg(buf, 8, t);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E7F1C`: when var 3 is the holder and the Pokemon in var 4 has
/// species 0x32 or 0x33 or condition 0x1F or 0x15, sets var 0x40 to 1.
pub fn h_021e7f1c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let t = b.var(4) as u32 & 0xFF;
        let mon = b.mon(t);
        let sp = mon.species();
        if sp == 0x32 || sp == 0x33 || mon.has_sick(0x1F) || mon.has_sick(0x15) {
            b.rewrite_var(0x40, 1);
        }
    }
}

/// C: unlabeled routine at `0x021E7F6C`: when var 3 is the holder, sets var 0x1D to 0x20 and
/// the message parameter (var 0x3F) to string 0x474 type 2 with var 4 as its argument.
pub fn h_021e7f6c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        b.rewrite_var(0x1D, 0x20);
        let v4 = b.var(4) as u32;
        let buf = b.host.msg_param();
        str_param_setup(buf, 0, 2, 0x474);
        str_param_add_arg(buf, 0, v4);
    }
}

/// C: unlabeled routine at `0x021E7FE4`: when var 3 is the holder: `t` = var 6 as a byte: if
/// `t` is 0x1F or `fun_021c846c(holder, t)` fails: sets var 0x41 to 1 and, if the query set its
/// output, var 0x4F to 1; else when rewriting var 0x51 to 1 succeeds pushes HANDEX work 4 (str
/// 0x45E at +4 with the holder and `t`).
pub fn h_021e7fe4<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let t = b.var(6) as u32 & 0xFF;
    let mut out = 0u32;
    if t == 0x1F || !b.host.fun_021c846c(me as u32, t, &mut out) {
        b.rewrite_var(0x41, 1);
        if out != 0 {
            b.rewrite_var(0x4F, 1);
        }
    } else if b.rewrite_var(0x51, 1) {
        push_str_arg2(b, 4, me as u32, 0x45E, me as u32, t);
    }
}

/// C: unlabeled routine at `0x021E8190`: when the holder is species 0x288 and var 3 is the
/// holder: pushes HANDEX work 0x39 (+4 holder, +5 `get(0x13) == 0`, str 0xDE at +8 with the
/// holder).
pub fn h_021e8190<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if b.mon(me as u32).species() == 0x288 && is_me(b, 3, me) {
        let v = b.mon_get(me as u32, 0x13);
        let w = b.host.handex_push(0x39, me as u32);
        let buf = b.host.handex(w);
        buf.put_u8(4, me);
        buf.put_u8(5, (v == 0) as u8);
        str_param_setup(buf, 8, 2, 0xDE);
        str_param_add_arg(buf, 8, me as u32);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E84B8`: when var 2 is the holder and `work[0]` is set: var
/// 0x13 = 0x206 (`work[1]` 1), 0x207 (2), 0x208 (3), else 0.
pub fn h_021e84b8<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && sys.work(f, 0) != 0 {
        let v = match sys.work(f, 1) as i32 {
            1 => 0x206,
            2 => 0x207,
            3 => 0x208,
            _ => 0,
        };
        b.rewrite_var(0x13, v);
    }
}

/// C: unlabeled routine at `0x021E3E84(flow, poke_id)`: the first of the holder's history
/// slots 0..3 with a record gives its attacker `a`; if `a` has no position (6) the result is
/// a random Pokemon of the holder's side group (`expand_poke_ids(0x100 | position of the
/// holder)`), 0x1F if there is none; else `a`. 0x1F when there is no record.
pub fn fun_021e3e84<H: FlowHost>(b: &mut Battle<H>, me: u32) -> u8 {
    let mut i = 0u32;
    while i < 3 {
        if b.mon(me).history_count(i) != 0 {
            let rec = b.mon(me).history_record(i, 0);
            let mut a = rec.map(|r| (r[2] >> 8) as u8).unwrap_or(0);
            if b.host.poke_pos(a as u32) == 6 {
                let pos = b.host.poke_pos_in_battle(me);
                let mut ids = [0u8; 8];
                let n = b.host.expand_poke_ids(0x100 | pos, &mut ids);
                if n == 0 {
                    a = 0x1F;
                } else {
                    let k = b.calc.rand(n) & 0xFF;
                    a = ids[k as usize & 7];
                }
            }
            return a;
        }
        i += 1;
    }
    0x1F
}

/// C: unlabeled routine at `0x021E3DCC`: when var 3 is the holder and `work[0] > 1`:
/// `work[2]` and var 4 = `fun_021e3e84(holder)`.
pub fn h_021e3dcc<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && (sys.work(f, 0) as i32) > 1 {
        let v = fun_021e3e84(b, me as u32) as u32;
        sys.set_work(f, 2, v);
        b.rewrite_var(4, v as i32);
    }
}

/// C: unlabeled routine at `0x021E3544` (Magic Bounce, `magic_bounce_hit`).
pub fn h_021e3544<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    crate::handlers::abilities::magic_bounce_hit(b, me);
}

/// C: unlabeled routine at `0x021E354C`: when var 2 is the holder and rewriting var 0x51 to 1
/// succeeds, Magic Bounce's message (`magic_bounce_message`).
pub fn h_021e354c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) && b.rewrite_var(0x51, 1) {
        crate::handlers::abilities::magic_bounce_message(b, me);
    }
}

/// C: unlabeled routine at `0x021E34B4`: when the low byte of var 2 is the holder: if
/// `fun_021c80ac(holder)` sets var 0x22 to 0x1A; else if var 2 is on the other side,
/// `magic_bounce_move`.
pub fn h_021e34b4<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    let v2 = b.var(2) as u32 & 0xFF;
    if v2 == me as u32 {
        if b.host.fun_021c80ac(me as u32) {
            b.rewrite_var(0x22, 0x1A);
        }
    } else if !same_side(me as u32, v2) {
        crate::handlers::abilities::magic_bounce_move(b, me);
    }
}

/// C: unlabeled routine at `0x021E35A0`: when var 3 is the holder and `fun_021c80ac` fails for
/// it: pushes HANDEX work 4 (str 0x2EF at +4 with the holder) and sets `work[6]`.
pub fn h_021e35a0<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) && !b.host.fun_021c80ac(me as u32) {
        push_str_arg(b, 4, me as u32, 0x2EF, me as u32);
        sys.set_work(f, 6, 1);
    }
}

/// C: unlabeled routine at `0x021E3B64`: when var 2 is the holder: with `n` = its byte
/// `+0x157[3]` (nonzero): `rand(table[min(n, 9)])` nonzero sets var 0x22 to 0x1A and stops (table
/// 1, 2, 4, 8, 16, 32, 64, 128, 0, 0); otherwise sets it if `fun_021c80ac(holder)` holds.
pub fn h_021e3b64<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let n = b.mon(me as u32).u8_157(3) as usize;
        if n != 0 {
            const TBL: [u32; 10] = [1, 2, 4, 8, 0x10, 0x20, 0x40, 0x80, 0, 0];
            let r = b.calc.rand(TBL[n.min(9)]);
            if r != 0 {
                b.rewrite_var(0x22, 0x1A);
                return;
            }
        }
        if b.host.fun_021c80ac(me as u32) {
            b.rewrite_var(0x22, 0x1A);
        }
    }
}

/// `n - 1` as a byte if `n` is 1..=24, else 0x1F (the decoding `h_021e8070` and friends apply
/// to the holder's byte `+0x157[4]`).
fn decode_157_4(n: u8) -> u32 {
    if n == 0 {
        return 0x1F;
    }
    let v = n.wrapping_sub(1) as u32;
    if v >= 0x18 {
        0x1F
    } else {
        v
    }
}

/// C: unlabeled routine at `0x021E8070`: when var 3 is the holder: `v` = `decode(u8_157(4))`;
/// if `v` is 0x1F or `poke_pos(v)` is 6 sets var 0x41 to 1; then `fun_021c8494(holder)`.
pub fn h_021e8070<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let v = decode_157_4(b.mon(me as u32).u8_157(4));
        if v == 0x1F || b.host.poke_pos(v) == 6 {
            b.rewrite_var(0x41, 1);
        }
        b.host.fun_021c8494(me as u32);
    }
}

/// C: unlabeled routine at `0x021E80C0`: when the battle rule is neither 0 nor 3: `v` =
/// `decode(u8_157(4))` of the Pokemon; if not 0x1F sets var 4 to `v`.
pub fn h_021e80c0<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if b.host.rule() != 0 && b.host.rule() != 3 {
        let v = decode_157_4(b.mon(me as u32).u8_157(4));
        if v != 0x1F {
            b.rewrite_var(4, v as i32);
        }
    }
}

/// C: unlabeled routine at `0x021E8104`: when var 2 is the holder: `v` = `decode(u8_157(4))`;
/// if not 0x1F and `fun_021c814c(holder)` fails sets var 0x22 to 0x1A.
pub fn h_021e8104<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let v = decode_157_4(b.mon(me as u32).u8_157(4));
        if v != 0x1F && !b.host.fun_021c814c(v) {
            b.rewrite_var(0x22, 0x1A);
        }
    }
}

/// C: unlabeled routine at `0x021E71E4`: when var 3 is the holder: counts the consecutive
/// previous turns (from the one before the current) in whose move ring this factor's move
/// appears (`fun_0689d064`); var 0x30 = 40, 80, 120, 160 for 0..3, else 200.
pub fn h_021e71e4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let sub = sys.sub_id(f) as u32;
        let mut turn = b.host.fun_021c8138() as i32;
        let mut n = 0u32;
        loop {
            turn -= 1;
            if !(turn > -1 && b.host.fun_0689d064(sub, turn as u32)) {
                break;
            }
            n += 1;
        }
        let v = match n {
            0 => 0x28,
            1 => 0x50,
            2 => 0x78,
            3 => 0xA0,
            _ => 200,
        };
        b.rewrite_var(0x30, v);
    }
}

/// C: unlabeled routine at `0x021E727C`: when var 3 is the holder: for each member of side 1's
/// party record on the holder's side, multiplies var 0x31 by 0x2000 and stops.
pub fn h_021e727c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let n = b.host.fun_0689d174(1);
        let mut i = 0u32;
        while i < n {
            let id = b.host.fun_0689d184(1, i);
            if same_side(me as u32, id) {
                b.mul_var(0x31, 0x2000);
                return;
            }
            i = (i + 1) & 0xFF;
        }
    }
}

/// C: unlabeled routine at `0x021E7D54`: when var 2 is the holder and this factor's move was
/// used exactly once this turn (`fun_0689d098`): pushes HANDEX work 0x30 (+4 `u16` the move).
pub fn h_021e7d54<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let sub = sys.sub_id(f) as u32;
        let turn = b.host.fun_021c8138();
        if b.host.fun_0689d098(sub, turn) == 1 {
            let w = b.host.handex_push(0x30, me as u32);
            b.host.handex(w).put_u16(4, sub as u16);
            b.host.handex_pop(w);
        }
    }
}

/// C: unlabeled routine at `0x021E7DB0`: when var 3 is the holder and this factor's move was
/// used more than once this turn (`fun_0689d098`), `fun_021e8860(2)`.
pub fn h_021e7db0<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let sub = sys.sub_id(f) as u32;
        let turn = b.host.fun_021c8138();
        if b.host.fun_0689d098(sub, turn) > 1 {
            fun_021e8860(b, 2);
        }
    }
}

/// C: unlabeled routine at `0x021E8310`: when var 3 is the holder and this factor's move is
/// 0x22E or 0x22F: if the other of the two was the move used this turn (`fun_0689d0c8`),
/// multiplies var 0x31 by 0x2000 and calls `fun_021c83dc(1)`.
pub fn h_021e8310<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let other = match sys.sub_id(f) {
            0x22E => 0x22F,
            0x22F => 0x22E,
            _ => 0,
        };
        if other != 0 {
            let turn = b.host.fun_021c8138();
            if b.host.fun_0689d0c8(turn) == other {
                b.mul_var(0x31, 0x2000);
                b.host.fun_021c83dc(1);
            }
        }
    }
}

/// C: unlabeled routine at `0x021E8638(species, item)`: whether `item` is the one tied to that
/// species (species 0x1E7: item 0x70; 0x1ED: `item_fun_0689c980`; 0x289: `item_fun_0689c990`).
pub fn fun_021e8638<H: FlowHost>(b: &mut Battle<H>, species: u32, item: u32) -> bool {
    if species == 0x1E7 {
        item == 0x70
    } else if species == 0x1ED {
        b.host.item_fun_0689c980(item)
    } else if species == 0x289 {
        b.host.item_fun_0689c990(item)
    } else {
        false
    }
}

/// C: unlabeled routine at `0x021E869C(flow, poke_id)`: `fun_021e8638(species, held item)` of
/// the Pokemon.
pub fn fun_021e869c<H: FlowHost>(b: &mut Battle<H>, id: u32) -> bool {
    let sp = b.mon(id).species() as u32;
    let item = b.mon(id).held_item() as u32;
    fun_021e8638(b, sp, item)
}

/// C: unlabeled routine at `0x021E8D48(pair_a, pair_b)`: whether the two type pairs share a
/// type.
fn pairs_share_type(a: u16, c: u16) -> bool {
    let (a1, a2) = ((a & 0xFF) as u8, (a >> 8) as u8);
    let (c1, c2) = ((c & 0xFF) as u8, (c >> 8) as u8);
    a2 == c2 || a2 == c1 || a1 == c2 || a1 == c1
}

/// C: unlabeled routine at `0x021E8384(a, b)`: the entry of the table at 0x021F2CF0 (move
/// pairs 0x206/0x207 -> 1, 0x208/0x207 -> 2, 0x206/0x208 -> 3) matching the two values in
/// either order, else 0.
fn fun_021e8384(a: u32, c: u32) -> u32 {
    const T: [(u32, u32, u32); 3] = [(0x206, 0x207, 1), (0x208, 0x207, 2), (0x206, 0x208, 3)];
    for (x, y, v) in T {
        if (a == x && c == y) || (c == x && a == y) {
            return v;
        }
    }
    0
}

/// C: unlabeled routine at `0x021E60A8(flow, poke_id, n)`: the `n`-th (0-based) member of the
/// party of `poke_id`'s side that is `poke_id` itself or alive without a major status; `None`
/// when `n` is past the party size or there are not enough.
pub fn fun_021e60a8<H: FlowHost>(b: &mut Battle<H>, poke_id: u32, n: u32) -> Option<u32> {
    let mut ids = [0u8; 8];
    let count = b.host.party_ids(poke_id, &mut ids);
    if count <= n {
        return None;
    }
    let mut n = n;
    for &id in ids.iter().take((count as usize).min(8)) {
        let id = id as u32;
        let mut ok = b.mon(id).poke_id() as u32 == poke_id;
        if !ok {
            let alive = {
                let mon = match b.mons.get(id as usize) {
                    Some(m) => m,
                    None => &b.null_mon,
                };
                mon.is_alive(&mut b.host)
            };
            ok = alive && b.mon(id).major_status() == 0;
        }
        if ok {
            if n == 0 {
                return Some(id);
            }
            n = (n - 1) & 0xFF;
        }
    }
    None
}

/// C: unlabeled routine at `0x021D7E44(list, count)`: a random move ID in 1..0x230 that is not
/// in `list` (`fun_021d7e90`).
pub fn fun_021d7e44<H: FlowHost>(b: &mut Battle<H>, list: &[u16], count: u32) -> u16 {
    let mut pool: Vec<u16> = Vec::new();
    let mut v = 1u16;
    while v < 0x230 {
        if !fun_021d7e90(list, count, v as u32) {
            pool.push(v);
        }
        v += 1;
    }
    let r = b.calc.rand(pool.len() as u32) & 0xFFFF;
    pool.get(r as usize).copied().unwrap_or(0)
}

/// C: unlabeled routine at `0x021E2F48`: when var 2 is the holder: by the weather (1: type 9
/// and `fun_021c83dc(1)`, 2: 10 / 4, 3: 14 / 2, 4: 5 / 3; else the current var 0x16 byte and
/// 0) sets var 0x16.
pub fn h_021e2f48<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let w = b.host.weather();
        let cur = b.var(0x16) as u32 & 0xFF;
        let (ty, k) = match w {
            1 => (9, 1),
            2 => (10, 4),
            3 => (14, 2),
            4 => (5, 3),
            _ => (cur, 0),
        };
        b.rewrite_var(0x16, ty as i32);
        b.host.fun_021c83dc(k);
    }
}

/// C: unlabeled routine at `0x021E237C`: when var 3 is the holder, it lacks condition 0x19 and
/// `work[6]` is 0: pushes HANDEX work 0xC (+4 = 0x19, +8 `turn_param(3, sub ID)`, +0xC = 0,
/// +0xF holder) and work 4 (str 0x2BF with the holder); then for each Pokemon of the holder's
/// position group (`expand_poke_ids(position | 0x800)`) with condition 2 pushes work 0xB (+4 =
/// 2, +0x14 = 1, +8 the Pokemon, str 0x2C2 at +0x18 with it); sets `work[6]`.
pub fn h_021e237c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    let m = me as u32;
    if !(is_me(b, 3, me) && !b.mon(m).has_sick(0x19) && sys.work(f, 6) == 0) {
        return;
    }
    let w = b.host.handex_push(0xC, m);
    let buf = b.host.handex(w);
    buf.put_u32(4, 0x19);
    buf.put_u32(8, SickCont::turn_param(3, sys.sub_id(f) as u32).0);
    buf.put_u8(0xC, 0);
    buf.put_u8(0xF, me);
    b.host.handex_pop(w);
    push_str_arg(b, 4, m, 0x2BF, m);
    let pos = b.host.poke_pos(m);
    let mut ids = [0u8; 8];
    let n = b.host.expand_poke_ids((pos & 0xFFFF) | 0x800, &mut ids) as usize;
    for &id in ids.iter().take(n.min(8)) {
        if b.mon(id as u32).has_sick(2) {
            let w = b.host.handex_push(0xB, m);
            let buf = b.host.handex(w);
            buf.put_u32(4, 2);
            buf.put_u8(0x14, 1);
            buf.put_u8(8, id);
            str_param_setup(buf, 0x18, 2, 0x2C2);
            str_param_add_arg(buf, 0x18, id as u32);
            b.host.handex_pop(w);
        }
    }
    sys.set_work(f, 6, 1);
}

/// C: unlabeled routine at `0x021E4828`: when var 3 is the holder: `t` = var 6, `mv` = the
/// original last move of `t`: if `t` lacks condition 0xD and `mv` is nonzero, not 0xA5 and
/// known to `t`: pushes HANDEX work 0xC (+4 = 0xD, +8 `turn_param(4 (5 if `t` has flag A 1),
/// mv)`, +0xF `t`).
pub fn h_021e4828<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let v6 = b.var(6) as u32;
    let t = v6 & 0xFF;
    if b.mon(t).has_sick(0xD) {
        return;
    }
    let mv = b.mon(t).last_move_orig() as u32;
    if mv != 0 && mv != 0xA5 && b.mon(t).knows_move(mv) {
        let n = if b.mon(t).flag_a(1) { 5 } else { 4 };
        let w = b.host.handex_push(0xC, me as u32);
        let buf = b.host.handex(w);
        buf.put_u32(4, 0xD);
        buf.put_u32(8, SickCont::turn_param(n, mv).0);
        buf.put_u8(0xF, v6 as u8);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E45CC` (Conversion 2): when var 3 is the holder: `ty` = the
/// type of the last move of the Pokemon in var 6; unless it is 0x11, the types resisting it
/// (`resisting_types`) minus those the holder already has; if any remain, picks one at
/// random and pushes HANDEX work 0x14 (+4 its mono pair, +6 holder).
pub fn h_021e45cc<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let t = b.var(6) as u32 & 0xFF;
    let ty = b.mon(t).last_move_type();
    if ty == 0x11 {
        return;
    }
    let mut buf = [0u8; types::COUNT as usize];
    let mut n = crate::calc::resisting_types(ty, &mut buf) as u32;
    if n == 0 {
        return;
    }
    let mut i = 0u32;
    loop {
        if b.mon(me as u32).has_type(buf[i as usize] as u32) {
            let cnt = n - 1 - i;
            if (cnt as i32) > 0 {
                let (s, e) = (i as usize + 1, i as usize + 1 + cnt as usize);
                buf.copy_within(s..e, i as usize);
            }
            n -= 1;
        } else {
            i += 1;
        }
        if i >= n {
            break;
        }
    }
    if n != 0 {
        let r = b.calc.rand(n);
        let pick = buf[r as usize];
        let w = b.host.handex_push(0x14, me as u32);
        let buf = b.host.handex(w);
        buf.put_u16(4, make_mono_pair(pick));
        buf.put_u8(6, me);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E4C74`: when var 3 is the holder: `t` = var 6: pushes HANDEX
/// work 0xF (+4 holder, values 1..=7 of `t` as bytes at +5..), work 0xF (+4 `t`, the holder's
/// values 1..=7 as bytes at +5..) and work 4 (str 0x2A1 with the holder) (swaps all stats).
pub fn h_021e4c74<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let m = me as u32;
    let v6 = b.var(6) as u32;
    let t = v6 & 0xFF;
    let mut mine = [0u32; 7];
    for (k, slot) in mine.iter_mut().enumerate() {
        *slot = b.mon_get(m, k as u32 + 1);
    }
    let w = b.host.handex_push(0xF, m);
    b.host.handex(w).put_u8(4, me);
    store_stats(b, w, t, &[1, 2, 3, 4, 5, 6, 7]);
    b.host.handex_pop(w);
    let w = b.host.handex_push(0xF, m);
    let buf = b.host.handex(w);
    buf.put_u8(4, v6 as u8);
    for (k, v) in mine.iter().enumerate() {
        buf.put_u8(5 + k, *v as u8);
    }
    b.host.handex_pop(w);
    push_str_arg(b, 4, m, 0x2A1, m);
}

/// C: unlabeled routine at `0x021E6A84`: when var 2 is the holder: picks a random move not in
/// the excluded list (`fun_0689ca8c`, 0x29 entries) with `fun_021d7e44`, sets var 0x12 to it
/// and var 0xD to `fun_021c8030(holder, move)`.
pub fn h_021e6a84<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let list = b.host.fun_0689ca8c();
        let count = list.len() as u32;
        let mv = fun_021d7e44(b, &list, count) as u32;
        let x = b.host.fun_021c8030(me as u32, mv);
        b.rewrite_var(0x12, mv as i32);
        b.rewrite_var(0xD, x as i32);
    }
}

/// C: unlabeled routine at `0x021E6BF4` (Assist): when var 2 is the holder: collects the moves
/// of the other party members that fail `fun_0689ca3c`; if any, picks one at random, sets var
/// 0x12 to it and var 0xD to `fun_021c8030(holder, move)`.
pub fn h_021e6bf4<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 2, me) {
        return;
    }
    let m = me as u32;
    let mut ids = [0u8; 8];
    let count = b.host.party_ids(m, &mut ids) as usize;
    let mut list: Vec<u16> = Vec::new();
    for &id in ids.iter().take(count.min(8)) {
        let id = id as u32;
        if b.mon(id).poke_id() as u32 != m {
            let n = b.mon(id).move_count() as u32;
            let mut slot = 0u32;
            while slot < n {
                let mv = b.mon(id).move_id(slot);
                if !b.host.fun_0689ca3c(mv as u32) {
                    list.push(mv);
                }
                slot = (slot + 1) & 0xFF;
            }
        }
    }
    if !list.is_empty() {
        let r = b.calc.rand(list.len() as u32 & 0xFF) & 0xFF;
        let mv = list[r as usize] as u32;
        let x = b.host.fun_021c8030(m, mv);
        b.rewrite_var(0x12, mv as i32);
        b.rewrite_var(0xD, x as i32);
    }
}

/// C: unlabeled routine at `0x021E6010`: when var 3 is the holder: var 0x2A = the number of the
/// first six party slots for which `fun_021e60a8` finds a Pokemon (at least 1); `work[0]` = 0.
pub fn h_021e6010<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let mut n = 0i32;
        for i in 0..6u32 {
            if fun_021e60a8(b, me as u32, i).is_some() {
                n += 1;
            }
        }
        if n == 0 {
            n = 1;
        }
        b.rewrite_var(0x2A, n);
        sys.set_work(f, 0, 0);
    }
}

/// C: unlabeled routine at `0x021E6054`: when var 3 is the holder: `m` = the `work[0]`-th
/// eligible party member (`fun_021e60a8`) or the holder; `work[0]` counts up to 5 then wraps
/// to 0; var 0x30 = `get(0x15)` of `m` / 10 + 5.
pub fn h_021e6054<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let w0 = sys.work(f, 0);
        let id = fun_021e60a8(b, me as u32, w0 & 0xFF).unwrap_or(me as u32);
        let n = w0.wrapping_add(1);
        sys.set_work(f, 0, if (n as i32) > 5 { 0 } else { n });
        let v = b.mon_get(id, 0x15);
        b.rewrite_var(0x30, u32_div(v, 10).wrapping_add(5) as i32);
    }
}

/// C: unlabeled routine at `0x021E5CD4`: when var 2 is the holder: sets var 0x22 to 0x1A if it
/// holds no item, the item's parameter 10 is 0, `fun_021c824c` fails for it, or
/// `fun_021e869c` holds.
pub fn h_021e5cd4<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 2, me) {
        let item = b.mon(me as u32).held_item() as u32;
        if item == 0
            || b.host.item_param(item, 10) == 0
            || !b.host.fun_021c824c(me as u32)
            || fun_021e869c(b, me as u32)
        {
            b.rewrite_var(0x22, 0x1A);
        }
    }
}

/// C: unlabeled routine at `0x021E7B40`: when var 3 is the holder and the type pairs of the
/// holder and of the Pokemon in var 4 share no type, sets var 0x40 to 1.
pub fn h_021e7b40<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let t = b.var(4) as u32 & 0xFF;
        let a = b.mon(me as u32).type_pair();
        let c = b.mon(t).type_pair();
        if !pairs_share_type(a, c) {
            b.rewrite_var(0x40, 1);
        }
    }
}

/// C: unlabeled routine at `0x021E7B98` (Trick): when var 3 is the holder with an item that
/// `fun_0202136c` rejects, `t` (var 6) holds none, neither passes `fun_021e869c` and
/// `fun_021e8638(species of t, item)` fails: pushes HANDEX work 0x20 (+6 holder, +4 = 0) and
/// work 0x20 (+6 var 6, +4 the item, str 0x457 at +0xC with `t`, the holder and the item).
pub fn h_021e7b98<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let m = me as u32;
    let v6 = b.var(6) as u32;
    let t = v6 & 0xFF;
    let item = b.mon(m).held_item();
    if item != 0
        && !fun_0202136c(item as u32)
        && b.mon(t).held_item() == 0
        && !fun_021e869c(b, m)
        && !fun_021e869c(b, t)
    {
        let sp = b.mon(t).species() as u32;
        if !fun_021e8638(b, sp, item as u32) {
            let w = b.host.handex_push(0x20, m);
            let buf = b.host.handex(w);
            buf.put_u8(6, me);
            buf.put_u16(4, 0);
            b.host.handex_pop(w);
            let w = b.host.handex_push(0x20, m);
            let buf = b.host.handex(w);
            buf.put_u8(6, v6 as u8);
            buf.put_u16(4, item);
            str_param_setup(buf, 0xC, 2, 0x457);
            str_param_add_arg(buf, 0xC, t);
            str_param_add_arg(buf, 0xC, m);
            str_param_add_arg(buf, 0xC, item as u32);
            b.host.handex_pop(w);
        }
    }
}

/// C: unlabeled routine at `0x021E83CC`: when var 3 is the holder and it has a `1f6` record
/// `(a, b)`: `v` = `fun_021e8384(sub ID, b)`; if nonzero: `work[1]` = `v`, `work[0]` = 1,
/// `work[2]` = `a` and var 0x16 = 10 / 9 / 11 for `v` 1 / 2 / 3.
pub fn h_021e83cc<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    if let Some((a, c)) = b.mon(me as u32).get_1f6() {
        let v = fun_021e8384(sys.sub_id(f) as u32, c as u32);
        let cur = b.var(0x16) as u32 & 0xFF;
        if v != 0 {
            sys.set_work(f, 1, v);
            sys.set_work(f, 0, 1);
            sys.set_work(f, 2, a as u32);
            let ty = match v {
                1 => 10,
                2 => 9,
                3 => 0xB,
                _ => cur,
            };
            b.rewrite_var(0x16, ty as i32);
        }
    }
}

/// C: unlabeled routine at `0x021E86C8(flow, poke_id)`.
fn fun_021e86c8<H: FlowHost>(b: &mut Battle<H>, poke_id: u32) -> bool {
    !b.host.fun_021c8168() && b.host.fun_021b8ddc(poke_id) == 1
}

/// C: unlabeled routine at `0x021E86E8(flow, me, other)`: `me` cannot take `other`'s item.
fn fun_021e86e8<H: FlowHost>(b: &mut Battle<H>, me: u32, other: u32) -> bool {
    if fun_021e86c8(b, me) {
        return true;
    }
    if fun_021e869c(b, other) {
        return true;
    }
    let sp = b.mon(me).species() as u32;
    let item = b.mon(other).held_item() as u32;
    fun_021e8638(b, sp, item)
}

/// C: unlabeled routine at `0x021E2004`: when var 3 is the holder, sets var 0x1E to
/// `permanent_param(position of the holder)`.
pub fn h_021e2004<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let pos = b.host.poke_pos(me as u32);
        b.rewrite_var(0x1E, SickCont::permanent_param(pos).0 as i32);
    }
}

/// C: unlabeled routine at `0x021E3420`: when var 3 is the holder: `t` = var 6: unless
/// `fun_021e86e8(holder, t)`, if `t` holds an item `i` pushes HANDEX work 0x20 (+6 var 6, +4 =
/// 0, str 0x41A at +0xC with the holder, `t` and `i`).
pub fn h_021e3420<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let m = me as u32;
    let v6 = b.var(6) as u32;
    let t = v6 & 0xFF;
    if fun_021e86e8(b, m, t) {
        return;
    }
    let item = b.mon(t).held_item();
    if item != 0 {
        let w = b.host.handex_push(0x20, m);
        let buf = b.host.handex(w);
        buf.put_u8(6, v6 as u8);
        buf.put_u16(4, 0);
        str_param_setup(buf, 0xC, 2, 0x41A);
        str_param_add_arg(buf, 0xC, m);
        str_param_add_arg(buf, 0xC, t);
        str_param_add_arg(buf, 0xC, item as u32);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E36F4`: when var 3 is the holder, it holds nothing and
/// `work[0]` is 0: `t` = var 6 (not 0x1F) holding an item, with `fun_021e86e8(holder, t)`
/// false: pushes HANDEX work 0x24 (+4 var 6, str 0x421 at +8 with the holder, `t` and the item)
/// and work 0x3A (bit 24 set, +4 = 1).
pub fn h_021e36f4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let m = me as u32;
    if b.mon(m).held_item() != 0 || sys.work(f, 0) != 0 {
        return;
    }
    let v6 = b.var(6) as u32;
    let t = v6 & 0xFF;
    if t == 0x1F {
        return;
    }
    let item = b.mon(t).held_item();
    if item != 0 && !fun_021e86e8(b, m, t) {
        let w = b.host.handex_push(0x24, m);
        let it = b.mon(t).held_item();
        let buf = b.host.handex(w);
        buf.put_u8(4, v6 as u8);
        str_param_setup(buf, 8, 2, 0x421);
        str_param_add_arg(buf, 8, m);
        str_param_add_arg(buf, 8, t);
        str_param_add_arg(buf, 8, it as u32);
        b.host.handex_pop(w);
        let w = b.host.handex_push(0x3A, m);
        let buf = b.host.handex(w);
        let h = buf.get_u32(0) | 0x100_0000;
        buf.put_u32(0, h);
        buf.put_u8(4, 1);
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E37C0` (Switcheroo-style swap): when var 3 is the holder and
/// `fun_021e86c8` fails: `i1` / `i2` = the items of the holder / of `t` (var 6); if either
/// is held and neither is rejected by `fun_0202136c` nor locked to the other's species
/// (`fun_021e8638`): pushes HANDEX work 0x24 (+4 var 6, str 0x2AA at +8 with the holder,
/// str 0x2AD at +0x50 with the holder and `i2` if `i2`, str 0x2AD at +0x2C with `t` and `i1`
/// if `i1`).
pub fn h_021e37c0<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if !is_me(b, 3, me) {
        return;
    }
    let m = me as u32;
    let v6 = b.var(6) as u32;
    let t = v6 & 0xFF;
    if fun_021e86c8(b, m) {
        return;
    }
    let i1 = b.mon(m).held_item() as u32;
    let i2 = b.mon(t).held_item() as u32;
    if i1 == 0 && i2 == 0 {
        return;
    }
    let s1 = b.mon(m).species() as u32;
    let s2 = b.mon(t).species() as u32;
    if !fun_0202136c(i1)
        && !fun_0202136c(i2)
        && !fun_021e8638(b, s1, i2)
        && !fun_021e8638(b, s2, i1)
        && !fun_021e8638(b, s1, i1)
        && !fun_021e8638(b, s2, i2)
    {
        let w = b.host.handex_push(0x24, m);
        let buf = b.host.handex(w);
        buf.put_u8(4, v6 as u8);
        str_param_setup(buf, 8, 2, 0x2AA);
        str_param_add_arg(buf, 8, m);
        if i2 != 0 {
            str_param_setup(buf, 0x50, 2, 0x2AD);
            str_param_add_arg(buf, 0x50, m);
            str_param_add_arg(buf, 0x50, i2);
        }
        if i1 != 0 {
            str_param_setup(buf, 0x2C, 2, 0x2AD);
            str_param_add_arg(buf, 0x2C, t);
            str_param_add_arg(buf, 0x2C, i1);
        }
        b.host.handex_pop(w);
    }
}

/// C: unlabeled routine at `0x021E84FC`: when var 3 is the holder and `work[0]` is set: by
/// `work[1]` (1, 2, 3) pushes HANDEX work 0x19 (+4 = 0xB / 0xC / 0xD, +0xC the side of the
/// holder (1) or of var 6 (2, 3), +8 `turn(4)`, str 0xA4 / 0xA8 / 0xAC type 1 at +0x10 with that
/// side); then for the Pokemon (holder for 1, var 6 for 2 and 3) other than 0x1F whose
/// `fun_021c7d34` is not 6 pushes work 0x37 (+6 that value, +7 = 6, +4 `u16` 0x28D / 0x28E /
/// 0x28F).
pub fn h_021e84fc<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if !(is_me(b, 3, me) && sys.work(f, 0) != 0) {
        return;
    }
    let m = me as u32;
    let side_me = poke_side(m) as u32;
    let side_t = poke_side(b.var(6) as u32 & 0xFF) as u32;
    let (sid, kind, str_v, t, side) = match sys.work(f, 1) as i32 {
        1 => (0xA4u32, 0xBu32, 0x28Du32, m, side_me),
        2 => (0xA8, 0xC, 0x28E, b.var(6) as u32 & 0xFF, side_t),
        3 => (0xAC, 0xD, 0x28F, b.var(6) as u32 & 0xFF, side_t),
        _ => return,
    };
    let w = b.host.handex_push(0x19, m);
    let buf = b.host.handex(w);
    buf.put_u32(4, kind);
    buf.put_u8(0xC, side as u8);
    buf.put_u32(8, SickCont::turn(4).0);
    str_param_setup(buf, 0x10, 1, sid);
    str_param_add_arg(buf, 0x10, side);
    b.host.handex_pop(w);
    if t != 0x1F {
        let p = b.host.fun_021c7d34(t);
        if p != 6 {
            let w = b.host.handex_push(0x37, m);
            let buf = b.host.handex(w);
            buf.put_u8(6, p as u8);
            buf.put_u8(7, 6);
            buf.put_u16(4, str_v as u16);
            b.host.handex_pop(w);
        }
    }
}

/// C: unlabeled routine at `0x021E29E8`: when var 3 is the holder, `hp` = `hp_percent_fx` of the
/// Pokemon in var 4: var 0x30 = `max(fx_mul_min1(0x78, hp) / 100, 1)`.
pub fn h_021e29e8<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let t = b.var(4) as u32 & 0xFF;
        let hp = b.mon(t).hp_percent_fx();
        let mut v = u32_div(fx_mul_min1(0x78, hp as u32), 100);
        if v == 0 {
            v = 1;
        }
        b.rewrite_var(0x30, v as i32);
    }
}

/// C: unlabeled routine at `0x021E2A38`: when var 3 is the holder and the Pokemon in var 4 has at
/// most 50% HP (`hp_percent_fx <= 0x32000`, signed), var 0x31 is multiplied by 0x2000 (2.0).
pub fn h_021e2a38<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let t = b.var(4) as u32 & 0xFF;
        if b.mon(t).hp_percent_fx() <= 0x32000 {
            b.mul_var(0x31, 0x2000);
        }
    }
}

/// C: unlabeled routine at `0x021E2A80`: when var 3 is the holder: var 0x30 = `max(10 * v / 25,
/// 1)` with `v` = `pp_get(pokemon, 9, NULL)` (friendship).
pub fn h_021e2a80<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let pk = b.mon(me as u32).pokemon();
        let v = b.host.pp_get(pk, 9);
        let mut q = u32_div(v.wrapping_mul(10), 0x19);
        if q == 0 {
            q = 1;
        }
        b.rewrite_var(0x30, q as i32);
    }
}

/// C: unlabeled routine at `0x021E2ACC`: as `h_021e2a80` with `10 * (0xFF - v) / 25`.
pub fn h_021e2acc<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if is_me(b, 3, me) {
        let pk = b.mon(me as u32).pokemon();
        let v = b.host.pp_get(pk, 9);
        let mut q = u32_div(0xFFu32.wrapping_sub(v).wrapping_mul(10), 0x19);
        if q == 0 {
            q = 1;
        }
        b.rewrite_var(0x30, q as i32);
    }
}

/// Ported move handlers (regenerated by tools/sync-handlers.py).
pub const PORTED: &[u32] = &[
    0x021E0468, 0x021E0518, 0x021E05B0, 0x021E0628, 0x021E068C, 0x021E06DC, 0x021E072C, 0x021E0818,
    0x021E0838, 0x021E0850, 0x021E0920, 0x021E0998, 0x021E0A80, 0x021E0B68, 0x021E0BA4, 0x021E0BE8,
    0x021E0C6C, 0x021E0C8C, 0x021E0CE0, 0x021E0D54, 0x021E0DC4, 0x021E0E00, 0x021E0E28, 0x021E0F50,
    0x021E0F74, 0x021E0FB8, 0x021E101C, 0x021E1050, 0x021E1074, 0x021E10D8, 0x021E11B8, 0x021E11DC,
    0x021E12A4, 0x021E1320, 0x021E1354, 0x021E1410, 0x021E1434, 0x021E1478, 0x021E14C0, 0x021E14E4,
    0x021E1508, 0x021E1558, 0x021E15B0, 0x021E15E8, 0x021E163C, 0x021E1668, 0x021E1784, 0x021E17B0,
    0x021E17DC, 0x021E18EC, 0x021E1938, 0x021E1948, 0x021E1958, 0x021E197C, 0x021E198C, 0x021E199C,
    0x021E19C0, 0x021E19D0, 0x021E19E0, 0x021E1B4C, 0x021E1BD4, 0x021E1BF8, 0x021E1C4C, 0x021E1C8C,
    0x021E1D20, 0x021E1D78, 0x021E1D9C, 0x021E1DCC, 0x021E1DF4, 0x021E1E10, 0x021E1E44, 0x021E1EB8,
    0x021E1F18, 0x021E1F44, 0x021E1F70, 0x021E1F9C, 0x021E1FCC, 0x021E2004, 0x021E203C, 0x021E2098,
    0x021E20E0, 0x021E2148, 0x021E2188, 0x021E21F4, 0x021E2268, 0x021E231C, 0x021E237C, 0x021E249C,
    0x021E24D4, 0x021E2574, 0x021E25DC, 0x021E25FC, 0x021E2678, 0x021E269C, 0x021E2700, 0x021E2768,
    0x021E278C, 0x021E27B0, 0x021E281C, 0x021E2898, 0x021E2908, 0x021E2948, 0x021E2988, 0x021E29E8,
    0x021E2A38, 0x021E2A80, 0x021E2ACC, 0x021E2B18, 0x021E2B5C, 0x021E2BB4, 0x021E2BF4, 0x021E2C4C,
    0x021E2C88, 0x021E2D20, 0x021E2D5C, 0x021E2DC0, 0x021E2E28, 0x021E2E90, 0x021E2ED4, 0x021E2F48,
    0x021E2FB0, 0x021E2FE8, 0x021E300C, 0x021E3050, 0x021E3080, 0x021E30A4, 0x021E30C8, 0x021E30F8,
    0x021E3128, 0x021E314C, 0x021E3190, 0x021E324C, 0x021E32CC, 0x021E32F8, 0x021E3330, 0x021E338C,
    0x021E33BC, 0x021E33EC, 0x021E3420, 0x021E34B4, 0x021E34FC, 0x021E3544, 0x021E354C, 0x021E357C,
    0x021E35A0, 0x021E35F0, 0x021E3648, 0x021E36A4, 0x021E36C8, 0x021E36F4, 0x021E37C0, 0x021E3904,
    0x021E396C, 0x021E399C, 0x021E39C0, 0x021E3A04, 0x021E3A48, 0x021E3A78, 0x021E3AC4, 0x021E3AF0,
    0x021E3B14, 0x021E3B64, 0x021E3BBC, 0x021E3BF4, 0x021E3CB0, 0x021E3D38, 0x021E3D98, 0x021E3DCC,
    0x021E3E00, 0x021E3E30, 0x021E3E64, 0x021E3F44, 0x021E3FC0, 0x021E4080, 0x021E4154, 0x021E4220,
    0x021E4238, 0x021E42B0, 0x021E4340, 0x021E4408, 0x021E448C, 0x021E4564, 0x021E45CC, 0x021E4688,
    0x021E4754, 0x021E47CC, 0x021E4828, 0x021E48C0, 0x021E492C, 0x021E4948, 0x021E4A70, 0x021E4B00,
    0x021E4BB0, 0x021E4C74, 0x021E4D98, 0x021E4EC4, 0x021E4FF4, 0x021E508C, 0x021E517C, 0x021E5264,
    0x021E52D4, 0x021E5318, 0x021E535C, 0x021E53A0, 0x021E53E4, 0x021E5428, 0x021E546C, 0x021E54AC,
    0x021E54EC, 0x021E552C, 0x021E55C4, 0x021E5620, 0x021E5680, 0x021E56E0, 0x021E5720, 0x021E5748,
    0x021E5800, 0x021E5828, 0x021E5870, 0x021E58D8, 0x021E5974, 0x021E5A34, 0x021E5B78, 0x021E5C18,
    0x021E5C64, 0x021E5C88, 0x021E5CD4, 0x021E5D24, 0x021E5D54, 0x021E5DAC, 0x021E5E2C, 0x021E5E7C,
    0x021E5EB8, 0x021E5F18, 0x021E5F30, 0x021E5F48, 0x021E5FD0, 0x021E5FEC, 0x021E6010, 0x021E6054,
    0x021E612C, 0x021E6164, 0x021E61BC, 0x021E6210, 0x021E6220, 0x021E6274, 0x021E6278, 0x021E62F4,
    0x021E6348, 0x021E6358, 0x021E63AC, 0x021E63BC, 0x021E6410, 0x021E6420, 0x021E6444, 0x021E6480,
    0x021E64B8, 0x021E6500, 0x021E6548, 0x021E65B0, 0x021E6654, 0x021E666C, 0x021E66A8, 0x021E66CC,
    0x021E6710, 0x021E6728, 0x021E6800, 0x021E6840, 0x021E6924, 0x021E695C, 0x021E6994, 0x021E69E8,
    0x021E6A84, 0x021E6AD8, 0x021E6B20, 0x021E6BAC, 0x021E6BF4, 0x021E6CDC, 0x021E6DAC, 0x021E6E38,
    0x021E6F14, 0x021E6F3C, 0x021E6F98, 0x021E6FC8, 0x021E700C, 0x021E704C, 0x021E7080, 0x021E70E4,
    0x021E7164, 0x021E71E4, 0x021E727C, 0x021E7300, 0x021E732C, 0x021E7388, 0x021E7420, 0x021E74E4,
    0x021E7554, 0x021E75E0, 0x021E7634, 0x021E7760, 0x021E781C, 0x021E78B0, 0x021E7924, 0x021E795C,
    0x021E7974, 0x021E7998, 0x021E7A08, 0x021E7A78, 0x021E7B40, 0x021E7B98, 0x021E7C78, 0x021E7CAC,
    0x021E7D00, 0x021E7D54, 0x021E7DB0, 0x021E7E0C, 0x021E7E58, 0x021E7F1C, 0x021E7F6C, 0x021E7FB4,
    0x021E7FE4, 0x021E8070, 0x021E80C0, 0x021E8104, 0x021E814C, 0x021E8190, 0x021E8200, 0x021E8238,
    0x021E8270, 0x021E82AC, 0x021E8310, 0x021E83CC, 0x021E8440, 0x021E8478, 0x021E8498, 0x021E84B8,
    0x021E84FC,
];

/// Ported move handlers by original address.
pub fn handler<H: FlowHost>(addr: u32) -> Option<crate::flow::Handler<H>> {
    Some(match addr & !1 {
        0x021E0468 => h_021e0468::<H>,
        0x021E0518 => h_021e0518::<H>,
        0x021E05B0 => h_021e05b0::<H>,
        0x021E0628 => h_021e0628::<H>,
        0x021E068C => h_021e068c::<H>,
        0x021E06DC => h_021e06dc::<H>,
        0x021E072C => h_021e072c::<H>,
        0x021E0818 => h_021e0818::<H>,
        0x021E0838 => h_021e0838::<H>,
        0x021E0850 => h_021e0850::<H>,
        0x021E0920 => h_021e0920::<H>,
        0x021E0998 => h_021e0998::<H>,
        0x021E0A80 => h_021e0a80::<H>,
        0x021E0B68 => h_021e0b68::<H>,
        0x021E0BA4 => h_021e0ba4::<H>,
        0x021E0BE8 => h_021e0be8::<H>,
        0x021E0C6C => h_021e0c6c::<H>,
        0x021E0C8C => h_021e0c8c::<H>,
        0x021E0CE0 => h_021e0ce0::<H>,
        0x021E0D54 => h_021e0d54::<H>,
        0x021E0DC4 => h_021e0dc4::<H>,
        0x021E0E00 => h_021e0e00::<H>,
        0x021E0E28 => h_021e0e28::<H>,
        0x021E0F50 => h_021e0f50::<H>,
        0x021E0F74 => h_021e0f74::<H>,
        0x021E0FB8 => h_021e0fb8::<H>,
        0x021E101C => h_021e101c::<H>,
        0x021E1050 => h_021e1050::<H>,
        0x021E1074 => h_021e1074::<H>,
        0x021E10D8 => h_021e10d8::<H>,
        0x021E11B8 => h_021e11b8::<H>,
        0x021E11DC => h_021e11dc::<H>,
        0x021E12A4 => h_021e12a4::<H>,
        0x021E1320 => h_021e1320::<H>,
        0x021E1354 => h_021e1354::<H>,
        0x021E1410 => h_021e1410::<H>,
        0x021E1434 => h_021e1434::<H>,
        0x021E1478 => h_021e1478::<H>,
        0x021E14C0 => h_021e14c0::<H>,
        0x021E14E4 => h_021e14e4::<H>,
        0x021E1508 => h_021e1508::<H>,
        0x021E1558 => h_021e1558::<H>,
        0x021E15B0 => h_021e15b0::<H>,
        0x021E15E8 => h_021e15e8::<H>,
        0x021E163C => h_021e163c::<H>,
        0x021E1668 => h_021e1668::<H>,
        0x021E1784 => h_021e1784::<H>,
        0x021E17B0 => h_021e17b0::<H>,
        0x021E17DC => h_021e17dc::<H>,
        0x021E18EC => h_021e18ec::<H>,
        0x021E1938 => h_021e1938::<H>,
        0x021E1948 => h_021e1948::<H>,
        0x021E1958 => h_021e1958::<H>,
        0x021E197C => h_021e197c::<H>,
        0x021E198C => h_021e198c::<H>,
        0x021E199C => h_021e199c::<H>,
        0x021E19C0 => h_021e19c0::<H>,
        0x021E19D0 => h_021e19d0::<H>,
        0x021E19E0 => h_021e19e0::<H>,
        0x021E1B4C => h_021e1b4c::<H>,
        0x021E1BD4 => h_021e1bd4::<H>,
        0x021E1BF8 => h_021e1bf8::<H>,
        0x021E1C4C => h_021e1c4c::<H>,
        0x021E1C8C => h_021e1c8c::<H>,
        0x021E1D20 => h_021e1d20::<H>,
        0x021E1D78 => h_021e1d78::<H>,
        0x021E1D9C => h_021e1d9c::<H>,
        0x021E1DCC => h_021e1dcc::<H>,
        0x021E1DF4 => h_021e1df4::<H>,
        0x021E1E10 => h_021e1e10::<H>,
        0x021E1E44 => h_021e1e44::<H>,
        0x021E1EB8 => h_021e1eb8::<H>,
        0x021E1F18 => h_021e1f18::<H>,
        0x021E1F44 => h_021e1f44::<H>,
        0x021E1F70 => h_021e1f70::<H>,
        0x021E1F9C => h_021e1f9c::<H>,
        0x021E1FCC => h_021e1fcc::<H>,
        0x021E2004 => h_021e2004::<H>,
        0x021E203C => h_021e203c::<H>,
        0x021E2098 => h_021e2098::<H>,
        0x021E20E0 => h_021e20e0::<H>,
        0x021E2148 => h_021e2148::<H>,
        0x021E2188 => h_021e2188::<H>,
        0x021E21F4 => h_021e21f4::<H>,
        0x021E2268 => h_021e2268::<H>,
        0x021E231C => h_021e231c::<H>,
        0x021E237C => h_021e237c::<H>,
        0x021E249C => h_021e249c::<H>,
        0x021E24D4 => h_021e24d4::<H>,
        0x021E2574 => h_021e2574::<H>,
        0x021E25DC => h_021e25dc::<H>,
        0x021E25FC => h_021e25fc::<H>,
        0x021E2678 => h_021e2678::<H>,
        0x021E269C => h_021e269c::<H>,
        0x021E2700 => h_021e2700::<H>,
        0x021E2768 => h_021e2768::<H>,
        0x021E278C => h_021e278c::<H>,
        0x021E27B0 => h_021e27b0::<H>,
        0x021E281C => h_021e281c::<H>,
        0x021E2898 => h_021e2898::<H>,
        0x021E2908 => h_021e2908::<H>,
        0x021E2948 => h_021e2948::<H>,
        0x021E2988 => h_021e2988::<H>,
        0x021E29E8 => h_021e29e8::<H>,
        0x021E2A38 => h_021e2a38::<H>,
        0x021E2A80 => h_021e2a80::<H>,
        0x021E2ACC => h_021e2acc::<H>,
        0x021E2B18 => h_021e2b18::<H>,
        0x021E2B5C => h_021e2b5c::<H>,
        0x021E2BB4 => h_021e2bb4::<H>,
        0x021E2BF4 => h_021e2bf4::<H>,
        0x021E2C4C => h_021e2c4c::<H>,
        0x021E2C88 => h_021e2c88::<H>,
        0x021E2D20 => h_021e2d20::<H>,
        0x021E2D5C => h_021e2d5c::<H>,
        0x021E2DC0 => h_021e2dc0::<H>,
        0x021E2E28 => h_021e2e28::<H>,
        0x021E2E90 => h_021e2e90::<H>,
        0x021E2ED4 => h_021e2ed4::<H>,
        0x021E2F48 => h_021e2f48::<H>,
        0x021E2FB0 => h_021e2fb0::<H>,
        0x021E2FE8 => h_021e2fe8::<H>,
        0x021E300C => h_021e300c::<H>,
        0x021E3050 => h_021e3050::<H>,
        0x021E3080 => h_021e3080::<H>,
        0x021E30A4 => h_021e30a4::<H>,
        0x021E30C8 => h_021e30c8::<H>,
        0x021E30F8 => h_021e30f8::<H>,
        0x021E3128 => h_021e3128::<H>,
        0x021E314C => h_021e314c::<H>,
        0x021E3190 => h_021e3190::<H>,
        0x021E324C => h_021e324c::<H>,
        0x021E32CC => h_021e32cc::<H>,
        0x021E32F8 => h_021e32f8::<H>,
        0x021E3330 => h_021e3330::<H>,
        0x021E338C => h_021e338c::<H>,
        0x021E33BC => h_021e33bc::<H>,
        0x021E33EC => h_021e33ec::<H>,
        0x021E3420 => h_021e3420::<H>,
        0x021E34B4 => h_021e34b4::<H>,
        0x021E34FC => h_021e34fc::<H>,
        0x021E3544 => h_021e3544::<H>,
        0x021E354C => h_021e354c::<H>,
        0x021E357C => h_021e357c::<H>,
        0x021E35A0 => h_021e35a0::<H>,
        0x021E35F0 => h_021e35f0::<H>,
        0x021E3648 => h_021e3648::<H>,
        0x021E36A4 => h_021e36a4::<H>,
        0x021E36C8 => h_021e36c8::<H>,
        0x021E36F4 => h_021e36f4::<H>,
        0x021E37C0 => h_021e37c0::<H>,
        0x021E3904 => h_021e3904::<H>,
        0x021E396C => h_021e396c::<H>,
        0x021E399C => h_021e399c::<H>,
        0x021E39C0 => h_021e39c0::<H>,
        0x021E3A04 => h_021e3a04::<H>,
        0x021E3A48 => h_021e3a48::<H>,
        0x021E3A78 => h_021e3a78::<H>,
        0x021E3AC4 => h_021e3ac4::<H>,
        0x021E3AF0 => h_021e3af0::<H>,
        0x021E3B14 => h_021e3b14::<H>,
        0x021E3B64 => h_021e3b64::<H>,
        0x021E3BBC => h_021e3bbc::<H>,
        0x021E3BF4 => h_021e3bf4::<H>,
        0x021E3CB0 => h_021e3cb0::<H>,
        0x021E3D38 => h_021e3d38::<H>,
        0x021E3D98 => h_021e3d98::<H>,
        0x021E3DCC => h_021e3dcc::<H>,
        0x021E3E00 => h_021e3e00::<H>,
        0x021E3E30 => h_021e3e30::<H>,
        0x021E3E64 => h_021e3e64::<H>,
        0x021E3F44 => h_021e3f44::<H>,
        0x021E3FC0 => h_021e3fc0::<H>,
        0x021E4080 => h_021e4080::<H>,
        0x021E4154 => h_021e4154::<H>,
        0x021E4220 => h_021e4220::<H>,
        0x021E4238 => h_021e4238::<H>,
        0x021E42B0 => h_021e42b0::<H>,
        0x021E4340 => h_021e4340::<H>,
        0x021E4408 => h_021e4408::<H>,
        0x021E448C => h_021e448c::<H>,
        0x021E4564 => h_021e4564::<H>,
        0x021E45CC => h_021e45cc::<H>,
        0x021E4688 => h_021e4688::<H>,
        0x021E4754 => h_021e4754::<H>,
        0x021E47CC => h_021e47cc::<H>,
        0x021E4828 => h_021e4828::<H>,
        0x021E48C0 => h_021e48c0::<H>,
        0x021E492C => h_021e492c::<H>,
        0x021E4948 => h_021e4948::<H>,
        0x021E4A70 => h_021e4a70::<H>,
        0x021E4B00 => h_021e4b00::<H>,
        0x021E4BB0 => h_021e4bb0::<H>,
        0x021E4C74 => h_021e4c74::<H>,
        0x021E4D98 => h_021e4d98::<H>,
        0x021E4EC4 => h_021e4ec4::<H>,
        0x021E4FF4 => h_021e4ff4::<H>,
        0x021E508C => h_021e508c::<H>,
        0x021E517C => h_021e517c::<H>,
        0x021E5264 => h_021e5264::<H>,
        0x021E52D4 => h_021e52d4::<H>,
        0x021E5318 => h_021e5318::<H>,
        0x021E535C => h_021e535c::<H>,
        0x021E53A0 => h_021e53a0::<H>,
        0x021E53E4 => h_021e53e4::<H>,
        0x021E5428 => h_021e5428::<H>,
        0x021E546C => h_021e546c::<H>,
        0x021E54AC => h_021e54ac::<H>,
        0x021E54EC => h_021e54ec::<H>,
        0x021E552C => h_021e552c::<H>,
        0x021E55C4 => h_021e55c4::<H>,
        0x021E5620 => h_021e5620::<H>,
        0x021E5680 => h_021e5680::<H>,
        0x021E56E0 => h_021e56e0::<H>,
        0x021E5720 => h_021e5720::<H>,
        0x021E5748 => h_021e5748::<H>,
        0x021E5800 => h_021e5800::<H>,
        0x021E5828 => h_021e5828::<H>,
        0x021E5870 => h_021e5870::<H>,
        0x021E58D8 => h_021e58d8::<H>,
        0x021E5974 => h_021e5974::<H>,
        0x021E5A34 => h_021e5a34::<H>,
        0x021E5B78 => h_021e5b78::<H>,
        0x021E5C18 => h_021e5c18::<H>,
        0x021E5C64 => h_021e5c64::<H>,
        0x021E5C88 => h_021e5c88::<H>,
        0x021E5CD4 => h_021e5cd4::<H>,
        0x021E5D24 => h_021e5d24::<H>,
        0x021E5D54 => h_021e5d54::<H>,
        0x021E5DAC => h_021e5dac::<H>,
        0x021E5E2C => h_021e5e2c::<H>,
        0x021E5E7C => h_021e5e7c::<H>,
        0x021E5EB8 => h_021e5eb8::<H>,
        0x021E5F18 => h_021e5f18::<H>,
        0x021E5F30 => h_021e5f30::<H>,
        0x021E5F48 => h_021e5f48::<H>,
        0x021E5FD0 => h_021e5fd0::<H>,
        0x021E5FEC => h_021e5fec::<H>,
        0x021E6010 => h_021e6010::<H>,
        0x021E6054 => h_021e6054::<H>,
        0x021E612C => h_021e612c::<H>,
        0x021E6164 => h_021e6164::<H>,
        0x021E61BC => h_021e61bc::<H>,
        0x021E6210 => h_021e6210::<H>,
        0x021E6220 => h_021e6220::<H>,
        0x021E6274 => h_021e6274::<H>,
        0x021E6278 => h_021e6278::<H>,
        0x021E62F4 => h_021e62f4::<H>,
        0x021E6348 => h_021e6348::<H>,
        0x021E6358 => h_021e6358::<H>,
        0x021E63AC => h_021e63ac::<H>,
        0x021E63BC => h_021e63bc::<H>,
        0x021E6410 => h_021e6410::<H>,
        0x021E6420 => h_021e6420::<H>,
        0x021E6444 => h_021e6444::<H>,
        0x021E6480 => h_021e6480::<H>,
        0x021E64B8 => h_021e64b8::<H>,
        0x021E6500 => h_021e6500::<H>,
        0x021E6548 => h_021e6548::<H>,
        0x021E65B0 => h_021e65b0::<H>,
        0x021E6654 => h_021e6654::<H>,
        0x021E666C => h_021e666c::<H>,
        0x021E66A8 => h_021e66a8::<H>,
        0x021E66CC => h_021e66cc::<H>,
        0x021E6710 => h_021e6710::<H>,
        0x021E6728 => h_021e6728::<H>,
        0x021E6800 => h_021e6800::<H>,
        0x021E6840 => h_021e6840::<H>,
        0x021E6924 => h_021e6924::<H>,
        0x021E695C => h_021e695c::<H>,
        0x021E6994 => h_021e6994::<H>,
        0x021E69E8 => h_021e69e8::<H>,
        0x021E6A84 => h_021e6a84::<H>,
        0x021E6AD8 => h_021e6ad8::<H>,
        0x021E6B20 => h_021e6b20::<H>,
        0x021E6BAC => h_021e6bac::<H>,
        0x021E6BF4 => h_021e6bf4::<H>,
        0x021E6CDC => h_021e6cdc::<H>,
        0x021E6DAC => h_021e6dac::<H>,
        0x021E6E38 => h_021e6e38::<H>,
        0x021E6F14 => h_021e6f14::<H>,
        0x021E6F3C => h_021e6f3c::<H>,
        0x021E6F98 => h_021e6f98::<H>,
        0x021E6FC8 => h_021e6fc8::<H>,
        0x021E700C => h_021e700c::<H>,
        0x021E704C => h_021e704c::<H>,
        0x021E7080 => h_021e7080::<H>,
        0x021E70E4 => h_021e70e4::<H>,
        0x021E7164 => h_021e7164::<H>,
        0x021E71E4 => h_021e71e4::<H>,
        0x021E727C => h_021e727c::<H>,
        0x021E7300 => h_021e7300::<H>,
        0x021E732C => h_021e732c::<H>,
        0x021E7388 => h_021e7388::<H>,
        0x021E7420 => h_021e7420::<H>,
        0x021E74E4 => h_021e74e4::<H>,
        0x021E7554 => h_021e7554::<H>,
        0x021E75E0 => h_021e75e0::<H>,
        0x021E7634 => h_021e7634::<H>,
        0x021E7760 => h_021e7760::<H>,
        0x021E781C => h_021e781c::<H>,
        0x021E78B0 => h_021e78b0::<H>,
        0x021E7924 => h_021e7924::<H>,
        0x021E795C => h_021e795c::<H>,
        0x021E7974 => h_021e7974::<H>,
        0x021E7998 => h_021e7998::<H>,
        0x021E7A08 => h_021e7a08::<H>,
        0x021E7A78 => h_021e7a78::<H>,
        0x021E7B40 => h_021e7b40::<H>,
        0x021E7B98 => h_021e7b98::<H>,
        0x021E7C78 => h_021e7c78::<H>,
        0x021E7CAC => h_021e7cac::<H>,
        0x021E7D00 => h_021e7d00::<H>,
        0x021E7D54 => h_021e7d54::<H>,
        0x021E7DB0 => h_021e7db0::<H>,
        0x021E7E0C => h_021e7e0c::<H>,
        0x021E7E58 => h_021e7e58::<H>,
        0x021E7F1C => h_021e7f1c::<H>,
        0x021E7F6C => h_021e7f6c::<H>,
        0x021E7FB4 => h_021e7fb4::<H>,
        0x021E7FE4 => h_021e7fe4::<H>,
        0x021E8070 => h_021e8070::<H>,
        0x021E80C0 => h_021e80c0::<H>,
        0x021E8104 => h_021e8104::<H>,
        0x021E814C => h_021e814c::<H>,
        0x021E8190 => h_021e8190::<H>,
        0x021E8200 => h_021e8200::<H>,
        0x021E8238 => h_021e8238::<H>,
        0x021E8270 => h_021e8270::<H>,
        0x021E82AC => h_021e82ac::<H>,
        0x021E8310 => h_021e8310::<H>,
        0x021E83CC => h_021e83cc::<H>,
        0x021E8440 => h_021e8440::<H>,
        0x021E8478 => h_021e8478::<H>,
        0x021E8498 => h_021e8498::<H>,
        0x021E84B8 => h_021e84b8::<H>,
        0x021E84FC => h_021e84fc::<H>,
        _ => return None,
    })
}

#[cfg(test)]
mod tests;
