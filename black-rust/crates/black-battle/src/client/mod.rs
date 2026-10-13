//! Battle client (`btl_client.c`, asm/battle_client.s, 0x021CD6EC-0x021D3F00; B13).
//!
//! The client is the other end of the D7 seam: it pulls the server's command frames through the adapter
//! ([`crate::server_flow::adapter`]: `eec20` data pointer + size, `eebe8` request kind), plays them on the battle view
//! (`battle_view_core`, 0x021E9xxx-0x021EBxxx, UI: screens U-phase), asks the player for an action and answers with `eec4c`.
//! This module is the **logic half only**: request dispatch, replay, selection checks, relays of server commands to
//! BattleMon / view calls. Everything the original does on the view is a trait/stub call (`FlowEnv::call`), exactly like the
//! server flow, so the same `svflow` machinery applies (`tools/rom-diff/svflow.py`, `tools/check-svflow.sh`; areas named
//! `clientN` live in this directory instead of `server_flow/`, see the two lines in those scripts).
//!
//! # Range and split (B13-spec1)
//!
//! `btl_client.c` = 0x021CD6EC-0x021D3F00 in overlay 93: right after `cmd_queue` (0x021CD6AC-0x021CD6EB; the filename string
//! `btl_client.c` is at 0x021F3A88 and used by the create routine 0x021CD6F0 `Heap_AllocDebug(heap, 0x1B8, 1, "btl_client.c",
//! 0x209)`), up to the first function of `battle_net.s` (0x021D3F04, the client's cursor helpers `021d3f04-021d3fd0` and the
//! comm layer behind them are the next file). `battle_client.s` labels 230 functions: 222 in this range (24 + 56 + 46 + 39 +
//! 57 per area below, of which 43 are `LAB_` entries Ghidra split off bigger functions, mostly `.byte` blocks and jump tables)
//! and 8 ARM/overlay-95 thunks at 0x0689xxxx. The call graph is sparse (the client is a table-driven state machine: nearly
//! every function is reached through a function pointer), so, as in B9, the split follows address order, with the tables as evidence:
//!
//! * **Request table** `0x021EFFEC`: 15 records of 16 bytes `{u8 request code, u8 aux, 2 pad, fn mode0, fn mode1, fn mode2}`
//!   (codes 1 2 3 7 5 6 4 8 0xA 0xB 0xC 0xE 0xF 0xD 0x10), indexed by the client mode (`c+0x1A7`) through `021cdd50`. These are
//!   the server's "request the client" codes (choose an action, send out, switch, replay...). The state functions are the step
//!   functions `u32 step(client *c, u32 *state)` at 0x021CDDDD-0x021D1A75 (Thumb pointers).
//! * **Playback table** `0x021F00DC`: pairs `{u32 command id, fn}`, ids 0x30-0x5D, fn `u32 handler(client *c, u32, u32 *args)`
//!   at 0x021D1E6D-0x021D3D51. Those are the server `BtlCmd` ids of
//!   [`crate::server_flow::cmd_queue`]; the bulk of the client's work is relaying each one to the view.
//!
//! | area | range | fns (labels) | what | specced B13-spec1 + B13-spec2 / ported |
//! |---|---|---|---|---|
//! | `client1` | 021CD6EC-021CDDDC | 24 | object lifecycle: create `021cd6f0` (0x1F0 bytes, 0x1B8-byte object), free `021cd8e0`, flags, replay reader, step-function setter `021cda0c`, the main step `021cda18` (`.byte` block with a jump table), request-table lookup `021cdd50` | 20 + 4 / 2 |
//! | `client2` | 021CDDDC-021CF5B0 | 56 | first request steps (`021cdddc` 2, `021cdf58` 3, `021ce1b0` 7 ...), their helpers (BGM fade, result block, random choice `021cdfe4`), mon HP ratio, three-slot flags; the action-selection callbacks (`021ce308` .. `021cf2f0`, run through `c+0xD0`) | 19 + 26 / 1 |
//! | `client3` | 021CF5B0-021D10B8 | 46 | selection checks (`021cf5b0` big pick routine, `021cf89c` can-use, `021cf9b4`), 6-slot index list `021cfba4..fbe0`, party/target helpers `021d0980 021d0b6c 021d0bb0 021d0c78`, trainer-AI switch heuristics (64-bit LCG at `c+0xF4`), AI action step `021d0770`, switch-in steps | 11 + 30 / 2 |
//! | `client4` | 021D10B8-021D2FD8 | 39 | tail of the request steps (`021d10b9 .. 021d1a75`) and the first playback handlers (`021d1e6d .. 021d2fd9`: HP bar, status, message relays), the command player `021d1a74`, level-up / learn-move `021d2a20` / `021d2ca8` | 3 + 51 / 3 |
//! | `client5` | 021D2FD8-021D3F04 | 57 | the rest of the playback handlers (one BattleMon view call each), client getters, cursor accessors, trainer-item helpers `021d3dc4 021d3e70 021d3ec8`, `021d3d84` (called from overlay 95) | 49 + 18 / 2 |
//!
//! The specs are `nowork` (the client object is not an `SvFlowWork`): the object is `arg buf 0x1C0` (allocation 0x1B8).
//! B13-spec2 specced every remaining function (231 in all; not specced: the ten Thumb-to-ARM veneers `021ce7c4 021cee98 021ceea8 021d0b3c
//! 021d0b4c 021d0b5c 021d121c 021d122c 021d3558 021d3a10`, which are stubs at their Thumb entry, and the `LAB_` pieces of `.byte`
//! functions). Conventions of the B13-spec2 blocks (headers of `client1.spec` .. `client5.spec`): closure stubs inside an area, `force`
//! stubs for callees in another client area, for adapter / server_flow / cmd_queue ports and for the targets of indirect calls (the step
//! function at `c+0x28`, the request handler at `c+0xC8`, the pending callback at `c+0xD0`, the playback handler at `c+0x17C`).
//!
//! # UI-side callees (D3 / D7)
//!
//! The client's view calls are all stubs in the specs; a port reaches them through `FlowEnv::call`, which is the future
//! battle-screen trait surface (U3 / U4 implement it in Bevy):
//!
//! * battle view (`battle_view_core`, `c+0x54`): 111 entry points 0x021E90A0-0x021EAF78 (messages `021eaf18 / 021eaf50 / 021eaf78` +
//!   `021ea614` print, `021ea710` "message done", menus `021e9760 .. 021e9e80`, yes/no `021eabc4 / 021eac54`, animations `021ea0a0 ..
//!   021ea950`, level-up windows `021eac64 .. 021eadf4`);
//! * the view-side object at `c+0x34`: `021ef9a8 021ef9c0 021ef9c8 021ef9f4 021efa94 021efad8 021efb34 021efc1c 021efc78`;
//! * sound and effects: overlay 94 `021f7998 .. 021f88bc` (22 calls) and the main-module sound layer `02005ca4 .. 02006740` (15 calls).
//!
//! # Client object (0x1B8 bytes)
//!
//! | ofs | meaning |
//! |---|---|
//! | `+0x000` | battle server-side object (main-module getters 021b8588 ...) |
//! | `+0x004` | pokecon (battle party container) |
//! | `+0x010` / `+0x014` | `BattleRecord*` (0 = not recording) / `BattleRecordReader*` |
//! | `+0x018` | command cursor (0x10 bytes; helpers in battle_net.s 021d3f04-021d3fd0) |
//! | `+0x028` | current step function (0x021CDC0D = wait-for-request step, 0x021CDA19 = main step) |
//! | `+0x02C/+0x030` | request argument pair; `+0x034` view-side object; `+0x050` adapter channel; `+0x054` view handle |
//! | `+0x0D0/+0x0D4` | pending callback and its state; `+0x0D8/+0x0DC` result pointer and byte size (answer to the server) |
//! | `+0x0E4/+0x0E6` | u16 BGM / SE ids; `+0x0E8` 4 x u16; `+0x118` party container; `+0x124` result words |
//! | `+0x130` | 3 bytes (`021cf41c/430/43c`); `+0x19C` u16 (active mon HP); `+0x1A0` 3 x u16; `+0x1A6` client id; `+0x1A7` mode |
//! | `+0x1AA/+0x1AD` | flag bytes; `+0x1AB` counter (<= 14); `+0x1B2` flag bits |

