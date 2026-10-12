//! Overlay-95 effect handlers (B7): the three registries of overlay 95 (`OVY_95`, linked at
//! 0x06898020, code 0x06898020-0x0689E7E0; the address is the link address in the battle set
//! ELF and the ROM build, so `0x0689xxxx` is right; the `0x021EF71C/0x021EF7AC` "overlay-95
//! visuals" noted for `battle_field` are call sites in overlay 93 that branch into it).
//!
//! # Registries (decoded by B7h)
//!
//! Overlay 95 holds the handler registries of the effects that are not tied to a Pokemon. All
//! three use the getter protocol of abilities, items and moves: a getter is
//! `movs r1,#n; str r1,[r0]; ldr r0,=table; bx lr` (asserted by the harness), so
//! `table = getter(&out)` and `out & 0xFFFF` = number of `(u32 event ID, thumb fn)` pairs. The
//! adders pass the table and count to `EventSystem::add` (`021d6f0c(type, sub ID, class,
//! priority, owner, table, count)`, reached through the thunk `06898280`), which stores the type
//! at +0x10, the key as sub ID (+0x38), the owner at +0x3A (the `poke_id` argument of every
//! handler, see `021d713c`) and `0x1F` at +0x3B (only types 0, 4 and 5 have an owning Pokemon).
//!
//! | registry | entries | adder | type / class | owner (`+0x3A`) |
//! |---|---|---|---|---|
//! | field effects 0x0689D650 | 8 `(key, getter)`, stride 8 | `06898020(key, owner = 0)` (thunk `021EFA84`, `Field::add_effect`); remove `06898080(factor)` | 3 / 3 | 0 |
//! | position effects 0x0689D6C8 | 5 `(key, getter)`, stride 8 | `068981f0(key, position, user, values, n)` (thunk `021C9A50`, HANDEX proc 0x1E `021c9a24`) | 1 / 1 | the battle position |
//! | side effects 0x0689D780 | 14 `(key, getter, max layers)`, stride 12 | `06898c10(side, key, cont)` (thunk `021CBFF8`; HANDEX proc 0x19 `021c97d0`); remove `06898d10(side, key)` (`021c988c`, proc 0x1A `021c9828`) | 2 / 2 | the side (0/1) |
//!
//! The side-effect registry is the table the literal at `0x06898cb8` points to (the adder's
//! own table); B4 had not found it. **The "type-2 adder at 0x06898328" noted by B4 is not an
//! adder**: `06898328` is the key-0 handler of the position registry (Wish). The real type-2
//! adder is `06898c10`.
//!
//! * **Position effects (type 1) belong to a battle position, not a side.** Evidence: the move
//!   handlers that start them push HANDEX work 0x1E with `+8 = poke_pos_in_battle(holder)` (the
//!   holder's *position*, see `moves.rs` `h_021e56e0`), the adder's second argument (`char
//!   param_2[2]` of the proc) becomes the owner byte, and the duplicate check `068982c0(key,
//!   position)` walks `021d732c(1, position)` (type at +0x10 and owner at +0x3A) and refuses a
//!   second factor with the same key (+0x38) at the same position. The other proc fields are
//!   the key (`+4`), initial work words (copied to work 0..n-1 by `0689828c`) and the user's
//!   poke ID (work 6, set by `0689829c`). Keys (moves found through the move handlers that
//!   push them): 0 Wish (move 273; `06898328`, event 0x76: stores the turn, later
//!   heals the position's Pokemon, message 700), 1 Lunar Dance (461), 2 Healing Wish (361)
//!   (keys 1 and 2: events 0x55 and 0xA4), 3 Future Sight / Doom Desire (+0x10 =
//!   move ID, due turn + 2, event 0x76), 4 Baton Pass (226, event 0x55).
//! * **Side effects (type 2) belong to a side.** `06898c10` keeps a state array at 0x0689E7E0
//!   (BSS, past the end of `OVY_95`): `side * 0xE0 + key * 0x10`, per entry `{factor, cont, 0,
//!   layers}`. The first add creates the factor (`owner = side`, work 6 = `cont`) with layers 1;
//!   later adds while `layers < max layers` (third registry field) only raise `layers`; else it
//!   returns 0. `06898de0(factor, side)` reads the layer count of the factor's key. The max
//!   layers are 1 except key 6 (3) and key 7 (2). Keys (from the move handlers that push HANDEX
//!   0x19 with that key in `+4`): 0 Reflect (115), 1 Light Screen (113), 2 Safeguard (219),
//!   3 Mist (54), 4 Tailwind (366), 5 Lucky Chant (381), 6 Spikes (191), 7 Toxic Spikes (390),
//!   8 Stealth Rock (446), 9 Wide Guard (469), 10 Quick Guard (501), 11-13 the Pledge combos
//!   (Rainbow, Sea of Fire, Swamp; the pledge handler pushes kinds 0xB, 0xC, 0xD). Keys 0 and 1
//!   share the helper `06898e54(flow, side, damage class)` (a tail call, not a handler of its
//!   own; port it with them).
//! * **Field effects** are the `btl_field.c` slots (`black-battle::field`). Keys and the
//!   moves that start them (`field_effect_start` in `moves.rs`): 1 Trick Room (433; rewrites
//!   var 0x4A), 2 Gravity (356; accuracy x 0x1AB8/4096, ground immunity off), 4 Water Sport,
//!   5 Mud Sport (power x 0x548/4096 for fire / electric moves; guessed from the types 9 / 12),
//!   6 Wonder Room (472), 7 Magic Room (478), 3 Imprison (`Field::foe_knows_move`). Keys 3 and 7
//!   use the no-op `068981ec` (event 0, no hooks); key 0 (`06898094`, event 0x47) is a no-op
//!   too and is not started by any move handler found yet.
//!
//! # Harness
//!
//! `handlers.py effects` / `tools/check-handlers.sh effects`: overlay 95 is loaded at its link
//! address next to overlay 93 (the `bx pc` veneers inside it reach the real overlay-93
//! functions or their harness stubs). The factor is of the handler's registry kind, the sub ID
//! a key using the handler (else any key of the kind), the owner 0 / `me % 6` / `me & 1`
//! (`me` = the usual random Pokemon). Stubs: the move-mode stubs plus `06898de0` (tag 0x155).
//! `handlers.py effects --list` prints the 30 handlers with kind, keys and events. See
//! `tools/rom-diff/README.md`. Event IDs are kept numeric and their meaning is unverified.

