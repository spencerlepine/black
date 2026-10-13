//! Trainer battle AI, overlay 96 (`tr_ai.c`, 0x02187F00-0x0218A574; B14).
//!
//! # What overlay 96 is (B14-spec)
//!
//! All of overlay 96 is the **trainer AI** — it has no other parts. It is a bytecode interpreter:
//! `02187f00` (`ai_create`) allocates the 0xD8-byte AI object and a script VM on the generic script
//! engine of ARM9 main (`black-script` `vm.c`, step `0x02011298`), then runs AI scripts whose commands
//! live here. The AI object (`ai`) is the VM's *ctx* (`vm[0x20]`), so every command handler receives
//! `(vm, ai)` and reads its operands from the script through `0201134c(vm)` (next u32) / sets the PC with
//! `020113b0(vm, ptr)`. 160 real functions: the command handlers of a 120-entry dispatch table at
//! `0x0218A5A8` (indexed by the VM via the init record at `0x0218A574`), their per-slot wrappers, and the
//! shared helpers below. 37 of the 197 entries the asm/Ghidra label are `.byte` fragments / `bx pc`
//! veneers (not real functions). File naming per the decomp.
//!
//! # Split (one svflow area `ai1`)
//!
//! One area for the whole overlay. A DP over the 189 in-range call edges shows at most ~13 cross any
//! balanced cut (the three dispatchers `0218a0f4` / `0218a160` / `0218a2fc` are reached from almost every
//! handler), so a single area has **zero internal crossing calls** — the minimal-crossing split. Specs:
//! `tools/rom-diff/svflow_specs/ai1.spec` (`nowork`, `set battle+96`). All 160 real functions are specced;
//! `svflow.py ai1 --all --strict-callees` is clean; closure stubs (every block lists the stubs of its
//! callees and of all its in-area descendants, so porting in any order needs no spec edit).
//!
//! # AI object layout (`ai`, 0xD8 bytes; from the disassembly)
//!
//! | ofs | field |
//! |---|---|
//! | `+0x00` | u8 phase (the `021885bc` 0/1/2 state loop) |
//! | `+0x01` | u8 current move-slot index |
//! | `+0x02` | u16 score of the current slot |
//! | `+0x24 + 2*i` | s16 per-slot score |
//! | `+0x30 + i` | s8 chosen slot per rank |
//! | `+0x38` | u32 handler result (most leaf commands write it here, then `0218a0f4` compares) |
//! | `+0x3C` / `+0x40` | u32 working mask / active-slot mask |
//! | `+0x44` | u8 flags: bit0 running, bit1 (`=2`) abort, bit4 (`=0x10`) done |
//! | `+0x46` / `+0x47` | u8 chosen move slot / chosen target slot |
//! | `+0x48 + i` | u8[4] priority |
//! | `+0x4C + 8*slot + 2*i` | u16[4][4] per-slot table (`0218a324` fills it) |
//! | `+0x94` / `+0x95` | u8 own battle position / iterating position (both 0..3) |
//! | `+0x98` / `+0x9C` | u32 kind (1 or 2) / aux |
//! | `+0xA0` | u16 heap handle; `+0xA4` / `+0xA8` ptr; `+0xB0` main module M; `+0xB4` pokecon/flow |
//! | `+0xB8` | setup; `+0xBC` self `BTL_POKEPARAM*`; `+0xC0` target `BTL_POKEPARAM*` |
//! | `+0xC4` | u32 status (the handler return value: 0 continue, 1 yield) |
//! | `+0xC8` | u32 = 4; `+0xCC` u32; `+0xD0`/`+0xD4` u64 OS tick |
//!
//! Shared helpers: `0218a0f4(vm, kind<=7, a, b, [c])` compares `a`/`b` per `kind` and, when true, writes a VM
//! register (`020113b0`); `0218a160(ai, slot<=3)` -> battle position; `0218a2fc(ai)` -> self `BTL_POKEPARAM*`
//! (tail call `021b98ac`); `0218a378` / `0218a384(ai, waza, id)` -> move-data field (tail `0201bd28` /
//! `0201beb8`). `0201c328` (`cache_create`, P4) is created once by `02187f00` — ov96 0x02187F92 is its only
//! caller.

