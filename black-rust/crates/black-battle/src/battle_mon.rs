//! Battle Pokemon (`BTL_POKEPARAM`): asm/battle_pokeparam.s (`btl_pokeparam.c`, overlay 93).
//!
//! [`BattleMon`] keeps the original 0x1F8-byte little-endian layout (so tests can compare it
//! byte for byte with the original code) and exposes named accessors. Known fields:
//!
//! | offset | field |
//! |---|---|
//! | +0x000 | `u32` party Pokemon handle (`POKEMON_PARAM*`) |
//! | +0x004 | `u32` Illusion disguise handle |
//! | +0x008 | `u32` EXP |
//! | +0x00C | `u16` species |
//! | +0x00E / +0x010 | `u16` max HP / HP |
//! | +0x012 | `u16` held item |
//! | +0x016 | `u16` ability as loaded (PP param 0x0A) |
//! | +0x018 / +0x019 | `u8` level / battle Pokemon ID |
//! | +0x01A | `u8` personal param 1 |
//! | +0x01B | `u8` bits 0-4 form, bit 5 "reload from party", bit 6 Illusion active, bit 7 |
//! | +0x01C | `u32 sick_cont[36]` ([`SickCont`] per condition) |
//! | +0x0AC | `u8 sick_turns[36]` |
//! | +0x0EE..+0x0F6 | `u16` Atk, Def, SpA, SpD, Spe |
//! | +0x0F8 / +0x0F9 / +0x0FA | `u8` type 1 / type 2 / PP `FUN_02017F00` value |
//! | +0x0FC..+0x102 | `s8` stages: Atk, Def, SpA, SpD, Spe, Acc, Eva (6 = neutral, 0..=12) |
//! | +0x104 | 4 x 14-byte move slots (see [`MoveSlot`]) |
//! | +0x13C | `u16` ability |
//! | +0x13E | `u16` personal param 0x26 (at least 1) |
//! | +0x140 | `u8` move count |
//! | +0x141 | `u8` form |
//! | +0x142 | `u8` critical-hit stage boost (0-4) |
//! | +0x143 | `u8` number of distinct moves used |
//! | +0x144 | `u8` (0x11 on reset) |
//! | +0x153 / +0x155 | `u8[2]` flag bitsets |
//! | +0x157 | `u8[5]` |
//!
//! A move slot holds the party's original move (+0) and the in-battle copy (+6) that
//! Transform/Mimic can override; while `linked` (+0xC) is set, PP changes are mirrored.

use crate::sick_cont::SickCont;

/// Size of `BTL_POKEPARAM`.
pub const BPP_SIZE: usize = 0x1F8;
/// Number of condition ("sick") slots.
pub const SICK_COUNT: usize = 36;
/// Neutral stat stage.
pub const STAGE_NEUTRAL: i8 = 6;
/// Highest stat stage.
pub const STAGE_MAX: i8 = 12;

const OFS_SICK_CONT: usize = 0x1C;
const OFS_SICK_TURNS: usize = 0xAC;
const OFS_STATS: usize = 0xEE;
const OFS_STAGES: usize = 0xFC;
const OFS_MOVES: usize = 0x104;
const MOVE_SLOT_SIZE: usize = 0xE;
const OFS_MOVE_COUNT: usize = 0x140;
const OFS_FORM: usize = 0x141;
const OFS_CRIT_BOOST: usize = 0x142;
const OFS_MOVES_USED: usize = 0x143;
const OFS_FLAGS_A: usize = 0x153;
const OFS_FLAGS_B: usize = 0x155;

/// `PokemonParam` IDs passed to `FUN_02017E1C` / `FUN_02017E40`.
pub mod pp {
    pub const SPECIES: u32 = 0x05;
    pub const HELD_ITEM: u32 = 0x06;
    pub const EXP: u32 = 0x08;
    pub const ABILITY: u32 = 0x0A;
    pub const MOVE1: u32 = 0x36;
    pub const MOVE1_PP: u32 = 0x3A;
    pub const MOVE1_PP_UPS: u32 = 0x3E;
    pub const MOVE1_MAX_PP: u32 = 0x42;
    pub const IS_EGG: u32 = 0x4C;
    pub const FORM: u32 = 0x6F;
    pub const LEVEL: u32 = 0x9E;
    pub const HP: u32 = 0xA0;
    pub const MAX_HP: u32 = 0xA1;
    pub const ATTACK: u32 = 0xA2;
    pub const DEFENSE: u32 = 0xA3;
    pub const SPEED: u32 = 0xA4;
    pub const SP_ATTACK: u32 = 0xA5;
    pub const SP_DEFENSE: u32 = 0xA6;
    pub const TYPE1: u32 = 0xAE;
    pub const TYPE2: u32 = 0xAF;
}

/// `BattleMon_Get` value IDs (verified against its jump table).
pub mod value {
    pub const NONE: u32 = 0;
    pub const ATTACK_STAGE: u32 = 1;
    pub const DEFENSE_STAGE: u32 = 2;
    pub const SP_ATTACK_STAGE: u32 = 3;
    pub const SP_DEFENSE_STAGE: u32 = 4;
    pub const SPEED_STAGE: u32 = 5;
    pub const ACCURACY_STAGE: u32 = 6;
    pub const EVASION_STAGE: u32 = 7;
    pub const ATTACK: u32 = 8;
    pub const DEFENSE: u32 = 9;
    pub const SP_ATTACK: u32 = 10;
    pub const SP_DEFENSE: u32 = 11;
    pub const SPEED: u32 = 12;
    pub const HP: u32 = 13;
    pub const MAX_HP: u32 = 14;
    pub const LEVEL: u32 = 15;
    pub const ABILITY_RAW: u32 = 16;
    pub const ABILITY: u32 = 17;
    pub const FA_BYTE: u32 = 18;
    pub const FORM: u32 = 19;
    pub const EXP: u32 = 20;
    pub const PERSONAL_1: u32 = 21;
}

/// Field effect checked by the Def/SpD swap (`FUN_overlay_d_93__021ef948(6)`), presumably
/// Wonder Room.
pub const FIELD_EFFECT_SWAP_DEFENSES: u32 = 6;
/// Condition that suppresses the ability in `BattleMon_Get(ABILITY)` (presumably Gastro Acid).
pub const SICK_ABILITY_SUPPRESSED: u32 = 0x10;
/// Condition that removes the Flying type (presumably Roost).
pub const SICK_GROUNDED_FLYING: u32 = 0x18;
/// Condition that ends when its source move is no longer known (`0x1B`).
pub const SICK_MOVE_LINKED: u32 = 0x1B;
/// Flag in bitset B that adds 2 critical-hit stages (presumably Focus Energy).
pub const FLAG_B_CRIT_BOOST: u32 = 9;

/// Undecompiled routines `BTL_POKEPARAM` code calls. Party Pokemon are opaque `u32` handles
/// (the original pointers).
pub trait BppEnv {
    /// C: `FUN_02017E1C(pp, id, NULL)`: read a party Pokemon parameter.
    fn pp_get(&mut self, pp: u32, id: u32) -> u32;
    /// C: `FUN_02017E40(pp, id, value)`: write a party Pokemon parameter.
    fn pp_put(&mut self, pp: u32, id: u32, value: u32);
    /// C: `FUN_02017D30(pp)`: begin fast (decrypted) access; returns a token for `fast_end`.
    fn pp_fast_begin(&mut self, pp: u32) -> u32;
    /// C: `FUN_02017D70(pp, token)`.
    fn pp_fast_end(&mut self, pp: u32, token: u32);
    /// C: `FUN_020181F4(pp, move, slot)`: set a move in a slot.
    fn pp_set_move(&mut self, pp: u32, mv: u32, slot: u32);
    /// C: `FUN_02017F00(pp)`.
    fn pp_fun_02017f00(&mut self, pp: u32) -> u32;
    /// C: `FUN_02017F90(pp)`: the party Pokemon's major status condition (0 = none).
    fn pp_status(&mut self, pp: u32) -> u32;
    /// C: `FUN_0201C158(move, pp_ups)`: max PP of a move with PP Ups applied.
    fn move_max_pp(&mut self, mv: u32, pp_ups: u32) -> u32;
    /// C: `FUN_02019C90(species, form, param)`: personal data.
    fn personal(&mut self, species: u32, form: u32, param: u32) -> u32;
    /// C: `FUN_overlay_d_93__021ef948(id)`: whether a field effect is active.
    fn field_effect_active(&mut self, id: u32) -> bool;
    /// C: `FUN_02018584(species, form, level)`: total EXP needed for `level`.
    fn exp_for_level(&mut self, species: u32, form: u32, level: u32) -> u32;
    /// C: `FUN_020185C4(pp)`: recalculate the party Pokemon's stats.
    fn pp_recalc_stats(&mut self, pp: u32);
    /// C: `FUN_02017F9C(pp, status)`: set the party Pokemon's major status.
    fn pp_set_status(&mut self, pp: u32, status: u32);
    /// C: `FUN_02017988(pp, form)`: change the party Pokemon's form.
    fn pp_set_form(&mut self, pp: u32, form: u32);
    /// C: `FUN_0202142C(value)`.
    fn fun_0202142c(&mut self, value: u32) -> u32;
    /// C: `FUN_021D66A0` / `thunk_EXT_FUN_0689c868`: calls ARM code at 0x0689C868 (autoload):
    /// whether a condition is carried over by `copy_passable_state` (Baton Pass).
    fn sick_is_passable(&mut self, sick: u32) -> bool;
}

/// One 14-byte move slot view (offsets relative to the slot).
pub mod move_slot {
    pub const ORIG_ID: usize = 0x0;
    pub const ORIG_PP: usize = 0x2;
    pub const ORIG_MAX_PP: usize = 0x3;
    pub const ORIG_PP_UPS: usize = 0x4;
    pub const ORIG_FLAGS: usize = 0x5;
    pub const CUR_ID: usize = 0x6;
    pub const CUR_PP: usize = 0x8;
    pub const CUR_MAX_PP: usize = 0x9;
    pub const CUR_PP_UPS: usize = 0xA;
    pub const CUR_FLAGS: usize = 0xB;
    pub const LINKED: usize = 0xC;
}
use move_slot as ms;

/// Marker type documenting the move slot layout ([`move_slot`] holds the offsets).
pub struct MoveSlot;

/// C: `BTL_POKEPARAM`.
#[derive(Clone, PartialEq, Eq)]
pub struct BattleMon {
    pub raw: [u8; BPP_SIZE],
}

impl core::fmt::Debug for BattleMon {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BattleMon")
            .field("species", &self.species())
            .field("poke_id", &self.poke_id())
            .field("hp", &self.hp())
            .finish_non_exhaustive()
    }
}

