//! F8e: overlay-21 `scrcmd_c03center.c`, Thumb code 0x021B5A18-0x021B66B8 (F1-SPLIT row 21.50): the turn tween 307, party /
//! record queries, move tutors (484..487), in-game trades (446 / 447) and name registration (22 command ids, 28 functions).
//! Nothing in this file touches the comm / C-Gear state (no X1 networking).
//!
//! Every function of the area is specced in `tools/rom-diff/specs/black-script/scrcmd_c03center.spec`, ported here, listed in
//! [`PORTED`] and checked with `tools/check-fn.sh black-script scrcmd_c03center`. [`COMMANDS`] maps the command ids to their
//! handlers, [`dispatch`] runs one by address and [`call_task`] the turn-tween TCB of command 307.
//!
//! Callees outside the area are [`C03Hooks`] methods named by address (vm.c, the operand readers, the ctx getters and the
//! child-event call come from [`CoreHooks`]). PokeParam ids (`FUN_02017E1C(mon, id, 0)`): 5 species, 9 level, 0x4C egg flag,
//! 0x6E gender, 0x6F form, 0x46..0x4B EVs, 3 / 0x97 (UNVERIFIED roles). ROM tables embedded and compared with the build by
//! the tests: [`TABLE_463_1`] / [`TABLE_463_2`] / [`TABLE_463_3`].

use crate::scrcmd_core::{s32_div, CoreHooks};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021B_5A18,
    0x021B_5B0C,
    0x021B_5B4C,
    0x021B_5B8C,
    0x021B_5BFC,
    0x021B_5C80,
    0x021B_5CE8,
    0x021B_5DD0,
    0x021B_5DF8,
    0x021B_5EB8,
    0x021B_5F24,
    0x021B_5FB4,
    0x021B_5FB8,
    0x021B_6048,
    0x021B_60F4,
    0x021B_6124,
    0x021B_6164,
    0x021B_61E8,
    0x021B_627C,
    0x021B_62C8,
    0x021B_631C,
    0x021B_636C,
    0x021B_63D4,
    0x021B_647C,
    0x021B_654C,
    0x021B_6590,
    0x021B_65D8,
    0x021B_6628,
];

/// Command id -> handler address (Thumb entry, bit 0 clear) of the commands of this area (the words of the command table
/// 0x0217062C at index id; checked against the ROM by the tests).
pub const COMMANDS: &[(u16, u32)] = &[
    (256, 0x021B_627C),
    (261, 0x021B_631C),
    (262, 0x021B_63D4),
    (263, 0x021B_5B4C),
    (265, 0x021B_636C),
    (285, 0x021B_5B8C),
    (286, 0x021B_5BFC),
    (287, 0x021B_5C80),
    (288, 0x021B_65D8),
    (307, 0x021B_5A18),
    (446, 0x021B_654C),
    (447, 0x021B_647C),
    (462, 0x021B_5EB8),
    (463, 0x021B_5DF8),
    (484, 0x021B_60F4),
    (485, 0x021B_6164),
    (486, 0x021B_6124),
    (487, 0x021B_61E8),
    (566, 0x021B_62C8),
    (567, 0x021B_5CE8),
    (568, 0x021B_6590),
    (578, 0x021B_6628),
];

/// `__FILE__` string "scrcmd_c03center.c" passed to `Heap_AllocDebug`.
pub const FILE: u32 = 0x021D_E25C;
/// TCB function of command 307's turn tween ([`turn_task`], Thumb).
pub const TURN_TASK: u32 = 0x021B_5B0D;
/// Overlay-21 BSS u16 the turn tween animates (an angle; handed to `FUN_0216CDB8`).
pub const TURN_ANGLE: u32 = 0x021E_E724;

/// Tutor kinds (the first operand of commands 484..487).
pub const TUTOR_DRACO: u32 = 0;
pub const TUTOR_ULTIMATE: u32 = 1;
pub const TUTOR_PLEDGE: u32 = 2;
/// Move ids the tutors teach: Draco Meteor, the three ultimate moves, the three pledges.
pub const MOVE_DRACO_METEOR: u32 = 0x1B2;
pub const MOVE_FRENZY_PLANT: u32 = 0x152;
pub const MOVE_BLAST_BURN: u32 = 0x133;
pub const MOVE_HYDRO_CANNON: u32 = 0x134;
pub const MOVE_GRASS_PLEDGE: u32 = 0x208;
pub const MOVE_FIRE_PLEDGE: u32 = 0x207;
pub const MOVE_WATER_PLEDGE: u32 = 0x206;
/// Level a tutored mon must reach (`FUN_021B5FB4`: 0xFF, i.e. no level check in effect).
pub const TUTOR_MAX_LEVEL: u8 = 0xFF;
/// The move word command 487 hands to `FUN_0215A0E8` by tutor kind (0, 1, 2, other).
pub const MOVES_487: [u32; 4] = [MOVE_DRACO_METEOR, 0xFFFE, 0xFFFD, 0xA5];
/// The item whose data command 566 tests the mon against (`FUN_0202130C(0x151)`).
pub const ITEM_566: u32 = 0x151;

/// PokeParam ids read here.
pub const PP_SPECIES: u32 = 5;
pub const PP_LEVEL: u32 = 9;
pub const PP_EGG: u32 = 0x4C;
pub const PP_GENDER: u32 = 0x6E;
pub const PP_FORM: u32 = 0x6F;
pub const PP_EV_HP: u32 = 0x46;
/// UNVERIFIED: read as "has Pokerus" by command 256.
pub const PP_97: u32 = 0x97;

