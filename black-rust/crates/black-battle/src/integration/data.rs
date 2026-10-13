//! Game data (personal records, moves, learnsets, growth) from the ROM file system, and [`Unp`]:
//! the integration's answer to the `BppEnv` / party-Pokemon calls and the `Unported` methods of
//! the production `FlowWorkHost`. Party Pokemon (`POKEMON_PARAM`, 0xDC bytes, encrypted) live in
//! the object table and are read / written with the verified `black-pokemon::poke_tool` ports.

use super::objects::Objects;
use super::rom::Rom;
use crate::battle_mon::BppEnv;
use crate::field::Field;
use black_pokemon::hooks::{GrowthHooks, PersonalHooks, PokeToolHooks};
use black_pokemon::poke_tool;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

/// Raw ROM archives.
pub struct Archives {
    pub personal: nds_rom::narc::Narc,
    pub learnset: nds_rom::narc::Narc,
    pub waza: nds_rom::narc::Narc,
    pub growth: nds_rom::narc::Narc,
    pub items: nds_rom::narc::Narc,
}

impl Archives {
    pub fn load(rom: &Rom) -> Self {
        let rd = |p: &str| {
            let b = std::fs::read(rom.files.join(p)).unwrap_or_else(|e| panic!("{p}: {e}"));
            nds_rom::narc::Narc::parse(&b).unwrap()
        };
        Archives {
            personal: rd("a/0/1/6"),
            learnset: rd("a/0/1/8"),
            waza: rd("a/0/2/1"),
            growth: rd("a/0/1/7"),
            items: rd("a/0/2/4"),
        }
    }

    /// The 0x3C-byte personal record of `species` / `form` (`FUN_0201AFD4`: alternate forms from
    /// the halfword at 0x1C, count at 0x20).
    pub fn personal_record(&self, species: u32, form: u32) -> [u8; 0x3C] {
        let mut out = [0u8; 0x3C];
        if (species.wrapping_add(0xFD76) & 0xFFFF) < 2 {
            let r = self.personal.file(0);
            out[..r.len().min(0x3C)].copy_from_slice(&r[..r.len().min(0x3C)]);
            return out;
        }
        let base = self.personal.file(species as usize);
        let first = u16::from_le_bytes([base[0x1C], base[0x1D]]) as u32;
        let count = base[0x20] as i32;
        let idx = if first == 0 || count <= form as i32 || form == 0 {
            species
        } else {
            (first + form - 1) & 0xFFFF
        };
        let r = self.personal.file(idx as usize);
        out[..r.len().min(0x3C)].copy_from_slice(&r[..r.len().min(0x3C)]);
        out
    }

    pub fn personal_param(&self, species: u32, form: u32, param: u32) -> u32 {
        black_pokemon::personal::field(&self.personal_record(species, form), param)
    }

    /// The 36-byte move record.
    pub fn move_record(&self, mv: u32) -> Vec<u8> {
        self.waza.file(mv as usize).to_vec()
    }

    /// EXP table of a growth curve (101 `u32`).
    pub fn growth_curve(&self, curve: u32) -> [u32; 101] {
        let b = self.growth.file(curve as usize);
        let mut t = [0u32; 101];
        for (i, t) in t.iter_mut().enumerate() {
            *t = u32::from_le_bytes(b[4 * i..4 * i + 4].try_into().unwrap());
        }
        t
    }

    pub fn exp_for_level(&self, species: u32, form: u32, level: u32) -> u32 {
        let curve = self.personal_param(species, form, 0x17);
        self.growth_curve(curve)[level as usize]
    }

    pub fn level_from_exp(&self, species: u32, form: u32, exp: u32) -> u32 {
        let t = self.growth_curve(self.personal_param(species, form, 0x17) & 0xFFFF);
        let mut level = 1usize;
        while level < 101 && exp >= t[level] {
            level += 1;
        }
        level as u32 - 1
    }

    /// Learnset of `species` (form 0): `(move, level)`.
    pub fn learnset(&self, species: u32) -> Vec<(u32, u32)> {
        let b = self.learnset.file(species as usize);
        let mut v = Vec::new();
        for c in b.chunks(4) {
            let mv = u16::from_le_bytes([c[0], c[1]]) as u32;
            let lv = u16::from_le_bytes([c[2], c[3]]) as u32;
            if mv == 0xFFFF {
                break;
            }
            v.push((mv, lv));
        }
        v
    }

    /// `FUN_0201C158(move, pp_ups)`: max PP with PP Ups (`pp + pp * 20% * ups`).
    pub fn max_pp(&self, mv: u32, ups: u32) -> u32 {
        let pp = self.move_record(mv)[5] as u32;
        let ups = ups.min(3);
        pp + pp * 20 * ups / 100
    }
}

