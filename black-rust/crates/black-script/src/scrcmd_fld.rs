//! F8 (second batch): overlay-21 field script commands, Thumb code 0x021AE948-0x021B0084 (the first part of F1-SPLIT row
//! 21.42): trainer-eye / trainer-battle starts, field events and sounds, the WORDSET commands 76..99 / 538 / 539, name / string
//! queries and warps (80 command ids, 97 functions).
//!
//! Every function of the area is specced in `tools/rom-diff/specs/black-script/scrcmd_fld.spec`; its header has the command-id
//! table and a one-line role per function. All 97 are ported here, listed in [`PORTED`] and checked with
//! `tools/check-fn.sh black-script scrcmd_fld` (F8c). [`COMMANDS`] maps the command ids to their handler addresses and
//! [`dispatch`] runs a handler by its table address; [`call_wait`] / [`init_proc`] route the wait callbacks and init procs
//! of this area.
//!
//! Pointers are `u32` addresses in the [`Mem`](crate::hooks::Mem) window; the callees are [`FldHooks`] methods named by address
//! (the ones shared with the scrcmd core, vm.c and the script_sys operand readers among them, come from [`CoreHooks`]).
//! Names of the main / overlay-21 callees (WORDSET word kinds, sound, field events, warps) are inferred from the spec and the
//! call sites (UNVERIFIED unless the doc says otherwise). In-area callees are called as Rust (the ROM side runs them as real
//! code).

use crate::hooks::MemEnv;
use crate::scrcmd_core::CoreHooks;
use crate::vm::VM_PC;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021A_E948,
    0x021A_E994,
    0x021A_E9CC,
    0x021A_E9F0,
    0x021A_EA3C,
    0x021A_EA8C,
    0x021A_EABC,
    0x021A_EB3C,
    0x021A_EBB0,
    0x021A_EC1C,
    0x021A_EC40,
    0x021A_EC84,
    0x021A_ECC0,
    0x021A_ED00,
    0x021A_ED24,
    0x021A_EDAC,
    0x021A_EDE0,
    0x021A_EE28,
    0x021A_EE78,
    0x021A_EEC0,
    0x021A_EEE0,
    0x021A_EF00,
    0x021A_EF10,
    0x021A_EF60,
    0x021A_EFA4,
    0x021A_EFD8,
    0x021A_F044,
    0x021A_F0BC,
    0x021A_F0F8,
    0x021A_F130,
    0x021A_F174,
    0x021A_F1B0,
    0x021A_F1D0,
    0x021A_F1E8,
    0x021A_F21C,
    0x021A_F23C,
    0x021A_F26C,
    0x021A_F294,
    0x021A_F2BC,
    0x021A_F2CC,
    0x021A_F2E4,
    0x021A_F2F4,
    0x021A_F324,
    0x021A_F370,
    0x021A_F380,
    0x021A_F3D8,
    0x021A_F430,
    0x021A_F448,
    0x021A_F458,
    0x021A_F484,
    0x021A_F4B0,
    0x021A_F4E8,
    0x021A_F544,
    0x021A_F594,
    0x021A_F5E4,
    0x021A_F634,
    0x021A_F684,
    0x021A_F6C4,
    0x021A_F6E8,
    0x021A_F71C,
    0x021A_F750,
    0x021A_F75C,
    0x021A_F794,
    0x021A_F7C4,
    0x021A_F7D4,
    0x021A_F7E4,
    0x021A_F844,
    0x021A_F878,
    0x021A_F8A8,
    0x021A_F8D8,
    0x021A_F908,
    0x021A_F918,
    0x021A_F928,
    0x021A_F958,
    0x021A_F9A4,
    0x021A_F9F0,
    0x021A_FA38,
    0x021A_FA80,
    0x021A_FAC8,
    0x021A_FAFC,
    0x021A_FB2C,
    0x021A_FB5C,
    0x021A_FB90,
    0x021A_FBC4,
    0x021A_FBEC,
    0x021A_FC18,
    0x021A_FC28,
    0x021A_FC38,
    0x021A_FC88,
    0x021A_FCD8,
    0x021A_FD44,
    0x021A_FDAC,
    0x021A_FE64,
    0x021A_FECC,
    0x021A_FEFC,
    0x021A_FF7C,
    0x021B_0000,
];

/// Command id -> handler address (Thumb entry, bit 0 clear) of the commands of this area (the words of the command table
/// 0x0217062C at index id; checked against the ROM by the tests).
pub const COMMANDS: &[(u16, u32)] = &[
    (76, 0x021A_F75C),
    (77, 0x021A_F7C4),
    (78, 0x021A_F7E4),
    (79, 0x021A_F7D4),
    (80, 0x021A_F844),
    (81, 0x021A_F878),
    (82, 0x021A_F928),
    (83, 0x021A_F958),
    (84, 0x021A_F9A4),
    (85, 0x021A_F9F0),
    (86, 0x021A_F8A8),
    (87, 0x021A_F908),
    (88, 0x021A_F918),
    (89, 0x021A_FAC8),
    (90, 0x021A_FAFC),
    (91, 0x021A_FA38),
    (92, 0x021A_FA80),
    (94, 0x021A_FB2C),
    (95, 0x021A_FB5C),
    (96, 0x021A_FBC4),
    (97, 0x021A_FC18),
    (98, 0x021A_FC28),
    (99, 0x021A_FB90),
    (127, 0x021A_E9F0),
    (128, 0x021A_E948),
    (129, 0x021A_E994),
    (130, 0x021A_EA3C),
    (131, 0x021A_E9CC),
    (132, 0x021A_EA8C),
    (133, 0x021A_EABC),
    (134, 0x021A_EB3C),
    (136, 0x021A_EBB0),
    (138, 0x021A_EC1C),
    (139, 0x021A_EC40),
    (142, 0x021A_EC84),
    (146, 0x021A_ECC0),
    (147, 0x021A_ED00),
    (148, 0x021A_ED24),
    (152, 0x021A_EDE0),
    (155, 0x021A_EEC0),
    (158, 0x021A_EFD8),
    (159, 0x021A_F0BC),
    (160, 0x021A_EF00),
    (161, 0x021A_F0F8),
    (162, 0x021A_F130),
    (163, 0x021A_F458),
    (164, 0x021A_F484),
    (165, 0x021A_F4B0),
    (166, 0x021A_F294),
    (167, 0x021A_F2BC),
    (168, 0x021A_F2E4),
    (169, 0x021A_F2F4),
    (170, 0x021A_F370),
    (171, 0x021A_F380),
    (172, 0x021A_F448),
    (181, 0x021A_F544),
    (182, 0x021A_F594),
    (183, 0x021A_F5E4),
    (184, 0x021A_F634),
    (185, 0x021A_F684),
    (186, 0x021A_F6C4),
    (187, 0x021A_F6E8),
    (189, 0x021A_F71C),
    (190, 0x021A_FD44),
    (191, 0x021A_FE64),
    (192, 0x021A_FF7C),
    (193, 0x021A_FDAC),
    (194, 0x021A_FCD8),
    (195, 0x021A_FECC),
    (196, 0x021A_FEFC),
    (538, 0x021A_FC38),
    (539, 0x021A_FC88),
    (584, 0x021A_EE28),
    (587, 0x021A_F044),
    (589, 0x021A_F3D8),
    (595, 0x021A_EF10),
    (596, 0x021A_EF60),
    (601, 0x021A_EE78),
    (602, 0x021B_0000),
    (607, 0x021A_F1B0),
];

/// SCRIPT_WORK field (`FUN_02159124`): the 0x1C-byte trainer-eye records start at SW + 0x44; record +0x14 is read by command
/// 131.
pub const TRAINER_REC_14: u32 = 0x14;
/// Trainer-eye record +0x10 (command 127) and +0x18 (the event of `FUN_021AE454`, commands 128 / 129).
pub const TRAINER_REC_10: u32 = 0x10;
pub const TRAINER_REC_EVENT: u32 = 0x18;

/// Wait callbacks of this area (Thumb bit set): command 160 ([`wait_160`]), 168 ([`wait_168`]), 170 ([`wait_170`]), 172
/// ([`wait_172`]).
pub const WAIT_160: u32 = 0x021A_EEE1;
pub const WAIT_168: u32 = 0x021A_F2CD;
pub const WAIT_170: u32 = 0x021A_F325;
pub const WAIT_172: u32 = 0x021A_F431;
/// The GMEVENT function of commands 171 / 589 ([`event_171`]).
pub const EVENT_171: u32 = 0x021A_F4E9;
/// The init procs of this area (table 0x021705F0 slots 10 / 12 / 13; Thumb bit set).
pub const INIT_PROC_10: u32 = 0x021A_F175;
pub const INIT_PROC_12: u32 = 0x021A_EFA5;
pub const INIT_PROC_13: u32 = 0x021A_F1D1;

/// The WORDSET routines the shared bodies [`wordset_body_77`] / [`wordset_body_87`] / [`wordset_body_97`] call through a
/// register (`blx r6` / `blx r5`), as the thunks load them: commands 77 / 79 / 87 / 88 / 97 / 98.
pub const WORDSET_FN_77: u32 = 0x0201_EF41;
pub const WORDSET_FN_79: u32 = 0x0201_EF4D;
pub const WORDSET_FN_87: u32 = 0x0201_EE61;
pub const WORDSET_FN_88: u32 = 0x0201_EEC5;
pub const WORDSET_FN_97: u32 = 0x0201_F0F9;
pub const WORDSET_FN_98: u32 = 0x0201_F125;

