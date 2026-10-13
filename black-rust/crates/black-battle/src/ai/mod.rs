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

/// Little-endian word at `ofs`.
pub(crate) fn rd32(b: &[u8], ofs: usize) -> u32 {
    u32::from_le_bytes(b[ofs..ofs + 4].try_into().unwrap())
}

/// Stores a little-endian word at `ofs`.
pub(crate) fn wr32(b: &mut [u8], ofs: usize, v: u32) {
    b[ofs..ofs + 4].copy_from_slice(&v.to_le_bytes());
}

/// Little-endian halfword at `ofs`.
pub(crate) fn rd16(b: &[u8], ofs: usize) -> u16 {
    u16::from_le_bytes(b[ofs..ofs + 2].try_into().unwrap())
}
