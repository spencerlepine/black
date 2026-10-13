//! Generic differential driver for event handlers; mirrors tools/rom-diff/handlers.py.

use crate::battle_mon::tests::{mix, random_bpp, MockEnv};
use crate::battle_mon::BppEnv;
use crate::calc::MathRandContext32;
use crate::event_factor::EventSystem;
use crate::event_var::EventVarStack;
use crate::flow::{Battle, FlowHost, HandexId};
use crate::testutil::{Fnv, Rng};

/// Every event-variable label the handlers read or write (0 and 1 are reserved).
pub const LABELS: [u16; 57] = [
    2, 3, 4, 5, 6, 12, 13, 14, 16, 17, 18, 20, 21, 22, 24, 25, 26, 27, 29, 30, 31, 32, 33, 34, 36,
    37, 38, 39, 40, 42, 43, 44, 45, 46, 48, 49, 50, 51, 53, 55, 56, 57, 58, 62, 63, 64, 65, 66, 68,
    69, 70, 71, 72, 75, 77, 81, 83,
];
pub const MONS: usize = 8;
const HANDEX_SIZE: usize = 0x80;

/// V-0007: return domain of the stubbed black-pokemon `waza` getters (`0201BD28` and its wrappers);
/// mirrors `wide` in handlers.py. 3 of 4 keys keep the small, branch-friendly answer; the 4th gets any
/// value of the real record field (`mask` bits, sign-extended when `signed`), so handler branches on
/// rarer values are reachable without hints.
fn wide(x: u32, salt: u32, small: u32, mask: u32, signed: bool) -> u32 {
    if !mix(x, salt ^ 0x10000).is_multiple_of(4) {
        return small;
    }
    let v = mix(x, salt ^ 0x20000) & mask;
    if signed && v > mask >> 1 {
        v.wrapping_sub(mask + 1)
    } else {
        v
    }
}

/// `0201BD28(mv, param)`: the domain of each parameter as `waza::record_param` reads it from the
/// record; mirrors `wide_param` in handlers.py.
fn wide_param(mv: u32, param: u32) -> u32 {
    let small = mix(mv, param ^ 0xBD28) % 3;
    let salt = param ^ 0xBD28;
    match param {
        6 | 17 | 20 | 23 => wide(mv, salt, small, 0xFF, true),
        8 | 9 => wide(mv, salt, small, 0xF, false),
        11 | 28 => wide(mv, salt, small, 0xFFFF, false),
        25 | 26 => wide(mv, salt, small, 0x7F, false),
        30 | 31 => wide(mv, salt, small, 0xFF, false) % 129,
        29 | 32.. => 0,
        _ => match wide(mv, salt, small, 0xFF, false) {
            101 if param == 4 => 100,
            6 if param == 7 => 0,
            v => v,
        },
    }
}

/// Mirrors `Host` in tools/rom-diff/handlers.py.
#[derive(Default)]
pub struct TestHost {
    pub env: MockEnv,
    pub log: Vec<u32>,
    pub bufs: Vec<Vec<u8>>,
    pub wseed: u32,
    /// The message (string parameter) buffer that event variable 0x3F points to (moves).
    pub strparam: Vec<u8>,
    /// Harness hint for item handler `021dfe28`: `021c830c` and `021c7f24` use other salts (with
    /// the usual ones `021c830c` is always the opposite of `021c83d4`, and `021c7f24` and
    /// `021c8374` cannot both pass, so that handler's tail is unreachable).
    pub alt_dfe28: bool,
    /// Z18c: address of the move handler whose hint swaps a stub salt (0 = none): `021e7634` ->
    /// `021c8234` uses 0x8236 (with 0x8234 it always agrees with `021c8374`); `021e6e38` ->
    /// `021c804c`'s result uses 0x804E (with 0x804D it is the opposite of `021c814c`).
    pub alt_hint: u32,
}

impl BppEnv for TestHost {
    fn pp_get(&mut self, pp: u32, id: u32) -> u32 {
        self.env.pp_get(pp, id)
    }
    fn pp_put(&mut self, pp: u32, id: u32, value: u32) {
        self.env.pp_put(pp, id, value)
    }
    fn pp_fast_begin(&mut self, pp: u32) -> u32 {
        self.env.pp_fast_begin(pp)
    }
    fn pp_fast_end(&mut self, pp: u32, token: u32) {
        self.env.pp_fast_end(pp, token)
    }
    fn pp_set_move(&mut self, pp: u32, mv: u32, slot: u32) {
        self.env.pp_set_move(pp, mv, slot)
    }
    fn pp_fun_02017f00(&mut self, pp: u32) -> u32 {
        self.env.pp_fun_02017f00(pp)
    }
    fn pp_status(&mut self, pp: u32) -> u32 {
        self.env.pp_status(pp)
    }
    fn move_max_pp(&mut self, mv: u32, pp_ups: u32) -> u32 {
        self.env.move_max_pp(mv, pp_ups)
    }
    fn personal(&mut self, species: u32, form: u32, param: u32) -> u32 {
        self.env.personal(species, form, param)
    }
    fn field_effect_active(&mut self, id: u32) -> bool {
        self.env.field_effect_active(id)
    }
    fn exp_for_level(&mut self, species: u32, form: u32, level: u32) -> u32 {
        self.env.exp_for_level(species, form, level)
    }
    fn pp_recalc_stats(&mut self, pp: u32) {
        self.env.pp_recalc_stats(pp)
    }
    fn pp_set_status(&mut self, pp: u32, status: u32) {
        self.env.pp_set_status(pp, status)
    }
    fn pp_set_form(&mut self, pp: u32, form: u32) {
        self.env.pp_set_form(pp, form)
    }
    fn fun_0202142c(&mut self, value: u32) -> u32 {
        self.env.fun_0202142c(value)
    }
    fn sick_is_passable(&mut self, sick: u32) -> bool {
        self.env.sick_is_passable(sick)
    }
}

