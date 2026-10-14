//! F8f (fifth batch): script commands in overlay-20 / overlay-21 files that are not scrcmd files: the WFBC (Black City / White
//! Forest) commands 450..457 at the end of overlay-20 `field_wfbc_data_local.c` (+ the counter accessors they use), the
//! field-skill commands 197..200 after `sodateya.c`, `fld_wipe_3dobj.c` 415..417 / 576, `pleasure_boat.c` 421..424 / 426 / 428
//! (+ the boat-work helpers) and `fld_trade.c` 443 (39 functions, 23 command ids; the WFBC talk dialog 0x021871F8 is UI flow,
//! left out).
//!
//! Every function of the area is specced in `tools/rom-diff/specs/black-script/scrcmd_misc21b.spec`, ported here, listed in
//! [`PORTED`] and checked with `tools/check-fn.sh black-script scrcmd_misc21b`. [`COMMANDS`] maps the command ids to their
//! handlers, [`dispatch`] runs one by address and [`call_wait`] the wait callbacks of commands 197 / 198.
//!
//! Callees outside the area are [`Misc21bHooks`] methods named by address (vm.c, the operand readers, the ctx getters and the
//! GMEVENT calls come from [`CoreHooks`]); the WFBC talk dialog (the GMEVENT 0x021871F8 of command 456, UI flow, D3) is not
//! ported: command 456 only creates it. Roles are UNVERIFIED (named by address).

use crate::hooks::MemEnv;
use crate::scrcmd_core::CoreHooks;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0218_67A8,
    0x0218_67B4,
    0x0218_69AC,
    0x0218_69B0,
    0x0218_69B8,
    0x0218_69BC,
    0x0218_69D0,
    0x0218_69D8,
    0x0218_6BE4,
    0x0218_6C44,
    0x0218_6CC4,
    0x0218_6D48,
    0x0218_6E94,
    0x0218_6EB8,
    0x0218_7000,
    0x0218_7098,
    0x0218_70C4,
    0x021C_457C,
    0x021C_459C,
    0x021C_4608,
    0x021C_4628,
    0x021C_4678,
    0x021C_46A4,
    0x021C_812C,
    0x021C_8170,
    0x021C_81B4,
    0x021C_81F8,
    0x021C_8608,
    0x021C_86E8,
    0x021C_8738,
    0x021C_87D4,
    0x021C_8810,
    0x021C_881C,
    0x021C_8848,
    0x021C_8868,
    0x021C_88AC,
    0x021C_88EC,
    0x021C_8960,
    0x021C_C544,
];

/// Command id -> handler address (Thumb entry, bit 0 clear) of the commands of this area (the words of the command table
/// 0x0217062C at index id; checked against the ROM by the tests).
pub const COMMANDS: &[(u16, u32)] = &[
    (197, 0x021C_459C),
    (198, 0x021C_4628),
    (199, 0x021C_4678),
    (200, 0x021C_46A4),
    (415, 0x021C_812C),
    (416, 0x021C_8170),
    (417, 0x021C_81B4),
    (421, 0x021C_881C),
    (422, 0x021C_8848),
    (423, 0x021C_8868),
    (424, 0x021C_88AC),
    (426, 0x021C_88EC),
    (428, 0x021C_8960),
    (443, 0x021C_C544),
    (450, 0x0218_6BE4),
    (451, 0x0218_6C44),
    (452, 0x0218_6D48),
    (453, 0x0218_6CC4),
    (454, 0x0218_6EB8),
    (455, 0x0218_6E94),
    (456, 0x0218_7098),
    (457, 0x0218_7000),
    (576, 0x021C_81F8),
];

/// The wait callbacks of commands 197 / 198 (VM +0x10, Thumb).
pub const WAIT_197: u32 = 0x021C_457D;
pub const WAIT_198: u32 = 0x021C_4609;
/// The WFBC talk dialog GMEVENT of command 456 (UI flow, not ported) and its work size.
pub const EVENT_WFBC_TALK: u32 = 0x0218_71F9;
pub const WFBC_TALK_SIZE: u32 = 0x58;

/// WFBC counter b is raised in steps of 10 while it is below this cap (`FUN_021869BC`).
pub const WFBC_COUNT_B_CAP: u16 = 10000;
/// Records of the pleasure-boat work (0x1C bytes each from W + 0x24).
pub const BOAT_RECORDS: u32 = 15;
/// Size of a pleasure-boat record.
pub const BOAT_RECORD_SIZE: u32 = 0x1C;
/// The pleasure-boat work: size, heap, `__FILE__` / line of its allocation, its overlay.
pub const BOAT_SIZE: u32 = 0x1B8;
pub const BOAT_HEAP: u32 = 5;
pub const FILE_BOAT: u32 = 0x021D_E534;
pub const LINE_BOAT: u32 = 0x31;
pub const OVERLAY_BOAT: u32 = 36;
/// One boat-time unit (s16 W+4 counts in these).
pub const BOAT_TIME_UNIT: i32 = 0x546;
/// The 17 `{u16 key, u16 kind}` pairs of command 576 (0x021DAAE0, overlay 21 ROM).
pub const WIPE_KEYS_ADDR: u32 = 0x021D_AAE0;
pub const WIPE_KEYS: [(u16, u16); 17] = [
    (0x268, 0x00),
    (0x269, 0x1A),
    (0x26E, 0x1B),
    (0x1D2, 0x1C),
    (0x1B5, 0x01),
    (0x1BA, 0x1D),
    (0x1BF, 0x1E),
    (0x1C2, 0x1F),
    (0x1D1, 0x20),
    (0x1D7, 0x21),
    (0x26D, 0x34),
    (0x272, 0x23),
    (0x242, 0x24),
    (0x26F, 0x25),
    (0x270, 0x26),
    (0x271, 0x27),
    (0x243, 0x28),
];