impl Default for BattleMon {
    fn default() -> Self {
        BattleMon { raw: [0; BPP_SIZE] }
    }
}

// ---- raw access -----------------------------------------------------------------------------

impl BattleMon {
    pub fn from_bytes(b: &[u8; BPP_SIZE]) -> Self {
        BattleMon { raw: *b }
    }

    pub fn u8_at(&self, o: usize) -> u8 {
        self.raw[o]
    }
    pub fn i8_at(&self, o: usize) -> i8 {
        self.raw[o] as i8
    }
    pub fn set_u8(&mut self, o: usize, v: u8) {
        self.raw[o] = v;
    }
    pub fn u16_at(&self, o: usize) -> u16 {
        u16::from_le_bytes([self.raw[o], self.raw[o + 1]])
    }
    pub fn set_u16(&mut self, o: usize, v: u16) {
        self.raw[o..o + 2].copy_from_slice(&v.to_le_bytes());
    }
    pub fn u32_at(&self, o: usize) -> u32 {
        u32::from_le_bytes(self.raw[o..o + 4].try_into().unwrap())
    }
    pub fn set_u32(&mut self, o: usize, v: u32) {
        self.raw[o..o + 4].copy_from_slice(&v.to_le_bytes());
    }

    fn slot(i: u32) -> usize {
        OFS_MOVES + MOVE_SLOT_SIZE * i as usize
    }

    // Named fields.
    pub fn pokemon(&self) -> u32 {
        self.u32_at(0x00)
    }
    pub fn species(&self) -> u16 {
        self.u16_at(0x0C)
    }
    pub fn max_hp(&self) -> u16 {
        self.u16_at(0x0E)
    }
    pub fn hp(&self) -> u16 {
        self.u16_at(0x10)
    }
    pub fn level(&self) -> u8 {
        self.u8_at(0x18)
    }
    pub fn poke_id(&self) -> u8 {
        self.u8_at(0x19)
    }
    pub fn stage(&self, i: usize) -> i8 {
        self.i8_at(OFS_STAGES + i)
    }
    pub fn sick_cont(&self, sick: u32) -> SickCont {
        SickCont(self.u32_at(OFS_SICK_CONT + 4 * sick as usize))
    }
    fn set_sick_cont(&mut self, sick: u32, c: SickCont) {
        self.set_u32(OFS_SICK_CONT + 4 * sick as usize, c.0);
    }
}

// ---- move slots -----------------------------------------------------------------------------

/// C: `FUN_overlay_d_93__021d5208`: loads `mv` into the slot half at `half` with full PP
/// (capped at `pp_cap` when non-zero).
fn move_half_load<E: BppEnv>(b: &mut BattleMon, half: usize, mv: u32, pp_cap: u32, env: &mut E) {
    b.set_u16(half, mv as u16);
    b.set_u8(half + 5, 0);
    let max = if mv != 0 {
        env.move_max_pp(mv, 0) as u8
    } else {
        0
    };
    b.set_u8(half + 3, max);
    if pp_cap != 0 && b.u8_at(half + 3) as u32 > pp_cap {
        b.set_u8(half + 3, pp_cap as u8);
    }
    b.set_u8(half + 2, b.u8_at(half + 3));
}

/// Copies the original half (+0..+6) to the current half (+6..+12) as three `u16`s.
fn move_copy_orig_to_cur(b: &mut BattleMon, s: usize) {
    for k in 0..3 {
        let v = b.u16_at(s + 2 * k);
        b.set_u16(s + 6 + 2 * k, v);
    }
}

impl BattleMon {
    /// C: `FUN_overlay_d_93__021d51cc`: clears the "used" low nibble of both flag bytes.
    pub fn move_clear_used(&mut self, slot: u32) {
        let s = Self::slot(slot);
        self.raw[s + ms::CUR_FLAGS] &= !0xF;
        self.raw[s + ms::ORIG_FLAGS] &= !0xF;
    }

    /// C: `FUN_overlay_d_93__021d51dc` (via `021d5734`): replaces a move. With `permanent`,
    /// the original half is rewritten (and copied to the current half while linked);
    /// otherwise only the current half changes and the slot is unlinked.
    pub fn move_replace<E: BppEnv>(
        &mut self,
        slot: u32,
        mv: u32,
        pp_cap: u32,
        permanent: bool,
        env: &mut E,
    ) {
        let s = Self::slot(slot);
        if permanent {
            move_half_load(self, s, mv, pp_cap, env);
            if self.u8_at(s + ms::LINKED) != 0 {
                move_copy_orig_to_cur(self, s);
            }
        } else {
            move_half_load(self, s + ms::CUR_ID, mv, pp_cap, env);
            self.set_u8(s + ms::LINKED, 0);
        }
    }

    /// C: `FUN_overlay_d_93__021d5244`: reloads the original half of `slot` from the party
    /// Pokemon. Returns whether the slot holds a move.
    pub fn move_load_from_party<E: BppEnv>(&mut self, slot: u32, pp: u32, env: &mut E) -> bool {
        let s = Self::slot(slot);
        let id = env.pp_get(pp, pp::MOVE1 + slot) as u16;
        if self.u16_at(s) != id {
            self.raw[s + ms::ORIG_FLAGS] = 0;
        }
        self.set_u16(s, id);
        if id != 0 {
            let v = env.pp_get(pp, pp::MOVE1_PP + slot) as u8;
            self.set_u8(s + ms::ORIG_PP, v);
            let v = env.pp_get(pp, pp::MOVE1_MAX_PP + slot) as u8;
            self.set_u8(s + ms::ORIG_MAX_PP, v);
            let v = env.pp_get(pp, pp::MOVE1_PP_UPS + slot) as u8;
            self.set_u8(s + ms::ORIG_PP_UPS, v);
            true
        } else {
            self.set_u8(s + ms::ORIG_PP, 0);
            self.set_u8(s + ms::ORIG_MAX_PP, 0);
            self.set_u8(s + ms::ORIG_PP_UPS, 0);
            false
        }
    }

    /// C: `FUN_overlay_d_93__021d4f84`: loads all four moves from the party Pokemon; with
    /// `reset`, clears flags first and copies them into linked current halves. Returns the
    /// number of slots holding a move.
    pub fn moves_load<E: BppEnv>(&mut self, pp: u32, reset: bool, env: &mut E) -> u32 {
        let token = env.pp_fast_begin(pp) as u8 as u32;
        let mut n = 0;
        if reset {
            for i in 0..4 {
                self.raw[Self::slot(i) + ms::ORIG_FLAGS] = 0;
            }
        }
        for i in 0..4 {
            if self.move_load_from_party(i, pp, env) {
                n += 1;
            }
        }
        if reset {
            for i in 0..4 {
                let s = Self::slot(i);
                move_copy_orig_to_cur(self, s);
                self.set_u8(s + ms::LINKED, 1);
            }
        }
        env.pp_fast_end(pp, token);
        n
    }

    /// C: `FUN_overlay_d_93__021d505c`: writes the original moves, PP and PP Ups back to the
    /// party Pokemon.
    pub fn moves_store<E: BppEnv>(&self, env: &mut E) {
        let pp = self.pokemon();
        for i in 0..4 {
            let s = Self::slot(i);
            env.pp_set_move(pp, self.u16_at(s) as u32, i);
            env.pp_put(pp, pp::MOVE1_PP + i, self.u8_at(s + ms::ORIG_PP) as u32);
            env.pp_put(
                pp,
                pp::MOVE1_PP_UPS + i,
                self.u8_at(s + ms::ORIG_PP_UPS) as u32,
            );
        }
    }

    /// C: `FUN_overlay_d_93__021d50b4`: reloads the original moves, recounts them, and
    /// refreshes linked current halves.
    pub fn moves_reload<E: BppEnv>(&mut self, env: &mut E) {
        let pp = self.pokemon();
        self.set_u8(OFS_MOVE_COUNT, 0);
        for i in 0..4 {
            let s = Self::slot(i);
            if self.move_load_from_party(i, pp, env) {
                self.raw[OFS_MOVE_COUNT] = self.raw[OFS_MOVE_COUNT].wrapping_add(1);
            }
            if self.u8_at(s + ms::LINKED) != 0 {
                move_copy_orig_to_cur(self, s);
            }
        }
    }

    /// C: `FUN_overlay_d_93__021d5150`: copies every original half to the current half,
    /// relinks, and recounts moves.
    pub fn moves_restore(&mut self) {
        self.set_u8(OFS_MOVE_COUNT, 0);
        for i in 0..4 {
            let s = Self::slot(i);
            move_copy_orig_to_cur(self, s);
            if self.u16_at(s + ms::CUR_ID) != 0 {
                self.raw[OFS_MOVE_COUNT] = self.raw[OFS_MOVE_COUNT].wrapping_add(1);
            }
            self.set_u8(s + ms::LINKED, 1);
        }
    }

    /// C: `FUN_overlay_d_93__021d5448`
    pub fn move_count(&self) -> u8 {
        self.u8_at(OFS_MOVE_COUNT)
    }

    /// C: `FUN_overlay_d_93__021d5450`: number of leading slots with an original move.
    pub fn orig_move_count(&self) -> u8 {
        let mut n = 0u8;
        for i in 0..4 {
            if self.u16_at(Self::slot(i)) == 0 {
                break;
            }
            n += 1;
        }
        n
    }

