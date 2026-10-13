//! F8 (second batch): overlay-21 field-object / camera script commands, Thumb code 0x021B1668-0x021B2830 (F1-SPLIT row 21.44 from
//! the movement commands on): apply / wait for movements, field-object (MMDL) queries and setters, the player, camera / scroll
//! tasks, the talk lock, and thin wrappers of an overlay at 0x021F3xxx (55 command ids, 63 functions).
//!
//! Every function of the area is specced in `tools/rom-diff/specs/black-script/scrcmd_obj.spec` (header: command-id table and
//! roles). All 63 are ported here, listed in [`PORTED`] and checked with `tools/check-fn.sh black-script scrcmd_obj` (F8c).
//! [`COMMANDS`] maps the command ids to their handler addresses, [`dispatch`] runs a handler by its table address and
//! [`call_wait`] routes the wait callbacks of this area. MMDL = the overlay-10 field-object system (fldmmdl.c, F4a: its
//! functions are called by address through [`ObjHooks`], as the spec stubs them). Commands 384..410 / 608 wrap an overlay at
//! 0x021F3xxx that is outside the field set; those callees stay hooks (`fun_021f3xxx`).
//!
//! Pointers are `u32` addresses in the [`Mem`](crate::hooks::Mem) window; the callees are [`ObjHooks`] methods named by address
//! (the ones shared with the scrcmd core come from [`CoreHooks`]). In-area callees ([`object_of`], [`player_mmdl`],
//! [`talk_lock`]) are called as Rust (the ROM side runs them as real code).

use crate::scrcmd_core::CoreHooks;
use crate::vm::VM_PC;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021B_1668,
    0x021B_1698,
    0x021B_16DC,
    0x021B_16F0,
    0x021B_1700,
    0x021B_1738,
    0x021B_176C,
    0x021B_17B4,
    0x021B_17F8,
    0x021B_181C,
    0x021B_18B8,
    0x021B_18E4,
    0x021B_1950,
    0x021B_19B0,
    0x021B_19E0,
    0x021B_1A20,
    0x021B_1A44,
    0x021B_1B08,
    0x021B_1BB8,
    0x021B_1BE8,
    0x021B_1BF4,
    0x021B_1C2C,
    0x021B_1CB0,
    0x021B_1CFC,
    0x021B_1D20,
    0x021B_1D64,
    0x021B_1DEC,
    0x021B_1E74,
    0x021B_1E80,
    0x021B_1F60,
    0x021B_1FB8,
    0x021B_2040,
    0x021B_2070,
    0x021B_20B0,
    0x021B_2104,
    0x021B_2154,
    0x021B_21AC,
    0x021B_2218,
    0x021B_229C,
    0x021B_2380,
    0x021B_23B8,
    0x021B_23E0,
    0x021B_23F0,
    0x021B_240C,
    0x021B_2444,
    0x021B_247C,
    0x021B_24B8,
    0x021B_24F4,
    0x021B_252C,
    0x021B_2568,
    0x021B_25A4,
    0x021B_25DC,
    0x021B_2614,
    0x021B_2650,
    0x021B_268C,
    0x021B_269C,
    0x021B_26AC,
    0x021B_26E4,
    0x021B_271C,
    0x021B_273C,
    0x021B_277C,
    0x021B_27B4,
    0x021B_27F8,
];

/// Command id -> handler address (Thumb entry, bit 0 clear) of the commands of this area (the words of the command table
/// 0x0217062C at index id; checked against the ROM by the tests).
pub const COMMANDS: &[(u16, u32)] = &[
    (100, 0x021B_1698),
    (101, 0x021B_16F0),
    (102, 0x021B_1700),
    (103, 0x021B_176C),
    (104, 0x021B_17B4),
    (105, 0x021B_181C),
    (106, 0x021B_19B0),
    (107, 0x021B_18E4),
    (108, 0x021B_18B8),
    (109, 0x021B_1950),
    (110, 0x021B_17F8),
    (111, 0x021B_1CB0),
    (112, 0x021B_1C2C),
    (113, 0x021B_2070),
    (114, 0x021B_20B0),
    (115, 0x021B_1738),
    (116, 0x021B_1BF4),
    (117, 0x021B_1D20),
    (118, 0x021B_1D64),
    (119, 0x021B_1E80),
    (120, 0x021B_1F60),
    (121, 0x021B_19E0),
    (122, 0x021B_2104),
    (123, 0x021B_1FB8),
    (124, 0x021B_2154),
    (125, 0x021B_21AC),
    (126, 0x021B_2218),
    (384, 0x021B_2380),
    (385, 0x021B_23B8),
    (387, 0x021B_23F0),
    (388, 0x021B_240C),
    (389, 0x021B_2444),
    (391, 0x021B_247C),
    (392, 0x021B_24F4),
    (393, 0x021B_24B8),
    (394, 0x021B_268C),
    (395, 0x021B_269C),
    (396, 0x021B_252C),
    (397, 0x021B_2568),
    (398, 0x021B_25A4),
    (400, 0x021B_25DC),
    (401, 0x021B_2614),
    (402, 0x021B_2650),
    (403, 0x021B_26AC),
    (404, 0x021B_26E4),
    (405, 0x021B_271C),
    (407, 0x021B_273C),
    (408, 0x021B_277C),
    (409, 0x021B_27F8),
    (410, 0x021B_27B4),
    (577, 0x021B_1A20),
    (583, 0x021B_229C),
    (588, 0x021B_2040),
    (603, 0x021B_1DEC),
    (608, 0x021B_23E0),
];

/// Object id of the object with MMDL id 0x30 (`FUN_0216DE54(mmdls, 0x30)`; UNVERIFIED: the player's partner).
pub const OBJ_ID_PARTNER: u32 = 0xF2;
/// Object id that stands for the SCRIPT_WORK itself in [`object_of`].
pub const OBJ_ID_SW: u32 = 0xF1;
/// The flag byte of the talk lock (overlay-21 BSS): bit 0 player, 1 the 0x30 object, 2 the talk target, 3 its pair object
/// still moving.
pub const LOCK_FLAGS: u32 = 0x021E_E71C;

/// Wait callbacks of this area (Thumb bit set): command 101 ([`wait_movements`]), the talk lock ([`wait_lock`]), command 117
/// ([`wait_117`]).
pub const WAIT_MOVEMENTS: u32 = 0x021B_16DD;
pub const WAIT_LOCK: u32 = 0x021B_1A45;
pub const WAIT_117: u32 = 0x021B_1CFD;

/// The direction order command 119 tries (overlay-21 data at 0x021D93FC, u16 x 4).
pub const DIRS_119: [u16; 4] = [0, 1, 2, 3];
/// ROM address of [`DIRS_119`].
pub const DIRS_119_ADDR: u32 = 0x021D_93FC;
/// The 8-byte records command 119 passes to `FUN_021618A0` for directions 0..3 (overlay-21 data at 0x021D941C / 9414 / 940C /
/// 9404: `{u16 0x20 + 3 - .., u16 1, u16 0xFE, u16 0}`; UNVERIFIED meaning).
pub const RECORDS_119: [[u8; 8]; 4] = [
    [0x20, 0, 1, 0, 0xFE, 0, 0, 0],
    [0x21, 0, 1, 0, 0xFE, 0, 0, 0],
    [0x22, 0, 1, 0, 0xFE, 0, 0, 0],
    [0x23, 0, 1, 0, 0xFE, 0, 0, 0],
];
/// ROM addresses of [`RECORDS_119`] (direction 0..3).
pub const RECORDS_119_ADDR: [u32; 4] = [0x021D_941C, 0x021D_9414, 0x021D_940C, 0x021D_9404];
/// The VecFx32 `{0, 0xFA000, 0}` command 583 passes to `FUN_0216D8C0` (overlay-21 data at 0x021D9424).
pub const VEC_583: [u8; 12] = [0, 0, 0, 0, 0, 0xA0, 0x0F, 0, 0, 0, 0, 0];
/// ROM address of [`VEC_583`].
pub const VEC_583_ADDR: u32 = 0x021D_9424;