/// Callees of the scrcmd_misc21b functions outside the area, named by address (meaning per the spec; UNVERIFIED unless
/// stated). vm.c, the operand readers, the ctx getters and the GMEVENT calls come from [`CoreHooks`].
#[allow(clippy::too_many_arguments)]
pub trait Misc21bHooks: CoreHooks {
    // ---- main
    /// C: `FUN_0201210C(gamesys)`: the field map.
    fn fun_0201210c(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_02012138(gamesys)`.
    fn fun_02012138(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_020127C4(gamedata)`: the WFBC data.
    fn fun_020127c4(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_020127D0(gamedata, k)`.
    fn fun_020127d0(&mut self, gamedata: u32, k: u32) -> u32;
    /// C: `FUN_020127F0(gamedata)`: the WFBC counters (u16 a at +0, u16 b at +2).
    fn fun_020127f0(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012934(gamedata)`: the party.
    fn fun_02012934(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012ED0(gamedata)`: the boat work slot (a pointer to W).
    fn fun_02012ed0(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02017E1C(mon, id, x)`: a PokeParam value.
    fn fun_02017e1c(&mut self, mon: u32, id: u32, x: u32) -> u32;
    /// C: `FUN_0201AA18(party)`: the party count.
    fn fun_0201aa18(&mut self, party: u32) -> u32;
    /// C: `FUN_0201AB28(party, i)`: party mon `i`.
    fn fun_0201ab28(&mut self, party: u32, i: u32) -> u32;
    /// C: `FUN_0201EC64(heap)`: a word set.
    fn fun_0201ec64(&mut self, heap: u32) -> u32;
    /// C: `FUN_02030734(heap, size, flag, file, line)` (Heap_AllocDebug).
    fn fun_02030734(&mut self, heap: u32, size: u32, flag: u32, file: u32, line: u32) -> u32;
    /// C: `FUN_02034A5C(id)`: unload an overlay.
    fn fun_02034a5c(&mut self, id: u32);
    /// C: `FUN_02034AC4(id)`: load an overlay.
    fn fun_02034ac4(&mut self, id: u32);
    /// C: `FUN_0203F4E4(&date)`: the date (year, month, day words).
    fn fun_0203f4e4(&mut self, date: &mut [u32; 3]);
    /// C: `FUN_020457B0(len, heap)`: a string buffer.
    fn fun_020457b0(&mut self, len: u32, heap: u32) -> u32;
    /// C: `FUN_02045B38(a, b, c, heap)`: a message data.
    fn fun_02045b38(&mut self, a: u32, b: u32, c: u32, heap: u32) -> u32;
    // ---- script_sys / overlay 10
    /// C: `FUN_02158344(x)`: the intrude comm state or null (`delivery_irc`, X1).
    fn fun_02158344(&mut self, x: u32) -> u32;
    /// C: `FUN_02158434()`: `r0` is the value the original leaves in `r0` (the comm state; the spec logs it under command 456).
    fn fun_02158434(&mut self, r0: u32) -> u32;
    /// C: `FUN_02158F58(sw)`: a record whose word +4 is the field map.
    fn fun_02158f58(&mut self, sw: u32) -> u32;
    /// C: `FUN_02158F74(sw)`: the word set.
    fn fun_02158f74(&mut self, sw: u32) -> u32;
    /// C: `FUN_0215A5CC(ctx)`: the heap id of the script.
    fn fun_0215a5cc(&mut self, ctx: u32) -> u32;
    /// C: `FUN_0215A644(ctx, task)`: set the ctx's field-skill task.
    fn fun_0215a644(&mut self, ctx: u32, task: u32);
    /// C: `FUN_0215A64C(ctx)`: the ctx's field-skill task.
    fn fun_0215a64c(&mut self, ctx: u32) -> u32;
    /// C: `FUN_021666A4(slot)` (symbol_map.c / boat): free the boat work.
    fn fun_021666a4(&mut self, slot: u32);
    /// C: `FUN_0216C204(gamesys, fm)` (namein_setup.c): the name-input event of a trade.
    fn fun_0216c204(&mut self, gamesys: u32, fm: u32) -> u32;
    /// C: `FUN_0216D588(obj)`.
    fn fun_0216d588(&mut self, obj: u32) -> u32;
    /// C: `FUN_0216D5F8(obj)`: the direction.
    fn fun_0216d5f8(&mut self, obj: u32) -> u32;
    /// C: `FUN_0216D88C(obj, &pos)`: the position.
    fn fun_0216d88c(&mut self, obj: u32, pos: u32);
    /// C: `FUN_0216DDC0(objsys, x, z, k)`: the OBJ at a tile or null.
    fn fun_0216ddc0(&mut self, objsys: u32, x: u32, z: u32, k: u32) -> u32;
    // ---- overlay 20 field_wfbc_data_local.c / overlay 21
    /// C: `FUN_02186090(obj)` (ov20): the WFBC person record of an OBJ.
    fn fun_02186090(&mut self, obj: u32) -> u32;
    /// C: `FUN_02186278(data, k)` (ov20).
    fn fun_02186278(&mut self, data: u32, k: u32) -> u32;
    /// C: `FUN_02188C54(fm)`.
    fn fun_02188c54(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188C7C(fm)`: the field OBJ system.
    fn fun_02188c7c(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188C80(fm)`: the GAMESYS.
    fn fun_02188c80(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188C84(fm)`: the field heap id.
    fn fun_02188c84(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188C98(fm)`.
    fn fun_02188c98(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188CB4(fm)`: the player.
    fn fun_02188cb4(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188CD0(fm)`.
    fn fun_02188cd0(&mut self, fm: u32) -> u32;
    /// C: `FUN_02189A00(fm)`: the 3D-object system.
    fn fun_02189a00(&mut self, fm: u32) -> u32;
    /// C: `FUN_0218DC70(x)` (field_wfbc.c): the WFBC object E.
    fn fun_0218dc70(&mut self, x: u32) -> u32;
    /// C: `FUN_021938F4(E)` (thunk of 0x02193D08).
    fn fun_021938f4(&mut self, e: u32) -> u32;
    /// C: `FUN_02193904(E)`.
    fn fun_02193904(&mut self, e: u32);
    /// C: `FUN_0219390C(E, rec)`: the person of a record or null.
    fn fun_0219390c(&mut self, e: u32, rec: u32) -> u32;
    /// C: `FUN_0219391C(E)`: a flag.
    fn fun_0219391c(&mut self, e: u32) -> u32;
    /// C: `FUN_02193B40(E, rec)`.
    fn fun_02193b40(&mut self, e: u32, rec: u32);
    /// C: `FUN_02193B68(E, ws, rec, v, heap)`.
    fn fun_02193b68(&mut self, e: u32, ws: u32, rec: u32, v: u32, heap: u32);
    /// C: `FUN_02193BC4(E, rec)`.
    fn fun_02193bc4(&mut self, e: u32, rec: u32);
    /// C: `FUN_02193BE4(E, rec)`: a flag.
    fn fun_02193be4(&mut self, e: u32, rec: u32) -> u32;
    /// C: `FUN_02193C00()`: a record.
    fn fun_02193c00(&mut self) -> u32;
    /// C: `FUN_02193C04(person)`: its record.
    fn fun_02193c04(&mut self, person: u32) -> u32;
    /// C: `FUN_021A3050(player)`: the direction.
    fn fun_021a3050(&mut self, player: u32) -> u32;
    /// C: `FUN_021A3074(player)`: the player OBJ.
    fn fun_021a3074(&mut self, player: u32) -> u32;
    /// C: `FUN_021A3364(player, dir, &pos)`: the position in front.
    fn fun_021a3364(&mut self, player: u32, dir: u32, pos: &mut [u32; 3]);
    /// C: `FUN_021A3460(player, &x, &y, &z)`: the tile in front (x: s16 in a word, y / z: u16).
    fn fun_021a3460(&mut self, player: u32, x: &mut [u8; 4], y: &mut [u8; 2], z: &mut [u8; 2]);
    /// C: `FUN_021AB0D4(map, &pos)`.
    fn fun_021ab0d4(&mut self, map: u32, pos: &[u32; 3]) -> u32;
    /// C: `FUN_021C1024(player, dir, x, heap)` (field_player_grid_event.c): a field-skill task.
    fn fun_021c1024(&mut self, player: u32, dir: u32, x: u32, heap: u32) -> u32;
    /// C: `FUN_021C1094(task)`: 1 when done.
    fn fun_021c1094(&mut self, task: u32) -> u32;
    /// C: `FUN_021C10A0(task)`: free it.
    fn fun_021c10a0(&mut self, task: u32);
    /// C: `FUN_021C1228(player, dir, v, heap)`: a field-skill task.
    fn fun_021c1228(&mut self, player: u32, dir: u32, v: u32, heap: u32) -> u32;
    /// C: `FUN_021C12C4(task)`: 1 when done.
    fn fun_021c12c4(&mut self, task: u32) -> u32;
    /// C: `FUN_021C12D0(task)`: free it.
    fn fun_021c12d0(&mut self, task: u32);
    /// C: `FUN_021C1CB0(gs, x, k)` (fld3d_ci.c): a wipe event or null.
    fn fun_021c1cb0(&mut self, gamesys: u32, x: u32, k: u32) -> u32;
    /// C: `FUN_021C1D68(gs, x, k)` (fld3d_ci.c): a wipe event or null.
    fn fun_021c1d68(&mut self, gamesys: u32, x: u32, k: u32) -> u32;
    /// C: `FUN_021C30B8(gs, x, k)` (fld3d_ci.c): a wipe event or null.
    fn fun_021c30b8(&mut self, gamesys: u32, x: u32, k: u32) -> u32;
    /// C: `FUN_021C9550(obj, x)` (fldeff_iaigiri.c).
    fn fun_021c9550(&mut self, obj: u32, x: u32);
    /// C: `FUN_021D2418(gs, fm)`: an event.
    fn fun_021d2418(&mut self, gamesys: u32, fm: u32) -> u32;
    /// C: `FUN_021D2440(gs, fm, k)`: an event.
    fn fun_021d2440(&mut self, gamesys: u32, fm: u32, k: u32) -> u32;
    /// C: `FUN_021F36C0(W)` (overlay 36): init the boat work.
    fn fun_021f36c0(&mut self, w: u32);
}

// ------------------------------------------------------------------ overlay 20: WFBC

/// C: `FUN_overlay_d_20__021867a8`: the high nibble of the record's byte 0.
pub fn wfbc_rec_hi4<H: MemEnv>(h: &mut H, rec: u32) -> u8 {
    h.r8(rec) >> 4
}

/// C: `FUN_overlay_d_20__021867b4`: the record's byte 2 is at most 0x14.
pub fn wfbc_rec_near<H: MemEnv>(h: &mut H, rec: u32) -> bool {
    h.r8(rec + 2) <= 0x14
}

/// C: `FUN_overlay_d_20__021869ac`: counter a (u16 +0).
pub fn wfbc_count_a<H: MemEnv>(h: &mut H, c: u32) -> u16 {
    h.r16(c)
}

/// C: `FUN_overlay_d_20__021869b0`: counter a += 1 (u16, wrapping).
pub fn wfbc_count_a_inc<H: MemEnv>(h: &mut H, c: u32) {
    let n = h.r16(c);
    h.w16(c, n.wrapping_add(1));
}

/// C: `FUN_overlay_d_20__021869b8`: counter b (u16 +2).
pub fn wfbc_count_b<H: MemEnv>(h: &mut H, c: u32) -> u16 {
    h.r16(c + 2)
}

/// C: `FUN_overlay_d_20__021869bc`: counter b += 10 while it is below 10000.
pub fn wfbc_count_b_add10<H: MemEnv>(h: &mut H, c: u32) {
    let n = h.r16(c + 2);
    if n < WFBC_COUNT_B_CAP {
        h.w16(c + 2, n + 10);
    }
}

/// C: `FUN_overlay_d_20__021869d0`: counter b again (a second accessor of u16 +2).
pub fn wfbc_count_b2<H: MemEnv>(h: &mut H, c: u32) -> u16 {
    h.r16(c + 2)
}

/// C: `FUN_overlay_d_20__021869d8`: counter a again (a second accessor of u16 +0).
pub fn wfbc_count_a2<H: MemEnv>(h: &mut H, c: u32) -> u16 {
    h.r16(c)
}

/// The OBJ on the tile in front of the player (`FUN_021A3460` -> `FUN_0216DDC0(objsys, x, z, 0)`).
fn front_obj<H: Misc21bHooks>(h: &mut H, player: u32, objsys: u32) -> u32 {
    let mut x = [0u8; 4];
    let mut y = [0u8; 2];
    let mut z = [0u8; 2];
    h.fun_021a3460(player, &mut x, &mut y, &mut z);
    let x = i16::from_le_bytes([x[0], x[1]]) as i32 as u32;
    let z = i16::from_le_bytes(z) as i32 as u32;
    h.fun_0216ddc0(objsys, x, z, 0)
}

/// The WFBC object E, the player and the OBJ system of the field map of `R = FUN_02158F58(sw)` (R+4).
fn wfbc_ctx<H: Misc21bHooks>(h: &mut H, sw: u32) -> (u32, u32, u32) {
    let r = h.fun_02158f58(sw);
    let fm = h.r32(r + 4);
    let x = h.fun_02188c98(fm);
    let e = h.fun_0218dc70(x);
    let fm = h.r32(r + 4);
    let pl = h.fun_02188cb4(fm);
    let fm = h.r32(r + 4);
    let os = h.fun_02188c7c(fm);
    (e, pl, os)
}

/// C: `FUN_overlay_d_20__02186be4` (command 450): `FUN_02193B40(E, record of the person in front)`.
pub fn cmd_450<H: Misc21bHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let (e, pl, os) = wfbc_ctx(h, sw);
    let o = front_obj(h, pl, os);
    let rec = h.fun_02186090(o);
    h.fun_02193b40(e, rec);
    0
}

/// C: `FUN_overlay_d_20__02186c44` (command 451): when `FUN_02193BE4(E, rec)`, `FUN_02193BC4(E, rec)` and counter a += 1.
pub fn cmd_451<H: Misc21bHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let r = h.fun_02158f58(sw);
    let fm = h.r32(r + 4);
    let x = h.fun_02188c98(fm);
    let e = h.fun_0218dc70(x);
    let gd = h.fun_0215a5e4(ctx);
    let c = h.fun_020127f0(gd);
    let fm = h.r32(r + 4);
    let pl = h.fun_02188cb4(fm);
    let fm = h.r32(r + 4);
    let os = h.fun_02188c7c(fm);
    let o = front_obj(h, pl, os);
    let rec = h.fun_02186090(o);
    if h.fun_02193be4(e, rec) != 0 {
        h.fun_02193bc4(e, rec);
        wfbc_count_a_inc(h, c);
    }
    0
}

/// C: `FUN_overlay_d_20__02186cc4` (command 453): `FUN_02193B68(E, wordset, rec, operand, heap)`.
pub fn cmd_453<H: Misc21bHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let (e, pl, os) = wfbc_ctx(h, sw);
    let ws = h.fun_02158f74(sw);
    let v = h.fun_02159b10(vm, ctx);
    let heap = h.fun_0215a5cc(ctx);
    let o = front_obj(h, pl, os);
    let rec = h.fun_02186090(o);
    h.fun_02193b68(e, ws, rec, v, heap);
    0
}

/// C: `sub_02186d48` (command 452): one of 12 WFBC fields into a var (operand: 0 the person's u32 +0x18, 1 / 2
/// [`wfbc_rec_near`] / [`wfbc_rec_hi4`] of its record, 3 `FUN_02186278(data, 0)`, 4 / 5 / 7 / 8 the counters, 6
/// `FUN_021938F4(E)` != 0, 9 the record, 10 `FUN_02193BE4(E, rec)`, 11 `FUN_0216D588(obj)`; others store nothing).
pub fn cmd_452<H: Misc21bHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gd = h.fun_0215a5e4(ctx);
    let data = h.fun_020127c4(gd);
    let k = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    if k == 3 {
        let v = h.fun_02186278(data, 0);
        h.w16(var, v as u16);
        return 0;
    }
    let (e, pl, os) = wfbc_ctx(h, sw);
    let c = h.fun_020127f0(gd);
    let o = front_obj(h, pl, os);
    // the original's locals for "no OBJ / no person" stay as its fresh stack (0)
    let (mut rec, mut v8, mut person_rec, mut rec2) = (0u32, 0u32, 0u32, 0u32);
    if o != 0 {
        rec = h.fun_02186090(o);
        v8 = h.fun_0216d588(o);
        h.fun_02193904(e);
        let p = h.fun_0219390c(e, rec);
        if p != 0 {
            person_rec = h.fun_02193c00();
            rec2 = h.fun_02193c04(p);
        }
    }
    let v = match k {
        0 => h.r32(person_rec + 0x18),
        1 => u32::from(wfbc_rec_near(h, rec2)),
        2 => u32::from(wfbc_rec_hi4(h, rec2)),
        4 => u32::from(wfbc_count_a(h, c)),
        5 => u32::from(wfbc_count_b(h, c)),
        6 => u32::from(h.fun_021938f4(e) != 0),
        7 => u32::from(wfbc_count_a2(h, c)),
        8 => u32::from(wfbc_count_b2(h, c)),
        9 => rec,
        10 => h.fun_02193be4(e, rec),
        11 => v8,
        _ => return 0,
    };
    h.w16(var, v as u16);
    0
}

/// C: `FUN_overlay_d_20__02186e94` (command 455): counter b += 10 ([`wfbc_count_b_add10`]).
pub fn cmd_455<H: Misc21bHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let r = h.fun_02158f58(sw);
    let fm = h.r32(r + 4);
    let gs = h.fun_02188c80(fm);
    let gd = h.fun_020120f4(gs);
    let c = h.fun_020127f0(gd);
    wfbc_count_b_add10(h, c);
    0
}

/// C: `FUN_overlay_d_20__02186eb8` (command 454): scan the party for a non-egg mon of the species in counter b with version 0x33
/// and zero 0x8F / 0x90 / 0x91 fields: var 1 = found, var 3 = its slot (0xFFFF none); var 2 = 1 (and var 3 its slot) when
/// its met date is today.
pub fn cmd_454<H: Misc21bHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let gd = h.fun_0215a5e4(ctx);
    h.fun_020127c4(gd);
    let c = h.fun_020127f0(gd);
    let party = h.fun_02012934(gd);
    let v1 = h.fun_02159ae8(vm, ctx);
    let v2 = h.fun_02159ae8(vm, ctx);
    let v3 = h.fun_02159ae8(vm, ctx);
    let n = h.fun_0201aa18(party);
    let species = u32::from(wfbc_count_b2(h, c));
    let mut date = [0u32; 3];
    h.fun_0203f4e4(&mut date);
    h.w16(v1, 0);
    h.w16(v2, 0);
    h.w16(v3, 0xFFFF);
    let mut i = 0u32;
    while i < n {
        let mon = h.fun_0201ab28(party, i);
        let s = h.fun_02017e1c(mon, 5, 0);
        let y = h.fun_02017e1c(mon, 0x92, 0);
        let m = h.fun_02017e1c(mon, 0x93, 0);
        let d = h.fun_02017e1c(mon, 0x94, 0);
        let ver = h.fun_02017e1c(mon, 0x96, 0);
        let egg = h.fun_02017e1c(mon, 0x4C, 0);
        let a = h.fun_02017e1c(mon, 0x8F, 0);
        let b = h.fun_02017e1c(mon, 0x90, 0);
        let cc = h.fun_02017e1c(mon, 0x91, 0);
        if s == species && egg == 0 && ver == 0x33 && b.wrapping_add(a.wrapping_add(cc)) == 0 {
            h.w16(v1, 1);
            if h.r16(v3) == 0xFFFF {
                h.w16(v3, i as u16);
            }
            if y == date[0] && m == date[1] && d == date[2] {
                h.w16(v2, 1);
                h.w16(v3, i as u16);
            }
        }
        i += 1;
    }
    0
}

