//! Server <-> client adapter (`btl_adapter.c`, asm/battle_adapter.s, 0x021EEA30-0x021EF71C; B12).
//!
//! The file holds two unrelated halves (the filename string `btl_adapter.c` at 0x021F3AE8 belongs to
//! the first):
//!
//! * `0x021EEA30-0x021EEC76`: the **adapter proper**, a four-slot table of per-client channel objects
//!   (`0x021F6388[4]`, flag word `0x021F6384`) and the functions the server (`server_side`, 0x021BAA94) and
//!   the client (`btl_client.c`, 0x021CD6F0) call on them.
//! * `0x021EEC78-0x021EF71C`: the **battle message printer** that the UI half of overlay 93 (0x021E9034-0x021EE200)
//!   calls: a context at `0x021F6398` (`ctx`), `eec78` init / `eecd8` free, the word-set helpers
//!   `eed00..eed90`, the tag expander `eefe0` and the message-id dispatch `eeef4` with its handlers. It
//!   belongs to the battle UI (screens), not to the logic seam, and is only specced here.
//!
//! Specs: `tools/rom-diff/svflow_specs/adapter.spec` (45 functions; `nowork`: the objects are `arg buf`s and
//! the globals `global 021f6384 0x14` / `global 021f6398 0x60`). Callees outside the area go through
//! [`FlowEnv`]; the comm layer (`btl_comm`, 0x021D4xxx) is stubbed.
//!
//! # Adapter object layout (`0xBD0` bytes, allocated by `eea58` with `Heap_AllocDebug`)
//!
//! | ofs | size | meaning |
//! |---|---|---|
//! | `+0x00` | u32 | owner (first argument of `eea58`: the server or client state) |
//! | `+0x04` | u32 | pointer to the **reply** block (local: posted by the client with `eec4c`; networked: set by `eeba4`) |
//! | `+0x08` | u32 | size of the reply block |
//! | `+0x0C` | u8 | low nibble = reply state (0 empty, 1 filled); high nibble = **transport** (0 local, else networked through `btl_comm`) |
//! | `+0x0D` | u8 | client id (the comm slot for the networked transport) |
//! | `+0x0E` | u8 | send phase: 0 idle, 1 queued, 3 sent, 4 done |
//! | `+0x0F` | u8 | message code (low byte) |
//! | `+0x10` | u16 | message code |
//! | `+0x12` | u16 | message payload size |
//! | `+0x14` | u8[] | message payload (copied by `eeb00`; the local reader gets a pointer to it from `eec20`) |
//!
//! # Proposed command-stream type (D7)
//!
//! The original already is a two-layer stream, and the Rust seam should keep both layers:
//!
//! 1. **Command frames** (server -> client, the payload of every message): the bytes of the server command
//!    queue ([`super::cmd_queue`]: `{u32 write_count; u32 read_cursor; u8 data[..0xBB8]}`, records written by
//!    `put_cmd*`, decoded by `read_cmd` with the `CMD_ARG_TABLE` argument formats). The Rust type is the
//!    decoded record, not the bytes:
//!    `struct BtlCmd { id: u16, args: smallvec<[u32; 8]> }` (ids 0..0x60, the table in `cmd_queue`), plus a
//!    typed `enum BtlEvent` the UI matches on once `btl_client` (B13) knows what each id means. Frames never
//!    contain UI state; the client owns the playback cursor (`read_cursor`).
//! 2. **Adapter messages** (the envelope; one per server state step):
//!    * `ServerToClient { code: u16, payload: Frame | Bytes }` = `eeb00(obj, code, data, size)`; `code` is the
//!      server state's request ("run these commands", "choose an action", ...; the values are in the
//!      `server_side` state functions that call `021bb724` / `021bb768`), `payload` at most `0xBBC` bytes.
//!      `broadcast` = `021bb724` (every used slot, bracketed by `eeae8` / `eeafc`), `send_one` = `021bb768`.
//!    * `ClientToServer { ptr, size }` = `eec4c(obj, ptr, size)` (the reply block is posted by reference,
//!      never copied in the local transport) read back with `eeb64`; in the networked transport it
//!      arrives through `eeba4`.
//!    * Completion is polled, not signalled: `eeb2c(obj)` runs the send phase (1 -> 3 -> 4) and returns 1 when
//!      the message was delivered *and* the reply is available; `021bb798` = "all clients ready".
//!
//!    Rust shape: `trait BattleAdapter { fn open(&mut self, client: u8) -> Handle; fn send(&mut self, h: Handle,
//!    code: u16, payload: &[u8]); fn poll(&mut self, h: Handle) -> bool; fn post_reply(&mut self, h: Handle,
//!    reply: &[u8]); fn reply(&self, h: Handle) -> Option<&[u8]>; fn reset(&mut self, h: Handle); }`
//!    with a `LocalAdapter` (an `[Slot; 4]` of `{ code, payload: Vec<u8>, reply: Option<Vec<u8>>, phase }`; the
//!    original's high-nibble-0 path) and nothing for the networked transport (D9: Wi-Fi/comm is out of scope; the
//!    stubbed `btl_comm` calls become `unreachable!`). The battle UI (D7) is a `BattleAdapter` client: it
//!    pulls frames with `recv()` (= `eec20` + `eebe8`), plays them, and answers with `post_reply`.
//!
//! How the original calls map onto it: `eea58` = `open` (allocate or return the slot; the flag word makes it
//! release), `eead0` / `eeb70` = `reset`, `eeb00` = `send`, `eeb2c` = `poll`, `eec4c` = `post_reply`,
//! `eeb64` = `reply` (the pointer and size), `eec20` = the client's `payload()` (size, data pointer),
//! `eebe8` = its first byte (the request kind), `eebdc` = clear the pending block (size 0, 4 header bytes).
//!
//! # Ports in this module
//!
//! All 45 functions of the file are ported and verified ([`PORTED`], `tests::svflow_match_rom`, area hash in
//! `tests.rs`): the adapter proper here, the message printer in [`printer`]. `server_side.spec` stubs
//! `021eea58 eead0 eeae8 eeafc eeb00 eeb2c eeb64 eeb70`; their stub lines there carry `force` (the server-side
//! ports reach the adapter through `env.call`). The D7 types over these ports are in [`stream`].
//!
//! Correction to the B12-spec notes above: the flag word `0x021F6384` is not a "release" flag. With the flag
//! clear, `eea58` keeps the slot's first object and returns the slot's; with it set, the slot is overwritten
//! with the object just created (0 when the slot was already filled) and that is returned.