    /// C: `FUN_overlay_d_93__021d5478`: moves (of the first `move_count`) used this battle.
    pub fn used_move_count(&self) -> u8 {
        let mut n = 0u8;
        let mut i = 0u8;
        while i < self.move_count() {
            if self.u8_at(Self::slot(i as u32) + ms::CUR_FLAGS) & 0xF != 0 {
                n = n.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }
        n
    }

    /// C: `FUN_overlay_d_93__021d54b4`
    pub fn distinct_moves_used(&self) -> u8 {
        self.u8_at(OFS_MOVES_USED)
    }

    /// C: `FUN_overlay_d_93__021d54c0`: current move ID in `slot`.
    pub fn move_id(&self, slot: u32) -> u16 {
        self.u16_at(Self::slot(slot) + ms::CUR_ID)
    }

    /// C: unlabeled routine at `0x021D54D0`: original move ID in `slot`.
    pub fn orig_move_id(&self, slot: u32) -> u16 {
        self.u16_at(Self::slot(slot) + ms::ORIG_ID)
    }

    /// C: unlabeled routine at `0x021D54E0`: low nibble of the current flags.
    pub fn move_used_flag(&self, slot: u32) -> u8 {
        self.u8_at(Self::slot(slot) + ms::CUR_FLAGS) & 0xF
    }

    /// C: `FUN_overlay_d_93__021d54f4`: copies all move slots and the move count into `dst`.
    pub fn copy_moves_to(&self, dst: &mut BattleMon) {
        let end = OFS_MOVES + 4 * MOVE_SLOT_SIZE;
        dst.raw[OFS_MOVES..end].copy_from_slice(&self.raw[OFS_MOVES..end]);
        dst.raw[OFS_MOVE_COUNT] = self.raw[OFS_MOVE_COUNT];
    }

    /// C: `FUN_overlay_d_93__021d5528`: `(move, pp, max_pp)` of the current half.
    pub fn move_info(&self, slot: u32) -> (u16, u8, u8) {
        let s = Self::slot(slot);
        (
            self.u16_at(s + ms::CUR_ID),
            self.u8_at(s + ms::CUR_PP),
            self.u8_at(s + ms::CUR_MAX_PP),
        )
    }

    /// C: `FUN_overlay_d_93__021d554c`: PP missing from the current half.
    pub fn move_pp_missing(&self, slot: u32) -> u8 {
        let s = Self::slot(slot);
        self.u8_at(s + ms::CUR_MAX_PP)
            .wrapping_sub(self.u8_at(s + ms::CUR_PP))
    }

    /// C: unlabeled routine at `0x021D5568`: PP missing from the original half.
    pub fn orig_move_pp_missing(&self, slot: u32) -> u8 {
        let s = Self::slot(slot);
        self.u8_at(s + ms::ORIG_MAX_PP)
            .wrapping_sub(self.u8_at(s + ms::ORIG_PP))
    }

    /// C: `FUN_overlay_d_93__021d5584`: current PP of `slot`, or 0 past `move_count`.
    pub fn move_pp(&self, slot: u32) -> u8 {
        if slot < self.move_count() as u32 {
            self.u8_at(Self::slot(slot) + ms::CUR_PP)
        } else {
            0
        }
    }

    /// C: `FUN_overlay_d_93__021d55a0`: current PP of move `mv`, or 0 if not known.
    pub fn move_pp_of(&self, mv: u32) -> u8 {
        let mut i = 0u32;
        while i < self.move_count() as u32 {
            let s = Self::slot(i);
            if mv == self.u16_at(s + ms::CUR_ID) as u32 {
                return self.u8_at(s + ms::CUR_PP);
            }
            i += 1;
        }
        0
    }

    /// C: `FUN_overlay_d_93__021d55d8`: slot holds a move with full PP (original half when
    /// `orig`, else current).
    pub fn move_pp_full(&self, slot: u32, orig: bool) -> bool {
        let s = Self::slot(slot) + if orig { 0 } else { ms::CUR_ID };
        self.u16_at(s) != 0 && self.u8_at(s + 2) == self.u8_at(s + 3)
    }

    /// C: `FUN_overlay_d_93__021d5608`: spends current PP; while linked, also the original
    /// PP and the party Pokemon's.
    pub fn move_spend_pp<E: BppEnv>(&mut self, slot: u32, amount: u32, env: &mut E) {
        let s = Self::slot(slot);
        let cur = self.u8_at(s + ms::CUR_PP);
        let v = if cur as u32 >= amount {
            (cur as u32).wrapping_sub(amount) as u8
        } else {
            0
        };
        self.set_u8(s + ms::CUR_PP, v);
        if self.u8_at(s + ms::LINKED) != 0 {
            self.set_u8(s + ms::ORIG_PP, v);
            env.pp_put(self.pokemon(), pp::MOVE1_PP + slot, v as u32);
        }
    }

    /// C: `FUN_overlay_d_93__021d5640`: spends original PP of an unlinked slot (and the party
    /// Pokemon's).
    pub fn move_spend_orig_pp<E: BppEnv>(&mut self, slot: u32, amount: u32, env: &mut E) {
        let s = Self::slot(slot);
        if self.u8_at(s + ms::LINKED) != 0 {
            return;
        }
        let mut v = (self.u8_at(s + ms::ORIG_PP) as i32).wrapping_sub(amount as i32);
        if v < 0 {
            v = 0;
        }
        self.set_u8(s + ms::ORIG_PP, v as u8);
        env.pp_put(
            self.pokemon(),
            pp::MOVE1_PP + slot,
            self.u8_at(s + ms::ORIG_PP) as u32,
        );
    }

    /// C: `FUN_overlay_d_93__021d5674`: restores current PP (capped); mirrors to the original
    /// when linked. Returns the current move ID.
    pub fn move_restore_pp(&mut self, slot: u32, amount: u32) -> u16 {
        let s = Self::slot(slot);
        let v = self.u8_at(s + ms::CUR_PP).wrapping_add(amount as u8);
        self.set_u8(s + ms::CUR_PP, v);
        let max = self.u8_at(s + ms::CUR_MAX_PP);
        if v > max {
            self.set_u8(s + ms::CUR_PP, max);
        }
        if self.u8_at(s + ms::LINKED) != 0 {
            self.set_u8(s + ms::ORIG_PP, self.u8_at(s + ms::CUR_PP));
        }
        self.u16_at(s + ms::CUR_ID)
    }

    /// C: `FUN_overlay_d_93__021d56a0`: restores original PP (capped); mirrors to the current
    /// half when linked. Returns the original move ID.
    pub fn move_restore_orig_pp(&mut self, slot: u32, amount: u32) -> u16 {
        let s = Self::slot(slot);
        let v = self.u8_at(s + ms::ORIG_PP).wrapping_add(amount as u8);
        self.set_u8(s + ms::ORIG_PP, v);
        let max = self.u8_at(s + ms::ORIG_MAX_PP);
        if v > max {
            self.set_u8(s + ms::ORIG_PP, max);
        }
        if self.u8_at(s + ms::LINKED) != 0 {
            self.set_u8(s + ms::CUR_PP, self.u8_at(s + ms::ORIG_PP));
        }
        self.u16_at(s + ms::ORIG_ID)
    }

    /// C: `FUN_overlay_d_93__021d56cc`: marks a move used (counting first uses).
    pub fn move_mark_used(&mut self, slot: u32) {
        let s = Self::slot(slot);
        let f = self.u8_at(s + ms::CUR_FLAGS);
        if f >> 4 == 0 {
            self.set_u8(s + ms::CUR_FLAGS, (f & 0x0F) | 0x10);
            self.raw[OFS_MOVES_USED] = self.raw[OFS_MOVES_USED].wrapping_add(1);
        }
        let f = self.u8_at(s + ms::CUR_FLAGS);
        self.set_u8(s + ms::CUR_FLAGS, (f & 0xF0) | 1);
        if self.u8_at(s + ms::LINKED) != 0 {
            let cur = self.u8_at(s + ms::CUR_FLAGS);
            let o = self.u8_at(s + ms::ORIG_FLAGS);
            let o = (o & 0xF0) | (cur & 0xF);
            self.set_u8(s + ms::ORIG_FLAGS, (o & 0x0F) | (cur & 0xF0));
        }
    }

    /// C: `FUN_overlay_d_93__021d5750`: knows `mv` (current half).
    pub fn knows_move(&self, mv: u32) -> bool {
        (0..4).any(|i| mv == self.u16_at(Self::slot(i) + ms::CUR_ID) as u32)
    }

    /// C: `FUN_overlay_d_93__021d5778`: slot of `mv`, or 4 (also for `mv == 0`).
    pub fn move_slot_of(&self, mv: u32) -> u8 {
        if mv != 0 {
            for i in 0..4u8 {
                if mv == self.u16_at(Self::slot(i as u32) + ms::CUR_ID) as u32 {
                    return i;
                }
            }
        }
        4
    }

    /// C: `FUN_overlay_d_93__021d5364`: clears used flags and per-switch-in state.
    pub fn reset_switch_state(&mut self) {
        for i in 0..4 {
            self.move_clear_used(i);
        }
        self.set_u16(0x14A, 0);
        self.set_u16(0x14C, 0);
        self.set_u8(0x144, 0x11);
        self.set_u16(0x14E, 0);
    }

    /// C: `FUN_overlay_d_93__021d539c`: clears the five bytes at +0x157.
    pub fn clear_157(&mut self) {
        self.raw[0x157..0x15C].fill(0);
    }
}

// ---- stages ---------------------------------------------------------------------------------

/// C: `FUN_overlay_d_93__021d53c0` / `LAB_overlay_d_93__021d53c8`: sets all seven stages to
/// neutral.
fn stages_reset(s: &mut [u8]) {
    s[..7].fill(STAGE_NEUTRAL as u8);
}

/// C: `LAB_overlay_d_93__021d53dc`: raises negative stages to neutral; returns whether any
/// changed.
fn stages_clear_negative(s: &mut [u8]) -> bool {
    let mut changed = false;
    for v in s[..7].iter_mut() {
        if (*v as i8) < STAGE_NEUTRAL {
            *v = STAGE_NEUTRAL as u8;
            changed = true;
        }
    }
    changed
}

/// C: `FUN_overlay_d_93__021d5cb4`: offset of the stage byte for stat ID 1..=7, else `None`
/// (the original returns a pointer, or NULL, and reports the bounds 0 and 12).
fn stage_offset(stat: u32) -> Option<usize> {
    match stat {
        1..=7 => Some(OFS_STAGES + stat as usize - 1),
        _ => None,
    }
}

impl BattleMon {
    /// C: `FUN_overlay_d_93__021d5f08`
    pub fn reset_stages(&mut self) {
        stages_reset(&mut self.raw[OFS_STAGES..]);
    }

    /// C: `FUN_overlay_d_93__021d5efc`
    pub fn clear_negative_stages(&mut self) -> bool {
        stages_clear_negative(&mut self.raw[OFS_STAGES..])
    }

    /// C: `FUN_overlay_d_93__021d5d68`: any stage below neutral.
    pub fn has_negative_stage(&self) -> bool {
        (0..7).any(|i| self.stage(i) < STAGE_NEUTRAL)
    }

    /// C: `FUN_overlay_d_93__021d5d14`: whether `stat`'s stage can move in the direction of
    /// `delta`. `stat` must be 1..=7 (the original dereferences NULL otherwise).
    pub fn can_change_stage(&self, stat: u32, delta: i32) -> bool {
        let o = stage_offset(stat).expect("stat 1..=7");
        let v = self.i8_at(o);
        if delta > 0 {
            v < STAGE_MAX
        } else {
            v > 0
        }
    }

    /// C: unlabeled routine at `0x021D5D50`: stages left before the maximum.
    pub fn stage_headroom(&self, stat: u32) -> i32 {
        let o = stage_offset(stat).expect("stat 1..=7");
        STAGE_MAX as i32 - self.i8_at(o) as i32
    }

