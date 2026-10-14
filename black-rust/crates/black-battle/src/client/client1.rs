//! `btl_client.c` part 1 (0x021CD6EC-0x021CDDDC): client object lifecycle, accessors, the main step and the wait-for-request
//! step, the request-table lookup (B13-spec1 / B13-spec2; ported B13a).
//!
//! Specs: `tools/rom-diff/svflow_specs/client1.spec` (`nowork`, the client object is `arg buf 0x1C0`). The functions take the
//! client object as a byte slice `c` (0x1B8 bytes; the layout table is in [`super`]) and, where the original passes its address
//! on (to a callee or into a result pointer), its address `c_addr`.
//!
//! Callees: battle-view / sound calls go through [`ClientUi`] (D3 / D7), the adapter (B12, D7 seam) and every other unported
//! or other-area callee through [`FlowEnv::call`]; a step function or request handler held in a function-pointer field is
//! dispatched through `env.call(ptr & !1, ..)` (the original `blx`).
//!
//! Deviation (B13a): the two steps reach the request-table lookup [`request_handler`] (`021cdd50`) through `env.call`, not
//! directly, and their spec stubs of `021cdd50` carry `force`. The original indexes the table by the client mode with no
//! bound check; the step specs draw any mode byte (only 0..2 are real modes), and for a mode above 2 the original reads
//! the rodata after the table (the playback table, then `battle_net` data), which the port does not embed. The lookup is
//! verified by its own block (modes 0..2); the steps' blocks verify the call sites.

use super::{rd16, rd32, wr16, wr32, ClientUi};
use crate::server_flow::env::{Arg, FlowEnv};

/// Functions of this area that are ported and checked by `tests::svflow_match_rom`.
pub const PORTED: &[u32] = &[
    0x021C_D6EC,
    0x021C_D6F0,
    0x021C_D8E0,
    0x021C_D914,
    0x021C_D924,
    0x021C_D938,
    0x021C_D95C,
    0x021C_D960,
    0x021C_D978,
    0x021C_D97C,
    0x021C_D984,
    0x021C_D9B0,
    0x021C_D9C8,
    0x021C_D9DC,
    0x021C_D9E0,
    0x021C_D9F4,
    0x021C_DA0C,
    0x021C_DA18,
    0x021C_DC0C,
    0x021C_DD34,
    0x021C_DD50,
    0x021C_DD94,
    0x021C_DDA0,
    0x021C_DDCC,
];

const OFS_SERVER: usize = 0x00;
const OFS_POKECON: usize = 0x04;
const OFS_RECORD: usize = 0x10;
const OFS_READER: usize = 0x14;
const OFS_STEP: usize = 0x28;
const OFS_REQ_PAIR: usize = 0x2C;
const OFS_VIEW_OBJ: usize = 0x34;
const OFS_ADAPTER: usize = 0x50;
const OFS_VIEW: usize = 0x54;
const OFS_A0: usize = 0xA0;
const OFS_C4: usize = 0xC4;
const OFS_HANDLER: usize = 0xC8;
const OFS_HANDLER_STATE: usize = 0xCC;
const OFS_RESULT_PTR: usize = 0xD8;
const OFS_RESULT_SIZE: usize = 0xDC;
const OFS_RESULT_WORD: usize = 0xE0;
const OFS_BGM: usize = 0xE4;
const OFS_SE: usize = 0xE6;
const OFS_AI: usize = 0xF0;
const OFS_AI_BLOCK: usize = 0xF4;
const OFS_112: usize = 0x112;
const OFS_PARTY: usize = 0x118;
const OFS_11C: usize = 0x11C;
const OFS_11D: usize = 0x11D;
const OFS_121: usize = 0x121;
const OFS_HEAP_BLOCK: usize = 0x134;
const OFS_HEAP_ID: usize = 0x19A;
const OFS_HP_REF: usize = 0x19C;
const OFS_ID: usize = 0x1A6;
const OFS_MODE: usize = 0x1A7;
const OFS_PHASE: usize = 0x1A8;
const OFS_1A9: usize = 0x1A9;
const OFS_1AA: usize = 0x1AA;
const OFS_COUNTER: usize = 0x1AB;
const OFS_1AD: usize = 0x1AD;
const OFS_1AE: usize = 0x1AE;
const OFS_FLAGS: usize = 0x1B2;