pub mod printer;
pub mod stream;

use super::env::{Arg, FlowEnv};

/// Functions of this area that are ported and checked by `tests::svflow_match_rom`.
pub const PORTED: &[u32] = &[
    // adapter proper
    0x021E_EA30,
    0x021E_EA54,
    0x021E_EA58,
    0x021E_EAD0,
    0x021E_EAE0,
    0x021E_EAE8,
    0x021E_EAFC,
    0x021E_EB00,
    0x021E_EB2C,
    0x021E_EB64,
    0x021E_EB70,
    0x021E_EB84,
    0x021E_EBA4,
    0x021E_EBDC,
    0x021E_EBE8,
    0x021E_EC20,
    0x021E_EC4C,
    // message printer (`printer`)
    0x021E_EC78,
    0x021E_ECD8,
    0x021E_ED00,
    0x021E_ED24,
    0x021E_ED48,
    0x021E_ED6C,
    0x021E_ED90,
    0x021E_EE24,
    0x021E_EE80,
    0x021E_EEC8,
    0x021E_EEF4,
    0x021E_EF20,
    0x021E_EF98,
    0x021E_EFE0,
    0x021E_F1CC,
    0x021E_F204,
    0x021E_F280,
    0x021E_F3A4,
    0x021E_F460,
    0x021E_F508,
    0x021E_F594,
    0x021E_F5E8,
    0x021E_F664,
    0x021E_F694,
    0x021E_F6A8,
    0x021E_F6B0,
    0x021E_F6E8,
    0x021E_F6F0,
];

/// `0x021F6384` word: the flag `eea58` tests (see the module note on its two modes).
const OFS_FLAG: usize = 0;
/// `0x021F6388`: four adapter-object pointers.
const OFS_TABLE: usize = 4;
/// Transport / reply-state byte, client id, message size and payload offsets of the object.
const OBJ_NIBBLES: usize = 0xC;
const OBJ_ID: usize = 0xD;
const OBJ_SIZE: usize = 0x12;
const OBJ_PAYLOAD: usize = 0x14;