    /// C: `FUN_overlay_d_93__021d5dc0`: raises a stage by up to `amount`; returns the amount
    /// applied (or `amount` unchanged when not clamped, as the original returns its register).
    pub fn raise_stage(&mut self, stat: u32, amount: u32) -> u32 {
        let Some(o) = stage_offset(stat) else {
            return 0;
        };
        let v = self.i8_at(o) as i32;
        if v >= STAGE_MAX as i32 {
            return 0;
        }
        let mut amount = amount;
        if v.wrapping_add(amount as i32) > STAGE_MAX as i32 {
            amount = (STAGE_MAX as i32 - v) as u8 as u32;
        }
        self.set_u8(o, (v as i8).wrapping_add(amount as u8 as i8) as u8);
        amount
    }

    /// C: `FUN_overlay_d_93__021d5e38`: lowers a stage by up to `amount`; returns the amount
    /// applied.
    pub fn lower_stage(&mut self, stat: u32, amount: u32) -> u32 {
        let Some(o) = stage_offset(stat) else {
            return 0;
        };
        let v = self.i8_at(o) as i32;
        if v <= 0 {
            return 0;
        }
        let mut amount = amount;
        if v.wrapping_sub(amount as i32) < 0 {
            amount = v as u8 as u32;
        }
        self.set_u8(o, (v as i8).wrapping_sub(amount as u8 as i8) as u8);
        amount
    }

    /// C: `FUN_overlay_d_93__021d5ea8`: sets a stage when `value <= 12`.
    pub fn set_stage(&mut self, stat: u32, value: u32) {
        if let Some(o) = stage_offset(stat) {
            if value <= STAGE_MAX as u32 {
                self.set_u8(o, value as u8);
            }
        }
    }

    /// C: `FUN_overlay_d_93__021d5f14`: critical-hit stage including the flag-B boost.
    pub fn crit_stage(&self) -> u8 {
        let mut v = self.u8_at(OFS_CRIT_BOOST);
        if self.flag_b(FLAG_B_CRIT_BOOST) {
            v = v.wrapping_add(2);
            if v > 4 {
                v = 4;
            }
        }
        v
    }

    /// C: `FUN_overlay_d_93__021d5f38`: adds to the crit boost (0..=4); returns whether it
    /// could change.
    pub fn add_crit_boost(&mut self, delta: i32) -> bool {
        let cur = self.u8_at(OFS_CRIT_BOOST);
        if delta > 0 {
            if cur >= 4 {
                return false;
            }
            let v = cur.wrapping_add(delta as u8);
            self.set_u8(OFS_CRIT_BOOST, if v > 4 { 4 } else { v });
            true
        } else {
            let neg = delta.wrapping_mul(-1);
            if cur == 0 {
                return false;
            }
            let v = if (cur as i32) > neg {
                cur.wrapping_sub(neg as u8)
            } else {
                0
            };
            self.set_u8(OFS_CRIT_BOOST, v);
            true
        }
    }
}

// ---- HP -------------------------------------------------------------------------------------

impl BattleMon {
    /// C: `FUN_overlay_d_93__021d5f90`
    pub fn damage(&mut self, amount: u32) {
        let hp = self.hp() as u32;
        self.set_u16(0x10, if hp > amount { (hp - amount) as u16 } else { 0 });
    }

    /// C: `FUN_overlay_d_93__021d5fa4`
    pub fn heal(&mut self, amount: u32) {
        self.set_u16(0x10, (self.hp() as u32).wrapping_add(amount) as u16);
        if self.hp() > self.max_hp() {
            self.set_u16(0x10, self.max_hp());
        }
    }

    /// C: `FUN_overlay_d_93__021d5fb8`
    pub fn faint(&mut self) {
        self.set_u16(0x10, 0);
    }

    /// C: `FUN_overlay_d_93__021d5b40` / `021d5b44`
    pub fn held_item(&self) -> u16 {
        self.u16_at(0x12)
    }
    pub fn set_held_item(&mut self, item: u16) {
        self.set_u16(0x12, item);
    }

    /// C: unlabeled routine at `0x021D5BA4`
    pub fn u16_146(&self) -> u16 {
        self.u16_at(0x146)
    }
}

// ---- flag bitsets ---------------------------------------------------------------------------

fn bit_of(n: u32) -> (usize, u8) {
    (((n << 21) >> 24) as usize, (1u32 << (n & 7)) as u8)
}

impl BattleMon {
    /// C: `FUN_overlay_d_93__021d5bb0`
    pub fn flag_a(&self, n: u32) -> bool {
        let (i, m) = bit_of(n);
        self.raw[OFS_FLAGS_A + i] & m != 0
    }
    /// C: `FUN_overlay_d_93__021d5fc0`
    pub fn set_flag_a(&mut self, n: u32) {
        let (i, m) = bit_of(n);
        self.raw[OFS_FLAGS_A + i] |= m;
    }
    /// C: `FUN_overlay_d_93__021d5bdc`
    pub fn flag_b(&self, n: u32) -> bool {
        let (i, m) = bit_of(n);
        self.raw[OFS_FLAGS_B + i] & m != 0
    }
    /// C: `FUN_overlay_d_93__021d5fe4`
    pub fn set_flag_b(&mut self, n: u32) {
        let (i, m) = bit_of(n);
        self.raw[OFS_FLAGS_B + i] |= m;
    }
    /// C: `FUN_overlay_d_93__021d6008`
    pub fn clear_flag_b(&mut self, n: u32) {
        let (i, m) = bit_of(n);
        self.raw[OFS_FLAGS_B + i] &= !m;
    }

    /// C: `FUN_overlay_d_93__021d5c08`: first of flag-B bits 3, 5, 4, 6 that is set, else 0x10.
    pub fn first_flag_b_3546(&self) -> u32 {
        const ORDER: [u32; 4] = [3, 5, 4, 6]; // 0x021F040C (battle_pokeparam rodata)
        ORDER
            .iter()
            .copied()
            .find(|&n| self.flag_b(n))
            .unwrap_or(0x10)
    }

    /// C: `FUN_overlay_d_93__021d5c30`
    pub fn any_flag_b_3546(&self) -> bool {
        self.first_flag_b_3546() != 0x10
    }
}

// ---- Illusion -------------------------------------------------------------------------------

impl BattleMon {
    /// C: `BattleMon_GetPokemon`
    pub fn get_pokemon(&self) -> u32 {
        self.pokemon()
    }
    /// C: `BattleMon_SetIllusionMon`
    pub fn set_illusion_mon(&mut self, pp: u32) {
        self.set_u32(0x04, pp);
        self.raw[0x1B] |= 0x40;
    }
    /// C: `BattleMon_ClearIllusionMon` / `FUN_overlay_d_93__021d6c2c`
    pub fn clear_illusion_mon(&mut self) {
        self.set_u32(0x04, 0);
        self.raw[0x1B] &= !0x40;
    }
    /// C: `FUN_overlay_d_93__021d6c24`: Illusion is active.
    pub fn is_illusion_active(&self) -> bool {
        self.raw[0x1B] & 0x40 != 0
    }

    /// C: `FUN_021D4D5C` / `FUN_021D53B8` (tail calls to `Heap_Free`): releases the battle
    /// Pokemon. Rust frees on drop.
    pub fn free(self) {}

    /// C: `BattleMon_GetVisibleMon`: the disguise while Illusion is active, else the Pokemon.
    pub fn visible_mon(&self) -> u32 {
        let ill = self.u32_at(0x04);
        if ill != 0 && self.raw[0x1B] & 0x40 != 0 {
            ill
        } else {
            self.pokemon()
        }
    }
}

#[cfg(test)]
mod tests;

// ---- types ----------------------------------------------------------------------------------

impl BattleMon {
    /// C: `FUN_overlay_d_93__021d57a8`: effective types. Flying is removed while
    /// [`SICK_GROUNDED_FLYING`] is active; an emptied slot copies the other (or Normal when
    /// both are empty).
    pub fn types(&self) -> (u8, u8) {
        use crate::poke_type::types::{FLYING, NONE, NORMAL};
        let grounded = self.has_sick(SICK_GROUNDED_FLYING);
        let mut t1 = self.u8_at(0xF8);
        if t1 == FLYING && grounded {
            t1 = NONE;
        }
        let mut t2 = self.u8_at(0xF9);
        if t2 == FLYING && grounded {
            t2 = NONE;
        }
        if t1 == NONE {
            if t2 == NONE {
                t2 = NORMAL;
            }
            t1 = t2;
        } else if t2 == NONE {
            t2 = t1;
        }
        (t1, t2)
    }

    /// C: `FUN_overlay_d_93__021d57fc`: effective types as a packed pair.
    pub fn type_pair(&self) -> u16 {
        let (a, b) = self.types();
        crate::poke_type::make_pair(a, b)
    }

    /// C: `FUN_overlay_d_93__021d5814`
    pub fn has_type(&self, ty: u32) -> bool {
        if ty == crate::poke_type::types::NONE as u32 {
            return false;
        }
        let (a, b) = self.types();
        a as u32 == ty || b as u32 == ty
    }

    /// C: `FUN_overlay_d_93__021d66b0`: sets both base types from a pair.
    pub fn set_type_pair(&mut self, pair: u16) {
        self.set_u8(0xF8, crate::poke_type::pair_type1(pair));
        self.set_u8(0xF9, crate::poke_type::pair_type2(pair));
    }

    /// C: `FUN_overlay_d_93__021d66d0`
    pub fn set_ability(&mut self, ability: u16) {
        self.set_u16(0x13C, ability);
    }
}

// ---- BattleMon_Get --------------------------------------------------------------------------

impl BattleMon {
    /// C: `FUN_overlay_d_93__021d587c`: swaps the Def/SpD IDs while
    /// [`FIELD_EFFECT_SWAP_DEFENSES`] is active.
    pub fn remap_value_id<E: BppEnv>(id: u32, env: &mut E) -> u32 {
        match id {
            value::DEFENSE if env.field_effect_active(FIELD_EFFECT_SWAP_DEFENSES) => {
                value::SP_DEFENSE
            }
            value::SP_DEFENSE if env.field_effect_active(FIELD_EFFECT_SWAP_DEFENSES) => {
                value::DEFENSE
            }
            _ => id,
        }
    }

    /// C: `FUN_overlay_d_93__021d58a8`: stat without stages (IDs 8-12; 6 and 7 give 6;
    /// others defer to [`Self::get`] with the remapped ID, which remaps again).
    pub fn get_unstaged<E: BppEnv>(&self, id: u32, env: &mut E) -> u32 {
        let id = Self::remap_value_id(id, env);
        match id {
            value::ATTACK..=value::SPEED => {
                self.u16_at(OFS_STATS + 2 * (id - value::ATTACK) as usize) as u32
            }
            value::ACCURACY_STAGE | value::EVASION_STAGE => 6,
            _ => self.get(id, env),
        }
    }