/// Callees of the scrcmd_fld functions that this area does not port, named by address (meaning per the spec; UNVERIFIED
/// unless said otherwise). The vm.c operand readers, the scrcmd ctx getters, `0215897C` (call a child event) and the
/// GMEVENT / GAMEDATA getters shared with the scrcmd core come from [`CoreHooks`].
pub trait FldHooks: CoreHooks {
    // ---- script_sys / scrcmd_work (overlay 10)
    /// C: `FUN_0215A5CC` (script_sys `ctx_heap`): the script's heap id.
    fn fun_0215a5cc(&mut self, ctx: u32) -> u32;
    /// C: `FUN_02158F54` (script_sys): the SCRIPT_WORK's GAMESYS.
    fn fun_02158f54(&mut self, sw: u32) -> u32;
    /// C: `FUN_02158F58` (script_sys): the SW field block (+4 = the field map).
    fn fun_02158f58(&mut self, sw: u32) -> u32;
    /// C: `FUN_02158F74` (script_sys): the script's WORDSET (SW + 0x2C).
    fn fun_02158f74(&mut self, sw: u32) -> u32;
    /// C: `FUN_02158F88` (script_sys): address SW + 0x40 (a byte of sound flags, bits 0..4).
    fn fun_02158f88(&mut self, sw: u32) -> u32;
    /// C: `FUN_02158F8C` (script_sys): the script id (u16 SW + 4).
    fn fun_02158f8c(&mut self, sw: u32) -> u32;
    /// C: `FUN_02159124` (script_sys `sw_record`): address of the `i`-th 0x1C-byte trainer-eye record at SW + 0x44.
    fn fun_02159124(&mut self, sw: u32, i: u32) -> u32;
    /// C: `FUN_02159144` (script_sys `proc_mask_set`): init-proc mask bit `bit` on.
    fn fun_02159144(&mut self, bit: u32);
    /// C: `FUN_02159168` (script_sys `proc_mask_clear`): init-proc mask bit `bit` off.
    fn fun_02159168(&mut self, bit: u32);
    /// C: `FUN_0215A460(id)` (overlay 10, trainer data of a script id).
    fn fun_0215a460(&mut self, id: u32) -> u32;
    /// C: `FUN_0215A4A0(id)` (overlay 10).
    fn fun_0215a4a0(&mut self, id: u32) -> u32;
    /// C: `FUN_0215A4B4(id)` (overlay 10; 1 = a double-battle trainer, commands 133 / 148).
    fn fun_0215a4b4(&mut self, id: u32) -> u32;
    /// C: `FUN_0215A4C8(id)` (overlay 10).
    fn fun_0215a4c8(&mut self, id: u32) -> u32;
    /// C: `FUN_0215A6AC` (scrcmd_work): the sound handle at SCRCMD WORK +0x108.
    fn fun_0215a6ac(&mut self, ctx: u32) -> u32;
    /// C: `FUN_0215A6B8` (scrcmd_work): SCRCMD WORK +0x108 = `v`.
    fn fun_0215a6b8(&mut self, ctx: u32, v: u32);

    // ---- overlay 10 (fldmmdl, event_battle, trainer eye)
    /// C: `FUN_0216D588(mmdl)` (fldmmdl): the OBJ id of a field object.
    fn fun_0216d588(&mut self, mmdl: u32) -> u32;
    /// C: `FUN_0216E818(gamesys, fieldmap, a, b, c, d, e)`: the trainer-battle event (commands 133 / 134; args 5..7 on the
    /// stack).
    #[allow(clippy::too_many_arguments)]
    fn fun_0216e818(
        &mut self,
        gamesys: u32,
        fm: u32,
        a: u32,
        b: u32,
        c: u32,
        d: u32,
        e: u32,
    ) -> u32;
    /// C: `FUN_0216EAB0(gamesys, fieldmap, a, b, c, d, e, f)`: the battle event of command 148.
    #[allow(clippy::too_many_arguments)]
    fn fun_0216eab0(
        &mut self,
        gamesys: u32,
        fm: u32,
        a: u32,
        b: u32,
        c: u32,
        d: u32,
        e: u32,
        f: u32,
    ) -> u32;

