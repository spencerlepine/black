//! F8d: the unanchored overlay-10 script commands: F1-SPLIT rows 10.33 (Thumb 0x0216A420-0x0216AC40: commands 506..520,
//! 549 / 550 object moves, 559, the seasonal-form helper 0x0216AAE4) and 10.35 (0x0216B504-0x0216B86C: commands 528..543,
//! save-block accessors). 29 command ids, 32 functions. Command 359 at 0x0215D34C (the first function of `scrcmd_musical.c`,
//! musicals = X2, out of scope) is specced in the same spec but deliberately **not** ported nor dispatched.
//!
//! Every other function of the area is specced in `tools/rom-diff/specs/black-script/scrcmd_misc10.spec`, ported here, listed
//! in [`PORTED`] and checked with `tools/check-fn.sh black-script scrcmd_misc10`. [`COMMANDS`] maps the command ids to their
//! handlers, [`dispatch`] runs one by address and [`call_event`] the GMEVENT functions of commands 549 / 550.
//!
//! Callees outside the area are [`Misc10Hooks`] methods named by address (vm.c, the operand readers, the ctx getters, the
//! GMEVENT calls and the child-event call come from [`CoreHooks`]). The callees of 506..520 are `fld_btl_inst_tool.c` (row
//! 10.30) / overlay 21, those of 528..543 save-block accessors in main; their roles are UNVERIFIED (named by address).
//! ROM data the commands read is embedded and compared with the build by the tests: [`COUNTS_532`] (0x0217219C),
//! [`ROWS_532`] (0x021721BC); `FX_SinCosTable_` (main 0x020A1B38) is computed ([`fx_sin`] / [`fx_cos`]).

