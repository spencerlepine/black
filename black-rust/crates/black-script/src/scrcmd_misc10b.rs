//! F8f (fifth batch): overlay-10 script commands of F1 rows 10.27 ("symbol_map.c", 0x021664CC-0x02166F9C: the command handlers
//! and their helpers only) and 10.34 (0x0216AC40-0x0216B504, an unanchored scrcmd file: the Entree Forest symbol encounters):
//! pleasure-boat queries 425 / 427, the 20-record save block 477 / 478 / 480, 493, weather 310 and 554..558 / 560 (35 functions,
//! 13 command ids).
//!
//! Every function of the area is specced in `tools/rom-diff/specs/black-script/scrcmd_misc10b.spec`, ported here, listed in
//! [`PORTED`] and checked with `tools/check-fn.sh black-script scrcmd_misc10b`. [`COMMANDS`] maps the command ids to their
//! handlers, [`dispatch`] runs one by address and [`call_event`] the GMEVENT functions.
//!
//! Callees outside the area are [`Misc10bHooks`] methods named by address (vm.c, the operand readers, the ctx getters and the
//! GMEVENT calls come from [`CoreHooks`]); the overlay-18 / overlay-11 intrude (Entralink) callees of row 10.34 stay hooks
//! (X1). Roles are UNVERIFIED (named by address). A callee the spec models as returning nothing whose `r0` the original
//! uses afterwards (`FUN_02166618`, `FUN_021C87A0` / `AC` / `B8`, `FUN_02029C00`, `FUN_02029CC0`) is a hook returning `u32`:
//! the value the original finds in `r0`.

use crate::scrcmd_core::CoreHooks;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0216_6658,
    0x0216_666C,
    0x0216_66B8,
    0x0216_670C,
    0x0216_6714,
    0x0216_6734,
    0x0216_67E0,
    0x0216_6824,
    0x0216_6834,
    0x0216_6854,
    0x0216_6908,
    0x0216_6980,
    0x0216_69C0,
    0x0216_6A00,
    0x0216_6D30,
    0x0216_6E50,
    0x0216_6F1C,
    0x0216_AC40,
    0x0216_AC90,
    0x0216_ACCC,
    0x0216_AD1C,
    0x0216_AD70,
    0x0216_AE10,
    0x0216_AED0,
    0x0216_AEE8,
    0x0216_AF1C,
    0x0216_AF3C,
    0x0216_AF74,
    0x0216_B004,
    0x0216_B24C,
    0x0216_B280,
    0x0216_B2E4,
    0x0216_B340,
    0x0216_B3A8,
    0x0216_B3EC,
];

/// Command id -> handler address (Thumb entry, bit 0 clear) of the commands of this area (the words of the command table
/// 0x0217062C at index id; checked against the ROM by the tests).
pub const COMMANDS: &[(u16, u32)] = &[
    (310, 0x0216_6F1C),
    (425, 0x0216_6734),
    (427, 0x0216_67E0),
    (477, 0x0216_6980),
    (478, 0x0216_69C0),
    (480, 0x0216_6A00),
    (493, 0x0216_6D30),
    (554, 0x0216_AE10),
    (555, 0x0216_AD70),
    (556, 0x0216_ACCC),
    (557, 0x0216_AD1C),
    (558, 0x0216_AC40),
    (560, 0x0216_AC90),
];

/// GMEVENT functions of this area (Thumb): the boat wait of 427, the save-block app of 480, the app of 493, the symbol
/// battle, the intrude wait and the forest warp.
pub const EVENT_BOAT: u32 = 0x0216_66B9;
pub const EVENT_480: u32 = 0x0216_6909;
pub const EVENT_493: u32 = 0x0216_6E51;
pub const EVENT_SYMBOL_BATTLE: u32 = 0x0216_B005;
pub const EVENT_INTRUDE_WAIT: u32 = 0x0216_B281;
pub const EVENT_FOREST_WARP: u32 = 0x0216_B3ED;

/// One boat-time unit (s16 W+4 counts in these; `FUN_02166714` / `FUN_021C86E8`).
pub const BOAT_TIME_UNIT: i32 = 0x546;
/// App proc of the boat event (`FUN_02012444(event, 0xCB, 0x021BFB3C, E)`).
pub const PROC_BOAT: (u32, u32) = (0xCB, 0x021B_FB3C);
/// App proc of command 480's event.
pub const PROC_480: (u32, u32) = (0xDA, 0x021B_AB5C);
/// App proc of command 493's event.
pub const PROC_493: (u32, u32) = (0x6E, 0x021C_39B8);
/// Event work sizes.
pub const BOAT_EVENT_SIZE: u32 = 0x20;
pub const EVENT_480_SIZE: u32 = 0x128;
pub const EVENT_493_SIZE: u32 = 0x1C;
pub const SYMBOL_BATTLE_SIZE: u32 = 0x3C;
pub const INTRUDE_WAIT_SIZE: u32 = 8;
pub const FOREST_WARP_SIZE: u32 = 0x24;
/// The weather operand meaning "the zone's weather" (command 310).
pub const WEATHER_ZONE: u32 = 0xFFFF;
/// Script variable id read by `FUN_0216AEE8` (`FUN_0215901C(sw, gd, 0x8011)`).
pub const VAR_SYMBOL_OBJ: u32 = 0x8011;
/// The species limit passed with the symbol mon (`FUN_020173EC` returns it: 0xDC).
pub const SPECIES_LIMIT: u32 = 0xDC;
/// The default packed symbol word bits of `FUN_0216AF74` (W+0x20 low 11 bits when the check fails).
pub const SYMBOL_DEFAULT_LOW: u32 = 0x28B;
/// Overlay ids loaded around the symbol battle apps (`FUN_02034AC4` / `FUN_02034A5C`).
pub const OVERLAY_SYMBOL_A: u32 = 0x62;
pub const OVERLAY_SYMBOL_B: u32 = 0xE3;
/// The two app-proc tables of the symbol battle event (overlays 0x62 / 0xE3).
pub const PROC_SYMBOL_A: u32 = 0x021B_6D74;
pub const PROC_SYMBOL_B: u32 = 0x021B_98A4;
/// The 02029BCC argument of the symbol battle's first step.
pub const SYMBOL_BATTLE_ID: u32 = 0x468;
/// The 02007E50 / 02007F1C id of the symbol battle (9 << 6).
pub const SYMBOL_ZUKAN_ID: u32 = 0x240;
/// The SE of the forest warp event (W+0x1C set).
pub const SE_FOREST_WARP: u32 = 0x559;
/// The map change of command 554's sub-command 1 (`FUN_02184D38(gs, fm, 279, &pos, 1, 0)`) and its x offset (0x3E << 14).
pub const ZONE_554: u32 = 279;
pub const WARP_554_DX: u32 = 0x3E << 14;