use crate::event_factor::{EventSystem, FactorId};
use crate::flow::{str_param_add_arg, str_param_setup, Battle, FlowHost, HandexBytes};
use crate::handlers::moves::fun_021d7d6c;
use crate::main_util::poke_side;
use crate::sick_cont::SickCont;

/// 0x06898020-0x0689E7E0 is loaded at this address.
pub const OVY95_BASE: u32 = 0x0689_8020;

/// The poke ID in var `label` (a byte), as the original reads it (`var & 0xFF`).
fn var_poke<H: FlowHost>(b: &Battle<H>, label: u16) -> u32 {
    b.var(label) as u32 & 0xFF
}

/// C: `sub_06898094`: field-effect key 0 (event 0x47): no hook.
pub fn h_06898094<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, _b: &mut Battle<H>, _me: u8) {}

/// C: `sub_068980a4`: field-effect key 1 (Trick Room, event 0x13): tail call `021d76bc(0x4A, 1)`.
pub fn h_068980a4<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, _me: u8) {
    b.rewrite_var(0x4A, 1);
}

/// 0x068980BC: field-effect slot 2 (Gravity), event 0x34: multiplies var 0x35 (accuracy) by
/// 0x1AB8 / 4096 (about 1.67).
pub fn h_068980bc<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, _: u8) {
    b.mul_var(0x35, 0x1AB8);
}

/// C: `sub_068980cc`: field-effect key 2 (Gravity, event 0x3E): when var 0x16 is 4, rewrites
/// var 0x4B to 1 (ground immunity off).
pub fn h_068980cc<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, _me: u8) {
    if b.var(0x16) == 4 {
        b.rewrite_var(0x4B, 1);
    }
}