fn wr32(b: &mut [u8], ofs: usize, v: u32) {
    b[ofs..ofs + 4].copy_from_slice(&v.to_le_bytes());
}

fn rd16(b: &[u8], ofs: usize) -> u16 {
    u16::from_le_bytes(b[ofs..ofs + 2].try_into().unwrap())
}

fn wr16(b: &mut [u8], ofs: usize, v: u16) {
    b[ofs..ofs + 2].copy_from_slice(&v.to_le_bytes());
}

/// The transport nibble of the object (`+0x0C` bits 4-7): 0 = local, else networked.
fn networked(obj: &[u8]) -> bool {
    obj[OBJ_NIBBLES] >> 4 != 0
}

/// C: `FUN_overlay_d_93__021eea30`: clears the four table slots and stores the flag word. `g` is the
/// 0x14 bytes at `0x021F6384` (flag, then the table).
pub fn table_reset(g: &mut [u8], flag: u32) {
    for i in 0..4 {
        wr32(g, OFS_TABLE + 4 * i, 0);
    }
    wr32(g, OFS_FLAG, flag);
}

/// C: `FUN_overlay_d_93__021eea54`: empty (`bx lr`).
pub fn nop() {}

/// C: `FUN_overlay_d_93__021eebdc`: empties the pending message block (code 0, size 4).
pub fn block_reset(obj: &mut [u8]) {
    wr16(obj, 0x10, 0);
    wr16(obj, OBJ_SIZE, 4);
}

/// Heap allocation size of an adapter object and the heap-debug file/line the allocator is told.
const OBJ_SIZE_BYTES: u32 = 0xBD0;
const HEAP_ALLOC: u32 = 0x0203_0734;
const HEAP_FREE: u32 = 0x0203_07B0;
const COMM_RESET: u32 = 0x021D_4CEC;
const COMM_SEND: u32 = 0x021D_4B58;
const COMM_SEND2: u32 = 0x021D_4C84;
/// `"btl_adapter.c"` (0x021F3AE8), passed to the allocator with the line number `0x7E`.
const FILE_NAME: u32 = 0x021F_3AE8;

fn rd32(b: &[u8], ofs: usize) -> u32 {
    u32::from_le_bytes(b[ofs..ofs + 4].try_into().unwrap())
}

/// Initial field values of a fresh adapter object (the writes `eea58` makes after the allocation):
/// owner, client id, transport nibble, send phase 0, message code byte 0, reply state 0.
///
/// C: part of `FUN_overlay_d_93__021eea58`.
pub fn obj_init(obj: &mut [u8], owner: u32, client: u32, transport: u32) {
    wr32(obj, 0, owner);
    obj[OBJ_ID] = client as u8;
    obj[OBJ_NIBBLES] = (obj[OBJ_NIBBLES] & 0x0F) | ((transport as u8 & 0xF) << 4);
    obj[0xE] = 0;
    obj[0xF] = 0;
    obj[OBJ_NIBBLES] &= 0xF0;
}

/// C: `FUN_overlay_d_93__021eea58`: get-or-create the adapter object of client `idx` (0..=3).
/// `g` = the 0x14 bytes at `0x021F6384`. When the slot is empty the object is allocated
/// (`Heap_AllocDebug(heap, 0xBD0, 1, "btl_adapter.c", 0x7E)`); `alloc_at` turns the address the
/// allocator returned into the bytes of the object (here: the one buffer `arena` at `arena_addr`).
/// Flag word 0: the slot keeps its first object and the call returns the slot's; flag nonzero: the
/// slot is overwritten by the object just created (0 when none was) and that is returned.
/// Returns the object's address (0 = null).
#[allow(clippy::too_many_arguments)]
pub fn open_slot<E: FlowEnv>(
    env: &mut E,
    g: &mut [u8],
    arena: &mut [u8],
    arena_addr: u32,
    owner: u32,
    idx: u32,
    transport: u32,
    heap: u32,
) -> u32 {
    let slot = OFS_TABLE + 4 * idx as usize;
    let mut obj = 0;
    if rd32(g, slot) == 0 {
        obj = env.call(
            HEAP_ALLOC,
            &mut [
                Arg::V(heap),
                Arg::V(OBJ_SIZE_BYTES),
                Arg::V(1),
                Arg::V(FILE_NAME),
                Arg::V(0x7E),
            ],
        );
        obj_init(
            &mut arena[local_ofs(obj, arena_addr)..],
            owner,
            idx,
            transport,
        );
    }
    if rd32(g, OFS_FLAG) == 0 {
        if rd32(g, slot) == 0 {
            wr32(g, slot, obj);
        }
        rd32(g, slot)
    } else {
        wr32(g, slot, obj);
        obj
    }
}