use crate::scrcmd_core::CoreHooks;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`). Not 0x0215D34C (command 359,
/// `scrcmd_musical.c`, X2).
pub const PORTED: &[u32] = &[
    0x0216_A420,
    0x0216_A440,
    0x0216_A470,
    0x0216_A4CC,
    0x0216_A510,
    0x0216_A560,
    0x0216_A594,
    0x0216_A5C4,
    0x0216_A5FC,
    0x0216_A634,
    0x0216_A674,
    0x0216_A6B4,
    0x0216_A6EC,
    0x0216_A72C,
    0x0216_A774,
    0x0216_A7A0,
    0x0216_A838,
    0x0216_A884,
    0x0216_A8F0,
    0x0216_AAE4,
    0x0216_ABD8,
    0x0216_B504,
    0x0216_B538,
    0x0216_B578,
    0x0216_B5AC,
    0x0216_B60C,
    0x0216_B63C,
    0x0216_B6DC,
    0x0216_B720,
    0x0216_B7B0,
    0x0216_B7F8,
    0x0216_B81C,
];

/// Command id -> handler address (Thumb entry, bit 0 clear) of the ported commands of this area (the words of the command
/// table 0x0217062C at index id; checked against the ROM by the tests). Command 359 (0x0215D34C) is left out (X2).
pub const COMMANDS: &[(u16, u32)] = &[
    (506, 0x0216_A420),
    (507, 0x0216_A440),
    (508, 0x0216_A470),
    (509, 0x0216_A4CC),
    (510, 0x0216_A510),
    (511, 0x0216_A560),
    (512, 0x0216_A594),
    (513, 0x0216_A5C4),
    (514, 0x0216_A5FC),
    (515, 0x0216_A634),
    (516, 0x0216_A674),
    (517, 0x0216_A6B4),
    (518, 0x0216_A6EC),
    (519, 0x0216_A72C),
    (520, 0x0216_A774),
    (528, 0x0216_B63C),
    (529, 0x0216_B6DC),
    (530, 0x0216_B578),
    (531, 0x0216_B5AC),
    (532, 0x0216_B720),
    (533, 0x0216_B81C),
    (535, 0x0216_B60C),
    (540, 0x0216_B7B0),
    (541, 0x0216_B7F8),
    (542, 0x0216_B504),
    (543, 0x0216_B538),
    (549, 0x0216_A7A0),
    (550, 0x0216_A884),
    (559, 0x0216_ABD8),
];

/// Command 359 (`scrcmd_musical.c`, X2): specced, not ported, not in [`COMMANDS`].
pub const CMD_359_MUSICAL: u32 = 0x0215_D34C;

/// GMEVENT function of command 549 ([`event_549`], Thumb).
pub const EVENT_549: u32 = 0x0216_A839;
/// GMEVENT function of command 550 ([`event_550`], Thumb).
pub const EVENT_550: u32 = 0x0216_A8F1;

/// Species with seasonal forms: Deerling, Sawsbuck.
pub const SPECIES_DEERLING: u32 = 0x249;
pub const SPECIES_SAWSBUCK: u32 = 0x24A;
/// PokeParam ids (`FUN_02017E1C(mon, id, 0)`).
pub const PP_SPECIES: u32 = 5;
pub const PP_FORM: u32 = 0x6F;

/// ROM address of [`COUNTS_532`].
pub const COUNTS_532_ADDR: u32 = 0x0217_219C;
/// Command 532: the number of rows of each category `c` (overlay-10 0x0217219C; 30 categories, the 2 bytes after them are 0
/// and kept so that `c` 30 / 31 read what the original reads).
pub const COUNTS_532: [u8; 32] = [
    2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 6, 6, 6, 8, 8, 11, 17, 11, 0,
    0,
];
/// ROM address of [`ROWS_532`].
pub const ROWS_532_ADDR: u32 = 0x0217_21BC;
/// Command 532: the value of row `k` of category `c` (overlay-10 0x021721BC, 30 rows of 17 bytes; UNVERIFIED meaning).
pub const ROWS_532: [[u8; 17]; 30] = [
    [1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [3, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [5, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [7, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [9, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [11, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [13, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [15, 16, 17, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [18, 19, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [21, 22, 23, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [24, 25, 26, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [27, 28, 29, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [30, 31, 32, 33, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [34, 35, 36, 37, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [38, 39, 40, 41, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [42, 43, 44, 45, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [46, 47, 48, 49, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [50, 51, 52, 53, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [54, 55, 56, 57, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [58, 59, 60, 61, 62, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [63, 64, 65, 66, 67, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [68, 69, 70, 71, 72, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [73, 74, 75, 76, 77, 78, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [79, 80, 81, 82, 83, 84, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [85, 86, 87, 88, 89, 90, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [91, 92, 93, 94, 95, 96, 97, 98, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [
        99, 100, 101, 102, 103, 104, 105, 106, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 0, 0, 0, 0, 0, 0,
    ],
    [
        118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 133, 134,
    ],
    [
        135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 0, 0, 0, 0, 0, 0,
    ],
];

/// `FX_SinCosTable_` sine entry `i` (ROM main `0x020A1B38`, 4096 `(sin, cos)` pairs of fx16): `round(4096 * sin(2 pi i /
/// 4096))`. Computed instead of embedded (as `black_field::fieldmap::fx_cos`); the tests check all 4096 ROM entries.
pub fn fx_sin(i: u32) -> i16 {
    (((i as f64) * std::f64::consts::TAU / 4096.0).sin() * 4096.0).round() as i16
}

/// `FX_SinCosTable_` cosine entry `i` (see [`fx_sin`]).
pub fn fx_cos(i: u32) -> i16 {
    (((i as f64) * std::f64::consts::TAU / 4096.0).cos() * 4096.0).round() as i16
}

/// Callees of the scrcmd_misc10 functions outside the area, named by address (meaning per the spec; UNVERIFIED unless
/// stated). vm.c, the operand readers, the scrcmd ctx getters, `FUN_020120F4` / `FUN_020122C0` / `FUN_020124E8` and the
/// child-event call come from [`CoreHooks`].
#[allow(clippy::too_many_arguments)]
pub trait Misc10Hooks: CoreHooks {
    // ---- main
    /// C: `FUN_0201210C(gamesys)`: the field map.
    fn fun_0201210c(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_02012EBC(gamedata)`: the save data.
    fn fun_02012ebc(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012F50(gamedata)`: address of the GAMEDATA word holding the `fld_btl_inst_tool` record (506 passes the
    /// address, the others the word).
    fn fun_02012f50(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02008C58(save)`: save block 0x26 (commands 513 / 514).
    fn fun_02008c58(&mut self, save: u32) -> u32;
    /// C: `FUN_02008E68(block, v)` (command 513).
    fn fun_02008e68(&mut self, block: u32, v: u32);
    /// C: `FUN_02008E78(block)` (command 514).
    fn fun_02008e78(&mut self, block: u32) -> u32;
    /// C: `FUN_0200C0E8(save)`: a save block (command 533).
    fn fun_0200c0e8(&mut self, save: u32) -> u32;
    /// C: `FUN_0200C0F0(save)`: save block 0x34.
    fn fun_0200c0f0(&mut self, save: u32) -> u32;
    /// C: `FUN_0200C224(block)` (commands 542 / 543: a level / count 0..).
    fn fun_0200c224(&mut self, block: u32) -> u32;
    /// C: `FUN_0200C22C(block, n)` (command 543: set it).
    fn fun_0200c22c(&mut self, block: u32, n: u32);
    /// C: `FUN_0200C32C(block)` (commands 530 / 533).
    fn fun_0200c32c(&mut self, block: u32) -> u32;
    /// C: `FUN_0200C334(block, v)` (commands 528 / 529).
    fn fun_0200c334(&mut self, block: u32, v: u32);
    /// C: `FUN_0200C33C(block, i)` (command 531).
    fn fun_0200c33c(&mut self, block: u32, i: u32) -> u32;
    /// C: `FUN_0200C344(block, i, v)` (commands 528 / 529).
    fn fun_0200c344(&mut self, block: u32, i: u32, v: u32);
    /// C: `FUN_0200C358(block, i, v)` (command 528).
    fn fun_0200c358(&mut self, block: u32, i: u32, v: u32);
    /// C: `FUN_0200C384(block, lo, hi)` (command 528: stores the 64-bit clock value).
    fn fun_0200c384(&mut self, block: u32, lo: u32, hi: u32);
    /// C: `FUN_0200E394(save)`: save block 0x3F.
    fn fun_0200e394(&mut self, save: u32) -> u32;
    /// C: `FUN_0200E3A0(block)` (command 540).
    fn fun_0200e3a0(&mut self, block: u32) -> u32;
    /// C: `FUN_0200E3E8(x, a, b)` (command 540).
    fn fun_0200e3e8(&mut self, x: u32, a: u32, b: u32);
    /// C: `FUN_0200E4B0(block, c, k)` (command 532).
    fn fun_0200e4b0(&mut self, block: u32, c: u32, k: u32) -> u32;
    /// C: `FUN_0200E4F8(block, c, k)` (command 532).
    fn fun_0200e4f8(&mut self, block: u32, c: u32, k: u32) -> u32;
    /// C: `FUN_02027100(x)` (command 540).
    fn fun_02027100(&mut self, x: u32);
    /// C: `FUN_02027138(n)` (command 543).
    fn fun_02027138(&mut self, n: u32);
    /// C: `FUN_0203F524()`: a 64-bit clock / tick value (command 528).
    fn fun_0203f524(&mut self) -> u64;
    /// C: `FUN_0200CA1C(x)`: the Pokedex object.
    fn fun_0200ca1c(&mut self, x: u32) -> u32;
    /// C: `FUN_0200CFA0(dex, mon)`: register the mon in the Pokedex.
    fn fun_0200cfa0(&mut self, dex: u32, mon: u32);
    /// C: `FUN_0201AA18(party)`: the party count.
    fn fun_0201aa18(&mut self, party: u32) -> u32;
    /// C: `FUN_0201AB28(party, i)`: the PokeParam of party slot `i`.
    fn fun_0201ab28(&mut self, party: u32, i: u32) -> u32;
    /// C: `FUN_02017E1C(mon, id, 0)`: a PokeParam value.
    fn fun_02017e1c(&mut self, mon: u32, id: u32, x: u32) -> u32;
    /// C: `FUN_02017988(mon, form)`: set the form.
    fn fun_02017988(&mut self, mon: u32, form: u32);

    // ---- overlay 10 (other files)
    /// C: `FUN_021670F8(gamesys)`: the child event of command 541.
    fn fun_021670f8(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_02168BF0(gamesys, rec, a, b, var)` (`fld_btl_inst_tool.c`): the child event of command 508.
    fn fun_02168bf0(&mut self, gamesys: u32, rec: u32, a: u32, b: u32, var: u32) -> u32;
    /// C: `FUN_02168CF4(gamesys, rec)` (`fld_btl_inst_tool.c`): the child event of command 511.
    fn fun_02168cf4(&mut self, gamesys: u32, rec: u32) -> u32;
    /// C: `FUN_02168E5C(gamesys, a, b)` (`fld_btl_inst_tool.c`): the child event of command 519.
    fn fun_02168e5c(&mut self, gamesys: u32, a: u32, b: u32) -> u32;
    /// C: `FUN_0216D568(mmdl, flags)` (fldmmdl.c; command 549 passes 0x8000).
    fn fun_0216d568(&mut self, mmdl: u32, flags: u32);
    /// C: `FUN_0216D88C(mmdl, out)` (fldmmdl.c): the object's position (VecFx32, 12 bytes) written to `out`.
    fn fun_0216d88c(&mut self, mmdl: u32, out: u32);
    /// C: `FUN_0216D89C(mmdl, &pos)` (fldmmdl.c): set the object's position (VecFx32).
    fn fun_0216d89c(&mut self, mmdl: u32, pos: &[u8; 12]);
    /// C: `FUN_0216DE94(mmdl sys, id)` (fldmmdl.c): the field object with OBJ id `id`, 0 if none.
    fn fun_0216de94(&mut self, sys: u32, id: u32) -> u32;
    /// C: `FUN_0216AED0(ctx, v)` (report_event.c): a lookup, 0 if none (command 559).
    fn fun_0216aed0(&mut self, ctx: u32, v: u32) -> u32;
    /// C: `FUN_0216AF1C(ctx, p, 4, &k)` (report_event.c): a lookup that also writes the word `k`, 0 if none.
    fn fun_0216af1c(&mut self, ctx: u32, p: u32, n: u32, k: &mut [u8; 4]) -> u32;
    /// C: `FUN_0216AF74(gamesys, field map, q, k, var, 4)` (report_event.c): the child event of command 559.
    fn fun_0216af74(&mut self, gamesys: u32, fm: u32, q: u32, k: u32, var: u32, n: u32) -> u32;
    /// C: `FUN_0216B8B8(save, var)` (command 533).
    fn fun_0216b8b8(&mut self, save: u32, var: u32);
    /// C: `FUN_0216B91C(save, v, var)` (command 533).
    fn fun_0216b91c(&mut self, save: u32, v: u32, var: u32);
    /// C: `FUN_0216BB18(save)` (command 535).
    fn fun_0216bb18(&mut self, save: u32) -> u32;

    // ---- overlay 21
    /// C: `FUN_02188C7C(field map)`: the field-object system.
    fn fun_02188c7c(&mut self, fm: u32) -> u32;
    /// C: `FUN_021D0A48(gamesys, &rec)` (command 506).
    fn fun_021d0a48(&mut self, gamesys: u32, rec: u32);
    /// C: `FUN_021D0A74(rec, v)` (command 507).
    fn fun_021d0a74(&mut self, rec: u32, v: u32);
    /// C: `FUN_021D0AB8(rec, v)` (command 512).
    fn fun_021d0ab8(&mut self, rec: u32, v: u32);
    /// C: `FUN_021D0AC0(gamesys, rec, v)` (command 509).
    fn fun_021d0ac0(&mut self, gamesys: u32, rec: u32, v: u32) -> u32;
    /// C: `FUN_021D0BD8(gamesys, rec, a, b)`: the child event of command 510.
    fn fun_021d0bd8(&mut self, gamesys: u32, rec: u32, a: u32, b: u32) -> u32;
    /// C: `FUN_021D0BE4(gamesys, rec, var)`: the child event of command 515.
    fn fun_021d0be4(&mut self, gamesys: u32, rec: u32, var: u32) -> u32;
    /// C: `FUN_021D0DB0(gamesys, rec, a, b)` (command 516; `a`, `b` var addresses).
    fn fun_021d0db0(&mut self, gamesys: u32, rec: u32, a: u32, b: u32);
    /// C: `FUN_021D0FE0(gamesys, rec)` (command 517).
    fn fun_021d0fe0(&mut self, gamesys: u32, rec: u32) -> u32;
    /// C: `FUN_021D0FE8(gamesys, rec, v)`: the child event of command 518.
    fn fun_021d0fe8(&mut self, gamesys: u32, rec: u32, v: u32) -> u32;
    /// C: `FUN_021D1028(gamesys)` (command 520).
    fn fun_021d1028(&mut self, gamesys: u32) -> u32;
}

// ---- 506..520: fld_btl_inst_tool.c / overlay 21

/// The GAMESYS and the `fld_btl_inst_tool` record word address (`FUN_02012F50(GAMEDATA)`), in the order the commands fetch them.
fn inst_record<H: Misc10Hooks>(h: &mut H, ctx: u32) -> (u32, u32) {
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    (gs, h.fun_02012f50(gd))
}

/// C: `sub_0216a420`: command 506: `FUN_021D0A48(GAMESYS, FUN_02012F50(GAMEDATA))`; 0.
pub fn cmd_506<H: Misc10Hooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let (gs, rec) = inst_record(h, ctx);
    h.fun_021d0a48(gs, rec);
    0
}

/// C: `sub_0216a440`: command 507: v = value; `FUN_021D0A74(*rec, v)`; 0.
pub fn cmd_507<H: Misc10Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let (_, rec) = inst_record(h, ctx);
    let v = h.fun_02159b10(vm, ctx);
    let r = h.r32(rec);
    h.fun_021d0a74(r, v);
    0
}

/// C: `sub_0216a470`: command 508: a, b = values, var; the event `FUN_02168BF0(GAMESYS, *rec, a, b, var)` runs as a child
/// event; 1.
pub fn cmd_508<H: Misc10Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let (gs, rec) = inst_record(h, ctx);
    let a = h.fun_02159b10(vm, ctx);
    let b = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let r = h.r32(rec);
    let ev = h.fun_02168bf0(gs, r, a, b, var);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_0216a4cc`: command 509: v = value, var; var = `FUN_021D0AC0(GAMESYS, *rec, v)`; 0.
pub fn cmd_509<H: Misc10Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let (gs, rec) = inst_record(h, ctx);
    let v = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let r = h.r32(rec);
    let x = h.fun_021d0ac0(gs, r, v);
    h.w16(var, x as u16);
    0
}

/// C: `sub_0216a510`: command 510: a, b = values; the event `FUN_021D0BD8(GAMESYS, *rec, a, b)` runs as a child event; 1.
pub fn cmd_510<H: Misc10Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let (gs, rec) = inst_record(h, ctx);
    let a = h.fun_02159b10(vm, ctx);
    let b = h.fun_02159b10(vm, ctx);
    let r = h.r32(rec);
    let ev = h.fun_021d0bd8(gs, r, a, b);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_0216a560`: command 511: the event `FUN_02168CF4(GAMESYS, *rec)` runs as a child event; 1.
pub fn cmd_511<H: Misc10Hooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let (gs, rec) = inst_record(h, ctx);
    let r = h.r32(rec);
    let ev = h.fun_02168cf4(gs, r);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_0216a594`: command 512: v = value; `FUN_021D0AB8(*rec, v)`; 0.
pub fn cmd_512<H: Misc10Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let (_, rec) = inst_record(h, ctx);
    let v = h.fun_02159b10(vm, ctx);
    let r = h.r32(rec);
    h.fun_021d0ab8(r, v);
    0
}

/// C: `sub_0216a5c4`: command 513: v = value; `FUN_02008E68(FUN_02008C58(SAVE), v & 0xFF)`; 0.
pub fn cmd_513<H: Misc10Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    let v = h.fun_02159b10(vm, ctx);
    let save = h.fun_02012ebc(gd);
    let b = h.fun_02008c58(save);
    h.fun_02008e68(b, v & 0xFF);
    0
}

/// C: `sub_0216a5fc`: command 514: var = `FUN_02008E78(FUN_02008C58(SAVE))`; 0.
pub fn cmd_514<H: Misc10Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    let var = h.fun_02159ae8(vm, ctx);
    let save = h.fun_02012ebc(gd);
    let b = h.fun_02008c58(save);
    let v = h.fun_02008e78(b);
    h.w16(var, v as u16);
    0
}

/// C: `sub_0216a634`: command 515: var; the event `FUN_021D0BE4(GAMESYS, *rec, var)` runs as a child event; 1.
pub fn cmd_515<H: Misc10Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let (gs, rec) = inst_record(h, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let r = h.r32(rec);
    let ev = h.fun_021d0be4(gs, r, var);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_0216a674`: command 516: vars a, b; `FUN_021D0DB0(GAMESYS, *rec, a, b)`; 0.
pub fn cmd_516<H: Misc10Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let (gs, rec) = inst_record(h, ctx);
    let a = h.fun_02159ae8(vm, ctx);
    let b = h.fun_02159ae8(vm, ctx);
    let r = h.r32(rec);
    h.fun_021d0db0(gs, r, a, b);
    0
}

/// C: `sub_0216a6b4`: command 517: var = `FUN_021D0FE0(GAMESYS, *rec)`; 0.
pub fn cmd_517<H: Misc10Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let (gs, rec) = inst_record(h, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let r = h.r32(rec);
    let v = h.fun_021d0fe0(gs, r);
    h.w16(var, v as u16);
    0
}

/// C: `sub_0216a6ec`: command 518: v = value; the event `FUN_021D0FE8(GAMESYS, *rec, v)` runs as a child event; 1.
pub fn cmd_518<H: Misc10Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let (gs, rec) = inst_record(h, ctx);
    let v = h.fun_02159b10(vm, ctx);
    let r = h.r32(rec);
    let ev = h.fun_021d0fe8(gs, r, v);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_0216a72c`: command 519: `FUN_02012F50(GAMEDATA)` (unused); a, b = values; the event `FUN_02168E5C(GAMESYS, a, b)`
/// runs as a child event; 1.
pub fn cmd_519<H: Misc10Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let (gs, _) = inst_record(h, ctx);
    let a = h.fun_02159b10(vm, ctx);
    let b = h.fun_02159b10(vm, ctx);
    let ev = h.fun_02168e5c(gs, a, b);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_0216a774`: command 520: var = `FUN_021D1028(GAMESYS)`; 0.
pub fn cmd_520<H: Misc10Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let v = h.fun_021d1028(gs);
    h.w16(var, v as u16);
    0
}

// ---- 549 / 550: field-object moves

/// C: `sub_0216a7a0`: command 549: id, d, n = values; the GMEVENT `FUN_020122C0(GAMESYS, 0, EVENT_549, 0x18)` with the work
/// W = {s32 dist, u16 n (+4), u16 0 (+6), MMDL (+8), VecFx32 start (+0xC)}: MMDL = the field object `id`, start = its position
/// (`FUN_0216D88C`), `FUN_0216D568(MMDL, 0x8000)`, dist = `(d << 15) - start.y`; runs as a child event; 1. (Moves the object
/// vertically to `d / 2` tiles over `n` frames: [`event_549`].)
pub fn cmd_549<H: Misc10Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5e4(ctx);
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let id = h.fun_02159b10(vm, ctx);
    let d = h.fun_02159b10(vm, ctx);
    let n = h.fun_02159b10(vm, ctx);
    let ev = h.fun_020122c0(gs, 0, EVENT_549, 0x18);
    let w = h.fun_020124e8(ev);
    h.w16(w + 4, n as u16);
    h.w16(w + 6, 0);
    let fm = h.fun_0201210c(gs);
    let sys = h.fun_02188c7c(fm);
    let mmdl = h.fun_0216de94(sys, id);
    h.fun_0216d88c(mmdl, w + 0xC);
    h.w32(w + 8, mmdl);
    h.fun_0216d568(mmdl, 0x8000);
    let y = h.r32(w + 0x10);
    h.w32(w, (d << 15).wrapping_sub(y));
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_0216a838`: the GMEVENT function of command 549 (`w` = {s32 dist, u16 frames, u16 done, MMDL, VecFx32 start}): one
/// frame of a vertical move: done += 1; y = start.y + dist * done / frames (`_s32_div_f`, truncating); set the object's
/// position; finished (1) once done >= frames.
pub fn event_549<H: Misc10Hooks>(h: &mut H, _event: u32, _seq: u32, w: u32) -> u32 {
    let done = h.r16(w + 6).wrapping_add(1);
    h.w16(w + 6, done);
    let mut pos = [0u8; 12];
    for (i, b) in pos.iter_mut().enumerate() {
        *b = h.r8(w + 0xC + i as u32);
    }
    let prod = (h.r16(w + 6) as u32).wrapping_mul(h.r32(w));
    let frames = h.r16(w + 4) as i32;
    let q = (prod as i32).wrapping_div(frames);
    let y = u32::from_le_bytes(pos[4..8].try_into().unwrap()).wrapping_add(q as u32);
    pos[4..8].copy_from_slice(&y.to_le_bytes());
    let mmdl = h.r32(w + 8);
    h.fun_0216d89c(mmdl, &pos);
    (h.r16(w + 6) >= h.r16(w + 4)) as u32
}

/// C: `sub_0216a884`: command 550: id = value; the GMEVENT `FUN_020122C0(GAMESYS, 0, EVENT_550, 0x14)` with the work W = {t 0,
/// MMDL (+4), VecFx32 pos (+8)}: MMDL = the field object `id`, pos = its position; runs as a child event; 1 ([`event_550`]).
pub fn cmd_550<H: Misc10Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5e4(ctx);
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let id = h.fun_02159b10(vm, ctx);
    let ev = h.fun_020122c0(gs, 0, EVENT_550, 0x14);
    let w = h.fun_020124e8(ev);
    h.w32(w, 0);
    let fm = h.fun_0201210c(gs);
    let sys = h.fun_02188c7c(fm);
    let mmdl = h.fun_0216de94(sys, id);
    h.fun_0216d88c(mmdl, w + 8);
    h.w32(w + 4, mmdl);
    h.fun_0215897c(sw, ev);
    1
}

/// The angle of a step: `_u32_div_f(n, d)` (`0x0209C2B0`, inlined) reduced modulo 0x10000 the way the compiler's signed
/// `% 0x10000` does it (only for quotients >= 0x10000, signed), then the table index `angle >> 4` (arithmetic).
fn step_index(n: u32, d: u32) -> u32 {
    let mut a = n / d;
    if (a as i32) >= 0x10000 {
        let s = a >> 31;
        a = s.wrapping_add((a << 16).wrapping_sub(s).rotate_right(16));
    }
    ((a as i32) >> 4) as u32
}

/// `((cos or sin entry) * r + 0x800) >> 12` in 64 bits (`_ll_mul` `0x0209C054`, inlined), low word: the fx32 product the
/// curved path adds to a coordinate.
fn fx_mul(t: i16, r: i64) -> u32 {
    ((t as i64).wrapping_mul(r).wrapping_add(0x800) as u64 >> 12) as u32
}

/// C: `FUN_overlay_d_10__0216a8f0`: the GMEVENT function of command 550 (`seq` = address of the sequence word, `w` = {t,
/// MMDL, VecFx32 pos}): a 3-phase curved path around the start position, one step per frame: t += 1; with the angle `a` and
/// radius `r` of the phase, pos.x += cos(a) * r and pos.z += sin(a) * r (`FX_SinCosTable_`, fx32 products rounded) on a copy of
/// the start position, which is set as the object's position. Phase 0: a = t * 0x20000 / 0x6E, r = 3t * 0x10000 / 0x6E
/// (growing), t >= 0x6E -> t = 0, phase 1; phase 1: a = t * 0x10000 / 0x50, r = 3.0, t >= 0x50 -> t = 0, phase 2; phase 2:
/// a as phase 1, r = (0x50 - t) * 3 * 0x10000 / 0x50 (shrinking), finished (1) once t >= 0x50. Other phases: nothing (0).
pub fn event_550<H: Misc10Hooks>(h: &mut H, _event: u32, seq: u32, w: u32) -> u32 {
    let mut pos = [0u8; 12];
    for (i, b) in pos.iter_mut().enumerate() {
        *b = h.r8(w + 8 + i as u32);
    }
    let t = h.r32(w).wrapping_add(1);
    h.w32(w, t);
    let phase = h.r32(seq);
    let (i, r): (u32, i64) = match phase {
        0 => {
            let i = step_index(t << 17, 0x6E);
            let r = (t.wrapping_mul(3) << 16) / 0x6E;
            (i, r as i32 as i64)
        }
        1 => (step_index(t << 16, 0x50), 0x30000),
        2 => {
            let i = step_index(t << 16, 0x50);
            let r = (0x50u32.wrapping_sub(t).wrapping_mul(3) << 16) / 0x50;
            (i, r as i32 as i64)
        }
        _ => return 0,
    };
    let x = u32::from_le_bytes(pos[0..4].try_into().unwrap()).wrapping_add(fx_mul(fx_cos(i), r));
    pos[0..4].copy_from_slice(&x.to_le_bytes());
    let z = u32::from_le_bytes(pos[8..12].try_into().unwrap()).wrapping_add(fx_mul(fx_sin(i), r));
    pos[8..12].copy_from_slice(&z.to_le_bytes());
    let mmdl = h.r32(w + 4);
    h.fun_0216d89c(mmdl, &pos);
    let t = h.r32(w);
    match phase {
        0 => {
            if t >= 0x6E {
                h.w32(w, 0);
                h.w32(seq, 1);
            }
            0
        }
        1 => {
            if t >= 0x50 {
                h.w32(w, 0);
                h.w32(seq, 2);
            }
            0
        }
        _ => (t >= 0x50) as u32,
    }
}

/// C: `FUN_0216AAE4`: the seasonal forms (`season` 0..3, anything else = 0): every party Deerling / Sawsbuck
/// whose form differs gets the season's form, and the first one of each species changed is registered in the Pokedex of `x`.
pub fn update_season_forms<H: Misc10Hooks>(h: &mut H, x: u32, party: u32, season: u32) {
    let dex = h.fun_0200ca1c(x);
    let form = if season <= 3 { season } else { 0 };
    let (mut seen_deerling, mut seen_sawsbuck) = (false, false);
    let n = h.fun_0201aa18(party) as i32;
    let mut i = 0i32;
    while i < n {
        let mon = h.fun_0201ab28(party, i as u32);
        let species = h.fun_02017e1c(mon, PP_SPECIES, 0);
        let f = h.fun_02017e1c(mon, PP_FORM, 0);
        let seen = if species == SPECIES_DEERLING && f != form {
            Some(&mut seen_deerling)
        } else if species == SPECIES_SAWSBUCK && f != form {
            Some(&mut seen_sawsbuck)
        } else {
            None
        };
        if let Some(seen) = seen {
            h.fun_02017988(mon, form & 0xFFFF);
            if !*seen {
                h.fun_0200cfa0(dex, mon);
                *seen = true;
            }
        }
        i += 1;
    }
}

/// C: `sub_0216abd8`: command 559: v = value, var; p = `FUN_0216AED0(ctx, v)`; when p and q = `FUN_0216AF1C(ctx, p, 4, &k)`
/// are nonzero, the event `FUN_0216AF74(GAMESYS, field map, q, k, var, 4)` (report_event.c) runs as a child event; 1.
pub fn cmd_559<H: Misc10Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let v = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let p = h.fun_0216aed0(ctx, v);
    if p == 0 {
        return 1;
    }
    let mut k = [0u8; 4];
    let q = h.fun_0216af1c(ctx, p, 4, &mut k);
    if q == 0 {
        return 1;
    }
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    let ev = h.fun_0216af74(gs, fm, q, u32::from_le_bytes(k), var, 4);
    h.fun_0215897c(sw, ev);
    1
}

// ---- 528..543: save blocks 0x34 / 0x3F

/// The save data of the ctx (`FUN_02012EBC(GAMEDATA)`), after the ctx's SCRIPT_WORK fetch every one of these commands makes
/// (unused).
fn save_of<H: Misc10Hooks>(h: &mut H, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let gd = h.fun_0215a5e4(ctx);
    h.fun_02012ebc(gd)
}

/// C: `sub_0216b504`: command 542: var = `FUN_0200C224(block 0x34)`; 0.
pub fn cmd_542<H: Misc10Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let save = save_of(h, ctx);
    let s = h.fun_0200c0f0(save);
    let var = h.fun_02159ae8(vm, ctx);
    let v = h.fun_0200c224(s);
    h.w16(var, v as u16);
    0
}

/// C: `sub_0216b538`: command 543: with the save block 0x34 `s`: n = `FUN_0200C224(s)`; n < 5 (signed) -> `FUN_0200C22C(s,
/// n + 1)` and `FUN_02027138((n + 1) & 0xFF)`; 0.
pub fn cmd_543<H: Misc10Hooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let save = save_of(h, ctx);
    let s = h.fun_0200c0f0(save);
    let n = h.fun_0200c224(s);
    if (n as i32) < 5 {
        h.fun_0200c22c(s, n.wrapping_add(1));
        h.fun_02027138(n.wrapping_add(1) & 0xFF);
    }
    0
}

/// C: `sub_0216b578`: command 530: var = `FUN_0200C32C(block 0x34)`; 0.
pub fn cmd_530<H: Misc10Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let save = save_of(h, ctx);
    let s = h.fun_0200c0f0(save);
    let var = h.fun_02159ae8(vm, ctx);
    let v = h.fun_0200c32c(s);
    h.w16(var, v as u16);
    0
}