/// C: `sub_06898110`: field-effect key 4 (Water Sport, event 0x38): when var 0x16 is 9,
/// multiplies var 0x31 by 0x548 / 4096.
pub fn h_06898110<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, _me: u8) {
    if b.var(0x16) == 9 {
        b.mul_var(0x31, 0x548);
    }
}

/// C: `sub_06898158`: field-effect key 5 (Mud Sport, event 0x38): when var 0x16 is 12,
/// multiplies var 0x31 by 0x548 / 4096.
pub fn h_06898158<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, _me: u8) {
    if b.var(0x16) == 12 {
        b.mul_var(0x31, 0x548);
    }
}

/// C: `sub_068981a0`: field-effect key 6 (Wonder Room, event 0x3A): var 0x3D += 1.
pub fn h_068981a0<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, _me: u8) {
    let v = b.var(0x3D);
    b.rewrite_var(0x3D, v.wrapping_add(1));
}

/// C: `sub_068981ec`: field-effect keys 3 and 7 (event 0): no hook.
pub fn h_068981ec<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, _b: &mut Battle<H>, _me: u8) {}

/// 0x068991B0: side-effect key 4 (Tailwind, move 366), event 0x13: when the side of the Pokemon in
/// var 2 (a byte) is the factor's side (`owner`), multiplies var 0x35 (speed) by 2.0.
pub fn h_068991b0<H: FlowHost>(_: &mut EventSystem, _: FactorId, b: &mut Battle<H>, me: u8) {
    if me == poke_side(b.var(2) as u32 & 0xFF) {
        b.mul_var(0x35, 2 << 12);
    }
}

/// C: `sub_06898328`: position-effect key 0 (Wish, move 273, event 0x76). On the first call
/// stores the turn and sets work 0; later, once the turn has advanced past work 1, heals the
/// position's Pokemon by half of the user's max HP (work 6) and removes itself (message 700).
pub fn h_06898328<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if b.var(2) != 0x1F || b.host.fun_021c83d4() {
        return;
    }
    if sys.work(f, 0) == 0 {
        let turn = b.host.fun_021c8138();
        sys.set_work(f, 1, turn);
        sys.set_work(f, 0, 1);
        return;
    }
    let turn = b.host.fun_021c8138();
    if turn <= sys.work(f, 1) {
        return;
    }
    let pos = b.host.poke_id_at_pos(me as u32);
    if pos != 0x1F && !b.mon_is_full_hp(pos) {
        let user = sys.work(f, 6) & 0xFF;
        let hp = b.max_hp_div_min1(user, 2);
        let w = b.host.handex_push(5, user);
        let buf = b.host.handex(w);
        buf.put_u8(6, pos as u8);
        buf.put_u16(4, hp as u16);
        str_param_setup(buf, 8, 2, 700);
        str_param_add_arg(buf, 8, user);
        b.host.handex_pop(w);
    }
    sys.remove(f);
}