/// C: `FUN_overlay_d_20__02187000` (command 457): `(operand - 1) + 4 * rec + (FUN_0219391C(E) ? 9 : 1)` into a var.
pub fn cmd_457<H: Misc21bHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let (e, pl, os) = wfbc_ctx(h, sw);
    h.fun_02158f74(sw);
    let var = h.fun_02159ae8(vm, ctx);
    let v = h.fun_02159b10(vm, ctx);
    let o = front_obj(h, pl, os);
    let rec = h.fun_02186090(o);
    let v = v.wrapping_sub(1) & 0xFFFF;
    let x = if h.fun_0219391c(e) == 0 {
        (rec << 2) + 1
    } else {
        (rec << 2) + 9
    };
    h.w16(var, v.wrapping_add(x) as u16);
    0
}

/// C: `FUN_overlay_d_20__02187098` (command 456): start the WFBC talk dialog ([`wfbc_talk_create`]).
pub fn cmd_456<H: Misc21bHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    let ev = wfbc_talk_create(h, gs, fm);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `FUN_overlay_d_20__021870c4`: the WFBC talk dialog event (work 0x58; null without the intrude comm): +0 gs, +4 fm, +8 heap
/// | 0x8000, +0xC, +0x18 message data, +0x1C word set, +0x20 / +0x24 strings, +0x28 E, +0x2C the person, +0x30 the OBJ, +0x34
/// its position, +0x40 the comm state, +0x44 / +0x48, +0x4C the record, +0x56 = 1.
pub fn wfbc_talk_create<H: Misc21bHooks>(h: &mut H, gamesys: u32, fm: u32) -> u32 {
    let heap = h.fun_02188c84(fm);
    let x = h.fun_02188c54(fm);
    let gd = h.fun_020120f4(gamesys);
    let y = h.fun_02012138(gamesys);
    let comm = h.fun_02158344(y);
    if comm == 0 {
        return 0;
    }
    let st = h.fun_02158434(comm);
    let ev = h.fun_020122c0(gamesys, 0, EVENT_WFBC_TALK, WFBC_TALK_SIZE);
    let e = h.fun_020124e8(ev);
    h.w32(e, gamesys);
    h.w32(e + 4, fm);
    let s = h.fun_020457b0(0x100, heap);
    h.w32(e + 0x20, s);
    let s = h.fun_020457b0(0x100, heap);
    h.w32(e + 0x24, s);
    let m = h.fun_02045b38(0, 3, 0x142, heap);
    h.w32(e + 0x18, m);
    let ws = h.fun_0201ec64(heap);
    h.w32(e + 0x1C, ws);
    h.w32(e + 0xC, x);
    h.w32(e + 0x40, st);
    h.w8(e + 0x56, 1);
    h.w16(e + 8, ((heap & 0x7FFF) | 0x8000) as u16);
    let v = h.fun_020127d0(gd, 1);
    h.w32(e + 0x44, v);
    let v = h.fun_020127d0(gd, 0);
    h.w32(e + 0x48, v);
    let x = h.fun_02188c98(fm);
    let eo = h.fun_0218dc70(x);
    let pl = h.fun_02188cb4(fm);
    let os = h.fun_02188c7c(fm);
    let gd = h.fun_020120f4(gamesys);
    h.fun_020127f0(gd);
    let o = front_obj(h, pl, os);
    h.fun_0216d88c(o, e + 0x34);
    let rec = h.fun_02186090(o);
    let p = h.fun_0219390c(eo, rec);
    h.w32(e + 0x28, eo);
    h.w32(e + 0x2C, p);
    h.w32(e + 0x30, o);
    h.w16(e + 0x4C, rec as u16);
    ev
}