/// C: `sub_0216b5ac`: command 531: three vars = `FUN_0200C33C(s, 0 / 1 / 2)` of the save block 0x34; 0.
pub fn cmd_531<H: Misc10Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let save = save_of(h, ctx);
    let s = h.fun_0200c0f0(save);
    let vars = [
        h.fun_02159ae8(vm, ctx),
        h.fun_02159ae8(vm, ctx),
        h.fun_02159ae8(vm, ctx),
    ];
    for (i, var) in vars.into_iter().enumerate() {
        let v = h.fun_0200c33c(s, i as u32);
        h.w16(var, v as u16);
    }
    0
}

/// C: `sub_0216b60c`: command 535: var = `FUN_0216BB18(SAVE)`; 0.
pub fn cmd_535<H: Misc10Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let save = save_of(h, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let v = h.fun_0216bb18(save);
    h.w16(var, v as u16);
    0
}

/// C: `sub_0216b63c`: command 528: k, a0, a1, a2 = values; with the save block 0x34 `s` (and block 0x3F fetched, unused):
/// `FUN_0200C334(s, k & 0xFF)`; for i 0..2 `FUN_0200C344(s, i, a_i & 0xFF)` and `FUN_0200C358(s, i, 0)`; then
/// `FUN_0200C384(s, clock)` with the 64-bit `FUN_0203F524()`; 0.
pub fn cmd_528<H: Misc10Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let save = save_of(h, ctx);
    let s = h.fun_0200c0f0(save);
    h.fun_0200e394(save);
    let k = h.fun_02159b10(vm, ctx);
    let a = [
        h.fun_02159b10(vm, ctx) as u16,
        h.fun_02159b10(vm, ctx) as u16,
        h.fun_02159b10(vm, ctx) as u16,
    ];
    h.fun_0200c334(s, k & 0xFF);
    for (i, v) in a.into_iter().enumerate() {
        h.fun_0200c344(s, i as u32, (v & 0xFF) as u32);
        h.fun_0200c358(s, i as u32, 0);
    }
    let t = h.fun_0203f524();
    h.fun_0200c384(s, t as u32, (t >> 32) as u32);
    0
}