/// Size of the client object (`Heap_AllocDebug(heap, 0x1B8, ..)` in 021cd6f0).
pub const CLIENT_SIZE: u32 = 0x1B8;

/// The client mode at `+0x1A7` that replays a recording.
const MODE_REPLAY: u8 = 2;

/// Thumb pointer of the wait-for-request step [`wait_step`] (`0x021CDC0D`).
pub const STEP_WAIT: u32 = 0x021C_DC0D;
/// Thumb pointer of the main step [`main_step`] (`0x021CDA19`).
pub const STEP_MAIN: u32 = 0x021C_DA19;

/// Flag bits of `+0x1B2`.
const FLAG_1: u8 = 1;
const FLAG_DONE: u8 = 2;
const FLAG_8: u8 = 8;
const FLAG_10: u8 = 0x10;
const FLAG_20: u8 = 0x20;

/// Address of the source-file name string "btl_client.c" (`Heap_AllocDebug` debug argument).
const SRC_FILE: u32 = 0x021F_3A88;

/// The request table at `0x021EFFEC` (ROM data): 15 records `{u8 code, u8 aux, 2 pad, fn mode0, fn mode1, fn mode2}`, as
/// little-endian words (word 0 = code | aux << 8). The handlers are Thumb pointers to the request step functions.
const REQUEST_TABLE: [[u32; 4]; 15] = [
    [0x0000_0101, 0x021C_DDDD, 0x0000_0000, 0x021C_DE8D],
    [0x0000_0002, 0x021C_DF59, 0x021C_E045, 0x021C_E041],
    [0x0000_0003, 0x021C_E171, 0x021D_0771, 0x021C_E1B1],
    [0x0000_0007, 0x021D_0CC5, 0x0000_0000, 0x0000_0000],
    [0x0000_0005, 0x021D_0D39, 0x021D_0D51, 0x021D_0E75],
    [0x0000_0006, 0x021D_0D45, 0x021D_0D51, 0x021D_0E75],
    [0x0000_0004, 0x021D_10B9, 0x0000_0000, 0x021D_0E75],
    [0x0000_0008, 0x021D_1A75, 0x0000_0000, 0x021D_1A61],
    [0x0000_000A, 0x021D_123D, 0x0000_0000, 0x0000_0000],
    [0x0000_010B, 0x021D_1411, 0x0000_0000, 0x021D_13FD],
    [0x0000_010C, 0x021D_171D, 0x0000_0000, 0x021D_1709],
    [0x0000_010E, 0x021D_18ED, 0x0000_0000, 0x0000_0000],
    [0x0000_010F, 0x021D_1961, 0x0000_0000, 0x0000_0000],
    [0x0000_010D, 0x021D_1291, 0x0000_0000, 0x021D_127D],
    [0x0000_0110, 0x021D_1A19, 0x0000_0000, 0x0000_0000],
];

/// The arguments of [`create`] after the first four registers (the stack words of the original call).
pub struct CreateArgs {
    /// `[sp]`: client id (stored as a byte at `+0x1A6`, passed whole to 021b9af8).
    pub id: u32,
    /// `[sp+4]`: low byte stored at `+0x11C`.
    pub b11c: u32,
    /// `[sp+8]`: client mode (byte at `+0x1A7`).
    pub mode: u32,
    /// `[sp+0xC]`: byte at `+0x1AA`.
    pub b1aa: u32,
    /// `[sp+0x10]`: nonzero = no trainer AI for a mode-1 client.
    pub no_ai: u32,
    /// `[sp+0x18]`: heap id (`[sp+0x14]` is the 0x18-byte block copied to `+0xF4`, the `block` argument of [`create`]).
    pub heap: u32,
}

/// C: `FUN_overlay_d_93__021cd6ec`: empty function (`bx lr`).
pub fn nop() {}