// ------------------------------------------------------------------ overlay 21: field skills 197..200

/// C: `sub_021c457c` (wait of command 197): the task is done (`FUN_021C1094 == 1`): free it.
pub fn wait_197<H: Misc21bHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let t = h.fun_0215a64c(ctx);
    if h.fun_021c1094(t) == 1 {
        h.fun_021c10a0(t);
        1
    } else {
        0
    }
}

/// C: `sub_021c459c` (command 197): the field-skill task `FUN_021C1024` on the tile in front, waited for by [`wait_197`].
pub fn cmd_197<H: Misc21bHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    let pl = h.fun_02188cb4(fm);
    let po = h.fun_021a3074(pl);
    let d = h.fun_0216d5f8(po);
    let map = h.fun_02188c98(fm);
    let heap = h.fun_0215a5cc(ctx);
    let mut pos = [0u32; 3];
    h.fun_021a3364(pl, d, &mut pos);
    let x = h.fun_021ab0d4(map, &pos);
    let t = h.fun_021c1024(pl, d, x, heap);
    h.fun_0215a644(ctx, t);
    h.fun_020113d0(vm, WAIT_197);
    1
}

/// C: `sub_021c4608` (wait of command 198): as [`wait_197`] with `FUN_021C12C4` / `FUN_021C12D0`.
pub fn wait_198<H: Misc21bHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let t = h.fun_0215a64c(ctx);
    if h.fun_021c12c4(t) == 1 {
        h.fun_021c12d0(t);
        1
    } else {
        0
    }
}