/// Callees of the scrcmd_misc10b functions outside the area, named by address (meaning per the spec; UNVERIFIED unless
/// stated). vm.c, the operand readers, the ctx getters and the GMEVENT calls come from [`CoreHooks`].
#[allow(clippy::too_many_arguments)]
pub trait Misc10bHooks: CoreHooks {
    // ---- script_sys / overlay 10
    /// C: `FUN_0215A5CC(ctx)`: the heap id of the script.
    fn fun_0215a5cc(&mut self, ctx: u32) -> u32;
    /// C: `FUN_02158F74(sw)`: the word set.
    fn fun_02158f74(&mut self, sw: u32) -> u32;
    /// C: `FUN_0215901C(sw, gd, id)`: the value of script variable `id`.
    fn fun_0215901c(&mut self, sw: u32, gd: u32, id: u32) -> u32;
    /// C: `FUN_02158344(x)`: the intrude comm state or null (`delivery_irc`, X1).
    fn fun_02158344(&mut self, x: u32) -> u32;
    /// C: `FUN_02161CDC(e, gamesys, a, b)`.
    fn fun_02161cdc(&mut self, e: u32, gamesys: u32, a: u32, b: u32);
    /// C: `FUN_0216628C(y, v)` (symbol_map.c).
    fn fun_0216628c(&mut self, y: u32, v: u32) -> u32;
    /// C: `FUN_021664CC(heap, gamesys, &out)` (symbol_map.c): a record (freed with `FUN_020307B0`) or null; fills the word.
    fn fun_021664cc(&mut self, heap: u32, gamesys: u32, out: &mut u32) -> u32;
    /// C: `FUN_021665D0(gamesys, x)` (symbol_map.c).
    fn fun_021665d0(&mut self, gamesys: u32, x: u32) -> u32;
    /// C: `FUN_021665F0(gamesys, a, b)` (symbol_map.c).
    fn fun_021665f0(&mut self, gamesys: u32, a: u32, b: u32) -> u32;
    /// C: `FUN_02166618(x)` (symbol_map.c): nothing returned; the value left in `r0` (the argument under the stub).
    fn fun_02166618(&mut self, x: u32) -> u32;
    /// C: `FUN_0216662C(x)` (symbol_map.c): a flag.
    fn fun_0216662c(&mut self, x: u32) -> u32;
    /// C: `FUN_0216D888(x)`: a position (VecFx32 pointer).
    fn fun_0216d888(&mut self, x: u32) -> u32;
    /// C: `FUN_0216D9A0(objsys)`: pause the OBJs.
    fn fun_0216d9a0(&mut self, objsys: u32);
    /// C: `FUN_0216D9E4(objsys)`: resume the OBJs.
    fn fun_0216d9e4(&mut self, objsys: u32);
    /// C: `FUN_0216DE94(objsys, id)`: the OBJ of an id or null.
    fn fun_0216de94(&mut self, objsys: u32, id: u32) -> u32;
    // ---- main
    /// C: `FUN_020061E4(se)`: play an SE.
    fn fun_020061e4(&mut self, se: u32);
    /// C: `FUN_02007E50(zukan, id, a, b)`.
    fn fun_02007e50(&mut self, zukan: u32, id: u32, a: u32, b: u32);
    /// C: `FUN_02007F1C(zukan, id, a, b)`.
    fn fun_02007f1c(&mut self, zukan: u32, id: u32, a: u32, b: u32);
    /// C: `FUN_0200856C(record)`: a u16 of a record.
    fn fun_0200856c(&mut self, rec: u32) -> u32;
    /// C: `FUN_0200873C(save)`.
    fn fun_0200873c(&mut self, save: u32) -> u32;
    /// C: `FUN_02009470(save)`: the 20-record save block.
    fn fun_02009470(&mut self, save: u32) -> u32;
    /// C: `FUN_020095C8(block)`: the record count.
    fn fun_020095c8(&mut self, block: u32) -> u32;
    /// C: `FUN_0200973C(block, i)`: record `i`.
    fn fun_0200973c(&mut self, block: u32, i: u32) -> u32;
    /// C: `FUN_02009794(block, i)`: flag `i`.
    fn fun_02009794(&mut self, block: u32, i: u32) -> u32;
    /// C: `FUN_0200CA10(save)`.
    fn fun_0200ca10(&mut self, save: u32) -> u32;
    /// C: `FUN_0200E0B0(save)`: the symbol save block Y.
    fn fun_0200e0b0(&mut self, save: u32) -> u32;
    /// C: `FUN_0200E18C(y, v)`.
    fn fun_0200e18c(&mut self, y: u32, v: u32);
    /// C: `FUN_0200E2BC(save, &word, species, x)`: a check on a packed symbol word (read only).
    fn fun_0200e2bc(&mut self, save: u32, word: &u32, species: u32, x: u32) -> u32;
    /// C: `FUN_0200E6F0(save)`.
    fn fun_0200e6f0(&mut self, save: u32) -> u32;
    /// C: `FUN_0200E6FC(x, k, &out)`: fill a 12-byte record.
    fn fun_0200e6fc(&mut self, x: u32, k: u32, out: &mut [u32; 3]);
    /// C: `FUN_0200E714(x, k, &rec)`: store a 12-byte record.
    fn fun_0200e714(&mut self, x: u32, k: u32, rec: &[u32; 3]);
    /// C: `FUN_0201210C(gamesys)`: the field map.
    fn fun_0201210c(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_02012138(gamesys)`.
    fn fun_02012138(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_02012374(event, child)`: call a child event.
    fn fun_02012374(&mut self, event: u32, child: u32);
    /// C: `FUN_02012444(event, id, data, work)`: run an app proc.
    fn fun_02012444(&mut self, event: u32, id: u32, data: u32, work: u32);
    /// C: `FUN_020124E4(event)`: the event's GAMESYS.
    fn fun_020124e4(&mut self, event: u32) -> u32;
    /// C: `FUN_0201292C(gamedata)`: the zukan.
    fn fun_0201292c(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012944(gamedata)`: the player status.
    fn fun_02012944(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_020129A0(gamedata, weather)`.
    fn fun_020129a0(&mut self, gamedata: u32, weather: u32);
    /// C: `FUN_02012EBC(gamedata)`: the save data.
    fn fun_02012ebc(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012ED0(gamedata)`: the boat work slot (a pointer to W).
    fn fun_02012ed0(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012F38(gamedata)`: the 20-record save-block record R.
    fn fun_02012f38(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012FA4(gamedata, x)`.
    fn fun_02012fa4(&mut self, gamedata: u32, x: u32);
    /// C: `FUN_02012FB0(gamedata)`.
    fn fun_02012fb0(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_020144DC(gamesys, x)`: a child event.
    fn fun_020144dc(&mut self, gamesys: u32, x: u32) -> u32;
    /// C: `FUN_020145CC(gamesys)`: a child event.
    fn fun_020145cc(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_02017E1C(mon, id, x)`: a PokeParam value.
    fn fun_02017e1c(&mut self, mon: u32, id: u32, x: u32) -> u32;
    /// C: `FUN_0201EE84(wordset, i, x)`.
    fn fun_0201ee84(&mut self, ws: u32, i: u32, x: u32);
    /// C: `FUN_02024044(a, b, c, d, save, e)`: a record (command 493).
    fn fun_02024044(&mut self, a: u32, b: u32, c: u32, d: u32, save: u32, e: u32) -> u32;
    /// C: `FUN_020240FC(rec)`: free it.
    fn fun_020240fc(&mut self, rec: u32);
    /// C: `FUN_02024108(rec, &v)`: store 12 bytes.
    fn fun_02024108(&mut self, rec: u32, v: &[u32; 3]);
    /// C: `FUN_0202411C(rec)`.
    fn fun_0202411c(&mut self, rec: u32) -> u32;
    /// C: `FUN_02024124(rec)`.
    fn fun_02024124(&mut self, rec: u32) -> u32;
    /// C: `FUN_02024128(rec, &out)`: 4 bytes (two u16).
    fn fun_02024128(&mut self, rec: u32, out: &mut u32);
    /// C: `FUN_02024134(rec, &out)`: 12 bytes.
    fn fun_02024134(&mut self, rec: u32, out: &mut [u32; 3]);
    /// C: `FUN_02029BCC(gamesys, x)`: a child event.
    fn fun_02029bcc(&mut self, gamesys: u32, x: u32) -> u32;
    /// C: `FUN_02029C00(gamesys)`: a child event (the spec's stub returns nothing: `r0` = the argument).
    fn fun_02029c00(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_02029CC0(gamesys)`: a child event (the spec's stub returns nothing: `r0` = the argument).
    fn fun_02029cc0(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_0202C794(gamesys, zone)`: the zone's weather.
    fn fun_0202c794(&mut self, gamesys: u32, zone: u32) -> u32;
    /// C: `FUN_020307B0(p)`: free a block.
    fn fun_020307b0(&mut self, p: u32);
    /// C: `FUN_02034A5C(id)`: unload an overlay.
    fn fun_02034a5c(&mut self, id: u32);
    /// C: `FUN_02034AC4(id)`: load an overlay.
    fn fun_02034ac4(&mut self, id: u32);
    /// C: `FUN_0203F1FC(mon, limit)`.
    fn fun_0203f1fc(&mut self, mon: u32, limit: u32) -> u32;
    // ---- overlays 11 / 18 / 20 / 21 and the symbol-battle overlays
    /// C: `FUN_021767A0(x, gd)` (ov11): the intrude state.
    fn fun_021767a0(&mut self, x: u32, gd: u32) -> u32;
    /// C: `FUN_021767F0(x, gd, &pos)` (ov11): a position.
    fn fun_021767f0(&mut self, x: u32, gd: u32, pos: &mut [u32; 3]);
    /// C: `FUN_02176850(comm, x)` (ov11).
    fn fun_02176850(&mut self, comm: u32, x: u32) -> u32;
    /// C: `FUN_02176870(comm)` (ov11).
    fn fun_02176870(&mut self, comm: u32) -> u32;
    /// C: `FUN_0217688C(comm, x)` (ov11).
    fn fun_0217688c(&mut self, comm: u32, x: u32);
    /// C: `FUN_0217C274(&word, obj)` (ov18 intrude_field.c): the packed symbol word of an OBJ.
    fn fun_0217c274(&mut self, word: &mut u32, obj: u32);
    /// C: `FUN_0217C294(obj)` (ov18).
    fn fun_0217c294(&mut self, obj: u32) -> u32;
    /// C: `FUN_0217C2A0(k, gd, &word)` (ov18): a record (freed with `FUN_020307B0`) or null.
    fn fun_0217c2a0(&mut self, k: u32, gd: u32, word: &u32) -> u32;
    /// C: `FUN_0217C37C(fieldmap, w, rec, a, b)` (ov18).
    fn fun_0217c37c(&mut self, fm: u32, w: u32, rec: u32, a: u32, b: u32);
    /// C: `FUN_02184D38(gamesys, fm, zone, &pos, a, b)` (ov20): the warp event.
    fn fun_02184d38(
        &mut self,
        gamesys: u32,
        fm: u32,
        zone: u32,
        pos: &[u32; 3],
        a: u32,
        b: u32,
    ) -> u32;
    /// C: `FUN_021854B0(gamesys, fm, zone, &pos, dir)` (ov20): the map-change event.
    fn fun_021854b0(&mut self, gamesys: u32, fm: u32, zone: u32, pos: &[u32; 3], dir: u32) -> u32;
    /// C: `FUN_02188C6C(fm)`: the weather system.
    fn fun_02188c6c(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188C7C(fm)`: the field OBJ system.
    fn fun_02188c7c(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188C9C(fm)`: the zone.
    fn fun_02188c9c(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188CB4(fm)`: the player.
    fn fun_02188cb4(&mut self, fm: u32) -> u32;
    /// C: `FUN_02189A08(x)`.
    fn fun_02189a08(&mut self, x: u32) -> u32;
    /// C: `FUN_0219C630(objsys)`: nonzero while busy.
    fn fun_0219c630(&mut self, objsys: u32) -> u32;
    /// C: `FUN_021A1BDC(weather, w)`: set the weather now.
    fn fun_021a1bdc(&mut self, weather: u32, w: u32);
    /// C: `FUN_021A1BE4(weather, w)`: change the weather.
    fn fun_021a1be4(&mut self, weather: u32, w: u32);
    /// C: `FUN_021A3038(player, &pos)`: the player position.
    fn fun_021a3038(&mut self, player: u32, pos: &mut [u32; 3]);
    /// C: `FUN_021A3074(player)`.
    fn fun_021a3074(&mut self, player: u32) -> u32;
    /// C: `FUN_021AA304(rec, gd, x)`.
    fn fun_021aa304(&mut self, rec: u32, gd: u32, x: u32);
    /// C: `FUN_021B6100(gd, mon, rec, status, zukan, a, b, species)` (overlay 0x62): the app work.
    fn fun_021b6100(
        &mut self,
        gd: u32,
        mon: u32,
        rec: u32,
        status: u32,
        zukan: u32,
        a: u32,
        b: u32,
        species: u32,
    ) -> u32;
    /// C: `FUN_021B6158(work)` (overlay 0x62): the app result.
    fn fun_021b6158(&mut self, work: u32) -> u32;
    /// C: `FUN_021B95A0(gd, result, mon, species)` (overlay 0xE3): the app work.
    fn fun_021b95a0(&mut self, gd: u32, result: u32, mon: u32, species: u32) -> u32;
    /// C: `FUN_021B95C8(work)` (overlay 0xE3): free it.
    fn fun_021b95c8(&mut self, work: u32);
    /// C: `FUN_021BE8D4(gamesys, fm, a, b)`: a fade-out event.
    fn fun_021be8d4(&mut self, gamesys: u32, fm: u32, a: u32, b: u32) -> u32;
    /// C: `FUN_021BE92C(gamesys, fm, a, b, c, d, e)`: a fade-in event.
    fn fun_021be92c(
        &mut self,
        gamesys: u32,
        fm: u32,
        a: u32,
        b: u32,
        c: u32,
        d: u32,
        e: u32,
    ) -> u32;
    /// C: `FUN_021C87A0(W, i)` (pleasure_boat.c; the spec's stub returns nothing: `r0` = W).
    fn fun_021c87a0(&mut self, w: u32, i: u32) -> u32;
    /// C: `FUN_021C87AC(W, i)` (pleasure_boat.c; as `FUN_021C87A0`).
    fn fun_021c87ac(&mut self, w: u32, i: u32) -> u32;
    /// C: `FUN_021C87B8(W, i, x)` (pleasure_boat.c; as `FUN_021C87A0`).
    fn fun_021c87b8(&mut self, w: u32, i: u32, x: u32) -> u32;
    /// C: `FUN_021C87C8(W, i)` (pleasure_boat.c).
    fn fun_021c87c8(&mut self, w: u32, i: u32) -> u32;
    /// C: `FUN_021CF498(x, event, a)`.
    fn fun_021cf498(&mut self, x: u32, event: u32, a: u32);
}

// ------------------------------------------------------------------ row 10.27: boat, save block, 493, weather

/// C: `FUN_overlay_d_10__02166658`: the boat record word `W + 0x14 + 0x1C * i`, or 10 without a boat work.
pub fn boat_slot_word<H: MemEnv>(h: &mut H, w: u32, i: u32) -> u32 {
    if w == 0 {
        return 10;
    }
    h.r32(w.wrapping_add(i.wrapping_mul(0x1C)).wrapping_add(0x14))
}

/// C: `FUN_overlay_d_10__0216666c`: the boat wait event (work 0x20: W at +0x18, the result var at +0x1C, the time at +8).
pub fn boat_event_create<H: Misc10bHooks>(h: &mut H, gamesys: u32, w: u32, var: u32) -> u32 {
    let ev = h.fun_020122c0(gamesys, 0, EVENT_BOAT, BOAT_EVENT_SIZE);
    let e = h.fun_020124e8(ev);
    h.w32(e + 0x18, w);
    h.w32(e + 0x1C, var);
    h.fun_02161cdc(e, gamesys, 1, 0);
    let t = boat_time(h, w);
    h.w32(e + 8, t);
    ev
}

/// C: `sub_021666b8` (GMEVENT of [`boat_event_create`]): seq 0 runs the app 0xCB; seq 1 stores `E[0x14] == 2` into the var,
/// sets the boat time to `E[0x10]` and ends.
pub fn event_boat<H: Misc10bHooks>(h: &mut H, event: u32, seq: u32, _work: u32) -> u32 {
    let e = h.fun_020124e8(event);
    match h.r32(seq) {
        0 => {
            h.fun_02012444(event, PROC_BOAT.0, PROC_BOAT.1, e);
            let s = h.r32(seq);
            h.w32(seq, s.wrapping_add(1));
            0
        }
        1 => {
            let done = u16::from(h.r32(e + 0x14) == 2);
            let var = h.r32(e + 0x1C);
            h.w16(var, done);
            let (w, t) = (h.r32(e + 0x18), h.r32(e + 0x10));
            boat_time_set(h, w, t);
            1
        }
        _ => 0,
    }
}

/// C: `FUN_overlay_d_10__0216670c`: the boat time, s16 W+4 sign-extended.
pub fn boat_time<H: MemEnv>(h: &mut H, w: u32) -> u32 {
    h.r16(w + 4) as i16 as i32 as u32
}

/// C: `FUN_overlay_d_10__02166714`: set the boat time to `t` (u16 W+4); the unit count `(s16 W+4) / 0x546` (signed division,
/// `_s32_div_f` inlined) replaces W[8] when W[8] is not above it (signed).
pub fn boat_time_set<H: MemEnv>(h: &mut H, w: u32, t: u32) {
    h.w16(w + 4, t as u16);
    let q = i32::from(h.r16(w + 4) as i16) / BOAT_TIME_UNIT;
    if h.r32(w + 8) as i32 <= q {
        h.w32(w + 8, q as u32);
    }
}

/// C: `sub_02166734` (command 425): query the boat work: operand `i`, sub-command (u8 script byte) 4..8, operand `x`; the
/// answer goes into a var.
pub fn cmd_425<H: Misc10bHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    let p = h.fun_02012ed0(gd);
    let i = h.fun_02159b10(vm, ctx);
    let k = h.fun_02011330(vm);
    let x = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let v = match k {
        4 => {
            let w = h.r32(p);
            boat_slot_word(h, w, i)
        }
        5 => {
            let w = h.r32(p);
            h.fun_021c87a0(w, i)
        }
        6 => {
            let w = h.r32(p);
            h.fun_021c87ac(w, i)
        }
        7 => {
            let w = h.r32(p);
            h.fun_021c87b8(w, i, x)
        }
        8 => {
            let w = h.r32(p);
            u32::from(h.fun_021c87c8(w, i) != 0)
        }
        _ => 0,
    };
    h.w16(var, v as u16);
    0
}

/// C: `sub_021667e0` (command 427): start the boat wait event into a var.
pub fn cmd_427<H: Misc10bHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    let p = h.fun_02012ed0(gd);
    let var = h.fun_02159ae8(vm, ctx);
    let w = h.r32(p);
    let ev = boat_event_create(h, gs, w, var);
    if ev != 0 {
        h.fun_0215897c(sw, ev);
    }
    1
}

/// C: `FUN_02166824`: byte `3 + i` of the record when `i < R[1]`, else 0xB.
pub fn rec20_byte<H: MemEnv>(h: &mut H, r: u32, i: u32) -> u8 {
    if i < u32::from(h.r8(r + 1)) {
        h.r8(r.wrapping_add(i).wrapping_add(3))
    } else {
        0xB
    }
}

/// C: `FUN_overlay_d_10__02166834`: field `k` of the record: 0 -> byte 1, 1 -> byte 0, 2 -> byte 2, else 0.
pub fn rec20_field<H: MemEnv>(h: &mut H, r: u32, k: u32) -> u8 {
    match k {
        0 => h.r8(r + 1),
        1 => h.r8(r),
        2 => h.r8(r + 2),
        _ => 0,
    }
}

/// C: `FUN_overlay_d_10__02166854`: command 480's event (work 0x128): byte 0 = `v`, the three result vars at +0x11C /
/// +0x120 / +0x124, the 0xE8 flags of the save block at +0x30.., the u16 of each of its first (at most 20) records at +8...
pub fn event_480_create<H: Misc10bHooks>(
    h: &mut H,
    gamesys: u32,
    v: u32,
    p1: u32,
    p2: u32,
    p3: u32,
) -> u32 {
    let ev = h.fun_020122c0(gamesys, 0, EVENT_480, EVENT_480_SIZE);
    let e = h.fun_020124e8(ev);
    h.w8(e, v as u8);
    h.w32(e + 0x124, p3);
    h.w32(e + 0x11C, p1);
    h.w32(e + 0x120, p2);
    let gd = h.fun_020120f4(gamesys);
    let save = h.fun_02012ebc(gd);
    let blk = h.fun_02009470(save);
    for i in 1..0xE9u32 {
        let f = h.fun_02009794(blk, i);
        h.w8(e + 0x2F + i, u8::from(f != 0));
    }
    let n = (h.fun_020095c8(blk) as i32).min(0x14);
    let mut i = 0i32;
    while i < n {
        let r = h.fun_0200973c(blk, i as u32);
        let x = h.fun_0200856c(r);
        h.w16(e + 8 + 2 * i as u32, x as u16);
        i += 1;
    }
    while i < 0x14 {
        h.w16(e + 8 + 2 * i as u32, 0);
        i += 1;
    }
    ev
}

/// C: `sub_02166908` (GMEVENT of [`event_480_create`]): seq 0 runs the app 0xDA; seq 1 copies the result (`E[4] == 1`: 1,
/// byte E+1, u16 E+2; else zeros) into the three vars and ends.
pub fn event_480<H: Misc10bHooks>(h: &mut H, event: u32, seq: u32, _work: u32) -> u32 {
    let e = h.fun_020124e8(event);
    match h.r32(seq) {
        0 => {
            h.fun_02012444(event, PROC_480.0, PROC_480.1, e);
            let s = h.r32(seq);
            h.w32(seq, s.wrapping_add(1));
            0
        }
        1 => {
            let (a, b, c) = (h.r32(e + 0x124), h.r32(e + 0x11C), h.r32(e + 0x120));
            if h.r32(e + 4) == 1 {
                h.w16(a, 1);
                let x = h.r8(e + 1);
                h.w16(b, u16::from(x));
                let y = h.r16(e + 2);
                h.w16(c, y);
            } else {
                h.w16(a, 0);
                h.w16(b, 0);
                h.w16(c, 0);
            }
            1
        }
        _ => 0,
    }
}

/// C: `sub_02166980` (command 477): byte `i` (u8 script byte) of the 20-record save-block record into a var.
pub fn cmd_477<H: Misc10bHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    let r = h.fun_02012f38(gd);
    let i = h.fun_02011330(vm) & 0xFF;
    let var = h.fun_02159ae8(vm, ctx);
    let v = rec20_byte(h, r, i);
    h.w16(var, u16::from(v));
    0
}

/// C: `FUN_overlay_d_10__021669c0` (command 478): field `k` of the record into a var.
pub fn cmd_478<H: Misc10bHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    let r = h.fun_02012f38(gd);
    let k = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let v = rec20_field(h, r, k);
    h.w16(var, u16::from(v));
    0
}

/// C: `FUN_overlay_d_10__02166a00` (command 480): start [`event_480`] with an operand and three vars.
pub fn cmd_480<H: Misc10bHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    h.fun_02012f38(gd);
    let v = h.fun_02159b10(vm, ctx);
    let p1 = h.fun_02159ae8(vm, ctx);
    let p2 = h.fun_02159ae8(vm, ctx);
    let p3 = h.fun_02159ae8(vm, ctx);
    let ev = event_480_create(h, gs, v, p1, p2, p3);
    if ev != 0 {
        h.fun_0215897c(sw, ev);
    }
    1
}

/// C: `sub_02166d30` (command 493): the app 0x6E on a `FUN_02024044` record. Work (0x1C, cleared): +0 the mode (operand),
/// +4 / +8 / +0xC the three result vars (the third cleared now), +0x10 the record, +0x14 / +0x18 the record kind / the
/// "store back" flag by mode (0: 1 / -, 1: 2 / 1, 2: 3 / 1, 3: 4 / 1, 4: - / 0, 5: - / 0 with record kind 1). A mode above 5
/// starts the event with a null record.
pub fn cmd_493<H: Misc10bHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_0215a5e4(ctx);
    let sw = h.fun_0215a5f4(ctx);
    let mode = h.fun_02159b10(vm, ctx);
    let v1 = h.fun_02159ae8(vm, ctx);
    let v2 = h.fun_02159ae8(vm, ctx);
    let v3 = h.fun_02159ae8(vm, ctx);
    let ev = h.fun_020122c0(gs, 0, EVENT_493, EVENT_493_SIZE);
    let e = h.fun_020124e8(ev);
    for o in (0..EVENT_493_SIZE).step_by(4) {
        h.w32(e + o, 0);
    }
    h.w32(e, mode);
    h.w32(e + 4, v1);
    h.w32(e + 8, v2);
    h.w32(e + 0xC, v3);
    h.w16(v3, 0);
    let kind = match mode {
        0 => {
            h.w32(e + 0x14, 1);
            h.w32(e + 0x18, 1);
            2
        }
        1 => {
            h.w32(e + 0x14, 2);
            h.w32(e + 0x18, 1);
            2
        }
        2 => {
            h.w32(e + 0x14, 3);
            h.w32(e + 0x18, 1);
            2
        }
        3 => {
            h.w32(e + 0x14, 4);
            h.w32(e + 0x18, 1);
            2
        }
        4 => {
            h.w32(e + 0x18, 0);
            0
        }
        5 => {
            h.w32(e + 0x18, 0);
            1
        }
        _ => {
            h.w32(e + 0x10, 0);
            h.fun_0215897c(sw, ev);
            return 1;
        }
    };
    let save = h.fun_02012ebc(gd);
    let rec = h.fun_02024044(kind, 0, 0, 0, save, 1);
    h.w32(e + 0x10, rec);
    if h.r32(e + 0x18) != 0 {
        let gd2 = h.fun_020120f4(gs);
        let save2 = h.fun_02012ebc(gd2);
        let x = h.fun_0200e6f0(save2);
        let k = h.r32(e + 0x14);
        let mut v = [0u32; 3];
        h.fun_0200e6fc(x, k, &mut v);
        h.fun_02024108(rec, &v);
    }
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_02166e50` (GMEVENT of command 493): seq 0 runs the app 0x6E (a null record ends at once); seq 1 copies the
/// result into the vars (by mode: 5 -> the two u16 of `FUN_02024128`; 4 -> `FUN_02024124`, 0; else the 12-byte record's
/// u16 at +4 / +6, stored back with `FUN_0200E714` when +0x18 is set), sets the third var (0 when `FUN_0202411C` says so,
/// else 1), frees the record and ends.
pub fn event_493<H: Misc10bHooks>(h: &mut H, event: u32, seq: u32, _work: u32) -> u32 {
    let e = h.fun_020124e8(event);
    let rec = h.r32(e + 0x10);
    match h.r32(seq) {
        0 => {
            if rec == 0 {
                return 1;
            }
            h.fun_02012444(event, PROC_493.0, PROC_493.1, rec);
            let s = h.r32(seq);
            h.w32(seq, s.wrapping_add(1));
            0
        }
        1 => {
            let flag = if h.fun_0202411c(rec) != 0 {
                0
            } else {
                let second = match h.r32(e) {
                    5 => {
                        let mut w = 0u32;
                        h.fun_02024128(rec, &mut w);
                        let a = h.r32(e + 4);
                        h.w16(a, w as u16);
                        (w >> 16) as u16
                    }
                    4 => {
                        let v = h.fun_02024124(rec);
                        let a = h.r32(e + 4);
                        h.w16(a, v as u16);
                        0
                    }
                    _ => {
                        let mut v = [0u32; 3];
                        h.fun_02024134(rec, &mut v);
                        if h.r32(e + 0x18) != 0 {
                            let gs = h.fun_020124e4(event);
                            let gd = h.fun_020120f4(gs);
                            let save = h.fun_02012ebc(gd);
                            let x = h.fun_0200e6f0(save);
                            let k = h.r32(e + 0x14);
                            h.fun_0200e714(x, k, &v);
                        }
                        let a = h.r32(e + 4);
                        h.w16(a, v[1] as u16);
                        (v[1] >> 16) as u16
                    }
                };
                let b = h.r32(e + 8);
                h.w16(b, second);
                1
            };
            let c = h.r32(e + 0xC);
            h.w16(c, flag);
            let rec = h.r32(e + 0x10);
            h.fun_020240fc(rec);
            1
        }
        _ => 0,
    }
}

/// C: `sub_02166f1c` (command 310): set the field weather (operand `w`, 0xFFFF = the zone's weather): at once when the
/// second operand is 0, else as a change; then `FUN_020129A0(gd, w & 0xFF)`.
pub fn cmd_310<H: Misc10bHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_0215a5e4(ctx);
    let fm = h.fun_0201210c(gs);
    let weather = h.fun_02188c6c(fm);
    let mut w = h.fun_02159b10(vm, ctx);
    let change = h.fun_02159b10(vm, ctx);
    if w == WEATHER_ZONE {
        let zone = h.fun_02188c9c(fm);
        w = h.fun_0202c794(gs, zone);
    }
    if change != 0 {
        h.fun_021a1be4(weather, w);
    } else {
        h.fun_021a1bdc(weather, w);
    }
    h.fun_020129a0(gd, w & 0xFF);
    0
}

// ------------------------------------------------------------------ row 10.34: the Entree Forest symbol encounters

/// C: `FUN_overlay_d_10__0216ac40` (command 558): `FUN_021664CC` record (or nothing) -> `FUN_0217C37C(fm, word, rec, rec[0x50]
/// & 0x3F, (rec[0x50] >> 18) & 0x3F)`, then free the record.
pub fn cmd_558<H: Misc10bHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    h.fun_02012138(gs);
    let heap = h.fun_0215a5cc(ctx);
    let mut word = 0u32;
    let rec = h.fun_021664cc(heap, gs, &mut word);
    if rec != 0 {
        let x = h.r32(rec + 0x50);
        h.fun_0217c37c(fm, word, rec, x & 0x3F, (x << 8) >> 26);
        h.fun_020307b0(rec);
    }
    0
}

/// C: `FUN_overlay_d_10__0216ac90` (command 560): for the OBJ of an id, `FUN_0200E18C(Y, FUN_0217C294(obj))` and
/// [`symbol_notify`].
pub fn cmd_560<H: Misc10bHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let id = h.fun_02159b10(vm, ctx);
    let gd = h.fun_0215a5e4(ctx);
    let save = h.fun_02012ebc(gd);
    let y = h.fun_0200e0b0(save);
    let obj = symbol_obj(h, ctx, id);
    if obj != 0 {
        let v = h.fun_0217c294(obj);
        h.fun_0200e18c(y, v);
        symbol_notify(h, ctx);
    }
    0
}

/// C: `FUN_overlay_d_10__0216accc` (command 556): `FUN_0216628C(Y, FUN_0217C294(obj))` for the OBJ of an id into a var
/// (0 without the OBJ), then [`symbol_notify`].
pub fn cmd_556<H: Misc10bHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let id = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let gd = h.fun_0215a5e4(ctx);
    let save = h.fun_02012ebc(gd);
    let y = h.fun_0200e0b0(save);
    let obj = symbol_obj(h, ctx, id);
    if obj != 0 {
        let v = h.fun_0217c294(obj);
        let r = h.fun_0216628c(y, v);
        h.w16(var, r as u16);
        symbol_notify(h, ctx);
    } else {
        h.w16(var, 0);
    }
    0
}

/// C: `FUN_overlay_d_10__0216ad1c` (command 557): the symbol record of the OBJ of an id ([`symbol_record`]) into word-set
/// slot `i` (`FUN_0201EE84`), then free it.
pub fn cmd_557<H: Misc10bHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let id = h.fun_02159b10(vm, ctx);
    let i = h.fun_02159b10(vm, ctx);
    let obj = symbol_obj(h, ctx, id);
    if obj != 0 {
        let heap = h.fun_0215a5cc(ctx);
        let mut word = 0u32;
        let rec = symbol_record(h, ctx, obj, heap, &mut word);
        if rec != 0 {
            let sw = h.fun_0215a5f4(ctx);
            let ws = h.fun_02158f74(sw);
            h.fun_0201ee84(ws, i, rec);
            h.fun_020307b0(rec);
        }
    }
    0
}