    // ---- overlay 21
    /// C: `FUN_021AE430(gamesys, ev0, ev1)` (event_trainer_eye.c): the event that runs the two trainer-eye events.
    fn fun_021ae430(&mut self, gamesys: u32, ev0: u32, ev1: u32) -> u32;
    /// C: `FUN_021AE454(heap, fieldmap, record, n)` (event_trainer_eye.c): the trainer-eye event of a record.
    fn fun_021ae454(&mut self, heap: u32, fm: u32, rec: u32, n: u32) -> u32;
    /// C: `FUN_021BE92C(gamesys, fieldmap, a, b, c, d, e)` (command 142's event).
    #[allow(clippy::too_many_arguments)]
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
    /// C: `FUN_02188C9C(fieldmap)`.
    fn fun_02188c9c(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188C5C(fieldmap)`.
    fn fun_02188c5c(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188CB4(fieldmap)`: the player object.
    fn fun_02188cb4(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188CD0(fieldmap)`.
    fn fun_02188cd0(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188D50(fieldmap)`: the daycare (sodateya) object.
    fn fun_02188d50(&mut self, fm: u32) -> u32;
    /// C: `FUN_021A3038(player, &pos)`: the player's position (12-byte record).
    fn fun_021a3038(&mut self, player: u32, pos: &mut [u8; 12]);
    /// C: `FUN_021A3074(player)`: the player's MMDL.
    fn fun_021a3074(&mut self, player: u32) -> u32;
    /// C: `FUN_021BB3E8(x, a, b, c, d, &loc, 0x15)` (fills the 12-byte location record of commands 192 / 602).
    #[allow(clippy::too_many_arguments)]
    fn fun_021bb3e8(&mut self, x: u32, a: u32, b: u32, c: u32, d: u32, loc: &mut [u8; 12], e: u32);
    /// C: `FUN_021BB578(x, d, 0, 1)`.
    fn fun_021bb578(&mut self, x: u32, d: u32, a: u32, b: u32);
    /// C: `FUN_021C34F8(daycare, i)` (sodateya.c): the daycare mon `i`.
    fn fun_021c34f8(&mut self, daycare: u32, i: u32) -> u32;

    // ---- overlay 20 (event_mapchange): warp events
    /// C: `FUN_02184D38(gamesys, fieldmap, zone, &pos, dir, 0)` (command 194).
    fn fun_02184d38(
        &mut self,
        gamesys: u32,
        fm: u32,
        zone: u32,
        pos: &[u8; 12],
        d: u32,
        e: u32,
    ) -> u32;
    /// C: `FUN_02184D90(gamesys, fieldmap, zone, &loc, e, 0)` (command 192).
    fn fun_02184d90(
        &mut self,
        gamesys: u32,
        fm: u32,
        zone: u32,
        loc: &[u8; 12],
        d: u32,
        e: u32,
    ) -> u32;
    /// C: `FUN_02184DE8(gamesys, fieldmap, zone, &loc, e, 0)` (command 602).
    fn fun_02184de8(
        &mut self,
        gamesys: u32,
        fm: u32,
        zone: u32,
        loc: &[u8; 12],
        d: u32,
        e: u32,
    ) -> u32;
    /// C: `FUN_02184E40(gamesys, fieldmap, &p2, zone, &pos)` (command 193).
    fn fun_02184e40(
        &mut self,
        gamesys: u32,
        fm: u32,
        p2: &[u8; 12],
        zone: u32,
        pos: &[u8; 12],
    ) -> u32;
    /// C: `FUN_02185050(gamesys, fieldmap, zone, &pos, dir)` (command 191).
    fn fun_02185050(&mut self, gamesys: u32, fm: u32, zone: u32, pos: &[u8; 12], d: u32) -> u32;
    /// C: `FUN_021850A8(gamesys, fieldmap)` (command 195).
    fn fun_021850a8(&mut self, gamesys: u32, fm: u32) -> u32;
    /// C: `FUN_021854B0(gamesys, fieldmap, zone, &pos, dir)` (command 196).
    fn fun_021854b0(&mut self, gamesys: u32, fm: u32, zone: u32, pos: &[u8; 12], d: u32) -> u32;
    /// C: `FUN_02185534(gamesys, fieldmap, zone, &pos, dir)` (command 190).
    fn fun_02185534(&mut self, gamesys: u32, fm: u32, zone: u32, pos: &[u8; 12], d: u32) -> u32;

    // ---- overlay 10 (zone / event data)
    /// C: `FUN_0216282C(x, ew, &pos, 9)` (command 193): the event-data entry at the position, or 0.
    fn fun_0216282c(&mut self, x: u32, ew: u32, pos: &[u8; 12], n: u32) -> u32;
    /// C: `FUN_02162A6C(entry, &p2)`: its 12-byte position record.
    fn fun_02162a6c(&mut self, entry: u32, p2: &mut [u8; 12]);

    // ---- main: game system / game data
    /// C: `FUN_02012104(gamesys)`: the GMEVENT the warp events are called from.
    fn fun_02012104(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_0201210C(gamesys)`: the field map.
    fn fun_0201210c(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_02012134(gamesys)`.
    fn fun_02012134(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_02012374(event, child)`: call the child event.
    fn fun_02012374(&mut self, event: u32, child: u32);
    /// C: `FUN_02012844(gamedata)`.
    fn fun_02012844(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_0201292C(gamedata)`.
    fn fun_0201292c(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012934(gamedata)`: the party.
    fn fun_02012934(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012944(gamedata)`.
    fn fun_02012944(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012984(gamedata)`.
    fn fun_02012984(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012EBC(gamedata)`.
    fn fun_02012ebc(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012F2C(gamedata)`.
    fn fun_02012f2c(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02008DF0(x, v)`.
    fn fun_02008df0(&mut self, x: u32, v: u32);
    /// C: `FUN_02014024(x)`.
    fn fun_02014024(&mut self, x: u32) -> u32;
    /// C: `FUN_02013E4C(v)`.
    fn fun_02013e4c(&mut self, v: u32) -> u32;
    /// C: `FUN_0201AB28(party, i)`: the party mon `i`.
    fn fun_0201ab28(&mut self, party: u32, i: u32) -> u32;
    /// C: `FUN_0200C0F0(x)`.
    fn fun_0200c0f0(&mut self, x: u32) -> u32;
    /// C: `FUN_0200C1C4(x)`.
    fn fun_0200c1c4(&mut self, x: u32) -> u32;
    /// C: `FUN_0200C1DC(x)`.
    fn fun_0200c1dc(&mut self, x: u32) -> u32;

    // ---- main: field events (02028xxx / 02029xxx event factories)
    /// C: `FUN_02028A58(x, a, b)`.
    fn fun_02028a58(&mut self, x: u32, a: u32, b: u32);
    /// C: `FUN_020289B8(x)`.
    fn fun_020289b8(&mut self, x: u32) -> u32;
    /// C: `FUN_02028BA0(x, v)`.
    fn fun_02028ba0(&mut self, x: u32, v: u32);
    /// C: `FUN_02028BDC(x, v)`.
    fn fun_02028bdc(&mut self, x: u32, v: u32);
    /// C: `FUN_020295A4(gamesys, v)`.
    fn fun_020295a4(&mut self, gamesys: u32, v: u32) -> u32;
    /// C: `FUN_02029620(gamesys, a, b)`.
    fn fun_02029620(&mut self, gamesys: u32, a: u32, b: u32) -> u32;
    /// C: `FUN_02029830(gamesys, v)`.
    fn fun_02029830(&mut self, gamesys: u32, v: u32) -> u32;
    /// C: `FUN_020298A0(gamesys, x, a, b)`.
    fn fun_020298a0(&mut self, gamesys: u32, x: u32, a: u32, b: u32) -> u32;
    /// C: `FUN_02029984(gamesys, v)`.
    fn fun_02029984(&mut self, gamesys: u32, v: u32) -> u32;
    /// C: `FUN_02029A04(gamesys, a, b)`.
    fn fun_02029a04(&mut self, gamesys: u32, a: u32, b: u32) -> u32;
    /// C: `FUN_02029A74(gamesys, v)`.
    fn fun_02029a74(&mut self, gamesys: u32, v: u32) -> u32;
    /// C: `FUN_02029AF4(gamesys, v)`.
    fn fun_02029af4(&mut self, gamesys: u32, v: u32) -> u32;
    /// C: `FUN_02029DF4(gamedata, a, b)`.
    fn fun_02029df4(&mut self, gamedata: u32, a: u32, b: u32) -> u32;
    /// C: `FUN_02029E20(v)`.
    fn fun_02029e20(&mut self, v: u32) -> u32;
    /// C: `FUN_0202A230(v, k)`: a property test of v.
    fn fun_0202a230(&mut self, v: u32, k: u32) -> u32;
    /// C: `FUN_0202B578(x)`.
    fn fun_0202b578(&mut self, x: u32) -> u32;
    /// C: `FUN_0202C0D0(x, v)`.
    fn fun_0202c0d0(&mut self, x: u32, v: u32);
    /// C: `FUN_0202C0E4(x, v)`.
    fn fun_0202c0e4(&mut self, x: u32, v: u32);
    /// C: `FUN_0202C0F8(x, v)`.
    fn fun_0202c0f8(&mut self, x: u32, v: u32) -> u32;

    // ---- main: sound
    /// C: `FUN_02005C64()`: nonzero while busy (command 171's event waits).
    fn fun_02005c64(&mut self) -> u32;
    /// C: `FUN_02005F38()`: nonzero while something (a sound?) is busy.
    fn fun_02005f38(&mut self) -> u32;
    /// C: `FUN_020061E4(se)`: play sound effect `se` (`ScriptHost::play_se`).
    fn fun_020061e4(&mut self, se: u32);
    /// C: `FUN_020061F8(i)`: stop sound slot `i`.
    fn fun_020061f8(&mut self, i: u32);
    /// C: `FUN_02006148(x)`: the slot / player of sound x.
    fn fun_02006148(&mut self, x: u32) -> u32;
    /// C: `FUN_02006224(i)`: 0 once sound slot i finished.
    fn fun_02006224(&mut self, i: u32) -> u32;
    /// C: `FUN_02006984(a, b, 0x40, 0, 0, 0, 0, tmp)`: start a sound (8 args, 5..8 on the stack; `tmp` = the 4-byte record of
    /// `FUN_02006CAC` or null).
    #[allow(clippy::too_many_arguments)]
    fn fun_02006984(
        &mut self,
        a: u32,
        b: u32,
        c: u32,
        d: u32,
        e: u32,
        f: u32,
        g: u32,
        tmp: Option<&[u8; 4]>,
    ) -> u32;
    /// C: `FUN_02006BD0(handle)`: nonzero while the sound plays.
    fn fun_02006bd0(&mut self, handle: u32) -> u32;
    /// C: `FUN_02006CAC(&tmp)`: fill a 4-byte record.
    fn fun_02006cac(&mut self, tmp: &mut [u8; 4]);

    // ---- main: names / items
    /// C: `FUN_02007E3C(x, a, b, heap)`.
    fn fun_02007e3c(&mut self, x: u32, a: u32, b: u32, heap: u32) -> u32;
    /// C: `FUN_02007E50(x, a, b, heap)`.
    fn fun_02007e50(&mut self, x: u32, a: u32, b: u32, heap: u32) -> u32;
    /// C: `FUN_02007F1C(x, a, b, heap)`.
    fn fun_02007f1c(&mut self, x: u32, a: u32, b: u32, heap: u32) -> u32;
    /// C: `FUN_02007F68(x, a, b, heap)`.
    fn fun_02007f68(&mut self, x: u32, a: u32, b: u32, heap: u32) -> u32;
    /// C: `FUN_02007FB8(x, a, heap)`.
    fn fun_02007fb8(&mut self, x: u32, a: u32, heap: u32) -> u32;
    /// C: `FUN_02020FE4(item, field, heap)`: an item data field (`scrcmd_work` uses the same callee).
    fn fun_02020fe4(&mut self, item: u32, field: u32, heap: u32) -> u32;
    /// C: `FUN_02021288(x)`.
    fn fun_02021288(&mut self, x: u32) -> u32;
    /// C: `FUN_020212B0(x)`.
    fn fun_020212b0(&mut self, x: u32) -> u32;

    // ---- main: WORDSET (black_text::wordset; word / number insertion)
    /// C: `FUN_0201EE84(wordset, idx, mon)`: a mon's nickname word.
    fn fun_0201ee84(&mut self, ws: u32, idx: u32, mon: u32);
    /// C: `FUN_0201EEF4(wordset, idx, mon)`: another word of a mon.
    fn fun_0201eef4(&mut self, ws: u32, idx: u32, mon: u32);
    /// C: `FUN_0201EF34(wordset, idx, v)`.
    fn fun_0201ef34(&mut self, ws: u32, idx: u32, v: u32);
    /// C: `FUN_0201EF40(wordset, idx, v)` (returns its r0: command 78 returns it).
    fn fun_0201ef40(&mut self, ws: u32, idx: u32, v: u32) -> u32;
    /// C: `FUN_0201EF4C(wordset, idx, v)` (returns its r0).
    fn fun_0201ef4c(&mut self, ws: u32, idx: u32, v: u32) -> u32;
    /// C: `FUN_0201EF5C(wordset, idx, v)` (returns its r0).
    fn fun_0201ef5c(&mut self, ws: u32, idx: u32, v: u32) -> u32;
    /// C: `FUN_0201EFA4(wordset, idx, v, digits, a, b)`: a number word (args 5 / 6 on the stack).
    fn fun_0201efa4(&mut self, ws: u32, idx: u32, v: u32, n: u32, a: u32, b: u32);
    /// C: `FUN_0201EFC8(wordset, idx, v)`.
    fn fun_0201efc8(&mut self, ws: u32, idx: u32, v: u32);
    /// C: `FUN_0201F020(wordset, idx, v)`.
    fn fun_0201f020(&mut self, ws: u32, idx: u32, v: u32);
    /// C: `FUN_0201F02C(wordset, idx, v)`.
    fn fun_0201f02c(&mut self, ws: u32, idx: u32, v: u32);
    /// C: `FUN_0201F048(wordset, idx, v)`.
    fn fun_0201f048(&mut self, ws: u32, idx: u32, v: u32);
    /// C: `FUN_0201F14C(wordset, idx, v)`.
    fn fun_0201f14c(&mut self, ws: u32, idx: u32, v: u32);
    /// C: `FUN_0201F158(wordset, idx, v)`.
    fn fun_0201f158(&mut self, ws: u32, idx: u32, v: u32);
    /// C: `FUN_0201F184(wordset, idx, v)`.
    fn fun_0201f184(&mut self, ws: u32, idx: u32, v: u32);
    /// C: `FUN_0201F198(wordset, idx, v)`.
    fn fun_0201f198(&mut self, ws: u32, idx: u32, v: u32);
    /// C: `FUN_0201F1B0(wordset, idx, v)`.
    fn fun_0201f1b0(&mut self, ws: u32, idx: u32, v: u32);
    /// C: `blx r6` / `blx r5` in the shared bodies 0x021AF794 / 0x021AF8D8 / 0x021AFBEC: the WORDSET routine `f` (Thumb
    /// address; the commands pass [`WORDSET_FN_77`] = `FUN_0201EF40`, [`WORDSET_FN_79`] = `FUN_0201EF4C`, [`WORDSET_FN_87`] =
    /// `FUN_0201EE60`, [`WORDSET_FN_88`] = `FUN_0201EEC4`, [`WORDSET_FN_97`] = `FUN_0201F0F8`, [`WORDSET_FN_98`] =
    /// `FUN_0201F124`) as `f(wordset, idx, v)`; its result is unused.
    fn call_wordset_fn(&mut self, f: u32, ws: u32, idx: u32, v: u32);
}

/// The next script byte (`ldrb [PC]; PC += 1`, inlined by the WORDSET commands).
fn read_byte<H: MemEnv>(h: &mut H, vm: u32) -> u32 {
    let pc = h.r32(vm + VM_PC);
    let b = h.r8(pc) as u32;
    h.w32(vm + VM_PC, pc.wrapping_add(1));
    b
}

/// A 12-byte VecFx32 `{x, y, z}` as the stack record the original passes.
fn vec3(x: u32, y: u32, z: u32) -> [u8; 12] {
    let mut v = [0u8; 12];
    v[0..4].copy_from_slice(&x.to_le_bytes());
    v[4..8].copy_from_slice(&y.to_le_bytes());
    v[8..12].copy_from_slice(&z.to_le_bytes());
    v
}

// ---- trainer eye / trainer battles (127..148)

/// C: `FUN_overlay_d_21__021ae948`: command 128: n = value; the trainer-eye record r = `FUN_02159124(SW, n)` for n 0 / 1;
/// r +0x18 = `FUN_021AE454(heap, field map, r, n)`; 0. An original bug, kept: for any other n, r is the caller's r5, which the
/// function never sets (`stale_r5`; UNVERIFIED: the spec bounds n to 0 / 1, and [`dispatch`] passes 0, the Z19 entry value).
pub fn cmd_128<H: FldHooks>(h: &mut H, vm: u32, ctx: u32, stale_r5: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let fb = h.fun_02158f58(sw);
    let n = h.fun_02159b10(vm, ctx);
    let rec = match n {
        0 => h.fun_02159124(sw, 0),
        1 => h.fun_02159124(sw, 1),
        _ => stale_r5,
    };
    let heap = h.fun_0215a5cc(ctx);
    let fm = h.r32(fb + 4);
    let ev = h.fun_021ae454(heap, fm, rec, n);
    h.w32(rec + TRAINER_REC_EVENT, ev);
    0
}

/// C: `FUN_overlay_d_21__021ae994`: command 129: child event `FUN_021AE430(GAMESYS, record 0 +0x18, record 1 +0x18)` (the two
/// trainer-eye events); yield.
pub fn cmd_129<H: FldHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let r0 = h.fun_02159124(sw, 0);
    let r1 = h.fun_02159124(sw, 1);
    let gs = h.fun_0215a5dc(ctx);
    let e0 = h.r32(r0 + TRAINER_REC_EVENT);
    let e1 = h.r32(r1 + TRAINER_REC_EVENT);
    let ev = h.fun_021ae430(gs, e0, e1);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_021ae9cc`: command 131: var = the u16 at trainer-eye record 0 +0x14 (`FUN_02159124(SW, 0)`).
pub fn cmd_trainer_rec_14<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let rec = h.fun_02159124(sw, 0);
    let var = h.fun_02159ae8(vm, ctx);
    let v = h.r32(rec + TRAINER_REC_14);
    h.w16(var, v as u16);
    0
}

/// The trainer-eye record of `n` for commands 127 / 130: `FUN_02159124(SW, n)` for n 0 / 1, else none (0).
fn record_0_1<H: FldHooks>(h: &mut H, sw: u32, n: u32) -> u32 {
    match n {
        0 => h.fun_02159124(sw, 0),
        1 => h.fun_02159124(sw, 1),
        _ => 0,
    }
}

/// C: `sub_021ae9f0`: command 127: n = value; var = record n +0x10 (u16) when n < 2 and the record exists, else 0.
pub fn cmd_127<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let n = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let rec = record_0_1(h, sw, n);
    let v = if rec != 0 {
        h.r32(rec + TRAINER_REC_10)
    } else {
        0
    };
    h.w16(var, v as u16);
    0
}

/// C: `sub_021aea3c`: command 130: n = value; var = `FUN_0216D588(*record n)` (the OBJ id of the record's MMDL) when n < 2
/// and the record exists, else 0.
pub fn cmd_130<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let n = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let rec = record_0_1(h, sw, n);
    let v = if rec != 0 {
        let mmdl = h.r32(rec);
        h.fun_0216d588(mmdl)
    } else {
        0
    };
    h.w16(var, v as u16);
    0
}

/// C: `sub_021aea8c`: command 132: var = `FUN_0215A460(script id)` (the `FUN_02158F58` result is unused).
pub fn cmd_132<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    h.fun_02158f58(sw);
    let id = h.fun_02158f8c(sw);
    let var = h.fun_02159ae8(vm, ctx);
    let v = h.fun_0215a460(id);
    h.w16(var, v as u16);
    0
}

/// C: `sub_021aeabc`: command 133: a, b, c = values; d = `FUN_0215A4C8(a)`; b == 0 and `FUN_0215A4B4(a)` == 1 -> b = a; child
/// event `FUN_0216E818(GAMESYS, field map, d, 0, a, b, c)` (a trainer battle); [`after_battle_start`]; yield.
pub fn cmd_133<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    h.fun_02158f8c(sw);
    let a = h.fun_02159b10(vm, ctx);
    let mut b = h.fun_02159b10(vm, ctx);
    let c = h.fun_02159b10(vm, ctx);
    let d = h.fun_0215a4c8(a);
    if b == 0 && h.fun_0215a4b4(a) == 1 {
        b = a;
    }
    let gs = h.fun_02158f54(sw);
    let fb = h.fun_02158f58(sw);
    let fm = h.r32(fb + 4);
    let ev = h.fun_0216e818(gs, fm, d, 0, a, b, c);
    h.fun_0215897c(sw, ev);
    after_battle_start(h, ctx);
    1
}

/// C: `sub_021aeb3c`: command 134: a, b, c, d = values; child event `FUN_0216E818(GAMESYS, field map, 1, a, b, c, d)`;
/// [`after_battle_start`]; yield.
pub fn cmd_134<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    h.fun_02158f8c(sw);
    let a = h.fun_02159b10(vm, ctx);
    let b = h.fun_02159b10(vm, ctx);
    let c = h.fun_02159b10(vm, ctx);
    let d = h.fun_02159b10(vm, ctx);
    let gs = h.fun_02158f54(sw);
    let fb = h.fun_02158f58(sw);
    let fm = h.r32(fb + 4);
    let ev = h.fun_0216e818(gs, fm, 1, a, b, c, d);
    h.fun_0215897c(sw, ev);
    after_battle_start(h, ctx);
    1
}

/// C: `sub_021aebb0`: command 136: three vars = (0, 2, 0x18) when `FUN_0215A4B4(FUN_0215A460(script id))` is 0 (u16), else
/// (3, 5, 6) when `FUN_0215A4A0(script id)` is 0 (u16), else (7, 9, 0xA).
pub fn cmd_three_vars<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let id = h.fun_02158f8c(sw);
    let a = h.fun_02159ae8(vm, ctx);
    let b = h.fun_02159ae8(vm, ctx);
    let c = h.fun_02159ae8(vm, ctx);
    let x = h.fun_0215a460(id);
    let (va, vb, vc) = if h.fun_0215a4b4(x) & 0xFFFF == 0 {
        (0, 2, 0x18)
    } else if h.fun_0215a4a0(id) & 0xFFFF == 0 {
        (3, 5, 6)
    } else {
        (7, 9, 0xA)
    };
    h.w16(a, va);
    h.w16(b, vb);
    h.w16(c, vc);
    0
}

/// C: `sub_021aec1c`: command 138: var = `FUN_0215A4C8(value)`.
pub fn cmd_138<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let v = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let r = h.fun_0215a4c8(v);
    h.w16(var, r as u16);
    0
}

/// C: `sub_021aec40`: command 139: child event `FUN_02029AF4(GAMESYS, FUN_02029E20(FUN_0202A230(value, 1)))`; yield (the
/// GAMEDATA getter's result is unused).
pub fn cmd_139<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    h.fun_0215a5e4(ctx);
    let sw = h.fun_0215a5f4(ctx);
    let v = h.fun_02159b10(vm, ctx);
    let x = h.fun_0202a230(v, 1);
    let y = h.fun_02029e20(x);
    let ev = h.fun_02029af4(gs, y);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_021aec84`: command 142: child event `FUN_021BE92C(GAMESYS, field map, 0, 0, 1, 0, 0)`; yield.
pub fn cmd_142<H: FldHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    let ev = h.fun_021be92c(gs, fm, 0, 0, 1, 0, 0);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_021aecc0`: command 146: v = value; var = `FUN_0202A230(v, 9)` ? 1 : `FUN_0202A230(v, 0xB)` ? 2 : 0.
pub fn cmd_classify_146<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let v = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let r = if h.fun_0202a230(v, 9) != 0 {
        1
    } else if h.fun_0202a230(v, 0xB) != 0 {
        2
    } else {
        0
    };
    h.w16(var, r);
    0
}

/// C: `sub_021aed00`: command 147: var = `FUN_0202A230(value, 0xB)`.
pub fn cmd_147<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let v = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let r = h.fun_0202a230(v, 0xB);
    h.w16(var, r as u16);
    0
}

/// C: `sub_021aed24`: command 148: a, b, c, d = values; e = `FUN_0215A4C8(a)`; b == 0 and `FUN_0215A4B4(a)` == 1 -> b = a;
/// child event `FUN_0216EAB0(GAMESYS, field map, e, 0, a, b, c, d)`; yield.
pub fn cmd_148<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    h.fun_02158f8c(sw);
    let a = h.fun_02159b10(vm, ctx);
    let mut b = h.fun_02159b10(vm, ctx);
    let c = h.fun_02159b10(vm, ctx);
    let d = h.fun_02159b10(vm, ctx);
    let e = h.fun_0215a4c8(a);
    if b == 0 && h.fun_0215a4b4(a) == 1 {
        b = a;
    }
    let gs = h.fun_02158f54(sw);
    let fb = h.fun_02158f58(sw);
    let fm = h.r32(fb + 4);
    let ev = h.fun_0216eab0(gs, fm, e, 0, a, b, c, d);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `FUN_overlay_d_21__021aedac`: after a trainer battle starts (commands 133 / 134): `FUN_02014024(FUN_02188C9C(field
/// map))` != 0 -> `FUN_02008DF0(FUN_02012F2C(GAMEDATA), 0x20)`.
pub fn after_battle_start<H: FldHooks>(h: &mut H, ctx: u32) {
    let sw = h.fun_0215a5f4(ctx);
    let fb = h.fun_02158f58(sw);
    let gd = h.fun_0215a5e4(ctx);
    let x = h.fun_02012f2c(gd);
    let fm = h.r32(fb + 4);
    let y = h.fun_02188c9c(fm);
    if h.fun_02014024(y) != 0 {
        h.fun_02008df0(x, 0x20);
    }
}

// ---- field events (152..172, 584..607)

/// `FUN_02028BA0(FUN_02012A30(GAMEDATA of gs), 2)` + init-proc bit 13 on: the tail of commands 152 / 584 / 601.
fn event_bit13_on<H: FldHooks>(h: &mut H, gs: u32) {
    let gd = h.fun_020120f4(gs);
    let x = h.fun_02012a30(gd);
    h.fun_02028ba0(x, 2);
    h.fun_02159144(0xD);
}

/// C: `sub_021aede0`: command 152: child event `FUN_02029984(GAMESYS, u16 operand)`; `FUN_02028BA0(FUN_02012A30(GAMEDATA),
/// 2)`; init-proc bit 13 on; yield.
pub fn cmd_152<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let sw = h.fun_0215a5f4(ctx);
    let v = h.fun_02011330(vm);
    let ev = h.fun_02029984(gs, v);
    h.fun_0215897c(sw, ev);
    event_bit13_on(h, gs);
    1
}

/// C: `sub_021aee28`: command 584: child event `FUN_02029A04(GAMESYS, u16 a, u16 b)`; as command 152 after it.
pub fn cmd_584<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let sw = h.fun_0215a5f4(ctx);
    let a = h.fun_02011330(vm);
    let b = h.fun_02011330(vm);
    let ev = h.fun_02029a04(gs, a, b);
    h.fun_0215897c(sw, ev);
    event_bit13_on(h, gs);
    1
}

/// C: `sub_021aee78`: command 601: child event `FUN_02029A74(GAMESYS, u16 operand)`; as command 152 after it.
pub fn cmd_601<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let sw = h.fun_0215a5f4(ctx);
    let v = h.fun_02011330(vm);
    let ev = h.fun_02029a74(gs, v);
    h.fun_0215897c(sw, ev);
    event_bit13_on(h, gs);
    1
}

/// C: `sub_021aeec0`: command 155: skips a u16 operand; var = `FUN_02005F38()`.
pub fn cmd_155<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_02011330(vm);
    let var = h.fun_02159ae8(vm, ctx);
    let v = h.fun_02005f38();
    h.w16(var, v as u16);
    0
}

/// C: `sub_021aeee0`: wait callback of command 160: done (1) once `FUN_02005F38()` is 0. (The GAMESYS / SW getter results are
/// unused, as in the original.)
pub fn wait_160<H: FldHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5dc(ctx);
    h.fun_0215a5f4(ctx);
    (h.fun_02005f38() == 0) as u32
}

/// C: `sub_021aef00`: command 160: wait on [`WAIT_160`]; yield.
pub fn cmd_160<H: FldHooks>(h: &mut H, vm: u32, _ctx: u32) -> u32 {
    h.fun_020113d0(vm, WAIT_160);
    1
}

/// C: `sub_021aef10`: command 595: a, b = values; `FUN_02028A58(FUN_02012A30(GAMEDATA), a & 0xFF, b & 0xFF)`; init-proc bit
/// 12 on; 0 (the field-map getter's result is unused).
pub fn cmd_595<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    h.fun_0201210c(gs);
    let x = h.fun_02012a30(gd);
    let a = h.fun_02159b10(vm, ctx);
    let b = h.fun_02159b10(vm, ctx);
    h.fun_02028a58(x, a & 0xFF, b & 0xFF);
    h.fun_02159144(0xC);
    0
}

/// C: `sub_021aef60`: command 596: `FUN_02028A58(FUN_02012A30(GAMEDATA), 0x7F, value & 0xFF)`; init-proc bit 12 off; 0.
pub fn cmd_596<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    h.fun_0201210c(gs);
    let x = h.fun_02012a30(gd);
    let v = h.fun_02159b10(vm, ctx);
    h.fun_02028a58(x, 0x7F, v & 0xFF);
    h.fun_02159168(0xC);
    0
}

/// C: `sub_021aefa4`: init proc of mask bit 12 (`(task, seq)`, task +0 = GAMESYS): seq 0 ->
/// `FUN_02028A58(FUN_02012A30(GAMEDATA), 0x7F, 0)`, seq = 1, 0; seq 1 -> 1 (done); other -> 0.
pub fn init_proc_12<H: FldHooks>(h: &mut H, task: u32, seq: u32) -> u32 {
    let gs = h.r32(task);
    let gd = h.fun_020120f4(gs);
    let x = h.fun_02012a30(gd);
    match h.r32(seq) {
        0 => {
            h.fun_02028a58(x, 0x7F, 0);
            let s = h.r32(seq);
            h.w32(seq, s.wrapping_add(1));
            0
        }
        1 => 1,
        _ => 0,
    }
}

/// C: `sub_021aefd8`: command 158: child event `FUN_020298A0(GAMESYS, FUN_02029DF4(GAMEDATA, FUN_02188C9C(field map),
/// FUN_02012984(GAMEDATA)), 0x5A, 0x3C)`; `FUN_02028BDC(FUN_02012A30(GAMEDATA), 2)`; init-proc bit 13 off; yield.
pub fn cmd_158<H: FldHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    let fm = h.fun_0201210c(gs);
    let x = h.fun_02012a30(gd);
    let sw = h.fun_0215a5f4(ctx);
    let y = h.fun_02012984(gd);
    let z = h.fun_02188c9c(fm);
    let w = h.fun_02029df4(gd, z, y);
    let ev = h.fun_020298a0(gs, w, 0x5A, 0x3C);
    h.fun_0215897c(sw, ev);
    h.fun_02028bdc(x, 2);
    h.fun_02159168(0xD);
    1
}

/// C: `sub_021af044`: command 587: as command 158 with a u16 operand instead of 0x5A.
pub fn cmd_587<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    let fm = h.fun_0201210c(gs);
    let x = h.fun_02012a30(gd);
    let sw = h.fun_0215a5f4(ctx);
    let v = h.fun_02011330(vm);
    let y = h.fun_02012984(gd);
    let z = h.fun_02188c9c(fm);
    let w = h.fun_02029df4(gd, z, y);
    let ev = h.fun_020298a0(gs, w, v, 0x3C);
    h.fun_0215897c(sw, ev);
    h.fun_02028bdc(x, 2);
    h.fun_02159168(0xD);
    1
}