/// C: `sub_021c4628` (command 198): the field-skill task `FUN_021C1228(player, dir, operand, heap)`, waited for by
/// [`wait_198`].
pub fn cmd_198<H: Misc21bHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let heap = h.fun_0215a5cc(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    let pl = h.fun_02188cb4(fm);
    let d = h.fun_021a3050(pl);
    let v = h.fun_02159b10(vm, ctx);
    let t = h.fun_021c1228(pl, d, v, heap);
    h.fun_0215a644(ctx, t);
    h.fun_020113d0(vm, WAIT_198);
    1
}

/// C: `sub_021c4678` (command 199): `FUN_021C9550(player OBJ, FUN_02188CD0(fm))`.
pub fn cmd_199<H: Misc21bHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    let pl = h.fun_02188cb4(fm);
    let o = h.fun_021a3074(pl);
    let x = h.fun_02188cd0(fm);
    h.fun_021c9550(o, x);
    0
}

/// C: `FUN_overlay_d_21__021c46a4` (command 200): call the event `FUN_021D2418(gs, fm)` (operand 0) or `FUN_021D2440(gs, fm,
/// operand != 1)`.
pub fn cmd_200<H: Misc21bHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    let sw = h.fun_0215a5f4(ctx);
    let ev = match h.fun_02159b10(vm, ctx) {
        0 => h.fun_021d2418(gs, fm),
        1 => h.fun_021d2440(gs, fm, 0),
        _ => h.fun_021d2440(gs, fm, 1),
    };
    h.fun_0215897c(sw, ev);
    1
}