impl FlowHost for TestHost {
    fn handex_push(&mut self, ty: u32, poke_id: u32) -> HandexId {
        self.log.extend([0x100, ty, poke_id]);
        self.bufs.push(vec![0; HANDEX_SIZE]);
        self.bufs.len() - 1
    }
    fn handex(&mut self, id: HandexId) -> &mut [u8] {
        &mut self.bufs[id]
    }
    fn handex_pop(&mut self, id: HandexId) {
        self.log.push(0x101);
        let words: Vec<u32> = self.bufs[id]
            .chunks(4)
            .map(|c| u32::from_le_bytes(c.try_into().unwrap()))
            .collect();
        self.log.extend(words);
    }
    fn handex_push_pop(&mut self, ty: u32, poke_id: u32) {
        self.log.extend([0x102, ty, poke_id]);
    }
    fn poke_pos(&mut self, poke_id: u32) -> u32 {
        self.log.extend([0x103, poke_id]);
        mix(poke_id, 0x7D00) % 6
    }
    fn expand_poke_ids(&mut self, code: u32, out: &mut [u8]) -> u32 {
        self.log.extend([0x104, code]);
        let n = mix(code, 0x7F68) % 7;
        for (i, o) in out.iter_mut().enumerate().take(n as usize) {
            *o = (mix(code, i as u32) % MONS as u32) as u8;
        }
        n
    }
    fn weather(&mut self) -> u32 {
        self.log.push(0x105);
        mix(self.wseed, 0x2AB0) % 5
    }
    fn move_category(&mut self, mv: u32) -> u32 {
        self.log.extend([0x106, mv]);
        wide(mv, 0xC1E8, mix(mv, 0xC1E8) % 3, 0xFF, false)
    }
    fn move_param(&mut self, mv: u32, param: u32) -> u32 {
        self.log.extend([0x109, mv, param]);
        if self.alt_hint == 0x0689_93B0 {
            4 + mix(mv, param ^ 0xBD29) % 2
        } else {
            wide_param(mv, param)
        }
    }
    fn move_flag(&mut self, mv: u32, flag: u32) -> bool {
        self.log.extend([0x10A, mv, flag]);
        mix(mv, flag ^ 0xBEB8) & 1 != 0
    }
    fn fun_021c82b8(&mut self, x: u32) -> bool {
        self.log.extend([0x10B, x]);
        mix(self.wseed, 0x82B8 ^ x) & 1 != 0
    }
    fn fun_021c83d4(&mut self) -> bool {
        self.log.push(0x10C);
        mix(self.wseed, 0x83D4) & 1 != 0
    }
    fn rule(&mut self) -> u32 {
        self.log.push(0x10E);
        mix(self.wseed, 0x815C) % 4
    }
    fn positions_adjacent(&mut self, a: u32, b: u32) -> bool {
        self.log.extend([0x10F, a, b]);
        mix(a ^ (b << 8), 0x9A6C) & 1 != 0
    }
    fn opponent_ids(&mut self, poke_id: u32, out: &mut [u8]) -> u32 {
        self.log.extend([0x110, poke_id]);
        let n = mix(poke_id, 0x7D54) % 4;
        for (i, o) in out.iter_mut().enumerate().take(n as usize) {
            *o = (mix(poke_id, 0x7D55 + i as u32) % MONS as u32) as u8;
        }
        n
    }
    fn move_power(&mut self, mv: u32) -> u32 {
        self.log.extend([0x111, mv]);
        let small = [0, 1, 1, 40, 60, 90, 120, 250][(mix(mv, 0xC1D8) % 8) as usize];
        wide(mv, 0xC1D8, small, 0xFF, false)
    }
    fn move_effect_class(&mut self, mv: u32) -> u32 {
        self.log.extend([0x112, mv]);
        wide(mv, 0xC1BC, mix(mv, 0xC1BC) % 12, 0xFF, false)
    }
    fn move_fun_0201c314(&mut self, mv: u32) -> bool {
        self.log.extend([0x113, mv]);
        mix(mv, 0xC314) & 1 != 0
    }
    fn move_type(&mut self, mv: u32) -> u32 {
        self.log.extend([0x114, mv]);
        wide(mv, 0xC1C8, mix(mv, 0xC1C8) % 18, 0xFF, false)
    }
    fn ability_breakable(&mut self, ability: u32) -> bool {
        self.log.extend([0x115, ability]);
        mix(ability, 0xC9B0) & 1 != 0
    }
    fn move_unredirectable(&mut self, mv: u32) -> bool {
        self.log.extend([0x116, mv]);
        mix(mv, 0xC8EC).is_multiple_of(4)
    }
    fn fun_021c8290(&mut self, attacker: u32, defender: u32, mv: u32) -> bool {
        self.log.extend([0x117, attacker, defender, mv]);
        mix(attacker ^ (defender << 8), mv ^ 0x8290).is_multiple_of(4)
    }
    fn find_action(&mut self, poke_id: u32, out: &mut u32) -> bool {
        self.log.extend([0x107, poke_id]);
        if self.alt_hint == 0x021D_89B8 {
            // AB_ALT_89B8 in handlers.py (V-0015): keyed on poke ^ wseed, so Slow Start's
            // counting path runs (by poke ID alone only poke 0 is found, with action kind 3).
            let k = poke_id ^ self.wseed;
            *out = mix(k, 0x89BA);
            return mix(k, 0x89B9) & 1 != 0;
        }
        *out = mix(poke_id, 0x804C);
        mix(
            poke_id,
            if self.alt_hint == 0x021E_6E38 {
                0x804E
            } else {
                0x804D
            },
        ) & 1
            != 0
    }
    fn poke_pos_in_battle(&mut self, poke_id: u32) -> u32 {
        self.log.extend([0x108, poke_id]);
        mix(poke_id, 0x8010) % 7
    }
    fn fun_021c8234(&mut self, poke_id: u32) -> bool {
        self.log.extend([0x118, poke_id]);
        mix(
            poke_id,
            if self.alt_hint == 0x021E_7634 {
                0x8236
            } else {
                0x8234
            },
        ) & 1
            != 0
    }
    fn move_fun_0689c8dc(&mut self, mv: u32) -> bool {
        self.log.extend([0x119, mv]);
        mix(mv, 0xC8DC) & 1 != 0
    }
    fn move_fun_0689c8cc(&mut self, mv: u32) -> bool {
        self.log.extend([0x11A, mv]);
        mix(mv, 0xC8CC) & 1 != 0
    }
    fn item_fun_0689c980(&mut self, item: u32) -> bool {
        self.log.extend([0x11B, item]);
        mix(item, 0xC980) & 1 != 0
    }
    fn item_fun_0689c990(&mut self, item: u32) -> bool {
        self.log.extend([0x11C, item]);
        mix(item, 0xC990) & 1 != 0
    }
    fn fun_021c80ac(&mut self, poke_id: u32) -> bool {
        self.log.extend([0x11D, poke_id]);
        mix(self.wseed ^ poke_id, 0x80AC) & 1 != 0
    }
    fn fun_021c8168(&mut self) -> bool {
        self.log.push(0x11E);
        mix(self.wseed, 0x8168) & 1 != 0
    }
    fn fun_021b8ddc(&mut self, poke_id: u32) -> u32 {
        self.log.extend([0x11F, poke_id]);
        mix(poke_id, 0x8DDC) % 3
    }
    fn fun_021c8434(&mut self, poke_id: u32, attacker: u32) {
        self.log.extend([0x120, poke_id, attacker]);
    }
    fn item_fun_020213c0(&mut self, item: u32) -> bool {
        self.log.extend([0x121, item]);
        mix(item, 0x13C0) & 1 != 0
    }
    fn move_fun_0201c020(&mut self, mv: u32) -> u32 {
        self.log.extend([0x122, mv]);
        mix(mv, 0xC020) % 4
    }
    fn move_fun_0201c0bc(&mut self, mv: u32, index: u32) -> i32 {
        self.log.extend([0x123, mv, index]);
        let small = ((mix(mv, index ^ 0xC0BC) % 5) as i32 - 1) as u32;
        wide(mv ^ (index << 16), 0xC0BC, small, 0xFF, true) as i32
    }
    fn fun_021b8c5c(&mut self, rule: u32, pos: u32) -> u32 {
        self.log.extend([0x124, rule, pos]);
        mix(rule ^ (pos << 8), 0x8C5C) % 6
    }
    fn poke_id_at_pos(&mut self, pos: u32) -> u32 {
        self.log.extend([0x125, pos]);
        mix(pos, 0x8020) % MONS as u32
    }
    fn fun_021c7ec4(&mut self) -> bool {
        self.log.push(0x126);
        mix(self.wseed, 0x7EC4) & 1 != 0
    }
    fn fun_021c7f24(&mut self, poke_id: u32) -> bool {
        self.log.extend([0x127, poke_id]);
        mix(poke_id, if self.alt_dfe28 { 0x7F26 } else { 0x7F24 }) & 1 != 0
    }
    fn fun_021c818c(&mut self, poke_id: u32) -> bool {
        self.log.extend([0x128, poke_id]);
        mix(poke_id, 0x818C) & 1 != 0
    }
    fn fun_021c830c(&mut self) -> bool {
        self.log.push(0x129);
        mix(self.wseed, if self.alt_dfe28 { 0x830D } else { 0x830C }) & 1 != 0
    }
    fn fun_021c8374(&mut self, poke_id: u32) -> bool {
        self.log.extend([0x12A, poke_id]);
        mix(poke_id, 0x8374) & 1 != 0
    }
    fn fun_021c8408(&mut self, poke_id: u32) {
        self.log.extend([0x12B, poke_id]);
    }
    fn item_param(&mut self, item: u32, param: u32) -> u32 {
        self.log.extend([0x12C, item, param]);
        mix(item, param ^ 0x805C) & 0xFF
    }
    fn sick_list_entry(&mut self, index: u32) -> u32 {
        self.log.extend([0x12D, index]);
        if index < 6 {
            mix(index, 0xC9E4) % 0x20
        } else {
            0
        }
    }
    fn fun_020189d0(&mut self, pp: u32, x: u32) -> i32 {
        self.log.extend([0x12E, pp, x]);
        if mix(pp, x ^ 0x89D0).is_multiple_of(3) {
            -1
        } else {
            (mix(pp, x ^ 0x89D1) % 5) as i32
        }
    }
    fn fun_021c7d34(&mut self, poke_id: u32) -> u32 {
        self.log.extend([0x12F, poke_id]);
        mix(poke_id, 0x7D34) % 7
    }
    fn fun_021c7d44(&mut self) -> u32 {
        self.log.push(0x130);
        if mix(self.wseed, 0x7D44).is_multiple_of(4) {
            0x1F
        } else {
            mix(self.wseed, 0x7D45) % MONS as u32
        }
    }
    fn fun_021c7ed8(&mut self, poke_id: u32) -> u32 {
        self.log.extend([0x131, poke_id]);
        mix(poke_id, 0x7ED8) % 4
    }
    fn party_ids(&mut self, poke_id: u32, out: &mut [u8]) -> u32 {
        self.log.extend([0x132, poke_id]);
        let n = mix(poke_id, 0x7FCC) % 7;
        for (i, o) in out.iter_mut().enumerate().take(n as usize) {
            *o = (mix(poke_id, 0x7FCD + i as u32) % MONS as u32) as u8;
        }
        n
    }
    fn partner_party_ids(&mut self, poke_id: u32, out: &mut [u8]) -> Option<u32> {
        self.log.extend([0x133, poke_id]);
        if mix(poke_id, 0x7FE0).is_multiple_of(3) {
            return None;
        }
        let n = mix(poke_id, 0x7FE1) % 7;
        for (i, o) in out.iter_mut().enumerate().take(n as usize) {
            *o = (mix(poke_id, 0x7FE2 + i as u32) % MONS as u32) as u8;
        }
        Some(n)
    }
    fn fun_021c8030(&mut self, poke_id: u32, x: u32) -> u32 {
        self.log.extend([0x134, poke_id, x]);
        mix(poke_id, x ^ 0x8030) % 7
    }
    fn fun_021c8114(&mut self) -> u32 {
        self.log.push(0x135);
        if mix(self.wseed, 0x8114).is_multiple_of(4) {
            0
        } else {
            mix(self.wseed, 0x8115) % 0x230
        }
    }
    fn fun_021c8138(&mut self) -> u32 {
        self.log.push(0x136);
        mix(self.wseed, 0x8138) % 16
    }
    fn fun_021c814c(&mut self, poke_id: u32) -> bool {
        self.log.extend([0x137, poke_id]);
        mix(poke_id, 0x814C) & 1 != 0
    }
    fn fun_021c8174(&mut self) -> u32 {
        self.log.push(0x138);
        mix(self.wseed, 0x8174) % 17
    }
    fn fun_021c8180(&mut self) -> u32 {
        self.log.push(0x139);
        mix(self.wseed, 0x8180) % 4
    }
    fn fun_021c81c8(&mut self, poke_id: u32, x: u32) -> u32 {
        self.log.extend([0x13A, poke_id, x]);
        1 + mix(poke_id, x ^ 0x81C8) % 0x800
    }
    fn fun_021c824c(&mut self, poke_id: u32) -> bool {
        self.log.extend([0x13B, poke_id]);
        mix(poke_id, 0x824C) & 1 != 0
    }
    fn fun_021c82c4(&mut self, side: u32, kind: u32) -> bool {
        self.log.extend([0x13C, side, kind]);
        mix(side, kind ^ 0x82C4) & 1 != 0
    }
    fn fun_021c8300(&mut self, x: u32) -> u32 {
        self.log.extend([0x13D, x]);
        mix(x, 0x8300) % 5
    }
    fn fun_021c831c(&mut self, poke_id: u32) {
        self.log.extend([0x13E, poke_id & 0xFF]);
    }
    fn fun_021c83a0(&mut self, poke_id: u32) -> u32 {
        self.log.extend([0x13F, poke_id]);
        1 + mix(poke_id, 0x83A0) % 3000
    }
    fn fun_021c83dc(&mut self, v: u32) {
        self.log.extend([0x140, v & 0xFF]);
    }
    fn fun_021c83e8(&mut self, n: u32) -> bool {
        self.log.extend([0x141, n]);
        mix(n, 0x83E8) & 1 != 0
    }
    fn fun_021c846c(&mut self, poke_id: u32, target: u32, out: &mut u32) -> bool {
        self.log.extend([0x142, poke_id, target]);
        *out = mix(poke_id, target ^ 0x846D) & 1;
        mix(poke_id, target ^ 0x846C) & 1 != 0
    }
    fn fun_021c8494(&mut self, poke_id: u32) {
        self.log.extend([0x143, poke_id]);
    }
    fn fun_021c2e4c(&mut self) -> u32 {
        self.log.push(0x144);
        mix(self.wseed, 0x2E4C)
    }
    fn fun_06898cf4(&mut self, side: u32, kind: u32) -> bool {
        self.log.extend([0x145, side, kind]);
        mix(side, kind ^ 0x8CF4) & 1 != 0
    }
    fn fun_0689c8ac(&mut self, mv: u32) -> bool {
        self.log.extend([0x146, mv]);
        mix(mv, 0xC8AC) & 1 != 0
    }
    fn fun_0689c8bc(&mut self, mv: u32) -> bool {
        self.log.extend([0x147, mv]);
        mix(mv, 0xC8BC) & 1 != 0
    }
    fn fun_0689c91c(&mut self, ability: u32) -> bool {
        self.log.extend([0x148, ability]);
        mix(ability, 0xC91C) & 1 != 0
    }
    fn fun_0689ca1c(&mut self, mv: u32) -> bool {
        self.log.extend([0x14A, mv]);
        mix(mv, 0xCA1C) & 1 != 0
    }
    fn fun_0689ca3c(&mut self, mv: u32) -> bool {
        self.log.extend([0x14B, mv]);
        mix(mv, 0xCA3C) & 1 != 0
    }
    fn fun_0689ca5c(&mut self, mv: u32) -> bool {
        self.log.extend([0x14C, mv]);
        mix(mv, 0xCA5C) & 1 != 0
    }
    fn fun_0689ca7c(&mut self, mv: u32) -> bool {
        self.log.extend([0x14D, mv]);
        mix(mv, 0xCA7C) & 1 != 0
    }
    fn fun_0689ca8c(&mut self) -> Vec<u16> {
        self.log.push(0x14E);
        (0..0x29).map(|i| (mix(i, 0xCA8C) % 0x230) as u16).collect()
    }
    fn fun_0689d064(&mut self, mv: u32, index: u32) -> bool {
        self.log.extend([0x14F, mv, index]);
        mix(mv, index ^ 0xD064) & 1 != 0
    }
    fn fun_0689d098(&mut self, mv: u32, turn: u32) -> u32 {
        self.log.extend([0x150, mv, turn]);
        mix(mv, turn ^ 0xD098) % 4
    }
    fn fun_0689d0c8(&mut self, turn: u32) -> u32 {
        self.log.extend([0x151, turn]);
        match mix(turn, 0xD0C8) % 4 {
            0 => 0,
            1 => 558,
            2 => 559,
            _ => mix(turn, 0xD0C9) % 0x230,
        }
    }
    fn fun_0689d174(&mut self, side: u32) -> u32 {
        self.log.extend([0x152, side]);
        if side < 4 {
            mix(side, 0xD174) % 7
        } else {
            0
        }
    }
    fn fun_021c7f4c(&mut self) -> bool {
        self.log.push(0x154);
        mix(self.wseed, 0x7F4C) & 1 != 0
    }
    fn fun_06898de0(&mut self, sub_id: u32, side: u32) -> u32 {
        self.log.extend([0x155, sub_id, side]);
        if side < 2 {
            mix(sub_id ^ (side << 16), 0x8DE0) % 4
        } else {
            0
        }
    }
    fn msg_param(&mut self) -> &mut [u8] {
        &mut self.strparam
    }
    fn fun_0689d184(&mut self, side: u32, index: u32) -> u32 {
        self.log.extend([0x153, side, index]);
        if side >= 4 || mix(side, index ^ 0xD184).is_multiple_of(4) {
            0x1F
        } else {
            mix(side, index ^ 0xD185) % 12
        }
    }
}