/// C: `FUN_overlay_d_93__021cd6f0`: creates a client. `c` is the memory the allocator returns for
/// `Heap_AllocDebug(heap, 0x1B8, 1, "btl_client.c", 0x209)`; the call's result (its address) is also the return value.
/// Initialises every field: adapter channel `021eea58(r3, id, r2 != 0 && mode == 0, heap)` (the spec comment says `mode != 0`; the code tests `== 0`), party container
/// `021b9af8(pokecon, id)`, a 0xBC0-byte heap block at `+0x134`, view-side object `021ba08c(server)`, the request pair
/// (`021d8230`), the command cursor (`021d3f04`), `021d3d9c(c)` (client5), the 0x18-byte block at `+0xF4`, the trainer AI
/// (`02187f00`, overlay 96) of a mode-1 client without `no_ai`, four zero bytes at `+0x112` (the spec comment says 0 1 2 3; the loop stores a constant 0), and a `BattleRecord`
/// (`021ee5e4`) for a mode-0 client when `021b90f8(server)` says so. Returns the object's address.
#[allow(clippy::too_many_arguments)]
pub fn create<E: FlowEnv>(
    env: &mut E,
    server: u32,
    pokecon: u32,
    r2: u32,
    r3: u32,
    a: &CreateArgs,
    block: &[u8],
    c: &mut [u8],
) -> u32 {
    let heap = a.heap;
    let c_addr = env.callh(
        0x0203_0734,
        &mut [
            Arg::V(heap),
            Arg::V(CLIENT_SIZE),
            Arg::V(1),
            Arg::V(SRC_FILE),
            Arg::V(0x209),
        ],
        c,
    );
    let flag = (r2 != 0 && a.mode as u8 == 0) as u32;
    c[OFS_ID] = a.id as u8;
    wr16(c, OFS_HEAP_ID, heap as u16);
    c[OFS_MODE] = a.mode as u8;
    let adapter = env.callh(
        0x021E_EA58,
        &mut [Arg::V(r3), Arg::V(a.id & 0xFF), Arg::V(flag), Arg::V(heap)],
        c,
    );
    wr32(c, OFS_ADAPTER, adapter);
    let party = env.callh(0x021B_9AF8, &mut [Arg::V(pokecon), Arg::V(a.id)], c);
    wr32(c, OFS_PARTY, party);
    wr32(c, OFS_SERVER, server);
    wr32(c, OFS_POKECON, pokecon);
    c[OFS_11C] = a.b11c as u8;
    c[OFS_11D] = 0;
    c[OFS_121] = a.id as u8;
    c[OFS_C4] = 0;
    wr32(c, OFS_VIEW, 0);
    wr16(c, OFS_HP_REF, 0);
    let blk = env.callh(
        0x0203_0734,
        &mut [
            Arg::V(heap),
            Arg::V(0xBC0),
            Arg::V(1),
            Arg::V(SRC_FILE),
            Arg::V(0x21A),
        ],
        c,
    );
    wr32(c, OFS_HEAP_BLOCK, blk);
    wr32(c, OFS_STEP, STEP_MAIN);
    c[OFS_PHASE] = 0;
    wr32(c, OFS_A0, 0);
    c[OFS_FLAGS] &= !FLAG_8;
    c[OFS_1AE] = 0;
    c[OFS_1A9] = 0;
    c[OFS_COUNTER] = 0;
    wr16(c, OFS_BGM, 0);
    wr16(c, OFS_SE, 0);
    c[OFS_1AD] = 0;
    c[OFS_FLAGS] &= !FLAG_1;
    c[OFS_FLAGS] &= !FLAG_DONE;
    c[OFS_FLAGS] &= !FLAG_10;
    c[OFS_FLAGS] &= !FLAG_20;
    let view_obj = env.callh(0x021B_A08C, &mut [Arg::V(server)], c);
    wr32(c, OFS_VIEW_OBJ, view_obj);
    c[OFS_1AA] = a.b1aa as u8;
    env.call(0x021D_8230, &mut [Arg::PBuf(&mut c[0x2C..0x34])]);
    env.call(0x021D_3F04, &mut [Arg::PBuf(&mut c[0x18..0x28])]);
    env.callh(0x021D_3D9C, &mut [Arg::PHost(c.as_ptr() as usize)], c);
    c[OFS_AI_BLOCK..OFS_AI_BLOCK + 0x18].copy_from_slice(&block[..0x18]);
    let ai = if c[OFS_MODE] == 1 && a.no_ai == 0 {
        let srv = rd32(c, OFS_SERVER);
        let r6 = env.callh(0x021B_9FBC, &mut [Arg::V(srv), Arg::V(c[OFS_ID] as u32)], c);
        let r1 = env.callh(0x021B_A820, &mut [Arg::V(srv)], c);
        let heap_id = rd16(c, OFS_HEAP_ID) as u32;
        env.callh(
            0x0218_7F00,
            &mut [
                Arg::V(srv),
                Arg::V(r1),
                Arg::V(rd32(c, OFS_POKECON)),
                Arg::V(r6),
                Arg::V(heap_id),
            ],
            c,
        )
    } else {
        0
    };
    wr32(c, OFS_AI, ai);
    c[OFS_112..OFS_112 + 4].fill(0);
    let record =
        if c[OFS_MODE] == 0 && env.callh(0x021B_90F8, &mut [Arg::V(rd32(c, OFS_SERVER))], c) != 0 {
            env.callh(0x021E_E5E4, &mut [Arg::V(heap)], c)
        } else {
            0
        };
    wr32(c, OFS_RECORD, record);
    c_addr
}