// ------------------------------------------------------------------ fld_wipe_3dobj.c 415..417 / 576

/// The GAMESYS and the 3D-object system of the field map.
fn wipe_ctx<H: Misc21bHooks>(h: &mut H, ctx: u32) -> (u32, u32) {
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    let x = h.fun_02189a00(fm);
    (gs, x)
}

/// Call the event `ev` when there is one.
fn call_if<H: Misc21bHooks>(h: &mut H, sw: u32, ev: u32) {
    if ev != 0 {
        h.fun_0215897c(sw, ev);
    }
}

/// C: `FUN_overlay_d_21__021c812c` (command 415): the wipe event `FUN_021C1CB0(gs, x, k)` (k = u8 of a script word).
pub fn cmd_415<H: Misc21bHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let (gs, x) = wipe_ctx(h, ctx);
    let k = h.fun_02011330(vm);
    let sw = h.fun_0215a5f4(ctx);
    let ev = h.fun_021c1cb0(gs, x, k & 0xFF);
    call_if(h, sw, ev);
    1
}

/// C: `FUN_overlay_d_21__021c8170` (command 416): the wipe event `FUN_021C1D68(gs, x, k)` (k = u8 operand).
pub fn cmd_416<H: Misc21bHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let (gs, x) = wipe_ctx(h, ctx);
    let k = h.fun_02159b10(vm, ctx);
    let sw = h.fun_0215a5f4(ctx);
    let ev = h.fun_021c1d68(gs, x, k & 0xFF);
    call_if(h, sw, ev);
    1
}