/// The three `{u16 key, u16 value}` tables of command 463 (kinds 1 / 2 / 3; 0x021D992C / 0x021D9904 / 0x021D98DC, overlay 21
/// ROM), each ending at key 0.
pub const TABLE_463_1_ADDR: u32 = 0x021D_992C;
pub const TABLE_463_2_ADDR: u32 = 0x021D_9904;
pub const TABLE_463_3_ADDR: u32 = 0x021D_98DC;
pub const TABLE_463_1: [(u16, u16); 17] = [
    (0x27, 0x22),
    (0x4F, 0x23),
    (0x77, 0x24),
    (0x9F, 0x25),
    (0xC7, 0x26),
    (0xF9, 0x27),
    (0x12B, 0x28),
    (0x15D, 0x29),
    (0x18F, 0x2A),
    (0x1C1, 0x2B),
    (0x1F3, 0x2C),
    (0x225, 0x2D),
    (0x257, 0x2E),
    (0x266, 0x2F),
    (0x275, 0x30),
    (0x279, 0x31),
    (0, 0x31),
];
pub const TABLE_463_2: [(u16, u16); 10] = [
    (0xE, 4),
    (0x18, 5),
    (0x27, 6),
    (0x3B, 7),
    (0x59, 8),
    (0x72, 9),
    (0x86, 0xA),
    (0x90, 0xB),
    (0x95, 0xC),
    (0, 0xC),
];
pub const TABLE_463_3: [(u16, u16); 10] = [
    (0x1D, 0x15),
    (0x31, 0x16),
    (0x45, 0x17),
    (0x59, 0x18),
    (0x68, 0x19),
    (0x77, 0x1A),
    (0x86, 0x1B),
    (0x90, 0x1C),
    (0x95, 0x1D),
    (0, 0x1D),
];

