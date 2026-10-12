//! D7: the server -> client command stream, as a thin Rust API over the verified adapter ports.
//!
//! The original is a two-layer stream (see the design note in [`super`]):
//!
//! 1. **Command frames**: the bytes of the server command queue ([`crate::server_flow::cmd_queue`]), decoded
//!    record by record with `read_cmd` (`021cd5e0`). [`BtlCmd`] is one decoded record.
//! 2. **Adapter messages** (the envelope): [`ServerToClient`] (`eeb00`) and [`ClientToServer`] (`eec4c` /
//!    `eeb64`), moved through a per-client channel object ([`BattleAdapter`]).
//!
//! [`LocalAdapter`] is the local transport (transport nibble 0): a four-slot table of channel objects, each the
//! 0xBD0-byte object of the original, driven only through the verified functions of [`super`]
//! (`obj_init`, `queue_message`, `poll`, `post_reply`, `reply_block`, `block_data`, `first_byte`,
//! `block_reset`, `obj_reset`, `obj_close`). The networked transport goes through `btl_comm`, which is out of
//! scope (D9): [`NoComm`] panics on any comm call, which a local object never makes.
//!
//! Deviations from the original, all forced by Rust ownership:
//!
//! * The reply is posted *by reference* in the original (`+4` holds a raw pointer, `+8` its size). Here the
//!   reply bytes live in the slot and the object's `+4` word holds a non-null token (`client + 1`), `+8` the size.
//! * `open` is the flag-word-0 path of `eea58` (get-or-create); the flag-set path (overwrite) is not exposed.
//! * The typed `BtlEvent` the UI will match on is not defined yet: ids are the raw command ids until
//!   `btl_client` (B13) says what each means.

use super::{
    block_data, block_reset, first_byte, obj_close, obj_init, obj_reset, poll, post_reply,
    queue_message, reply_block, OBJ_PAYLOAD, OBJ_SIZE_BYTES,
};
use crate::server_flow::cmd_queue::{read_cmd, CMD_ARG_TABLE};
use crate::server_flow::env::{Arg, FlowEnv};

/// Largest message payload an adapter object holds (`0xBD0 - 0x14`).
pub const MAX_PAYLOAD: usize = OBJ_SIZE_BYTES as usize - OBJ_PAYLOAD;

/// Number of client slots of the adapter table (`0x021F6388[4]`).
pub const SLOTS: usize = 4;

/// One decoded server command: the id (0..0x60, the argument-format table of `cmd_queue`) and its argument
/// words, exactly as `read_cmd` decodes them (the list formats 0 and 0x10 keep their words up to, not
/// including, the 0xFFFF0000 terminator).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BtlCmd {
    pub id: u16,
    pub args: Vec<u32>,
}

/// The id `read_cmd` returns when the frame is exhausted.
const CMD_END: u32 = 0x5E;
const LIST_END: u32 = 0xFFFF_0000;

impl BtlCmd {
    /// Reads the next command of a command-queue frame `q` (`{u32 write_count; u32 read_cursor; u8 data[]}`),
    /// advancing its read cursor; `None` when the cursor has reached the write count or `read_cmd` reports the
    /// end (id `0x5E`). An id outside the 0x80-entry argument table (corrupt data) panics like `read_cmd` does.
    ///
    /// C: `FUN_overlay_d_93__021cd5e0` (`read_cmd`) plus the cursor test its callers make.
    pub fn read_next(q: &mut [u8]) -> Option<BtlCmd> {
        let count = u32::from_le_bytes(q[0..4].try_into().unwrap());
        let cursor = u32::from_le_bytes(q[4..8].try_into().unwrap());
        if cursor >= count {
            return None;
        }
        let mut out = [0u8; 0x40];
        let id = read_cmd(q, &mut out);
        if id == CMD_END {
            return None;
        }
        let fmt = CMD_ARG_TABLE[id as usize];
        let words: Vec<u32> = (0..16)
            .map(|i| u32::from_le_bytes(out[4 * i..4 * i + 4].try_into().unwrap()))
            .collect();
        let args = if fmt == 0 || fmt == 0x10 {
            words.into_iter().take_while(|&w| w != LIST_END).collect()
        } else {
            words.into_iter().take((fmt & 0xF) as usize).collect()
        };
        Some(BtlCmd {
            id: id as u16,
            args,
        })
    }
}