/// C: `sub_021c81b4` (command 417): the wipe event `FUN_021C30B8(gs, x, operand)`.
pub fn cmd_417<H: Misc21bHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let (gs, x) = wipe_ctx(h, ctx);
    let k = h.fun_02159b10(vm, ctx);
    let sw = h.fun_0215a5f4(ctx);
    let ev = h.fun_021c30b8(gs, x, k);
    call_if(h, sw, ev);
    1
}

/// C: `FUN_overlay_d_21__021c81f8` (command 576): the wipe event `FUN_021C1CB0` of the kind paired with the operand in
/// [`WIPE_KEYS`] (none for another operand).
pub fn cmd_576<H: Misc21bHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let (gs, x) = wipe_ctx(h, ctx);
    let key = h.fun_02159b10(vm, ctx);
    if let Some(&(_, kind)) = WIPE_KEYS.iter().find(|p| u32::from(p.0) == key) {
        let sw = h.fun_0215a5f4(ctx);
        let ev = h.fun_021c1cb0(gs, x, u32::from(kind) & 0xFF);
        call_if(h, sw, ev);
    }
    1
}

// ------------------------------------------------------------------ pleasure_boat.c

/// C: `FUN_overlay_d_21__021c8608`: create the boat work W (heap 5, 0x1B8 bytes; time, rounds, the bytes W+0x1B0..0x1B3 and
/// the flag W+0x1B4 cleared) and init it in overlay 36 (`FUN_021F36C0`).
pub fn boat_alloc<H: Misc21bHooks>(h: &mut H) -> u32 {
    let w = h.fun_02030734(BOAT_HEAP, BOAT_SIZE, 1, FILE_BOAT, LINE_BOAT);
    h.w32(w, 0);
    h.w16(w + 4, 0);
    h.w32(w + 8, 0);
    for i in 0..4 {
        h.w8(w + 0x1B0 + i, 0);
    }
    h.w32(w + 0x1B4, 0);
    h.fun_02034ac4(OVERLAY_BOAT);
    h.fun_021f36c0(w);
    h.fun_02034a5c(OVERLAY_BOAT);
    w
}

/// C: `FUN_overlay_d_21__021c86e8`: add `30 n` to the boat time; crossing a unit (0x546) ends the round: with `stop`, the time
/// stays just below the next unit, else the round count W[8] goes up and the time is its start.
pub fn boat_time_add<H: MemEnv>(h: &mut H, w: u32, n: u32, stop: u32) {
    let t = i32::from(h.r16(w + 4) as i16);
    let add = 30u32.wrapping_mul(n);
    let rem = (t % BOAT_TIME_UNIT) as u32;
    if (rem.wrapping_add(add) as i32) >= BOAT_TIME_UNIT {
        let r = h.r32(w + 8).wrapping_add(1);
        let start = r.wrapping_mul(BOAT_TIME_UNIT as u32);
        if stop != 0 {
            h.w16(w + 4, start.wrapping_sub(1) as u16);
        } else {
            h.w16(w + 4, start as u16);
            h.w32(w + 8, r);
        }
    } else {
        let t = h.r16(w + 4) as i16;
        h.w16(w + 4, t.wrapping_add(add as u16 as i16) as u16);
    }
}

/// C: `FUN_overlay_d_21__021c8738`: a pleasure-boat query: k 0 -> byte W+0x1B0, 1..3 -> bytes W+0x1B1..0x1B3, 4 -> u16 W+6,
/// 5 -> the number of the 15 records whose first word is nonzero, other k -> 0.
pub fn boat_query<H: MemEnv>(h: &mut H, w: u32, k: u32) -> u32 {
    match k {
        0 => u32::from(h.r8(w + 0x1B0)),
        1 => u32::from(h.r8(w + 0x1B1)),
        2 => u32::from(h.r8(w + 0x1B2)),
        3 => u32::from(h.r8(w + 0x1B3)),
        4 => u32::from(h.r16(w + 6)),
        5 => (0..BOAT_RECORDS)
            .filter(|&i| h.r32(w + 0x24 + BOAT_RECORD_SIZE * i) != 0)
            .count() as u32,
        _ => 0,
    }
}

/// C: `FUN_overlay_d_21__021c87d4`: the first word of boat record `i` = `v`.
pub fn boat_set<H: MemEnv>(h: &mut H, w: u32, i: u32, v: u32) {
    h.w32(
        w.wrapping_add(BOAT_RECORD_SIZE.wrapping_mul(i))
            .wrapping_add(0x24),
        v,
    );
}