    /// C: `FUN_overlay_d_93__021d590c`: reads max HP and the five stats from the party
    /// Pokemon into `out` (u16 at +2..+0xC; +0 untouched).
    pub fn read_party_stats<E: BppEnv>(&self, out: &mut [u16; 7], env: &mut E) {
        let pp = self.pokemon();
        let token = env.pp_fast_begin(pp) as u8 as u32;
        for (i, id) in [
            pp::MAX_HP,
            pp::ATTACK,
            pp::DEFENSE,
            pp::SP_ATTACK,
            pp::SP_DEFENSE,
            pp::SPEED,
        ]
        .into_iter()
        .enumerate()
        {
            out[1 + i] = env.pp_get(pp, id) as u16;
        }
        env.pp_fast_end(pp, token);
    }

    /// C: `FUN_overlay_d_93__021d5970`: sets a raw stat (IDs 8-12 after remapping).
    pub fn set_stat<E: BppEnv>(&mut self, id: u32, v: u16, env: &mut E) {
        let id = Self::remap_value_id(id, env);
        if (value::ATTACK..=value::SPEED).contains(&id) {
            self.set_u16(OFS_STATS + 2 * (id - value::ATTACK) as usize, v);
        }
    }

    /// C: `BattleMon_Get`
    pub fn get<E: BppEnv>(&self, id: u32, env: &mut E) -> u32 {
        let id = Self::remap_value_id(id, env);
        match id {
            value::ATTACK_STAGE..=value::EVASION_STAGE => {
                self.i8_at(OFS_STAGES + (id - 1) as usize) as i32 as u32
            }
            value::ATTACK..=value::SPEED => {
                let stat = self.get_unstaged(id, env) as u16 as u32;
                let stage = self.u8_at(OFS_STAGES + (id - value::ATTACK) as usize) as u32;
                crate::calc::apply_stat_stage(stat, stage) as u32
            }
            value::HP => self.hp() as u32,
            value::MAX_HP => self.max_hp() as u32,
            value::LEVEL => self.level() as u32,
            value::ABILITY_RAW => self.u16_at(0x13C) as u32,
            value::ABILITY => {
                if self.has_sick(SICK_ABILITY_SUPPRESSED) {
                    0
                } else {
                    self.u16_at(0x13C) as u32
                }
            }
            value::FA_BYTE => self.u8_at(0xFA) as u32,
            value::FORM => self.u8_at(OFS_FORM) as u32,
            value::EXP => self.u32_at(0x08),
            value::PERSONAL_1 => self.u8_at(0x1A) as u32,
            _ => 0,
        }
    }

    /// C: `FUN_overlay_d_93__021d5ad8`: stat for a critical hit: ignores a lowered Atk/SpA
    /// and a raised Def/SpD.
    pub fn get_for_crit<E: BppEnv>(&self, id: u32, env: &mut E) -> u32 {
        let r = Self::remap_value_id(id, env);
        let ignore = match r {
            value::ATTACK => self.stage(0) < STAGE_NEUTRAL,
            value::DEFENSE => self.stage(1) > STAGE_NEUTRAL,
            value::SP_ATTACK => self.stage(2) < STAGE_NEUTRAL,
            value::SP_DEFENSE => self.stage(3) > STAGE_NEUTRAL,
            _ => false,
        };
        if ignore {
            self.get_unstaged(id, env)
        } else {
            self.get(id, env)
        }
    }

    /// C: `FUN_overlay_d_93__021d5b48`
    pub fn is_full_hp<E: BppEnv>(&self, env: &mut E) -> bool {
        self.get(value::HP, env) == self.get(value::MAX_HP, env)
    }

    /// C: `FUN_overlay_d_93__021d5b68`
    pub fn is_fainted<E: BppEnv>(&self, env: &mut E) -> bool {
        self.get(value::HP, env) == 0
    }

    /// C: `FUN_overlay_d_93__021d5b7c`: not an egg and not fainted.
    pub fn is_alive<E: BppEnv>(&self, env: &mut E) -> bool {
        if env.pp_get(self.pokemon(), pp::IS_EGG) != 0 {
            return false;
        }
        !self.is_fainted(env)
    }

    /// C: `FUN_overlay_d_93__021d5c44`: HP percentage as 4.12 fixed point, rounded to nearest
    /// (computed in double precision like the original's soft-float calls).
    pub fn hp_percent_fx(&self) -> i32 {
        let x = (self.hp() as u32).wrapping_mul(100) as i32 as f64 / self.max_hp() as f64;
        if x > 0.0 {
            (x * 4096.0 + 0.5) as i32
        } else {
            (x * 4096.0 - 0.5) as i32
        }
    }
}

// ---- conditions -----------------------------------------------------------------------------

/// Major status conditions occupy sick IDs 1..=5.
fn is_major_sick(sick: u32) -> bool {
    (sick as i32) < 6
}

impl BattleMon {
    /// C: `FUN_overlay_d_93__021d62c4`
    pub fn has_sick(&self, sick: u32) -> bool {
        self.sick_cont(sick).kind() != 0
    }

    /// C: `FUN_overlay_d_93__021d6314`
    pub fn get_sick_cont(&self, sick: u32) -> SickCont {
        self.sick_cont(sick)
    }

    /// C: `FUN_overlay_d_93__021d631c`
    pub fn sick_turns(&self, sick: u32) -> u8 {
        self.u8_at(OFS_SICK_TURNS + sick as usize)
    }

    /// C: `FUN_overlay_d_93__021d62d8`: the condition's parameter: `param` (kind 2), the
    /// linked Pokemon ID (kinds 3 and 4), else 0.
    pub fn sick_param(&self, sick: u32) -> u16 {
        let c = self.sick_cont(sick);
        match c.kind() {
            2 => (c.0 >> 9) as u16,
            3 => ((c.0 >> 3) & 0x3F) as u16,
            4 => ((c.0 >> 9) & 0x3F) as u16,
            _ => 0,
        }
    }

    /// C: `FUN_overlay_d_93__021d62a8`: first active major status (1..=5), else 0.
    pub fn major_status(&self) -> u32 {
        (1..6).find(|&s| self.has_sick(s)).unwrap_or(0)
    }

    /// C: `FUN_overlay_d_93__021d6034`: sets a condition (clearing other major statuses
    /// first when `sick` is one).
    pub fn set_sick(&mut self, sick: u32, cont: SickCont) {
        if is_major_sick(sick) {
            self.cure_major_status();
        }
        self.set_sick_cont(sick, cont);
        self.set_u8(OFS_SICK_TURNS + sick as usize, 0);
    }

    /// C: `FUN_overlay_d_93__021d61f0`
    pub fn cure_major_status(&mut self) {
        for s in 1..6 {
            self.set_sick_cont(s, SickCont::NONE);
            self.set_u8(OFS_SICK_TURNS + s as usize, 0);
            self.on_sick_cured(s);
        }
    }

    /// C: `FUN_overlay_d_93__021d621c`: curing sleep also ends condition 9 (Nightmare).
    fn on_sick_cured(&mut self, sick: u32) {
        if sick == 2 {
            self.set_sick_cont(9, SickCont::NONE);
            self.set_u8(OFS_SICK_TURNS + 9, 0);
        }
    }

    /// C: `FUN_overlay_d_93__021d6238`
    pub fn cure_sick(&mut self, sick: u32) {
        if is_major_sick(sick) {
            self.cure_major_status();
        } else {
            self.set_sick_cont(sick, SickCont::NONE);
            self.set_u8(OFS_SICK_TURNS + sick as usize, 0);
        }
    }

    /// C: `FUN_overlay_d_93__021d6264`: ends every condition linked to `poke_id` (none for
    /// 0x1F).
    pub fn cure_sicks_linked_to(&mut self, poke_id: u32) {
        if poke_id == 0x1F {
            return;
        }
        for s in 0..SICK_COUNT as u32 {
            let c = self.sick_cont(s);
            if !c.is_none() && poke_id == c.linked_poke_id() {
                self.set_sick_cont(s, SickCont::NONE);
                self.on_sick_cured(s);
            }
        }
    }

    /// C: `FUN_overlay_d_93__021d6064`: end-of-turn bookkeeping for one condition. Returns
    /// whether it was active; `out_prev` receives the cont before the update and `out_ended`
    /// whether it ended. Sleep (2) and confusion (6) are handled elsewhere.
    pub fn tick_sick(
        &mut self,
        sick: u32,
        out_prev: Option<&mut SickCont>,
        mut out_ended: Option<&mut u32>,
    ) -> bool {
        if sick == 2 || sick == 6 {
            return false;
        }
        let c = self.sick_cont(sick);
        if c.kind() == 0 {
            return false;
        }
        let turns = c.turns();
        if let Some(p) = out_prev {
            *p = c;
        }
        if let Some(e) = out_ended.as_deref_mut() {
            *e = 0;
        }
        let ti = OFS_SICK_TURNS + sick as usize;
        if sick == SICK_MOVE_LINKED && !self.knows_move(c.param_any() as u32) {
            self.set_sick_cont(sick, SickCont::NONE);
            self.set_u8(ti, 0);
            if let Some(e) = out_ended.as_deref_mut() {
                *e = 1;
            }
        }
        if turns != 0 {
            self.raw[ti] = self.raw[ti].wrapping_add(1);
            if self.raw[ti] as u32 >= turns {
                self.set_sick_cont(sick, SickCont::NONE);
                self.set_u8(ti, 0);
                if let Some(e) = out_ended {
                    *e = 1;
                }
            }
        } else {
            let c = self.sick_cont(sick);
            if c.kind() == 1 {
                let cap = c.slot();
                if cap != 0 && (self.raw[ti] as u32) < cap {
                    self.raw[ti] += 1;
                }
            }
        }
        true
    }

    /// C: `FUN_overlay_d_93__021d6130`: advances sleep; Early Bird (ability 48) counts
    /// double. Returns whether the Pokemon woke up.
    pub fn tick_sleep(&mut self) -> bool {
        let c = self.sick_cont(2);
        if c.kind() == 0 {
            return false;
        }
        let turns = c.turns();
        if turns == 0 {
            return false;
        }
        let early_bird = !self.has_sick(SICK_ABILITY_SUPPRESSED) && self.u16_at(0x13C) == 0x30;
        let inc = if early_bird { 2 } else { 1 };
        let ti = OFS_SICK_TURNS + 2;
        self.raw[ti] = self.raw[ti].wrapping_add(inc);
        if (self.raw[ti] as u32) < turns {
            return false;
        }
        self.set_sick_cont(2, SickCont::NONE);
        self.cure_sick(2);
        true
    }