/// Callees of the scrcmd_c03center functions outside the area, named by address (meaning per the spec; UNVERIFIED unless
/// stated). vm.c, the operand readers, the ctx getters and the child-event call come from [`CoreHooks`].
pub trait C03Hooks: CoreHooks {
    /// C: `FUN_0215A5CC(ctx)` (script_sys): the heap id of the script.
    fn fun_0215a5cc(&mut self, ctx: u32) -> u32;
    /// C: `FUN_02158F74(sw)` (script_sys): the script's word set.
    fn fun_02158f74(&mut self, sw: u32) -> u32;
    /// C: `FUN_02158F7C(sw)` (script_sys): the script's string buffer.
    fn fun_02158f7c(&mut self, sw: u32) -> u32;
    /// C: `FUN_0215B4AC(ctx, slot, &mon)` (scrcmd_work): the party mon of `slot` into `mon`; 1 when there is one.
    fn fun_0215b4ac(&mut self, ctx: u32, slot: u32, mon: &mut [u8; 4]) -> u32;
    /// C: `FUN_0215B500(mon, move)` (scrcmd_work): 1 when `mon` knows `move`.
    fn fun_0215b500(&mut self, mon: u32, mv: u32) -> u32;
    /// C: `FUN_0215A07C(gamesys, a, b)` (overlay 10): an event.
    fn fun_0215a07c(&mut self, gamesys: u32, a: u32, b: u32) -> u32;
    /// C: `FUN_0215A0E8(gamesys, a, b, mask, move)` (overlay 10): the move-tutor event.
    fn fun_0215a0e8(&mut self, gamesys: u32, a: u32, b: u32, mask: u32, mv: u32) -> u32;
    /// C: `FUN_0215A2E4(gamesys, fm, c, a, b, d == 0, d)` (overlay 10): an event.
    #[allow(clippy::too_many_arguments)]
    fn fun_0215a2e4(
        &mut self,
        gamesys: u32,
        fm: u32,
        c: u32,
        a: u32,
        b: u32,
        z: u32,
        d: u32,
    ) -> u32;
    /// C: `FUN_0201210C(gamesys)` (main): the field map.
    fn fun_0201210c(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_02188C58(fm)` (overlay 21): the field camera / effect object E.
    fn fun_02188c58(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188C7C(fm)` (overlay 21).
    fn fun_02188c7c(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188C84(fm)` (overlay 21): the field heap id.
    fn fun_02188c84(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188D30(fm)` (overlay 21): the field TCB system.
    fn fun_02188d30(&mut self, fm: u32) -> u32;
    /// C: `FUN_0218ED50(E)` (overlay 21): a record whose first u16 is the current angle.
    fn fun_0218ed50(&mut self, e: u32) -> u32;
    /// C: `FUN_0218FE70(E)` (overlay 21): a record whose u16 at +0x26 is the target angle.
    fn fun_0218fe70(&mut self, e: u32) -> u32;
    /// C: `FUN_0216CDB8(x, &angle)` (overlay 21): hand the angle word to `x`.
    fn fun_0216cdb8(&mut self, x: u32, angle: u32);
    /// C: `Heap_AllocDebug(heap, size, from_end, file, line)` (`0x02030734`).
    fn fun_02030734(&mut self, heap: u32, size: u32, from_end: u32, file: u32, line: u32) -> u32;
    /// C: `Heap_Free(block)` (`0x020307B0`).
    fn fun_020307b0(&mut self, block: u32);
    /// C: `FUN_02030DA8(tcbsys, f, work, pri)` (main): add a TCB.
    fn fun_02030da8(&mut self, tcbsys: u32, f: u32, work: u32, pri: u32);
    /// C: `FUN_02030EAC(tcb)` (main): delete a TCB.
    fn fun_02030eac(&mut self, tcb: u32) -> u32;
    /// C: `FUN_02012F08(gamedata)` (main): a save object (the money / records owner).
    fn fun_02012f08(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012FD8(gamedata)` (main): the name-registry object.
    fn fun_02012fd8(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012934(gamedata)` (main): the party.
    fn fun_02012934(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012944(gamedata)` (main).
    fn fun_02012944(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_0201292C(gamedata)` (main): the bag.
    fn fun_0201292c(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_0201AA18(party)` (main): the party count.
    fn fun_0201aa18(&mut self, party: u32) -> u32;
    /// C: `FUN_0201AB28(party, i)` (main): party mon `i`.
    fn fun_0201ab28(&mut self, party: u32, i: u32) -> u32;
    /// C: `FUN_02017E1C(mon, id, 0)` (main): a PokeParam value.
    fn fun_02017e1c(&mut self, mon: u32, id: u32, x: u32) -> u32;
    /// C: `FUN_0201891C(mon, k)` (main): a PokeParam type / flag test (UNVERIFIED: has type k).
    fn fun_0201891c(&mut self, mon: u32, k: u32) -> u32;
    /// C: `FUN_020188AC(mon, data)` (main): nonzero when `mon` passes the item-data test.
    fn fun_020188ac(&mut self, mon: u32, data: u32) -> u32;
    /// C: `FUN_02018790(mon)` (main): the value command 566 stores.
    fn fun_02018790(&mut self, mon: u32) -> u32;
    /// C: `FUN_0202130C(item)` (main): item data.
    fn fun_0202130c(&mut self, item: u32) -> u32;
    /// C: `FUN_0200C11C(z, species, form, egg, gender)` (main): register a seen mon (UNVERIFIED).
    fn fun_0200c11c(&mut self, z: u32, species: u32, form: u32, egg: u32, gender: u32);
    /// C: `FUN_0200C2B0(z, v, &arr)` (main): store the 6 u16 species `arr`.
    fn fun_0200c2b0(&mut self, z: u32, v: u32, arr: &[u8; 12]);
    /// C: `FUN_0200C2D0(z, v, &arr)` (main): the 6 u16 species of record `v` into `arr`.
    fn fun_0200c2d0(&mut self, z: u32, v: u32, arr: &mut [u8; 12]);
    /// C: `FUN_0201EE60(list, i, v)` (main): store entry `i` of the word-set list.
    fn fun_0201ee60(&mut self, list: u32, i: u32, v: u32);
    /// C: `FUN_0201EE0C(wordset, i, strbuf, a, b, c)` (main): put a string into the word set.
    #[allow(clippy::too_many_arguments)]
    fn fun_0201ee0c(&mut self, ws: u32, i: u32, buf: u32, a: u32, b: u32, c: u32);
    /// C: `FUN_0200CA1C(gamedata)` (main): the Pokedex object (UNVERIFIED).
    fn fun_0200ca1c(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_0200CA38(x)` (main).
    fn fun_0200ca38(&mut self, x: u32) -> u32;
    /// C: `FUN_0200CF08(x, heap)` (main): a count (command 463 kind 1).
    fn fun_0200cf08(&mut self, x: u32, heap: u32) -> u32;
    /// C: `FUN_0200CF54(x, heap)` (main): a count (kind 3).
    fn fun_0200cf54(&mut self, x: u32, heap: u32) -> u32;
    /// C: `FUN_0200D0F8(x, heap)` (main): a count (kind 2).
    fn fun_0200d0f8(&mut self, x: u32, heap: u32) -> u32;
    /// C: `FUN_0200D298(x)` (main): 1 when the kind-1 set is complete (UNVERIFIED).
    fn fun_0200d298(&mut self, x: u32) -> u32;
    /// C: `FUN_0200D2B0(x, heap)` (main): 1 when the kind-3 set is complete (UNVERIFIED).
    fn fun_0200d2b0(&mut self, x: u32, heap: u32) -> u32;
    /// C: `FUN_0200D2C4(x, heap)` (main): 1 when the kind-2 set is complete (UNVERIFIED).
    fn fun_0200d2c4(&mut self, x: u32, heap: u32) -> u32;
    /// C: `FUN_0200E740(z, i)` (main): nonzero when name `i` is registered.
    fn fun_0200e740(&mut self, z: u32, i: u32) -> u32;
    /// C: `FUN_0200E7AC(z, i)` (main): name `i` (a string).
    fn fun_0200e7ac(&mut self, z: u32, i: u32) -> u32;
    /// C: `FUN_0200E7BC(z, i, mon)` (main): register the name of `mon` as name `i`.
    fn fun_0200e7bc(&mut self, z: u32, i: u32, mon: u32);
    /// C: `FUN_02045924(strbuf, src)` (main): copy the string at `src` into the buffer.
    fn fun_02045924(&mut self, buf: u32, src: u32);
    /// C: `FUN_02045924(strbuf, &s)` (main) with a 1-character stack string `s` (u16 code + terminator slot).
    fn fun_02045924_str(&mut self, buf: u32, s: &[u8; 2]);
    /// C: `FUN_02045A98()` (main): a character code (called with r0 = 0).
    fn fun_02045a98(&mut self, r0: u32) -> u32;
    /// C: `FUN_02007FA0(bag, pocket)` (main): the item count of the pocket.
    fn fun_02007fa0(&mut self, bag: u32, pocket: u32) -> u32;
    /// C: `FUN_020081DC(bag, pocket, i)` (main): the item slot `i` of the pocket (u16 item first).
    fn fun_020081dc(&mut self, bag: u32, pocket: u32, i: u32) -> u32;
    /// C: `FUN_020212B0(item)` (main).
    fn fun_020212b0(&mut self, item: u32) -> u32;
    /// C: `FUN_020056EC()` (main RNG): a random number (command 568 calls it with r0 = the pocket count left over).
    fn fun_020056ec(&mut self, r0: u32) -> u32;
    /// C: `FUN_021C5AAC(gamesys, var, a, b)` (overlay 21): an event.
    fn fun_021c5aac(&mut self, gamesys: u32, var: u32, a: u32, b: u32) -> u32;
    /// C: `FUN_02166C4C(gamesys, fm, mon)` (overlay 21): an event.
    fn fun_02166c4c(&mut self, gamesys: u32, fm: u32, mon: u32) -> u32;
    /// C: `FUN_021CBFB0(heap, i)` (fld_trade.c): the trade record `i`.
    fn fun_021cbfb0(&mut self, heap: u32, i: u32) -> u32;
    /// C: `FUN_021CC040(t)` (fld_trade.c): free the trade record.
    fn fun_021cc040(&mut self, t: u32);
    /// C: `FUN_021CC060(t)` (fld_trade.c): the species the trade wants.
    fn fun_021cc060(&mut self, t: u32) -> u32;
    /// C: `FUN_021CC068(t)` (fld_trade.c): the gender the trade wants (2 = any).
    fn fun_021cc068(&mut self, t: u32) -> u32;
    /// C: `FUN_021CC500(gamesys, a, b)` (fld_trade.c): the trade event.
    fn fun_021cc500(&mut self, gamesys: u32, a: u32, b: u32) -> u32;
}

/// C: `sub_021b5a18`: command 307 (turn tween): n = frames; the u16 [`TURN_ANGLE`] = the current angle (`*FUN_0218ED50(E)`)
/// and goes to `FUN_0216CDB8(X, &angle)`; W = `Heap_AllocDebug(heap, 0x10, 1, ..)` = {start, delta, 0, n} with delta = the
/// target angle (`FUN_0218FE70(E)` +0x26) rounded to the nearest multiple of 0x4000, minus the start, wrapped into
/// -0x8000..0x7FFF; the TCB [`turn_task`] runs it (`FUN_02030DA8(T, .., W, 0)`); 0.
pub fn cmd_307<H: C03Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    h.fun_0215a5e4(ctx);
    let fm = h.fun_0201210c(gs);
    let tcbsys = h.fun_02188d30(fm);
    let heap = h.fun_02188c84(fm);
    let e = h.fun_02188c58(fm);
    let x = h.fun_02188c7c(fm);
    let n = h.fun_02159b10(vm, ctx);
    let w = h.fun_02030734(heap, 0x10, 1, FILE, 0x5D);
    let p = h.fun_0218ed50(e);
    let a = h.r16(p);
    h.w16(TURN_ANGLE, a);
    h.fun_0216cdb8(x, TURN_ANGLE);
    let q = h.fun_0218fe70(e);
    let start = u32::from(h.r16(TURN_ANGLE));
    h.w32(w, start);
    let t = u32::from(h.r16(q + 0x26));
    let r = t & 0x3FFF;
    let target = if r < 0x2000 { t - r } else { t + (0x4000 - r) };
    let mut d = target.wrapping_sub(start) as i32;
    if d.wrapping_abs() >= 0x8000 {
        d = if d > 0 { d - 0x10000 } else { d + 0x10000 };
    }
    h.w32(w + 4, d as u32);
    h.w32(w + 8, 0);
    h.w32(w + 0xC, n);
    h.fun_02030da8(tcbsys, TURN_TASK, w, 0);
    0
}

/// C: `sub_021b5b0c`: the TCB of command 307 (tcb, W = {start, delta, i, n}): i += 1; the angle = start + delta * i / n (signed
/// division, u16); i >= n -> W is freed and the TCB deleted. n = 0: `_s32_div_f` returns the dividend (angle = start +
/// delta * i) and i >= 0 ends the task at once (V-0028).
pub fn turn_task<H: C03Hooks>(h: &mut H, tcb: u32, w: u32) {
    let i = h.r32(w + 8).wrapping_add(1);
    h.w32(w + 8, i);
    let num = i.wrapping_mul(h.r32(w + 4)) as i32;
    let q = s32_div(num, h.r32(w + 0xC) as i32);
    h.w16(TURN_ANGLE, q as u16);
    let a = h.r16(TURN_ANGLE).wrapping_add(h.r32(w) as u16);
    h.w16(TURN_ANGLE, a);
    if h.r32(w + 8) as i32 >= h.r32(w + 0xC) as i32 {
        h.fun_020307b0(w);
        h.fun_02030eac(tcb);
    }
}

/// C: `sub_021b5b4c`: command 263: a, b = vars, a value (ignored); the event `FUN_0215A07C(GAMESYS, a, b)` runs as a child
/// event; 1.
pub fn cmd_263<H: C03Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a = h.fun_02159ae8(vm, ctx);
    let b = h.fun_02159ae8(vm, ctx);
    h.fun_02159b10(vm, ctx);
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let ev = h.fun_0215a07c(gs, a, b);
    h.fun_0215897c(sw, ev);
    1
}

/// The party mon of `slot` (`FUN_0215B4AC(ctx, slot, &mon)`): `Some(mon)` when the call answers `want`.
fn party_mon<H: C03Hooks>(h: &mut H, ctx: u32, slot: u32) -> (u32, u32) {
    let mut m = [0u8; 4];
    let r = h.fun_0215b4ac(ctx, slot, &mut m);
    (r, u32::from_le_bytes(m))
}

/// C: `sub_021b5b8c`: command 285: the party mon of slot `idx` (if any) -> `FUN_0200C11C(FUN_02012F08(GAMEDATA), species,
/// form, egg, gender)`; 0.
pub fn cmd_285<H: C03Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let idx = h.fun_02159b10(vm, ctx);
    let (r, mon) = party_mon(h, ctx, idx);
    if r == 1 {
        let gd = h.fun_0215a5e4(ctx);
        let z = h.fun_02012f08(gd);
        let sp = h.fun_02017e1c(mon, PP_SPECIES, 0) & 0xFFFF;
        let form = h.fun_02017e1c(mon, PP_FORM, 0) & 0xFFFF;
        let egg = h.fun_02017e1c(mon, PP_EGG, 0) & 0xFFFF;
        let gender = h.fun_02017e1c(mon, PP_GENDER, 0) & 0xFFFF;
        h.fun_0200c11c(z, sp, form, egg, gender);
    }
    0
}

/// C: `sub_021b5bfc`: command 286: v; the species of the party mons that are not eggs (in order, u16 arr[6], the rest 0) go
/// to `FUN_0200C2B0(FUN_02012F08(GAMEDATA), v, arr)`; 0. The party count is re-read every step.
pub fn cmd_286<H: C03Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let party = h.fun_02012934(gd);
    let v = h.fun_02159b10(vm, ctx);
    let mut arr = [0u8; 12];
    let mut k = 0usize;
    let mut i: i32 = 0;
    while i < h.fun_0201aa18(party) as i32 {
        let mon = h.fun_0201ab28(party, i as u32);
        if h.fun_02017e1c(mon, PP_EGG, 0) == 0 {
            let sp = h.fun_02017e1c(mon, PP_SPECIES, 0) as u16;
            arr[2 * k..2 * k + 2].copy_from_slice(&sp.to_le_bytes());
            k += 1;
        }
        i += 1;
    }
    let z = h.fun_02012f08(gd);
    h.fun_0200c2b0(z, v, &arr);
    0
}

/// C: `sub_021b5c80`: command 287: v, var; `FUN_0200C2D0(FUN_02012F08(GAMEDATA), v, arr)` fills u16 arr[6]; each nonzero entry
/// goes to the word set `FUN_02158F74(SW)` (`FUN_0201EE60(L, k++, arr[i])`); `*var = k`; 0.
pub fn cmd_287<H: C03Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let sw = h.fun_0215a5f4(ctx);
    let list = h.fun_02158f74(sw);
    let v = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let z = h.fun_02012f08(gd);
    let mut arr = [0u8; 12];
    h.fun_0200c2d0(z, v, &mut arr);
    let mut k = 0u32;
    for i in 0..6 {
        let x = u16::from_le_bytes([arr[2 * i], arr[2 * i + 1]]);
        if x != 0 {
            h.fun_0201ee60(list, k, u32::from(x));
            k += 1;
        }
    }
    h.w16(var, k as u16);
    0
}