/// Server -> client envelope: `eeb00(obj, code, data, size)`. `code` is the server state's request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerToClient<'a> {
    pub code: u16,
    pub payload: &'a [u8],
}

/// Client -> server envelope: `eec4c(obj, ptr, size)` (post) and `eeb64` (read back). `ptr` is the reply
/// token (see the module deviations), `size` its length in bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientToServer {
    pub ptr: u32,
    pub size: u32,
}

/// Handle of an open channel (the client id / slot index).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Handle(pub u8);

/// The server <-> client seam (D7). Maps the original calls: `open` = `eea58`, `send` = `eeb00`, `poll` =
/// `eeb2c`, `post_reply` = `eec4c`, `reply` = `eeb64`, `reset` = `eead0`/`eeb70`, `recv` = `eec20` + `eebe8`.
pub trait BattleAdapter {
    /// Gets or creates the channel of `client` (0..4).
    fn open(&mut self, client: u8) -> Handle;
    /// Queues a message for the client (state 1); clears any earlier reply.
    fn send(&mut self, h: Handle, msg: ServerToClient<'_>);
    /// Advances delivery; true when delivered and the reply is available.
    fn poll(&mut self, h: Handle) -> bool;
    /// Client side: the pending message (kind byte, payload) while no reply has been posted.
    fn recv(&self, h: Handle) -> Option<(u8, &[u8])>;
    /// Client side: posts the reply (ignored when one is already pending, like the original).
    fn post_reply(&mut self, h: Handle, reply: &[u8]);
    /// Server side: the reply block, if posted.
    fn reply(&self, h: Handle) -> Option<(ClientToServer, &[u8])>;
    /// Back to idle (`eead0`).
    fn reset(&mut self, h: Handle);
    /// Close the channel after a round (`eeb70`: phase 0).
    fn close(&mut self, h: Handle);
}

/// `FlowEnv` of the local transport: the comm layer is never reached.
struct NoComm;

impl FlowEnv for NoComm {
    fn call(&mut self, addr: u32, _args: &mut [Arg<'_>]) -> u32 {
        unreachable!("local adapter reached the comm layer ({addr:#010x}); networked transport is out of scope (D9)")
    }
}

struct Slot {
    obj: Box<[u8]>,
    reply: Vec<u8>,
}

/// Four-slot local adapter: the high-nibble-0 path of the original.
#[derive(Default)]
pub struct LocalAdapter {
    slots: [Option<Slot>; SLOTS],
}

impl LocalAdapter {
    pub fn new() -> Self {
        Self::default()
    }

    fn slot(&self, h: Handle) -> &Slot {
        self.slots[h.0 as usize].as_ref().expect("channel not open")
    }

    fn slot_mut(&mut self, h: Handle) -> &mut Slot {
        self.slots[h.0 as usize].as_mut().expect("channel not open")
    }
}

impl BattleAdapter for LocalAdapter {
    fn open(&mut self, client: u8) -> Handle {
        let i = client as usize;
        assert!(i < SLOTS, "client id out of range");
        if self.slots[i].is_none() {
            let mut obj = vec![0u8; OBJ_SIZE_BYTES as usize].into_boxed_slice();
            obj_init(&mut obj, 0, client as u32, 0);
            self.slots[i] = Some(Slot {
                obj,
                reply: Vec::new(),
            });
        }
        Handle(client)
    }