    /// C: `FUN_overlay_d_93__021d61a8`: advances confusion; returns whether it ended.
    pub fn tick_confusion(&mut self) -> bool {
        let c = self.sick_cont(6);
        if c.kind() == 0 {
            return false;
        }
        let turns = c.turns();
        if turns == 0 {
            return false;
        }
        let ti = OFS_SICK_TURNS + 6;
        self.raw[ti] = self.raw[ti].wrapping_add(1);
        if (self.raw[ti] as u32) < turns {
            return false;
        }
        self.set_sick_cont(6, SickCont::NONE);
        self.cure_sick(6);
        true
    }

    /// C: `FUN_overlay_d_93__021d6324`: end-of-turn damage from a condition: burn and poison
    /// 1/8 (bad poison 1/16 x turns), Nightmare 1/4 while asleep, condition 10 (Curse) 1/4.
    pub fn sick_damage<E: BppEnv>(&self, sick: u32, env: &mut E) -> u32 {
        if !self.has_sick(sick) {
            return 0;
        }
        let div = |d: u32, env: &mut E| -> u32 {
            match crate::calc::u32_div(self.get(value::MAX_HP, env), d) {
                0 => 1,
                v => v,
            }
        };
        match sick {
            5 => {
                if self.sick_cont(5).is_badly_poisoned() {
                    div(16, env).wrapping_mul(self.sick_turns(5) as u32)
                } else {
                    div(8, env)
                }
            }
            4 => div(8, env),
            9 => {
                if self.has_sick(2) {
                    div(4, env)
                } else {
                    0
                }
            }
            10 => div(4, env),
            _ => 0,
        }
    }

    /// C: `FUN_overlay_d_93__021d63b8`: clears conditions (all when `all`, else from 6 up,
    /// keeping major statuses) and every turn counter.
    pub fn clear_sicks(&mut self, all: bool) {
        let start = if all { 0 } else { 6 };
        for s in start..SICK_COUNT as u32 {
            self.set_sick_cont(s, SickCont::NONE);
        }
        self.raw[OFS_SICK_TURNS..OFS_SICK_TURNS + SICK_COUNT].fill(0);
    }
}

// ---- switching ------------------------------------------------------------------------------

impl BattleMon {
    /// C: `FUN_overlay_d_93__021d4efc`: refreshes fields from the party Pokemon.
    pub fn reload_from_party<E: BppEnv>(
        &mut self,
        pp: u32,
        with_hp: bool,
        with_ability: bool,
        env: &mut E,
    ) {
        if with_hp {
            let v = env.pp_get(pp, pp::HP) as u16;
            self.set_u16(0x10, v);
            let v = env.pp_get(pp, pp::MAX_HP) as u16;
            self.set_u16(0x0E, v);
        }
        let v = env.pp_get(pp, pp::EXP);
        self.set_u32(0x08, v);
        self.reload_types_and_stats(pp, with_ability, env);
        if with_ability {
            let v = env.pp_get(pp, pp::ABILITY) as u16;
            self.set_u16(0x13C, v);
        }
        let form = env.pp_get(pp, pp::FORM) as u8;
        self.set_u8(OFS_FORM, form);
        let w = env.personal(self.species() as u32, form as u32, 0x26) as u16;
        self.set_u16(0x13E, w);
        if self.u16_at(0x13E) < 1 {
            self.set_u16(0x13E, 1);
        }
    }

    /// C: `FUN_overlay_d_93__021d52b4`
    pub fn reload_types_and_stats<E: BppEnv>(&mut self, pp: u32, with_types: bool, env: &mut E) {
        if with_types {
            let v = env.pp_get(pp, pp::TYPE1) as u8;
            self.set_u8(0xF8, v);
            let v = env.pp_get(pp, pp::TYPE2) as u8;
            self.set_u8(0xF9, v);
        }
        let v = env.pp_fun_02017f00(pp) as u8;
        self.set_u8(0xFA, v);
        for (o, id) in [
            (0xEE, pp::ATTACK),
            (0xF0, pp::DEFENSE),
            (0xF2, pp::SP_ATTACK),
            (0xF4, pp::SP_DEFENSE),
            (0xF6, pp::SPEED),
        ] {
            let v = env.pp_get(pp, id) as u16;
            self.set_u16(o, v);
        }
    }

    /// C: `FUN_overlay_d_93__021d533c`: undoes Transform.
    pub fn undo_transform<E: BppEnv>(&mut self, env: &mut E) {
        if self.is_transformed() {
            self.reload_from_party(self.pokemon(), false, true, env);
            self.moves_restore();
            self.raw[0x1B] &= !0x20;
        }
    }

    /// C: `FUN_overlay_d_93__021d646c`: full reset when switching out.
    pub fn reset_on_switch_out<E: BppEnv>(&mut self, env: &mut E) {
        self.raw[OFS_FLAGS_A..OFS_FLAGS_A + 2].fill(0);
        self.raw[OFS_FLAGS_B..OFS_FLAGS_B + 2].fill(0);
        self.undo_transform(env);
        self.moves_restore();
        self.reset_switch_state();
        self.clear_157();
        self.clear_1f6();
        self.clear_illusion_mon();
        self.set_substitute_hp(0);
        self.clear_sicks(true);
        self.reset_stages();
        self.restore_form_and_ability(env);
    }

    fn restore_form_and_ability<E: BppEnv>(&mut self, env: &mut E) {
        let form = self.raw[0x1B] & 0x1F;
        self.set_u8(OFS_FORM, form);
        self.set_u16(0x13C, self.u16_at(0x16));
        env.pp_put(self.pokemon(), pp::FORM, self.u8_at(OFS_FORM) as u32);
        env.pp_recalc_stats(self.pokemon());
    }

    /// C: `FUN_overlay_d_93__021d64f0`: reset at the end of battle (or on faint); flag-B bit
    /// 14 keeps conditions, stages and flag set B.
    pub fn reset_on_battle_end<E: BppEnv>(&mut self, env: &mut E) {
        self.raw[OFS_FLAGS_A..OFS_FLAGS_A + 2].fill(0);
        self.undo_transform(env);
        self.moves_restore();
        self.reset_switch_state();
        self.clear_157();
        self.clear_1f6();
        self.clear_illusion_mon();
        if !self.flag_b(0xE) {
            self.set_substitute_hp(0);
            self.clear_sicks(false);
            self.reset_stages();
            self.raw[OFS_FLAGS_B..OFS_FLAGS_B + 2].fill(0);
        }
        self.restore_form_and_ability(env);
    }

    /// C: `FUN_overlay_d_93__021d6580`: reset when sent back in by a replacement.
    pub fn reset_on_replace<E: BppEnv>(&mut self, env: &mut E) {
        self.reload_from_party(self.pokemon(), false, true, env);
        self.raw[OFS_FLAGS_B..OFS_FLAGS_B + 2].fill(0);
        self.raw[OFS_FLAGS_A..OFS_FLAGS_A + 2].fill(0);
        self.set_substitute_hp(0);
        self.reset_switch_state();
        self.clear_sicks(false);
        self.reset_stages();
    }

    /// C: `FUN_overlay_d_93__021d65c8`: Baton Pass: copies stages, substitute, passable
    /// conditions and two flags from `src` to `self`, then resets `src`.
    pub fn copy_passable_state<E: BppEnv>(&mut self, src: &mut BattleMon, env: &mut E) {
        self.raw[OFS_STAGES..OFS_STAGES + 7].copy_from_slice(&src.raw[OFS_STAGES..OFS_STAGES + 7]);
        self.set_u16(0x1F2, src.u16_at(0x1F2));
        for s in 0..SICK_COUNT as u32 {
            if src.sick_cont(s).kind() != 0 && env.sick_is_passable(s) {
                self.set_sick_cont(s, src.sick_cont(s));
                self.set_u8(OFS_SICK_TURNS + s as usize, src.sick_turns(s));
            }
        }
        if src.flag_b(0xA) {
            self.set_flag_b(0xA);
            let atk = self.get_unstaged(value::ATTACK, env) as u16;
            let def = self.get_unstaged(value::DEFENSE, env) as u16;
            self.set_stat(value::ATTACK, def, env);
            self.set_stat(value::DEFENSE, atk, env);
        }
        if src.flag_b(9) {
            self.set_flag_b(9);
        }
        src.set_substitute_hp(0);
        src.clear_sicks(false);
        src.reset_stages();
        src.raw[OFS_FLAGS_B..OFS_FLAGS_B + 2].fill(0);
    }

    /// C: `FUN_overlay_d_93__021d66d8`: changes form; Shaymin (492) reverting to form 0 also
    /// resets the stored form bits and reloads its ability.
    pub fn change_form<E: BppEnv>(&mut self, form: u8, env: &mut E) {
        self.set_u8(OFS_FORM, form);
        let pp = self.pokemon();
        let token = env.pp_fast_begin(pp);
        env.pp_set_form(pp, form as u32);
        self.reload_types_and_stats(pp, true, env);
        if self.species() == 0x1EC && form == 0 {
            self.raw[0x1B] &= !0x1F;
            let v = env.pp_get(pp, pp::ABILITY) as u16;
            self.set_u16(0x13C, v);
        }
        env.pp_fast_end(pp, token);
    }

    /// C: `FUN_overlay_d_93__021d6bfc`: reloads stats and moves from the party Pokemon (moves
    /// only while transformed).
    pub fn refresh_from_party<E: BppEnv>(&mut self, env: &mut E) {
        let pp = self.pokemon();
        if !self.is_transformed() {
            self.reload_from_party(pp, true, false, env);
            self.moves_reload(env);
        } else {
            self.moves_load(pp, false, env);
        }
    }

    /// C: `FUN_overlay_d_93__021d6b84`: writes EXP, HP, status, moves, form and item back to
    /// the party Pokemon.
    pub fn store_to_party<E: BppEnv>(&self, keep_form: bool, env: &mut E) {
        let pp = self.pokemon();
        env.pp_put(pp, pp::EXP, self.u32_at(0x08));
        env.pp_put(pp, pp::HP, self.hp() as u32);
        let status = if self.hp() != 0 {
            self.major_status()
        } else {
            0
        };
        env.pp_set_status(pp, status);
        self.moves_store(env);
        let form = if self.is_transformed() || keep_form {
            self.raw[0x1B] & 0x1F
        } else {
            self.u8_at(OFS_FORM)
        };
        env.pp_put(pp, pp::FORM, form as u32);
        env.pp_recalc_stats(pp);
        env.pp_put(pp, pp::HELD_ITEM, self.held_item() as u32);
    }

    /// C: `FUN_overlay_d_93__021d6c3c`: rebinds to another party Pokemon (the species read
    /// is discarded).
    pub fn set_pokemon<E: BppEnv>(&mut self, pp: u32, env: &mut E) {
        env.pp_get(pp, pp::SPECIES);
        self.set_u32(0x00, pp);
    }