/// C: `sub_068984c8`: position-effect key 1 (Lunar Dance, move 461; events 0x55 and 0xA4). When
/// the holder (var 2) is at position `me`, and it is not at full HP, has a major status or a
/// move with missing PP, pushes HANDEX works 0x37 (heal, message 0x2B6), 5 (HP), 0xB (status) and
/// 9 (PP per move) as needed, then removes the factor.
pub fn h_068984c8<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    let poke = var_poke(b, 2);
    if me as u32 != b.host.poke_pos_in_battle(poke) {
        return;
    }
    let mut flag = !b.mon_is_full_hp(poke);
    if b.mon(poke).major_status() != 0 {
        flag = true;
    }
    let moves = b.mon(poke).move_count();
    for i in 0..moves {
        if b.mon(poke).move_pp_missing(i as u32) != 0 {
            flag = true;
        }
    }
    if flag {
        let w = b.host.handex_push(0x37, poke);
        let buf = b.host.handex(w);
        buf.put_u16(4, 0x280);
        buf.put_u8(6, me);
        buf.put_u8(7, 6);
        buf.put_u8(0xB, 1);
        str_param_setup(buf, 0xC, 2, 0x2B6);
        str_param_add_arg(buf, 0xC, poke);
        b.host.handex_pop(w);

        if !b.mon_is_full_hp(poke) {
            let hp = b.mon_get(poke, 0xE).wrapping_sub(b.mon_get(poke, 0xD));
            let w = b.host.handex_push(5, poke);
            let buf = b.host.handex(w);
            buf.put_u8(6, poke as u8);
            buf.put_u16(4, hp as u16);
            b.host.handex_pop(w);
        }
        if b.mon(poke).major_status() != 0 {
            let w = b.host.handex_push(0xB, poke);
            let buf = b.host.handex(w);
            buf.put_u8(0x14, 1);
            buf.put_u8(8, poke as u8);
            buf.put_u32(4, 0x24);
            b.host.handex_pop(w);
        }
        let moves = b.mon(poke).move_count();
        for i in 0..moves {
            let pp = b.mon(poke).move_pp_missing(i as u32);
            if pp != 0 {
                let w = b.host.handex_push(9, poke);
                let buf = b.host.handex(w);
                buf.put_u8(5, poke as u8);
                buf.put_u8(4, pp);
                buf.put_u8(6, i);
                b.host.handex_pop(w);
            }
        }
    }
    sys.remove(f);
}

/// C: `sub_068987b4`: position-effect key 2 (Healing Wish, move 361; events 0x55 and 0xA4). When
/// the holder is at position `me` and is not at full HP or has a major status, pushes HANDEX
/// works 0x37 (message 0x2B9), 5 (HP) and 0xB (status), then removes the factor.
pub fn h_068987b4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    let poke = var_poke(b, 2);
    if me as u32 != b.host.poke_pos_in_battle(poke) {
        return;
    }
    let mut flag = !b.mon_is_full_hp(poke);
    if b.mon(poke).major_status() != 0 {
        flag = true;
    }
    if flag {
        let w = b.host.handex_push(0x37, poke);
        let buf = b.host.handex(w);
        buf.put_u16(4, 0x27F);
        buf.put_u8(6, me);
        buf.put_u8(7, 6);
        str_param_setup(buf, 0xC, 2, 0x27F + 0x3A);
        str_param_add_arg(buf, 0xC, poke);
        b.host.handex_pop(w);

        if !b.mon_is_full_hp(poke) {
            let hp = b.mon_get(poke, 0xE).wrapping_sub(b.mon_get(poke, 0xD));
            let w = b.host.handex_push(5, poke);
            let buf = b.host.handex(w);
            buf.put_u8(6, poke as u8);
            buf.put_u16(4, hp as u16);
            b.host.handex_pop(w);
        }
        if b.mon(poke).major_status() != 0 {
            let w = b.host.handex_push(0xB, poke);
            let buf = b.host.handex(w);
            buf.put_u8(0x14, 1);
            buf.put_u8(8, poke as u8);
            buf.put_u32(4, 0x24);
            b.host.handex_pop(w);
        }
    }
    sys.remove(f);
}

/// C: `sub_068989cc`: position-effect key 3 (Future Sight / Doom Desire, event 0x76). Once the
/// turn reaches the due turn (work 0) and the team check passes, the Pokemon at position `me`
/// is hit on behalf of the user (work 6). Pushes HANDEX work 4 (message 0x438 with the position
/// and work 1) and work 0x27 (position, user, work 1 as u16), unless the position is empty, is
/// the user, or is fainted. Removes the factor whenever the due turn was reached.
pub fn h_068989cc<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    if b.var(2) != 0x1F || b.host.fun_021c83d4() {
        return;
    }
    let turn = b.host.fun_021c8138() & 0xFF;
    if (turn as i32) < sys.work(f, 0) as i32 {
        return;
    }
    let pos = b.host.poke_id_at_pos(me as u32);
    let user = sys.work(f, 6) & 0xFF;
    if pos == 0x1F || pos == user || b.mon_is_fainted(pos) {
        sys.remove(f);
        return;
    }
    let w = b.host.handex_push(4, user);
    let buf = b.host.handex(w);
    str_param_setup(buf, 4, 2, 0x438);
    str_param_add_arg(buf, 4, pos);
    str_param_add_arg(buf, 4, sys.work(f, 1));
    b.host.handex_pop(w);

    let w = b.host.handex_push(0x27, user);
    let buf = b.host.handex(w);
    buf.put_u8(4, user as u8);
    buf.put_u8(5, pos as u8);
    buf.put_u16(6, sys.work(f, 1) as u16);
    b.host.handex_pop(w);
    sys.remove(f);
}