fn value_for(r: &mut Rng, me: u32) -> u32 {
    match r.next() % 8 {
        0..=2 => me,
        3 | 4 => r.next() % MONS as u32,
        5 => r.next() % 0x200,
        6 => 0x800 + r.next() % 0x1800,
        _ => r.next(),
    }
}

/// `w(section, word)`; section names match handlers.py `run`.
fn hash_battle(
    w: &mut dyn FnMut(&str, u32),
    b: &mut Battle<TestHost>,
    sys: &EventSystem,
    strparam: bool,
) {
    for chunk in b.vars.to_bytes().chunks(4) {
        let mut word = [0u8; 4];
        word[..chunk.len()].copy_from_slice(chunk);
        w("ev", u32::from_le_bytes(word));
    }
    for (i, m) in b
        .mons
        .iter()
        .chain(std::iter::once(&b.null_mon))
        .enumerate()
    {
        let sec = if i < MONS {
            format!("mon{i}")
        } else {
            "null_mon".to_string()
        };
        for c in m.raw.chunks(4) {
            w(&sec, u32::from_le_bytes(c.try_into().unwrap()));
        }
    }
    let f = sys.factor(0);
    f.work.iter().for_each(|&x| w("work", x));
    w("skip_check", f.skip_check);
    w("flags", f.flags);
    w("rng", b.calc.rng.x as u32);
    w("rng", (b.calc.rng.x >> 32) as u32);
    for v in b.host.log.drain(..) {
        w("hostlog", v);
    }
    for e in b.host.env.log.drain(..) {
        e.iter().for_each(|&v| w("envlog", v));
    }
    if strparam {
        for c in b.host.strparam.chunks(4) {
            w("strparam", u32::from_le_bytes(c.try_into().unwrap()));
        }
    }
    b.host.bufs.clear();
}