/// C: `FUN_overlay_d_93__021cd8e0`: frees a client: `BattleRecord_Free(record)` if there is one, `02188080()` (overlay 96,
/// the trainer AI) if `+0xF0` is set, `Heap_Free` of the block at `+0x134`, of the adapter channel (thunk 021eeae0) and of the
/// object (`c_addr`).
pub fn free<E: FlowEnv>(env: &mut E, c: &[u8], c_addr: u32) {
    let record = rd32(c, OFS_RECORD);
    if record != 0 {
        env.callr(0x021E_E600, &mut [Arg::V(record)], c);
    }
    if rd32(c, OFS_AI) != 0 {
        // ai_delete(vm): the original passes the VM in r0 from an earlier load; the spec stub
        // takes no arguments (B16: the integration reads it from the lent client object)
        env.callr(0x0218_8080, &mut [], c);
    }
    env.callr(0x0203_07B0, &mut [Arg::V(rd32(c, OFS_HEAP_BLOCK))], c);
    env.callr(0x0203_07B0, &mut [Arg::V(rd32(c, OFS_ADAPTER))], c);
    env.callr(0x0203_07B0, &mut [Arg::V(c_addr)], c);
}

/// C: `FUN_overlay_d_93__021cd914`: sets flag bit 2 ("request done").
pub fn set_done(c: &mut [u8]) {
    c[OFS_FLAGS] |= FLAG_DONE;
}

/// C: `FUN_overlay_d_93__021cd924`: the recorded data (`BattleRecord_GetData(record)`), 0 when not recording.
pub fn record_data<E: FlowEnv>(env: &mut E, c: &[u8]) -> u32 {
    let record = rd32(c, OFS_RECORD);
    if record == 0 {
        return 0;
    }
    env.callr(0x021E_E63C, &mut [Arg::V(record)], c)
}

/// C: `FUN_overlay_d_93__021cd938`: replays from `reader`: mode 2, reader at `+0x14`, cursor init (`021d3f48`) with the
/// size `021ee830(reader)` reports.
pub fn set_replay<E: FlowEnv>(env: &mut E, c: &mut [u8], reader: u32) {
    c[OFS_MODE] = MODE_REPLAY;
    wr32(c, OFS_READER, reader);
    let size = env.callh(0x021E_E830, &mut [Arg::V(reader)], c);
    env.call(
        0x021D_3F48,
        &mut [Arg::PBuf(&mut c[0x18..0x28]), Arg::V(size)],
    );
}

/// C: `FUN_overlay_d_93__021cd95c`: sets the view handle (`+0x54`).
pub fn set_view(c: &mut [u8], view: u32) {
    wr32(c, OFS_VIEW, view);
}

/// C: `FUN_overlay_d_93__021cd960`: `+0xA0` = `v`, flag bit 8.
pub fn set_a0(c: &mut [u8], v: u32) {
    wr32(c, OFS_A0, v);
    c[OFS_FLAGS] |= FLAG_8;
}

/// C: `FUN_overlay_d_93__021cd978`: the adapter channel object (`+0x50`).
pub fn adapter(c: &[u8]) -> u32 {
    rd32(c, OFS_ADAPTER)
}