/// C: `sub_06898b6c`: position-effect key 4 (Baton Pass, move 226, event 0x55): when the holder
/// (var 2) is the Pokemon at position `me`, pushes HANDEX work 0x2A (work 0 and the holder) and
/// removes the factor.
pub fn h_06898b6c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    let holder = var_poke(b, 2);
    if holder == b.host.poke_id_at_pos(me as u32) {
        let work0 = sys.work(f, 0);
        let w = b.host.handex_push(0x2A, 0x1F);
        let buf = b.host.handex(w);
        buf.put_u8(4, work0 as u8);
        buf.put_u8(5, holder as u8);
        b.host.handex_pop(w);
        sys.remove(f);
    }
}

/// C: `sub_06898e54(flow, side, damage class)` (overlay 95, tail-called by `06898e28` and
/// `06898e44`, not a handler of its own): when the holder (var 4) is on `side`, var 0x1A is
/// `class` and var 0x45 is 0, multiplies var 0x35 (speed) by 0x800 / 4096 in singles and triple
/// battles (rule 0 or 3), else by 0xA8F / 4096.
pub fn helper_06898e54<H: FlowHost>(b: &mut Battle<H>, side: u32, class: i32) {
    if side != poke_side(var_poke(b, 4)) as u32 {
        return;
    }
    if b.var(0x1A) != class || b.var(0x45) != 0 {
        return;
    }
    let rule = b.host.rule();
    let ratio = if rule == 0 || rule == 3 { 0x800 } else { 0xA8F };
    b.mul_var(0x35, ratio);
}

/// C: `sub_06898e28`: side-effect key 0 (Reflect, move 115, event 0x47): tail call of
/// `06898e54(flow, side, 1)`.
pub fn h_06898e28<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    helper_06898e54(b, me as u32, 1);
}

/// C: `sub_06898e44`: side-effect key 1 (Light Screen, move 113, event 0x47): tail call of
/// `06898e54(flow, side, 2)`.
pub fn h_06898e44<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    helper_06898e54(b, me as u32, 2);
}

/// C: `sub_06898f10`: side-effect key 2 (Safeguard, move 219, event 0x65): when the holder (var
/// 4) is on `side` and not var 3, and var 0x1D is below 6, is 6 or is 0xE, rewrites var 0x41 to 1
/// and stores the result in work 0.
pub fn h_06898f10<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    let holder = var_poke(b, 4);
    if me as u32 != poke_side(holder) as u32 || b.var(3) as u32 == holder {
        return;
    }
    let v = b.var(0x1D);
    if fun_021d7d6c(v) || v == 6 || v == 0xE {
        let r = b.rewrite_var(0x41, 1);
        sys.set_work(f, 0, r as u32);
    }
}

/// C: `sub_06898fb8`: side-effect key 2 (Safeguard, event 0x67): when work 0 is set, pushes
/// HANDEX work 4 (message 0x347 with the holder), clears work 0.
pub fn h_06898fb8<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, _me: u8) {
    if sys.work(f, 0) == 0 {
        return;
    }
    let holder = var_poke(b, 4);
    let w = b.host.handex_push(4, holder);
    let buf = b.host.handex(w);
    str_param_setup(buf, 4, 2, 0x347);
    str_param_add_arg(buf, 4, holder);
    b.host.handex_pop(w);
    sys.set_work(f, 0, 0);
}

/// C: `sub_0689905c`: side-effect key 3 (Mist, move 54, event 0x5B): when the holder (var 2) is
/// on `side`, var 0x20 is negative and var 3 is not the holder, rewrites var 0x41 to 1 and stores
/// the result in work 0.
pub fn h_0689905c<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    let holder = var_poke(b, 2);
    if me as u32 != poke_side(holder) as u32 || b.var(0x20) >= 0 {
        return;
    }
    if b.var(3) as u32 != holder {
        let r = b.rewrite_var(0x41, 1);
        sys.set_work(f, 0, r as u32);
    }
}