/// C: `sub_021af0bc`: command 159: child event `FUN_02029AF4(GAMESYS, value)`; yield (the GAMEDATA / field-map getter
/// results are unused).
pub fn cmd_159<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    h.fun_020120f4(gs);
    h.fun_0201210c(gs);
    let sw = h.fun_0215a5f4(ctx);
    let v = h.fun_02159b10(vm, ctx);
    let ev = h.fun_02029af4(gs, v);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_021af0f8`: command 161: child event `FUN_020295A4(GAMESYS, value)`; init-proc bit 10 on; yield.
pub fn cmd_161<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let sw = h.fun_0215a5f4(ctx);
    let v = h.fun_02159b10(vm, ctx);
    let ev = h.fun_020295a4(gs, v);
    h.fun_0215897c(sw, ev);
    h.fun_02159144(0xA);
    1
}

/// C: `sub_021af130`: command 162: child event `FUN_02029620(GAMESYS, a, b)` (values); init-proc bit 10 off; yield.
pub fn cmd_162<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let sw = h.fun_0215a5f4(ctx);
    let a = h.fun_02159b10(vm, ctx);
    let b = h.fun_02159b10(vm, ctx);
    let ev = h.fun_02029620(gs, a, b);
    h.fun_0215897c(sw, ev);
    h.fun_02159168(0xA);
    1
}

/// C: `sub_021af174`: init proc of mask bit 10 (`(task, seq)`, task +0 = GAMESYS, +8 = SW): seq 0 -> child event
/// `FUN_02029620(GAMESYS, 0, 0)` of the SW, seq = 1, 0; seq 1 -> 1; other -> 0 (the field-block getter's result is unused).
pub fn init_proc_10<H: FldHooks>(h: &mut H, task: u32, seq: u32) -> u32 {
    let sw = h.r32(task + 8);
    h.fun_02158f58(sw);
    match h.r32(seq) {
        0 => {
            let gs = h.r32(task);
            let ev = h.fun_02029620(gs, 0, 0);
            h.fun_0215897c(sw, ev);
            let s = h.r32(seq);
            h.w32(seq, s.wrapping_add(1));
            0
        }
        1 => 1,
        _ => 0,
    }
}

/// C: `sub_021af1b0`: command 607: `FUN_02028BDC(FUN_02012A30(GAMEDATA), 2)`; init-proc bit 13 off; 0.
pub fn cmd_607<H: FldHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    let x = h.fun_02012a30(gd);
    h.fun_02028bdc(x, 2);
    h.fun_02159168(0xD);
    0
}

/// C: `sub_021af1d0`: init proc of mask bit 13 (`(task, seq)`): `FUN_02028BDC(FUN_02012A30(GAMEDATA of *task), 2)`; 1.
pub fn init_proc_13<H: FldHooks>(h: &mut H, task: u32, _seq: u32) -> u32 {
    let gs = h.r32(task);
    let gd = h.fun_020120f4(gs);
    let x = h.fun_02012a30(gd);
    h.fun_02028bdc(x, 2);
    1
}

/// C: `FUN_overlay_d_21__021af1e8`: for every bit i < 5 set in the sound-flag byte at SW +0x40: `FUN_02006224(i) == 0` (that
/// sound finished) clears it.
pub fn sound_flags_update<H: FldHooks>(h: &mut H, sw: u32) {
    let p = h.fun_02158f88(sw);
    for i in 0..5u32 {
        let bit = 1u32 << i;
        if h.r8(p) as u32 & bit != 0 && h.fun_02006224(i) == 0 {
            let v = h.r8(p) & !(bit as u8);
            h.w8(p, v);
        }
    }
}

/// C: `FUN_overlay_d_21__021af21c`: set bit `FUN_02006148(x)` of the sound-flag byte at SW +0x40 (a shift of 32 or more sets
/// nothing, as Thumb `lsls` by register).
pub fn sound_flags_add<H: FldHooks>(h: &mut H, sw: u32, x: u32) {
    let p = h.fun_02158f88(sw);
    let n = h.fun_02006148(x) & 0xFF;
    let bit = if n < 32 { 1u32 << n } else { 0 };
    let v = h.r8(p) | bit as u8;
    h.w8(p, v);
}

/// C: `FUN_overlay_d_21__021af23c`: 1 if some bit i < 5 of the sound-flag byte has `FUN_02006224(i)` == 1 (still playing);
/// else the byte = 0 and 0.
pub fn sound_flags_busy<H: FldHooks>(h: &mut H, sw: u32) -> u32 {
    let p = h.fun_02158f88(sw);
    for i in 0..5u32 {
        if (1u32 << i) & h.r8(p) as u32 != 0 && h.fun_02006224(i) == 1 {
            return 1;
        }
    }
    h.w8(p, 0);
    0
}

/// C: `FUN_overlay_d_21__021af26c`: `FUN_020061F8(i)` (stop) for every bit i < 5 set in the sound-flag byte; the byte = 0.
pub fn sound_flags_stop<H: FldHooks>(h: &mut H, sw: u32) {
    let p = h.fun_02158f88(sw);
    for i in 0..5u32 {
        if (1u32 << i) & h.r8(p) as u32 != 0 {
            h.fun_020061f8(i);
        }
    }
    h.w8(p, 0);
}

/// C: `sub_021af294`: command 166: v = value; [`sound_flags_update`]; `FUN_020061E4(v)` (play); [`sound_flags_add`]`(SW, v)`;
/// 0.
pub fn cmd_166<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let v = h.fun_02159b10(vm, ctx);
    let sw = h.fun_0215a5f4(ctx);
    sound_flags_update(h, sw);
    h.fun_020061e4(v);
    sound_flags_add(h, sw, v);
    0
}

/// C: `sub_021af2bc`: command 167: [`sound_flags_stop`]; 0.
pub fn cmd_167<H: FldHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    sound_flags_stop(h, sw);
    0
}

/// C: `sub_021af2cc`: wait callback of command 168: done once [`sound_flags_busy`] is 0.
pub fn wait_168<H: FldHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    (sound_flags_busy(h, sw) == 0) as u32
}

/// C: `sub_021af2e4`: command 168: wait on [`WAIT_168`]; yield.
pub fn cmd_168<H: FldHooks>(h: &mut H, vm: u32, _ctx: u32) -> u32 {
    h.fun_020113d0(vm, WAIT_168);
    1
}

/// C: `sub_021af2f4`: command 169: child event `FUN_02029830(GAMESYS, u16 operand)`; yield.
pub fn cmd_169<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let sw = h.fun_0215a5f4(ctx);
    let v = h.fun_02011330(vm);
    let ev = h.fun_02029830(gs, v);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_021af324`: wait callback of command 170: `FUN_02005F38()` == 0 and `FUN_020289B8(FUN_02012A30(GAMEDATA))` == 0 ->