/// C: `FUN_overlay_d_10__0216ad70` (command 555): query the symbol state into a var by sub-command: 0 = intrude state is 4,
/// 1 / 2 = the symbol_map flags `FUN_02166618` / `FUN_0216662C`, 3 / 4 = the low 11 bits / bits 23..28 of the packed symbol
/// word of the talked-to OBJ ([`symbol_word_of_talker`]); others store nothing.
pub fn cmd_555<H: Misc10bHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let k = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_0215a5e4(ctx);
    let save = h.fun_02012ebc(gd);
    h.fun_0200e0b0(save);
    let x = h.fun_02012138(gs);
    let v = match k {
        0 => u32::from(h.fun_021767a0(x, gd) == 4),
        1 => {
            let f = h.fun_02012fb0(gd);
            h.fun_02166618(f)
        }
        2 => {
            let f = h.fun_02012fb0(gd);
            h.fun_0216662c(f)
        }
        3 => {
            let mut w = 0u32;
            symbol_word_of_talker(h, ctx, &mut w);
            w & 0x7FF
        }
        4 => {
            let mut w = 0u32;
            symbol_word_of_talker(h, ctx, &mut w);
            (w << 3) >> 26
        }
        _ => return 0,
    };
    h.w16(var, v as u16);
    0
}