/// C: `sub_068990e8`: side-effect key 3 (Mist, event 0x5C): when work 0 is set, and var 0x19 is 0
/// or differs from work 1, pushes HANDEX work 4 (message 0x34A with the holder); stores var 0x19
/// in work 1 and clears work 0.
pub fn h_068990e8<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, _me: u8) {
    if sys.work(f, 0) == 0 {
        return;
    }
    let v = b.var(0x19);
    if v == 0 || sys.work(f, 1) as i32 != v {
        let holder = var_poke(b, 2);
        let w = b.host.handex_push(4, holder);
        let buf = b.host.handex(w);
        str_param_setup(buf, 4, 2, 0x34A);
        str_param_add_arg(buf, 4, holder);
        b.host.handex_pop(w);
        sys.set_work(f, 1, v as u32);
    }
    sys.set_work(f, 0, 0);
}

/// C: `sub_06899210`: side-effect key 5 (Lucky Chant, move 381, event 0x36): when the holder
/// (var 4) is on `side`, rewrites var 0x41 to 1.
pub fn h_06899210<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if me as u32 == poke_side(var_poke(b, 4)) as u32 {
        b.rewrite_var(0x41, 1);
    }
}

/// C: `sub_0689926c`: side-effect key 8 (Stealth Rock, move 446, event 0x55): when the holder
/// (var 2) is on `side`, pushes HANDEX work 7 with the holder's max HP divided by a divisor from
/// the Rock's effectiveness against the holder's types (2 -> 16, 3 -> 8, 4 -> 4, 5 -> 2, else
/// 32) and message 0x356.
pub fn h_0689926c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    let holder = var_poke(b, 2);
    if me as u32 != poke_side(holder) as u32 {
        return;
    }
    let pair = b.mon(holder).type_pair();
    let eff = crate::calc::type_effectiveness_vs_pair(5, pair);
    let div = match eff {
        2 => 0x10,
        3 => 8,
        4 => 4,
        5 => 2,
        _ => 0x20,
    };
    let w = b.host.handex_push(7, 0x1F);
    let hp = b.max_hp_div_min1(holder, div);
    let buf = b.host.handex(w);
    buf.put_u8(6, holder as u8);
    buf.put_u16(4, hp as u16);
    str_param_setup(buf, 0xC, 2, 0x356);
    str_param_add_arg(buf, 0xC, holder);
    b.host.handex_pop(w);
}

/// C: `sub_068993b0`: side-effect key 9 (Wide Guard, move 469, event 0x2C): when the holder
/// (var 4) is on `side` and the move in var 0x12 has parameter 0x1B in {4, 5} and is a
/// protecting move, rewrites var 0x40 to 1 and writes message 0x31D (with the holder) into the
/// message buffer.
pub fn h_068993b0<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    let holder = var_poke(b, 4);
    if me as u32 != poke_side(holder) as u32 {
        return;
    }
    let mv = b.var(0x12) as u16 as u32;
    let p = b.host.move_param(mv, 0x1B);
    if !b.host.move_fun_0201c314(mv) || p.wrapping_sub(4) >= 2 {
        return;
    }
    if b.rewrite_var(0x40, 1) {
        let buf = b.host.msg_param();
        str_param_setup(buf, 0, 2, 0x31D);
        str_param_add_arg(buf, 0, holder);
    }
}

/// C: `sub_068994b4`: side-effect key 10 (Quick Guard, move 501, event 0x2C): when the holder
/// (var 4) is on `side` and not var 3, and the move in var 0x12 has a parameter 6 that is
/// positive, rewrites var 0x40 to 1 and writes message 800 (with the holder).
pub fn h_068994b4<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    let holder = var_poke(b, 4);
    if me as u32 != poke_side(holder) as u32 || b.var(3) as u32 == holder {
        return;
    }
    let mv = b.var(0x12) as u16 as u32;
    let p = b.host.move_param(mv, 6);
    if (p as i32) >= 1 && b.rewrite_var(0x40, 1) {
        let buf = b.host.msg_param();
        str_param_setup(buf, 0, 2, 0x320);
        str_param_add_arg(buf, 0, holder);
    }
}