pub mod ai1;

use crate::server_flow::env::{Arg, FlowEnv};

/// The memory model of the AI ports: byte blocks at fixed addresses (the AI object, the VM, the global AI table), with
/// every object passed as its address, as the original sees it (same type as the `battle_server.s` ports).
pub use crate::server::server3::Mem;

/// The environment of the AI: [`FlowEnv`] plus callees with a 64-bit result in `r0:r1`.
pub trait AiEnv: FlowEnv {
    /// Calls the unported callee at `addr` whose result is 64-bit (`r1 << 32 | r0`), e.g. `OS_GetTick` (0x020870D0).
    fn call64(&mut self, addr: u32, args: &mut [Arg<'_>]) -> u64;
}

/// C: `0x0218A574`, the VM init record of `ai_create` (`0201121c(heap, &rec)`): word 0 = 0x80010, word 1 = the command
/// table [`CMD_TABLE`], word 2 = its length 120; words 3..5 (0x0218A580) = the kind table `{1, 0, 2}` of `021895c4`;
/// words 6..12 (0x0218A58C) = the value ids `{1..7}` of `02189c8c`.
pub const INIT_RECORD: [u32; 13] = [0x0008_0010, 0x0218_A5A8, 0x78, 1, 0, 2, 1, 2, 3, 4, 5, 6, 7];

/// C: `0x0218A5A8`, the 120-entry AI command table (Thumb addresses, bit 0 set); the VM calls entry `id` as
/// `fn(vm, ai)` and the handler returns the AI status word `ai[0xC4]` (except `021895ac`, "stop", which returns 1).
pub const CMD_TABLE: [u32; 120] = [
    0x0218871D, 0x0218872D, 0x0218873D, 0x0218874D, 0x0218878D, 0x021887B5, 0x021887C5,
    0x021887D5, //
    0x021887E5, 0x0218883D, 0x0218884D, 0x0218889D, 0x021888AD, 0x02188905, 0x02188915,
    0x02188981, //
    0x02188991, 0x021889E9, 0x021889F9, 0x02188A55, 0x02188A65, 0x02188A75, 0x02188A85,
    0x02188A95, //
    0x02188AA5, 0x02188AD9, 0x02188AE9, 0x02188B1D, 0x02188B65, 0x02188BB1, 0x02188C1D,
    0x02188C89, //
    0x02188CA1, 0x02188D49, 0x02188D5D, 0x02188E01, 0x02188A75, 0x02188A85, 0x02188E25,
    0x02188E91, //
    0x02188F09, 0x02188F15, 0x02188F2D, 0x02188F51, 0x02188F59, 0x02188FAD, 0x02188FBD,
    0x0218905D, //
    0x0218906D, 0x0218907D, 0x021890C1, 0x021890D1, 0x021890E1, 0x021890F1, 0x02189151,
    0x02189161, //
    0x021891D5, 0x021891E5, 0x021892D9, 0x021892E9, 0x021893AD, 0x021893B5, 0x021893CD,
    0x021893D5, //
    0x021893DD, 0x02189401, 0x02189435, 0x0218945D, 0x0218948D, 0x021894B5, 0x021894C5,
    0x021894D5, //
    0x021894F9, 0x02189501, 0x02189525, 0x02189549, 0x02189591, 0x021895AD, 0x021895C5,
    0x02189621, //
    0x02189631, 0x02189669, 0x02189695, 0x021896E1, 0x02189719, 0x02189759, 0x0218979D,
    0x021897CD, //
    0x02189801, 0x0218989D, 0x02189979, 0x021899C5, 0x021899DD, 0x02189A2D, 0x02189A45,
    0x02189A6D, //
    0x02189A9D, 0x02189AC1, 0x02189B79, 0x02189BFD, 0x02189C8D, 0x02189CED, 0x02189D31,
    0x02189D39, //
    0x02189D41, 0x02189D49, 0x02189E71, 0x02189E81, 0x02189ED1, 0x02189EF9, 0x02189F35,
    0x02189F59, //
    0x02189F69, 0x02189F79, 0x02189F89, 0x02189FC1, 0x0218A035, 0x0218A075, 0x0218A085,
    0x0218A095, //
];