/// C: `FUN_overlay_d_10__0216ae10` (command 554): sub-command 1 with the `FUN_0216662C` flag set warps out of the forest
/// (`FUN_02184D38` to zone 279 at the intrude position shifted by the player's x - 0x3E << 14); else the forest warp event
/// [`forest_warp_start`] (with `FUN_021665F0(gs, FUN_02012FB0(gd), k)` unless k = 9, which passes `FUN_02012FB0(gd)`).
pub fn cmd_554<H: Misc10bHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_0215a5e4(ctx);
    let k = h.fun_02159b10(vm, ctx);
    let mut x = h.fun_02012fb0(gd);
    let ev = if k == 1 && h.fun_0216662c(x) != 0 {
        let fm = h.fun_0201210c(gs);
        let player = h.fun_02188cb4(fm);
        let c = h.fun_02012138(gs);
        let mut p = [0u32; 3];
        h.fun_021a3038(player, &mut p);
        let mut pos = [0u32; 3];
        h.fun_021767f0(c, gd, &mut pos);
        pos[0] = pos[0].wrapping_add(p[0].wrapping_sub(WARP_554_DX));
        h.fun_02184d38(gs, fm, ZONE_554, &pos, 1, 0)
    } else {
        if k != 9 {
            let f = h.fun_02012fb0(gd);
            x = h.fun_021665f0(gs, f, k);
        }
        forest_warp_start(h, gs, k, x)
    };
    h.fun_0215897c(sw, ev);
    1
}

