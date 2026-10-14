//! F8e: overlay-21 `scrcmd_shop.c`, Thumb code 0x021B2FD8-0x021B4BEC (F1-SPLIT row 21.46 minus its tail, which is
//! `scrcmd_proc_fld.c`): the shop command 329, its wait callback and camera step, the item lists, the purchase check and the
//! payment (13 logic functions). The shop UI (windows, list menu, sprites, the main loop 0x021B35A4, 36 functions) stays behind
//! hooks (D3).
//!
//! Every logic function of the area is specced in `tools/rom-diff/specs/black-script/scrcmd_shop.spec` (header: command-id
//! table, the shop work layout, the render / UI functions left out), ported here, listed in [`PORTED`] and checked with
//! `tools/check-fn.sh black-script scrcmd_shop`. [`COMMANDS`] maps command 329 to its handler, [`dispatch`] runs it and
//! [`call_wait`] its wait callback.
//!
//! Callees outside the area and the unported UI functions are [`ShopHooks`] methods named by address (vm.c, the operand
//! readers and the ctx getters come from [`CoreHooks`]). ROM data the functions read is embedded and compared with the build
//! by the tests: [`MART_LISTS`] (0x021DE1A8 / counts 0x021D95DE), [`MART_BY_BADGES`] (0x021D946C), [`SPECIAL_LISTS`]
//! (0x021DE164 / counts 0x021D9430), [`CAMERA_OFFSETS`] (0x021D9708), [`CAMERA_BACK`] (0x021D9738).

use crate::scrcmd_core::{u32_div, CoreHooks};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021B_2FD8,
    0x021B_30C8,
    0x021B_3158,
    0x021B_3258,
    0x021B_3350,
    0x021B_3370,
    0x021B_3568,
    0x021B_3834,
    0x021B_38E0,
    0x021B_39C4,
    0x021B_3A64,
    0x021B_4710,
    0x021B_49EC,
];

/// Command id -> handler address (Thumb entry, bit 0 clear) of the command of this area (the word of the command table
/// 0x0217062C at index id; checked against the ROM by the tests).
pub const COMMANDS: &[(u16, u32)] = &[(329, 0x021B_2FD8)];

/// Wait callback of command 329 ([`wait_329`], Thumb).
pub const WAIT_329: u32 = 0x021B_3159;
/// `__FILE__` string "scrcmd_shop.c" passed to `Heap_AllocDebug`.
pub const FILE: u32 = 0x021D_E210;
/// Size of the shop work W (the SW + 0x3C slot); the shop UI work W2 is W + 8.
pub const WORK_SIZE: u32 = 0x214;
pub const W2: u32 = 8;
/// Heap id the shop work keeps at W2 + 4.
pub const SHOP_HEAP: u16 = 0x15;

/// Offsets in the shop UI work W2 (= the shop work + 8).
pub const W2_HEAP: u32 = 0x4;
pub const W2_MONEY: u32 = 0xC;
pub const W2_BAG: u32 = 0x10;
pub const W2_RECORDS: u32 = 0x14;
pub const W2_BP: u32 = 0x18;
pub const W2_DYN: u32 = 0x1C;
/// The item list: `{u16 item, u16 -, u32 price}` records from W2 + 0x20.
pub const W2_LIST: u32 = 0x20;
/// u16: the number of list rows.
pub const W2_COUNT: u32 = 0x160;
/// `u8` flag (set by a list whose first item `FUN_02021288` accepts, or by the special list 1).
pub const W2_FLAG: u32 = 0x162;
/// `u8` flag: the shop sells for BP instead of money.
pub const W2_BP_SHOP: u32 = 0x163;
/// The list-menu data of the rows (`FUN_0201F81C`) and the word W2 + 0x188 handed with it to `FUN_0201F878`.
pub const W2_ROWS: u32 = 0x180;
pub const W2_ROWS_ARG: u32 = 0x188;
/// The chosen item (u16), its price and the purchase maximum.
pub const W2_ITEM: u32 = 0x1FC;
pub const W2_PRICE: u32 = 0x200;
pub const W2_MAX: u32 = 0x204;
/// Record counters bumped by a purchase (`FUN_02008DAC` / `FUN_02008DF0`).
pub const REC_PURCHASES: u32 = 0x15;
pub const REC_MONEY_SPENT: u32 = 0x16;
pub const REC_BP_SPENT: u32 = 0x22;
/// Shop kinds (command 329's value): 0xF7..0xFB dynamic lists, 0xFC mart list b + 6, 0xFD / 0xFE special lists 1 / 0 (BP),
/// 0xFF the mart by badge count.
pub const KIND_DYN_FIRST: u32 = 0xF7;
pub const KIND_DYN_LAST: u32 = 0xFB;
pub const KIND_MART: u32 = 0xFC;
pub const KIND_SPECIAL_1: u32 = 0xFD;
pub const KIND_SPECIAL_0: u32 = 0xFE;
pub const KIND_BADGES: u32 = 0xFF;