/// Callees of the scrcmd_obj functions that this area does not port, named by address (meaning per the spec; UNVERIFIED
/// unless said otherwise). The vm.c operand readers, the scrcmd ctx getters and `0215897C` (call a child event) come from
/// [`CoreHooks`].
pub trait ObjHooks: CoreHooks {
    // ---- script_sys / scrcmd_work (overlay 10)
    /// C: `FUN_0215A5EC` (script_sys `ctx_word3`): the MMDL system (word 3 of the scrcmd work).
    fn fun_0215a5ec(&mut self, ctx: u32) -> u32;
    /// C: `FUN_0215A6DC` (scrcmd_work): put movement `m` into a free slot (command 101 waits for the slots).
    fn fun_0215a6dc(&mut self, ctx: u32, m: u32);
    /// C: `FUN_0215A6FC` (scrcmd_work): 1 while a slotted movement still runs.
    fn fun_0215a6fc(&mut self, ctx: u32) -> u32;
    /// C: `FUN_02158F58` (script_sys): the SW field block address (+4 = the field map).
    fn fun_02158f58(&mut self, sw: u32) -> u32;
    /// C: `FUN_02158F90` (script_sys): the talk target MMDL (SW +8).
    fn fun_02158f90(&mut self, sw: u32) -> u32;

    // ---- overlay 10: fldmmdl (F4a; by address)
    /// C: `FUN_0216CE6C(mmdls, x, z, c, d, e, f, g)`: add a field object (args 5..8 on the stack); the new MMDL or 0.
    #[allow(clippy::too_many_arguments)]
    fn fun_0216ce6c(
        &mut self,
        mmdls: u32,
        x: u32,
        z: u32,
        c: u32,
        d: u32,
        e: u32,
        f: u32,
        g: u32,
    ) -> u32;
    /// C: `FUN_0216CEE4(mmdls, a, zone, data, ew, id)`: add a field object from the zone's event data (args 5 / 6 on the
    /// stack).
    fn fun_0216cee4(&mut self, mmdls: u32, a: u32, zone: u32, data: u32, ew: u32, id: u32) -> u32;
    /// C: `FUN_0216CF2C(mmdl)`: delete a field object.
    fn fun_0216cf2c(&mut self, mmdl: u32);
    /// C: `FUN_0216D454(mmdl)`: 1 while it is moving.
    fn fun_0216d454(&mut self, mmdl: u32) -> u32;
    /// C: `FUN_0216D588(mmdl)`: its OBJ id.
    fn fun_0216d588(&mut self, mmdl: u32) -> u32;
    /// C: `FUN_0216D5A0(mmdl)`.
    fn fun_0216d5a0(&mut self, mmdl: u32) -> u32;
    /// C: `FUN_0216D5B0(mmdl)`.
    fn fun_0216d5b0(&mut self, mmdl: u32) -> u32;
    /// C: `FUN_0216D5E0(mmdl, d)`: set its direction.
    fn fun_0216d5e0(&mut self, mmdl: u32, d: u32);
    /// C: `FUN_0216D5F8(mmdl)`: its direction.
    fn fun_0216d5f8(&mut self, mmdl: u32) -> u32;
    /// C: `FUN_0216D63C(mmdl, n)`: a u16 property n (0..2) of the MMDL.
    fn fun_0216d63c(&mut self, mmdl: u32, n: u32) -> u32;
    /// C: `FUN_0216D674(mmdl, v)`.
    fn fun_0216d674(&mut self, mmdl: u32, v: u32);
    /// C: `FUN_0216D83C(mmdl)`: grid x.
    fn fun_0216d83c(&mut self, mmdl: u32) -> u32;
    /// C: `FUN_0216D86C(mmdl)`: grid z.
    fn fun_0216d86c(&mut self, mmdl: u32) -> u32;
    /// C: `FUN_0216D88C(mmdl, &pos)`: its VecFx32 position.
    fn fun_0216d88c(&mut self, mmdl: u32, pos: &mut [u8; 12]);
    /// C: `FUN_0216D89C(mmdl, &pos)`: set its position.
    fn fun_0216d89c(&mut self, mmdl: u32, pos: &[u8; 12]);
    /// C: `FUN_0216D8C0(mmdl, &v)`: set a second VecFx32 of it.
    fn fun_0216d8c0(&mut self, mmdl: u32, v: &[u8; 12]);
    /// C: `FUN_0216D9A0(mmdls)`: pause all field objects.
    fn fun_0216d9a0(&mut self, mmdls: u32);
    /// C: `FUN_0216D9E4(mmdls)`: release the pause (returns its r0).
    fn fun_0216d9e4(&mut self, mmdls: u32) -> u32;
    /// C: `FUN_0216DA5C(mmdl)`: its pause / move flag.
    fn fun_0216da5c(&mut self, mmdl: u32) -> u32;
    /// C: `FUN_0216DAF8(mmdl)`: pause it.
    fn fun_0216daf8(&mut self, mmdl: u32);
    /// C: `FUN_0216DB04(mmdl)`: (talk lock) release its pause flag.
    fn fun_0216db04(&mut self, mmdl: u32);
    /// C: `FUN_0216DB80(mmdl, v)`.
    fn fun_0216db80(&mut self, mmdl: u32, v: u32);
    /// C: `FUN_0216DDDC(x, gx, gz, y, 0x8000, 0)`: the field object at a grid position, or 0.
    fn fun_0216dddc(&mut self, x: u32, gx: u32, gz: u32, y: u32, a: u32, b: u32) -> u32;
    /// C: `FUN_0216DE54(mmdls, id)`: the MMDL with an MMDL id, or 0.
    fn fun_0216de54(&mut self, mmdls: u32, id: u32) -> u32;
    /// C: `FUN_0216DE94(mmdls, id)`: the MMDL with an OBJ id, or 0.
    fn fun_0216de94(&mut self, mmdls: u32, id: u32) -> u32;
    /// C: `FUN_0216DEE0(mmdls)`: the player's MMDL.
    fn fun_0216dee0(&mut self, mmdls: u32) -> u32;
    /// C: `FUN_0216E084(mmdl, x, y, z, d)`: place it at a grid position (arg 5 on the stack).
    fn fun_0216e084(&mut self, mmdl: u32, x: u32, y: u32, z: u32, d: u32);
    /// C: `FUN_0216E0E8(mmdl, v)`.
    fn fun_0216e0e8(&mut self, mmdl: u32, v: u32);
    /// C: `FUN_0216E3D0(ew, v)`.
    fn fun_0216e3d0(&mut self, ew: u32, v: u32) -> u32;

    // ---- overlay 10: zone / event data
    /// C: `FUN_021618A0(gamesys, fieldmap, 0xFF, &rec)`: the event of command 119 (rec = one of [`RECORDS_119`]).
    fn fun_021618a0(&mut self, gamesys: u32, fm: u32, a: u32, rec: &[u8; 8]) -> u32;
    /// C: `FUN_02161AC0(gamesys, mmdl, &from, &to)`: the event of command 123.
    fn fun_02161ac0(&mut self, gamesys: u32, mmdl: u32, from: &[u8; 12], to: &[u8; 12]) -> u32;
    /// C: `FUN_02162740(x)`.
    fn fun_02162740(&mut self, x: u32) -> u32;
    /// C: `FUN_02162744(x)`.
    fn fun_02162744(&mut self, x: u32) -> u32;
    /// C: `FUN_021627C8(x, ew, &pos, dir)`: nonzero when direction `dir` from `pos` passes.
    fn fun_021627c8(&mut self, x: u32, ew: u32, pos: &[u8; 12], dir: u32) -> u32;
    /// C: `FUN_02163E58(mmdl, &pos, &h)`: the ground height at a position.
    fn fun_02163e58(&mut self, mmdl: u32, pos: &[u8; 12], h: &mut [u8; 4]);
    /// C: `FUN_021642A4(d)`: the opposite direction.
    fn fun_021642a4(&mut self, d: u32) -> u32;