    /// C: `FUN_overlay_d_93__021d6e24`
    pub fn set_pp_0x98<E: BppEnv>(&self, value: u32, env: &mut E) {
        let v = match env.fun_0202142c(value) {
            0 => 4,
            v => v,
        };
        env.pp_put(self.pokemon(), 0x98, v);
    }

    /// C: unlabeled routine at `0x021D6EA0`: recalculates the party Pokemon's stats.
    pub fn recalc_party_stats<E: BppEnv>(&self, env: &mut E) {
        env.pp_recalc_stats(self.pokemon());
    }
}

// ---- Transform ------------------------------------------------------------------------------

impl BattleMon {
    /// C: `FUN_overlay_d_93__021d6d54`
    pub fn is_transformed(&self) -> bool {
        self.raw[0x1B] & 0x20 != 0
    }

    /// C: `FUN_overlay_d_93__021d6c50`: Transform into `target`. Copies everything from
    /// +0xEC on (stats, types, stages, moves, ...) except substitute HP; moves get 5 PP and
    /// are unlinked. Fails if either side is transformed or the target has a substitute.
    ///
    /// Deviation: the original stages data through two overlay globals (0x021F3B8C,
    /// 0x021F3BC4); they are not modelled.
    pub fn transform_into<E: BppEnv>(&mut self, target: &BattleMon, env: &mut E) -> bool {
        if self.is_transformed() || target.is_transformed() || target.u16_at(0x1F2) != 0 {
            return false;
        }
        let own_moves: [u8; 0x38] = self.raw[OFS_MOVES..OFS_MOVES + 0x38].try_into().unwrap();
        let head: [u8; 0xEC] = self.raw[..0xEC].try_into().unwrap();
        let sub = self.u16_at(0x1F2);
        self.raw = target.raw;
        self.raw[..0xEC].copy_from_slice(&head);
        self.set_u16(0x1F2, sub);
        self.raw[OFS_MOVES..OFS_MOVES + 0x38].copy_from_slice(&own_moves);
        for i in 0..4 {
            let mv = target.u16_at(Self::slot(i) + ms::CUR_ID) as u32;
            self.move_replace(i, mv, 5, false, env);
        }
        self.raw[OFS_FLAGS_A..OFS_FLAGS_A + 2].fill(0);
        self.raw[OFS_FLAGS_B..OFS_FLAGS_B + 2].fill(0);
        self.set_u16(0x148, 10000);
        self.set_u16(0x146, 0);
        self.set_u16(0x14A, 0);
        self.set_u16(0x14C, 0);
        self.set_u8(0x144, 0x11);
        self.set_u16(0x14E, 0);
        self.raw[0x1B] |= 0x20;
        true
    }
}

// ---- move history and misc fields -----------------------------------------------------------

/// One 8-byte action record (four `u16`s) in the 3-turn history ring at +0x15C.
pub type ActionRecord = [u16; 4];

const OFS_HISTORY: usize = 0x15C;
const OFS_HISTORY_COUNT: usize = 0x1EC;
const OFS_HISTORY_INDEX: usize = 0x1EF;

impl BattleMon {
    /// C: `FUN_overlay_d_93__021d6744`: records the move just used. `consecutive` extends
    /// the same-move streak (+0x14E) when `mv` repeats.
    pub fn record_move(&mut self, target: u8, consecutive: bool, mv_type: u8, mv: u16, orig: u16) {
        let prev = self.u16_at(0x14C);
        self.set_u16(0x14A, orig);
        self.set_u16(0x14C, mv);
        self.set_u8(0x152, target);
        self.set_u8(0x144, mv_type);
        if prev == mv {
            if consecutive {
                self.set_u16(0x14E, self.u16_at(0x14E).wrapping_add(1));
            } else {
                self.set_u16(0x14E, 0);
            }
        } else {
            self.set_u16(0x14E, consecutive as u16);
        }
    }

    /// C: unlabeled routine at `0x021D679C`
    pub fn move_streak(&self) -> u16 {
        self.u16_at(0x14E)
    }
    /// C: `FUN_overlay_d_93__021d67a8`
    pub fn last_move(&self) -> u16 {
        self.u16_at(0x14C)
    }
    /// C: unlabeled routine at `0x021D67B0`
    pub fn last_move_type(&self) -> u8 {
        self.u8_at(0x144)
    }
    /// C: `FUN_overlay_d_93__021d67b8`
    pub fn last_move_orig(&self) -> u16 {
        self.u16_at(0x14A)
    }
    /// C: `FUN_overlay_d_93__021d67c4`
    pub fn last_move_target(&self) -> u8 {
        self.u8_at(0x152)
    }
    /// C: `FUN_overlay_d_93__021d67d0`: bit 7 of +0x1B.
    pub fn flag_1b_bit7(&self) -> bool {
        self.raw[0x1B] & 0x80 != 0
    }
    /// C: `FUN_overlay_d_93__021d67d8`
    pub fn set_weight(&mut self, w: u16) {
        self.set_u16(0x13E, if w < 1 { 1 } else { w });
    }
    /// C: `FUN_overlay_d_93__021d67e8`
    pub fn weight(&self) -> u16 {
        self.u16_at(0x13E)
    }

    /// C: `FUN_overlay_d_93__021d63ec`
    pub fn set_148(&mut self, v: u16) {
        self.set_u16(0x148, v);
        self.set_u16(0x146, 0);
        self.raw[0x1B] |= 0x80;
        self.clear_history();
    }

    /// C: `FUN_overlay_d_93__021d6408`: turn start: clears flag set A, bumps the +0x146
    /// counter (to 9999) and advances the history ring.
    pub fn on_turn_start(&mut self) {
        self.raw[OFS_FLAGS_A..OFS_FLAGS_A + 2].fill(0);
        if self.u16_at(0x146) < 0x270F {
            self.set_u16(0x146, self.u16_at(0x146) + 1);
        }
        self.advance_history();
    }

    /// C: `FUN_overlay_d_93__021d6440`
    pub fn clear_flag_a(&mut self, n: u32) {
        let (i, m) = bit_of(n);
        self.raw[OFS_FLAGS_A + i] &= !m;
    }

    /// C: `FUN_overlay_d_93__021d6730` / `021d6738` / unlabeled `0x021D6740`
    pub fn set_u14(&mut self, v: u16) {
        self.set_u16(0x14, v);
        self.set_u16(0x12, 0);
    }
    pub fn clear_u14(&mut self) {
        self.set_u16(0x14, 0);
    }
    pub fn u14(&self) -> u16 {
        self.u16_at(0x14)
    }

    /// C: `FUN_overlay_d_93__021d67f4`
    pub fn clear_history(&mut self) {
        self.raw[OFS_HISTORY..OFS_HISTORY + 0x90].fill(0);
        self.raw[OFS_HISTORY_COUNT..OFS_HISTORY_COUNT + 3].fill(0);
        self.set_u8(OFS_HISTORY_INDEX, 0);
        self.set_u8(0x1F0, 0);
    }

    /// C: `FUN_overlay_d_93__021d6824`
    pub fn advance_history(&mut self) {
        let mut i = self.u8_at(OFS_HISTORY_INDEX).wrapping_add(1);
        if i >= 3 {
            i = 0;
        }
        self.set_u8(OFS_HISTORY_INDEX, i);
        self.set_u8(OFS_HISTORY_COUNT + i as usize, 0);
    }

    /// C: `FUN_overlay_d_93__021d6848`: appends to this turn's history (6 records, oldest
    /// dropped).
    pub fn push_history(&mut self, rec: &ActionRecord) {
        let idx = self.u8_at(OFS_HISTORY_INDEX) as usize;
        let base = OFS_HISTORY + idx * 0x30;
        let mut cnt = self.u8_at(OFS_HISTORY_COUNT + idx);
        if cnt == 6 {
            self.raw.copy_within(base + 8..base + 0x30, base);
            cnt -= 1;
        }
        let o = base + cnt as usize * 8;
        for (k, &v) in rec.iter().enumerate() {
            self.set_u16(o + 2 * k, v);
        }
        let c = self.u8_at(OFS_HISTORY_COUNT + idx);
        if c < 6 {
            self.set_u8(OFS_HISTORY_COUNT + idx, c + 1);
        }
    }

    fn history_slot(&self, turns_ago: u32) -> usize {
        let mut i = self.u8_at(OFS_HISTORY_INDEX) as i32 - turns_ago as i32;
        if i < 0 {
            i += 3;
        }
        i as usize
    }

    /// C: `FUN_overlay_d_93__021d6920`: records `turns_ago` (0..3) turns back.
    pub fn history_count(&self, turns_ago: u32) -> u8 {
        if turns_ago >= 3 {
            return 0;
        }
        self.u8_at(OFS_HISTORY_COUNT + self.history_slot(turns_ago))
    }

    /// C: `FUN_overlay_d_93__021d6940`: the `n`th most recent record of a turn.
    pub fn history_record(&self, turns_ago: u32, n: u32) -> Option<ActionRecord> {
        if turns_ago >= 3 {
            return None;
        }
        let i = self.history_slot(turns_ago);
        let cnt = self.u8_at(OFS_HISTORY_COUNT + i) as u32;
        if n >= cnt {
            return None;
        }
        let k = (cnt - n - 1) as u8 as usize;
        let o = OFS_HISTORY + i * 0x30 + k * 8;
        Some([
            self.u16_at(o),
            self.u16_at(o + 2),
            self.u16_at(o + 4),
            self.u16_at(o + 6),
        ])
    }

    /// C: `FUN_overlay_d_93__021d69a0` / `021d69ac`
    pub fn set_u8_157(&mut self, i: u32, v: u8) {
        self.set_u8(0x157 + i as usize, v);
    }
    pub fn u8_157(&self, i: u32) -> u8 {
        self.u8_at(0x157 + i as usize)
    }