/// child event `FUN_02029620(GAMESYS, 0, 6)`, done (1); else 0.
pub fn wait_170<H: FldHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let sw = h.fun_0215a5f4(ctx);
    let gd = h.fun_020120f4(gs);
    let x = h.fun_02012a30(gd);
    if h.fun_02005f38() == 0 && h.fun_020289b8(x) == 0 {
        let ev = h.fun_02029620(gs, 0, 6);
        h.fun_0215897c(sw, ev);
        return 1;
    }
    0
}

/// C: `sub_021af370`: command 170: wait on [`WAIT_170`]; yield.
pub fn cmd_170<H: FldHooks>(h: &mut H, vm: u32, _ctx: u32) -> u32 {
    h.fun_020113d0(vm, WAIT_170);
    1
}

/// Commands 171 / 589: a, b = values; child event E = `FUN_020122C0(GAMESYS, 0, `[`EVENT_171`]`, 0xC)` with the work
/// `{u16 a, u16 b, u32 mode, ctx}`; yield.
fn start_event_171<H: FldHooks>(h: &mut H, vm: u32, ctx: u32, mode: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let a = h.fun_02159b10(vm, ctx);
    let b = h.fun_02159b10(vm, ctx);
    let ev = h.fun_020122c0(gs, 0, EVENT_171, 0xC);
    let w = h.fun_020124e8(ev);
    h.w16(w, a as u16);
    h.w16(w + 2, b as u16);
    h.w32(w + 4, mode);
    h.w32(w + 8, ctx);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_021af380`: command 171: the sound event [`event_171`] with mode 0; yield.
pub fn cmd_171<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    start_event_171(h, vm, ctx, 0)
}

/// C: `sub_021af3d8`: command 589: as command 171 with mode 1.
pub fn cmd_589<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    start_event_171(h, vm, ctx, 1)
}

/// C: `sub_021af430`: wait callback of command 172: done once `FUN_02006BD0(SCRCMD WORK +0x108)` == 0.
pub fn wait_172<H: FldHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let x = h.fun_0215a6ac(ctx);
    (h.fun_02006bd0(x) == 0) as u32
}

/// C: `sub_021af448`: command 172: wait on [`WAIT_172`]; yield.
pub fn cmd_172<H: FldHooks>(h: &mut H, vm: u32, _ctx: u32) -> u32 {
    h.fun_020113d0(vm, WAIT_172);
    1
}

/// C: `sub_021af458`: command 163: `FUN_0202C0D0(FUN_0202B578(FUN_02012134(GAMESYS)), value)`; 0.
pub fn cmd_163<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let y = h.fun_02012134(gs);
    let z = h.fun_0202b578(y);
    let v = h.fun_02159b10(vm, ctx);
    h.fun_0202c0d0(z, v);
    0
}

/// C: `sub_021af484`: command 164: `FUN_0202C0E4(FUN_0202B578(FUN_02012134(GAMESYS)), value)`; 0.
pub fn cmd_164<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let y = h.fun_02012134(gs);
    let z = h.fun_0202b578(y);
    let v = h.fun_02159b10(vm, ctx);
    h.fun_0202c0e4(z, v);
    0
}

/// C: `sub_021af4b0`: command 165: var = `FUN_0202C0F8(FUN_0202B578(FUN_02012134(GAMESYS)), value)`.
pub fn cmd_165<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let y = h.fun_02012134(gs);
    let z = h.fun_0202b578(y);
    let var = h.fun_02159ae8(vm, ctx);
    let v = h.fun_02159b10(vm, ctx);
    let r = h.fun_0202c0f8(z, v);
    h.w16(var, r as u16);
    0
}

/// C: `sub_021af4e8`: the GMEVENT function of commands 171 / 589 (`(event, seq, work)`, work `{u16 a, u16 b, u32 mode, ctx}`):
/// `FUN_02005C64()` != 0 -> 0 (wait); else SCRCMD WORK +0x108 (`FUN_0215A6B8`) = `FUN_02006984(a, b, 0x40, 0, 0, 0, 0, tmp)`
/// where tmp is a 4-byte record filled by `FUN_02006CAC` for mode != 0 and null for mode 0; 1 (done). (The spec header
/// says `(b, a, ..)` for mode != 0; the code passes a, b in both cases.)
pub fn event_171<H: FldHooks>(h: &mut H, _event: u32, _seq: u32, work: u32) -> u32 {
    if h.fun_02005c64() != 0 {
        return 0;
    }
    let r = if h.r32(work + 4) != 0 {
        let b = h.r16(work + 2) as u32;
        let a = h.r16(work) as u32;
        let mut tmp = [0u8; 4];
        h.fun_02006cac(&mut tmp);
        h.fun_02006984(a, b, 0x40, 0, 0, 0, 0, Some(&tmp))
    } else {
        let a = h.r16(work) as u32;
        let b = h.r16(work + 2) as u32;
        h.fun_02006984(a, b, 0x40, 0, 0, 0, 0, None)
    };
    let ctx = h.r32(work + 8);
    h.fun_0215a6b8(ctx, r);
    1
}

// ---- name / string queries (181..189)

/// Commands 181..184: var = `f(FUN_0201292C(GAMEDATA), a, b, heap)` (a, b = values).
fn query_ab<H: FldHooks>(
    h: &mut H,
    vm: u32,
    ctx: u32,
    f: fn(&mut H, u32, u32, u32, u32) -> u32,
) -> u32 {
    let heap = h.fun_0215a5cc(ctx);
    let gd = h.fun_0215a5e4(ctx);
    let x = h.fun_0201292c(gd);
    let a = h.fun_02159b10(vm, ctx);
    let b = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let r = f(h, x, a, b, heap);
    h.w16(var, r as u16);
    0
}

/// C: `sub_021af544`: command 181: var = `FUN_02007E50(FUN_0201292C(GAMEDATA), a, b, heap)`.
pub fn cmd_181<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    query_ab(h, vm, ctx, |h, x, a, b, heap| h.fun_02007e50(x, a, b, heap))
}

/// C: `sub_021af594`: command 182: var = `FUN_02007F1C(FUN_0201292C(GAMEDATA), a, b, heap)`.
pub fn cmd_182<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    query_ab(h, vm, ctx, |h, x, a, b, heap| h.fun_02007f1c(x, a, b, heap))
}

/// C: `sub_021af5e4`: command 183: var = `FUN_02007E3C(FUN_0201292C(GAMEDATA), a, b, heap)`.
pub fn cmd_183<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    query_ab(h, vm, ctx, |h, x, a, b, heap| h.fun_02007e3c(x, a, b, heap))
}

/// C: `sub_021af634`: command 184: var = `FUN_02007F68(FUN_0201292C(GAMEDATA), a, b, heap)`.
pub fn cmd_184<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    query_ab(h, vm, ctx, |h, x, a, b, heap| h.fun_02007f68(x, a, b, heap))
}

/// C: `sub_021af684`: command 185: var = `FUN_02007FB8(FUN_0201292C(GAMEDATA), value, heap)`.
pub fn cmd_185<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let heap = h.fun_0215a5cc(ctx);
    let gd = h.fun_0215a5e4(ctx);
    let x = h.fun_0201292c(gd);
    let a = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let r = h.fun_02007fb8(x, a, heap);
    h.w16(var, r as u16);
    0
}

/// C: `sub_021af6c4`: command 186: var = [`item_word_021af750`]`(value)`.
pub fn cmd_186<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let v = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let r = item_word_021af750(h, v);
    h.w16(var, r);
    0
}

/// C: `sub_021af6e8`: command 187: var = `FUN_02020FE4(value, 5, heap)` (item data field 5).
pub fn cmd_187<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let heap = h.fun_0215a5cc(ctx);
    let v = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let r = h.fun_02020fe4(v, 5, heap);
    h.w16(var, r as u16);
    0
}

/// C: `sub_021af71c`: command 189: var = `FUN_02020FE4(value, 0xF, heap)` (item data field 0xF).
pub fn cmd_189<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let heap = h.fun_0215a5cc(ctx);
    let v = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let r = h.fun_02020fe4(v, 0xF, heap);
    h.w16(var, r as u16);
    0
}

/// C: `FUN_overlay_d_21__021af750`: the u16 `FUN_02021288(x)`.
pub fn item_word_021af750<H: FldHooks>(h: &mut H, x: u32) -> u16 {
    h.fun_02021288(x) as u16
}

// ---- WORDSET commands (76..99, 538, 539)

/// C: `sub_021af75c`: command 76: b = script byte; `FUN_0201F02C(WORDSET, b, FUN_02012944(GAMEDATA))`; 0 (the field-block
/// getter's result is unused).
pub fn cmd_76<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    h.fun_02158f58(sw);
    let gd = h.fun_0215a5e4(ctx);
    let x = h.fun_02012944(gd);
    let ws = h.fun_02158f74(sw);
    let b = read_byte(h, vm);
    h.fun_0201f02c(ws, b, x);
    0
}

/// C: `sub_021af794`: the shared body of commands 77 / 79 (entered by `ldr r2, =f; ldr r3, =0x021AF795; bx r3`): b = script
/// byte, v = value; `f(WORDSET, b, v)` ([`FldHooks::call_wordset_fn`]); 0.
pub fn wordset_body_77<H: FldHooks>(h: &mut H, vm: u32, ctx: u32, f: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let ws = h.fun_02158f74(sw);
    let b = read_byte(h, vm);
    let v = h.fun_02159b10(vm, ctx);
    h.call_wordset_fn(f, ws, b, v);
    0
}

/// C: `sub_021af7c4`: command 77: [`wordset_body_77`] with [`WORDSET_FN_77`] (`FUN_0201EF40`).
pub fn cmd_77<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    wordset_body_77(h, vm, ctx, WORDSET_FN_77)
}

/// C: `sub_021af7d4`: command 79: [`wordset_body_77`] with [`WORDSET_FN_79`] (`FUN_0201EF4C`).
pub fn cmd_79<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    wordset_body_77(h, vm, ctx, WORDSET_FN_79)
}

/// C: `sub_021af7e4`: command 78: b = script byte, v = value, w = value, c = script byte: w > 1 -> `FUN_0201EF5C(WORDSET, b,
/// v)`, else c != 0 -> `FUN_0201EF4C(WORDSET, b, v)`, else `FUN_0201EF40(WORDSET, b, v)`; returns that call's r0 (as the
/// original's tail call does).
pub fn cmd_78<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let ws = h.fun_02158f74(sw);
    let b = read_byte(h, vm);
    let v = h.fun_02159b10(vm, ctx);
    let w = h.fun_02159b10(vm, ctx);
    let c = read_byte(h, vm);
    if w > 1 {
        h.fun_0201ef5c(ws, b, v)
    } else if c != 0 {
        h.fun_0201ef4c(ws, b, v)
    } else {
        h.fun_0201ef40(ws, b, v)
    }
}

/// The shape of commands 80..99: b = script byte, v = value; `f(WORDSET, b, v)`; 0.
fn wordset_byte_value<H: FldHooks>(
    h: &mut H,
    vm: u32,
    ctx: u32,
    f: fn(&mut H, u32, u32, u32),
) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let ws = h.fun_02158f74(sw);
    let b = read_byte(h, vm);
    let v = h.fun_02159b10(vm, ctx);
    f(h, ws, b, v);
    0
}