/// Mart list index by badge count (0x021D946C, overlay 21 ROM).
pub const MART_BY_BADGES_ADDR: u32 = 0x021D_946C;
pub const MART_BY_BADGES: [u8; 9] = [0, 1, 1, 2, 2, 3, 3, 4, 5];
/// The 26 mart item lists (pointer table 0x021DE1A8, lengths 0x021D95DE; overlay 21 ROM).
pub const MART_LISTS_ADDR: u32 = 0x021D_E1A8;
pub const MART_COUNTS_ADDR: u32 = 0x021D_95DE;
pub const MART_LISTS: [&[u16]; 26] = [
    &[0x4, 0x11],
    &[
        0x4, 0x3, 0x11, 0x1A, 0x12, 0x16, 0x15, 0x13, 0x14, 0x4E, 0x4F,
    ],
    &[
        0x4, 0x3, 0x11, 0x1A, 0x19, 0x1C, 0x12, 0x16, 0x15, 0x13, 0x14, 0x4E, 0x4F, 0x4C,
    ],
    &[
        0x4, 0x3, 0x2, 0x11, 0x1A, 0x19, 0x1C, 0x12, 0x16, 0x15, 0x13, 0x14, 0x1B, 0x4E, 0x4F,
        0x4C, 0x4D,
    ],
    &[
        0x4, 0x3, 0x2, 0x11, 0x1A, 0x19, 0x18, 0x1C, 0x12, 0x16, 0x15, 0x13, 0x14, 0x1B, 0x4E,
        0x4F, 0x4C, 0x4D,
    ],
    &[
        0x4, 0x3, 0x2, 0x11, 0x1A, 0x19, 0x18, 0x17, 0x1C, 0x12, 0x16, 0x15, 0x13, 0x14, 0x1B,
        0x4E, 0x4F, 0x4C, 0x4D,
    ],
    &[0x89, 0x8A, 0x8B, 0x8C, 0x8D, 0x8E, 0x8F],
    &[0xE, 0x89, 0x8A, 0x8B, 0x8C, 0x8D, 0x8E, 0x8F],
    &[0xE, 0x6, 0x89, 0x8A, 0x8B, 0x8C, 0x8D, 0x8E, 0x8F],
    &[
        0xE, 0x6, 0x8, 0x90, 0x89, 0x8A, 0x8B, 0x8C, 0x8D, 0x8E, 0x8F,
    ],
    &[0x15C, 0x162, 0x190, 0x191, 0x157, 0x15B, 0x168],
    &[
        0x6, 0x8, 0xD, 0x91, 0x89, 0x8A, 0x8B, 0x8C, 0x8D, 0x8E, 0x8F,
    ],
    &[0x14E, 0x152, 0x159, 0x16C],
    &[0x155, 0x160, 0x16D],
    &[
        0xD, 0xA, 0xF, 0x92, 0x89, 0x8A, 0x8B, 0x8C, 0x8D, 0x8E, 0x8F,
    ],
    &[0xE, 0x6, 0x8, 0xF, 0xD, 0xA, 0x9, 0xB],
    &[0x93, 0x89, 0x8A, 0x8B, 0x8C, 0x8D, 0x8E, 0x8F],
    &[0xB, 0x7, 0x89, 0x8A, 0x8B, 0x8C, 0x8D, 0x8E, 0x8F],
    &[0x94, 0x89, 0x8A, 0x8B, 0x8C, 0x8D, 0x8E, 0x8F],
    &[0x3B, 0x39, 0x3A, 0x37, 0x38, 0x3C, 0x3D, 0x3E],
    &[0x24, 0x22, 0x23, 0x25],
    &[0x13A, 0xFE, 0x13E, 0x13D, 0x13B, 0x140, 0x13F, 0xFF, 0x13C],
    &[
        0x11, 0x1A, 0x19, 0x18, 0x1C, 0x12, 0x16, 0x13, 0x14, 0x15, 0x1B, 0x4E, 0x241, 0x4F, 0x4C,
        0x4D,
    ],
    &[0x156, 0x18B],
    &[
        0x4, 0x3, 0x2, 0xE, 0x6, 0x8, 0xD, 0xF, 0xA, 0x89, 0x8A, 0x8B, 0x8C, 0x8D, 0x8E, 0x8F,
    ],
    &[0x2E, 0x2F, 0x31, 0x34, 0x30, 0x2D],
];
/// The 2 special lists (BP shops) of `{u16 item, u32 price}` records (pointer table 0x021DE164, lengths 0x021D9430;
/// overlay 21 ROM).
pub const SPECIAL_LISTS_ADDR: u32 = 0x021D_E164;
pub const SPECIAL_COUNTS_ADDR: u32 = 0x021D_9430;
pub const SPECIAL_LISTS: [&[(u16, u32)]; 2] = [
    &[
        (0x2E, 1),
        (0x31, 1),
        (0x2F, 1),
        (0x34, 1),
        (0x30, 1),
        (0x2D, 1),
        (0x121, 16),
        (0x122, 16),
        (0x123, 16),
        (0x124, 16),
        (0x125, 16),
        (0x126, 16),
        (0x110, 16),
        (0x111, 16),
        (0xD6, 32),
        (0x10F, 32),
        (0x221, 32),
        (0x222, 32),
        (0x21E, 32),
        (0x223, 32),
        (0x10B, 48),
        (0x129, 48),
        (0xE8, 48),
        (0x114, 48),
        (0x109, 48),
        (0x10A, 48),
        (0xE6, 48),
        (0xDC, 48),
        (0x11F, 48),
        (0x113, 48),
        (0x146, 48),
        (0x147, 48),
        (0xD5, 48),
        (0x10E, 48),
        (0x116, 48),
        (0x21D, 48),
        (0x220, 48),
        (0x32, 48),
    ],
    &[
        (0x15E, 36),
        (0x177, 36),
        (0x19E, 36),
        (0x19F, 36),
        (0x169, 48),
        (0x17A, 48),
        (0x182, 48),
        (0x183, 48),
        (0x187, 48),
        (0x194, 48),
    ],
];
/// Camera target offsets (VecFx32) by the player's direction (0x021D9708, overlay 21 ROM): the shop camera step.
pub const CAMERA_OFFSETS_ADDR: u32 = 0x021D_9708;
pub const CAMERA_OFFSETS: [[u32; 3]; 4] = [
    [0x40000, 0, 0xFFFF_0000],
    [0x40000, 0xFFFE_0000, 0],
    [0x40000, 0, 0],
    [0x50000, 0xFFFF_0000, 0],
];
/// The 6-word camera record the wait callback moves back with (0x021D9738, overlay 21 ROM).
pub const CAMERA_BACK_ADDR: u32 = 0x021D_9738;
pub const CAMERA_BACK: [u32; 6] = [0, 0, 0, 0, 0, 1];