/// C: `sub_0216b6dc`: command 529: with the save block 0x34 `s` (and block 0x3F fetched, unused): `FUN_0200C334(s, 0)`,
/// `FUN_0200C344(s, i, 0xFF)` for i 0..2; 0.
pub fn cmd_529<H: Misc10Hooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let save = save_of(h, ctx);
    let s = h.fun_0200c0f0(save);
    h.fun_0200e394(save);
    h.fun_0200c334(s, 0);
    for i in 0..3 {
        h.fun_0200c344(s, i, 0xFF);
    }
    0
}

/// C: `sub_0216b720`: command 532: c = value, var; with the save block 0x3F `e`: over the rows k = 1..[`COUNTS_532`]`[c]` the
/// first maximum (signed, from -1) of `FUN_0200E4F8(e, c, k) + FUN_0200E4B0(e, c, k)` picks [`ROWS_532`]`[c][k - 1]`; var =
/// that byte; 0. An original quirk, kept: when no row wins (no rows, or every sum below 0) var gets an uninitialised stack
/// word of the function (`stale`; [`dispatch`] passes 0, the Z19 entry value the differential test runs with). `c` past the
/// embedded tables panics where the original reads the following bytes.
pub fn cmd_532<H: Misc10Hooks>(h: &mut H, vm: u32, ctx: u32, stale: u32) -> u32 {
    let save = save_of(h, ctx);
    let e = h.fun_0200e394(save);
    let c = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let n = COUNTS_532[c as usize] as u32;
    let mut best: i32 = -1;
    let mut pick = stale;
    for k in 0..n {
        let a = h.fun_0200e4f8(e, c, k + 1);
        let b = h.fun_0200e4b0(e, c, k + 1);
        let s = a.wrapping_add(b) as i32;
        if best < s {
            best = s;
            pick = ROWS_532[c as usize][k as usize] as u32;
        }
    }
    h.w16(var, pick as u16);
    0
}

