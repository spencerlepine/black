//! F8e: overlay-21 `scrcmd_pokemon_fld.c`, Thumb code 0x021B66B8-0x021B7400 (F1-SPLIT row 21.51): party / box Pokemon
//! commands, calendar names, record tables, the Dream World delivery table (cmd 494; X2 scope) and ARC 208 lookups (14
//! command ids, 48 functions).
//!
//! Every function of the area is specced in `tools/rom-diff/specs/black-script/scrcmd_pokemon_fld.spec`; all but the command
//! 494 handler (0x021B6BDC, the Dream World delivery of `pdw_postman.c`, X2 / D9: specced, not ported, not dispatched) are
//! ported here, listed in [`PORTED`] and checked with `tools/check-fn.sh black-script scrcmd_pokemon_fld`. The delivery-row
//! handlers of the ROM table [`PDW_ROWS`] (0x021D9970) that command 494 calls are logic and ported (reachable only through
//! command 494). [`COMMANDS`] maps the other 13 command ids to their handlers and [`dispatch`] runs one by address.
//!
//! Callees outside the area are [`PokemonFldHooks`] methods named by address (vm.c, the operand readers, the ctx getters and
//! the child-event call come from [`CoreHooks`]). PokeParam ids (`FUN_02017E1C(mon, id, x)`): 5 species, 6 held item, 7 OT
//! id, 0x4C egg flag, 0xA7 mail, 0xAB form species (UNVERIFIED names).