/// C: `sub_021b5ce8`: command 567: idx, sel, var; no party mon in slot `idx` -> `*var = 0`; else the 6 EVs e[] (u16): sel 0
/// -> their sum (u16), 1 -> the largest, 2..7 -> e[sel - 2], other sel: var unchanged; 0.
pub fn cmd_567<H: C03Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let idx = h.fun_02159b10(vm, ctx);
    let sel = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let (r, mon) = party_mon(h, ctx, idx);
    if r == 0 {
        h.w16(var, 0);
        return 0;
    }
    let mut e = [0u16; 6];
    for (k, x) in e.iter_mut().enumerate() {
        *x = h.fun_02017e1c(mon, PP_EV_HP + k as u32, 0) as u16;
    }
    let mut sum: u16 = 0;
    let mut max: u16 = 0;
    for &x in &e {
        sum = sum.wrapping_add(x);
        if max < x {
            max = x;
        }
    }
    match sel {
        0 => h.w16(var, sum),
        1 => h.w16(var, max),
        2..=7 => h.w16(var, e[sel as usize - 2]),
        _ => {}
    }
    0
}

/// The first-key-0-or-not-below lookup of [`table_lookup`] over pair `k` from `pair(k)`.
fn lookup_by(mut pair: impl FnMut(u32) -> (u16, u16), v: u32) -> u16 {
    let mut k = 0;
    loop {
        let (key, val) = pair(k);
        if key == 0 || v as i32 <= i32::from(key) {
            return val;
        }
        k += 1;
    }
}