    // ---- main
    /// C: `FUN_0201210C(gamesys)`: the field map.
    fn fun_0201210c(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_020127A4(gamedata)`.
    fn fun_020127a4(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012844(gamedata)`.
    fn fun_02012844(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012AC8(x)`: the u16 zone id at `x`.
    fn fun_02012ac8(&mut self, x: u32) -> u32;
    /// C: `FUN_020061E4(se)`: play sound effect `se` (`ScriptHost::play_se`).
    fn fun_020061e4(&mut self, se: u32);

    // ---- overlay 21
    /// C: `FUN_02188C7C(fieldmap)`.
    fn fun_02188c7c(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188C9C(fieldmap)`.
    fn fun_02188c9c(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188CB4(fieldmap)`: the player object.
    fn fun_02188cb4(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188D38(fieldmap)`: the camera / scroll task system.
    fn fun_02188d38(&mut self, fm: u32) -> u32;
    /// C: `FUN_02197D3C(mmdl, script address)` (fldmmdl_acmd, F4b): start a movement list.
    fn fun_02197d3c(&mut self, mmdl: u32, list: u32) -> u32;
    /// C: `FUN_0219BBA4(mmdl)`: the object paired with it, or 0.
    fn fun_0219bba4(&mut self, mmdl: u32) -> u32;
    /// C: `FUN_0219E2C0(mmdl, &t)`.
    fn fun_0219e2c0(&mut self, mmdl: u32, t: &[u8; 8]);
    /// C: `FUN_0219E348(mmdl)`.
    fn fun_0219e348(&mut self, mmdl: u32) -> u32;
    /// C: `FUN_021A2F48(player, v)`.
    fn fun_021a2f48(&mut self, player: u32, v: u32);
    /// C: `FUN_021A2F54(player)`: 1 when done (command 117).
    fn fun_021a2f54(&mut self, player: u32) -> u32;
    /// C: `FUN_021A3038(player, &pos)`: the player's position.
    fn fun_021a3038(&mut self, player: u32, pos: &mut [u8; 12]);
    /// C: `FUN_021A3050(player)`: its direction.
    fn fun_021a3050(&mut self, player: u32) -> u32;
    /// C: `FUN_021A3074(player)`: the player's MMDL.
    fn fun_021a3074(&mut self, player: u32) -> u32;
    /// C: `FUN_021A3098(player)`: its facing direction (0..3).
    fn fun_021a3098(&mut self, player: u32) -> u32;
    /// C: `FUN_021A3238(player)`.
    fn fun_021a3238(&mut self, player: u32) -> u32;
    /// C: `FUN_021A32A8(player, 0x8000)`: the object in front of the player, or 0.
    fn fun_021a32a8(&mut self, player: u32, v: u32) -> u32;
    /// C: `FUN_021B7DC0(x, &t)`: fill an 8-byte record.
    fn fun_021b7dc0(&mut self, x: u32, t: &mut [u8; 8]);
    /// C: `FUN_021B83FC(x, var1, var2, var3)`: store three values into the vars.
    fn fun_021b83fc(&mut self, x: u32, v1: u32, v2: u32, v3: u32);
    /// C: `FUN_021B8484(x, a, b, c)`.
    fn fun_021b8484(&mut self, x: u32, a: u32, b: u32, c: u32);
    /// C: `FUN_021C474C(gamesys, fieldmap, mmdl)`: the event of command 122.
    fn fun_021c474c(&mut self, gamesys: u32, fm: u32, mmdl: u32) -> u32;
    /// C: `FUN_021C6DE0(tasks, task, 0)`: start a camera / scroll task.
    fn fun_021c6de0(&mut self, tasks: u32, task: u32, a: u32);
    /// C: `FUN_021C7220(fieldmap, 0x18, 3, mmdl)`: a task.
    fn fun_021c7220(&mut self, fm: u32, a: u32, b: u32, mmdl: u32) -> u32;
    /// C: `FUN_021C7684(fieldmap, mmdl, 0x28, 0xFA)`: a task.
    fn fun_021c7684(&mut self, fm: u32, mmdl: u32, a: u32, b: u32) -> u32;
    /// C: `FUN_021C7878(fieldmap, 0x18, &v, mmdl)`: a task.
    fn fun_021c7878(&mut self, fm: u32, a: u32, v: &[u8; 12], mmdl: u32) -> u32;
    /// C: `FUN_021C78DC(fieldmap, a, &v, mmdl)`: a scroll task.
    fn fun_021c78dc(&mut self, fm: u32, a: u32, v: &[u8; 12], mmdl: u32) -> u32;

    // ---- the overlay at 0x021F3xxx (outside the field set; not ported)
    /// C: `FUN_021F3770(gamesys, flag)`.
    fn fun_021f3770(&mut self, gamesys: u32, v: u32) -> u32;
    /// C: `FUN_021F37A4(fieldmap)`.
    fn fun_021f37a4(&mut self, fm: u32);
    /// C: `FUN_021F37C8(fieldmap)`.
    fn fun_021f37c8(&mut self, fm: u32) -> u32;
    /// C: `FUN_021F37EC(gamesys, v)`.
    fn fun_021f37ec(&mut self, gamesys: u32, v: u32) -> u32;
    /// C: `FUN_021F3834(gamesys, v, flag)`.
    fn fun_021f3834(&mut self, gamesys: u32, v: u32, flag: u32) -> u32;
    /// C: `FUN_021F384C(gamesys, v)`.
    fn fun_021f384c(&mut self, gamesys: u32, v: u32) -> u32;
    /// C: `FUN_021F3888(gamesys, v)`.
    fn fun_021f3888(&mut self, gamesys: u32, v: u32) -> u32;
    /// C: `FUN_021F38C4(gamesys, v)`.
    fn fun_021f38c4(&mut self, gamesys: u32, v: u32);
    /// C: `FUN_021F3970(gamesys)`.
    fn fun_021f3970(&mut self, gamesys: u32);
    /// C: `FUN_021F398C(gamesys, v)`.
    fn fun_021f398c(&mut self, gamesys: u32, v: u32) -> u32;
    /// C: `FUN_021F3A28(gamesys, flag)`.
    fn fun_021f3a28(&mut self, gamesys: u32, v: u32) -> u32;
    /// C: `FUN_021F3A88(gamesys, a, b)`.
    fn fun_021f3a88(&mut self, gamesys: u32, a: u32, b: u32) -> u32;
    /// C: `FUN_021F3AD0(gamesys, a, b)`.
    fn fun_021f3ad0(&mut self, gamesys: u32, a: u32, b: u32) -> u32;
    /// C: `FUN_021F3B18(gamesys, v)`.
    fn fun_021f3b18(&mut self, gamesys: u32, v: u32) -> u32;
    /// C: `FUN_021F3B5C(gamesys, v)`.
    fn fun_021f3b5c(&mut self, gamesys: u32, v: u32) -> u32;
    /// C: `FUN_021F3D74(gamesys)`.
    fn fun_021f3d74(&mut self, gamesys: u32);
    /// C: `FUN_021F3E24(gamesys, v)`.
    fn fun_021f3e24(&mut self, gamesys: u32, v: u32) -> u32;
    /// C: `FUN_021F4134(gamesys, v)`.
    fn fun_021f4134(&mut self, gamesys: u32, v: u32);
    /// C: `FUN_021F4184(gamesys, v)`.
    fn fun_021f4184(&mut self, gamesys: u32, v: u32) -> u32;
    /// C: `FUN_021F41B0(gamesys, v)`.
    fn fun_021f41b0(&mut self, gamesys: u32, v: u32) -> u32;
    /// C: `FUN_021F41DC(gamesys, v)`.
    fn fun_021f41dc(&mut self, gamesys: u32, v: u32) -> u32;
    /// C: `FUN_021F42A8(gamesys)`.
    fn fun_021f42a8(&mut self, gamesys: u32);
}

/// The value of `v` as the s16 the original passes (`lsls 16; asrs 16`).
fn s16(v: u32) -> u32 {
    v as u16 as i16 as i32 as u32
}

/// A 12-byte VecFx32 `{x, y, z}` as the stack record the original passes.
fn vec3(x: u32, y: u32, z: u32) -> [u8; 12] {
    let mut v = [0u8; 12];
    v[0..4].copy_from_slice(&x.to_le_bytes());
    v[4..8].copy_from_slice(&y.to_le_bytes());
    v[8..12].copy_from_slice(&z.to_le_bytes());
    v
}

/// The field map (`*(FUN_02158F58(SW) + 4)`).
fn field_map<H: ObjHooks>(h: &mut H, sw: u32) -> u32 {
    let fb = h.fun_02158f58(sw);
    h.r32(fb + 4)
}

// ---- objects and movements

/// C: `FUN_overlay_d_21__021b1668`: the field object a script names: [`OBJ_ID_PARTNER`] -> `FUN_0216DE54(mmdls, 0x30)`, else
/// `FUN_0216DE94(mmdls, id)` (0 when there is none). An original bug, kept: for [`OBJ_ID_SW`] the original calls the SW
/// getter but returns its caller's r6, which it never sets (`stale_r6`; its only caller, command 100, has its movement offset
/// there, and passes it).
pub fn object_of<H: ObjHooks>(h: &mut H, ctx: u32, id: u32, stale_r6: u32) -> u32 {
    let mmdls = h.fun_0215a5ec(ctx);
    match id {
        OBJ_ID_PARTNER => h.fun_0216de54(mmdls, 0x30),
        OBJ_ID_SW => {
            h.fun_0215a5f4(ctx);
            stale_r6
        }
        _ => h.fun_0216de94(mmdls, id),
    }
}

/// C: `sub_021b1698`: command 100 (apply movement): id = value, ofs = u32 operand; obj = [`object_of`]; none -> 0; else
/// `FUN_0215A6DC(ctx, FUN_02197D3C(obj, PC + ofs))` (start the movement list at that script address and slot it for command
/// 101); 0. For id [`OBJ_ID_SW`] the object is `ofs` itself (the [`object_of`] bug).
pub fn cmd_100<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let id = h.fun_02159b10(vm, ctx);
    let ofs = h.fun_0201134c(vm);
    let obj = object_of(h, ctx, id, ofs);
    if obj == 0 {
        return 0;
    }
    let pc = h.r32(vm + VM_PC);
    let m = h.fun_02197d3c(obj, pc.wrapping_add(ofs));
    h.fun_0215a6dc(ctx, m);
    0
}

/// C: `sub_021b16dc`: the wait callback of command 101: done (1) once no slotted movement runs.
pub fn wait_movements<H: ObjHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    (h.fun_0215a6fc(ctx) == 0) as u32
}

/// C: `sub_021b16f0`: command 101 (wait for the movements): wait on [`WAIT_MOVEMENTS`]; yield.
pub fn cmd_101<H: ObjHooks>(h: &mut H, vm: u32, _ctx: u32) -> u32 {
    h.fun_020113d0(vm, WAIT_MOVEMENTS);
    1
}

/// C: `sub_021b1700`: command 102: var = 0, then var = `FUN_0216D5A0(FUN_0216DE94(mmdls, value))` (the object passed
/// unchecked).
pub fn cmd_102<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let mmdls = h.fun_0215a5ec(ctx);
    let var = h.fun_02159ae8(vm, ctx);
    h.w16(var, 0);
    let id = h.fun_02159b10(vm, ctx);
    let obj = h.fun_0216de94(mmdls, id);
    let r = h.fun_0216d5a0(obj);
    h.w16(var, r as u16);
    0
}