/// C: `sub_068995b0`: side-effect key 6 (Spikes, move 191, events 0x55, max 3 layers): when the
/// holder (var 2) is on `side` and is not a flow-query target (`021c8234`), pushes HANDEX work 7
/// with its max HP divided by 8, 6 or 4 for 1, 2 or 3 layers (else 8), message 0x353.
pub fn h_068995b0<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    let holder = var_poke(b, 2);
    if me as u32 != poke_side(holder) as u32 || b.host.fun_021c8234(holder) {
        return;
    }
    let layers = b.host.fun_06898de0(sys.sub_id(f) as u32, me as u32);
    let div = match layers {
        2 => 6,
        3 => 4,
        _ => 8,
    };
    let w = b.host.handex_push(7, 0x1F);
    let hp = b.max_hp_div_min1(holder, div);
    let buf = b.host.handex(w);
    buf.put_u16(4, hp as u16);
    buf.put_u8(6, holder as u8);
    str_param_setup(buf, 0xC, 2, 0x353);
    str_param_add_arg(buf, 0xC, holder);
    b.host.handex_pop(w);
}

/// C: `sub_068996d4`: side-effect key 7 (Toxic Spikes, move 390, events 0x55, max 2 layers):
/// when the holder (var 2) is on `side` and not a flow-query target: a Poison-type holder
/// pushes HANDEX work 0x1A (+4 = 3, +5 = 0x80, +7 = side) and absorbs the layers; otherwise
/// pushes work 0xC (+4 = 5, a permanent condition (1 layer) or badly poisoned (2+), +0xF =
/// holder).
pub fn h_068996d4<H: FlowHost>(sys: &mut EventSystem, f: FactorId, b: &mut Battle<H>, me: u8) {
    let holder = var_poke(b, 2);
    if me as u32 != poke_side(holder) as u32 || b.host.fun_021c8234(holder) {
        return;
    }
    if b.mon(holder).has_type(3) {
        let w = b.host.handex_push(0x1A, holder);
        let buf = b.host.handex(w);
        buf.put_u8(4, 3);
        buf.put_u8(5, 0);
        buf.put_u8(6, 0);
        buf.put_u8(5, 0x80);
        buf.put_u8(7, me);
        b.host.handex_pop(w);
        return;
    }
    let w = b.host.handex_push(0xC, 0x1F);
    let layers = b.host.fun_06898de0(sys.sub_id(f) as u32, me as u32);
    let sick = if layers <= 1 {
        SickCont::permanent()
    } else {
        SickCont::badly_poisoned()
    };
    let buf = b.host.handex(w);
    buf.put_u32(4, 5);
    buf.put_u32(8, sick.0);
    buf.put_u8(0xF, holder as u8);
    b.host.handex_pop(w);
}

/// C: `sub_0689983c`: side-effect key 11 (Pledge combo, event 0x51): when the holder (var 3) is
/// on `side`, rewrites var 0x26 to twice its low 15 bits.
pub fn h_0689983c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if me as u32 == poke_side(var_poke(b, 3)) as u32 {
        let x = b.var(0x26) as u32;
        b.rewrite_var(0x26, ((x & 0x7FFF) << 1) as i32);
    }
}

/// C: `sub_068998a4`: side-effect key 11 (Pledge combo, event 0x6C): when the holder (var 3) is
/// on `side`, rewrites var 0x45 to 1.
pub fn h_068998a4<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if me as u32 == poke_side(var_poke(b, 3)) as u32 {
        b.rewrite_var(0x45, 1);
    }
}