/// C: `FUN_overlay_d_21__021b5dd0`: the value of the first `{u16 key, u16 value}` pair whose key is 0 or not below `v`.
pub fn table_lookup<H: crate::hooks::MemEnv>(h: &mut H, table: u32, v: u32) -> u16 {
    lookup_by(|k| (h.r16(table + 4 * k), h.r16(table + 4 * k + 2)), v)
}

/// [`table_lookup`] over an embedded table (the ROM tables of command 463).
fn table_lookup_rom(table: &[(u16, u16)], v: u32) -> u16 {
    lookup_by(|k| table[k as usize], v)
}

/// C: `sub_021b5df8`: command 463: k, A, B = vars; R = `FUN_0200CA1C(GAMEDATA)`; `FUN_0200CA38(R)`; `*A = 5`, `*B = 0`; kind 1:
/// n = `FUN_0200CF08(R, heap)`, `*A` = `FUN_0200D298(R) == 1` ? 0x32 : [`table_lookup`]`(TABLE_463_1, n)`; kind 2: n =
/// `FUN_0200D0F8`, `*A` = `FUN_0200D2C4 == 1` ? 0xD : lookup(TABLE_463_2, n); kind 3: n = `FUN_0200CF54`, `*A` =
/// `FUN_0200D2B0 == 1` ? 0x1E : lookup(TABLE_463_3, n); then `*B = n`; other kinds stop after the defaults; 0.
pub fn cmd_463<H: C03Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let k = h.fun_02159b10(vm, ctx);
    let a = h.fun_02159ae8(vm, ctx);
    let b = h.fun_02159ae8(vm, ctx);
    let heap = h.fun_0215a5cc(ctx);
    let gd = h.fun_0215a5e4(ctx);
    let r = h.fun_0200ca1c(gd);
    h.fun_0200ca38(r);
    h.w16(a, 5);
    h.w16(b, 0);
    let (n, v) = match k {
        1 => {
            let n = h.fun_0200cf08(r, heap);
            let v = if h.fun_0200d298(r) == 1 {
                0x32
            } else {
                table_lookup_rom(&TABLE_463_1, n)
            };
            (n, v)
        }
        2 => {
            let n = h.fun_0200d0f8(r, heap);
            let v = if h.fun_0200d2c4(r, heap) == 1 {
                0xD
            } else {
                table_lookup_rom(&TABLE_463_2, n)
            };
            (n, v)
        }
        3 => {
            let n = h.fun_0200cf54(r, heap);
            let v = if h.fun_0200d2b0(r, heap) == 1 {
                0x1E
            } else {
                table_lookup_rom(&TABLE_463_3, n)
            };
            (n, v)
        }
        _ => return 0,
    };
    h.w16(a, v);
    h.w16(b, n as u16);
    0
}