/// C: `sub_0216b7b0`: command 540: x = `FUN_0200E3A0(block 0x3F)`; a, b = values; `FUN_0200E3E8(x, a & 0xFF, b)`,
/// `FUN_02027100(x)`; 0.
pub fn cmd_540<H: Misc10Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let save = save_of(h, ctx);
    let e = h.fun_0200e394(save);
    let x = h.fun_0200e3a0(e);
    let a = h.fun_02159b10(vm, ctx);
    let b = h.fun_02159b10(vm, ctx);
    h.fun_0200e3e8(x, a & 0xFF, b);
    h.fun_02027100(x);
    0
}

/// C: `sub_0216b7f8`: command 541: the event `FUN_021670F8(GAMESYS)` runs as a child event; 1.
pub fn cmd_541<H: Misc10Hooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let ev = h.fun_021670f8(gs);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_0216b81c`: command 533: v = value & 0xFF, var; with s = `FUN_0200C0E8(SAVE)`: v == `FUN_0200C32C(s)` ->
/// `FUN_0216B8B8(SAVE, var)`, else `FUN_0216B91C(SAVE, v, var)`; 0.
pub fn cmd_533<H: Misc10Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let save = h.fun_02012ebc(gd);
    let s = h.fun_0200c0e8(save);
    let v = h.fun_02159b10(vm, ctx) & 0xFF;
    let var = h.fun_02159ae8(vm, ctx);
    if v == h.fun_0200c32c(s) {
        h.fun_0216b8b8(save, var);
    } else {
        h.fun_0216b91c(save, v, var);
    }
    0
}

// ---- dispatch

/// Run the command handler at `handler` (bit 0 ignored) when it is one of [`COMMANDS`]; `None` otherwise (also for command
/// 359, which is not ported). Command 532's uninitialised stack word is passed as 0 (see [`cmd_532`]).
pub fn dispatch<H: Misc10Hooks>(h: &mut H, handler: u32, vm: u32, ctx: u32) -> Option<u32> {
    Some(match handler & !1 {
        0x0216_A420 => cmd_506(h, vm, ctx),
        0x0216_A440 => cmd_507(h, vm, ctx),
        0x0216_A470 => cmd_508(h, vm, ctx),
        0x0216_A4CC => cmd_509(h, vm, ctx),
        0x0216_A510 => cmd_510(h, vm, ctx),
        0x0216_A560 => cmd_511(h, vm, ctx),
        0x0216_A594 => cmd_512(h, vm, ctx),
        0x0216_A5C4 => cmd_513(h, vm, ctx),
        0x0216_A5FC => cmd_514(h, vm, ctx),
        0x0216_A634 => cmd_515(h, vm, ctx),
        0x0216_A674 => cmd_516(h, vm, ctx),
        0x0216_A6B4 => cmd_517(h, vm, ctx),
        0x0216_A6EC => cmd_518(h, vm, ctx),
        0x0216_A72C => cmd_519(h, vm, ctx),
        0x0216_A774 => cmd_520(h, vm, ctx),
        0x0216_B63C => cmd_528(h, vm, ctx),
        0x0216_B6DC => cmd_529(h, vm, ctx),
        0x0216_B578 => cmd_530(h, vm, ctx),
        0x0216_B5AC => cmd_531(h, vm, ctx),
        0x0216_B720 => cmd_532(h, vm, ctx, 0),
        0x0216_B81C => cmd_533(h, vm, ctx),
        0x0216_B60C => cmd_535(h, vm, ctx),
        0x0216_B7B0 => cmd_540(h, vm, ctx),
        0x0216_B7F8 => cmd_541(h, vm, ctx),
        0x0216_B504 => cmd_542(h, vm, ctx),
        0x0216_B538 => cmd_543(h, vm, ctx),
        0x0216_A7A0 => cmd_549(h, vm, ctx),
        0x0216_A884 => cmd_550(h, vm, ctx),
        0x0216_ABD8 => cmd_559(h, vm, ctx),
        _ => return None,
    })
}

/// Run the GMEVENT function `f(event, seq, work)` when it is [`EVENT_549`] / [`EVENT_550`]; `None` otherwise.
pub fn call_event<H: Misc10Hooks>(
    h: &mut H,
    f: u32,
    event: u32,
    seq: u32,
    work: u32,
) -> Option<u32> {
    match f {
        EVENT_549 => Some(event_549(h, event, seq, work)),
        EVENT_550 => Some(event_550(h, event, seq, work)),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