/// C: `sub_021b1738`: command 115: id, v = values; `FUN_0216E0E8(FUN_0216DE94(mmdls, id), v)` (unchecked); 0.
pub fn cmd_115<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let mmdls = h.fun_0215a5ec(ctx);
    let id = h.fun_02159b10(vm, ctx);
    let v = h.fun_02159b10(vm, ctx);
    let obj = h.fun_0216de94(mmdls, id);
    h.fun_0216e0e8(obj, v);
    0
}

/// C: `sub_021b176c`: command 103: obj = `FUN_0216DE94(mmdls, value)`; var1 = `FUN_0216D83C(obj)`, var2 = `FUN_0216D86C(obj)`
/// (grid x / z; unchecked).
pub fn cmd_103<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let mmdls = h.fun_0215a5ec(ctx);
    let id = h.fun_02159b10(vm, ctx);
    let obj = h.fun_0216de94(mmdls, id);
    let v1 = h.fun_02159ae8(vm, ctx);
    let v2 = h.fun_02159ae8(vm, ctx);
    let x = h.fun_0216d83c(obj);
    h.w16(v1, x as u16);
    let z = h.fun_0216d86c(obj);
    h.w16(v2, z as u16);
    0
}

/// C: `sub_021b17b4`: command 104: player = `FUN_021A3074(FUN_02188CB4(field map))`; var1 / var2 = its grid x / z.
pub fn cmd_104<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let fm = field_map(h, sw);
    let p = h.fun_02188cb4(fm);
    let pl = h.fun_021a3074(p);
    let v1 = h.fun_02159ae8(vm, ctx);
    let v2 = h.fun_02159ae8(vm, ctx);
    let x = h.fun_0216d83c(pl);
    h.w16(v1, x as u16);
    let z = h.fun_0216d86c(pl);
    h.w16(v2, z as u16);
    0
}

/// C: `sub_021b17f8`: command 110: var = `FUN_0216D5F8(`[`player_mmdl`]`)` (the player's direction).
pub fn cmd_110<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let pl = player_mmdl(h, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let d = h.fun_0216d5f8(pl);
    h.w16(var, d as u16);
    0
}

/// C: `sub_021b181c`: command 105: a..f = values; obj = `FUN_0216CE6C(mmdls, (s16)a, (s16)b, c, d, FUN_0216E3D0(EVENT WORK,
/// e), f, FUN_02188C9C(field map))` (add a field object); obj != 0 -> `FUN_0216DAF8(obj)`; 0.
pub fn cmd_105<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let fb = h.fun_02158f58(sw);
    let a = h.fun_02159b10(vm, ctx);
    let b = h.fun_02159b10(vm, ctx);
    let c = h.fun_02159b10(vm, ctx);
    let d = h.fun_02159b10(vm, ctx);
    let e = h.fun_02159b10(vm, ctx);
    let f = h.fun_02159b10(vm, ctx);
    let fm = h.r32(fb + 4);
    let x = h.fun_02188c9c(fm);
    let mmdls = h.fun_0215a5ec(ctx);
    let gd = h.fun_0215a5e4(ctx);
    let ew = h.fun_0201296c(gd);
    let g = h.fun_0216e3d0(ew, e);
    let obj = h.fun_0216ce6c(mmdls, s16(a), s16(b), c, d, g, f, x);
    if obj != 0 {
        h.fun_0216daf8(obj);
    }
    0
}

/// C: `sub_021b18b8`: command 108: obj = `FUN_0216DE94(mmdls, value)`; obj != 0 -> `FUN_0216CF2C(obj)` (delete it); 0.
pub fn cmd_108<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let mmdls = h.fun_0215a5ec(ctx);
    let id = h.fun_02159b10(vm, ctx);
    let obj = h.fun_0216de94(mmdls, id);
    if obj != 0 {
        h.fun_0216cf2c(obj);
    }
    0
}

/// C: `sub_021b18e4`: command 107: data = `FUN_02162744(FUN_02012844(GAMEDATA))`; data != 0 -> obj = `FUN_0216CEE4(mmdls,
/// FUN_02162740(..), zone id (FUN_02012AC8(FUN_020127A4(GAMEDATA))), data, EVENT WORK, value)` (add a zone event object);
/// obj != 0 -> `FUN_0216DAF8(obj)`; 0.
pub fn cmd_107<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let mmdls = h.fun_0215a5ec(ctx);
    let gd = h.fun_0215a5e4(ctx);
    let t = h.fun_02012844(gd);
    let data = h.fun_02162744(t);
    let x = h.fun_020127a4(gd);
    let zone = h.fun_02012ac8(x);
    let id = h.fun_02159b10(vm, ctx);
    if data != 0 {
        let ew = h.fun_0201296c(gd);
        let k = h.fun_02162740(t);
        let obj = h.fun_0216cee4(mmdls, k, zone, data, ew, id);
        if obj != 0 {
            h.fun_0216daf8(obj);
        }
    }
    0
}

/// C: `sub_021b1950`: command 109: id, a, b, c, d = values (a, b, c as s16); obj = `FUN_0216DE94(mmdls, id)`; obj != 0 ->
/// `FUN_0216E084(obj, a, b, c, d)` (place it); 0.
pub fn cmd_109<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let id = h.fun_02159b10(vm, ctx);
    let a = s16(h.fun_02159b10(vm, ctx));
    let b = s16(h.fun_02159b10(vm, ctx));
    let c = s16(h.fun_02159b10(vm, ctx));
    let d = h.fun_02159b10(vm, ctx);
    let mmdls = h.fun_0215a5ec(ctx);
    let obj = h.fun_0216de94(mmdls, id);
    if obj != 0 {
        h.fun_0216e084(obj, a, b, c, d);
    }
    0
}