/// C: `sub_021af844`: command 80: b = script byte; `FUN_0201EF34(WORDSET, b, FUN_020212B0(value))`; 0.
pub fn cmd_80<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    wordset_byte_value(h, vm, ctx, |h, ws, b, v| {
        let x = h.fun_020212b0(v);
        h.fun_0201ef34(ws, b, x);
    })
}

/// C: `sub_021af878`: command 81: b = script byte; `FUN_0201EF34(WORDSET, b, value)`; 0.
pub fn cmd_81<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    wordset_byte_value(h, vm, ctx, |h, ws, b, v| h.fun_0201ef34(ws, b, v))
}

/// C: `sub_021af8a8`: command 86: b = script byte; `FUN_0201F020(WORDSET, b, value)`; 0.
pub fn cmd_86<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    wordset_byte_value(h, vm, ctx, |h, ws, b, v| h.fun_0201f020(ws, b, v))
}

/// C: `sub_021af8d8`: the shared body of commands 87 / 88 (same shape as [`wordset_body_77`]): `f(WORDSET, script byte,
/// value)`; 0.
pub fn wordset_body_87<H: FldHooks>(h: &mut H, vm: u32, ctx: u32, f: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let ws = h.fun_02158f74(sw);
    let b = read_byte(h, vm);
    let v = h.fun_02159b10(vm, ctx);
    h.call_wordset_fn(f, ws, b, v);
    0
}