/// C: `FUN_overlay_d_93__021eead0`: reset the send phase, the message code byte and the reply state.
pub fn obj_reset(obj: &mut [u8]) {
    obj[0xE] = 0;
    obj[0xF] = 0;
    obj[OBJ_NIBBLES] &= 0xF0;
}

/// C: `FUN_021EEAE0`: thunk to `Heap_Free` (`020307b0`).
pub fn heap_free<E: FlowEnv>(env: &mut E, ptr: u32) {
    env.call(HEAP_FREE, &mut [Arg::V(ptr)]);
}

/// C: `FUN_overlay_d_93__021eeae8`: if the flag word at `0x021F6384` is set, reset the comm layer.
pub fn comm_reset_if_flag<E: FlowEnv>(env: &mut E, g: &[u8]) {
    if rd32(g, OFS_FLAG) != 0 {
        env.call(COMM_RESET, &mut []);
    }
}

/// C: `FUN_overlay_d_93__021eeafc`: empty (`bx lr`).
pub fn nop_eafc() {}

/// C: `FUN_overlay_d_93__021eeb00`: queue a message: send phase 1, code (byte at `+0xF`, halfword at
/// `+0x10`), size (`+0x12`), `size` bytes of `data` copied to `+0x14` (`MI_CpuCopy8`), reply pointer
/// and size cleared and the reply state nibble emptied.
pub fn queue_message(obj: &mut [u8], code: u16, data: &[u8], size: u32) {
    obj[0xE] = 1;
    obj[0xF] = code as u8;
    wr16(obj, 0x10, code);
    wr16(obj, OBJ_SIZE, size as u16);
    let n = size as usize;
    obj[OBJ_PAYLOAD..OBJ_PAYLOAD + n].copy_from_slice(&data[..n]);
    wr32(obj, 4, 0);
    wr32(obj, 8, 0);
    obj[OBJ_NIBBLES] &= 0xF0;
}

/// C: `FUN_overlay_d_93__021eeb84`: send step. Networked: the comm send of the `+0x10` block
/// (`id`, block, `size + 4`, and `r3` passed through); local: 1.
pub fn send_step<E: FlowEnv>(env: &mut E, obj: &[u8], r3: u32) -> u32 {
    if !networked(obj) {
        return 1;
    }
    let size = rd16(obj, OBJ_SIZE) as u32 + 4;
    env.call(
        COMM_SEND,
        &mut [Arg::V(obj[OBJ_ID] as u32), Arg::P, Arg::V(size), Arg::V(r3)],
    )
}

/// C: `FUN_overlay_d_93__021eeb2c`: advance the send state machine (phase 1 -> send step -> 3 ->
/// receive step -> 4); 1 when the message was delivered and the reply is there. `r3` is the
/// pass-through register the send step hands to the comm layer.
pub fn poll<E: FlowEnv>(env: &mut E, obj: &mut [u8], r3: u32) -> u32 {
    match obj[0xE] {
        1 => {
            if send_step(env, obj, r3) == 0 {
                return 0;
            }
            obj[0xE] = 3;
        }
        3 => {}
        4 => return 1,
        _ => return 0,
    }
    if recv_step(env, obj) == 0 {
        return 0;
    }
    obj[0xE] = 4;
    1
}

/// C: `FUN_overlay_d_93__021eeb64`: returns the reply pointer (`+4`); when `out` is given it gets the
/// reply size (`+8`).
pub fn reply_block(obj: &[u8], out: Option<&mut [u8]>) -> u32 {
    if let Some(out) = out {
        wr32(out, 0, rd32(obj, 8));
    }
    rd32(obj, 4)
}