/// C: `sub_021b19b0`: command 106: v = value; obj = `FUN_0216DE94(mmdls, v)`; obj != 0 -> var = `FUN_0216D5B0(obj)`; 0.
pub fn cmd_106<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let id = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let mmdls = h.fun_0215a5ec(ctx);
    let obj = h.fun_0216de94(mmdls, id);
    if obj != 0 {
        let r = h.fun_0216d5b0(obj);
        h.w16(var, r as u16);
    }
    0
}

/// C: `sub_021b19e0`: command 121: id, n = values; var = `FUN_0216D63C(object id, n)` when n <= 2 and the object exists (else
/// the var is left alone).
pub fn cmd_object_prop<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let id = h.fun_02159b10(vm, ctx);
    let n = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let mmdls = h.fun_0215a5ec(ctx);
    let obj = h.fun_0216de94(mmdls, id);
    if n <= 2 && obj != 0 {
        let v = h.fun_0216d63c(obj, n);
        h.w16(var, v as u16);
    }
    0
}

/// C: `sub_021b1a20`: command 577: obj = `FUN_0216DE94(mmdls, value)`; obj != 0 -> `FUN_0216DB80(obj, 1)`; 0.
pub fn cmd_577<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let id = h.fun_02159b10(vm, ctx);
    let mmdls = h.fun_0215a5ec(ctx);
    let obj = h.fun_0216de94(mmdls, id);
    if obj != 0 {
        h.fun_0216db80(obj, 1);
    }
    0
}

// ---- the talk lock

/// C: `sub_021b1a44`: the wait callback of the talk lock ([`talk_lock`]): every object whose [`LOCK_FLAGS`] bit is set and
/// that has stopped (player: `FUN_0216D454` == 1; the others: `FUN_0216DA5C` == 0; a pair object that is gone counts as
/// stopped) is paused and its bit cleared; done (1) once no bit is left.
pub fn wait_lock<H: ObjHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let target = h.fun_02158f90(sw);
    let player = player_mmdl(h, ctx);
    let mmdls = h.fun_0215a5ec(ctx);
    let g = LOCK_FLAGS;
    if h.r8(g) & 1 != 0 && h.fun_0216d454(player) == 1 {
        h.fun_0216daf8(player);
        let f = h.r8(g) & 0xFE;
        h.w8(g, f);
    }
    if h.r8(g) & 4 != 0 && h.fun_0216da5c(target) == 0 {
        h.fun_0216daf8(target);
        let f = h.r8(g) & 0xFB;
        h.w8(g, f);
    }
    if h.r8(g) & 2 != 0 {
        let o = h.fun_0216de54(mmdls, 0x30);
        if h.fun_0216da5c(o) == 0 {
            h.fun_0216daf8(o);
            let f = h.r8(g) & 0xFD;
            h.w8(g, f);
        }
    }
    if h.r8(g) & 8 != 0 {
        let pair = h.fun_0219bba4(target);
        let stopped = if pair == 0 {
            true
        } else if h.fun_0216da5c(pair) == 0 {
            h.fun_0216daf8(pair);
            true
        } else {
            false
        };
        if stopped {
            let f = h.r8(g) & 0xF7;
            h.w8(g, f);
        }
    }
    (h.r8(g) == 0) as u32
}

/// C: `FUN_overlay_d_21__021b1b08`: lock the field objects for a talk: [`LOCK_FLAGS`] = 0; `FUN_0216D9A0(mmdls)` (pause all);
/// the player still moving (`FUN_0216D454` == 0) -> bit 0 + `FUN_0216DB04(player)`; the talk target (`FUN_02158F90`) and its
/// pair object (`FUN_0219BBA4`) moving (`FUN_0216DA5C` == 1) -> bits 2 / 3 + `FUN_0216DB04`; wait on [`WAIT_LOCK`]; yield.
/// (`FUN_0216DE54(mmdls, 0x30)` is called, its result unused.)
pub fn talk_lock<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let mmdls = h.fun_0215a5ec(ctx);
    let target = h.fun_02158f90(sw);
    let player = player_mmdl(h, ctx);
    h.fun_0216de54(mmdls, 0x30);
    let pair = h.fun_0219bba4(target);
    h.w8(LOCK_FLAGS, 0);
    h.fun_0216d9a0(mmdls);
    if h.fun_0216d454(player) == 0 {
        let f = h.r8(LOCK_FLAGS) | 1;
        h.w8(LOCK_FLAGS, f);
        h.fun_0216db04(player);
    }
    if target != 0 && h.fun_0216da5c(target) == 1 {
        let f = h.r8(LOCK_FLAGS) | 4;
        h.w8(LOCK_FLAGS, f);
        h.fun_0216db04(target);
    }
    if pair != 0 && h.fun_0216da5c(pair) == 1 {
        let f = h.r8(LOCK_FLAGS) | 8;
        h.w8(LOCK_FLAGS, f);
        h.fun_0216db04(pair);
    }
    h.fun_020113d0(vm, WAIT_LOCK);
    1
}

/// C: `FUN_overlay_d_21__021b1bb8`: the body of scrcmd_core command 46 (its thunk at 0x021598E4 jumps here;
/// `CoreHooks::fun_021b1bb8`): no talk target -> `FUN_0216D9A0(mmdls)` (pause all), 0; else [`talk_lock`].
pub fn cmd_46_body<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    if h.fun_02158f90(sw) == 0 {
        let mmdls = h.fun_0215a5ec(ctx);
        h.fun_0216d9a0(mmdls);
        return 0;
    }
    talk_lock(h, vm, ctx)
}

/// C: `FUN_overlay_d_21__021b1be8`: `FUN_0216D9E4(mmdls)` (release the pause; returns its r0; `CoreHooks::fun_021b1be8`, the
/// first step of scrcmd_core command 47).
pub fn release_objects<H: ObjHooks>(h: &mut H, ctx: u32) -> u32 {
    let mmdls = h.fun_0215a5ec(ctx);
    h.fun_0216d9e4(mmdls)
}

/// C: `sub_021b1bf4`: command 116: d = `FUN_021642A4(FUN_0216D5F8(player MMDL))` (opposite of the player's direction); the
/// talk target exists -> `FUN_0216D5E0(target, d)`, `FUN_0216D674(target, 0)`; 0.
pub fn cmd_116<H: ObjHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let pl = player_mmdl(h, ctx);
    let x = h.fun_0216d5f8(pl);
    let d = h.fun_021642a4(x);
    let t = h.fun_02158f90(sw);
    if t != 0 {
        h.fun_0216d5e0(t, d);
        h.fun_0216d674(t, 0);
    }
    0
}

/// C: `sub_021b1c2c`: command 112: id var, found var = two var addresses; a, b, c = values; obj = `FUN_0216DDDC(FUN_02188C7C(
/// field map), (s16)a, (s16)c, b << 16, 0x8000, 0)` (the object at a grid position); obj -> found = 1, id = `FUN_0216D588(obj)`;
/// else found = 0. (The spec header names the vars the other way round; the first one gets the id.)
pub fn cmd_112<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let fm = field_map(h, sw);
    let id_var = h.fun_02159ae8(vm, ctx);
    let found = h.fun_02159ae8(vm, ctx);
    let a = h.fun_02159b10(vm, ctx);
    let b = h.fun_02159b10(vm, ctx);
    let c = h.fun_02159b10(vm, ctx);
    let x = h.fun_02188c7c(fm);
    let obj = h.fun_0216dddc(x, s16(a), s16(c), b << 16, 0x8000, 0);
    if obj != 0 {
        h.w16(found, 1);
        let id = h.fun_0216d588(obj);
        h.w16(id_var, id as u16);
    } else {
        h.w16(found, 0);
    }
    0
}

/// C: `sub_021b1cb0`: command 111: id var, found var; obj = `FUN_021A32A8(FUN_02188CB4(field map), 0x8000)` (the object in
/// front of the player); obj -> found = 1, id = `FUN_0216D588(obj)`; else found = 0.
pub fn cmd_111<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    let p = h.fun_02188cb4(fm);
    let id_var = h.fun_02159ae8(vm, ctx);
    let found = h.fun_02159ae8(vm, ctx);
    let obj = h.fun_021a32a8(p, 0x8000);
    if obj != 0 {
        h.w16(found, 1);
        let id = h.fun_0216d588(obj);
        h.w16(id_var, id as u16);
    } else {
        h.w16(found, 0);
    }
    0
}