/// Handles to the shared state for the code paths that cannot borrow [`super::Env`] (the
/// `BppEnv` of battle Pokemon, the `Unported` methods of the flow host).
#[derive(Clone)]
pub struct Unp {
    /// Stand-in string-parameter buffer (`Unported::msg_param`).
    pub msg: Vec<u8>,
    pub objs: Rc<RefCell<Objects>>,
    pub fields: Rc<RefCell<BTreeMap<u32, Field>>>,
    pub arc: Rc<Archives>,
}

impl Unp {
    /// Runs `f` on the decoded-access image of the party Pokemon at `pp`.
    fn with_pp<R>(&mut self, pp: u32, f: impl FnOnce(&mut [u8], &mut PtHooks) -> R) -> R {
        let mut b = vec![0u8; poke_tool::PARTY_MON_SIZE];
        self.objs.borrow_mut().read(pp, &mut b);
        let mut h = PtHooks {
            arc: self.arc.clone(),
        };
        let r = f(&mut b, &mut h);
        self.objs.borrow_mut().write(pp, &b);
        r
    }
}

/// `PokeToolHooks` / `PersonalHooks` / `GrowthHooks` over the archives.
pub struct PtHooks {
    pub arc: Rc<Archives>,
}

fn gap(what: &str) -> ! {
    panic!("B16 gap: {what} is not modelled")
}

impl PokeToolHooks for PtHooks {
    /// `FUN_02017F50`: gender from the ratio and the PID's low byte.
    fn gender(&mut self, species: u32, form: u32, pid: u32) -> u32 {
        match self.arc.personal_param(species, form, 0x14) {
            0 => 0,
            254 => 1,
            255 => 2,
            r => u32::from((pid & 0xFF) < r),
        }
    }
    fn level_from_exp(&mut self, species: u32, form: u32, exp: u32) -> u32 {
        self.arc.level_from_exp(species, form, exp)
    }
    fn move_max_pp(&mut self, move_id: u32, pp_ups: u32) -> u32 {
        self.arc.max_pp(move_id, pp_ups)
    }
    fn personal_param(&mut self, species: u32, form: u32, param: u32) -> u32 {
        self.arc.personal_param(species, form, param)
    }
    fn strbuf_from_mon(&mut self, _strbuf: u32, _mon_ofs: usize) {}
    fn strbuf_to_mon(&mut self, _strbuf: u32, _mon_ofs: usize, _len: u32) {}
    fn u16str_from_mon(&mut self, _mon_ofs: usize, _dst: u32, _len: u32) {}
    fn u16str_to_mon(&mut self, _src: u32, _mon_ofs: usize, _len: u32) {}
    fn bad_egg_name_strbuf(&mut self, _strbuf: u32) {}
    fn bad_egg_name_u16(&mut self, _dst: u32, _len: u32) {}
    fn species_name_to_scratch(&mut self, _species: u32) {}
    fn scratch_equals_mon_string(&mut self, _mon_ofs: usize) -> u32 {
        0
    }
    fn copy_ext_out(&mut self, _mon_ofs: usize, _dst: u32) {
        gap("POKEMON_PARAM extension copy (get 0xA7)")
    }
    fn copy_ext_in(&mut self, _src: u32, _mon_ofs: usize) {}
}

impl PersonalHooks for PtHooks {
    fn arc_member_offset(&mut self, _handle: u32, member: u32) -> u32 {
        member
    }
    fn arc_seek(&mut self, _handle: u32, _offset: u32) {
        gap("personal archive seek")
    }
    fn arc_read(&mut self, _handle: u32, _size: u32, _dst: &mut [u8]) {
        gap("personal archive read")
    }
    fn open_archive(&mut self, _id: u32, _arg: u32) -> u32 {
        1
    }
    fn load_record(&mut self, species: u32, form: u32, dst: &mut [u8]) {
        dst[..0x3C].copy_from_slice(&self.arc.personal_record(species, form));
    }
    fn heap_alloc(&mut self, _heap: u32, _size: u32, _align: u32, _line: u32) -> u32 {
        gap("personal record allocation")
    }
}

impl GrowthHooks for PtHooks {
    fn personal_record(&mut self, species: u32, form: u32) -> u32 {
        species | form << 16
    }
    fn personal_field(&mut self, rec: u32, field: u32) -> u32 {
        self.arc.personal_param(rec & 0xFFFF, rec >> 16, field)
    }
    fn personal_param(&mut self, species: u32, form: u32, field: u32) -> u32 {
        self.arc.personal_param(species, form, field)
    }
    fn load_growth_curve(&mut self, curve: u32, dst: &mut [u32; 101]) {
        *dst = self.arc.growth_curve(curve);
    }
}