/// One handler's result, for the per-function JSON (same shape as `handlers.py --json`).
struct FuncResult {
    addr: u32,
    hash: String,
    seeds: Vec<String>,
}

fn write_json(path: &str, total: &str, funcs: &[FuncResult]) {
    let mut out = format!("{{\"hash\": \"{total}\", \"funcs\": {{");
    for (i, f) in funcs.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        let seeds: Vec<String> = f.seeds.iter().map(|s| format!("\"{s}\"")).collect();
        out.push_str(&format!(
            "\"{:08x}\": {{\"hash\": \"{}\", \"n_inputs\": {}, \"seeds\": [{}]}}",
            f.addr,
            f.hash,
            f.seeds.len(),
            seeds.join(", ")
        ));
    }
    out.push_str("}}\n");
    std::fs::write(path, out).unwrap();
}

/// Overlay 93 load address.
const OVY_BASE: u32 = 0x021B_6100;
/// Item registry: `(u32 item ID, getter)` pairs, ended by `0, 0`.
const ITEM_REGISTRY: u32 = 0x021F_1EA4;
/// Factor type of held items.
const TYPE_ITEM: u32 = 5;

/// The held-item registry read from the local ROM build; mirrors `item_registry` in
/// tools/rom-diff/handlers.py.
pub struct ItemRegistry {
    /// Item IDs in registry order.
    pub keys: Vec<u32>,
    /// Item IDs using each handler (address without the Thumb bit), in registry order.
    pub by_fn: std::collections::HashMap<u32, Vec<u32>>,
}

impl ItemRegistry {
    /// `None` when the ROM build (`BLACK_ROM_BUILD`) is missing.
    pub fn load() -> Option<Self> {
        let dir = std::env::var("BLACK_ROM_BUILD").unwrap_or_else(|_| {
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../../black-rom/build/black.us"
            )
            .to_string()
        });
        let ovy = std::fs::read(format!("{dir}/OVY_93.sbin")).ok()?;
        let r16 = |a: u32| {
            let o = (a - OVY_BASE) as usize;
            u16::from_le_bytes([ovy[o], ovy[o + 1]]) as u32
        };
        let r32 = |a: u32| r16(a) | r16(a + 2) << 16;
        let mut keys = Vec::new();
        let mut by_fn: std::collections::HashMap<u32, Vec<u32>> = Default::default();
        let mut i = 0;
        loop {
            let (k, g) = (r32(ITEM_REGISTRY + 8 * i), r32(ITEM_REGISTRY + 8 * i + 4));
            if k == 0 && g == 0 {
                return Some(ItemRegistry { keys, by_fn });
            }
            i += 1;
            // `movs r1, #n; str r1, [r0]; ldr r0, [pc, #k]; bx lr`, then the table address.
            let g = g & !1;
            let (mov, str, ldr, bx) = (r16(g), r16(g + 2), r16(g + 4), r16(g + 6));
            assert!(mov >> 8 == 0x21 && str == 0x6001 && ldr >> 8 == 0x48 && bx == 0x4770);
            let table = r32(((g + 8) & !3) + (ldr & 0xFF) * 4);
            keys.push(k);
            for j in 0..(mov & 0xFF) {
                let f = r32(table + 8 * j + 4) & !1;
                let items = by_fn.entry(f).or_default();
                if !items.contains(&k) {
                    items.push(k);
                }
            }
        }
    }
}