/// C: `sub_021b1cfc`: the wait callback of command 117: done once `FUN_021A2F54(FUN_02188CB4(field map))` == 1.
pub fn wait_117<H: ObjHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let fm = field_map(h, sw);
    let p = h.fun_02188cb4(fm);
    (h.fun_021a2f54(p) == 1) as u32
}

/// C: `sub_021b1d20`: command 117: v = u16 operand; p = `FUN_02188CB4(field map)`; `FUN_021A2F48(p, v)`; `FUN_021A2F54(p)` ==
/// 1 -> 0; else wait on [`WAIT_117`], yield.
pub fn cmd_117<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let fb = h.fun_02158f58(sw);
    let v = h.fun_02011330(vm);
    let fm = h.r32(fb + 4);
    let p = h.fun_02188cb4(fm);
    h.fun_021a2f48(p, v);
    if h.fun_021a2f54(p) == 1 {
        return 0;
    }
    h.fun_020113d0(vm, WAIT_117);
    1
}

/// C: `sub_021b1d64` (command 118) / `sub_021b1dec` (command 603, an identical copy): s, a, b, c = values; s != 0 -> a = -a;
/// v = `{0, c == 0 ? -b << 12 : b << 12, 0}`; `FUN_021C6DE0(FUN_02188D38(field map), FUN_021C78DC(field map, a, &v,
/// FUN_021A3074(FUN_02188CB4(field map))), 0)` (a scroll task); 0.
pub fn cmd_118<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let fm = field_map(h, sw);
    let cam = h.fun_02188d38(fm);
    let s = h.fun_02159b10(vm, ctx);
    let mut a = h.fun_02159b10(vm, ctx);
    let b = h.fun_02159b10(vm, ctx);
    let c = h.fun_02159b10(vm, ctx);
    if s != 0 {
        a = a.wrapping_neg();
    }
    let y = if c == 0 {
        b.wrapping_neg() << 12
    } else {
        b << 12
    };
    let v = vec3(0, y, 0);
    let p = h.fun_02188cb4(fm);
    let pl = h.fun_021a3074(p);
    let t = h.fun_021c78dc(fm, a, &v, pl);
    h.fun_021c6de0(cam, t, 0);
    0
}

/// C: `sub_021b1dec`: command 603: the same code as command 118 ([`cmd_118`]).
pub fn cmd_603<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    cmd_118(h, vm, ctx)
}

/// C: `FUN_overlay_d_21__021b1e74`: the player's MMDL (`FUN_0216DEE0(mmdls)`).
pub fn player_mmdl<H: ObjHooks>(h: &mut H, ctx: u32) -> u32 {
    let mmdls = h.fun_0215a5ec(ctx);
    h.fun_0216dee0(mmdls)
}

/// C: `sub_021b1e80`: command 119: pos = the player's position (`FUN_021A3038`); d = the first direction of [`DIRS_119`] with
/// `FUN_021627C8(FUN_02012844(GAMEDATA), EVENT WORK, &pos, d)` != 0; d == the player's direction (`FUN_021A3050`) -> 0; d 0..3
/// -> child event `FUN_021618A0(GAMESYS, field map, 0xFF, &RECORDS_119[d])`, yield; other d -> 0. An original bug, kept: when
/// no direction passes, d is the caller's r7, which the function never sets (`stale_r7`; UNVERIFIED: the spec's stub always
/// passes, and [`dispatch`] passes 0, the Z19 entry value).
pub fn cmd_119<H: ObjHooks>(h: &mut H, _vm: u32, ctx: u32, stale_r7: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let fm = field_map(h, sw);
    let gd = h.fun_0215a5e4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let t = h.fun_02012844(gd);
    let ew = h.fun_0201296c(gd);
    let p = h.fun_02188cb4(fm);
    let mut pos = [0u8; 12];
    h.fun_021a3038(p, &mut pos);
    let mut d = stale_r7;
    for &di in &DIRS_119 {
        if h.fun_021627c8(t, ew, &pos, di as u32) != 0 {
            d = di as u32;
            break;
        }
    }
    if h.fun_021a3050(p) == d {
        return 0;
    }
    let Some(rec) = RECORDS_119.get(d as usize) else {
        return 0;
    };
    let ev = h.fun_021618a0(gs, fm, 0xFF, rec);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `FUN_overlay_d_21__021b1f60`: command 120: var = the player's facing direction (`FUN_021A3098`) for 0..3, else 0.
pub fn cmd_player_dir<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let fb = h.fun_02158f58(sw);
    let fm = h.r32(fb + 4);
    let player = h.fun_02188cb4(fm);
    let var = h.fun_02159ae8(vm, ctx);
    let d = h.fun_021a3098(player);
    h.w16(var, if d <= 3 { d as u16 } else { 0 });
    0
}