/// C: `FUN_overlay_d_93__021cd97c`: runs the current step function `(c->step)(c, c->step)` and returns its answer.
pub fn run_step<E: FlowEnv>(env: &mut E, c: &mut [u8]) -> u32 {
    let step = rd32(c, OFS_STEP);
    let ca = c.as_ptr() as usize;
    env.call_held(c, step & !1, &mut [Arg::PHost(ca), Arg::V(step)])
}

/// C: `FUN_overlay_d_93__021cd984`: restarts the replay: reader reset (`021ee674`), adapter block reset (`021eebdc`),
/// cursor reset with `n` (`021d3f64`), step = the wait-for-request step.
pub fn replay_restart<E: FlowEnv>(env: &mut E, c: &mut [u8], n: u32) {
    env.callh(0x021E_E674, &mut [Arg::V(rd32(c, OFS_READER))], c);
    env.callh(0x021E_EBDC, &mut [Arg::V(rd32(c, OFS_ADAPTER))], c);
    env.call(0x021D_3F64, &mut [Arg::PBuf(&mut c[0x18..0x28]), Arg::V(n)]);
    set_step(c, STEP_WAIT);
}

/// C: `FUN_overlay_d_93__021cd9b0`: cursor reset (`021d3f88`), step = the main step.
pub fn restart_main<E: FlowEnv>(env: &mut E, c: &mut [u8]) {
    env.call(0x021D_3F88, &mut [Arg::PBuf(&mut c[0x18..0x28])]);
    set_step(c, STEP_MAIN);
}

/// C: `FUN_overlay_d_93__021cd9c8`: is the client replaying (mode 2)?
pub fn is_replay(c: &[u8]) -> bool {
    c[OFS_MODE] == MODE_REPLAY
}

/// C: `FUN_overlay_d_93__021cd9dc`: halfword at `+0x22` (in the command cursor block).
pub fn cursor_u16(c: &[u8]) -> u16 {
    rd16(c, 0x22)
}

/// C: `FUN_overlay_d_93__021cd9e0`: is the current step the wait-for-request step?
pub fn is_waiting(c: &[u8]) -> bool {
    rd32(c, OFS_STEP) == STEP_WAIT
}

/// C: `FUN_overlay_d_93__021cd9f4`: counter byte at `+0x1AB` += `n` (stored as a byte), clamped to 14.
pub fn counter_add(c: &mut [u8], n: u32) {
    let v = c[OFS_COUNTER].wrapping_add(n as u8);
    c[OFS_COUNTER] = if v > 0xE { 0xE } else { v };
}

/// C: `FUN_overlay_d_93__021cda0c`: step = `step` (a Thumb pointer), phase = 0.
pub fn set_step(c: &mut [u8], step: u32) {
    wr32(c, OFS_STEP, step);
    c[OFS_PHASE] = 0;
}

/// Answers the request handler at `+0xC8` with the result block (`021eec4c(adapter, result ptr, result size)`).
fn post_result<E: FlowEnv>(env: &mut E, c: &[u8]) -> u32 {
    env.callr(
        0x021E_EC4C,
        &mut [
            Arg::V(rd32(c, OFS_ADAPTER)),
            Arg::V(rd32(c, OFS_RESULT_PTR)),
            Arg::V(rd32(c, OFS_RESULT_SIZE)),
        ],
        c,
    )
}

/// Runs the request handler at `+0xC8`: `(c->handler)(c, &c->handler_state)`.
fn run_handler<E: FlowEnv>(env: &mut E, c: &mut [u8]) -> u32 {
    let h = rd32(c, OFS_HANDLER);
    let state = c[OFS_HANDLER_STATE..].as_ptr() as usize;
    let ca = c.as_ptr() as usize;
    env.call_held(c, h & !1, &mut [Arg::PHost(ca), Arg::PHost(state)])
}