/// Move registry: 258 `(u16 move ID, pad, getter)` entries, stride 8.
const MOVE_REGISTRY: u32 = 0x021F_3030;
const MOVE_COUNT: u32 = 258;
/// Factor type of moves.
const TYPE_MOVE: u32 = 0;
/// Value of event variable 0x3F in the move suite (an address in the emulator).
const STRP: u32 = 0x02EB_0000;
const STRP_SIZE: usize = 0x40;

/// The move registry read from the local ROM build; mirrors `move_registry` in
/// tools/rom-diff/handlers.py (the getters of moves 87, 100, 217 and 542 have the same shape
/// as the others).
pub struct MoveRegistry {
    /// Move IDs in registry order.
    pub keys: Vec<u32>,
    /// Move IDs using each handler (address without the Thumb bit), in registry order.
    pub by_fn: std::collections::HashMap<u32, Vec<u32>>,
}

impl MoveRegistry {
    /// `None` when the ROM build (`BLACK_ROM_BUILD`) is missing.
    pub fn load() -> Option<Self> {
        let dir = std::env::var("BLACK_ROM_BUILD").unwrap_or_else(|_| {
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../../black-rom/build/black.us"
            )
            .to_string()
        });
        let ovy = std::fs::read(format!("{dir}/OVY_93.sbin")).ok()?;
        let r16 = |a: u32| {
            let o = (a - OVY_BASE) as usize;
            u16::from_le_bytes([ovy[o], ovy[o + 1]]) as u32
        };
        let r32 = |a: u32| r16(a) | r16(a + 2) << 16;
        let mut keys = Vec::new();
        let mut by_fn: std::collections::HashMap<u32, Vec<u32>> = Default::default();
        for i in 0..MOVE_COUNT {
            let (k, g) = (r16(MOVE_REGISTRY + 8 * i), r32(MOVE_REGISTRY + 8 * i + 4));
            // `movs r1, #n; str r1, [r0]; ldr r0, [pc, #k]; bx lr`, then the table address.
            let g = g & !1;
            let (mov, str, ldr, bx) = (r16(g), r16(g + 2), r16(g + 4), r16(g + 6));
            assert!(mov >> 8 == 0x21 && str == 0x6001 && ldr >> 8 == 0x48 && bx == 0x4770);
            let table = r32(((g + 8) & !3) + (ldr & 0xFF) * 4);
            keys.push(k);
            for j in 0..(mov & 0xFF) {
                let f = r32(table + 8 * j + 4) & !1;
                let moves = by_fn.entry(f).or_default();
                if !moves.contains(&k) {
                    moves.push(k);
                }
            }
        }
        Some(MoveRegistry { keys, by_fn })
    }
}

/// Z18c: the "null mon" (address 0 in the ROM harness) as a hint mon index.
const NULLMON: u32 = u32::MAX;
/// Z18c: a poke ID on the far side (`id >= 12`) whose mon is the null mon; see `FAR` in handlers.py.
const FAR: i32 = 12;
const K: u32 = 0x55;
/// `CAP_ALL` in handlers.py.
const CAP_ALL: [u32; 4] = [0x021E_732C, 0x021E_7388, 0x021E_7420, 0x021E_2700];

/// `_val` in handlers.py: the smallest x >= 1 with `mix(x, salt) & 1 == want`.
fn hint_val(salt: u32, want: u32) -> u32 {
    (1..100).find(|&x| mix(x, salt) & 1 == want).unwrap()
}

/// `_pick` in handlers.py.
fn hint_pick(me: u32, conds: &[(u32, u32)]) -> u32 {
    let m = MONS as u32;
    (1..=m)
        .map(|i| (me + i) % m)
        .find(|&p| conds.iter().all(|&(salt, want)| mix(p, salt) & 1 == want))
        .unwrap_or((me + 1) % m)
}

/// `_knows` in handlers.py: one move `mv` in slot 0 with `pp` PP.
fn hint_knows(mon: u32, mv: u32, pp: u32) -> Vec<HintMon> {
    vec![(mon, 0x140, 1, 1), (mon, 0x10A, 2, mv), (mon, 0x10C, 1, pp)]
}

/// `_partner` in handlers.py.
fn hint_partner(mv: u32, slot: u32) -> u32 {
    if slot == 3 && (mv == 0x206 || mv == 0x208) {
        return 0x206 + 0x208 - mv;
    }
    if mv == 0x207 {
        0x206
    } else {
        0x207
    }
}

/// `_targets` in handlers.py.
fn hint_targets(me: u32) -> Vec<(u16, i32)> {
    let m = MONS as u32;
    (1..=3)
        .map(|i| (5 + i as u16, ((me + i) % m) as i32))
        .collect()
}

/// (mon, offset, width, value)
type HintMon = (u32, usize, usize, u32);
/// (event variables, mon bytes, work words)
type MoveHint = (Vec<(u16, i32)>, Vec<HintMon>, Vec<(usize, u32)>);