/// C: `sub_021b1fb8`: command 123: id, x, y, z = values; obj = `FUN_0216DE94(mmdls, id)`; from = its position
/// (`FUN_0216D88C`); to = `{x << 16 + 0x8000, y << 16, z << 16 + 0x8000}`; child event `FUN_02161AC0(GAMESYS, obj, &from,
/// &to)`; yield.
pub fn cmd_123<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let id = h.fun_02159b10(vm, ctx);
    let x = h.fun_02159b10(vm, ctx);
    let y = h.fun_02159b10(vm, ctx);
    let z = h.fun_02159b10(vm, ctx);
    let mmdls = h.fun_0215a5ec(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let obj = h.fun_0216de94(mmdls, id);
    let mut from = [0u8; 12];
    h.fun_0216d88c(obj, &mut from);
    let to = vec3(
        (x << 16).wrapping_add(0x8000),
        y << 16,
        (z << 16).wrapping_add(0x8000),
    );
    let ev = h.fun_02161ac0(gs, obj, &from, &to);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_021b2040`: command 588: var = `FUN_021A3238(FUN_02188CB4(field map))`.
pub fn cmd_588<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let fm = field_map(h, sw);
    let p = h.fun_02188cb4(fm);
    let var = h.fun_02159ae8(vm, ctx);
    let r = h.fun_021a3238(p);
    h.w16(var, r as u16);
    0
}

/// C: `sub_021b2070`: command 113: v1, v2, v3 = var addresses; `FUN_021B83FC(FUN_0219E348(player MMDL), v1, v2, v3)`; 0.
pub fn cmd_113<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let pl = player_mmdl(h, ctx);
    let v1 = h.fun_02159ae8(vm, ctx);
    let v2 = h.fun_02159ae8(vm, ctx);
    let v3 = h.fun_02159ae8(vm, ctx);
    let x = h.fun_0219e348(pl);
    h.fun_021b83fc(x, v1, v2, v3);
    0
}

/// C: `sub_021b20b0`: command 114: obj = `FUN_0216DE94(mmdls, value)`; v1, v2, v3 = var addresses;
/// `FUN_021B83FC(FUN_0219E348(obj), v1, v2, v3)`; 0.
pub fn cmd_114<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let mmdls = h.fun_0215a5ec(ctx);
    let id = h.fun_02159b10(vm, ctx);
    let obj = h.fun_0216de94(mmdls, id);
    let v1 = h.fun_02159ae8(vm, ctx);
    let v2 = h.fun_02159ae8(vm, ctx);
    let v3 = h.fun_02159ae8(vm, ctx);
    let x = h.fun_0219e348(obj);
    h.fun_021b83fc(x, v1, v2, v3);
    0
}

/// C: `sub_021b2104`: command 122: obj = `FUN_0216DE94(mmdls, value)`; child event `FUN_021C474C(GAMESYS, field map, obj)`;
/// yield.
pub fn cmd_122<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let sw = h.fun_0215a5f4(ctx);
    let mmdls = h.fun_0215a5ec(ctx);
    let id = h.fun_02159b10(vm, ctx);
    let obj = h.fun_0216de94(mmdls, id);
    let fm = field_map(h, sw);
    let ev = h.fun_021c474c(gs, fm, obj);
    h.fun_0215897c(sw, ev);
    1
}

/// The tail of commands 124 / 125: a, b, c = values; x = `FUN_0219E348(obj)`; `FUN_021B8484(x, a, b, c)`; t =
/// `FUN_021B7DC0(x)` (8 bytes); `FUN_0219E2C0(obj, &t)`; 0.
fn obj_8484<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32, obj: u32) -> u32 {
    let a = h.fun_02159b10(vm, ctx);
    let b = h.fun_02159b10(vm, ctx);
    let c = h.fun_02159b10(vm, ctx);
    let x = h.fun_0219e348(obj);
    h.fun_021b8484(x, a, b, c);
    let mut t = [0u8; 8];
    h.fun_021b7dc0(x, &mut t);
    h.fun_0219e2c0(obj, &t);
    0
}

/// C: `sub_021b2154`: command 124: [`obj_8484`] for the player MMDL.
pub fn cmd_124<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let pl = player_mmdl(h, ctx);
    obj_8484(h, vm, ctx, pl)
}

/// C: `sub_021b21ac`: command 125: as command 124 for obj = `FUN_0216DE94(mmdls, value)`.
pub fn cmd_125<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let mmdls = h.fun_0215a5ec(ctx);
    let id = h.fun_02159b10(vm, ctx);
    let obj = h.fun_0216de94(mmdls, id);
    obj_8484(h, vm, ctx, obj)
}

/// C: `sub_021b2218`: command 126: obj = `FUN_0216DE94(mmdls, value)`; cam = `FUN_02188D38(field map)`; `FUN_020061E4(0x676)`
/// (SE); v = `{0, 0x96000, 0}`; t1 = `FUN_021C7878(field map, 0x18, &v, obj)`; t2 = `FUN_021C7220(field map, 0x18, 3, obj)`;
/// `FUN_021C6DE0(cam, t2, 0)`; `FUN_021C6DE0(cam, t1, 0)`; 0.
pub fn cmd_126<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let id = h.fun_02159b10(vm, ctx);
    let mmdls = h.fun_0215a5ec(ctx);
    let obj = h.fun_0216de94(mmdls, id);
    let fm = field_map(h, sw);
    let cam = h.fun_02188d38(fm);
    h.fun_020061e4(0x676);
    let v = vec3(0, 0x96000, 0);
    let t1 = h.fun_021c7878(fm, 0x18, &v, obj);
    let t2 = h.fun_021c7220(fm, 0x18, 3, obj);
    h.fun_021c6de0(cam, t2, 0);
    h.fun_021c6de0(cam, t1, 0);
    0
}

/// C: `FUN_overlay_d_21__021b229c`: command 583: id, x, z, d = values; obj = `FUN_0216DE94(mmdls, id)`; none -> 0; p = `{x <<
/// 16 + 0x8000, 0xFA000, z << 16 + 0x8000}`; h = 0, `FUN_02163E58(obj, &p, &h)` (ground height); p.y = h; `FUN_0216E084(obj,
/// (s16)x, gy, (s16)z, d)` with gy = the s16 `((h >> 4) + (h < 0 ? 0xFFF : 0)) << 4 >> 16` (h / 16 rounded towards zero, in
/// tiles); `FUN_0216D89C(obj, &p)`; `FUN_0216D8C0(obj, &`[`VEC_583`]`)`; `FUN_021C6DE0(FUN_02188D38(field map),
/// FUN_021C7684(field map, obj, 0x28, 0xFA), 0)`; 0.
pub fn cmd_583<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let id = h.fun_02159b10(vm, ctx);
    let x = h.fun_02159b10(vm, ctx);
    let z = h.fun_02159b10(vm, ctx);
    let d = h.fun_02159b10(vm, ctx);
    let mmdls = h.fun_0215a5ec(ctx);
    let obj = h.fun_0216de94(mmdls, id);
    let fm = field_map(h, sw);
    let cam = h.fun_02188d38(fm);
    if obj == 0 {
        return 0;
    }
    let mut p = vec3(
        (x << 16).wrapping_add(0x8000),
        0xFA000,
        (z << 16).wrapping_add(0x8000),
    );
    let mut hb = [0u8; 4];
    h.fun_02163e58(obj, &p, &mut hb);
    let hy = u32::from_le_bytes(hb);
    p[4..8].copy_from_slice(&hb);
    let r3 = ((hy as i32) >> 4) as u32;
    let bias = (((r3 as i32) >> 11) as u32) >> 20;
    let gy = s16(r3.wrapping_add(bias).wrapping_shl(4) >> 16);
    h.fun_0216e084(obj, s16(x), gy, s16(z), d);
    h.fun_0216d89c(obj, &p);
    h.fun_0216d8c0(obj, &VEC_583);
    let t = h.fun_021c7684(fm, obj, 0x28, 0xFA);
    h.fun_021c6de0(cam, t, 0);
    0
}

// ---- wrappers of the overlay at 0x021F3xxx (384..410, 608)

/// The common shape of commands 384, 389, 391..393, 396..398, 400..404, 407..410: `e = f(GAMESYS, operand)`; e != 0 -> child
/// event e; yield (1 either way). `operand` reads the command's script operands (after the SW / GAMESYS getters).
fn child_or_none<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32, f: fn(&mut H, u32, u32) -> u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let e = f(h, gs, vm);
    if e != 0 {
        h.fun_0215897c(sw, e);
    }
    1
}

/// C: `sub_021b2380`: command 384: child event `FUN_021F3E24(GAMESYS, u16 operand & 0xFF)` if any; yield.
pub fn cmd_384<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    child_or_none(h, vm, ctx, |h, gs, vm| {
        let v = h.fun_02011330(vm) & 0xFF;
        h.fun_021f3e24(gs, v)
    })
}

/// C: `sub_021b23b8`: command 385: `FUN_021F4134(GAMESYS, u16 operand & 0xFF)`; 0 (the SW getter's result is unused).
pub fn cmd_385<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let v = h.fun_02011330(vm) & 0xFF;
    h.fun_021f4134(gs, v);
    0
}

/// C: `sub_021b23e0`: command 608: `FUN_021F3D74(GAMESYS)`; 0.
pub fn cmd_608<H: ObjHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    h.fun_021f3d74(gs);
    0
}

/// C: `sub_021b23f0`: command 387: `FUN_021F37A4(field map)`; 0 (the SW getter's result is unused).
pub fn cmd_387<H: ObjHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    h.fun_021f37a4(fm);
    0
}

/// C: `sub_021b240c`: command 388: var = `FUN_021F37C8(field map)` != 0.
pub fn cmd_388<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    let var = h.fun_02159ae8(vm, ctx);
    let r = h.fun_021f37c8(fm) != 0;
    h.w16(var, r as u16);
    0
}

/// C: `sub_021b2444`: command 389: child event `FUN_021F37EC(GAMESYS, u16 operand & 0xFF)` if any; yield.
pub fn cmd_389<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    child_or_none(h, vm, ctx, |h, gs, vm| {
        let v = h.fun_02011330(vm) & 0xFF;
        h.fun_021f37ec(gs, v)
    })
}

/// C: `sub_021b247c`: command 391: child event `FUN_021F3834(GAMESYS, u16 operand & 0xFF, 1)` if any; yield.
pub fn cmd_391<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    child_or_none(h, vm, ctx, |h, gs, vm| {
        let v = h.fun_02011330(vm) & 0xFF;
        h.fun_021f3834(gs, v, 1)
    })
}

/// C: `sub_021b24b8`: command 393: child event `FUN_021F3834(GAMESYS, u16 operand & 0xFF, 0)` if any; yield.
pub fn cmd_393<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    child_or_none(h, vm, ctx, |h, gs, vm| {
        let v = h.fun_02011330(vm) & 0xFF;
        h.fun_021f3834(gs, v, 0)
    })
}

/// C: `sub_021b24f4`: command 392: child event `FUN_021F3888(GAMESYS, u16 operand & 0xFF)` if any; yield.
pub fn cmd_392<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    child_or_none(h, vm, ctx, |h, gs, vm| {
        let v = h.fun_02011330(vm) & 0xFF;
        h.fun_021f3888(gs, v)
    })
}

/// C: `sub_021b252c`: command 396: child event `FUN_021F4184(GAMESYS, *var)` (the u16 at a var address) if any; yield.
pub fn cmd_396<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let v = h.r16(var) as u32;
    let e = h.fun_021f4184(gs, v);
    if e != 0 {
        h.fun_0215897c(sw, e);
    }
    1
}