/// C: `FUN_overlay_d_93__021eeb70`: send phase 0; networked: reset the comm layer.
pub fn obj_close<E: FlowEnv>(env: &mut E, obj: &mut [u8]) {
    obj[0xE] = 0;
    if networked(obj) {
        env.call(COMM_RESET, &mut []);
    }
}

/// C: `FUN_overlay_d_93__021eec4c`: post the reply `(ptr, size)`. Networked: comm send2. Local: when
/// the reply state is empty, `+4 = ptr`, `+8 = size`, state nibble 1. Returns 1 (local) or the comm result.
pub fn post_reply<E: FlowEnv>(env: &mut E, obj: &mut [u8], ptr: u32, size: u32) -> u32 {
    if networked(obj) {
        return env.call(COMM_SEND2, &mut [Arg::V(ptr), Arg::V(size)]);
    }
    if obj[OBJ_NIBBLES] & 0xF == 0 {
        wr32(obj, 4, ptr);
        wr32(obj, 8, size);
        obj[OBJ_NIBBLES] = (obj[OBJ_NIBBLES] & 0xF0) | 1;
    }
    1
}

/// C: `FUN_overlay_d_93__021eeba4`: receive step. Local transport: the reply state nibble
/// (`+0x0C & 0xF`). Networked: when the comm layer has data (`021d4b98`), fetch it (`021d4bd4(id, &obj[4])`),
/// store its size at `+8` and mark the reply filled (nibble 1); returns 1, or 0 when nothing arrived.
pub fn recv_step<E: FlowEnv>(env: &mut E, obj: &mut [u8]) -> u32 {
    if !networked(obj) {
        return (obj[OBJ_NIBBLES] & 0xF) as u32;
    }
    if env.call(0x021D_4B98, &mut []) == 0 {
        return 0;
    }
    let id = obj[OBJ_ID] as u32;
    let size = env.call(0x021D_4BD4, &mut [Arg::V(id), Arg::Out(&mut obj[4..8])]);
    wr32(obj, 8, size);
    obj[OBJ_NIBBLES] = (obj[OBJ_NIBBLES] & !0xF) | 1;
    1
}

/// Address base of pointer argument 0 in the harness (`0x02E10000`): the comm layer's receive buffer
/// lies inside the object buffer of the test, so a pointer it returns is turned back into an offset.
fn local_ofs(addr: u32, obj_addr: u32) -> usize {
    addr.wrapping_sub(obj_addr) as usize
}

/// C: `FUN_overlay_d_93__021eebe8`: first byte of the received message. Local: `+0x10 & 0xFF` while the
/// reply state is empty, else 0. Networked: the first halfword of the comm receive block (`021d4c60` says
/// whether there is one, `021d4c6c` returns its address), masked to a byte.
pub fn first_byte<E: FlowEnv>(env: &mut E, obj_addr: u32, obj: &[u8]) -> u32 {
    let b = obj[OBJ_NIBBLES];
    if b >> 4 == 0 {
        return if b & 0xF == 0 {
            (rd16(obj, 0x10) & 0xFF) as u32
        } else {
            0
        };
    }
    if env.call(0x021D_4C60, &mut []) == 0 {
        return 0;
    }
    let mut p = [0u8; 4];
    env.call(0x021D_4C6C, &mut [Arg::Out(&mut p)]);
    let a = u32::from_le_bytes(p);
    (rd16(obj, local_ofs(a, obj_addr)) & 0xFF) as u32
}

/// C: `FUN_overlay_d_93__021eec20`: size and data address of the received message. Local: size `+0x12`,
/// data `obj + 0x14`. Networked: the block `021d4c6c` returns (size = its halfword at `+2`, data = block + 4).
/// Writes the data address to `out` and returns the size.
pub fn block_data<E: FlowEnv>(env: &mut E, obj_addr: u32, obj: &[u8], out: &mut [u8]) -> u32 {
    if networked(obj) {
        let mut p = [0u8; 4];
        env.call(0x021D_4C6C, &mut [Arg::Out(&mut p)]);
        let a = u32::from_le_bytes(p);
        wr32(out, 0, a.wrapping_add(4));
        rd16(obj, local_ofs(a, obj_addr) + 2) as u32
    } else {
        wr32(out, 0, obj_addr.wrapping_add(OBJ_PAYLOAD as u32));
        rd16(obj, OBJ_SIZE) as u32
    }
}

#[cfg(test)]
mod tests;