/// Z18/Z18c harness hints for move handlers whose tails are gated on state the random inputs
/// almost never produce; mirrors `MOVE_HINTS` / `apply_move_hint` in tools/rom-diff/handlers.py.
/// Applied to 3 of 4 seeds (`seed % 4 != 0`) after every draw (no extra draws). `me` = the factor
/// owner, `mv` = its move ID, `slot` = `seed % 4`. Returns the event variables forced (label,
/// value; kind 0), the mon bytes written (mon, offset, width, value) and the work words set.
fn move_hint(addr: u32, me: u32, mv: u32, slot: u32) -> Option<MoveHint> {
    let m = MONS as u32;
    let me_i = me as i32;
    let t = (me + 1) % m;
    let t_i = t as i32;
    let sl = slot as usize;
    let b = |c: bool| c as u32;
    Some(match addr {
        0x021E_0998 | 0x021E_0A80 => (
            vec![(3, me_i), (6, t_i)],
            vec![(me, 0x10A + 14 * sl, 2, mv)],
            vec![],
        ),
        0x021E_0E28 => (vec![(3, me_i)], vec![(me, 0xF8, 1, 7)], vec![]),
        0x021E_11DC => (vec![(3, me_i), (0x47, 0)], vec![], vec![]),
        0x021E_12A4 => (vec![(3, me_i)], vec![(me, 0x0C, 2, 0x1B9)], vec![]),
        0x021E_1354 => (vec![(3, me_i), (0x1D, 8)], vec![], vec![]),
        0x021E_1668 => (vec![(3, me_i)], vec![(me, 0x157, 1, slot - 1)], vec![]),
        0x021E_36F4 => (
            vec![(3, me_i), (6, t_i)],
            vec![(me, 0x12, 2, 0), (t, 0x12, 2, 0x10)],
            vec![(0, 0)],
        ),
        0x021E_6728 => (
            vec![(4, me_i), (3, FAR), (0x12, hint_val(0xC8CC, 0) as i32)],
            vec![(me, 0x10, 2, 0), (NULLMON, 0x10, 2, 1)],
            vec![],
        ),
        0x021E_5B78 => (vec![(3, me_i)], vec![], vec![]),
        0x021E_20E0 => (vec![(4, me_i), (0x46, 0)], vec![(me, 0x10, 2, 1)], vec![]),
        0x021E_6840 => {
            let mut ms = vec![(me, 0x10, 2, 0)];
            ms.extend(hint_knows(t, K + b(slot == 3), 5));
            (
                vec![
                    (4, me_i),
                    (3, t_i),
                    (0x12, hint_val(0xC8CC, 0) as i32),
                    (0x14, K as i32),
                ],
                ms,
                vec![],
            )
        }
        0x021E_8190 => (vec![(3, me_i)], vec![(me, 0x0C, 2, 0x288)], vec![]),
        0x021E_3F44 => (vec![(3, me_i)], vec![(me, 0x12, 2, 0)], vec![]),
        0x021E_4B00 => {
            let mut ms = vec![(t, 0x14A, 2, K)];
            ms.extend(hint_knows(t, K, 2 * slot));
            (vec![(3, me_i), (6, t_i)], ms, vec![])
        }
        0x021E_1EB8 => (vec![(2, me_i), (0x12, mv as i32)], vec![], vec![]),
        0x021E_7634 => {
            let p = hint_pick(me, &[(0x8374, 0), (0x8236, 1)]);
            let ms = vec![
                (p, 0x1C + 4 * 0x21, 4, 0),
                (p, 0x1C + 4 * 0x1E, 4, b(slot == 2)),
                (p, 0x1C + 4 * 0x20, 4, b(slot == 3)),
            ];
            (vec![(3, me_i), (6, p as i32)], ms, vec![])
        }
        0x021E_25FC => (
            vec![(2, me_i)],
            vec![(me, 0x1C + 4 * 0x19, 4, 0), (me, 0x1C + 4 * 2, 4, 0)],
            vec![(6, 0)],
        ),
        0x021E_4408 => (vec![(2, me_i), (0x22, 0)], vec![], vec![]),
        0x021E_2E28 => (
            vec![(3, me_i)],
            vec![(me, 0x14C, 2, mv), (me, 0x14E, 2, slot)],
            vec![],
        ),
        0x021E_4688 | 0x021E_4828 => {
            let sick = if addr == 0x021E_4688 { 0x17 } else { 0xD };
            let mut ms = vec![
                (t, 0x14A, 2, K),
                (t, 0x1C + 4 * sick, 4, 0),
                (t, 0x153, 1, 2 * b(slot == 2)),
            ];
            ms.extend(hint_knows(t, K, 5));
            (vec![(3, me_i), (6, t_i)], ms, vec![])
        }
        0x021E_83CC => (
            vec![(3, me_i)],
            vec![(me, 0x1F6, 1, slot), (me, 0x1F4, 2, hint_partner(mv, slot))],
            vec![],
        ),
        0x021E_7924 => (vec![(3, me_i), (0x3C, 0xB)], vec![], vec![]),
        0x021E_5DAC => (
            vec![(3, me_i), (0x46, 0), (0x47, 0)],
            vec![(me, 0x12, 2, 0x10 * slot)],
            vec![],
        ),
        0x021E_2D5C => (
            vec![(3, me_i)],
            hint_knows(me, mv + b(slot == 3), 2 * slot - 1),
            vec![],
        ),
        0x021E_6E38 => (
            vec![(2, me_i), (0xE, if slot == 3 { 6 } else { 1 })],
            vec![(7, 0x153, 1, 0)],
            vec![],
        ),
        0x021E_732C | 0x021E_7388 | 0x021E_7420 => {
            let mut vs = vec![(3, me_i), (5, slot as i32)];
            vs.extend(hint_targets(me));
            (vs, vec![], vec![])
        }
        0x021E_2700 => (
            vec![(3, me_i)],
            vec![],
            vec![(0, slot - 1), (1, 2 + 2 * b(slot == 2) + b(slot == 3))],
        ),
        _ => return None,
    })
}

/// Overlay 95 load address (`OVY95_BASE` in handlers.py).
const OVY95_BASE: u32 = 0x0689_8020;
/// The three overlay-95 registries: (address, entries, stride, factor type) for the kinds
/// field (0), position (1) and side (2) effects (`FX_REGISTRIES` in handlers.py).
const FX_REGISTRIES: [(u32, u32, u32, u32); 3] = [
    (0x0689_D650, 8, 8, 3),
    (0x0689_D6C8, 5, 8, 1),
    (0x0689_D780, 14, 12, 2),
];

/// The overlay-95 effect registries read from the local ROM build; mirrors `fx_registry` in
/// tools/rom-diff/handlers.py.
pub struct EffectRegistry {
    /// Keys of each kind (field, position, side) in registry order.
    pub keys: [Vec<u32>; 3],
    /// For each handler (address without the Thumb bit): its kind and the keys using it, in
    /// registry order.
    pub by_fn: std::collections::HashMap<u32, (usize, Vec<u32>)>,
}

impl EffectRegistry {
    /// `None` when the ROM build (`BLACK_ROM_BUILD`) is missing.
    pub fn load() -> Option<Self> {
        let dir = std::env::var("BLACK_ROM_BUILD").unwrap_or_else(|_| {
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../../black-rom/build/black.us"
            )
            .to_string()
        });
        let ovy = std::fs::read(format!("{dir}/OVY_95.sbin")).ok()?;
        let r16 = |a: u32| {
            let o = (a - OVY95_BASE) as usize;
            u16::from_le_bytes([ovy[o], ovy[o + 1]]) as u32
        };
        let r32 = |a: u32| r16(a) | r16(a + 2) << 16;
        let mut keys: [Vec<u32>; 3] = Default::default();
        let mut by_fn: std::collections::HashMap<u32, (usize, Vec<u32>)> = Default::default();
        for (kind, &(reg, n, stride, _)) in FX_REGISTRIES.iter().enumerate() {
            for i in 0..n {
                let (k, g) = (r32(reg + stride * i), r32(reg + stride * i + 4));
                // `movs r1, #n; str r1, [r0]; ldr r0, [pc, #k]; bx lr`, then the table address.
                let g = g & !1;
                let (mov, str, ldr, bx) = (r16(g), r16(g + 2), r16(g + 4), r16(g + 6));
                assert!(mov >> 8 == 0x21 && str == 0x6001 && ldr >> 8 == 0x48 && bx == 0x4770);
                let table = r32(((g + 8) & !3) + (ldr & 0xFF) * 4);
                keys[kind].push(k);
                for j in 0..(mov & 0xFF) {
                    let f = r32(table + 8 * j + 4) & !1;
                    let ent = by_fn.entry(f).or_insert((kind, Vec::new()));
                    assert_eq!(ent.0, kind);
                    if !ent.1.contains(&k) {
                        ent.1.push(k);
                    }
                }
            }
        }
        Some(EffectRegistry { keys, by_fn })
    }
}