/// Callees of the scrcmd_shop functions outside the area and the unported shop UI functions of the area, named by address
/// (meaning per the spec; UNVERIFIED unless stated). vm.c, the operand readers and the ctx getters come from [`CoreHooks`].
pub trait ShopHooks: CoreHooks {
    /// C: `FUN_0201210C(gamesys)` (main): the field map.
    fn fun_0201210c(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_02158FB4(sw)` (script_sys): the address of the SW + 0x3C slot.
    fn fun_02158fb4(&mut self, sw: u32) -> u32;
    /// C: `FUN_02188C54(fm)` (overlay 21).
    fn fun_02188c54(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188C58(fm)` (overlay 21): the field camera / effect object E.
    fn fun_02188c58(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188CB4(fm)` (overlay 21): the player.
    fn fun_02188cb4(&mut self, fm: u32) -> u32;
    /// C: `FUN_021904AC(x)` (overlay 21).
    fn fun_021904ac(&mut self, x: u32) -> u32;
    /// C: `FUN_020127C4(gamedata)` (main).
    fn fun_020127c4(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012F08(gamedata)` (main): the money object.
    fn fun_02012f08(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_0201292C(gamedata)` (main): the bag.
    fn fun_0201292c(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012EBC(gamedata)` (main): the save data.
    fn fun_02012ebc(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_020071CC(save, id)` (main): a save block (0x3A: the battle points).
    fn fun_020071cc(&mut self, save: u32, id: u32) -> u32;
    /// C: `FUN_02012F2C(gamedata)` (main): the record counters.
    fn fun_02012f2c(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012944(gamedata)` (main).
    fn fun_02012944(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_021655D4()` (overlay 21).
    fn fun_021655d4(&mut self);
    /// C: `Heap_AllocDebug(heap, size, from_end, file, line)` (`0x02030734`).
    fn fun_02030734(&mut self, heap: u32, size: u32, from_end: u32, file: u32, line: u32) -> u32;
    /// C: `Heap_Free(block)` (`0x020307B0`).
    fn fun_020307b0(&mut self, block: u32);
    /// C: `FUN_021A3050(player)` (overlay 21): the player's direction (0..3).
    fn fun_021a3050(&mut self, player: u32) -> u32;
    /// C: `FUN_0218EC54(E)` (overlay 21).
    fn fun_0218ec54(&mut self, e: u32);
    /// C: `FUN_0218EC60(E)` (overlay 21).
    fn fun_0218ec60(&mut self, e: u32);
    /// C: `FUN_0218ECB4(E, &t)` (overlay 21): the camera target (VecFx32).
    fn fun_0218ecb4(&mut self, e: u32, t: &mut [u8; 12]);
    /// C: `FUN_0218ECE4(E, &a)` (overlay 21): the camera angle (VecFx32).
    fn fun_0218ece4(&mut self, e: u32, a: &mut [u8; 12]);
    /// C: `FUN_0218F05C(E)` (overlay 21).
    fn fun_0218f05c(&mut self, e: u32);
    /// C: `FUN_0218F0A8(E)` (overlay 21).
    fn fun_0218f0a8(&mut self, e: u32);
    /// C: `FUN_0218F0E8(E)` (overlay 21): nonzero while busy.
    fn fun_0218f0e8(&mut self, e: u32) -> u32;
    /// C: `FUN_0218F1AC(E, v)` (overlay 21).
    fn fun_0218f1ac(&mut self, e: u32, v: u32);
    /// C: `FUN_0218F1B4(E)` (overlay 21).
    fn fun_0218f1b4(&mut self, e: u32) -> u32;
    /// C: `FUN_0218F848(E)` (overlay 21).
    fn fun_0218f848(&mut self, e: u32);
    /// C: `FUN_0218F878(E)` (overlay 21).
    fn fun_0218f878(&mut self, e: u32);
    /// C: `FUN_0218F918(E, &rec, frames)` (overlay 21): move the camera to the 0x48-byte record `rec`.
    fn fun_0218f918(&mut self, e: u32, rec: &[u8; 0x48], frames: u32);
    /// C: `FUN_0218F9C4(E, &rec, frames)` (overlay 21): move the camera with the 0x18-byte record `rec`.
    fn fun_0218f9c4(&mut self, e: u32, rec: &[u8; 0x18], frames: u32);
    /// C: `FUN_0218FE50(E)` (overlay 21): nonzero while the camera moves.
    fn fun_0218fe50(&mut self, e: u32) -> u32;
    /// C: `FUN_02193060(on)` (overlay 21).
    fn fun_02193060(&mut self, on: u32) -> u32;
    /// C: `FUN_021B35A4(gamesys, W2, kind, b)` (the shop UI main loop, D3): nonzero when the shop is closed.
    fn fun_021b35a4(&mut self, gamesys: u32, w2: u32, kind: u32, b: u32) -> u32;
    /// C: `FUN_021B3B14(W2, kind)` (shop UI: text resources): the kind.
    fn fun_021b3b14(&mut self, w2: u32, kind: u32) -> u32;
    /// C: `FUN_021B331C(W2, bp)` (shop UI: message strings).
    fn fun_021b331c(&mut self, w2: u32, bp: u32);
    /// C: `FUN_021B3BD8(W2)` (shop UI: BG setup).
    fn fun_021b3bd8(&mut self, w2: u32);
    /// C: `FUN_021B3FA0(W2)` (shop UI: windows).
    fn fun_021b3fa0(&mut self, w2: u32);
    /// C: `FUN_021B3F00(W2)` (shop UI: sprites).
    fn fun_021b3f00(&mut self, w2: u32);
    /// C: `FUN_021B40D4(W2)` (shop UI: money window).
    fn fun_021b40d4(&mut self, w2: u32);
    /// C: `FUN_021B43EC(W2)` (shop UI: list menu).
    fn fun_021b43ec(&mut self, w2: u32) -> u32;
    /// C: `FUN_021B385C(W2, row, flag, item, msgdata)` (shop UI: list row text).
    fn fun_021b385c(&mut self, w2: u32, row: u32, flag: u32, item: u32, msg: u32);
    /// C: `FUN_021B4714(W2, msg, item, x)` (shop UI: print a message).
    fn fun_021b4714(&mut self, w2: u32, msg: u32, item: u32, x: u32);
    /// C: `FUN_021B4804(W2, flag)` (shop UI: the count prompt).
    fn fun_021b4804(&mut self, w2: u32, flag: u32);
    /// C: `FUN_0200C258(money)` (main): the badge count.
    fn fun_0200c258(&mut self, x: u32) -> u32;
    /// C: `FUN_0200C274(money)` (main): the money.
    fn fun_0200c274(&mut self, money: u32) -> u32;
    /// C: `FUN_0200DABC(bp)` (main): the battle points.
    fn fun_0200dabc(&mut self, bp: u32) -> u32;
    /// C: `FUN_0200C29C(money, n)` (main): subtract money.
    fn fun_0200c29c(&mut self, money: u32, n: u32);
    /// C: `FUN_0200DA98(bp, n)` (main): subtract battle points (`n` u16).
    fn fun_0200da98(&mut self, bp: u32, n: u32);
    /// C: `FUN_02008DAC(records, id, n)` (main): add `n` to a record counter.
    fn fun_02008dac(&mut self, records: u32, id: u32, n: u32);
    /// C: `FUN_02008DF0(records, id)` (main): increment a record counter.
    fn fun_02008df0(&mut self, records: u32, id: u32) -> u32;
    /// C: `FUN_02007FB8(bag, item, heap)` (main): how many of `item` the bag holds.
    fn fun_02007fb8(&mut self, bag: u32, item: u32, heap: u32) -> u32;
    /// C: `FUN_02007E3C(bag, item, n, heap)` (main): nonzero when `n` more of `item` fit.
    fn fun_02007e3c(&mut self, bag: u32, item: u32, n: u32, heap: u32) -> u32;
    /// C: `FUN_0200823C(item)` (main): the most of `item` the bag holds.
    fn fun_0200823c(&mut self, item: u32) -> u32;
    /// C: `FUN_02021288(item)` (main): an item flag.
    fn fun_02021288(&mut self, item: u32) -> u32;
    /// C: `FUN_02020FE4(item, 0, heap)` (main): the item data.
    fn fun_02020fe4(&mut self, item: u32, x: u32, heap: u32) -> u32;
    /// C: `FUN_0201102C(data)` (main): the price in the item data.
    fn fun_0201102c(&mut self, data: u32) -> u32;
    /// C: `FUN_02045B38(a, b, arc, heap)` (main): open message data.
    fn fun_02045b38(&mut self, a: u32, b: u32, arc: u32, heap: u32) -> u32;
    /// C: `FUN_02045C04(msgdata)` (main): close message data.
    fn fun_02045c04(&mut self, msg: u32);
    /// C: `FUN_0201F81C(n, heap)` (main): list-menu data for `n` rows.
    fn fun_0201f81c(&mut self, n: u32, heap: u32) -> u32;
    /// C: `FUN_0201F878(row, a, b, c, heap)` (main): add a list-menu row.
    fn fun_0201f878(&mut self, row: u32, a: u32, b: u32, c: u32, heap: u32) -> u32;
    /// C: `FUN_02186670(x, list, &count, n, heap)` (overlay 21): fill the dynamic list `n` (u16 `*count` = its rows).
    fn fun_02186670(&mut self, x: u32, list: u32, count: u32, n: u32, heap: u32);
}

/// C: `sub_021b2fd8`: command 329 (shop): v = kind; var (unused); W = `Heap_AllocDebug(4, 0x214, 1, "scrcmd_shop.c", 0xFC)`
/// into the SW + 0x3C slot, W2 = W + 8 = {+4 u16 heap 0x15, +8 `FUN_021904AC(FUN_02188C54(fm))`, +0xC money, +0x10 bag, +0x14
/// records, +0x18 BP (save block 0x3A), +0x1C `FUN_020127C4(GAMEDATA)`}; W[2] = (u8)kind, and for a kind outside 0xF7..0xFB /
/// 0xFD..0xFF also W[3] = (u8)kind; waits on [`wait_329`]; kinds other than 0xFE / 0xFD call `FUN_021655D4()`; 1.
pub fn cmd_329<H: ShopHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let v = h.fun_02159b10(vm, ctx);
    h.fun_02159ae8(vm, ctx);
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    let fm = h.fun_0201210c(gs);
    let sw = h.fun_0215a5f4(ctx);
    let slot = h.fun_02158fb4(sw);
    let x = h.fun_02188c54(fm);
    let y = h.fun_020127c4(gd);
    let w = h.fun_02030734(4, WORK_SIZE, 1, FILE, 0xFC);
    h.w32(slot, w);
    let t = h.fun_021904ac(x);
    h.w32(w + 0x10, t);
    let t = h.fun_02012f08(gd);
    h.w32(w + 0x14, t);
    h.w16(w + 0xC, SHOP_HEAP);
    let t = h.fun_0201292c(gd);
    h.w32(w + 0x18, t);
    let save = h.fun_02012ebc(gd);
    let t = h.fun_020071cc(save, 0x3A);
    h.w32(w + 0x20, t);
    let t = h.fun_02012f2c(gd);
    h.w32(w + 0x1C, t);
    h.w32(w + 0x24, y);
    match v {
        KIND_DYN_FIRST..=KIND_DYN_LAST | KIND_SPECIAL_1..=KIND_BADGES => h.w8(w + 2, v as u8),
        _ => {
            h.w8(w + 2, v as u8);
            h.w8(w + 3, v as u8);
        }
    }
    h.fun_020113d0(vm, WAIT_329);
    if v != KIND_SPECIAL_0 && v != KIND_SPECIAL_1 {
        h.fun_021655d4();
    }
    1
}

/// C: `FUN_overlay_d_21__021b30c8`: the shop camera step (W, fm, E): W+4 = `FUN_0218F1B4(E)` (kept to restore it);
/// `FUN_0218F1AC(E, 0)`; `FUN_0218F848(E)`; d = the player's direction; a = the camera angle, t = the camera target;
/// `FUN_0218EC54(E)`; t += [`CAMERA_OFFSETS`]`[d]`; `FUN_0218F918(E, &{a, t, .., +0x30..+0x40 0, +0x44 1}, 0xA)` (the other
/// words of the stack record are never set: 0 here, the Z19 test state).
pub fn shop_camera<H: ShopHooks>(h: &mut H, w: u32, fm: u32, e: u32) {
    let v = h.fun_0218f1b4(e);
    h.w32(w + 4, v);
    h.fun_0218f1ac(e, 0);
    h.fun_0218f848(e);
    let player = h.fun_02188cb4(fm);
    let d = h.fun_021a3050(player);
    let mut rec = [0u8; 0x48];
    let mut a = [0u8; 12];
    h.fun_0218ece4(e, &mut a);
    rec[..12].copy_from_slice(&a);
    let mut t = [0u8; 12];
    h.fun_0218ecb4(e, &mut t);
    h.fun_0218ec54(e);
    let ofs = CAMERA_OFFSETS[d as usize];
    for k in 0..3 {
        let c = u32::from_le_bytes(t[4 * k..4 * k + 4].try_into().unwrap()).wrapping_add(ofs[k]);
        rec[0xC + 4 * k..0x10 + 4 * k].copy_from_slice(&c.to_le_bytes());
    }
    rec[0x44..0x48].copy_from_slice(&1u32.to_le_bytes());
    h.fun_0218f918(e, &rec, 0xA);
}

/// C: `sub_021b3158`: the wait callback of command 329: W = *SW slot; state W[0]: 0 `FUN_0218F0A8(E)`; 1 wait for
/// `FUN_0218F0E8(E) == 0`; 2 [`shop_camera`]; 3 wait for the camera (`FUN_0218FE50`), then W[8] = 0, `FUN_02193060(0)`; 4 the
/// shop UI `FUN_021B35A4(GAMESYS, W + 8, W[2], W[3])` until it answers nonzero; 5 `FUN_0218F9C4(E, &CAMERA_BACK, 8)`, W[1] = 0;
/// 6 wait for the camera, then `FUN_0218EC60(E)`, `FUN_0218F1AC(E, W+4)`, `FUN_0218F05C(E)`, `FUN_0218F878(E)`,
/// `FUN_02193060(1)`; each passes to the next state (0: the command waits); 7 frees W, 1 (done).
pub fn wait_329<H: ShopHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let slot = h.fun_02158fb4(sw);
    let w = h.r32(slot);
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    let e = h.fun_02188c58(fm);
    match h.r8(w) {
        0 => h.fun_0218f0a8(e),
        1 => {
            if h.fun_0218f0e8(e) != 0 {
                return 0;
            }
        }
        2 => shop_camera(h, w, fm, e),
        3 => {
            if h.fun_0218fe50(e) != 0 {
                return 0;
            }
            let s = h.r8(w);
            h.w8(w, s.wrapping_add(1));
            h.w8(w + 8, 0);
            h.fun_02193060(0);
            return 0;
        }
        4 => {
            let (kind, b) = (u32::from(h.r8(w + 2)), u32::from(h.r8(w + 3)));
            if h.fun_021b35a4(gs, w + W2, kind, b) == 0 {
                return 0;
            }
        }
        5 => {
            let mut rec = [0u8; 0x18];
            for (k, v) in CAMERA_BACK.iter().enumerate() {
                rec[4 * k..4 * k + 4].copy_from_slice(&v.to_le_bytes());
            }
            h.fun_0218f9c4(e, &rec, 8);
            let s = h.r8(w);
            h.w8(w, s.wrapping_add(1));
            h.w8(w + 1, 0);
            return 0;
        }
        6 => {
            if h.fun_0218fe50(e) != 0 {
                return 0;
            }
            h.fun_0218ec60(e);
            let v = h.r32(w + 4);
            h.fun_0218f1ac(e, v);
            h.fun_0218f05c(e);
            h.fun_0218f878(e);
            h.fun_02193060(1);
        }
        7 => {
            let w = h.r32(slot);
            h.fun_020307b0(w);
            return 1;
        }
        _ => return 0,
    }
    let s = h.r8(w);
    h.w8(w, s.wrapping_add(1));
    0
}

/// C: `FUN_overlay_d_21__021b3258`: shop_init(GAMESYS, W2, kind, b): W2[0x162] = 0; t = `FUN_021B3B14(W2, kind)`; t
/// 0xF7..0xFB -> [`dyn_list`]; 0xFD -> W2[0x163] = W2[0x162] = 1, [`special_list`]`(W2, 1)`; 0xFE -> [`special_list`]`(W2, 0)`,
/// W2[0x163] = 1; 0xFF -> [`mart_list`]`(W2, t, b, badge count)`; else [`mart_list`]`(W2, t, b, 0)`; then the UI setup
/// `FUN_021B331C(W2, W2[0x163])`, `FUN_021B3BD8` / `3FA0` / `3F00` / `40D4` / `43EC(W2)`.
pub fn shop_init<H: ShopHooks>(h: &mut H, gs: u32, w2: u32, kind: u32, b: u32) {
    let gd = h.fun_020120f4(gs);
    h.fun_02012944(gd);
    h.w8(w2 + W2_FLAG, 0);
    let t = h.fun_021b3b14(w2, kind);
    match t {
        KIND_DYN_FIRST..=KIND_DYN_LAST => dyn_list(h, w2, t),
        KIND_SPECIAL_1 => {
            h.w8(w2 + W2_BP_SHOP, 1);
            h.w8(w2 + W2_FLAG, 1);
            special_list(h, w2, 1);
        }
        KIND_SPECIAL_0 => {
            special_list(h, w2, 0);
            h.w8(w2 + W2_BP_SHOP, 1);
        }
        KIND_BADGES => {
            let money = h.fun_02012f08(gd);
            let badges = h.fun_0200c258(money);
            mart_list(h, w2, t, b, badges);
        }
        _ => mart_list(h, w2, t, b, 0),
    }
    let bp = u32::from(h.r8(w2 + W2_BP_SHOP));
    h.fun_021b331c(w2, bp);
    h.fun_021b3bd8(w2);
    h.fun_021b3fa0(w2);
    h.fun_021b3f00(w2);
    h.fun_021b40d4(w2);
    h.fun_021b43ec(w2);
}

/// C: `FUN_overlay_d_21__021b3350`: the currency of the shop: BP (`FUN_0200DABC(W2+0x18)`) when W2+0x163 is set, else the
/// money (`FUN_0200C274(W2+0xC)`).
pub fn currency<H: ShopHooks>(h: &mut H, w2: u32) -> u32 {
    if h.r8(w2 + W2_BP_SHOP) == 0 {
        let m = h.r32(w2 + W2_MONEY);
        h.fun_0200c274(m)
    } else {
        let b = h.r32(w2 + W2_BP);
        h.fun_0200dabc(b)
    }
}

/// C: `FUN_overlay_d_21__021b3370`: buy_check(W2): the chosen item (W2+0x1FC) does not fit in the bag -> message 0xA, state
/// {0xA, 4}; the [`currency`] is below the price (W2+0x200) -> message W2[0x163] ? 0x10 : 3, {0xA, 4}; below twice the price
/// -> the count prompt `FUN_021B4804`, {0xA, 5}; W2[0x162] -> the count prompt, {0xA, 6}; else message 4, {0xA, 2} and the
/// purchase maximum W2+0x204 = min(money / price, 99), cut to `FUN_0200823C(item)` minus what the bag holds.
/// A price of 0: `_u32_div_f` returns the dividend, so the maximum is min(money, 99) (V-0029).
pub fn buy_check<H: ShopHooks>(h: &mut H, w2: u32) {
    let money = currency(h, w2);
    let item = u32::from(h.r16(w2 + W2_ITEM));
    let heap = u32::from(h.r16(w2 + W2_HEAP));
    let bag = h.r32(w2 + W2_BAG);
    let have = h.fun_02007fb8(bag, item, heap);
    let item = u32::from(h.r16(w2 + W2_ITEM));
    let heap = u32::from(h.r16(w2 + W2_HEAP));
    let bag = h.r32(w2 + W2_BAG);
    if h.fun_02007e3c(bag, item, 1, heap) == 0 {
        let item = u32::from(h.r16(w2 + W2_ITEM));
        h.fun_021b4714(w2, 0xA, item, 0);
        h.w8(w2, 0xA);
        h.w8(w2 + 1, 4);
        return;
    }
    let price = h.r32(w2 + W2_PRICE);
    if money < price {
        let msg = if h.r8(w2 + W2_BP_SHOP) == 0 { 3 } else { 0x10 };
        let item = u32::from(h.r16(w2 + W2_ITEM));
        h.fun_021b4714(w2, msg, item, 0);
        h.w8(w2, 0xA);
        h.w8(w2 + 1, 4);
        return;
    }
    if money < price.wrapping_shl(1) {
        let flag = u32::from(h.r8(w2 + W2_FLAG));
        h.fun_021b4804(w2, flag);
        h.w8(w2, 0xA);
        h.w8(w2 + 1, 5);
        return;
    }
    let flag = u32::from(h.r8(w2 + W2_FLAG));
    if flag != 0 {
        h.fun_021b4804(w2, flag);
        h.w8(w2, 0xA);
        h.w8(w2 + 1, 6);
        return;
    }
    let item = u32::from(h.r16(w2 + W2_ITEM));
    h.fun_021b4714(w2, 4, item, 0);
    h.w8(w2, 0xA);
    h.w8(w2 + 1, 2);
    let price = h.r32(w2 + W2_PRICE);
    let n = u32_div(money, price);
    h.w32(w2 + W2_MAX, n);
    if n > 99 {
        h.w32(w2 + W2_MAX, 99);
    }
    let item = u32::from(h.r16(w2 + W2_ITEM));
    let most = h.fun_0200823c(item);
    if h.r32(w2 + W2_MAX).wrapping_add(have) > most {
        let item = u32::from(h.r16(w2 + W2_ITEM));
        let most = h.fun_0200823c(item);
        h.w32(w2 + W2_MAX, most.wrapping_sub(have));
    }
}

/// C: `FUN_overlay_d_21__021b3568`: pay `price`: money (record 0x16 += price) or BP (`(u16)price`, record 0x22 += price), then
/// record 0x15 (purchases) += 1.
pub fn pay<H: ShopHooks>(h: &mut H, w2: u32, price: u32) {
    if h.r8(w2 + W2_BP_SHOP) == 0 {
        let m = h.r32(w2 + W2_MONEY);
        h.fun_0200c29c(m, price);
        let r = h.r32(w2 + W2_RECORDS);
        h.fun_02008dac(r, REC_MONEY_SPENT, price);
    } else {
        let b = h.r32(w2 + W2_BP);
        h.fun_0200da98(b, price & 0xFFFF);
        let r = h.r32(w2 + W2_RECORDS);
        h.fun_02008dac(r, REC_BP_SPENT, price);
    }
    let r = h.r32(w2 + W2_RECORDS);
    h.fun_02008df0(r, REC_PURCHASES);
}

/// C: `FUN_overlay_d_21__021b3834`: list_flag(list, n): r = `FUN_02021288(list[0].item)`; `FUN_02021288(list[i].item)` for i < n
/// (results unused); returns r.
pub fn list_flag<H: ShopHooks>(h: &mut H, list: u32, n: u32) -> u32 {
    let item = u32::from(h.r16(list));
    let r = h.fun_02021288(item);
    let mut i: i32 = 0;
    while i < n as i32 {
        let item = u32::from(h.r16(list.wrapping_add(8 * i as u32)));
        h.fun_02021288(item);
        i += 1;
    }
    r
}

/// The message data the list rows are written with (`FUN_02045B38(0, 2, 0x36, heap)`).
fn open_msg<H: ShopHooks>(h: &mut H, w2: u32) -> u32 {
    let heap = u32::from(h.r16(w2 + W2_HEAP));
    h.fun_02045b38(0, 2, 0x36, heap)
}

/// The list-menu tail after `n` rows: `FUN_0201F878(W2[0x180] + 8 n, W2[0x188], 0xB, -2, heap)`.
fn rows_end<H: ShopHooks>(h: &mut H, w2: u32, n: u32) {
    let heap = u32::from(h.r16(w2 + W2_HEAP));
    let rows = h.r32(w2 + W2_ROWS);
    let a = h.r32(w2 + W2_ROWS_ARG);
    h.fun_0201f878(rows.wrapping_add(8 * n), a, 0xB, 0xFFFF_FFFE, heap);
}

/// C: `FUN_overlay_d_21__021b38e0`: mart_list(W2, kind, b, badges): list L = [`MART_LISTS`]`[kind == 0xFF ?
/// MART_BY_BADGES[badges] : b + 6]`; `FUN_02021288(L[0])` -> W2[0x162] = 1; the rows (`FUN_021B385C`), items and prices
/// (`FUN_0201102C(FUN_02020FE4(item, 0, heap))`) go to W2+0x20.., W2+0x160 = the count; [`list_flag`]; the list-menu tail.
/// The ROM tables are indexed without a bound check (Rust panics past them).
pub fn mart_list<H: ShopHooks>(h: &mut H, w2: u32, kind: u32, b: u32, badges: u32) {
    let heap = u32::from(h.r16(w2 + W2_HEAP));
    let msg = h.fun_02045b38(0, 2, 0x36, heap);
    let idx = if kind == KIND_BADGES {
        MART_BY_BADGES[badges as usize] as usize
    } else {
        b.wrapping_add(6) as usize
    };
    let list = MART_LISTS[idx];
    if h.fun_02021288(u32::from(list[0])) != 0 {
        h.w8(w2 + W2_FLAG, 1);
    }
    let n = list.len() as u32;
    let heap = u32::from(h.r16(w2 + W2_HEAP));
    let rows = h.fun_0201f81c(n + 1, heap);
    h.w32(w2 + W2_ROWS, rows);
    for (i, &item) in list.iter().enumerate() {
        let i = i as u32;
        let flag = u32::from(h.r8(w2 + W2_FLAG));
        h.fun_021b385c(w2, i, flag, u32::from(item), msg);
        let rec = w2 + W2_LIST + 8 * i;
        h.w16(rec, item);
        let heap = u32::from(h.r16(w2 + W2_HEAP));
        let data = h.fun_02020fe4(u32::from(item), 0, heap);
        let price = h.fun_0201102c(data);
        h.w32(rec + 4, price);
    }
    h.w16(w2 + W2_COUNT, n as u16);
    let c = u32::from(h.r16(w2 + W2_COUNT));
    list_flag(h, w2 + W2_LIST, c);
    rows_end(h, w2, n);
    h.fun_02045c04(msg);
}

/// C: `FUN_overlay_d_21__021b39c4`: special_list(W2, k): the rows, items and prices of [`SPECIAL_LISTS`]`[k]` go to
/// W2+0x20.., W2+0x160 = the count; the list-menu tail. (k is 0 or 1; Rust panics past the table.)
pub fn special_list<H: ShopHooks>(h: &mut H, w2: u32, k: u32) {
    let msg = open_msg(h, w2);
    let list = SPECIAL_LISTS[k as usize];
    let n = list.len() as u32;
    let heap = u32::from(h.r16(w2 + W2_HEAP));
    let rows = h.fun_0201f81c(n + 1, heap);
    h.w32(w2 + W2_ROWS, rows);
    for (i, &(item, price)) in list.iter().enumerate() {
        let i = i as u32;
        let flag = u32::from(h.r8(w2 + W2_FLAG));
        h.fun_021b385c(w2, i, flag, u32::from(item), msg);
        let rec = w2 + W2_LIST + 8 * i;
        h.w16(rec, item);
        h.w32(rec + 4, price);
    }
    h.w16(w2 + W2_COUNT, n as u16);
    rows_end(h, w2, n);
    h.fun_02045c04(msg);
}

/// C: `FUN_overlay_d_21__021b3a64`: dyn_list(W2, kind): `FUN_02186670(W2[0x1C], W2 + 0x20, &W2[0x160], kind - 0xF7, heap)`
/// fills the list; the rows (`FUN_021B385C`) of its W2[0x160] items; the list-menu tail.
pub fn dyn_list<H: ShopHooks>(h: &mut H, w2: u32, kind: u32) {
    let msg = open_msg(h, w2);
    let heap = u32::from(h.r16(w2 + W2_HEAP));
    let x = h.r32(w2 + W2_DYN);
    h.fun_02186670(
        x,
        w2 + W2_LIST,
        w2 + W2_COUNT,
        kind.wrapping_sub(KIND_DYN_FIRST),
        heap,
    );
    let n = u32::from(h.r16(w2 + W2_COUNT));
    let heap = u32::from(h.r16(w2 + W2_HEAP));
    let rows = h.fun_0201f81c(n + 1, heap);
    h.w32(w2 + W2_ROWS, rows);
    let mut i = 0;
    while i < u32::from(h.r16(w2 + W2_COUNT)) {
        let flag = u32::from(h.r8(w2 + W2_FLAG));
        let item = u32::from(h.r16(w2 + W2_LIST + 8 * i));
        h.fun_021b385c(w2, i, flag, item, msg);
        i += 1;
    }
    rows_end(h, w2, i);
    h.fun_02045c04(msg);
}

/// C: `FUN_overlay_d_21__021b4710`: a list-menu callback that does nothing: 0.
pub fn list_cb_nop() -> u32 {
    0
}

/// C: `FUN_overlay_d_21__021b49ec`: step the purchase count `*n` (s16) by the key `key`: -1 (0 wraps to `max`), +1 (past
/// `max` wraps to 1), -10 (floor 1), +10 (ceiling `max`). Returns 2 (went down) / 1 (went up) when `*n` changed, 0 when it
/// did not or for any other key.
pub fn count_step<H: crate::hooks::MemEnv>(h: &mut H, n: u32, max: u32, key: u32) -> u32 {
    let old = h.r16(n) as i16 as i32;
    let max = max as i32;
    let key = key as i32;
    let (new, up) = match key {
        -1 => {
            let v = (old - 1) as i16 as i32;
            (if v <= 0 { max } else { v }, false)
        }
        -10 => {
            let v = (old - 10) as i16 as i32;
            (if v <= 0 { 1 } else { v }, false)
        }
        1 => {
            let v = (old + 1) as i16 as i32;
            (if v > max { 1 } else { v }, true)
        }
        10 => {
            let v = (old + 10) as i16 as i32;
            (if v > max { max } else { v }, true)
        }
        _ => return 0,
    };
    h.w16(n, new as u16);
    let now = h.r16(n) as i16 as i32;
    if now == old {
        0
    } else if up {
        1
    } else {
        2
    }
}

/// Run the command handler at `handler` (bit 0 ignored) when it is one of [`COMMANDS`]; `None` otherwise.
pub fn dispatch<H: ShopHooks>(h: &mut H, handler: u32, vm: u32, ctx: u32) -> Option<u32> {
    match handler & !1 {
        0x021B_2FD8 => Some(cmd_329(h, vm, ctx)),
        _ => None,
    }
}

/// Run the wait callback `f` (Thumb bit set) when it is [`WAIT_329`]; `None` otherwise.
pub fn call_wait<H: ShopHooks>(h: &mut H, f: u32, vm: u32, ctx: u32) -> Option<u32> {
    (f == WAIT_329).then(|| wait_329(h, vm, ctx))
}

#[cfg(test)]
mod tests;