/// C: `FUN_overlay_d_10__0216aed0`: the OBJ of an id (`FUN_0216DE94(FUN_02188C7C(FUN_0201210C(gs)), id)`) or null.
pub fn symbol_obj<H: Misc10bHooks>(h: &mut H, ctx: u32, id: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    let objsys = h.fun_02188c7c(fm);
    h.fun_0216de94(objsys, id)
}

/// C: `FUN_overlay_d_10__0216aee8`: the packed symbol word of the OBJ whose id is script variable 0x8011 (`FUN_0217C274`).
pub fn symbol_word_of_talker<H: Misc10bHooks>(h: &mut H, ctx: u32, out: &mut u32) {
    let sw = h.fun_0215a5f4(ctx);
    let gd = h.fun_0215a5e4(ctx);
    let id = h.fun_0215901c(sw, gd, VAR_SYMBOL_OBJ);
    let obj = symbol_obj(h, ctx, id);
    h.fun_0217c274(out, obj);
}

/// C: `FUN_overlay_d_10__0216af1c`: the packed word of `obj` into `word`, then the record `FUN_0217C2A0(4, gd, &word)` (the
/// heap argument is unused).
pub fn symbol_record<H: Misc10bHooks>(
    h: &mut H,
    ctx: u32,
    obj: u32,
    _heap: u32,
    word: &mut u32,
) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    h.fun_0217c274(word, obj);
    h.fun_0217c2a0(4, gd, word)
}