/// Z18b hints for ability and item handlers; mirrors `apply_hint` in tools/rom-diff/handlers.py
/// (table generated from `HINTS` by tools/rom-diff/genhints.py). Applied to 3 of 4 seeds after
/// every draw (no extra draws), and only to the handlers listed in `HINT_TABLE`.
fn apply_hint(
    addr: u32,
    slot: u32,
    me: u32,
    b: &mut Battle<TestHost>,
    sys: &mut EventSystem,
) -> Option<()> {
    use super::hints::{HINT_TABLE, ME, NXT, REL};
    let m = MONS as u32;
    let val = |v: &[u32]| -> u32 {
        let v = v[(slot as usize - 1) % v.len()];
        if (REL..REL + m).contains(&v) {
            (me + v - REL) % m
        } else if v == ME {
            me
        } else if v == NXT {
            (me + 1) % m
        } else {
            v
        }
    };
    let h = HINT_TABLE.iter().find(|h| h.addr == addr)?;
    for &(lab, v) in h.vars {
        let s = LABELS.iter().position(|&l| l == lab).unwrap() + 1;
        b.vars.value[s] = val(v) as i32;
        b.vars.kind[s] = 0;
    }
    for &(who, off, width, v) in h.mons {
        let targets: Vec<u32> = match who {
            0 => vec![me],
            1 => vec![(me + 1) % m],
            _ => (0..m).collect(),
        };
        for mon in targets {
            b.mons[mon as usize].raw[off..off + width]
                .copy_from_slice(&val(v).to_le_bytes()[..width]);
        }
    }
    for &(k, v) in h.work {
        sys.factor_mut(0).work[k] = val(v);
    }
    if !h.wseed.is_empty() {
        b.host.wseed = val(h.wseed);
    }
    Some(())
}

/// Runs every handler in `addrs` over random states; returns the combined hash.
pub fn run_handler_suite(
    addrs: &[u32],
    get: impl Fn(u32) -> crate::flow::Handler<TestHost>,
) -> String {
    run_suite(addrs, get, None, None, None)
}

/// Like [`run_handler_suite`] for held-item handlers: the factor is a type-5 factor whose sub
/// ID is an item ID (`handlers.py items`).
pub fn run_item_suite(
    addrs: &[u32],
    get: impl Fn(u32) -> crate::flow::Handler<TestHost>,
    items: &ItemRegistry,
) -> String {
    run_suite(addrs, get, Some(items), None, None)
}

/// Like [`run_handler_suite`] for move handlers: the factor is a type-0 factor whose sub ID is
/// a move ID (`handlers.py moves`).
pub fn run_move_suite(
    addrs: &[u32],
    get: impl Fn(u32) -> crate::flow::Handler<TestHost>,
    moves: &MoveRegistry,
) -> String {
    run_suite(addrs, get, None, Some(moves), None)
}

/// Like [`run_handler_suite`] for the overlay-95 effect handlers: the factor is of the
/// handler's registry kind (type 3 field, 1 position, 2 side effect) and its sub ID a key
/// (`handlers.py effects`).
pub fn run_effect_suite(
    addrs: &[u32],
    get: impl Fn(u32) -> crate::flow::Handler<TestHost>,
    effects: &EffectRegistry,
) -> String {
    run_suite(addrs, get, None, None, Some(effects))
}