impl BppEnv for Unp {
    fn pp_get(&mut self, pp: u32, id: u32) -> u32 {
        self.with_pp(pp, |m, h| poke_tool::party_get_value(m, id, 0, h))
    }
    fn pp_put(&mut self, pp: u32, id: u32, value: u32) {
        self.with_pp(pp, |m, h| poke_tool::party_set_value(m, id, value, h))
    }
    /// `FUN_02017D30`: decrypt for fast access; the token is whether it did.
    fn pp_fast_begin(&mut self, pp: u32) -> u32 {
        self.with_pp(pp, |m, _| black_pokemon::create::decrypt_party(m) as u32)
    }
    /// `FUN_02017D70(pp, token)`: re-encrypt when the token says it was decrypted.
    fn pp_fast_end(&mut self, pp: u32, token: u32) {
        self.with_pp(pp, |m, _| {
            if token != 0 {
                poke_tool::party_exit(m);
            }
        })
    }
    /// `FUN_020181F4(pp, move, slot)`: move id 0x36 + slot, PP-ups 0x3E + slot = 0, PP 0x3A + slot =
    /// the move's max PP (`FUN_0201C158(move, 0)`; 0 without a move).
    fn pp_set_move(&mut self, pp: u32, mv: u32, slot: u32) {
        let max = if mv != 0 { self.arc.max_pp(mv, 0) } else { 0 };
        self.with_pp(pp, |m, h| {
            poke_tool::party_set_value(m, 0x36 + slot, mv, h);
            poke_tool::party_set_value(m, 0x3E + slot, 0, h);
            poke_tool::party_set_value(m, 0x3A + slot, max, h);
        })
    }
    /// `FUN_02017F00` (-> 02017F08): the gender from species, form and PID (`02017F50`).
    fn pp_fun_02017f00(&mut self, pp: u32) -> u32 {
        self.with_pp(pp, |m, h| {
            let sp = poke_tool::box_get_value(m, 5, 0, h) & 0xFFFF;
            let form = poke_tool::box_get_value(m, 0x6F, 0, h) & 0xFFFF;
            let pid = poke_tool::box_get_value(m, 0, 0, h);
            h.gender(sp, form, pid)
        })
    }
    /// `FUN_02017F90`: party get 0x9D (the status word).
    fn pp_status(&mut self, pp: u32) -> u32 {
        self.pp_get(pp, 0x9D)
    }
    fn move_max_pp(&mut self, mv: u32, pp_ups: u32) -> u32 {
        self.arc.max_pp(mv, pp_ups)
    }
    fn personal(&mut self, species: u32, form: u32, param: u32) -> u32 {
        self.arc.personal_param(species, form, param)
    }
    fn field_effect_active(&mut self, id: u32) -> bool {
        self.fields.borrow()[&super::field::FIELD_GLOBAL].is_active(id as usize) != 0
    }
    fn exp_for_level(&mut self, species: u32, form: u32, level: u32) -> u32 {
        self.arc.exp_for_level(species, form, level)
    }
    /// `FUN_020185C4(pp)`: enter, the level from the EXP (id 0x9E), the stats (`FUN_02018A7C`,
    /// `black_pokemon::personal::recalc_stats`), leave (`black_pokemon::create::recalc` with its
    /// `recalc_stats` hook wired).
    fn pp_recalc_stats(&mut self, pp: u32) {
        self.with_pp(pp, |m, h| {
            let entered = black_pokemon::create::decrypt_party(m);
            let level = black_pokemon::create::level_of(m, h);
            poke_tool::party_set_value(m, 0x9E, level, h);
            let mut cache = black_pokemon::personal::RecordCache {
                form: 0xFFFF,
                species: 0xFFFF,
                record: [0; 0x3C],
            };
            black_pokemon::personal::recalc_stats(m, &mut cache, h);
            black_pokemon::create::encrypt_party(m, entered as u32);
        })
    }
    /// `FUN_02017F9C`: party put 0x9D.
    fn pp_set_status(&mut self, pp: u32, status: u32) {
        self.pp_put(pp, 0x9D, status)
    }
    fn pp_set_form(&mut self, _pp: u32, _form: u32) {
        gap("FUN_02017988 (set form)")
    }
    fn fun_0202142c(&mut self, _value: u32) -> u32 {
        gap("FUN_0202142C")
    }
    fn sick_is_passable(&mut self, _sick: u32) -> bool {
        gap("0689C868 (sick passable)")
    }
}