/// C: `sub_021af908`: command 87: [`wordset_body_87`] with [`WORDSET_FN_87`] (`FUN_0201EE60`).
pub fn cmd_87<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    wordset_body_87(h, vm, ctx, WORDSET_FN_87)
}

/// C: `sub_021af918`: command 88: [`wordset_body_87`] with [`WORDSET_FN_88`] (`FUN_0201EEC4`).
pub fn cmd_88<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    wordset_body_87(h, vm, ctx, WORDSET_FN_88)
}

/// C: `sub_021af928`: command 82: b = script byte; `FUN_0201F14C(WORDSET, b, value)`; 0.
pub fn cmd_82<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    wordset_byte_value(h, vm, ctx, |h, ws, b, v| h.fun_0201f14c(ws, b, v))
}

/// Commands 83 / 84: b = script byte; `f(WORDSET, b, FUN_0201AB28(party, value))` (party = `FUN_02012934(GAMEDATA)`); 0.
fn wordset_party_mon<H: FldHooks>(
    h: &mut H,
    vm: u32,
    ctx: u32,
    f: fn(&mut H, u32, u32, u32),
) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gd = h.fun_0215a5e4(ctx);
    let ws = h.fun_02158f74(sw);
    let b = read_byte(h, vm);
    let v = h.fun_02159b10(vm, ctx);
    let party = h.fun_02012934(gd);
    let mon = h.fun_0201ab28(party, v);
    f(h, ws, b, mon);
    0
}

/// C: `sub_021af958`: command 83: b = script byte; `FUN_0201EE84(WORDSET, b, party mon value)` (its nickname).
pub fn cmd_83<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    wordset_party_mon(h, vm, ctx, |h, ws, b, m| h.fun_0201ee84(ws, b, m))
}

/// C: `sub_021af9a4`: command 84: as command 83 with `FUN_0201EEF4`.
pub fn cmd_84<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    wordset_party_mon(h, vm, ctx, |h, ws, b, m| h.fun_0201eef4(ws, b, m))
}

/// Commands 85 / 91: b = script byte; `f(WORDSET, b, FUN_021C34F8(FUN_02188D50(field map), value))` (a daycare mon); 0.
fn wordset_daycare_mon<H: FldHooks>(
    h: &mut H,
    vm: u32,
    ctx: u32,
    f: fn(&mut H, u32, u32, u32),
) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let ws = h.fun_02158f74(sw);
    let b = read_byte(h, vm);
    let v = h.fun_02159b10(vm, ctx);
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    let d = h.fun_02188d50(fm);
    let mon = h.fun_021c34f8(d, v);
    f(h, ws, b, mon);
    0
}

/// C: `sub_021af9f0`: command 85: b = script byte; `FUN_0201EE84(WORDSET, b, daycare mon value)`.
pub fn cmd_85<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    wordset_daycare_mon(h, vm, ctx, |h, ws, b, m| h.fun_0201ee84(ws, b, m))
}

/// C: `sub_021afa38`: command 91: as command 85 with `FUN_0201EEF4`.
pub fn cmd_91<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    wordset_daycare_mon(h, vm, ctx, |h, ws, b, m| h.fun_0201eef4(ws, b, m))
}

/// C: `sub_021afa80`: command 92: b = script byte; v, n = values; `FUN_0201EFA4(WORDSET, b, v, n, 0, 1)` (a number word); 0.
pub fn cmd_92<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let ws = h.fun_02158f74(sw);
    let b = read_byte(h, vm);
    let v = h.fun_02159b10(vm, ctx);
    let n = h.fun_02159b10(vm, ctx);
    h.fun_0201efa4(ws, b, v, n, 0, 1);
    0
}

/// C: `sub_021afac8`: command 89: b = script byte; `FUN_0201F158(WORDSET, b, FUN_02013E4C(value))`; 0.
pub fn cmd_89<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    wordset_byte_value(h, vm, ctx, |h, ws, b, v| {
        let x = h.fun_02013e4c(v);
        h.fun_0201f158(ws, b, x);
    })
}

/// C: `sub_021afafc`: command 90: b = script byte; `FUN_0201EFC8(WORDSET, b, value)`; 0.
pub fn cmd_90<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    wordset_byte_value(h, vm, ctx, |h, ws, b, v| h.fun_0201efc8(ws, b, v))
}

/// C: `sub_021afb2c`: command 94: b = script byte; `FUN_0201F048(WORDSET, b, value)`; 0.
pub fn cmd_94<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    wordset_byte_value(h, vm, ctx, |h, ws, b, v| h.fun_0201f048(ws, b, v))
}

/// C: `sub_021afb5c`: command 95: b = script byte; `FUN_0201F198(WORDSET, b, value & 0xFF)`; 0.
pub fn cmd_95<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    wordset_byte_value(h, vm, ctx, |h, ws, b, v| h.fun_0201f198(ws, b, v & 0xFF))
}

/// C: `sub_021afb90`: command 99: b = script byte; `FUN_0201F1B0(WORDSET, b, value & 0xFF)`; 0.
pub fn cmd_99<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    wordset_byte_value(h, vm, ctx, |h, ws, b, v| h.fun_0201f1b0(ws, b, v & 0xFF))
}

/// C: `sub_021afbc4`: command 96: b = script byte (read first), v = value; `FUN_0201F184(WORDSET, b, v)`; 0.
pub fn cmd_96<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let b = read_byte(h, vm);
    let v = h.fun_02159b10(vm, ctx);
    let sw = h.fun_0215a5f4(ctx);
    let ws = h.fun_02158f74(sw);
    h.fun_0201f184(ws, b, v);
    0
}

/// C: `sub_021afbec`: the shared body of commands 97 / 98: b = script byte, v = value; `f(WORDSET, b, v & 0xFF)`; 0.
pub fn wordset_body_97<H: FldHooks>(h: &mut H, vm: u32, ctx: u32, f: u32) -> u32 {
    let b = read_byte(h, vm);
    let v = h.fun_02159b10(vm, ctx);
    let sw = h.fun_0215a5f4(ctx);
    let ws = h.fun_02158f74(sw);
    h.call_wordset_fn(f, ws, b, v & 0xFF);
    0
}

/// C: `sub_021afc18`: command 97: [`wordset_body_97`] with [`WORDSET_FN_97`] (`FUN_0201F0F8`).
pub fn cmd_97<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    wordset_body_97(h, vm, ctx, WORDSET_FN_97)
}

/// C: `sub_021afc28`: command 98: [`wordset_body_97`] with [`WORDSET_FN_98`] (`FUN_0201F124`).
pub fn cmd_98<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    wordset_body_97(h, vm, ctx, WORDSET_FN_98)
}

/// Commands 538 / 539: b = script byte; `FUN_0201EFA4(WORDSET, b, f(FUN_0200C0F0(FUN_02012EBC(GAMEDATA))), 9, 1, 1)`; 0.
fn wordset_number_538<H: FldHooks>(h: &mut H, vm: u32, ctx: u32, f: fn(&mut H, u32) -> u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gd = h.fun_0215a5e4(ctx);
    let x = h.fun_02012ebc(gd);
    let y = h.fun_0200c0f0(x);
    let ws = h.fun_02158f74(sw);
    let b = read_byte(h, vm);
    let z = f(h, y);
    h.fun_0201efa4(ws, b, z, 9, 1, 1);
    0
}

/// C: `sub_021afc38`: command 538: number word `FUN_0200C1DC(FUN_0200C0F0(FUN_02012EBC(GAMEDATA)))` (9 digits).
pub fn cmd_538<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    wordset_number_538(h, vm, ctx, |h, y| h.fun_0200c1dc(y))
}

/// C: `sub_021afc88`: command 539: as command 538 with `FUN_0200C1C4`.
pub fn cmd_539<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    wordset_number_538(h, vm, ctx, |h, y| h.fun_0200c1c4(y))
}

// ---- warps / map changes (190..196, 602)

/// The common head of the warp commands: (GAMESYS = `FUN_02158F54(SW)`, field map = `FUN_0201210C(GAMESYS)`).
fn warp_head<H: FldHooks>(h: &mut H, ctx: u32) -> (u32, u32) {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_02158f54(sw);
    let fm = h.fun_0201210c(gs);
    (gs, fm)
}

/// C: `sub_021afcd8`: command 194: a, b, c, d = u16 operands; pos = `{b << 16, 0, c << 16}`; call
/// `FUN_02184D38(GAMESYS, field map, a, &pos, d, 0)` from `FUN_02012104(GAMESYS)`; yield.
pub fn cmd_194<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let (gs, fm) = warp_head(h, ctx);
    let a = h.fun_02011330(vm);
    let b = h.fun_02011330(vm);
    let c = h.fun_02011330(vm);
    let d = h.fun_02011330(vm);
    let pos = vec3(b << 16, 0, c << 16);
    let ev = h.fun_02012104(gs);
    let child = h.fun_02184d38(gs, fm, a, &pos, d, 0);
    h.fun_02012374(ev, child);
    1
}

/// C: `sub_021afd44`: command 190: as command 194 with `FUN_02185534(GAMESYS, field map, a, &pos, d)`.
pub fn cmd_190<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let (gs, fm) = warp_head(h, ctx);
    let a = h.fun_02011330(vm);
    let b = h.fun_02011330(vm);
    let c = h.fun_02011330(vm);
    let d = h.fun_02011330(vm);
    let pos = vec3(b << 16, 0, c << 16);
    let ev = h.fun_02012104(gs);
    let child = h.fun_02185534(gs, fm, a, &pos, d);
    h.fun_02012374(ev, child);
    1
}