/// C: `sub_021b2568`: command 397: child event `FUN_021F41B0(GAMESYS, *var)` if any; yield.
pub fn cmd_397<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let v = h.r16(var) as u32;
    let e = h.fun_021f41b0(gs, v);
    if e != 0 {
        h.fun_0215897c(sw, e);
    }
    1
}

/// C: `sub_021b25a4`: command 398: child event `FUN_021F41DC(GAMESYS, u16 operand)` if any; yield.
pub fn cmd_398<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    child_or_none(h, vm, ctx, |h, gs, vm| {
        let v = h.fun_02011330(vm);
        h.fun_021f41dc(gs, v)
    })
}

/// C: `sub_021b25dc`: command 400: child event `FUN_021F398C(GAMESYS, u16 operand)` if any; yield.
pub fn cmd_400<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    child_or_none(h, vm, ctx, |h, gs, vm| {
        let v = h.fun_02011330(vm);
        h.fun_021f398c(gs, v)
    })
}

/// C: `sub_021b2614`: command 401: child event `FUN_021F3A28(GAMESYS, u16 operand != 0)` if any; yield.
pub fn cmd_401<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    child_or_none(h, vm, ctx, |h, gs, vm| {
        let v = (h.fun_02011330(vm) != 0) as u32;
        h.fun_021f3a28(gs, v)
    })
}

/// C: `sub_021b2650`: command 402: child event `FUN_021F3770(GAMESYS, u16 operand != 0)` if any; yield.
pub fn cmd_402<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    child_or_none(h, vm, ctx, |h, gs, vm| {
        let v = (h.fun_02011330(vm) != 0) as u32;
        h.fun_021f3770(gs, v)
    })
}

/// C: `sub_021b268c`: command 394: `FUN_021F42A8(GAMESYS)`; 0.
pub fn cmd_394<H: ObjHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    h.fun_021f42a8(gs);
    0
}

/// C: `sub_021b269c`: command 395: `FUN_021F3970(GAMESYS)`; 0.
pub fn cmd_395<H: ObjHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    h.fun_021f3970(gs);
    0
}

/// C: `sub_021b26ac`: command 403: child event `FUN_021F384C(GAMESYS, u16 operand)` if any; yield.
pub fn cmd_403<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    child_or_none(h, vm, ctx, |h, gs, vm| {
        let v = h.fun_02011330(vm);
        h.fun_021f384c(gs, v)
    })
}

/// C: `sub_021b26e4`: command 404: child event `FUN_021F3888(GAMESYS, u16 operand)` (unmasked, unlike command 392) if any;
/// yield.
pub fn cmd_404<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    child_or_none(h, vm, ctx, |h, gs, vm| {
        let v = h.fun_02011330(vm);
        h.fun_021f3888(gs, v)
    })
}

/// C: `sub_021b271c`: command 405: `FUN_021F38C4(GAMESYS, u16 operand)`; 0.
pub fn cmd_405<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let v = h.fun_02011330(vm);
    h.fun_021f38c4(gs, v);
    0
}

/// C: `sub_021b273c`: command 407: child event `FUN_021F3A88(GAMESYS, u16 a, u16 b)` if any; yield.
pub fn cmd_407<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    child_or_none(h, vm, ctx, |h, gs, vm| {
        let a = h.fun_02011330(vm);
        let b = h.fun_02011330(vm);
        h.fun_021f3a88(gs, a, b)
    })
}

/// C: `sub_021b277c`: command 408: child event `FUN_021F3B5C(GAMESYS, u16 operand)` if any; yield.
pub fn cmd_408<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    child_or_none(h, vm, ctx, |h, gs, vm| {
        let v = h.fun_02011330(vm);
        h.fun_021f3b5c(gs, v)
    })
}

/// C: `sub_021b27b4`: command 410: child event `FUN_021F3AD0(GAMESYS, a, b)` (values) if any; yield.
pub fn cmd_410<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let a = h.fun_02159b10(vm, ctx);
    let b = h.fun_02159b10(vm, ctx);
    let e = h.fun_021f3ad0(gs, a, b);
    if e != 0 {
        h.fun_0215897c(sw, e);
    }
    1
}

/// C: `sub_021b27f8`: command 409: child event `FUN_021F3B18(GAMESYS, value)` if any; yield.
pub fn cmd_409<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let v = h.fun_02159b10(vm, ctx);
    let e = h.fun_021f3b18(gs, v);
    if e != 0 {
        h.fun_0215897c(sw, e);
    }
    1
}

// ---- dispatch

/// Run the command handler at `handler` (the command table word, Thumb bit allowed) when it is a command of this area;
/// `None` otherwise. Command 119 gets 0 for its uninitialised-register path ([`cmd_119`]). The bodies of scrcmd_core commands
/// 46 / 47 (`0x021B1BB8` [`cmd_46_body`], `0x021B1BE8` [`release_objects`]) are not table entries: route
/// `CoreHooks::fun_021b1bb8` / `fun_021b1be8` to them.
pub fn dispatch<H: ObjHooks>(h: &mut H, handler: u32, vm: u32, ctx: u32) -> Option<u32> {
    let f: fn(&mut H, u32, u32) -> u32 = match handler & !1 {
        0x021B_1698 => cmd_100,
        0x021B_16F0 => cmd_101,
        0x021B_1700 => cmd_102,
        0x021B_176C => cmd_103,
        0x021B_17B4 => cmd_104,
        0x021B_181C => cmd_105,
        0x021B_19B0 => cmd_106,
        0x021B_18E4 => cmd_107,
        0x021B_18B8 => cmd_108,
        0x021B_1950 => cmd_109,
        0x021B_17F8 => cmd_110,
        0x021B_1CB0 => cmd_111,
        0x021B_1C2C => cmd_112,
        0x021B_2070 => cmd_113,
        0x021B_20B0 => cmd_114,
        0x021B_1738 => cmd_115,
        0x021B_1BF4 => cmd_116,
        0x021B_1D20 => cmd_117,
        0x021B_1D64 => cmd_118,
        0x021B_1E80 => |h, vm, ctx| cmd_119(h, vm, ctx, 0),
        0x021B_1F60 => cmd_player_dir,
        0x021B_19E0 => cmd_object_prop,
        0x021B_2104 => cmd_122,
        0x021B_1FB8 => cmd_123,
        0x021B_2154 => cmd_124,
        0x021B_21AC => cmd_125,
        0x021B_2218 => cmd_126,
        0x021B_2380 => cmd_384,
        0x021B_23B8 => cmd_385,
        0x021B_23F0 => cmd_387,
        0x021B_240C => cmd_388,
        0x021B_2444 => cmd_389,
        0x021B_247C => cmd_391,
        0x021B_24F4 => cmd_392,
        0x021B_24B8 => cmd_393,
        0x021B_268C => cmd_394,
        0x021B_269C => cmd_395,
        0x021B_252C => cmd_396,
        0x021B_2568 => cmd_397,
        0x021B_25A4 => cmd_398,
        0x021B_25DC => cmd_400,
        0x021B_2614 => cmd_401,
        0x021B_2650 => cmd_402,
        0x021B_26AC => cmd_403,
        0x021B_26E4 => cmd_404,
        0x021B_271C => cmd_405,
        0x021B_273C => cmd_407,
        0x021B_277C => cmd_408,
        0x021B_27F8 => cmd_409,
        0x021B_27B4 => cmd_410,
        0x021B_1A20 => cmd_577,
        0x021B_229C => cmd_583,
        0x021B_2040 => cmd_588,
        0x021B_1DEC => cmd_603,
        0x021B_23E0 => cmd_608,
        _ => return None,
    };
    Some(f(h, vm, ctx))
}

/// Run the wait callback `f` (VM +0x10, Thumb bit set) when it belongs to this area (command 101, the talk lock, command
/// 117); `None` otherwise.
pub fn call_wait<H: ObjHooks>(h: &mut H, f: u32, vm: u32, ctx: u32) -> Option<u32> {
    Some(match f {
        WAIT_MOVEMENTS => wait_movements(h, vm, ctx),
        WAIT_LOCK => wait_lock(h, vm, ctx),
        WAIT_117 => wait_117(h, vm, ctx),
        _ => return None,
    })
}

#[cfg(test)]
mod tests;