/// C: `sub_021b5eb8`: command 462: R = `FUN_0200CA1C(GAMEDATA)`; var; kind 1 -> `*var = FUN_0200D298(R)`; 3 -> `*var =
/// FUN_0200D2B0(R, FUN_02188C84(fm))`; 2 -> `*var = FUN_0200D2C4(R, FUN_02188C84(fm))`; other kinds nothing; 0.
pub fn cmd_462<H: C03Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let r = h.fun_0200ca1c(gd);
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    let var = h.fun_02159ae8(vm, ctx);
    let v = match h.fun_02159b10(vm, ctx) {
        1 => h.fun_0200d298(r),
        3 => {
            let heap = h.fun_02188c84(fm);
            h.fun_0200d2b0(r, heap)
        }
        2 => {
            let heap = h.fun_02188c84(fm);
            h.fun_0200d2c4(r, heap)
        }
        _ => return 0,
    };
    h.w16(var, v as u16);
    0
}

/// C: `FUN_overlay_d_21__021b5f24`: the move tutor `k` teaches `mon`: Draco Meteor (0), the ultimate move (1) or the pledge (2)
/// of the first matching `FUN_0201891C` test (3 / 4 / 5 resp. 0 / 1 / 2); Blast Burn when none matches or for another `k`.
pub fn tutor_move<H: C03Hooks>(h: &mut H, mon: u32, k: u32) -> u32 {
    match k {
        TUTOR_DRACO => MOVE_DRACO_METEOR,
        TUTOR_ULTIMATE => {
            if h.fun_0201891c(mon, 3) != 0 {
                MOVE_FRENZY_PLANT
            } else if h.fun_0201891c(mon, 4) != 0 {
                MOVE_BLAST_BURN
            } else if h.fun_0201891c(mon, 5) != 0 {
                MOVE_HYDRO_CANNON
            } else {
                MOVE_BLAST_BURN
            }
        }
        TUTOR_PLEDGE => {
            if h.fun_0201891c(mon, 0) != 0 {
                MOVE_GRASS_PLEDGE
            } else if h.fun_0201891c(mon, 1) != 0 {
                MOVE_FIRE_PLEDGE
            } else if h.fun_0201891c(mon, 2) != 0 {
                MOVE_WATER_PLEDGE
            } else {
                MOVE_BLAST_BURN
            }
        }
        _ => MOVE_BLAST_BURN,
    }
}

/// C: `FUN_overlay_d_21__021b5fb4`: the level a tutored mon must have (0xFF for every tutor).
pub fn tutor_max_level(_k: u32) -> u8 {
    TUTOR_MAX_LEVEL
}

/// C: `FUN_overlay_d_21__021b5fb8`: whether tutor `k` can teach `mon`: k 0 test 6; k 1 any of the tests 3 / 4 / 5; k 2 any of
/// 0 / 1 / 2 (in that order, stopping at the first hit); other `k` no.
pub fn tutor_can_learn<H: C03Hooks>(h: &mut H, mon: u32, k: u32) -> bool {
    let tests: &[u32] = match k {
        TUTOR_DRACO => &[6],
        TUTOR_ULTIMATE => &[3, 4, 5],
        TUTOR_PLEDGE => &[0, 1, 2],
        _ => &[],
    };
    tests.iter().any(|&t| h.fun_0201891c(mon, t) != 0)
}

/// [`tutor_party_check`] with the mask returned instead of stored: (result, mask).
fn party_check<H: C03Hooks>(h: &mut H, ctx: u32, k: u32, level_check: u32) -> (u32, u8) {
    let gd = h.fun_0215a5e4(ctx);
    let party = h.fun_02012934(gd);
    let n = h.fun_0201aa18(party) as i32;
    let (mut any, mut ok, mut mask) = (false, false, 0u8);
    let mut i: i32 = 0;
    while i < n {
        let mon = h.fun_0201ab28(party, i as u32);
        if h.fun_02017e1c(mon, PP_EGG, 0) != 1 && tutor_can_learn(h, mon, k) {
            any = true;
            let pass = level_check == 0 || {
                let lvl = h.fun_02017e1c(mon, PP_LEVEL, 0);
                lvl >= u32::from(tutor_max_level(k))
            };
            if pass {
                ok = true;
                mask |= 1u32.checked_shl(i as u32).unwrap_or(0) as u8;
            }
        }
        i += 1;
    }
    let r = if !any {
        1
    } else if !ok {
        2
    } else {
        0
    };
    (r, mask)
}

/// C: `FUN_overlay_d_21__021b6048`: tutor_party_check(vm, ctx, k, u8 *mask, level_check): `*mask` = the party slots whose mon
/// is not an egg, can learn tutor `k`'s move and (when `level_check`) has the level [`tutor_max_level`]; returns 1 when no mon
/// can learn it, 2 when none passes the level check, else 0.
pub fn tutor_party_check<H: C03Hooks>(
    h: &mut H,
    _vm: u32,
    ctx: u32,
    k: u32,
    mask: u32,
    level_check: u32,
) -> u32 {
    h.w8(mask, 0);
    let (r, m) = party_check(h, ctx, k, level_check);
    h.w8(mask, m);
    r
}

/// C: `sub_021b60f4`: command 484: k, var; `*var = `[`tutor_party_check`]`(k, level check on)`; 0.
pub fn cmd_484<H: C03Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let k = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let (r, _) = party_check(h, ctx, k, 1);
    h.w16(var, r as u16);
    0
}

/// C: `sub_021b6124`: command 486: `k`, `slot`, `var`; `*var` = the tutor move of the party mon (0 when the slot is empty); 0.
pub fn cmd_486<H: C03Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let k = h.fun_02159b10(vm, ctx);
    let slot = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let (r, mon) = party_mon(h, ctx, slot);
    let v = if r == 1 { tutor_move(h, mon, k) } else { 0 };
    h.w16(var, v as u16);
    0
}