/// C: `sub_021cda18`: the main step (an 8-way jump table on the phase `+0x1A8`); `c_addr` is the object's address
/// (the result pointer and the adapter data pointer are addresses). Returns 1 when the client is done (flag bit 2 or
/// phase 7), else 0. `021d3fdc(c, cursor)` runs first.
///
/// * 0: request code = `021eebe8(adapter)`. A replaying client (mode 2) with flag bit 0x10 or 0x20 and no code answers 0.
///   Result block cleared ([`result_clear`]). Code 9: the request pair `+0x2C/+0x30` = the two words of the adapter data
///   (`021eec20`), handler state 0, phase 3. Other codes: handler `+0xC8` = `021cdd50(c, code, &aux)` ([`request_handler`], called through `env.call`, see the module doc); with
///   `aux`, `021d3fac(cursor) == 0` -> phase 6, else on; without, `021d3fc4(cursor)`; then handler state 0 and phase 1
///   (a handler) or 2 (none). No code: answer 1 when flag bit 2 is set.
/// * 1: the handler; when it is done: flag bit 2 -> 1, else phase `021d3f4c(cursor) ? 4 : 2`.
/// * 2 / 3: post the result (`021eec4c`); posted -> phase 0 / 7; then flag bit 2 -> 1.
/// * 4: `021d3f60(cursor) == 1` -> phase 7, else (with a view: sound `02005ca4(1, 0x3E)`, `02006740()`) phase 5.
/// * 5: once `02005e50()` is 0: `n = 021d3fa8(cursor)`, `02005ca4(0, 1)`, `021ba73c(server, n)`, `02006734(0)`.
/// * 6: `021d3f4c(cursor)` -> phase 4.   7: 1.   Other phases: 0.
pub fn main_step<E: ClientUi>(env: &mut E, c: &mut [u8], c_addr: u32) -> u32 {
    let cursor = c[0x18..].as_ptr() as usize;
    let ca = c.as_ptr() as usize;
    env.call_held(c, 0x021D_3FDC, &mut [Arg::PHost(ca), Arg::PHost(cursor)]);
    let done = |c: &[u8]| (c[OFS_FLAGS] & FLAG_DONE != 0) as u32;
    match c[OFS_PHASE] {
        0 => {
            let code = env.callh(0x021E_EBE8, &mut [Arg::V(rd32(c, OFS_ADAPTER))], c);
            if c[OFS_MODE] == MODE_REPLAY && c[OFS_FLAGS] & (FLAG_10 | FLAG_20) != 0 && code == 0 {
                return 0;
            }
            result_clear(c, c_addr);
            if code == 9 {
                let mut p = [0u8; 4];
                env.callh(
                    0x021E_EC20,
                    &mut [Arg::V(rd32(c, OFS_ADAPTER)), Arg::Out(&mut p)],
                    c,
                );
                // the adapter data lies in the block given to the client (`c`; the spec places it at `+0x1B8`);
                // in the game it is the adapter's channel object (B16: read through `peek`)
                let mut ext = [0u8; 8];
                let (w0, w1) = if env.peek(u32::from_le_bytes(p), &mut ext) {
                    (rd32(&ext, 0), rd32(&ext, 4))
                } else {
                    let data = u32::from_le_bytes(p).wrapping_sub(c_addr) as usize;
                    (rd32(c, data), rd32(c, data + 4))
                };
                wr32(c, OFS_REQ_PAIR + 4, w1);
                wr32(c, OFS_REQ_PAIR, w0);
                wr32(c, OFS_HANDLER_STATE, 0);
                c[OFS_PHASE] = 3;
                return 0;
            }
            if code == 0 {
                return done(c);
            }
            let mut aux = [0u8; 4];
            let h = env.callh(
                0x021C_DD50,
                &mut [
                    Arg::PHost(c.as_ptr() as usize),
                    Arg::V(code),
                    Arg::Out(&mut aux),
                ],
                c,
            );
            wr32(c, OFS_HANDLER, h);
            if u32::from_le_bytes(aux) != 0 {
                if env.call(0x021D_3FAC, &mut [Arg::PBuf(&mut c[0x18..0x28])]) == 0 {
                    c[OFS_PHASE] = 6;
                    return 0;
                }
            } else {
                env.call(0x021D_3FC4, &mut [Arg::PBuf(&mut c[0x18..0x28])]);
            }
            wr32(c, OFS_HANDLER_STATE, 0);
            c[OFS_PHASE] = if rd32(c, OFS_HANDLER) != 0 { 1 } else { 2 };
            0
        }
        1 => {
            if run_handler(env, c) == 0 {
                return 0;
            }
            if c[OFS_FLAGS] & FLAG_DONE != 0 {
                return 1;
            }
            c[OFS_PHASE] = if env.call(0x021D_3F4C, &mut [Arg::PBuf(&mut c[0x18..0x28])]) != 0 {
                4
            } else {
                2
            };
            0
        }
        2 => {
            if post_result(env, c) != 0 {
                c[OFS_PHASE] = 0;
            }
            done(c)
        }
        3 => {
            if post_result(env, c) != 0 {
                c[OFS_PHASE] = 7;
            }
            done(c)
        }
        4 => {
            if env.call(0x021D_3F60, &mut [Arg::PBuf(&mut c[0x18..0x28])]) == 1 {
                c[OFS_PHASE] = 7;
                return 0;
            }
            if rd32(c, OFS_VIEW) != 0 {
                env.sound(0x0200_5CA4, &mut [Arg::V(1), Arg::V(0x3E)]);
                env.sound(0x0200_6740, &mut []);
            }
            c[OFS_PHASE] = 5;
            0
        }
        5 => {
            if env.sound(0x0200_5E50, &mut []) == 0 {
                let n = env.call(0x021D_3FA8, &mut [Arg::PBuf(&mut c[0x18..0x28])]);
                env.sound(0x0200_5CA4, &mut [Arg::V(0), Arg::V(1)]);
                env.callh(
                    0x021B_A73C,
                    &mut [Arg::V(rd32(c, OFS_SERVER)), Arg::V(n)],
                    c,
                );
                env.sound(0x0200_6734, &mut [Arg::V(0)]);
            }
            0
        }
        6 => {
            if env.call(0x021D_3F4C, &mut [Arg::PBuf(&mut c[0x18..0x28])]) != 0 {
                c[OFS_PHASE] = 4;
            }
            0
        }
        7 => 1,
        _ => 0,
    }
}