use crate::scrcmd_core::CoreHooks;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`). Not 0x021B6BDC (command 494,
/// Dream World, X2).
pub const PORTED: &[u32] = &[
    0x021B_66B8,
    0x021B_6708,
    0x021B_6850,
    0x021B_6904,
    0x021B_6934,
    0x021B_69B0,
    0x021B_69D8,
    0x021B_6A78,
    0x021B_6AA4,
    0x021B_6AAC,
    0x021B_6AB0,
    0x021B_6AE0,
    0x021B_6AFC,
    0x021B_6B6C,
    0x021B_6D0C,
    0x021B_6D30,
    0x021B_6D64,
    0x021B_6D98,
    0x021B_6DC0,
    0x021B_6DE4,
    0x021B_6E0C,
    0x021B_6E24,
    0x021B_6E68,
    0x021B_6EB0,
    0x021B_6EE0,
    0x021B_6F00,
    0x021B_6F18,
    0x021B_6F5C,
    0x021B_6F88,
    0x021B_6FE8,
    0x021B_6FEC,
    0x021B_6FF0,
    0x021B_7020,
    0x021B_7038,
    0x021B_7068,
    0x021B_707C,
    0x021B_7080,
    0x021B_7098,
    0x021B_709C,
    0x021B_70D4,
    0x021B_70D8,
    0x021B_716C,
    0x021B_721C,
    0x021B_72CC,
    0x021B_7364,
    0x021B_7390,
    0x021B_73B8,
];

/// Command id -> handler address (Thumb entry, bit 0 clear) of the ported commands of this area (the words of the command
/// table 0x0217062C at index id; checked against the ROM by the tests). Command 494 (0x021B6BDC) is left out (X2).
pub const COMMANDS: &[(u16, u32)] = &[
    (308, 0x021B_69B0),
    (459, 0x021B_6AE0),
    (460, 0x021B_6AFC),
    (461, 0x021B_6B6C),
    (470, 0x021B_6A78),
    (471, 0x021B_69D8),
    (526, 0x021B_716C),
    (527, 0x021B_70D8),
    (564, 0x021B_7364),
    (565, 0x021B_73B8),
    (581, 0x021B_66B8),
    (585, 0x021B_721C),
    (586, 0x021B_72CC),
];

/// Command 494 (Dream World delivery, `pdw_postman.c`; X2 / D9): specced only, not ported, not dispatched.
pub const CMD_494_DREAM_WORLD: u32 = 0x021B_6BDC;
/// `__FILE__` string "scrcmd_pokemon_fld.c" passed to `Heap_AllocDebug`.
pub const FILE: u32 = 0x021D_E270;
/// The species command 581 looks for (Reshiram / Zekrom slot, UNVERIFIED which).
pub const SPECIES_581: u32 = 0x283;
/// The form species whose form the box swap reverts (`FUN_02017988(mon, 0)`).
pub const FORM_SPECIES_1EC: u32 = 0x1EC;
/// `FUN_020173EC()` (black-pokemon `create`, svflow `ALLOW`): the party-mon size the box swap copies.
pub const PARTY_MON_SIZE: u32 = 0xDC;
/// Boxes and slots per box walked by [`find_in_box`].
pub const BOXES: i32 = 24;
pub const BOX_SLOTS: u32 = 30;
/// PokeParam ids read here.
pub const PP_SPECIES: u32 = 5;
pub const PP_ITEM: u32 = 6;
pub const PP_OT_ID: u32 = 7;
pub const PP_EGG: u32 = 0x4C;
pub const PP_MAIL: u32 = 0xA7;
pub const PP_FORM_SPECIES: u32 = 0xAB;
/// Box-data ids `FUN_020079BC(box, b, s, id, 0)` reads.
pub const BOX_PRESENT: u32 = 0xA9;
/// Proc id / table of command 308 (`FUN_021BB990`).
pub const PROC_308: (u32, u32) = (0x37, 0x021F_45F0);
/// Heap id / size of the strbuf of command 471.
pub const STRBUF_471: (u32, u32) = (8, 0x8001);
/// OBJ code / record id of the delivery man (`FUN_021B6E24` / `FUN_021B6E68`).
pub const PDW_OBJ_CODE: u32 = 0x46;
/// Item ids row 2 gives (1..=0x272).
pub const PDW_ITEM_MAX: u32 = 0x272;
/// The special record of [`pdw_is_special`] (R+0xB0 u16 and R[0]).
pub const PDW_SPECIAL: (u16, u32) = (0x7FE, 0x23E);

/// The u16 table at 0x021D99E8 (overlay 21 ROM) that command 564 indexes (11 entries).
pub const TABLE_564_ADDR: u32 = 0x021D_99E8;
pub const TABLE_564: [u16; 11] = [
    0x75, 0x75, 0x76, 0x85, 0xF8, 0xF9, 0xFA, 0xFB, 0xFC, 0xFD, 0xFE,
];
/// The `month << 8 | day` days at 0x021D99D4 (overlay 21 ROM) that `FUN_021B7390` recognises.
pub const SPECIAL_DAYS_ADDR: u32 = 0x021D_99D4;
pub const SPECIAL_DAYS: [u16; 5] = [0x0A01, 0x0A0E, 0x0C1E, 0x0201, 0x060C];
/// Command 565's value by `day % 10` (0x021D99DE, overlay 21 ROM); 0xA on a special day.
pub const DAY_TABLE_ADDR: u32 = 0x021D_99DE;
pub const DAY_TABLE: [u8; 10] = [9, 1, 3, 5, 4, 2, 6, 8, 0, 7];
/// ARC 208 member by table index (0x021D99D0, overlay 21 ROM): commands 526 / 527 / 585 / 586.
pub const ARC208_INDEX_ADDR: u32 = 0x021D_99D0;
pub const ARC208_INDEX: [u8; 4] = [0, 3, 2, 1];
pub const ARC_208: u32 = 0xD0;
/// The Dream World delivery rows (0x021D9970, overlay 21 ROM): {kind, check, give, kind fn, name a, name b} (Thumb code
/// addresses of this area; row 0 is empty).
pub const PDW_ROWS_ADDR: u32 = 0x021D_9970;
pub const PDW_ROWS: [[u32; 6]; 4] = [
    [0, 0, 0, 0, 0, 0],
    [
        1,
        0x021B_6F01,
        0x021B_6F19,
        0x021B_6F5D,
        0x021B_6F89,
        0x021B_6FE9,
    ],
    [
        2,
        0x021B_6FED,
        0x021B_6FF1,
        0x021B_7021,
        0x021B_7039,
        0x021B_7069,
    ],
    [
        3,
        0x021B_707D,
        0x021B_7081,
        0x021B_7099,
        0x021B_709D,
        0x021B_70D5,
    ],
];
/// Records of the 10-byte table that command 460 / 461 walk; the list ends at an id of 0xFFFF (at most 8 records).
pub const RECORD_SIZE: u32 = 10;
pub const RECORD_MAX: u32 = 8;

/// Callees of the scrcmd_pokemon_fld functions outside the area, named by address (meaning per the spec; UNVERIFIED unless
/// stated). vm.c, the operand readers, the ctx getters and the child-event call come from [`CoreHooks`].
pub trait PokemonFldHooks: CoreHooks {
    /// C: `FUN_0215A5CC(ctx)` (script_sys): the heap id of the script.
    fn fun_0215a5cc(&mut self, ctx: u32) -> u32;
    /// C: `FUN_0215A5D0(ctx)` (script_sys).
    fn fun_0215a5d0(&mut self, ctx: u32) -> u32;
    /// C: `FUN_0215A6C4(ctx)` (scrcmd_work): the 10-byte record table of the script.
    fn fun_0215a6c4(&mut self, ctx: u32) -> u32;
    /// C: `FUN_0215A6D0(ctx, pc)` (scrcmd_work): jump.
    fn fun_0215a6d0(&mut self, ctx: u32, pc: u32);
    /// C: `FUN_0215A7F0(ctx, id, x, i, a, b)` (scrcmd_work): add a menu item.
    #[allow(clippy::too_many_arguments)]
    fn fun_0215a7f0(&mut self, ctx: u32, id: u32, x: u32, i: u32, a: u32, b: u32);
    /// C: `FUN_02158F54(sw)` (script_sys): the GAMESYS.
    fn fun_02158f54(&mut self, sw: u32) -> u32;
    /// C: `FUN_02158F58(sw)` (script_sys): a record whose word +4 is the field map.
    fn fun_02158f58(&mut self, sw: u32) -> u32;
    /// C: `FUN_02158F74(sw)` (script_sys): the word set.
    fn fun_02158f74(&mut self, sw: u32) -> u32;
    /// C: `FUN_02158F78(sw)` (script_sys).
    fn fun_02158f78(&mut self, sw: u32) -> u32;
    /// C: `FUN_02158F7C(sw)` (script_sys): the string buffer.
    fn fun_02158f7c(&mut self, sw: u32) -> u32;
    /// C: `FUN_0201210C(gamesys)` (main): the field map.
    fn fun_0201210c(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_020127A4(gamedata)` (main).
    fn fun_020127a4(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012844(gamedata)` (main).
    fn fun_02012844(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_0201292C(gamedata)` (main): the bag.
    fn fun_0201292c(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012934(gamedata)` (main): the party.
    fn fun_02012934(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_0201293C(gamedata)` (main): the box data.
    fn fun_0201293c(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012944(gamedata)` (main): the player status.
    fn fun_02012944(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012EBC(gamedata)` (main): the save data.
    fn fun_02012ebc(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012F08(gamedata)` (main).
    fn fun_02012f08(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02008530(status)` (main): the player's OT id.
    fn fun_02008530(&mut self, status: u32) -> u32;
    /// C: `FUN_0201AA18(party)` (main): the party count.
    fn fun_0201aa18(&mut self, party: u32) -> u32;
    /// C: `FUN_0201AB28(party, i)` (main): party mon `i`.
    fn fun_0201ab28(&mut self, party: u32, i: u32) -> u32;
    /// C: `FUN_0201AB48(party, i, mon)` (main): replace party mon `i`.
    fn fun_0201ab48(&mut self, party: u32, i: u32, mon: u32);
    /// C: `FUN_0201AB8C(party, i, a, heap)` (main): release party mon `i`.
    fn fun_0201ab8c(&mut self, party: u32, i: u32, a: u32, heap: u32) -> u32;
    /// C: `FUN_0201A98C(party, mon)` (main): add a mon to the party.
    fn fun_0201a98c(&mut self, party: u32, mon: u32);
    /// C: `FUN_02017E1C(mon, id, x)` (main): a PokeParam value.
    fn fun_02017e1c(&mut self, mon: u32, id: u32, x: u32) -> u32;
    /// C: `FUN_02017E40(mon, id, v)` (main): set a PokeParam value.
    fn fun_02017e40(&mut self, mon: u32, id: u32, v: u32);
    /// C: `FUN_02017988(mon, form)` (main): set the form.
    fn fun_02017988(&mut self, mon: u32, form: u32);
    /// C: `FUN_020185C4(mon)` (main).
    fn fun_020185c4(&mut self, mon: u32);
    /// C: `FUN_020185F0(mon)` (main): the box-mon part of `mon`.
    fn fun_020185f0(&mut self, mon: u32) -> u32;
    /// C: `FUN_0201747C(boxmon, heap)` (main): a party mon built from a box mon.
    fn fun_0201747c(&mut self, boxmon: u32, heap: u32) -> u32;
    /// C: `FUN_02007698(box)` (main): the current box.
    fn fun_02007698(&mut self, bx: u32) -> u32;
    /// C: `FUN_020079BC(box, b, s, id, x)` (main): a value of the box mon in box `b`, slot `s`.
    fn fun_020079bc(&mut self, bx: u32, b: u32, s: u32, id: u32, x: u32) -> u32;
    /// C: `FUN_02007A18(box, b, s)` (main): the box mon in box `b`, slot `s`.
    fn fun_02007a18(&mut self, bx: u32, b: u32, s: u32) -> u32;
    /// C: `FUN_020075B0(box, b, s, boxmon)` (main): store a box mon.
    fn fun_020075b0(&mut self, bx: u32, b: u32, s: u32, boxmon: u32);
    /// C: `FUN_02008FEC(gamedata)` (main): the mail box.
    fn fun_02008fec(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02008F08(heap)` (main): a new mail record.
    fn fun_02008f08(&mut self, heap: u32) -> u32;
    /// C: `FUN_02009020(mailbox, x)` (main): a free mail slot (< 0: none).
    fn fun_02009020(&mut self, mailbox: u32, x: u32) -> u32;
    /// C: `FUN_0200903C(mailbox, x, slot, mail)` (main): store the mail.
    fn fun_0200903c(&mut self, mailbox: u32, x: u32, slot: u32, mail: u32);
    /// C: `FUN_02008EAC(mail)` (main): clear the mail record.
    fn fun_02008eac(&mut self, mail: u32);
    /// C: `FUN_0202136C(item)` (main): 1 for a mail item.
    fn fun_0202136c(&mut self, item: u32) -> u32;
    /// C: `Heap_AllocDebug(heap, size, from_end, file, line)` (`0x02030734`).
    fn fun_02030734(&mut self, heap: u32, size: u32, from_end: u32, file: u32, line: u32) -> u32;
    /// C: `Heap_Free(block)` (`0x020307B0`).
    fn fun_020307b0(&mut self, block: u32);
    /// C: `FUN_02188C84(fm)` (overlay 21): the field heap id.
    fn fun_02188c84(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188C7C(fm)` (overlay 21): the field OBJ system.
    fn fun_02188c7c(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188D24(fm)` (overlay 21).
    fn fun_02188d24(&mut self, fm: u32) -> u32;
    /// C: `FUN_021BB990(proc, x, table)` (overlay 21): load the proc `proc` with `table`.
    fn fun_021bb990(&mut self, proc_id: u32, x: u32, table: u32) -> u32;
    /// C: `FUN_021854B0(gamesys, fm, zone, &pos, x)` (overlay 21): the warp event to the VecFx32 `pos`.
    fn fun_021854b0(&mut self, gamesys: u32, fm: u32, zone: u32, pos: &[u8; 12], x: u32) -> u32;
    /// C: `FUN_0200A8A0(save)` (main): the calendar-name save block.
    fn fun_0200a8a0(&mut self, save: u32) -> u32;
    /// C: `FUN_0200A8AC(s)` (main).
    fn fun_0200a8ac(&mut self, s: u32) -> u32;
    /// C: `FUN_0200A8C8(s, i)` (main): name `i` (a string).
    fn fun_0200a8c8(&mut self, s: u32, i: u32) -> u32;
    /// C: `FUN_0200A8F8(s, i)` (main): 1 when name `i` is ... (UNVERIFIED).
    fn fun_0200a8f8(&mut self, s: u32, i: u32) -> u32;
    /// C: `FUN_0200A940(s, month, day, x)` (main): the name index of a date.
    fn fun_0200a940(&mut self, s: u32, month: u32, day: u32, x: u32) -> u32;
    /// C: `FUN_021690A4(gamedata)` (overlay 21).
    fn fun_021690a4(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_021690D0(gamedata)` (overlay 21): the month.
    fn fun_021690d0(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_021690E0(gamedata)` (overlay 21): the day.
    fn fun_021690e0(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02087C88(&date)` (main, RTC): the date (8 bytes; month byte 2, day byte 3).
    fn fun_02087c88(&mut self, date: &mut [u8; 8]);
    /// C: `FUN_020457B0(n, heap)` (main): a new string buffer.
    fn fun_020457b0(&mut self, n: u32, heap: u32) -> u32;
    /// C: `FUN_02045808(strbuf)` (main): free the string buffer.
    fn fun_02045808(&mut self, buf: u32);
    /// C: `FUN_02045924(strbuf, src)` (main): copy a string.
    fn fun_02045924(&mut self, buf: u32, src: u32);
    /// C: `FUN_0201EE0C(wordset, i, strbuf, a, b, c)` (main): put a string into the word set.
    #[allow(clippy::too_many_arguments)]
    fn fun_0201ee0c(&mut self, ws: u32, i: u32, buf: u32, a: u32, b: u32, c: u32);
    /// C: `FUN_0201EFA4(wordset, i, n, digits, a, b)` (main): put a number into the word set.
    #[allow(clippy::too_many_arguments)]
    fn fun_0201efa4(&mut self, ws: u32, i: u32, n: u32, digits: u32, a: u32, b: u32);
    /// C: `FUN_0201EEC4(wordset, i, x)` (main).
    fn fun_0201eec4(&mut self, ws: u32, i: u32, x: u32);
    /// C: `FUN_0201EED4(wordset, i, mon)` (main): a mon's species name.
    fn fun_0201eed4(&mut self, ws: u32, i: u32, mon: u32);
    /// C: `FUN_0201EF40(wordset, i, item)` (main): an item name.
    fn fun_0201ef40(&mut self, ws: u32, i: u32, item: u32);
    /// C: `FUN_0201F02C(wordset, i, status)` (main): the player name.
    fn fun_0201f02c(&mut self, ws: u32, i: u32, status: u32);
    /// C: `FUN_0201F184(wordset, i, x)` (main).
    fn fun_0201f184(&mut self, ws: u32, i: u32, x: u32);
    /// C: `FUN_0204A298(arc, member, x, heap, &size)` (main): load an ARC member (`size` = its byte size).
    fn fun_0204a298(&mut self, arc: u32, member: u32, x: u32, heap: u32, size: &mut [u8; 4])
        -> u32;
    /// C: `FUN_02007F68(bag, item, n, heap)` (main): nonzero when the bag holds `n` of `item`.
    fn fun_02007f68(&mut self, bag: u32, item: u32, n: u32, heap: u32) -> u32;
    /// C: `FUN_02007E50(bag, item, n, heap)` (main): add `n` of `item`.
    fn fun_02007e50(&mut self, bag: u32, item: u32, n: u32, heap: u32) -> u32;
    /// C: `FUN_0200C278(x, v)` (main).
    fn fun_0200c278(&mut self, x: u32, v: u32);
    /// C: `FUN_0200CA1C(gamedata)` (main): the Pokedex (UNVERIFIED).
    fn fun_0200ca1c(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_0200CDE0(dex, mon)` (main): register a caught mon (UNVERIFIED).
    fn fun_0200cde0(&mut self, dex: u32, mon: u32);
    /// C: `FUN_0200BE48(save)` (main): a save block.
    fn fun_0200be48(&mut self, save: u32) -> u32;
    /// C: `FUN_0200BF18(block, v)` (main).
    fn fun_0200bf18(&mut self, block: u32, v: u32) -> u32;
    /// C: `FUN_02157E9C(rec, heap, gamedata)` (overlay 10): a mon built from a delivery record (or null).
    fn fun_02157e9c(&mut self, rec: u32, heap: u32, gamedata: u32) -> u32;
    /// C: `FUN_0216DD1C(objsys, &obj, &index)` (overlay 21, fldmmdl): the next OBJ (1) or the end (0).
    fn fun_0216dd1c(&mut self, sys: u32, obj: &mut [u8; 4], index: &mut [u8; 4]) -> u32;
    /// C: `FUN_0216D588(obj)` (overlay 21, fldmmdl): the OBJ id.
    fn fun_0216d588(&mut self, obj: u32) -> u32;
    /// C: `FUN_0216D598(obj)` (overlay 21, fldmmdl): the OBJ code.
    fn fun_0216d598(&mut self, obj: u32) -> u32;
    /// C: `FUN_02162740(s)` (overlay 21): the event-data OBJ records (0x24 bytes each, or null).
    fn fun_02162740(&mut self, s: u32) -> u32;
    /// C: `FUN_02162744(s)` (overlay 21): their count.
    fn fun_02162744(&mut self, s: u32) -> u32;
}

/// C: `sub_021b66b8`: command 581: v, var; v 0 -> `*var = `[`find_in_party`]` >= 0`; 1 -> [`release_found`]; 2 ->
/// [`swap_with_box`]`(ctx, *var)`, [`release_found`]; other v nothing; 0.
pub fn cmd_581<H: PokemonFldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let v = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let gd = h.fun_0215a5e4(ctx);
    match v {
        0 => {
            let r = find_in_party(h, gd) as i32 >= 0;
            h.w16(var, r as u16);
        }
        1 => release_found(h, ctx),
        2 => {
            let slot = u32::from(h.r16(var));
            swap_with_box(h, ctx, slot);
            release_found(h, ctx);
        }
        _ => {}
    }
    0
}

/// C: `FUN_overlay_d_21__021b6708`: swap_with_box(ctx, slot): the box mon [`find_in_box`]`(0x283)` finds replaces party mon
/// `slot`, which goes into its box place: M = a copy of the party mon (`Heap_AllocDebug(heap, 0xDC, 1, ..)`, MI_CpuCopy8); a
/// held mail item (`FUN_0202136C`) moves the mail into the mail box (`FUN_02009020` slot >= 0) and clears it from M; form
/// species 0x1EC reverts to form 0; the box mon becomes party mon `slot` (`FUN_0201747C`, `FUN_0201AB48`) and M's box part goes
/// to the box (`FUN_020075B0`). Nothing when none is found.
pub fn swap_with_box<H: PokemonFldHooks>(h: &mut H, ctx: u32, slot: u32) {
    let gd = h.fun_0215a5e4(ctx);
    let party = h.fun_02012934(gd);
    let Some((bx, pos)) = find_in_box_at(h, gd, SPECIES_581) else {
        return;
    };
    let heap = h.fun_0215a5cc(ctx);
    let boxes = h.fun_0201293c(gd);
    let m = h.fun_02030734(heap, PARTY_MON_SIZE, 1, FILE, 0x4C7);
    let src = h.fun_0201ab28(party, slot);
    for i in 0..PARTY_MON_SIZE {
        let b = h.r8(src.wrapping_add(i));
        h.w8(m.wrapping_add(i), b);
    }
    let item = h.fun_02017e1c(m, PP_ITEM, 0) & 0xFFFF;
    if h.fun_0202136c(item) == 1 {
        let mailbox = h.fun_02008fec(gd);
        let heap2 = h.fun_0215a5cc(ctx);
        let mail = h.fun_02008f08(heap2);
        let k = h.fun_02009020(mailbox, 0);
        h.fun_02017e1c(m, PP_MAIL, mail);
        h.fun_02012944(gd);
        if k as i32 >= 0 {
            h.fun_0200903c(mailbox, 0, k, mail);
        }
        h.fun_02008eac(mail);
        h.fun_02017e40(m, PP_MAIL, mail);
        h.fun_02017e40(m, PP_ITEM, 0);
        h.fun_020307b0(mail);
    }
    if h.fun_02017e1c(m, PP_FORM_SPECIES, 0) == FORM_SPECIES_1EC {
        h.fun_02017988(m, 0);
        h.fun_020185c4(m);
    }
    let boxmon = h.fun_02007a18(boxes, bx, pos);
    let s = h.fun_0201747c(boxmon, heap);
    h.fun_0201ab48(party, slot, s);
    h.fun_020307b0(s);
    let part = h.fun_020185f0(m);
    h.fun_020075b0(boxes, bx, pos, part);
    h.fun_020307b0(m);
}

/// C: `FUN_overlay_d_21__021b6850`: find_in_box(GD, species, u16 *box, u16 *pos): from the current box on (24 boxes, 30 slots)
/// the first box mon that is present, has the player's OT id and `species` and is not an egg -> `*box`, `*pos`, true; else
/// false.
pub fn find_in_box<H: PokemonFldHooks>(
    h: &mut H,
    gd: u32,
    species: u32,
    bx: u32,
    pos: u32,
) -> bool {
    match find_in_box_at(h, gd, species) {
        Some((b, s)) => {
            h.w16(bx, b as u16);
            h.w16(pos, s as u16);
            true
        }
        None => false,
    }
}

/// [`find_in_box`] with the place returned: `Some((box, slot))`.
fn find_in_box_at<H: PokemonFldHooks>(h: &mut H, gd: u32, species: u32) -> Option<(u32, u32)> {
    h.fun_02012934(gd);
    let status = h.fun_02012944(gd);
    let boxes = h.fun_0201293c(gd);
    let cur = h.fun_02007698(boxes);
    let id = h.fun_02008530(status);
    for b in 0..BOXES {
        let bx = (b.wrapping_add(cur as i32)).wrapping_rem(BOXES) as u32;
        for s in 0..BOX_SLOTS {
            if h.fun_020079bc(boxes, bx, s, BOX_PRESENT, 0) == 0 {
                continue;
            }
            if h.fun_020079bc(boxes, bx, s, PP_OT_ID, 0) != id {
                continue;
            }
            if h.fun_020079bc(boxes, bx, s, PP_SPECIES, 0) != species {
                continue;
            }
            if h.fun_020079bc(boxes, bx, s, PP_EGG, 0) != 0 {
                continue;
            }
            return Some((bx, s));
        }
    }
    None
}

/// C: `FUN_overlay_d_21__021b6904`: release_found(ctx): the party mon [`find_in_party`] finds (if any) is released
/// (`FUN_0201AB8C(party, i, 0, heap)`).
pub fn release_found<H: PokemonFldHooks>(h: &mut H, ctx: u32) {
    let gd = h.fun_0215a5e4(ctx);
    let party = h.fun_02012934(gd);
    let i = find_in_party(h, gd);
    if (i as i32) < 0 {
        return;
    }
    let heap = h.fun_0215a5cc(ctx);
    h.fun_0201ab8c(party, i, 0, heap);
}

/// C: `FUN_overlay_d_21__021b6934`: find_in_party(GD): the first party mon with the player's OT id, species 0x283 and not an
/// egg -> its slot; else -1.
pub fn find_in_party<H: PokemonFldHooks>(h: &mut H, gd: u32) -> u32 {
    let party = h.fun_02012934(gd);
    let status = h.fun_02012944(gd);
    let n = h.fun_0201aa18(party) as i32;
    let id = h.fun_02008530(status);
    let mut i: i32 = 0;
    while i < n {
        let mon = h.fun_0201ab28(party, i as u32);
        if h.fun_02017e1c(mon, PP_OT_ID, 0) == id
            && h.fun_02017e1c(mon, PP_SPECIES, 0) == SPECIES_581
            && h.fun_02017e1c(mon, PP_EGG, 0) == 0
        {
            return i as u32;
        }
        i += 1;
    }
    0xFFFF_FFFF
}

/// C: `sub_021b69b0`: command 308: `FUN_021BB990(0x37, FUN_02188D24(field map), 0x021F45F0)`; 0.
pub fn cmd_308<H: PokemonFldHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let r = h.fun_02158f58(sw);
    let fm = h.r32(r + 4);
    let x = h.fun_02188d24(fm);
    h.fun_021bb990(PROC_308.0, x, PROC_308.1);
    0
}

/// C: `sub_021b69d8`: command 471 (today's name): v; i = `FUN_0200A940(S, month, day, FUN_021690A4(GAMEDATA))` of the RTC date;
/// the name `FUN_0200A8C8(S, i)` goes into word-set slot v (`FUN_0201EE0C(L, v, B, FUN_0200A8F8(S, i) != 1, 1, 2)`) through a
/// temporary string buffer; 0.
pub fn cmd_471<H: PokemonFldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let v = h.fun_02159b10(vm, ctx);
    let save = h.fun_02012ebc(gd);
    let s = h.fun_0200a8a0(save);
    let x = h.fun_021690a4(gd);
    let mut date = [0u8; 8];
    h.fun_02087c88(&mut date);
    let i = h.fun_0200a940(s, u32::from(date[2]), u32::from(date[3]), x);
    let f = (h.fun_0200a8f8(s, i) != 1) as u32;
    let name = h.fun_0200a8c8(s, i);
    let sw = h.fun_0215a5f4(ctx);
    let ws = h.fun_02158f74(sw);
    let buf = h.fun_020457b0(STRBUF_471.0, STRBUF_471.1);
    h.fun_02045924(buf, name);
    h.fun_0201ee0c(ws, v, buf, f, 1, 2);
    h.fun_02045808(buf);
    0
}

/// C: `sub_021b6a78`: command 470: `*var = FUN_0200A8AC(FUN_0200A8A0(SAVE))`; 0.
pub fn cmd_470<H: PokemonFldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let save = h.fun_02012ebc(gd);
    let s = h.fun_0200a8a0(save);
    let v = h.fun_0200a8ac(s);
    h.w16(var, v as u16);
    0
}

/// C: `FUN_overlay_d_21__021b6aa4`: record `i` of a 10-byte record table.
pub fn record_at(table: u32, i: u32) -> u32 {
    table.wrapping_add(i.wrapping_mul(RECORD_SIZE))
}

/// C: `FUN_overlay_d_21__021b6aac`: an empty function (`bx lr`; called on each record by [`record_count`]).
pub fn record_nop(_rec: u32) {}

/// C: `FUN_overlay_d_21__021b6ab0`: the index of the first record (of 8) whose u16 id is 0xFFFF; 0 when none is.
pub fn record_count<H: crate::hooks::MemEnv>(h: &mut H, table: u32) -> u32 {
    for i in 0..RECORD_MAX {
        let rec = record_at(table, i);
        record_nop(rec);
        if h.r16(rec) == 0xFFFF {
            return i;
        }
    }
    0
}

/// C: `sub_021b6ae0`: command 459: off = u32 operand; jump to the script PC (VM +0x14) + off (`FUN_0215A6D0`); 0.
pub fn cmd_459<H: PokemonFldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let off = h.fun_0201134c(vm);
    let pc = h.r32(vm + crate::vm::VM_PC).wrapping_add(off);
    h.fun_0215a6d0(ctx, pc);
    0
}

/// C: `sub_021b6afc`: command 460: for each record i (below [`record_count`]) of the table `FUN_0215A6C4(ctx)` whose u16 at
/// +2 differs from `FUN_0215A5D0(ctx)`: the menu item `FUN_0215A7F0(ctx, id, 0xFFFF, i, A, B)`; 0.
pub fn cmd_460<H: PokemonFldHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let t = h.fun_0215a6c4(ctx);
    let sw = h.fun_0215a5f4(ctx);
    let a = h.fun_02158f78(sw);
    let b = h.fun_02158f7c(sw);
    let x = h.fun_0215a5d0(ctx);
    let n = record_count(h, t);
    let mut i = 0;
    while i < n {
        let e = record_at(t, i);
        if x != u32::from(h.r16(e + 2)) {
            let id = u32::from(h.r16(e));
            h.fun_0215a7f0(ctx, id, 0xFFFF, i, a, b);
        }
        i += 1;
    }
    0
}

/// C: `sub_021b6b6c`: command 461: i; record i (below [`record_count`]) of `FUN_0215A6C4(ctx)` -> the warp event
/// `FUN_021854B0(GAMESYS, fm, e[1], &{e[2] << 16, e[3] << 16, e[4] << 16}, 1)` runs as a child event; 1.
pub fn cmd_461<H: PokemonFldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let i = h.fun_02159b10(vm, ctx);
    let t = h.fun_0215a6c4(ctx);
    h.fun_0215a5d0(ctx);
    if i >= record_count(h, t) {
        return 1;
    }
    let e = record_at(t, i);
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_02158f54(sw);
    let fm = h.fun_0201210c(gs);
    let mut pos = [0u8; 12];
    for k in 0..3 {
        let c = u32::from(h.r16(e + 4 + 2 * k as u32)) << 16;
        pos[4 * k..4 * k + 4].copy_from_slice(&c.to_le_bytes());
    }
    let zone = u32::from(h.r16(e + 2));
    let ev = h.fun_021854b0(gs, fm, zone, &pos, 1);
    h.fun_0215897c(sw, ev);
    1
}

/// The delivery-row handlers of [`PDW_ROWS`] (Thumb addresses), called with the original's three register arguments.
fn pdw_call<H: PokemonFldHooks>(h: &mut H, f: u32, a: u32, b: u32, c: u32) -> u32 {
    match f {
        0x021B_6F01 => pdw_party_room(h, a, b),
        0x021B_6F19 => {
            pdw_give_mon(h, a, b, c);
            0
        }
        0x021B_6F5D => pdw_mon_kind(h, a, b, c),
        0x021B_6F89 => pdw_mon_names(h, a, b, c),
        0x021B_6FE9 => pdw_const_6(),
        0x021B_6FED | 0x021B_707D => pdw_const_1(f),
        0x021B_6FF1 => pdw_give_item(h, a, b, c),
        0x021B_7021 => pdw_item_kind(h, a, b, c),
        0x021B_7039 => pdw_item_names(h, a, b, c),
        0x021B_7069 => pdw_item_name(h, a, b),
        0x021B_7081 => pdw_give_3(h, a, b, c),
        0x021B_7099 => pdw_const_5(),
        0x021B_709D => pdw_names_3(h, a, b, c),
        0x021B_70D5 => pdw_const_a(),
        _ => unreachable!("{f:#x} is not a delivery-row handler"),
    }
}

/// The handler at `ofs` of row `k` of [`PDW_ROWS`] (0 = none). Rust panics for `k` > 3 (the original reads past the table).
fn pdw_row(k: u32, ofs: usize) -> u32 {
    PDW_ROWS[k as usize][ofs / 4]
}

/// C: `FUN_overlay_d_21__021b6d0c`: pdw_check(k, a, b, R): R and row k's check -> `check(a, b, R)`; else 0.
pub fn pdw_check<H: PokemonFldHooks>(h: &mut H, k: u32, a: u32, b: u32, r: u32) -> u32 {
    let f = pdw_row(k, 4);
    if r == 0 || f == 0 {
        return 0;
    }
    pdw_call(h, f, a, b, r)
}

/// C: `FUN_overlay_d_21__021b6d30`: pdw_name_a(k, R, ctx): R and row k's handler +0x10 -> `f(word set, R, ctx)`; else 7.
pub fn pdw_name_a<H: PokemonFldHooks>(h: &mut H, k: u32, r: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let ws = h.fun_02158f74(sw);
    let f = pdw_row(k, 0x10);
    if r == 0 || f == 0 {
        return 7;
    }
    pdw_call(h, f, ws, r, ctx)
}

/// C: `FUN_overlay_d_21__021b6d64`: pdw_name_b(k, R, ctx): R and row k's handler +0x14 -> `f(word set, R, ctx)`; else 8.
pub fn pdw_name_b<H: PokemonFldHooks>(h: &mut H, k: u32, r: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let ws = h.fun_02158f74(sw);
    let f = pdw_row(k, 0x14);
    if r == 0 || f == 0 {
        return 8;
    }
    pdw_call(h, f, ws, r, ctx)
}

/// C: `FUN_overlay_d_21__021b6d98`: pdw_give(k, a, b, R): R and row k's give -> `give(a, b, R)`, 1; else 0.
pub fn pdw_give<H: PokemonFldHooks>(h: &mut H, k: u32, a: u32, b: u32, r: u32) -> u32 {
    let f = pdw_row(k, 8);
    if r == 0 || f == 0 {
        return 0;
    }
    pdw_call(h, f, a, b, r);
    1
}

/// C: `FUN_overlay_d_21__021b6dc0`: pdw_kind(k, a, b, R): R and row k's handler +0xC -> `f(a, b, R)`; else 0.
pub fn pdw_kind<H: PokemonFldHooks>(h: &mut H, k: u32, a: u32, b: u32, r: u32) -> u32 {
    let f = pdw_row(k, 0xC);
    if r == 0 || f == 0 {
        return 0;
    }
    pdw_call(h, f, a, b, r)
}

/// C: `FUN_overlay_d_21__021b6de4`: the delivery man's OBJ id ([`pdw_find_obj`] -> `FUN_0216D588`), else his event-data
/// record id ([`pdw_find_record`], 0 when there is none).
pub fn pdw_obj_or_record<H: PokemonFldHooks>(h: &mut H, fm: u32, gd: u32) -> u32 {
    let o = pdw_find_obj(h, fm);
    if o != 0 {
        return h.fun_0216d588(o);
    }
    let r = pdw_find_record(h, gd);
    if (r as i32) < 0 {
        0
    } else {
        r & 0xFFFF
    }
}

/// C: `FUN_overlay_d_21__021b6e0c`: whether the delivery man's OBJ is on the map ([`pdw_find_obj`] != 0).
pub fn pdw_has_obj<H: PokemonFldHooks>(h: &mut H, fm: u32) -> u16 {
    (pdw_find_obj(h, fm) != 0) as u16
}

/// C: `FUN_overlay_d_21__021b6e24`: the first OBJ of the field OBJ system (`FUN_0216DD1C` iterator) whose code is 0x46; 0
/// when none is.
pub fn pdw_find_obj<H: PokemonFldHooks>(h: &mut H, fm: u32) -> u32 {
    let mut index = [0u8; 4];
    let mut obj = [0u8; 4];
    let sys = h.fun_02188c7c(fm);
    while h.fun_0216dd1c(sys, &mut obj, &mut index) == 1 {
        let o = u32::from_le_bytes(obj);
        if h.fun_0216d598(o) == PDW_OBJ_CODE {
            return u32::from_le_bytes(obj);
        }
    }
    0
}

/// C: `FUN_overlay_d_21__021b6e68`: the id (u16 +0) of the first event-data OBJ record (0x24 bytes) whose u16 +2 is 0x46;
/// -1 when there is none (or no records).
pub fn pdw_find_record<H: PokemonFldHooks>(h: &mut H, gd: u32) -> u32 {
    let s = h.fun_02012844(gd);
    let a = h.fun_02162740(s);
    let n = h.fun_02162744(s) as i32;
    if a == 0 || n == 0 {
        return 0xFFFF_FFFF;
    }
    let mut i: i32 = 0;
    while i < n {
        let e = a.wrapping_add(0x24u32.wrapping_mul(i as u32));
        if u32::from(h.r16(e + 2)) == PDW_OBJ_CODE {
            return u32::from(h.r16(e));
        }
        i += 1;
    }
    0xFFFF_FFFF
}

/// C: `FUN_overlay_d_21__021b6eb0`: the delivery record R is the special one: kind 2, R+0xB0 u16 0x7FE and R[0] 0x23E.
pub fn pdw_is_special<H: crate::hooks::MemEnv>(h: &mut H, r: u32, k: u32) -> bool {
    k == 2 && h.r16(r + 0xB0) == PDW_SPECIAL.0 && h.r32(r) == PDW_SPECIAL.1
}

/// C: `FUN_overlay_d_21__021b6ee0`: the mon of a delivery record: `FUN_02157E9C(R, heap, GAMEDATA)`.
pub fn pdw_make_mon<H: PokemonFldHooks>(h: &mut H, ctx: u32, r: u32) -> u32 {
    let heap = h.fun_0215a5cc(ctx);
    let gd = h.fun_0215a5e4(ctx);
    h.fun_02157e9c(r, heap, gd)
}

/// C: `FUN_overlay_d_21__021b6f00`: row 1 check (ctx, GAMEDATA): the party has room (count < 6).
pub fn pdw_party_room<H: PokemonFldHooks>(h: &mut H, _ctx: u32, gd: u32) -> u32 {
    let party = h.fun_02012934(gd);
    ((h.fun_0201aa18(party) as i32) < 6) as u32
}

/// C: `FUN_overlay_d_21__021b6f18`: row 1 give (ctx, GAMEDATA, R): the record's mon joins the party (when it has room and the
/// mon was built) and is registered in the Pokedex (`FUN_0200CDE0`); the mon is freed.
pub fn pdw_give_mon<H: PokemonFldHooks>(h: &mut H, ctx: u32, gd: u32, r: u32) {
    let party = h.fun_02012934(gd);
    let mon = pdw_make_mon(h, ctx, r);
    if (h.fun_0201aa18(party) as i32) >= 6 || mon == 0 {
        return;
    }
    h.fun_0201a98c(party, mon);
    let dex = h.fun_0200ca1c(gd);
    h.fun_0200cde0(dex, mon);
    h.fun_020307b0(mon);
}

/// C: `FUN_overlay_d_21__021b6f5c`: row 1 kind (ctx, GAMEDATA, R): no mon -> 1; an egg -> 2; else 1 (the mon is freed).
pub fn pdw_mon_kind<H: PokemonFldHooks>(h: &mut H, ctx: u32, _gd: u32, r: u32) -> u32 {
    let mon = pdw_make_mon(h, ctx, r);
    if mon == 0 {
        return 1;
    }
    let egg = h.fun_02017e1c(mon, PP_EGG, 0);
    h.fun_020307b0(mon);
    if egg == 1 {
        2
    } else {
        1
    }
}

/// C: `FUN_overlay_d_21__021b6f88`: row 1 names (L, R, ctx): word 0 = the player name; no mon -> word 1 = `FUN_0201EEC4(L, 1,
/// 0)`, 5; an egg -> 0xB; else word 1 = the species name, 5 (the mon is freed).
pub fn pdw_mon_names<H: PokemonFldHooks>(h: &mut H, ws: u32, r: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let status = h.fun_02012944(gd);
    let mon = pdw_make_mon(h, ctx, r);
    h.fun_0201f02c(ws, 0, status);
    if mon == 0 {
        h.fun_0201eec4(ws, 1, 0);
        return 5;
    }
    let v = if h.fun_02017e1c(mon, PP_EGG, 0) == 1 {
        0xB
    } else {
        h.fun_0201eed4(ws, 1, mon);
        5
    };
    h.fun_020307b0(mon);
    v
}

/// C: `FUN_overlay_d_21__021b6fe8`: row 1 handler +0x14: 6.
pub fn pdw_const_6() -> u32 {
    6
}

/// C: `FUN_overlay_d_21__021b6fec` / `FUN_overlay_d_21__021b707c` (rows 2 / 3 check, two identical functions): 1.
pub fn pdw_const_1(_f: u32) -> u32 {
    1
}

/// C: `FUN_overlay_d_21__021b6ff0`: row 2 give (ctx, GAMEDATA, R): the item (u16)R[0] in 1..=0x272 goes into the bag
/// (`FUN_02007E50(bag, item, 1, heap)`, its result returned); other items: the bag pointer is returned (r0 left over).
pub fn pdw_give_item<H: PokemonFldHooks>(h: &mut H, ctx: u32, gd: u32, r: u32) -> u32 {
    let heap = h.fun_0215a5cc(ctx);
    let bag = h.fun_0201292c(gd);
    let item = h.r32(r) & 0xFFFF;
    if item == 0 || item > PDW_ITEM_MAX {
        return bag;
    }
    h.fun_02007e50(bag, item, 1, heap)
}

/// C: `FUN_overlay_d_21__021b7020`: row 2 kind (ctx, GAMEDATA, R): [`pdw_is_special`]`(R, R[0xB3])` ? 4 : 3.
pub fn pdw_item_kind<H: PokemonFldHooks>(h: &mut H, _ctx: u32, _gd: u32, r: u32) -> u32 {
    let k = u32::from(h.r8(r + 0xB3));
    if pdw_is_special(h, r, k) {
        4
    } else {
        3
    }
}

/// C: `FUN_overlay_d_21__021b7038`: row 2 names (L, R, ctx): word 0 = the player name, word 1 = the item name of (u16)R[0]; 7.
pub fn pdw_item_names<H: PokemonFldHooks>(h: &mut H, ws: u32, r: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let status = h.fun_02012944(gd);
    let item = h.r32(r) & 0xFFFF;
    h.fun_0201f02c(ws, 0, status);
    h.fun_0201ef40(ws, 1, item);
    7
}

/// C: `FUN_overlay_d_21__021b7068`: row 2 handler +0x14 (L, R): word 0 = the item name of (u16)R[0]; 8.
pub fn pdw_item_name<H: PokemonFldHooks>(h: &mut H, ws: u32, r: u32) -> u32 {
    let item = h.r32(r) & 0xFFFF;
    h.fun_0201ef40(ws, 0, item);
    8
}

/// C: `FUN_overlay_d_21__021b7080`: row 3 give (ctx, GAMEDATA, R): `FUN_0200BF18(FUN_0200BE48(SAVE), R[0])` (its result).
pub fn pdw_give_3<H: PokemonFldHooks>(h: &mut H, _ctx: u32, gd: u32, r: u32) -> u32 {
    let save = h.fun_02012ebc(gd);
    let blk = h.fun_0200be48(save);
    let v = h.r32(r);
    h.fun_0200bf18(blk, v)
}

/// C: `FUN_overlay_d_21__021b7098`: row 3 kind: 5.
pub fn pdw_const_5() -> u32 {
    5
}

/// C: `FUN_overlay_d_21__021b709c`: row 3 names (L, R, ctx): `FUN_0200BE48(SAVE)` (unused); word 0 = the player name, word 1 =
/// `FUN_0201F184(L, 1, R[0])`; 9.
pub fn pdw_names_3<H: PokemonFldHooks>(h: &mut H, ws: u32, r: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let save = h.fun_02012ebc(gd);
    h.fun_0200be48(save);
    let status = h.fun_02012944(gd);
    let v = h.r32(r);
    h.fun_0201f02c(ws, 0, status);
    h.fun_0201f184(ws, 1, v);
    9
}

/// C: `FUN_overlay_d_21__021b70d4`: row 3 handler +0x14: 0xA.
pub fn pdw_const_a() -> u32 {
    0xA
}

/// The ARC 208 `{u32 key, u32 value}` table of [`ARC208_INDEX`]`[i]` (`FUN_0204A298(0xD0, .., 0, (heap & 0x7FFF) | 0x8000,
/// &size)`): (table, entry count = size / 8).
fn arc208_table<H: PokemonFldHooks>(h: &mut H, i: u32, heap: u32) -> (u32, u32) {
    let mut size = [0u8; 4];
    let member = u32::from(ARC208_INDEX[i as usize]);
    let t = h.fun_0204a298(
        ARC_208,
        member,
        0,
        ((heap & 0x7FFF) | 0x8000) & 0xFFFF,
        &mut size,
    );
    (t, u32::from_le_bytes(size) >> 3)
}

/// The field heap of the script (`FUN_02188C84(FUN_02158F58(SW) +4)`).
fn field_heap<H: PokemonFldHooks>(h: &mut H, sw: u32) -> u32 {
    let r = h.fun_02158f58(sw);
    let fm = h.r32(r + 4);
    h.fun_02188c84(fm)
}

/// C: `sub_021b70d8`: command 527: key, i, var; `*var` = the value of the first entry of ARC 208 table i whose key is `key`
/// (u16), else 0; the table is freed; 0.
pub fn cmd_527<H: PokemonFldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let heap = field_heap(h, sw);
    let key = h.fun_02159b10(vm, ctx);
    let i = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let (t, n) = arc208_table(h, i, heap);
    h.w16(var, 0);
    for e in 0..n {
        if key == h.r32(t + 8 * e) {
            let v = h.r32(t + 8 * e + 4);
            h.w16(var, v as u16);
            break;
        }
    }
    h.fun_020307b0(t);
    0
}

/// C: `sub_021b716c`: command 526: i, var; `*var` = 1 when the bag holds one of the items (u16 keys) of ARC 208 table i
/// (`FUN_02007F68(bag, item, 1, heap)`), else 0; the table is freed; 0.
pub fn cmd_526<H: PokemonFldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gd = h.fun_0215a5e4(ctx);
    let heap = field_heap(h, sw);
    let i = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let bag = h.fun_0201292c(gd);
    let (t, n) = arc208_table(h, i, heap);
    h.w16(var, 0);
    for e in 0..n {
        let item = h.r32(t + 8 * e) & 0xFFFF;
        if h.fun_02007f68(bag, item, 1, heap) != 0 {
            h.w16(var, 1);
            break;
        }
    }
    h.fun_020307b0(t);
    0
}

/// C: `sub_021b721c`: command 585: key, i, a, b; the value of the entry of ARC 208 table i whose key is `key` goes into
/// word-set slot a as a number (`FUN_0201EFA4(L, a, value, b, 0, 1)`); the table is freed; 0.
pub fn cmd_585<H: PokemonFldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let heap = field_heap(h, sw);
    let key = h.fun_02159b10(vm, ctx);
    let i = h.fun_02159b10(vm, ctx);
    let a = h.fun_02159b10(vm, ctx);
    let b = h.fun_02159b10(vm, ctx);
    let ws = h.fun_02158f74(sw);
    let (t, n) = arc208_table(h, i, heap);
    for e in 0..n {
        if key == h.r32(t + 8 * e) {
            let v = h.r32(t + 8 * e + 4);
            h.fun_0201efa4(ws, a, v, b, 0, 1);
            break;
        }
    }
    h.fun_020307b0(t);
    0
}

/// C: `sub_021b72cc`: command 586: key, i; `FUN_020127A4(GAMEDATA)` (unused); the value of the entry of ARC 208 table i whose
/// key is `key` -> `FUN_0200C278(FUN_02012F08(GAMEDATA), value)`; the table is freed; 0.
pub fn cmd_586<H: PokemonFldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let heap = field_heap(h, sw);
    let key = h.fun_02159b10(vm, ctx);
    let i = h.fun_02159b10(vm, ctx);
    let gd = h.fun_0215a5e4(ctx);
    h.fun_020127a4(gd);
    let (t, n) = arc208_table(h, i, heap);
    for e in 0..n {
        if key == h.r32(t + 8 * e) {
            let x = h.fun_02012f08(gd);
            let v = h.r32(t + 8 * e + 4);
            h.fun_0200c278(x, v);
            break;
        }
    }
    h.fun_020307b0(t);
    0
}

/// C: `sub_021b7364`: command 564: `v`, `var`; `*var = TABLE_564[v < 0xB ? v : 0]`; 0.
pub fn cmd_564<H: PokemonFldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let v = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let i = if v < TABLE_564.len() as u32 { v } else { 0 };
    h.w16(var, TABLE_564[i as usize]);
    0
}

/// C: `FUN_overlay_d_21__021b7390`: `(month << 8 | day)` (u16) is one of the [`SPECIAL_DAYS`].
pub fn is_special_day(month: u32, day: u32) -> bool {
    let x = ((month << 8) | day) as u16;
    SPECIAL_DAYS.contains(&x)
}

/// C: `sub_021b73b8`: command 565: var; `*var` = 0xA on a [`is_special_day`] (month, day of `FUN_021690D0` /
/// `FUN_021690E0(GAMEDATA)`), else [`DAY_TABLE`]`[day % 10]` (signed remainder; Rust panics for a negative day); 0.
pub fn cmd_565<H: PokemonFldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let var = h.fun_02159ae8(vm, ctx);
    let gd = h.fun_0215a5e4(ctx);
    let m = h.fun_021690d0(gd);
    let d = h.fun_021690e0(gd);
    let v = if is_special_day(m, d) {
        0xA
    } else {
        u16::from(DAY_TABLE[(d as i32).wrapping_rem(10) as usize])
    };
    h.w16(var, v);
    0
}

/// Run the command handler at `handler` (bit 0 ignored) when it is one of [`COMMANDS`]; `None` otherwise (also for command
/// 494, X2).
pub fn dispatch<H: PokemonFldHooks>(h: &mut H, handler: u32, vm: u32, ctx: u32) -> Option<u32> {
    Some(match handler & !1 {
        0x021B_69B0 => cmd_308(h, vm, ctx),
        0x021B_6AE0 => cmd_459(h, vm, ctx),
        0x021B_6AFC => cmd_460(h, vm, ctx),
        0x021B_6B6C => cmd_461(h, vm, ctx),
        0x021B_6A78 => cmd_470(h, vm, ctx),
        0x021B_69D8 => cmd_471(h, vm, ctx),
        0x021B_716C => cmd_526(h, vm, ctx),
        0x021B_70D8 => cmd_527(h, vm, ctx),
        0x021B_7364 => cmd_564(h, vm, ctx),
        0x021B_73B8 => cmd_565(h, vm, ctx),
        0x021B_66B8 => cmd_581(h, vm, ctx),
        0x021B_721C => cmd_585(h, vm, ctx),
        0x021B_72CC => cmd_586(h, vm, ctx),
        _ => return None,
    })
}

#[cfg(test)]
mod tests;