pub mod client1;
pub mod client2;
pub mod client3;
pub mod client4;
pub mod client5;

use crate::server_flow::env::{Arg, FlowEnv};

/// The D3 / D7 seam of the client: every call the client logic makes into the battle screen (view, view-side object,
/// sound and effects). The logic half (this module) is a faithful port; the other side of this trait is rewritten in
/// Bevy (U3 / U4) and is never ported. A call names the original entry point by its address (the UI rewrite matches
/// on it) and passes the argument registers of the original call site.
///
/// The provided bodies forward to [`FlowEnv::call`] (the stubs of the differential test); a production environment
/// overrides the three methods. The address ranges are checked in debug builds, so a call is never filed under the
/// wrong surface.
pub trait ClientUi: FlowEnv {
    /// Battle view (`battle_view_core`, the handle at `c+0x54`): entry points 0x021E90A0-0x021EAF78.
    fn view(&mut self, addr: u32, args: &mut [Arg<'_>]) -> u32 {
        debug_assert!(
            (0x021E_9000..0x021E_B000).contains(&addr),
            "{addr:#x} is not a view entry point"
        );
        self.call(addr, args)
    }

    /// [`Self::view`] lending the client object `held` (the view writes an answer record the
    /// client handed it earlier, e.g. the move menu's `021e98b4`; B16).
    fn view_h(&mut self, addr: u32, args: &mut [Arg<'_>], held: &mut [u8]) -> u32 {
        debug_assert!(
            (0x021E_9000..0x021E_B000).contains(&addr),
            "{addr:#x} is not a view entry point"
        );
        self.call_held(held, addr, args)
    }

    /// The view-side object at `c+0x34`: 0x021EF9A8-0x021EFC78.
    fn view_obj(&mut self, addr: u32, args: &mut [Arg<'_>]) -> u32 {
        debug_assert!(
            (0x021E_F900..0x021E_FD00).contains(&addr),
            "{addr:#x} is not a view-object call"
        );
        self.call(addr, args)
    }

    /// Sound and effects: overlay 94 (0x021F7998-0x021F88BC) and the main-module sound layer (0x02005CA4-0x02006740).
    fn sound(&mut self, addr: u32, args: &mut [Arg<'_>]) -> u32 {
        debug_assert!(
            (0x021F_7000..0x021F_9000).contains(&addr)
                || (0x0200_5000..0x0200_7000).contains(&addr),
            "{addr:#x} is not a sound call"
        );
        self.call(addr, args)
    }
}

#[cfg(test)]
impl ClientUi for crate::server_flow::testkit::TestEnv {}

/// A lending wrapper (B16) forwards the UI calls to the environment it wraps (the UI does not
/// take the lent objects; a call that writes into one uses [`ClientUi::view_h`]).
impl<E: ClientUi + ?Sized> ClientUi for crate::server_flow::env::Lend<'_, E> {
    fn view(&mut self, addr: u32, args: &mut [Arg<'_>]) -> u32 {
        self.env.view(addr, args)
    }
    fn view_h(&mut self, addr: u32, args: &mut [Arg<'_>], held: &mut [u8]) -> u32 {
        self.env.view_h(addr, args, held)
    }
    fn view_obj(&mut self, addr: u32, args: &mut [Arg<'_>]) -> u32 {
        self.env.view_obj(addr, args)
    }
    fn sound(&mut self, addr: u32, args: &mut [Arg<'_>]) -> u32 {
        self.env.sound(addr, args)
    }
}

/// Stores a little-endian halfword at `ofs`.
pub(crate) fn wr16(b: &mut [u8], ofs: usize, v: u16) {
    b[ofs..ofs + 2].copy_from_slice(&v.to_le_bytes());
}

/// Little-endian word at `ofs`.
pub(crate) fn rd32(b: &[u8], ofs: usize) -> u32 {
    u32::from_le_bytes(b[ofs..ofs + 4].try_into().unwrap())
}

/// Little-endian halfword at `ofs`.
pub(crate) fn rd16(b: &[u8], ofs: usize) -> u16 {
    u16::from_le_bytes(b[ofs..ofs + 2].try_into().unwrap())
}

/// Stores a little-endian word at `ofs`.
pub(crate) fn wr32(b: &mut [u8], ofs: usize, v: u32) {
    b[ofs..ofs + 4].copy_from_slice(&v.to_le_bytes());
}