/// C: `FUN_overlay_d_21__021c8810`: the boat flag word W+0x1B4 = 1.
pub fn boat_flag<H: MemEnv>(h: &mut H, w: u32) {
    h.w32(w + 0x1B4, 1);
}

/// The boat work slot `FUN_02012ED0(gd)` (through the field map's GAMESYS).
fn boat_slot<H: Misc21bHooks>(h: &mut H, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    let gs = h.fun_02188c80(fm);
    let gd = h.fun_020120f4(gs);
    h.fun_02012ed0(gd)
}

/// C: `FUN_overlay_d_21__021c881c` (command 421): create the boat work when the slot is empty ([`boat_alloc`]).
pub fn cmd_421<H: Misc21bHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let slot = boat_slot(h, ctx);
    if h.r32(slot) == 0 {
        let w = boat_alloc(h);
        h.w32(slot, w);
    }
    0
}

/// C: `FUN_overlay_d_21__021c8848` (command 422): free the boat work (`FUN_021666A4(slot)`).
pub fn cmd_422<H: Misc21bHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let slot = boat_slot(h, ctx);
    h.fun_021666a4(slot);
    0
}

/// C: `sub_021c8868` (command 423): [`boat_query`] (k = script word) into a var.
pub fn cmd_423<H: Misc21bHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let slot = boat_slot(h, ctx);
    let k = h.fun_02011330(vm);
    let var = h.fun_02159ae8(vm, ctx);
    let w = h.r32(slot);
    let v = boat_query(h, w, k);
    h.w16(var, v as u16);
    0
}

/// C: `sub_021c88ac` (command 424): [`boat_time_add`] (two script words).
pub fn cmd_424<H: Misc21bHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let slot = boat_slot(h, ctx);
    let n = h.fun_02011330(vm);
    let stop = h.fun_02011330(vm);
    let w = h.r32(slot);
    boat_time_add(h, w, n, stop);
    0
}

/// C: `sub_021c88ec` (command 426): sub-command (script word) 8: [`boat_set`] of record `i` to (second operand == 1); others do
/// nothing.
pub fn cmd_426<H: Misc21bHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    let slot = h.fun_02012ed0(gd);
    let i = h.fun_02159b10(vm, ctx);
    let k = h.fun_02011330(vm);
    h.fun_02011330(vm);
    let x = h.fun_02159b10(vm, ctx);
    if k == 8 {
        let w = h.r32(slot);
        boat_set(h, w, i, u32::from(x == 1));
    }
    0
}

/// C: `FUN_overlay_d_21__021c8960` (command 428): [`boat_flag`].
pub fn cmd_428<H: Misc21bHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let slot = boat_slot(h, ctx);
    let w = h.r32(slot);
    boat_flag(h, w);
    0
}

// ------------------------------------------------------------------ fld_trade.c 443

/// C: `sub_021cc544` (command 443): call the name-input event `FUN_0216C204(gs, fm)`.
pub fn cmd_443<H: Misc21bHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    let ev = h.fun_0216c204(gs, fm);
    h.fun_0215897c(sw, ev);
    1
}

/// Run the command handler at `handler` (Thumb bit allowed) when it is one of this area's; `None` otherwise.
pub fn dispatch<H: Misc21bHooks>(h: &mut H, handler: u32, vm: u32, ctx: u32) -> Option<u32> {
    Some(match handler & !1 {
        0x021C_459C => cmd_197(h, vm, ctx),
        0x021C_4628 => cmd_198(h, vm, ctx),
        0x021C_4678 => cmd_199(h, vm, ctx),
        0x021C_46A4 => cmd_200(h, vm, ctx),
        0x021C_812C => cmd_415(h, vm, ctx),
        0x021C_8170 => cmd_416(h, vm, ctx),
        0x021C_81B4 => cmd_417(h, vm, ctx),
        0x021C_881C => cmd_421(h, vm, ctx),
        0x021C_8848 => cmd_422(h, vm, ctx),
        0x021C_8868 => cmd_423(h, vm, ctx),
        0x021C_88AC => cmd_424(h, vm, ctx),
        0x021C_88EC => cmd_426(h, vm, ctx),
        0x021C_8960 => cmd_428(h, vm, ctx),
        0x021C_C544 => cmd_443(h, vm, ctx),
        0x0218_6BE4 => cmd_450(h, vm, ctx),
        0x0218_6C44 => cmd_451(h, vm, ctx),
        0x0218_6D48 => cmd_452(h, vm, ctx),
        0x0218_6CC4 => cmd_453(h, vm, ctx),
        0x0218_6EB8 => cmd_454(h, vm, ctx),
        0x0218_6E94 => cmd_455(h, vm, ctx),
        0x0218_7098 => cmd_456(h, vm, ctx),
        0x0218_7000 => cmd_457(h, vm, ctx),
        0x021C_81F8 => cmd_576(h, vm, ctx),
        _ => return None,
    })
}

/// Run the wait callback `f` (VM +0x10, Thumb bit set) when command 197 / 198 installed it; `None` otherwise.
pub fn call_wait<H: Misc21bHooks>(h: &mut H, f: u32, vm: u32, ctx: u32) -> Option<u32> {
    Some(match f {
        WAIT_197 => wait_197(h, vm, ctx),
        WAIT_198 => wait_198(h, vm, ctx),
        _ => return None,
    })
}

#[cfg(test)]
mod tests;