/// C: `sub_021b6164`: command 485: k, idx, var; `*var = 1`; no party mon -> stop; the mon cannot learn tutor `k`'s move -> 1;
/// it knows the move (`FUN_0215B500 == 1`) -> 3; its level is below [`tutor_max_level`] -> 2; else 0; 0.
pub fn cmd_485<H: C03Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let k = h.fun_02159b10(vm, ctx);
    let idx = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    h.w16(var, 1);
    let (r, mon) = party_mon(h, ctx, idx);
    if r == 0 {
        return 0;
    }
    let v = if !tutor_can_learn(h, mon, k) {
        1
    } else {
        let mv = tutor_move(h, mon, k);
        if h.fun_0215b500(mon, mv) == 1 {
            3
        } else {
            let lvl = h.fun_02017e1c(mon, PP_LEVEL, 0);
            if lvl < u32::from(tutor_max_level(k)) {
                2
            } else {
                0
            }
        }
    };
    h.w16(var, v);
    0
}

/// C: `sub_021b61e8`: command 487: k, A, B; [`tutor_party_check`]`(k, no level check)` fails -> `*A = 0`; else the tutor event
/// `FUN_0215A0E8(GAMESYS, A, B, mask, MOVES_487[k])` runs as a child event; 1.
pub fn cmd_487<H: C03Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let k = h.fun_02159b10(vm, ctx);
    let a = h.fun_02159ae8(vm, ctx);
    let b = h.fun_02159ae8(vm, ctx);
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let (r, mask) = party_check(h, ctx, k, 0);
    if r != 0 {
        h.w16(a, 0);
        return 1;
    }
    let mv = MOVES_487[k.min(3) as usize];
    let ev = h.fun_0215a0e8(gs, a, b, u32::from(mask), mv);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_021b627c`: command 256: `*var` = 1 when a party mon has PokeParam 0x97 set (stops at the first), else 0; 0.
pub fn cmd_256<H: C03Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let var = h.fun_02159ae8(vm, ctx);
    h.w16(var, 0);
    let gd = h.fun_0215a5e4(ctx);
    let party = h.fun_02012934(gd);
    let n = h.fun_0201aa18(party);
    let mut i = 0;
    while i < n {
        let mon = h.fun_0201ab28(party, i);
        if h.fun_02017e1c(mon, PP_97, 0) != 0 {
            h.w16(var, 1);
            break;
        }
        i += 1;
    }
    0
}

/// C: `sub_021b62c8`: command 566: var, idx; X = `FUN_0202130C(0x151)`; the party mon of `idx` exists and passes
/// `FUN_020188AC(mon, X)` -> `*var = FUN_02018790(mon)`, else 0xFFFF; 0.
pub fn cmd_566<H: C03Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let var = h.fun_02159ae8(vm, ctx);
    let idx = h.fun_02159b10(vm, ctx);
    let x = h.fun_0202130c(ITEM_566);
    let (r, mon) = party_mon(h, ctx, idx);
    let v = if r != 0 && h.fun_020188ac(mon, x) != 0 {
        h.fun_02018790(mon)
    } else {
        0xFFFF
    };
    h.w16(var, v as u16);
    0
}

/// C: `sub_021b631c`: command 261: var, a, b; the event `FUN_021C5AAC(GAMESYS, var, a, b)` runs as a child event; 1.
pub fn cmd_261<H: C03Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let a = h.fun_02159b10(vm, ctx);
    let b = h.fun_02159b10(vm, ctx);
    let ev = h.fun_021c5aac(gs, var, a, b);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_021b636c`: command 265: A, B = vars, c, d = values; the event `FUN_0215A2E4(GAMESYS, field map, c, A, B, d == 0, d)`
/// runs as a child event; 1.
pub fn cmd_265<H: C03Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a = h.fun_02159ae8(vm, ctx);
    let b = h.fun_02159ae8(vm, ctx);
    let c = h.fun_02159b10(vm, ctx);
    let d = h.fun_02159b10(vm, ctx);
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    let ev = h.fun_0215a2e4(gs, fm, c, a, b, (d == 0) as u32, d);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_021b63d4`: command 262: var; the first party mon with PokeParam 0x4C set and 3 / 9 both 0 -> the event
/// `FUN_02166C4C(GAMESYS, fm, mon)` runs as a child event, `*var` = its slot, 1; none -> 0. (The `FUN_02188C84` /
/// `FUN_02012944` results are unused.)
pub fn cmd_262<H: C03Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let var = h.fun_02159ae8(vm, ctx);
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    h.fun_02188c84(fm);
    let gd = h.fun_020120f4(gs);
    h.fun_02012944(gd);
    let party = h.fun_02012934(gd);
    let n = h.fun_0201aa18(party) as i32;
    let mut i: i32 = 0;
    while i < n {
        let mon = h.fun_0201ab28(party, i as u32);
        let a = h.fun_02017e1c(mon, PP_EGG, 0);
        let b = h.fun_02017e1c(mon, 3, 0);
        let c = h.fun_02017e1c(mon, PP_LEVEL, 0);
        if a != 0 && b == 0 && c == 0 {
            let ev = h.fun_02166c4c(gs, fm, mon);
            h.fun_0215897c(sw, ev);
            h.w16(var, i as u16);
            return 1;
        }
        i += 1;
    }
    0
}

/// C: `sub_021b647c`: command 447 (trade check): var, i, j; mon = party mon j; T = the trade record i
/// (`FUN_021CBFB0(FUN_02188C84(fm), i)`); `*var` = 0, then 1 for an egg, else 2 when the species differs from
/// `FUN_021CC060(T)`, else 3 when the trade wants a gender (`FUN_021CC068(T) != 2`) the mon does not have; T is freed; 0.
pub fn cmd_447<H: C03Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    let fm = h.fun_0201210c(gs);
    let heap = h.fun_02188c84(fm);
    let var = h.fun_02159ae8(vm, ctx);
    let i = h.fun_02159b10(vm, ctx);
    let j = h.fun_02159b10(vm, ctx);
    let party = h.fun_02012934(gd);
    let mon = h.fun_0201ab28(party, j);
    let t = h.fun_021cbfb0(heap, i);
    h.w16(var, 0);
    if h.r16(var) == 0 && h.fun_02017e1c(mon, PP_EGG, 0) == 1 {
        h.w16(var, 1);
    }
    if h.r16(var) == 0 {
        let sp = h.fun_02017e1c(mon, PP_SPECIES, 0);
        if sp != h.fun_021cc060(t) {
            h.w16(var, 2);
        }
    }
    if h.r16(var) == 0 && h.fun_021cc068(t) != 2 {
        let g = h.fun_02017e1c(mon, PP_GENDER, 0);
        if g != h.fun_021cc068(t) {
            h.w16(var, 3);
        }
    }
    h.fun_021cc040(t);
    0
}

/// C: `sub_021b654c`: command 446: a, b; the trade event `FUN_021CC500(GAMESYS, (u8)a, (u8)b)` runs as a child event; 1.
pub fn cmd_446<H: C03Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let a = h.fun_02159b10(vm, ctx);
    let b = h.fun_02159b10(vm, ctx);
    let ev = h.fun_021cc500(gs, a & 0xFF, b & 0xFF);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_021b6590`: command 568: var; n = the item count of bag pocket 2 (`FUN_02007FA0`); r = `FUN_020056EC()`; `*var` = n