    /// C: `FUN_overlay_d_93__021d6e40`
    pub fn set_1f6(&mut self, a: u8, b: u16) {
        self.set_u8(0x1F6, a);
        self.set_u16(0x1F4, b);
    }
    /// C: unlabeled routine at `0x021D6E50`: `(a, b)` unless +0x1F6 is 0x1F.
    pub fn get_1f6(&self) -> Option<(u8, u16)> {
        let a = self.u8_at(0x1F6);
        if a == 0x1F {
            None
        } else {
            Some((a, self.u16_at(0x1F4)))
        }
    }
    /// C: `FUN_overlay_d_93__021d6e74`
    pub fn has_1f6(&self) -> bool {
        self.u8_at(0x1F6) != 0x1F
    }
    /// C: `FUN_overlay_d_93__021d6e88`
    pub fn clear_1f6(&mut self) {
        if self.u8_at(0x1F6) != 0x1F {
            self.set_u8(0x1F6, 0x1F);
            self.set_u16(0x1F4, 0);
        }
    }
}

// ---- substitute and opponents faced ---------------------------------------------------------

impl BattleMon {
    /// C: `FUN_overlay_d_93__021d6d5c`: sets substitute HP and ends condition 8.
    pub fn make_substitute(&mut self, hp: u16) {
        self.set_u16(0x1F2, hp);
        self.cure_sick(8);
    }
    /// C: `FUN_overlay_d_93__021d6d70` (and a write helper)
    pub fn set_substitute_hp(&mut self, hp: u16) {
        self.set_u16(0x1F2, hp);
    }
    /// C: `FUN_overlay_d_93__021d6d7c`
    pub fn has_substitute(&self) -> bool {
        self.u16_at(0x1F2) != 0
    }
    /// C: `FUN_overlay_d_93__021d6d90`
    pub fn substitute_hp(&self) -> u16 {
        self.u16_at(0x1F2)
    }
    /// C: `FUN_overlay_d_93__021d6d9c`: the substitute absorbs `damage`; returns whether it
    /// broke (then `damage` is reduced to what it absorbed).
    pub fn hit_substitute(&mut self, damage: &mut u16) -> bool {
        let sub = self.u16_at(0x1F2);
        if sub <= *damage {
            *damage = sub;
            self.set_u16(0x1F2, 0);
            true
        } else {
            self.set_u16(0x1F2, sub - *damage);
            false
        }
    }

    /// C: `FUN_overlay_d_93__021d6dc4`
    pub fn clear_faced(&mut self) {
        self.set_u8(0xD0, 0);
    }
    /// C: `FUN_overlay_d_93__021d6dcc`: remembers an opponent (up to 24).
    pub fn add_faced(&mut self, poke_id: u8) {
        let n = self.u8_at(0xD0) as usize;
        let mut i = 0;
        while i < n {
            if self.u8_at(0xD1 + i) == poke_id {
                return;
            }
            i += 1;
        }
        if i < 0x18 {
            self.set_u8(0xD1 + i, poke_id);
            self.raw[0xD0] = self.raw[0xD0].wrapping_add(1);
        }
    }
    /// C: `FUN_overlay_d_93__021d6e04`
    pub fn faced_count(&self) -> u8 {
        self.u8_at(0xD0)
    }
    /// C: `FUN_overlay_d_93__021d6e0c`
    pub fn faced(&self, i: u32) -> u8 {
        if i < self.u8_at(0xD0) as u32 {
            self.u8_at(0xD1 + i as usize)
        } else {
            0x1F
        }
    }
}

// ---- EXP ------------------------------------------------------------------------------------

/// Level-up summary written by [`BattleMon::gain_exp`] (C: 14-byte struct).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LevelUpInfo {
    pub level: u8,
    pub hp: u16,
    pub attack: u16,
    pub defense: u16,
    pub sp_attack: u16,
    pub sp_defense: u16,
    pub speed: u16,
}

impl LevelUpInfo {
    pub fn to_bytes(&self) -> [u8; 14] {
        let mut b = [0u8; 14];
        b[0] = self.level;
        for (i, v) in [
            self.hp,
            self.attack,
            self.defense,
            self.sp_attack,
            self.sp_defense,
            self.speed,
        ]
        .iter()
        .enumerate()
        {
            b[2 + 2 * i..4 + 2 * i].copy_from_slice(&v.to_le_bytes());
        }
        b
    }
}

impl BattleMon {
    /// C: `FUN_overlay_d_93__021d69b8`: adds EXP. If it reaches the next level, applies one
    /// level (stat deltas in `info`), subtracts the EXP used from `exp`, and returns true;
    /// otherwise stores the EXP, zeroes `info`, and returns false.
    pub fn gain_exp<E: BppEnv>(
        &mut self,
        exp: &mut u32,
        info: Option<&mut LevelUpInfo>,
        env: &mut E,
    ) -> bool {
        let pp = self.pokemon();
        if self.level() < 100 {
            let cur = self.u32_at(0x08);
            let total = cur.wrapping_add(*exp);
            let next = env.exp_for_level(
                self.species() as u32,
                self.u8_at(OFS_FORM) as u32,
                (self.level() as u32 + 1) & 0xFFFF,
            );
            if total >= next {
                let old_max = self.max_hp();
                let gain = next.wrapping_sub(cur);
                let token = env.pp_fast_begin(pp) as u8 as u32;
                let a = env.pp_get(pp, pp::ATTACK) as u16;
                let d = env.pp_get(pp, pp::DEFENSE) as u16;
                let spe = env.pp_get(pp, pp::SPEED) as u16;
                let spa = env.pp_get(pp, pp::SP_ATTACK) as u16;
                let spd = env.pp_get(pp, pp::SP_DEFENSE) as u16;
                self.set_u32(0x08, cur.wrapping_add(gain));
                env.pp_put(pp, pp::EXP, self.u32_at(0x08));
                env.pp_recalc_stats(pp);
                let v = env.pp_get(pp, pp::MAX_HP) as u16;
                self.set_u16(0x0E, v);
                let v = env.pp_get(pp, pp::LEVEL) as u8;
                self.set_u8(0x18, v);
                let hp_gain = self.max_hp().wrapping_sub(old_max);
                let mut out = LevelUpInfo {
                    attack: env.pp_get(pp, pp::ATTACK) as u16,
                    defense: env.pp_get(pp, pp::DEFENSE) as u16,
                    sp_attack: env.pp_get(pp, pp::SP_ATTACK) as u16,
                    sp_defense: env.pp_get(pp, pp::SP_DEFENSE) as u16,
                    speed: env.pp_get(pp, pp::SPEED) as u16,
                    ..Default::default()
                };
                if !self.is_transformed() {
                    self.set_u16(0xEE, out.attack);
                    self.set_u16(0xF0, out.defense);
                    self.set_u16(0xF2, out.sp_attack);
                    self.set_u16(0xF4, out.sp_defense);
                    self.set_u16(0xF6, out.speed);
                }
                out.level = self.level();
                out.hp = hp_gain;
                out.attack = out.attack.wrapping_sub(a);
                out.defense = out.defense.wrapping_sub(d);
                out.sp_attack = out.sp_attack.wrapping_sub(spa);
                out.sp_defense = out.sp_defense.wrapping_sub(spd);
                out.speed = out.speed.wrapping_sub(spe);
                self.set_u16(0x10, self.hp().wrapping_add(hp_gain));
                env.pp_put(pp, pp::HP, self.hp() as u32);
                self.moves_store(env);
                env.pp_fast_end(pp, token);
                *exp = exp.wrapping_sub(gain);
                if let Some(i) = info {
                    *i = out;
                }
                return true;
            }
            self.set_u32(0x08, total);
            env.pp_put(pp, pp::EXP, total);
        }
        if let Some(i) = info {
            *i = LevelUpInfo::default();
        }
        false
    }

    /// C: `FUN_overlay_d_93__021d6b6c`: EXP needed for level 100.
    pub fn exp_for_max_level<E: BppEnv>(&self, env: &mut E) -> u32 {
        env.exp_for_level(self.species() as u32, self.u8_at(OFS_FORM) as u32, 100)
    }
}

// ---- construction ---------------------------------------------------------------------------

impl BattleMon {
    /// C: `FUN_overlay_d_93__021d4d84`: builds a battle Pokemon from party Pokemon `pp`. The
    /// battle RNG (`calc`) rolls sleep turns for an already-asleep Pokemon.
    ///
    /// Deviation: the original allocates from the heap without clearing it; fields it never
    /// writes (e.g. +0xD1.., +0x14F..+0x15B, +0x1F1, +0x1F7) start as 0 here.
    pub fn new<E: BppEnv>(
        pp: u32,
        poke_id: u8,
        calc: &mut crate::calc::BattleCalc,
        env: &mut E,
    ) -> Self {
        let mut b = Self::new_prefix(pp, poke_id, env);
        let status = env.pp_status(pp);
        if status != 0 {
            let c = calc.make_default_sick_cont(status);
            b.set_sick_cont(status, c);
            b.set_u8(OFS_SICK_TURNS + status as usize, 0);
        }
        b.finish_new();
        b
    }

    fn new_prefix<E: BppEnv>(pp: u32, poke_id: u8, env: &mut E) -> Self {
        let mut b = BattleMon::default();
        b.set_u8(0x19, poke_id);
        let v = env.pp_get(pp, pp::SPECIES) as u16;
        b.set_u16(0x0C, v);
        b.set_u32(0x00, pp);
        let v = env.pp_get(pp, pp::HP) as u16;
        b.set_u16(0x10, v);
        let v = env.pp_get(pp, pp::MAX_HP) as u16;
        b.set_u16(0x0E, v);
        let v = env.pp_get(pp, pp::HELD_ITEM) as u16;
        b.set_u16(0x12, v);
        if b.u16_at(0x12) > 0x272 {
            b.set_u16(0x12, 0);
        }
        b.set_u16(0x14, 0);
        b.raw[0x1B] &= !0xE0;
        b.set_u32(0x04, 0);
        let form = env.pp_get(pp, pp::FORM) as u8;
        b.raw[0x1B] = (b.raw[0x1B] & !0x1F) | (form & 0x1F);
        let v = env.pp_get(pp, pp::ABILITY) as u16;
        b.set_u16(0x16, v);
        let v = env.pp_get(pp, pp::LEVEL) as u8;
        b.set_u8(0x18, v);
        let v = env.personal(b.species() as u32, (b.raw[0x1B] & 0x1F) as u32, 1) as u8;
        b.set_u8(0x1A, v);
        b.reload_from_party(pp, true, true, env);
        let n = b.moves_load(pp, true, env) as u8;
        b.set_u8(OFS_MOVE_COUNT, n);
        b.set_u8(OFS_MOVES_USED, 0);
        b.reset_stages();
        b.clear_sicks(true);
        b
    }

    fn finish_new(&mut self) {
        self.set_u16(0x148, 10000);
        self.set_u16(0x146, 0);
        self.set_u16(0x1F2, 0);
        self.set_u16(0x14A, 0);
        self.set_u16(0x14C, 0);
        self.set_u8(0x144, 0x11);
        self.set_u16(0x14E, 0);
        self.set_u16(0x1F4, 0);
        self.set_u8(0x1F6, 0x1F);
        self.set_u8(OFS_CRIT_BOOST, 0);
        self.raw[OFS_FLAGS_A..OFS_FLAGS_A + 2].fill(0);
        self.raw[OFS_FLAGS_B..OFS_FLAGS_B + 2].fill(0);
        self.clear_history();
        self.clear_faced();
    }
}