/// C: `FUN_overlay_d_10__0216af3c`: tell the intrude comm (when there is one) `FUN_0217688C(comm, FUN_02012FB0(gd))`.
pub fn symbol_notify<H: Misc10bHooks>(h: &mut H, ctx: u32) {
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_0215a5e4(ctx);
    let save = h.fun_02012ebc(gd);
    h.fun_0200e0b0(save);
    let x = h.fun_02012138(gs);
    let comm = h.fun_02158344(x);
    if comm != 0 {
        let v = h.fun_02012fb0(gd);
        h.fun_0217688c(comm, v);
    }
}

/// C: `FUN_overlay_d_10__0216af74`: the symbol battle event (work 0x3C, cleared): +0 `a`, +0xC the mon, +0x20 the packed
/// word `word`, +0x24 the species (u16), +0x28 the result var, +0x2C `FUN_0203F1FC(mon, 0xDC)`; a failed `FUN_0200E2BC`
/// check sets the word's low 11 bits to 0x28B; `FUN_021AA304(E + 0x10, gd, a)`.
pub fn symbol_battle_create<H: Misc10bHooks>(
    h: &mut H,
    gamesys: u32,
    a: u32,
    mon: u32,
    word: u32,
    var: u32,
    species: u32,
) -> u32 {
    let gd = h.fun_020120f4(gamesys);
    let ev = h.fun_020122c0(gamesys, 0, EVENT_SYMBOL_BATTLE, SYMBOL_BATTLE_SIZE);
    let e = h.fun_020124e8(ev);
    for o in (0..SYMBOL_BATTLE_SIZE).step_by(4) {
        h.w32(e + o, 0);
    }
    h.w32(e, a);
    h.w32(e + 0x28, var);
    h.w16(e + 0x24, species as u16);
    h.w32(e + 0xC, mon);
    h.w32(e + 0x20, word);
    let v = h.fun_0203f1fc(mon, SPECIES_LIMIT);
    h.w16(e + 0x2C, v as u16);
    let save = h.fun_02012ebc(gd);
    if h.fun_0200e2bc(save, &word, species & 0xFFFF, 0) == 0 {
        let w = h.r32(e + 0x20);
        h.w32(e + 0x20, (w & 0xFFFF_F800) | SYMBOL_DEFAULT_LOW);
    }
    h.fun_021aa304(e + 0x10, gd, a);
    ev
}