/// C: `sub_021cdc0c`: the wait-for-request step (a 6-way jump table on the phase `+0x1A8`); returns 1 in phase 5, else 0.
/// `021d3fdc(c, cursor)` runs first.
///
/// * 0: phase 1, then as 1.
/// * 1: `021d3f98(cursor)` set: with a view, `02005ca4(1, 1)`, `021eae24(view)`, `02005df8(0x1E)`, `021f88bc(0)`,
///   `02006740()`; phase 4. Else code = `021eebe8(adapter)`; with a code, handler `+0xC8` = `021cdd50(c, code, 0)` ([`request_handler`], through `env.call`):
///   a handler -> phase 2 (state 0), none -> result cleared, phase 3.
/// * 2: the handler; done -> phase 3, state 0, and `021f88bc(1)` when `021f8678()`.
/// * 3: post the result (`021eec4c`) -> phase 1.
/// * 4: with a view, `021eae34(view)` and `02005e50() == 0`: `n = (u16) 021d3fa8(cursor)`, `021eae74(view, n, n)`,
///   `021ba7f8(server)`.
pub fn wait_step<E: ClientUi>(env: &mut E, c: &mut [u8], c_addr: u32) -> u32 {
    let cursor = c[0x18..].as_ptr() as usize;
    let ca = c.as_ptr() as usize;
    env.call_held(c, 0x021D_3FDC, &mut [Arg::PHost(ca), Arg::PHost(cursor)]);
    match c[OFS_PHASE] {
        0 | 1 => {
            c[OFS_PHASE] = 1;
            if env.call(0x021D_3F98, &mut [Arg::PBuf(&mut c[0x18..0x28])]) != 0 {
                let view = rd32(c, OFS_VIEW);
                if view != 0 {
                    env.sound(0x0200_5CA4, &mut [Arg::V(1), Arg::V(1)]);
                    env.view(0x021E_AE24, &mut [Arg::V(view)]);
                    env.sound(0x0200_5DF8, &mut [Arg::V(0x1E)]);
                    env.sound(0x021F_88BC, &mut [Arg::V(0)]);
                    env.sound(0x0200_6740, &mut []);
                }
                c[OFS_PHASE] = 4;
                return 0;
            }
            let code = env.callh(0x021E_EBE8, &mut [Arg::V(rd32(c, OFS_ADAPTER))], c);
            if code == 0 {
                return 0;
            }
            let h = env.callh(
                0x021C_DD50,
                &mut [Arg::PHost(c.as_ptr() as usize), Arg::V(code), Arg::V(0)],
                c,
            );
            wr32(c, OFS_HANDLER, h);
            if h != 0 {
                c[OFS_PHASE] = 2;
            } else {
                result_clear(c, c_addr);
                c[OFS_PHASE] = 3;
            }
            wr32(c, OFS_HANDLER_STATE, 0);
            0
        }
        2 => {
            if run_handler(env, c) != 0 {
                c[OFS_PHASE] = 3;
                wr32(c, OFS_HANDLER_STATE, 0);
                if env.sound(0x021F_8678, &mut []) != 0 {
                    env.sound(0x021F_88BC, &mut [Arg::V(1)]);
                }
            }
            0
        }
        3 => {
            if post_result(env, c) != 0 {
                c[OFS_PHASE] = 1;
            }
            0
        }
        4 => {
            let view = rd32(c, OFS_VIEW);
            if view != 0
                && env.view(0x021E_AE34, &mut [Arg::V(view)]) != 0
                && env.sound(0x0200_5E50, &mut []) == 0
            {
                let n = env.call(0x021D_3FA8, &mut [Arg::PBuf(&mut c[0x18..0x28])]) & 0xFFFF;
                env.view(
                    0x021E_AE74,
                    &mut [Arg::V(rd32(c, OFS_VIEW)), Arg::V(n), Arg::V(n)],
                );
                env.callh(0x021B_A7F8, &mut [Arg::V(rd32(c, OFS_SERVER))], c);
            }
            0
        }
        5 => 1,
        _ => 0,
    }
}