/// Shared driver of the two suites above.
///
/// Harness env vars (used by tools/check-handlers.sh):
/// - `HANDLER_JSON=<path>`: also write the per-function and per-input hashes there.
/// - `HANDLER_DETAIL=<hex addr>:<seed>` + `HANDLER_DETAIL_OUT=<path>`: run only that handler and
///   write that input's section-labelled word stream as JSON (`[["ev", 1], ...]`).
fn run_suite(
    addrs: &[u32],
    get: impl Fn(u32) -> crate::flow::Handler<TestHost>,
    items: Option<&ItemRegistry>,
    moves: Option<&MoveRegistry>,
    effects: Option<&EffectRegistry>,
) -> String {
    let mut h = Fnv::new();
    let verbose = std::env::var_os("HANDLER_VERBOSE").is_some();
    // HANDLER_SEEDS=<hex address>: print one hash per seed for that handler.
    let seed_dump = std::env::var("HANDLER_SEEDS")
        .ok()
        .and_then(|v| u32::from_str_radix(v.trim_start_matches("0x"), 16).ok());
    let json_out = std::env::var("HANDLER_JSON").ok();
    let detail: Option<(u32, u32)> = std::env::var("HANDLER_DETAIL").ok().and_then(|v| {
        let (a, s) = v.split_once(':')?;
        Some((
            u32::from_str_radix(a.trim_start_matches("0x"), 16).ok()?,
            s.parse().ok()?,
        ))
    });
    let mut stream: Vec<(String, u32)> = Vec::new();
    let mut funcs: Vec<FuncResult> = Vec::new();
    for &addr in addrs {
        if detail.is_some_and(|(a, _)| a != addr) {
            continue;
        }
        let f = get(addr);
        let mut ha = Fnv::new();
        let mut seed_hashes: Vec<String> = Vec::new();
        for seed in 0..200u32 {
            let mut r = Rng::new(addr ^ seed.wrapping_mul(0x9E37));
            let mut b = Battle::new(TestHost::default());
            b.mons = (0..MONS).map(|_| random_bpp(&mut r)).collect();
            b.host.env.field = r.next() & 1 != 0;
            b.host.wseed = r.next();
            let me = r.next() % MONS as u32;
            let mut vars = EventVarStack::new();
            vars.sp = 1;
            vars.label[0] = 1;
            for (i, &lab) in LABELS.iter().enumerate() {
                let s = i + 1;
                vars.label[s] = lab;
                let v = value_for(&mut r, me);
                // Label 5 is a target count used as a loop bound.
                vars.value[s] = if lab == 5 { v % 8 } else { v } as i32;
                vars.kind[s] = [0, 1, 2, 3, 3, 3, 4, 0][(r.next() % 8) as usize];
                if vars.kind[s] == 3 {
                    vars.value[s] = (0x100 + r.next() % 0x10000) as i32;
                    if r.next() & 1 != 0 {
                        vars.min[s] = (r.next() % 0x40) as i32;
                        vars.max[s] = (r.next() % 0x10000) as i32;
                    }
                }
            }
            b.host.alt_dfe28 = items.is_some() && addr == 0x021D_FE28;
            if items.is_some() && addr == 0x021D_FE28 && seed % 4 != 0 {
                // Harness hint (no extra draws), same as handlers.py: var 0x4D == 0 and the
                // holder is the only event target (var 5 = count, var 6 = ID).
                for (lab, v) in [(0x4D, 0), (5, 1), (6, me as i32)] {
                    let slot = LABELS.iter().position(|&l| l == lab).unwrap() + 1;
                    vars.value[slot] = v;
                    vars.kind[slot] = 0;
                }
            }
            if moves.is_some() || effects.is_some() {
                // Variable 0x3F is a pointer to the message buffer (`STRP` in handlers.py).
                let slot = LABELS.iter().position(|&l| l == 0x3F).unwrap() + 1;
                vars.value[slot] = STRP as i32;
                b.host.strparam = vec![0; STRP_SIZE];
            }
            b.vars = vars;
            let hi = r.next() as u64;
            let lo = r.next() as u64;
            b.calc.rng = MathRandContext32 {
                x: hi << 32 | lo,
                mul: 0x5D58_8B65_6C07_8965,
                add: 0x0026_9EC3,
            };
            let mut sys = EventSystem::new();
            let fac = sys.factor_mut(0);
            for w in fac.work.iter_mut() {
                *w = r.next();
            }
            if let Some(reg) = items {
                // Work words hold "already used" flags and counters, so make 0 and small values
                // common instead of 32-bit noise (which would skip those branches).
                for w in fac.work.iter_mut() {
                    match r.next() % 4 {
                        0 | 1 => *w = 0,
                        2 => *w = r.next() % 4,
                        _ => {}
                    }
                }
                // Mostly one of the items using this handler, else any registry item.
                let own = reg.by_fn.get(&addr).map_or(&[][..], |v| &v[..]);
                let (pick, which) = (r.next(), r.next() as usize);
                let item = if !own.is_empty() && pick % 4 != 0 {
                    own[which % own.len()]
                } else {
                    reg.keys[which % reg.keys.len()]
                };
                fac.ftype = TYPE_ITEM;
                fac.sub_id = item as u16;
                fac.owner = me as u8;
                fac.poke_id = me as u8;
            }
            if let Some(reg) = moves {
                // Same work-word distribution as items; mostly one of the moves using this
                // handler, else any registry move.
                for w in fac.work.iter_mut() {
                    match r.next() % 4 {
                        0 | 1 => *w = 0,
                        2 => *w = r.next() % 4,
                        _ => {}
                    }
                }
                let own = reg.by_fn.get(&addr).map_or(&[][..], |v| &v[..]);
                let (pick, which) = (r.next(), r.next() as usize);
                let mv = if !own.is_empty() && pick % 4 != 0 {
                    own[which % own.len()]
                } else {
                    reg.keys[which % reg.keys.len()]
                };
                fac.ftype = TYPE_MOVE;
                fac.sub_id = mv as u16;
                fac.owner = me as u8;
                fac.poke_id = me as u8;
                b.host.alt_hint = if seed % 4 != 0 && (addr == 0x021E_7634 || addr == 0x021E_6E38) {
                    addr
                } else {
                    0
                };
                if seed % 4 == 0 && CAP_ALL.contains(&addr) {
                    // CAP_ALL in handlers.py: plain seeds get the drawn bound reduced.
                    if addr == 0x021E_2700 {
                        fac.work[0] %= 8;
                    } else {
                        let slot = LABELS.iter().position(|&l| l == 5).unwrap() + 1;
                        b.vars.value[slot] = ((b.vars.value[slot] as u32) % 4) as i32;
                    }
                }
                if seed % 4 != 0 {
                    if let Some((vs, ms, ws)) = move_hint(addr, me, mv, seed % 4) {
                        // Labels absent from LABELS take the next free slots, in order of
                        // appearance (same as apply_move_hint in handlers.py).
                        let mut extra: Vec<u16> = Vec::new();
                        for (lab, v) in vs {
                            let slot = match LABELS.iter().position(|&l| l == lab) {
                                Some(i) => i + 1,
                                None => {
                                    if !extra.contains(&lab) {
                                        extra.push(lab);
                                    }
                                    let s = LABELS.len()
                                        + 1
                                        + extra.iter().position(|&l| l == lab).unwrap();
                                    b.vars.label[s] = lab;
                                    s
                                }
                            };
                            b.vars.value[slot] = v;
                            b.vars.kind[slot] = 0;
                        }
                        for (mon, off, width, v) in ms {
                            let raw = if mon == NULLMON {
                                &mut b.null_mon.raw
                            } else {
                                &mut b.mons[mon as usize].raw
                            };
                            raw[off..off + width].copy_from_slice(&v.to_le_bytes()[..width]);
                        }
                        for (k, v) in ws {
                            fac.work[k] = v;
                        }
                    }
                }
            }
            let mut owner = me;
            if let Some(reg) = effects {
                // Same work-word distribution as moves. The owner (`factor + 0x3A`, passed to
                // the handler) is 0 for field effects, a position (0..5) for position effects
                // and a side (0/1) for side effects; `+0x3B` is 0x1F for all three.
                let fac = sys.factor_mut(0);
                for w in fac.work.iter_mut() {
                    match r.next() % 4 {
                        0 | 1 => *w = 0,
                        2 => *w = r.next() % 4,
                        _ => {}
                    }
                }
                let (kind, own) = &reg.by_fn[&addr];
                let (pick, which) = (r.next(), r.next() as usize);
                let key = if pick % 4 != 0 {
                    own[which % own.len()]
                } else {
                    reg.keys[*kind][which % reg.keys[*kind].len()]
                };
                owner = [0, me % 6, me & 1][*kind];
                fac.ftype = FX_REGISTRIES[*kind].3;
                fac.sub_id = key as u16;
                fac.owner = owner as u8;
                fac.poke_id = 0x1F;
                // FX_ALT_BD28 in handlers.py: this handler's hint makes `move_param` answer 4 / 5
                b.host.alt_hint = if seed % 4 != 0 && addr == 0x0689_93B0 {
                    addr
                } else {
                    0
                };
            }
            if items.is_none() && moves.is_none() && effects.is_none() {
                b.host.alt_hint = if seed % 4 != 0 && addr == 0x021D_89B8 {
                    addr
                } else {
                    0
                };
            }
            if moves.is_none() && seed % 4 != 0 {
                apply_hint(addr, seed % 4, me, &mut b, &mut sys);
            }
            f(&mut sys, 0, &mut b, owner as u8);
            let mut hs = Fnv::new();
            if seed_dump == Some(addr) && seed < 2 {
                println!("log {:x?}", b.host.log);
            }
            let want = detail == Some((addr, seed));
            hash_battle(
                &mut |sec, v| {
                    h.w(v);
                    ha.w(v);
                    hs.w(v);
                    if want {
                        stream.push((sec.to_string(), v));
                    }
                },
                &mut b,
                &sys,
                moves.is_some() || effects.is_some(),
            );
            seed_hashes.push(hs.hex());
            if seed_dump == Some(addr) {
                println!("seed {} {}", seed, hs.hex());
            }
        }
        if verbose {
            println!("{:#x} {}", addr, ha.hex());
        }
        funcs.push(FuncResult {
            addr,
            hash: ha.hex(),
            seeds: seed_hashes,
        });
    }
    if let Some(p) = json_out {
        write_json(&p, &h.hex(), &funcs);
    }
    if let (Some(_), Ok(p)) = (detail, std::env::var("HANDLER_DETAIL_OUT")) {
        let items: Vec<String> = stream
            .iter()
            .map(|(s, v)| format!("[\"{s}\", {v}]"))
            .collect();
        std::fs::write(p, format!("[{}]\n", items.join(", "))).unwrap();
    }
    h.hex()
}