/// The packed symbol word of a mon: species (id 5) in bits 0..10, `FUN_02017E1C(mon, 0x6F, 0)` in bits 23..28, over `w`.
fn pack_symbol_word<H: Misc10bHooks>(h: &mut H, mon: u32, w: u32) -> u32 {
    let s = h.fun_02017e1c(mon, 5, 0);
    let w = (w & 0xFFFF_F800) | (s & 0x7FF);
    let f = h.fun_02017e1c(mon, 0x6F, 0);
    (w & 0xE07F_FFFF) | ((f << 26) >> 3)
}

/// C: `sub_0216b004` (GMEVENT of [`symbol_battle_create`]): the symbol battle, seq 0..8 (the battle, the overlay-0x62 app,
/// the overlay-0xE3 app, the child events); seq > 8 frees the mon and the app work and ends.
pub fn event_symbol_battle<H: Misc10bHooks>(h: &mut H, event: u32, seq: u32, e: u32) -> u32 {
    let gs = h.fun_020124e4(event);
    let gd = h.fun_020120f4(gs);
    let s = h.r32(seq);
    let child = match s {
        0 => Some(h.fun_02029bcc(gs, SYMBOL_BATTLE_ID)),
        1 => {
            let mon = h.r32(e + 0xC);
            let species = u32::from(h.r16(e + 0x24));
            let w = pack_symbol_word(h, mon, 0);
            let save = h.fun_02012ebc(gd);
            if h.fun_0200e2bc(save, &w, species, 1) == 0 {
                let x = h.r32(e + 0x20);
                h.w32(e + 0x20, (x & 0xE07F_FFFF) | (6 << 24));
            }
            let a = h.r32(e);
            let x = h.fun_02189a08(a);
            h.fun_021cf498(x, event, 0);
            None
        }
        2 => None,
        3 => {
            let a = h.r32(e);
            Some(h.fun_020144dc(gs, a))
        }
        4 => {
            let save = h.fun_02012ebc(gd);
            let zukan = h.fun_0201292c(gd);
            let species = u32::from(h.r16(e + 0x24));
            h.fun_02007e50(zukan, SYMBOL_ZUKAN_ID, 1, species);
            h.fun_02034ac4(OVERLAY_SYMBOL_A);
            let status = h.fun_02012944(gd);
            let zukan = h.fun_0201292c(gd);
            let a = h.fun_0200873c(save);
            let b = h.fun_0200ca10(save);
            let species = u32::from(h.r16(e + 0x24));
            let mon = h.r32(e + 0xC);
            let work = h.fun_021b6100(gd, mon, e + 0x10, status, zukan, a, b, species);
            h.w32(e + 4, work);
            h.fun_02012444(event, 0xFFFF_FFFF, PROC_SYMBOL_A, work);
            None
        }
        5 => {
            let work = h.r32(e + 4);
            let mut res = h.fun_021b6158(work);
            let mon = h.r32(e + 0xC);
            let species = u32::from(h.r16(e + 0x24));
            let w = pack_symbol_word(h, mon, 0);
            let save = h.fun_02012ebc(gd);
            if h.fun_0200e2bc(save, &w, species, 1) == 0 {
                res = 0;
            } else {
                let save = h.fun_02012ebc(gd);
                let w2 = h.r32(e + 0x20);
                let species = u32::from(h.r16(e + 0x24));
                if h.fun_0200e2bc(save, &w2, species, 0) == 0 {
                    res = 0;
                }
            }
            let var = h.r32(e + 0x28);
            h.w16(var, u16::from(res == 1));
            h.fun_02034a5c(OVERLAY_SYMBOL_A);
            h.fun_02034ac4(OVERLAY_SYMBOL_B);
            let species = u32::from(h.r16(e + 0x24));
            let mon = h.r32(e + 0xC);
            let work = h.fun_021b95a0(gd, res, mon, species);
            h.w32(e + 8, work);
            h.fun_02012444(event, 0xFFFF_FFFF, PROC_SYMBOL_B, work);
            let zukan = h.fun_0201292c(gd);
            let species = u32::from(h.r16(e + 0x24));
            h.fun_02007f1c(zukan, SYMBOL_ZUKAN_ID, 1, species);
            None
        }
        6 => {
            let work = h.r32(e + 8);
            h.fun_021b95c8(work);
            h.fun_02034a5c(OVERLAY_SYMBOL_B);
            Some(h.fun_02029c00(gs))
        }
        7 => Some(h.fun_020145cc(gs)),
        8 => Some(h.fun_02029cc0(gs)),
        _ => {
            let mon = h.r32(e + 0xC);
            h.fun_020307b0(mon);
            let work = h.r32(e + 4);
            h.fun_020307b0(work);
            return 1;
        }
    };
    if let Some(c) = child {
        h.fun_02012374(event, c);
    }
    let s = h.r32(seq);
    h.w32(seq, s.wrapping_add(1));
    0
}

/// C: `FUN_overlay_d_10__0216b24c`: the intrude wait event (work 8, cleared): +0 the result var `var`, +4 the byte `x`.
pub fn intrude_wait_create<H: Misc10bHooks>(h: &mut H, gamesys: u32, var: u32, x: u32) -> u32 {
    h.fun_020120f4(gamesys);
    let ev = h.fun_020122c0(gamesys, 0, EVENT_INTRUDE_WAIT, INTRUDE_WAIT_SIZE);
    let e = h.fun_020124e8(ev);
    h.w32(e, 0);
    h.w32(e + 4, 0);
    h.w32(e, var);
    h.w8(e + 4, x as u8);
    ev
}

/// C: `sub_0216b280` (GMEVENT of [`intrude_wait_create`]): without the intrude comm, var = 0 and end; else seq 0 waits for
/// `FUN_02176850(comm, x) == 1`, seq 1 for `FUN_02176870(comm) == 1`, seq 2 sets var = 1 and ends (a null var is skipped).
pub fn event_intrude_wait<H: Misc10bHooks>(h: &mut H, event: u32, seq: u32, e: u32) -> u32 {
    let gs = h.fun_020124e4(event);
    let x = h.fun_02012138(gs);
    let comm = h.fun_02158344(x);
    if comm == 0 {
        let var = h.r32(e);
        if var != 0 {
            h.w16(var, 0);
        }
        return 1;
    }
    match h.r32(seq) {
        0 => {
            let b = u32::from(h.r8(e + 4));
            if h.fun_02176850(comm, b) == 1 {
                let s = h.r32(seq);
                h.w32(seq, s.wrapping_add(1));
            }
        }
        1 => {
            if h.fun_02176870(comm) == 1 {
                let s = h.r32(seq);
                h.w32(seq, s.wrapping_add(1));
            }
        }
        2 => {
            let var = h.r32(e);
            if var != 0 {
                h.w16(var, 1);
            }
            return 1;
        }
        _ => {}
    }
    0
}

/// The edge position of warp kind `k` from `src` (VecFx32): 0 -> z = 0x188000, 1 -> z = 0x98000, 2 -> x = 0x1A8000, 3 ->
/// x = 0x58000, others unchanged.
pub fn warp_pos(src: [u32; 3], k: u32) -> [u32; 3] {
    let mut d = src;
    match k {
        0 => d[2] = 0x62 << 14,
        1 => d[2] = 0x26 << 14,
        2 => d[0] = 0x6A << 14,
        3 => d[0] = 0x16 << 14,
        _ => {}
    }
    d
}