/// C: `FUN_overlay_d_93__021cdd34`: result block = one zeroed word at `+0xE0` (`c_addr` is the address of the object):
/// result pointer `+0xD8` = `&c[0xE0]`, result size `+0xDC` = 4.
pub fn result_clear(c: &mut [u8], c_addr: u32) {
    wr32(c, OFS_RESULT_WORD, 0);
    wr32(
        c,
        OFS_RESULT_PTR,
        c_addr.wrapping_add(OFS_RESULT_WORD as u32),
    );
    wr32(c, OFS_RESULT_SIZE, 4);
}

/// C: `FUN_overlay_d_93__021cdd50`: the request-table handler for request `code` in the client's mode (`+0x1A7`, 0..2);
/// `aux` (if given) receives the record's aux byte. 0 when no record has the code (`aux` untouched).
pub fn request_handler(c: &[u8], code: u32, aux: Option<&mut u32>) -> u32 {
    for rec in REQUEST_TABLE.iter() {
        if code == rec[0] & 0xFF {
            if let Some(a) = aux {
                *a = (rec[0] >> 8) & 0xFF;
            }
            return rec[1 + c[OFS_MODE] as usize];
        }
    }
    0
}

/// C: `FUN_overlay_d_93__021cdd94`: the request argument pair (`+0x2C`, `+0x30`) into `out`.
pub fn request_pair(c: &[u8], out: &mut [u8]) {
    wr32(out, 0, rd32(c, OFS_REQ_PAIR));
    wr32(out, 4, rd32(c, OFS_REQ_PAIR + 4));
}

/// C: `FUN_overlay_d_93__021cdda0`: replaying: flag bit 0x10; else, with a sound effect id at `+0xE6`, `021f8524(0)`; else 0.
pub fn effect_pending<E: ClientUi>(env: &mut E, c: &[u8]) -> u32 {
    if c[OFS_MODE] == MODE_REPLAY {
        return ((c[OFS_FLAGS] >> 4) & 1) as u32;
    }
    if rd16(c, OFS_SE) != 0 {
        return env.sound(0x021F_8524, &mut [Arg::V(0)]);
    }
    0
}

/// C: `FUN_overlay_d_93__021cddcc`: flag bit 0x20.
pub fn flag_20(c: &[u8]) -> bool {
    c[OFS_FLAGS] & FLAG_20 != 0
}

#[cfg(test)]
mod tests;