/// C: `sub_021afdac`: command 193: a, b, c = u16 operands; p1 = the player's position (`FUN_021A3038(FUN_02188CB4(field
/// map))`); e = `FUN_0216282C(FUN_02012844(GAMEDATA), FUN_0201296C(GAMEDATA), &p1, 9)`; e != 0 -> p2 = `FUN_02162A6C(e)`;
/// pos = `{b << 16, 0, c << 16}`; call `FUN_02184E40(GAMESYS, field map, &p2, a, &pos)` from `FUN_02012104(GAMESYS)`;
/// `FUN_021BB578(FUN_02188CD0(field map), FUN_021A3074(player), 0, 1)`; yield. When e is 0, p2 is the original's
/// uninitialised stack record: the port passes zeros (the Z19 entry state of the differential test; UNVERIFIED in the game).
pub fn cmd_193<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let (gs, fm) = warp_head(h, ctx);
    let a = h.fun_02011330(vm);
    let b = h.fun_02011330(vm);
    let c = h.fun_02011330(vm);
    let gd = h.fun_020120f4(gs);
    let t = h.fun_02012844(gd);
    let ew = h.fun_0201296c(gd);
    let player = h.fun_02188cb4(fm);
    let mut p1 = [0u8; 12];
    h.fun_021a3038(player, &mut p1);
    let mut p2 = [0u8; 12];
    let e = h.fun_0216282c(t, ew, &p1, 9);
    if e != 0 {
        h.fun_02162a6c(e, &mut p2);
    }
    let pos = vec3(b << 16, 0, c << 16);
    let ev = h.fun_02012104(gs);
    let child = h.fun_02184e40(gs, fm, &p2, a, &pos);
    h.fun_02012374(ev, child);
    let player = h.fun_02188cb4(fm);
    let d = h.fun_021a3074(player);
    let x = h.fun_02188cd0(fm);
    h.fun_021bb578(x, d, 0, 1);
    1
}

/// C: `sub_021afe64`: command 191: as command 194 with `FUN_02185050(GAMESYS, field map, a, &pos, d)`.
pub fn cmd_191<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let (gs, fm) = warp_head(h, ctx);
    let a = h.fun_02011330(vm);
    let b = h.fun_02011330(vm);
    let c = h.fun_02011330(vm);
    let d = h.fun_02011330(vm);
    let pos = vec3(b << 16, 0, c << 16);
    let ev = h.fun_02012104(gs);
    let child = h.fun_02185050(gs, fm, a, &pos, d);
    h.fun_02012374(ev, child);
    1
}

/// C: `sub_021afecc`: command 195: call `FUN_021850A8(GAMESYS, field map)` from `FUN_02012104(GAMESYS)`; yield.
pub fn cmd_195<H: FldHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let (gs, fm) = warp_head(h, ctx);
    let ev = h.fun_02012104(gs);
    let child = h.fun_021850a8(gs, fm);
    h.fun_02012374(ev, child);
    1
}

/// C: `sub_021afefc`: command 196: a..e = u16 operands; pos = `{b << 16, c << 16, d << 16}`; call `FUN_021854B0(GAMESYS,
/// field map, a, &pos, e)` from `FUN_02012104(GAMESYS)`; yield (`FUN_021A3074(FUN_02188CB4(field map))` is called, its result
/// unused).
pub fn cmd_196<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let (gs, fm) = warp_head(h, ctx);
    let a = h.fun_02011330(vm);
    let b = h.fun_02011330(vm);
    let c = h.fun_02011330(vm);
    let d = h.fun_02011330(vm);
    let e = h.fun_02011330(vm);
    let player = h.fun_02188cb4(fm);
    h.fun_021a3074(player);
    let pos = vec3(b << 16, c << 16, d << 16);
    let ev = h.fun_02012104(gs);
    let child = h.fun_021854b0(gs, fm, a, &pos, e);
    h.fun_02012374(ev, child);
    1
}

/// Commands 192 / 602: a..e = u16 operands; loc = `FUN_021BB3E8(FUN_02188C5C(field map), a, b, c, d, &loc, 0x15)`; call
/// `f(GAMESYS, field map, a, &loc, e, 0)` from `FUN_02012104(GAMESYS)`; yield.
fn warp_loc<H: FldHooks>(
    h: &mut H,
    vm: u32,
    ctx: u32,
    f: fn(&mut H, u32, u32, u32, &[u8; 12], u32, u32) -> u32,
) -> u32 {
    let (gs, fm) = warp_head(h, ctx);
    let a = h.fun_02011330(vm);
    let b = h.fun_02011330(vm);
    let c = h.fun_02011330(vm);
    let d = h.fun_02011330(vm);
    let e = h.fun_02011330(vm);
    let ev = h.fun_02012104(gs);
    let x = h.fun_02188c5c(fm);
    let mut loc = [0u8; 12];
    h.fun_021bb3e8(x, a, b, c, d, &mut loc, 0x15);
    let child = f(h, gs, fm, a, &loc, e, 0);
    h.fun_02012374(ev, child);
    1
}

/// C: `sub_021aff7c`: command 192: the location warp `FUN_02184D90`.
pub fn cmd_192<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    warp_loc(h, vm, ctx, |h, gs, fm, a, loc, e, z| {
        h.fun_02184d90(gs, fm, a, loc, e, z)
    })
}

/// C: `FUN_overlay_d_21__021b0000`: command 602: as command 192 with `FUN_02184DE8`.
pub fn cmd_602<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    warp_loc(h, vm, ctx, |h, gs, fm, a, loc, e, z| {
        h.fun_02184de8(gs, fm, a, loc, e, z)
    })
}

// ---- dispatch

/// Run the command handler at `handler` (the command table word, Thumb bit allowed) when it is a command of this area;
/// `None` otherwise. Command 128 gets 0 for its uninitialised-register path ([`cmd_128`]).
pub fn dispatch<H: FldHooks>(h: &mut H, handler: u32, vm: u32, ctx: u32) -> Option<u32> {
    let f: fn(&mut H, u32, u32) -> u32 = match handler & !1 {
        0x021A_F75C => cmd_76,
        0x021A_F7C4 => cmd_77,
        0x021A_F7E4 => cmd_78,
        0x021A_F7D4 => cmd_79,
        0x021A_F844 => cmd_80,
        0x021A_F878 => cmd_81,
        0x021A_F928 => cmd_82,
        0x021A_F958 => cmd_83,
        0x021A_F9A4 => cmd_84,
        0x021A_F9F0 => cmd_85,
        0x021A_F8A8 => cmd_86,
        0x021A_F908 => cmd_87,
        0x021A_F918 => cmd_88,
        0x021A_FAC8 => cmd_89,
        0x021A_FAFC => cmd_90,
        0x021A_FA38 => cmd_91,
        0x021A_FA80 => cmd_92,
        0x021A_FB2C => cmd_94,
        0x021A_FB5C => cmd_95,
        0x021A_FBC4 => cmd_96,
        0x021A_FC18 => cmd_97,
        0x021A_FC28 => cmd_98,
        0x021A_FB90 => cmd_99,
        0x021A_E9F0 => cmd_127,
        0x021A_E948 => |h, vm, ctx| cmd_128(h, vm, ctx, 0),
        0x021A_E994 => cmd_129,
        0x021A_EA3C => cmd_130,
        0x021A_E9CC => cmd_trainer_rec_14,
        0x021A_EA8C => cmd_132,
        0x021A_EABC => cmd_133,
        0x021A_EB3C => cmd_134,
        0x021A_EBB0 => cmd_three_vars,
        0x021A_EC1C => cmd_138,
        0x021A_EC40 => cmd_139,
        0x021A_EC84 => cmd_142,
        0x021A_ECC0 => cmd_classify_146,
        0x021A_ED00 => cmd_147,
        0x021A_ED24 => cmd_148,
        0x021A_EDE0 => cmd_152,
        0x021A_EEC0 => cmd_155,
        0x021A_EFD8 => cmd_158,
        0x021A_F0BC => cmd_159,
        0x021A_EF00 => cmd_160,
        0x021A_F0F8 => cmd_161,
        0x021A_F130 => cmd_162,
        0x021A_F458 => cmd_163,
        0x021A_F484 => cmd_164,
        0x021A_F4B0 => cmd_165,
        0x021A_F294 => cmd_166,
        0x021A_F2BC => cmd_167,
        0x021A_F2E4 => cmd_168,
        0x021A_F2F4 => cmd_169,
        0x021A_F370 => cmd_170,
        0x021A_F380 => cmd_171,
        0x021A_F448 => cmd_172,
        0x021A_F544 => cmd_181,
        0x021A_F594 => cmd_182,
        0x021A_F5E4 => cmd_183,
        0x021A_F634 => cmd_184,
        0x021A_F684 => cmd_185,
        0x021A_F6C4 => cmd_186,
        0x021A_F6E8 => cmd_187,
        0x021A_F71C => cmd_189,
        0x021A_FD44 => cmd_190,
        0x021A_FE64 => cmd_191,
        0x021A_FF7C => cmd_192,
        0x021A_FDAC => cmd_193,
        0x021A_FCD8 => cmd_194,
        0x021A_FECC => cmd_195,
        0x021A_FEFC => cmd_196,
        0x021A_FC38 => cmd_538,
        0x021A_FC88 => cmd_539,
        0x021A_EE28 => cmd_584,
        0x021A_F044 => cmd_587,
        0x021A_F3D8 => cmd_589,
        0x021A_EF10 => cmd_595,
        0x021A_EF60 => cmd_596,
        0x021A_EE78 => cmd_601,
        0x021B_0000 => cmd_602,
        0x021A_F1B0 => cmd_607,
        _ => return None,
    };
    Some(f(h, vm, ctx))
}

/// Run the wait callback `f` (VM +0x10, Thumb bit set) when it belongs to a command of this area (160, 168, 170, 172);
/// `None` otherwise.
pub fn call_wait<H: FldHooks>(h: &mut H, f: u32, vm: u32, ctx: u32) -> Option<u32> {
    Some(match f {
        WAIT_160 => wait_160(h, vm, ctx),
        WAIT_168 => wait_168(h, vm, ctx),
        WAIT_170 => wait_170(h, vm, ctx),
        WAIT_172 => wait_172(h, vm, ctx),
        _ => return None,
    })
}

/// Run the init proc `f` of the table 0x021705F0 (`script_sys::ScriptSysHooks::call_init_proc`, Thumb bit set) when it is
/// one of this area's (slots 10 / 12 / 13); `None` otherwise.
pub fn init_proc<H: FldHooks>(h: &mut H, f: u32, task: u32, seq: u32) -> Option<u32> {
    Some(match f {
        INIT_PROC_10 => init_proc_10(h, task, seq),
        INIT_PROC_12 => init_proc_12(h, task, seq),
        INIT_PROC_13 => init_proc_13(h, task, seq),
        _ => return None,
    })
}

#[cfg(test)]
mod tests;