/// C: `FUN_overlay_d_10__0216b2e4`: copy the position `src` (VecFx32, 12 bytes) to `dst` and move it to the forest edge of
/// warp kind `k` ([`warp_pos`]).
pub fn symbol_warp_pos<H: MemEnv>(h: &mut H, src: u32, k: u32, dst: u32) {
    let s = [h.r32(src), h.r32(src + 4), h.r32(src + 8)];
    let d = warp_pos(s, k);
    for (i, v) in d.iter().enumerate() {
        h.w32(dst + 4 * i as u32, *v);
    }
}

/// C: `FUN_overlay_d_10__0216b340`: the forest warp event (work 0x24, cleared): +8 the position, +0x14 the zone `k` (u16),
/// +0x16 the byte `x`, +0x1C `se`, +0x20 = 1 when the intrude state is 4.
pub fn forest_warp_create<H: Misc10bHooks>(
    h: &mut H,
    gamesys: u32,
    se: u32,
    pos: &[u32; 3],
    k: u32,
    x: u32,
) -> u32 {
    let gd = h.fun_020120f4(gamesys);
    let ev = h.fun_020122c0(gamesys, 0, EVENT_FOREST_WARP, FOREST_WARP_SIZE);
    let e = h.fun_020124e8(ev);
    for o in (0..FOREST_WARP_SIZE).step_by(4) {
        h.w32(e + o, 0);
    }
    for (i, v) in pos.iter().enumerate() {
        h.w32(e + 8 + 4 * i as u32, *v);
    }
    h.w16(e + 0x14, k as u16);
    h.w32(e, 0);
    h.w8(e + 0x16, x as u8);
    h.w32(e + 0x1C, se);
    let c = h.fun_02012138(gamesys);
    if h.fun_021767a0(c, gd) == 4 {
        h.w32(e + 0x20, 1);
    }
    ev
}

/// C: `FUN_overlay_d_10__0216b3a8`: the forest warp of kind `k` from the player's position ([`warp_pos`]); the SE plays
/// unless k = 9.
pub fn forest_warp_start<H: Misc10bHooks>(h: &mut H, gamesys: u32, k: u32, x: u32) -> u32 {
    let fm = h.fun_0201210c(gamesys);
    h.fun_020120f4(gamesys);
    let player = h.fun_02188cb4(fm);
    let a = h.fun_021a3074(player);
    let p = h.fun_0216d888(a);
    let src = [h.r32(p), h.r32(p + 4), h.r32(p + 8)];
    let pos = warp_pos(src, k);
    let se = u32::from(k != 9);
    forest_warp_create(h, gamesys, se, &pos, k, x)
}

/// C: `FUN_overlay_d_10__0216b3ec` (GMEVENT of [`forest_warp_create`]): pause the OBJs, wait for the intrude comm (unless
/// state 4), fade out (+ SE), change the map, fade in, resume; a failed wait sets the var to 0 and skips to the end.
pub fn event_forest_warp<H: Misc10bHooks>(h: &mut H, event: u32, seq: u32, e: u32) -> u32 {
    let gs = h.fun_020124e4(event);
    h.fun_02012138(gs);
    let gd = h.fun_020120f4(gs);
    let fm = h.fun_0201210c(gs);
    let s = h.r32(seq);
    let child = match s {
        0 => {
            let objsys = h.fun_02188c7c(fm);
            h.fun_0216d9a0(objsys);
            if h.r32(e + 0x20) == 1 {
                h.w32(seq, 2);
                return 0;
            }
            let x = u32::from(h.r8(e + 0x16));
            h.w32(seq, s);
            intrude_wait_create(h, gs, e + 0x18, x)
        }
        1 => {
            if h.r16(e + 0x18) == 0 {
                let var = h.r32(e);
                if var != 0 {
                    h.w16(var, 0);
                }
                h.w32(seq, 6);
                return 0;
            }
            h.w32(seq, s.wrapping_add(1));
            return 0;
        }
        2 => {
            if h.r32(e + 0x1C) != 0 {
                h.fun_020061e4(SE_FOREST_WARP);
            }
            h.fun_021be8d4(gs, fm, 9, 0)
        }
        3 => {
            let x = u32::from(h.r8(e + 0x16));
            let zone = h.fun_021665d0(gs, x);
            h.w16(e + 4, zone as u16);
            let x = u32::from(h.r8(e + 0x16));
            h.fun_02012fa4(gd, x);
            let dir = u32::from(h.r16(e + 0x14));
            let zone = u32::from(h.r16(e + 4));
            let pos = [h.r32(e + 8), h.r32(e + 0xC), h.r32(e + 0x10)];
            h.fun_021854b0(gs, fm, zone, &pos, dir)
        }
        4 => {
            let fm2 = h.fun_0201210c(gs);
            let objsys = h.fun_02188c7c(fm2);
            if h.fun_0219c630(objsys) == 1 {
                return 0;
            }
            h.fun_021be92c(gs, fm, 9, 0, 0, 0, 0)
        }
        5 => {
            let var = h.r32(e);
            if var != 0 {
                h.w16(var, 1);
            }
            h.w32(seq, 6);
            return 0;
        }
        6 => {
            let objsys = h.fun_02188c7c(fm);
            h.fun_0216d9e4(objsys);
            return 1;
        }
        _ => return 0,
    };
    h.fun_02012374(event, child);
    let s = h.r32(seq);
    h.w32(seq, s.wrapping_add(1));
    0
}

/// Run the command handler at `handler` (Thumb bit allowed) when it is one of this area's; `None` otherwise.
pub fn dispatch<H: Misc10bHooks>(h: &mut H, handler: u32, vm: u32, ctx: u32) -> Option<u32> {
    Some(match handler & !1 {
        0x0216_6F1C => cmd_310(h, vm, ctx),
        0x0216_6734 => cmd_425(h, vm, ctx),
        0x0216_67E0 => cmd_427(h, vm, ctx),
        0x0216_6980 => cmd_477(h, vm, ctx),
        0x0216_69C0 => cmd_478(h, vm, ctx),
        0x0216_6A00 => cmd_480(h, vm, ctx),
        0x0216_6D30 => cmd_493(h, vm, ctx),
        0x0216_AE10 => cmd_554(h, vm, ctx),
        0x0216_AD70 => cmd_555(h, vm, ctx),
        0x0216_ACCC => cmd_556(h, vm, ctx),
        0x0216_AD1C => cmd_557(h, vm, ctx),
        0x0216_AC40 => cmd_558(h, vm, ctx),
        0x0216_AC90 => cmd_560(h, vm, ctx),
        _ => return None,
    })
}

/// Run the GMEVENT function `f(event, seq, work)` (Thumb bit set) when this area installed it; `None` otherwise.
pub fn call_event<H: Misc10bHooks>(
    h: &mut H,
    f: u32,
    event: u32,
    seq: u32,
    work: u32,
) -> Option<u32> {
    Some(match f {
        EVENT_BOAT => event_boat(h, event, seq, work),
        EVENT_480 => event_480(h, event, seq, work),
        EVENT_493 => event_493(h, event, seq, work),
        EVENT_SYMBOL_BATTLE => event_symbol_battle(h, event, seq, work),
        EVENT_INTRUDE_WAIT => event_intrude_wait(h, event, seq, work),
        EVENT_FOREST_WARP => event_forest_warp(h, event, seq, work),
        _ => return None,
    })
}

use crate::hooks::MemEnv;

#[cfg(test)]
mod tests;