/// == 0 ? 0 : `FUN_020212B0(the item of slot (u16)r of the pocket)`; 0.
pub fn cmd_568<H: C03Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let var = h.fun_02159ae8(vm, ctx);
    let gd = h.fun_0215a5e4(ctx);
    let bag = h.fun_0201292c(gd);
    let n = h.fun_02007fa0(bag, 2);
    let r = h.fun_020056ec(n);
    let v = if n == 0 {
        0
    } else {
        let slot = h.fun_020081dc(bag, 2, r & 0xFFFF);
        let item = u32::from(h.r16(slot));
        h.fun_020212b0(item)
    };
    h.w16(var, v as u16);
    0
}

/// C: `sub_021b65d8`: command 288: i, k; `FUN_0200E7BC(FUN_02012FD8(GAMEDATA), k, party mon i)`; 0.
pub fn cmd_288<H: C03Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    let z = h.fun_02012fd8(gd);
    let party = h.fun_02012934(gd);
    let i = h.fun_02159b10(vm, ctx);
    let k = h.fun_02159b10(vm, ctx);
    let mon = h.fun_0201ab28(party, i);
    h.fun_0200e7bc(z, k, mon);
    0
}

/// C: `sub_021b6628`: command 578: i, k; the string buffer B = `FUN_02158F7C(SW)` gets name `i` of `FUN_02012FD8(GAMEDATA)`
/// when it is registered (`FUN_0200E740`), else the 1-character string `{FUN_02045A98()}`; `FUN_0201EE0C(word set, k, B, 0,
/// 1, 2)`; 0.
pub fn cmd_578<H: C03Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    let z = h.fun_02012fd8(gd);
    let i = h.fun_02159b10(vm, ctx);
    let k = h.fun_02159b10(vm, ctx);
    let buf = h.fun_02158f7c(sw);
    let ws = h.fun_02158f74(sw);
    if h.fun_0200e740(z, i) != 0 {
        let name = h.fun_0200e7ac(z, i);
        h.fun_02045924(buf, name);
    } else {
        let c = h.fun_02045a98(0) as u16;
        h.fun_02045924_str(buf, &c.to_le_bytes());
    }
    h.fun_0201ee0c(ws, k, buf, 0, 1, 2);
    0
}

/// Run the command handler at `handler` (bit 0 ignored) when it is one of [`COMMANDS`]; `None` otherwise.
pub fn dispatch<H: C03Hooks>(h: &mut H, handler: u32, vm: u32, ctx: u32) -> Option<u32> {
    Some(match handler & !1 {
        0x021B_627C => cmd_256(h, vm, ctx),
        0x021B_631C => cmd_261(h, vm, ctx),
        0x021B_63D4 => cmd_262(h, vm, ctx),
        0x021B_5B4C => cmd_263(h, vm, ctx),
        0x021B_636C => cmd_265(h, vm, ctx),
        0x021B_5B8C => cmd_285(h, vm, ctx),
        0x021B_5BFC => cmd_286(h, vm, ctx),
        0x021B_5C80 => cmd_287(h, vm, ctx),
        0x021B_65D8 => cmd_288(h, vm, ctx),
        0x021B_5A18 => cmd_307(h, vm, ctx),
        0x021B_654C => cmd_446(h, vm, ctx),
        0x021B_647C => cmd_447(h, vm, ctx),
        0x021B_5EB8 => cmd_462(h, vm, ctx),
        0x021B_5DF8 => cmd_463(h, vm, ctx),
        0x021B_60F4 => cmd_484(h, vm, ctx),
        0x021B_6164 => cmd_485(h, vm, ctx),
        0x021B_6124 => cmd_486(h, vm, ctx),
        0x021B_61E8 => cmd_487(h, vm, ctx),
        0x021B_62C8 => cmd_566(h, vm, ctx),
        0x021B_5CE8 => cmd_567(h, vm, ctx),
        0x021B_6590 => cmd_568(h, vm, ctx),
        0x021B_6628 => cmd_578(h, vm, ctx),
        _ => return None,
    })
}

/// Run the TCB function `f(tcb, work)` when it is [`TURN_TASK`]; false otherwise.
pub fn call_task<H: C03Hooks>(h: &mut H, f: u32, tcb: u32, work: u32) -> bool {
    if f != TURN_TASK {
        return false;
    }
    turn_task(h, tcb, work);
    true
}

#[cfg(test)]
mod tests;