    fn send(&mut self, h: Handle, msg: ServerToClient<'_>) {
        assert!(msg.payload.len() <= MAX_PAYLOAD, "payload too large");
        let s = self.slot_mut(h);
        queue_message(&mut s.obj, msg.code, msg.payload, msg.payload.len() as u32);
        s.reply.clear();
    }

    fn poll(&mut self, h: Handle) -> bool {
        poll(&mut NoComm, &mut self.slot_mut(h).obj, 0) != 0
    }

    fn recv(&self, h: Handle) -> Option<(u8, &[u8])> {
        let obj = &self.slot(h).obj;
        // reply state nonzero: nothing pending for the client (first_byte answers 0)
        if obj[super::OBJ_NIBBLES] & 0xF != 0 {
            return None;
        }
        let kind = first_byte(&mut NoComm, 0, obj) as u8;
        let mut out = [0u8; 4];
        let size = block_data(&mut NoComm, 0, obj, &mut out) as usize;
        let ofs = u32::from_le_bytes(out) as usize;
        Some((kind, &obj[ofs..ofs + size]))
    }

    fn post_reply(&mut self, h: Handle, reply: &[u8]) {
        let s = self.slot_mut(h);
        let pending = s.obj[super::OBJ_NIBBLES] & 0xF != 0;
        post_reply(&mut NoComm, &mut s.obj, h.0 as u32 + 1, reply.len() as u32);
        if !pending {
            s.reply = reply.to_vec();
        }
    }

    fn reply(&self, h: Handle) -> Option<(ClientToServer, &[u8])> {
        let s = self.slot(h);
        if s.obj[super::OBJ_NIBBLES] & 0xF == 0 {
            return None;
        }
        let mut size = [0u8; 4];
        let ptr = reply_block(&s.obj, Some(&mut size));
        let size = u32::from_le_bytes(size);
        Some((ClientToServer { ptr, size }, &s.reply[..size as usize]))
    }

    fn reset(&mut self, h: Handle) {
        let s = self.slot_mut(h);
        obj_reset(&mut s.obj);
        block_reset(&mut s.obj);
        s.reply.clear();
    }

    fn close(&mut self, h: Handle) {
        obj_close(&mut NoComm, &mut self.slot_mut(h).obj);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server_flow::cmd_queue::{push_u16, push_u8};

    struct Panics;
    impl FlowEnv for Panics {
        fn call(&mut self, addr: u32, _: &mut [Arg<'_>]) -> u32 {
            panic!("unexpected call {addr:#x}");
        }
    }

    #[test]
    fn round_trip() {
        let mut a = LocalAdapter::new();
        let h = a.open(2);
        assert_eq!(a.open(2), h);
        assert!(!a.poll(h));
        a.send(
            h,
            ServerToClient {
                code: 0x1234,
                payload: b"hello",
            },
        );
        // delivered in one poll (local send is immediate), reply not yet there
        assert!(!a.poll(h));
        let (kind, payload) = a.recv(h).unwrap();
        assert_eq!((kind, payload), (0x34, &b"hello"[..]));
        a.post_reply(h, &[1, 2, 3]);
        assert!(a.poll(h));
        let (cts, bytes) = a.reply(h).unwrap();
        assert_eq!(cts.size, 3);
        assert_ne!(cts.ptr, 0);
        assert_eq!(bytes, &[1, 2, 3]);
        assert!(a.recv(h).is_none());
        a.reset(h);
        assert!(a.reply(h).is_none());
    }

    #[test]
    fn reads_commands() {
        // a hand-built frame: command id 3 (format 0x01: one argument), then the end
        let mut q = vec![0u8; 0x40];
        let mut env = Panics;
        push_u16(&mut env, &mut q, 3);
        push_u8(&mut env, &mut q, 7);
        let cmd = BtlCmd::read_next(&mut q).unwrap();
        assert_eq!(cmd.id, 3);
        assert_eq!(cmd.args, vec![7]);
        assert!(BtlCmd::read_next(&mut q).is_none());
    }
}
