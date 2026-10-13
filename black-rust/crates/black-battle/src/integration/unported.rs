//! [`Unported`] for the production `FlowWorkHost`: the methods with no port in this crate. Move
//! data comes from the ROM move records through the `black-pokemon::waza` getters (`FUN_0201BD28`
//! = `record_param`, `FUN_0201BEB8` = `flag_test`); the overlay-95 tables are models or gaps.

use super::data::Unp;
use crate::server_flow::host::Unported;
use black_pokemon::waza;

fn gap(what: &str) -> ! {
    panic!("B16 gap: {what} (FlowHost, no port) is not modelled")
}

impl Unp {
    /// `FUN_0201BD28(move, param)`.
    pub fn move_param(&self, mv: u32, param: u32) -> u32 {
        waza::record_param(&self.arc.move_record(mv), param)
    }
    /// `FUN_0201BEB8(move, flag)`.
    pub fn move_flag(&self, mv: u32, flag: u32) -> bool {
        waza::flag_test(&self.arc.move_record(mv), flag) != 0
    }
}

impl Unported for Unp {
    fn move_category(&mut self, mv: u32) -> u32 {
        self.move_param(mv, 2)
    }
    fn move_param(&mut self, mv: u32, param: u32) -> u32 {
        Unp::move_param(self, mv, param)
    }
    fn move_flag(&mut self, mv: u32, flag: u32) -> bool {
        Unp::move_flag(self, mv, flag)
    }
    fn move_power(&mut self, mv: u32) -> u32 {
        self.move_param(mv, 3) & 0xFFFF
    }
    fn move_effect_class(&mut self, mv: u32) -> u32 {
        self.move_param(mv, 1)
    }
    fn move_fun_0201c314(&mut self, mv: u32) -> bool {
        self.move_param(mv, 3) != 0
    }
    fn move_type(&mut self, mv: u32) -> u32 {
        self.move_param(mv, 0) & 0xFF
    }
    /// `FUN_0201C020(move)`: the number of stat changes (nonzero stat bytes at +0x15..+0x17).
    fn move_fun_0201c020(&mut self, mv: u32) -> u32 {
        let r = self.arc.move_record(mv);
        (0..3).filter(|&i| r[0x15 + i] != 0).count() as u32
    }
    /// `FUN_0201C0BC(move, i, &amount)` amount part: `(s8) +0x18 + i` (0 without a stat).
    fn move_fun_0201c0bc(&mut self, mv: u32, index: u32) -> i32 {
        let r = self.arc.move_record(mv);
        let i = index as usize;
        if r[0x15 + i] != 0 {
            r[0x18 + i] as i8 as i32
        } else {
            0
        }
    }
    fn item_fun_020213c0(&mut self, _item: u32) -> bool {
        gap("FUN_020213C0 (berry test)")
    }
    fn item_param(&mut self, _item: u32, _param: u32) -> u32 {
        gap("item_param (FUN_02020FE4)")
    }
    fn fun_020189d0(&mut self, _pp: u32, _x: u32) -> i32 {
        gap("FUN_020189D0")
    }
    fn ability_breakable(&mut self, _ability: u32) -> bool {
        gap("0689C9B0 (ability breakable)")
    }
    fn move_unredirectable(&mut self, _mv: u32) -> bool {
        gap("0689C8EC")
    }
    fn move_fun_0689c8dc(&mut self, _mv: u32) -> bool {
        gap("0689C8DC")
    }
    fn move_fun_0689c8cc(&mut self, _mv: u32) -> bool {
        gap("0689C8CC")
    }
    fn item_fun_0689c980(&mut self, _item: u32) -> bool {
        gap("0689C980")
    }
    fn item_fun_0689c990(&mut self, _item: u32) -> bool {
        gap("0689C990")
    }
    fn sick_list_entry(&mut self, _index: u32) -> u32 {
        gap("0689C9E4 (sick list)")
    }
    fn fun_06898cf4(&mut self, _side: u32, _kind: u32) -> bool {
        gap("06898CF4")
    }
    fn fun_0689c8ac(&mut self, _mv: u32) -> bool {
        gap("0689C8AC")
    }
    fn fun_0689c8bc(&mut self, _mv: u32) -> bool {
        gap("0689C8BC")
    }
    fn fun_0689c91c(&mut self, _ability: u32) -> bool {
        gap("0689C91C")
    }
    fn fun_0689ca1c(&mut self, _mv: u32) -> bool {
        gap("0689CA1C")
    }
    fn fun_0689ca3c(&mut self, _mv: u32) -> bool {
        gap("0689CA3C")
    }
    fn fun_0689ca5c(&mut self, _mv: u32) -> bool {
        gap("0689CA5C")
    }
    fn fun_0689ca7c(&mut self, _mv: u32) -> bool {
        gap("0689CA7C")
    }
    fn fun_0689ca8c(&mut self) -> Vec<u16> {
        gap("0689CA8C")
    }
    fn fun_0689d064(&mut self, _mv: u32, _index: u32) -> bool {
        gap("0689D064")
    }
    fn fun_0689d098(&mut self, _mv: u32, _turn: u32) -> u32 {
        gap("0689D098")
    }
    fn fun_0689d0c8(&mut self, _turn: u32) -> u32 {
        gap("0689D0C8")
    }
    fn fun_0689d174(&mut self, _side: u32) -> u32 {
        gap("0689D174")
    }
    fn fun_0689d184(&mut self, _side: u32, _index: u32) -> u32 {
        gap("0689D184")
    }
    fn fun_021c7d34(&mut self, _poke_id: u32) -> u32 {
        gap("021C7D34 (ov95 tail call)")
    }
    fn fun_021c7d44(&mut self) -> u32 {
        gap("021C7D44 (ov95 tail call)")
    }
    fn fun_021c814c(&mut self, _poke_id: u32) -> bool {
        gap("021C814C (ov95 tail call)")
    }
    fn fun_021c82c4(&mut self, _side: u32, _kind: u32) -> bool {
        gap("021C82C4 (ov95 tail call)")
    }
    fn msg_param(&mut self) -> &mut [u8] {
        &mut self.msg
    }
    fn store_u8(&mut self, addr: u32, v: u8) {
        self.objs.borrow_mut().w8(addr, v as u32)
    }
}