/// C: `sub_06899900`: side-effect key 12 (Pledge combo, event 0x76): when the holder (var 2) is
/// on `side`, is in battle, passes the team check and is not a Fire type: pushes HANDEX work
/// 0x37 (position, +7 = 6, message 0x28E) and work 7 (max HP / 8, message 0x484).
pub fn h_06899900<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    let holder = var_poke(b, 2);
    if holder == 0x1F
        || !b.host.fun_021c814c(holder)
        || b.host.fun_021c83d4()
        || me as u32 != poke_side(holder) as u32
        || b.mon(holder).has_type(9)
    {
        return;
    }
    let w = b.host.handex_push(0x37, holder);
    let pos = b.host.poke_pos_in_battle(holder);
    let buf = b.host.handex(w);
    buf.put_u8(6, pos as u8);
    buf.put_u8(7, 6);
    buf.put_u16(4, 0x28E);
    b.host.handex_pop(w);

    let w = b.host.handex_push(7, 0x1F);
    let hp = b.max_hp_div_min1(holder, 8);
    let buf = b.host.handex(w);
    buf.put_u8(6, holder as u8);
    buf.put_u16(4, hp as u16);
    str_param_setup(buf, 0xC, 2, 0x484);
    str_param_add_arg(buf, 0xC, holder);
    b.host.handex_pop(w);
}

/// C: `sub_06899a9c`: side-effect key 13 (Pledge combo, event 0x13): when the holder (var 2) is
/// on `side`, multiplies var 0x35 (speed) by 0x400 / 4096.
pub fn h_06899a9c<H: FlowHost>(_sys: &mut EventSystem, _f: FactorId, b: &mut Battle<H>, me: u8) {
    if me as u32 == poke_side(var_poke(b, 2)) as u32 {
        b.mul_var(0x35, 0x400);
    }
}

/// Ported overlay-95 effect handlers (regenerated by tools/sync-handlers.py).
pub const PORTED: &[u32] = &[
    0x06898094, 0x068980A4, 0x068980BC, 0x068980CC, 0x06898110, 0x06898158, 0x068981A0, 0x068981EC,
    0x06898328, 0x068984C8, 0x068987B4, 0x068989CC, 0x06898B6C, 0x06898E28, 0x06898E44, 0x06898F10,
    0x06898FB8, 0x0689905C, 0x068990E8, 0x068991B0, 0x06899210, 0x0689926C, 0x068993B0, 0x068994B4,
    0x068995B0, 0x068996D4, 0x0689983C, 0x068998A4, 0x06899900, 0x06899A9C,
];

/// Ported overlay-95 effect handlers by original address.
pub fn handler<H: FlowHost>(addr: u32) -> Option<crate::flow::Handler<H>> {
    Some(match addr & !1 {
        0x06898094 => h_06898094::<H>,
        0x068980A4 => h_068980a4::<H>,
        0x068980BC => h_068980bc::<H>,
        0x068980CC => h_068980cc::<H>,
        0x06898110 => h_06898110::<H>,
        0x06898158 => h_06898158::<H>,
        0x068981A0 => h_068981a0::<H>,
        0x068981EC => h_068981ec::<H>,
        0x06898328 => h_06898328::<H>,
        0x068984C8 => h_068984c8::<H>,
        0x068987B4 => h_068987b4::<H>,
        0x068989CC => h_068989cc::<H>,
        0x06898B6C => h_06898b6c::<H>,
        0x06898E28 => h_06898e28::<H>,
        0x06898E44 => h_06898e44::<H>,
        0x06898F10 => h_06898f10::<H>,
        0x06898FB8 => h_06898fb8::<H>,
        0x0689905C => h_0689905c::<H>,
        0x068990E8 => h_068990e8::<H>,
        0x068991B0 => h_068991b0::<H>,
        0x06899210 => h_06899210::<H>,
        0x0689926C => h_0689926c::<H>,
        0x068993B0 => h_068993b0::<H>,
        0x068994B4 => h_068994b4::<H>,
        0x068995B0 => h_068995b0::<H>,
        0x068996D4 => h_068996d4::<H>,
        0x0689983C => h_0689983c::<H>,
        0x068998A4 => h_068998a4::<H>,
        0x06899900 => h_06899900::<H>,
        0x06899A9C => h_06899a9c::<H>,
        _ => return None,
    })
}

#[cfg(test)]
mod tests;
