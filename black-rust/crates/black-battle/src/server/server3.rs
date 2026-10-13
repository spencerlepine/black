//! `battle_server.s` part 3 (0x021B902C-0x021BAA94): main-module helpers, pokecon and party container, player records,
//! battle-end results, teardown and the `btl_server.c` create 021ba9c8 (B11).
//!
//! Specs: `tools/rom-diff/svflow_specs/server3.spec` (`nowork`). All 116 functions of the area are ported (B11b).
//!
//! # Memory model
//!
//! The code of this range is pointer-heavy: the main module M points at its BATTLE_SETUP, a pokecon holds M, party containers
//! hold BattleMon pointers, the server object S is created inside a heap block, and the same functions are handed pointers
//! into the middle of these objects (`pc + 4 + cl * 0x1C`). The ports therefore work on [`Mem`], a set of byte blocks at fixed
//! addresses, with every object and pointer argument given as its address, exactly as the original sees it. Two read-only ROM
//! tables of `battle_server_rodata` are always mapped ([`FIRST_MON_SLOT`] at 0x021EFCF0, [`POS_LISTS`] at 0x021EFD70).
//!
//! Callees outside this area go through [`FlowEnv`] (`server1` / `server2`, `server_side`, `core*`, `api`, the client, the
//! BattleMon accessors 021d5xxx and the ARM9 main routines); callees inside the area are called directly.

use super::MemExt;
use crate::server_flow::env::{Arg, FlowEnv};
use std::rc::Rc;

/// Functions of this area that are ported and checked by `tests::svflow_match_rom`.
pub const PORTED: &[u32] = &[
    0x021B_902C,
    0x021B_904C,
    0x021B_9064,
    0x021B_9090,
    0x021B_90BC,
    0x021B_90E8,
    0x021B_90F0,
    0x021B_90F8,
    0x021B_9114,
    0x021B_911C,
    0x021B_9120,
    0x021B_916C,
    0x021B_9184,
    0x021B_919C,
    0x021B_91E4,
    0x021B_9210,
    0x021B_9234,
    0x021B_9244,
    0x021B_92A0,
    0x021B_92B0,
    0x021B_92D4,
    0x021B_9358,
    0x021B_937C,
    0x021B_93E0,
    0x021B_9424,
    0x021B_9460,
    0x021B_9634,
    0x021B_9674,
    0x021B_969C,
    0x021B_97A0,
    0x021B_985C,
    0x021B_9864,
    0x021B_98AC,
    0x021B_990C,
    0x021B_9920,
    0x021B_9934,
    0x021B_9940,
    0x021B_994C,
    0x021B_997C,
    0x021B_99C0,
    0x021B_99E0,
    0x021B_99F0,
    0x021B_9A00,
    0x021B_9A24,
    0x021B_9A40,
    0x021B_9A50,
    0x021B_9A5C,
    0x021B_9A6C,
    0x021B_9AB0,
    0x021B_9AC8,
    0x021B_9AE0,
    0x021B_9AEC,
    0x021B_9AF8,
    0x021B_9B04,
    0x021B_9B10,
    0x021B_9B40,
    0x021B_9B54,
    0x021B_9B60,
    0x021B_9B94,
    0x021B_9B98,
    0x021B_9BC4,
    0x021B_9BF0,
    0x021B_9C00,
    0x021B_9C10,
    0x021B_9C24,
    0x021B_9C50,
    0x021B_9C98,
    0x021B_9CBC,
    0x021B_9CE8,
    0x021B_9D10,
    0x021B_9DB8,
    0x021B_9DE8,
    0x021B_9E38,
    0x021B_9E7C,
    0x021B_9EF0,
    0x021B_9F70,
    0x021B_9F84,
    0x021B_9FAC,
    0x021B_9FBC,
    0x021B_A004,
    0x021B_A020,
    0x021B_A02C,
    0x021B_A064,
    0x021B_A080,
    0x021B_A08C,
    0x021B_A094,
    0x021B_A0CC,
    0x021B_A128,
    0x021B_A178,
    0x021B_A1A8,
    0x021B_A1B4,
    0x021B_A1CC,
    0x021B_A1E4,
    0x021B_A1EC,
    0x021B_A20C,
    0x021B_A238,
    0x021B_A250,
    0x021B_A2D4,
    0x021B_A2E4,
    0x021B_A534,
    0x021B_A54C,
    0x021B_A5C4,
    0x021B_A5D0,
    0x021B_A5D8,
    0x021B_A618,
    0x021B_A648,
    0x021B_A670,
    0x021B_A698,
    0x021B_A6C0,
    0x021B_A710,
    0x021B_A73C,
    0x021B_A7F8,
    0x021B_A820,
    0x021B_A82C,
    0x021B_A878,
    0x021B_A9C8,
];

// ---- memory -------------------------------------------------------------------------------------------------------------

/// Byte blocks at fixed addresses: the objects a function works on (and the ROM tables it reads). An address outside every
/// block goes to the [`MemExt`] backend when one is installed (B16: the battle integration's shared object table), else it
/// panics.
#[derive(Default)]
pub struct Mem {
    blocks: Vec<(u32, Vec<u8>)>,
    ext: Option<Rc<dyn MemExt>>,
}

/// `battle_server_rodata` 0x021EFCF0: first BattleMon slot (mon id) of each client {0, 12, 6, 18} (+ the bytes after it).
pub const FIRST_MON_SLOT: [u8; 16] = [0, 12, 6, 18, 3, 5, 0, 2, 4, 0, 3, 5, 7, 2, 4, 6];
const FIRST_MON_SLOT_ADDR: u32 = 0x021E_FCF0;
/// 0x021EFD70: 6 records of 10 bytes `{u16 n0, u16 n1, u8 list0[3], u8 list1[3]}` (021b99e0 / 021b9a6c).
pub const POS_LISTS: [u8; 60] = [
    2, 0, 2, 0, 5, 3, 6, 0, 2, 6, //
    2, 0, 2, 0, 4, 2, 6, 1, 3, 6, //
    3, 0, 3, 0, 1, 3, 5, 0, 2, 4, //
    3, 0, 3, 0, 0, 2, 4, 1, 3, 5, //
    2, 0, 2, 0, 1, 3, 6, 4, 2, 6, //
    2, 0, 2, 0, 0, 2, 6, 5, 3, 6, //
];
const POS_LISTS_ADDR: u32 = 0x021E_FD70;

impl Mem {
    /// A memory with only the ROM tables mapped.
    pub fn new() -> Self {
        let mut m = Mem::default();
        m.map(FIRST_MON_SLOT_ADDR, FIRST_MON_SLOT.to_vec());
        m.map(POS_LISTS_ADDR, POS_LISTS.to_vec());
        m
    }

    /// A memory with the ROM tables mapped over the shared memory `ext` (B16).
    pub fn with_ext(ext: Rc<dyn MemExt>) -> Self {
        let mut m = Mem::new();
        m.ext = Some(ext);
        m
    }

    /// Maps `bytes` at `base`.
    pub fn map(&mut self, base: u32, bytes: Vec<u8>) {
        self.blocks.push((base, bytes));
    }

    /// Unmaps the block at `base` and returns its bytes.
    pub fn unmap(&mut self, base: u32) -> Vec<u8> {
        let i = self
            .blocks
            .iter()
            .position(|b| b.0 == base)
            .expect("unmapped block");
        self.blocks.remove(i).1
    }

    fn find(&self, addr: u32, n: u32) -> Option<(usize, usize)> {
        for (i, (base, b)) in self.blocks.iter().enumerate() {
            let o = addr.wrapping_sub(*base);
            if o < b.len() as u32 && o + n <= b.len() as u32 {
                return Some((i, o as usize));
            }
        }
        None
    }

    fn unmapped(addr: u32) -> ! {
        panic!("server3: access to unmapped address {addr:#010x}");
    }

    fn read(&self, addr: u32, out: &mut [u8]) {
        match (self.find(addr, out.len() as u32), &self.ext) {
            (Some((i, o)), _) => out.copy_from_slice(&self.blocks[i].1[o..o + out.len()]),
            (None, Some(e)) => e.read(addr, out),
            (None, None) => Self::unmapped(addr),
        }
    }

    fn write(&mut self, addr: u32, data: &[u8]) {
        match self.find(addr, data.len() as u32) {
            Some((i, o)) => self.blocks[i].1[o..o + data.len()].copy_from_slice(data),
            None => match &self.ext {
                Some(e) => e.write(addr, data),
                None => Self::unmapped(addr),
            },
        }
    }

    pub fn r8(&self, addr: u32) -> u32 {
        let mut b = [0; 1];
        self.read(addr, &mut b);
        u32::from(b[0])
    }

    pub fn r16(&self, addr: u32) -> u32 {
        let mut b = [0; 2];
        self.read(addr, &mut b);
        u32::from(u16::from_le_bytes(b))
    }

    pub fn r32(&self, addr: u32) -> u32 {
        let mut b = [0; 4];
        self.read(addr, &mut b);
        u32::from_le_bytes(b)
    }

    pub fn w8(&mut self, addr: u32, v: u32) {
        self.write(addr, &[v as u8]);
    }

    pub fn w16(&mut self, addr: u32, v: u32) {
        self.write(addr, &(v as u16).to_le_bytes());
    }

    pub fn w32(&mut self, addr: u32, v: u32) {
        self.write(addr, &v.to_le_bytes());
    }

    /// `MI_CpuCopy8(src, dst, n)` (bytewise, ascending).
    pub fn copy8(&mut self, src: u32, dst: u32, n: u32) {
        for k in 0..n {
            let v = self.r8(src.wrapping_add(k));
            self.w8(dst.wrapping_add(k), v);
        }
    }
}

/// Calls an unported callee whose arguments are all plain values (`args=v,..` in the spec).
fn call<E: FlowEnv>(env: &mut E, addr: u32, args: &[u32]) -> u32 {
    let mut a: Vec<Arg<'_>> = args.iter().map(|&v| Arg::V(v)).collect();
    env.call(addr, &mut a)
}

// ---- object layouts (offsets; see `server/mod.rs`) -----------------------------------------------------------------------

/// M: server object S (`btl_server.c`).
const M_SERVER: u32 = 0x8;
/// M: 4 client object pointers.
const M_CLIENTS: u32 = 0x10;
/// M: 4 player records of 0x28 bytes from +0x20 (the record fields below are relative to M + 0x28 * i).
const REC_STRIDE: u32 = 0x28;
const REC_PTR0: u32 = 0x20;
const REC_PTR1: u32 = 0x24;
const REC_U16A: u32 = 0x28;
const REC_U16B: u32 = 0x2A;
const REC_WORD: u32 = 0x2C;
/// M: pokecon 0 / pokecon 1 (0xE8 bytes each, inline).
const M_PC0: u32 = 0xC8;
const M_PC1: u32 = 0x1B0;
/// M: 4 party copies, 4 second party copies, the scratch party, one more party word.
const M_PARTY_A: u32 = 0x298;
const M_PARTY_B: u32 = 0x2A8;
const M_PARTY_SCRATCH: u32 = 0x2B8;
const M_2BC: u32 = 0x2BC;
/// M: the Pokemon an Illusion disguise shows (020173F0, created once).
const M_ILLUSION_MON: u32 = 0x2C0;
/// M: 4 per-client view objects.
const M_VIEW_OBJS: u32 = 0x3E0;
/// M: 0x18-byte block handed to the server create (021d787c).
const M_3F0: u32 = 0x3F0;
/// M: u8[6] client id per battle position.
const M_POS_CLIENT: u32 = 0x428;
const M_430: u32 = 0x430;
/// M: money-like counter capped at [`MONEY_CAP`].
const M_MONEY: u32 = 0x434;
const M_438: u32 = 0x438;
const M_440: u32 = 0x440;
const M_442: u32 = 0x442;
/// M: battle result code (7 = none yet).
const M_RESULT: u32 = 0x444;
const M_448: u32 = 0x448;
/// M: u16 heap id.
const M_HEAP: u32 = 0x468;
/// M: u8 own client id.
const M_OWN_CLIENT: u32 = 0x46C;
const M_46E: u32 = 0x46E;
/// M: flag bits.
const M_FLAGS: u32 = 0x473;
const MONEY_CAP: u32 = 99_999;

/// BATTLE_SETUP fields.
const SET_TYPE: u32 = 0x00;
const SET_OWN_CLIENT: u32 = 0x21;
const SET_COMM: u32 = 0x23;
const SET_PARTIES: u32 = 0x24;
const SET_28: u32 = 0x28;
const SET_34: u32 = 0x34;
const SET_44: u32 = 0x44;
const SET_58: u32 = 0x58;
const SET_60: u32 = 0x60;
const SET_64: u32 = 0x64;
const SET_6C: u32 = 0x6C;
const SET_70: u32 = 0x70;
const SET_7F: u32 = 0x7F;
const SET_80: u32 = 0x80;
const SET_RESULT: u32 = 0x8C;
const SET_90: u32 = 0x90;
const SET_RECORD: u32 = 0x94;
const SET_RECORD_SIZE: u32 = 0x98;
const SET_9C: u32 = 0x9C;
const SET_C1: u32 = 0xC1;
const SET_C2: u32 = 0xC2;
const SET_STATUS: u32 = 0xC8;
const SET_E0: u32 = 0xE0;

/// Pokecon: M, party containers (0x1C bytes from +4), party source per client, BattleMon per mon id, "use copy" flag.
const PC_M: u32 = 0x00;
const PC_PARTIES: u32 = 0x04;
const PARTY_SIZE: u32 = 0x1C;
const PC_SRC: u32 = 0x74;
const PC_MONS: u32 = 0x84;
const PC_FLAG: u32 = 0xE4;

/// Party container: offset of the u8 mon count (after the 6 BattleMon pointers).
const PARTY_COUNT: u32 = 0x18;

/// Ability Illusion, species Zoroark (as the BattleMon getters report them).
const ABILITY_ILLUSION: u32 = 0x95;
const SPECIES_ZOROARK: u32 = 0x23B;
/// Heap id the battle allocates from.
const HEAP_BATTLE: u32 = 0x12;

/// The heap-id word that 0201B7A0 / 0201B7B4 / 0201B818 get: `(M[0x468] & 0x7FFF) | 0x8000`.
fn heap_flag(mem: &Mem, m: u32) -> u32 {
    (mem.r16(m + M_HEAP) & 0x7FFF) | 0x8000
}

fn setup(mem: &Mem, m: u32) -> u32 {
    mem.r32(m)
}

fn own_client(mem: &Mem, m: u32) -> u32 {
    mem.r8(m + M_OWN_CLIENT)
}

/// The client object `M[0x10 + 4 * cl]`.
fn client(mem: &Mem, m: u32, cl: u32) -> u32 {
    mem.r32(m.wrapping_add(cl << 2).wrapping_add(M_CLIENTS))
}

// ---- small pure helpers --------------------------------------------------------------------------------------------------

/// C: `FUN_overlay_d_93__021b902c`: 1 if `a == b` or `a == (b + 2) & 3` (the same position or its partner), else 0.
pub fn same_or_partner(a: u32, b: u32) -> u32 {
    u32::from(a == b || a == (b.wrapping_add(2) & 3))
}

/// C: `FUN_overlay_d_93__021b904c`: [`pick_client`] for the own client, as a byte.
pub fn pick_own_side_client<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, b: u32) -> u32 {
    pick_client(env, m, own_client(mem, m), b) & 0xFF
}

/// C: `FUN_overlay_d_93__021b9064`: `r = (a & 1) ^ 1` (the opposite side); with `b != 0`, `c = r + ((b & 1) << 1)` (u8) is
/// taken instead when the client exists (`021b8b9c(M, c)`).
pub fn pick_client<E: FlowEnv>(env: &mut E, m: u32, a: u32, b: u32) -> u32 {
    let mut r = (a & 1) ^ 1;
    if b != 0 {
        let c = (r + ((b & 1) << 1)) & 0xFF;
        if call(env, 0x021B_8B9C, &[m, c]) != 0 {
            r = c;
        }
    }
    r
}

/// C: `FUN_overlay_d_93__021b9090`: not in comm mode and `cl` is the own client: `02007F1C(setup[0x60], x, 1, M[0x468])`.
pub fn own_view_call_7f1c<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, cl: u32, x: u32) {
    own_view_call(env, mem, m, cl, x, 0x0200_7F1C);
}

/// C: `FUN_overlay_d_93__021b90bc`: as [`own_view_call_7f1c`] with `02007E50`.
pub fn own_view_call_7e50<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, cl: u32, x: u32) {
    own_view_call(env, mem, m, cl, x, 0x0200_7E50);
}

fn own_view_call<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, cl: u32, x: u32, f: u32) {
    let s = setup(mem, m);
    if mem.r8(s + SET_COMM) == 0 && cl == own_client(mem, m) {
        let h = mem.r16(m + M_HEAP);
        call(env, f, &[mem.r32(s + SET_60), x, 1, h]);
    }
}

/// C: `FUN_overlay_d_93__021b90e8`: `setup[0x60]`.
pub fn setup_60(mem: &Mem, m: u32) -> u32 {
    mem.r32(setup(mem, m) + SET_60)
}

/// C: `FUN_overlay_d_93__021b90f0`: `setup[0x64]`.
pub fn setup_64(mem: &Mem, m: u32) -> u32 {
    mem.r32(setup(mem, m) + SET_64)
}

/// C: `FUN_overlay_d_93__021b90f8`: 1 if the setup has a record buffer (`setup[0x94] != 0`) and the battle is not a comm
/// battle.
pub fn records_locally(mem: &Mem, m: u32) -> u32 {
    let s = setup(mem, m);
    u32::from(mem.r32(s + SET_RECORD) != 0 && mem.r8(s + SET_COMM) == 0)
}

/// C: `FUN_overlay_d_93__021b9114`: the comm flag `setup[0x23]`.
pub fn is_comm(mem: &Mem, m: u32) -> u32 {
    mem.r8(setup(mem, m) + SET_COMM)
}

/// C: `sub_021b911c`: returns 0 (the callers pass two arguments it never reads).
pub fn zero() -> u32 {
    0
}

/// C: `FUN_overlay_d_93__021b9120`: if `setup[0x90] == 6`: `c = 021b8cfc(M, n)`, and `setup[0x90]` becomes the number (u8)
/// of the positions `(n - 1) & 0xFF` down to 0 whose client `M[0x428 + p]` is `c` (nothing counted for `n == 0`).
pub fn count_positions_of<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32, n: u32) {
    let s = setup(mem, m);
    if mem.r8(s + SET_90) != 6 {
        return;
    }
    let c = call(env, 0x021B_8CFC, &[m, n]);
    let mut cnt = 0u32;
    let mut p = n.wrapping_sub(1) & 0xFF;
    let mut prev = n;
    while prev != 0 {
        if c == mem.r8(m + p + M_POS_CLIENT) {
            cnt = (cnt + 1) & 0xFF;
        }
        prev = p;
        p = p.wrapping_sub(1) & 0xFF;
    }
    let s = setup(mem, m);
    mem.w8(s + SET_90, cnt);
}

/// C: `FUN_overlay_d_93__021b916c`: sets the result code `M[0x444] = v` while it is still 7 (none).
pub fn set_result_once(mem: &mut Mem, m: u32, v: u32) {
    if mem.r32(m + M_RESULT) == 7 {
        mem.w32(m + M_RESULT, v);
    }
}

/// C: `FUN_overlay_d_93__021b9184`: `setup[0xC1] |= 1`.
pub fn set_setup_c1_bit0(mem: &mut Mem, m: u32) {
    let s = setup(mem, m);
    let v = mem.r8(s + SET_C1);
    mem.w8(s + SET_C1, v | 1);
}

/// C: `FUN_overlay_d_93__021b919c`: once (flag bit 3): `M[0x430]` is doubled when flag bit 6 is set, then replaced by
/// `0201119C(M[0x430])`; returns `M[0x430]`.
pub fn finalize_430<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32) -> u32 {
    let f = mem.r8(m + M_FLAGS);
    if f & 8 == 0 {
        if f & 0x40 != 0 {
            let v = mem.r32(m + M_430);
            mem.w32(m + M_430, v << 1);
        }
        let v = call(env, 0x0201_119C, &[mem.r32(m + M_430)]);
        mem.w32(m + M_430, v);
        let f = mem.r8(m + M_FLAGS);
        mem.w8(m + M_FLAGS, f | 8);
    }
    mem.r32(m + M_430)
}

/// C: `sub_021b91e4`: unless frozen (flag bit 3): `M[0x434] += v`, capped at 99999.
pub fn add_money(mem: &mut Mem, m: u32, v: u32) {
    if mem.r8(m + M_FLAGS) & 8 != 0 {
        return;
    }
    let s = mem.r32(m + M_MONEY).wrapping_add(v);
    mem.w32(m + M_MONEY, s);
    if s > MONEY_CAP {
        mem.w32(m + M_MONEY, MONEY_CAP);
    }
}

/// C: `FUN_overlay_d_93__021b9210`: `M[0x434]`, doubled (capped at 99999) when flag bit 6 is set.
pub fn money(mem: &Mem, m: u32) -> u32 {
    let mut v = mem.r32(m + M_MONEY);
    if mem.r8(m + M_FLAGS) & 0x40 != 0 {
        v <<= 1;
        if v > MONEY_CAP {
            v = MONEY_CAP;
        }
    }
    v
}

/// C: `sub_021b9234`: sets flag bit 6 (doubling).
pub fn set_doubling(mem: &mut Mem, m: u32) {
    let f = mem.r8(m + M_FLAGS);
    mem.w8(m + M_FLAGS, f | 0x40);
}

/// C: `FUN_overlay_d_93__021b9244`: once (flag bit 4): for battle types 0 / 1, `M[0x438] = min(021d7ff8(setup[0x80], party),
/// 0200C274(02012F08(setup[0x58])))` with the own client's party of pokecon 1. Returns the stored value (r0 at the
/// `pop {r4, r5, r6, pc}`), 0 when the flag was already set or the battle type is > 1 (V-0022; client4's 021d1410 /
/// 021d16b0 read it).
pub fn compute_438<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32) -> u32 {
    if mem.r8(m + M_FLAGS) & 0x10 != 0 {
        return 0;
    }
    let s = setup(mem, m);
    let mut r = 0;
    if mem.r32(s + SET_TYPE) <= 1 {
        let a = call(env, 0x0201_2F08, &[mem.r32(s + SET_58)]);
        let lim = call(env, 0x0200_C274, &[a]);
        let party = party_addr2(m + M_PC1, own_client(mem, m));
        let s = setup(mem, m);
        let v = call(env, 0x021D_7FF8, &[mem.r8(s + SET_80), party]);
        r = v.min(lim);
        mem.w32(m + M_438, r);
    }
    let f = mem.r8(m + M_FLAGS);
    mem.w8(m + M_FLAGS, f | 0x10);
    r
}

/// C: `FUN_overlay_d_93__021b92a0`: sets flag bit 5.
pub fn set_flag5(mem: &mut Mem, m: u32) {
    let f = mem.r8(m + M_FLAGS);
    mem.w8(m + M_FLAGS, f | 0x20);
}

/// C: `FUN_overlay_d_93__021b92b0`: battle types 0 / 1 outside comm mode: [`mon_pokemon_notify`] with 5 (`b != 0`) or 4.
pub fn mon_pokemon_notify_local<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, mon: u32, b: u32) {
    let s = setup(mem, m);
    if mem.r32(s + SET_TYPE) <= 1 && mem.r8(s + SET_COMM) == 0 {
        mon_pokemon_notify(env, mem, m, mon, if b != 0 { 5 } else { 4 });
    }
}

/// C: `FUN_overlay_d_93__021b92d4`: for the BattleMon with mon's id in pokecon 1 and in pokecon 0: `0201B7A0(pokemon, x,
/// 021b8714(M)[0xA], heap | 0x8000)`.
pub fn mon_pokemon_notify<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, mon: u32, x: u32) {
    for_both_pokemon(env, mem, m, mon, x, 0x0201_B7A0);
}

fn for_both_pokemon<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, mon: u32, x: u32, f: u32) {
    let id = call(env, 0x021D_5440, &[mon]);
    let a = pc_mon2(mem, m + M_PC1, id);
    let b = pc_mon2(mem, m + M_PC0, id);
    let pa = call(env, 0x021D_5844, &[a]);
    let pb = call(env, 0x021D_5844, &[b]);
    let s = call(env, 0x021B_8714, &[m]);
    call(env, f, &[pa, x, mem.r16(s + 0xA), heap_flag(mem, m)]);
    call(env, f, &[pb, x, mem.r16(s + 0xA), heap_flag(mem, m)]);
}

/// C: `FUN_overlay_d_93__021b9358`: `021d54f4(pc0 mon id, pc1 mon id)` (the BattleMons of mon id `id` in both pokecons).
pub fn compare_pc_mons<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, id: u32) -> u32 {
    let a = pc_mon(mem, m + M_PC1, id);
    let b = pc_mon(mem, m + M_PC0, id);
    call(env, 0x021D_54F4, &[b, a])
}

/// C: `FUN_overlay_d_93__021b937c` (r0 unused): if `021d6c24(mon)` (an Illusion is active), the member of mon's party whose
/// Pokemon is mon's visible one (`021d5864`); otherwise (or when there is none) `mon` itself.
pub fn illusion_source<E: FlowEnv>(env: &mut E, mem: &Mem, _r0: u32, pc: u32, mon: u32) -> u32 {
    if call(env, 0x021D_6C24, &[mon]) == 0 {
        return mon;
    }
    let id = call(env, 0x021D_5440, &[mon]);
    // through the thunk 021b8ddc
    let cl = call(env, 0x021B_8D08, &[id]);
    let vis = call(env, 0x021D_5864, &[mon]);
    let party = party_addr2(pc, cl);
    let n = party_count(mem, party);
    for i in 0..n {
        let bm = party_get2(mem, party, i & 0xFF);
        if call(env, 0x021D_5844, &[bm]) == vis {
            return bm;
        }
    }
    mon
}

// ---- pokecon (0xE8 bytes) ------------------------------------------------------------------------------------------------

/// C: `FUN_overlay_d_93__021b93e0`: init: `pc[0] = M`, `pc[0xE4] = flag`, the 4 party containers cleared, the 24 BattleMon
/// slots and the 4 party sources zeroed.
pub fn pokecon_init(mem: &mut Mem, pc: u32, m: u32, flag: u32) {
    mem.w32(pc + PC_M, m);
    mem.w32(pc + PC_FLAG, flag);
    for i in 0..4 {
        party_clear(mem, pc + PC_PARTIES + i * PARTY_SIZE);
    }
    for j in 0..24 {
        mem.w32(pc + j * 4 + PC_MONS, 0);
    }
    for i in 0..4 {
        mem.w32(pc + i * 4 + PC_SRC, 0);
    }
}

/// C: `FUN_overlay_d_93__021b9424`: every non-null BattleMon slot freed (`021D53B8`) and zeroed, the 4 parties cleared.
pub fn pokecon_free<E: FlowEnv>(env: &mut E, mem: &mut Mem, pc: u32) {
    pokecon_free_mons(env, mem, pc);
    for i in 0..4 {
        party_clear(mem, pc + PC_PARTIES + i * PARTY_SIZE);
    }
}

/// C: `FUN_overlay_d_93__021b9460`: sets up client `cl` of the pokecon: party source `src = 021ba178(M, cl, pc[0xE4])`, one
/// BattleMon (`021d4d84(pokemon, id, 0x12)`) per Pokemon of src into the mon-id slots from `0x021EFCF0[cl]` and appended to
/// party `cl`; the members with Illusion (ability 0x95) before the last eligible one (021b9634) disguise as it; for client 1
/// outside a link battle (`021b86b8(M) == 0`) a Zoroark with Illusion disguises as the first of client 0's species 243 /
/// 244 / 245 rotated by one (the disguise Pokemon `M[0x2C0]` created once by 020173F0); then the dead mons go to the end
/// (021b9b60).
pub fn pokecon_setup_client<E: FlowEnv>(env: &mut E, mem: &mut Mem, pc: u32, m: u32, cl: u32) {
    let src = party_source(env, mem, m, cl, mem.r32(pc + PC_FLAG));
    let party = pc + PC_PARTIES + cl.wrapping_mul(PARTY_SIZE);
    let n = call(env, 0x0201_AA18, &[src]);
    let base = mem.r8(FIRST_MON_SLOT_ADDR.wrapping_add(cl));
    mem.w32(pc.wrapping_add(cl << 2) + PC_SRC, src);
    let mut k = base;
    for i in 0..n {
        let pp = call(env, 0x0201_AB28, &[src, i]);
        let bm = call(env, 0x021D_4D84, &[pp, k, HEAP_BATTLE]);
        mem.w32(pc + (k << 2) + PC_MONS, bm);
        let bm = mem.r32(pc + (k << 2) + PC_MONS);
        party_append(mem, party, bm);
        k = (k + 1) & 0xFF;
    }
    let last = party_last_eligible(env, src) as i32;
    if last > 0 {
        let mut k = base;
        for _ in 0..last {
            let bm = mem.r32(pc + (k << 2) + PC_MONS);
            if call(env, 0x021D_59B4, &[bm, 0x11]) == ABILITY_ILLUSION {
                let pp = call(env, 0x0201_AB28, &[src, last as u32]);
                let bm = mem.r32(pc + (k << 2) + PC_MONS);
                call(env, 0x021D_5848, &[bm, pp]);
            }
            k = (k + 1) & 0xFF;
        }
    }
    if call(env, 0x021B_86B8, &[m]) == 0 && cl == 1 && n != 0 {
        let mut k = base;
        let mut j = 0u32;
        loop {
            let bm = mem.r32(pc + (k << 2) + PC_MONS);
            if call(env, 0x021D_5444, &[bm]) == SPECIES_ZOROARK
                && call(env, 0x021D_59B4, &[bm, 0x11]) == ABILITY_ILLUSION
            {
                let mut disguise = 0u32;
                let src0 = party_source(env, mem, m, 0, mem.r32(pc + PC_FLAG));
                let n0 = call(env, 0x0201_AA18, &[src0]) & 0xFF;
                let mut i = 0u32;
                while i < n0 {
                    let pp = call(env, 0x0201_AB28, &[src0, i]);
                    let sp = call(env, 0x0201_7E1C, &[pp, 5, 0]) & 0xFFFF;
                    let d = match sp {
                        243 => 244,
                        244 => 245,
                        245 => 243,
                        _ => 0,
                    };
                    if d != 0 {
                        disguise = d;
                        break;
                    }
                    i = (i + 1) & 0xFF;
                }
                if disguise != 0 {
                    if mem.r32(m + M_ILLUSION_MON) == 0 {
                        let lv = call(env, 0x021D_59B4, &[bm, 0xF]);
                        let h = mem.r16(m + M_HEAP);
                        let p = call(env, 0x0201_73F0, &[disguise, lv & 0xFFFF, 0, 0, h]);
                        mem.w32(m + M_ILLUSION_MON, p);
                    }
                    let p = mem.r32(m + M_ILLUSION_MON);
                    call(env, 0x021D_5848, &[bm, p]);
                }
            }
            j += 1;
            k = (k + 1) & 0xFF;
            if j >= n {
                break;
            }
        }
    }
    party_dead_to_end(env, mem, party);
}

/// C: `FUN_overlay_d_93__021b9634`: index of the last Pokemon `p` of `src` with `02017E1C(p, 0x4C, 0) == 0` and
/// `02017E1C(p, 0xA0, 0) != 0`, else -1 (signed index walk).
pub fn party_last_eligible<E: FlowEnv>(env: &mut E, src: u32) -> u32 {
    let n = call(env, 0x0201_AA18, &[src]);
    let mut i = n.wrapping_sub(1) as i32;
    while i >= 0 {
        let p = call(env, 0x0201_AB28, &[src, i as u32]);
        if call(env, 0x0201_7E1C, &[p, 0x4C, 0]) == 0 && call(env, 0x0201_7E1C, &[p, 0xA0, 0]) != 0
        {
            return i as u32;
        }
        i -= 1;
    }
    u32::MAX
}

/// C: `FUN_overlay_d_93__021b9674`: frees the 24 BattleMon slots (`021D53B8`, then zero).
pub fn pokecon_free_mons<E: FlowEnv>(env: &mut E, mem: &mut Mem, pc: u32) {
    for j in 0..24 {
        let a = pc + j * 4 + PC_MONS;
        let v = mem.r32(a);
        if v != 0 {
            call(env, 0x021D_53B8, &[v]);
            mem.w32(a, 0);
        }
    }
}

/// C: `FUN_overlay_d_93__021b969c` (pc, M, cl, x): syncs party `cl` back into its source: for each member `021d6b84(mon, x)`
/// and its Pokemon appended to the scratch party `M[0x2B8]` (cleared first), remembering for an Illusion user (021d6c24) the
/// index of the member it shows; then `0201AC2C(scratch, src)` and per member `021d6c3c(mon, src[i])`, re-applying the
/// disguises (`021d5848(mon, src[j])`).
pub fn pokecon_sync_back<E: FlowEnv>(env: &mut E, mem: &Mem, pc: u32, m: u32, cl: u32, x: u32) {
    let src = pc_source(mem, pc, cl);
    let party = party_addr(pc, cl);
    let n = call(env, 0x0201_AA18, &[src]);
    call(env, 0x0201_A948, &[mem.r32(m + M_PARTY_SCRATCH)]);
    let mut shows = [6u8; 6];
    for i in 0..n {
        let bm = party_get(mem, party, i & 0xFF);
        call(env, 0x021D_6B84, &[bm, x]);
        let pp = call(env, 0x021D_5844, &[bm]);
        call(env, 0x0201_A98C, &[mem.r32(m + M_PARTY_SCRATCH), pp]);
        if call(env, 0x021D_6C24, &[bm]) != 0 {
            let vis = call(env, 0x021D_5864, &[bm]);
            for j in 0..n {
                let o = party_get(mem, party, j & 0xFF);
                if vis == call(env, 0x021D_5844, &[o]) {
                    shows[i as usize] = j as u8;
                    break;
                }
            }
        }
    }
    call(env, 0x0201_AC2C, &[mem.r32(m + M_PARTY_SCRATCH), src]);
    for i in 0..n {
        let p = call(env, 0x0201_AB28, &[src, i]);
        let bm = party_get(mem, party, i & 0xFF);
        call(env, 0x021D_6C3C, &[bm, p]);
        let j = shows[i as usize];
        if j != 6 {
            let q = call(env, 0x0201_AB28, &[src, u32::from(j)]);
            call(env, 0x021D_5848, &[bm, q]);
        }
    }
}

/// C: `FUN_overlay_d_93__021b97a0` (pc, M, cl): for mon id `0x021EFCF0[cl] + i` (i below the size of src) the party member
/// with that id: `021d6b84(mon, 1)`, its Pokemon appended to the scratch party `M[0x2B8]` (cleared first), and
/// `setup[0xC2 + i] = 1` if `021d67d0(mon)`; then `0201AC2C(scratch, src)`.
pub fn pokecon_sync_ordered<E: FlowEnv>(env: &mut E, mem: &mut Mem, pc: u32, m: u32, cl: u32) {
    let src = pc_source(mem, pc, cl);
    let party = party_addr(pc, cl);
    let n = call(env, 0x0201_AA18, &[src]);
    call(env, 0x0201_A948, &[mem.r32(m + M_PARTY_SCRATCH)]);
    let base = mem.r8(FIRST_MON_SLOT_ADDR.wrapping_add(cl));
    for i in 0..n {
        let id = base.wrapping_add(i);
        for j in 0..n {
            let bm = party_get(mem, party, j & 0xFF);
            if id == call(env, 0x021D_5440, &[bm]) {
                call(env, 0x021D_6B84, &[bm, 1]);
                let pp = call(env, 0x021D_5844, &[bm]);
                call(env, 0x0201_A98C, &[mem.r32(m + M_PARTY_SCRATCH), pp]);
                if call(env, 0x021D_67D0, &[bm]) != 0 {
                    let s = setup(mem, m);
                    mem.w8(s.wrapping_add(i) + SET_C2, 1);
                }
                break;
            }
        }
    }
    call(env, 0x0201_AC2C, &[mem.r32(m + M_PARTY_SCRATCH), src]);
}

/// C: `FUN_overlay_d_93__021b985c`: the party source of client `cl`, `pc[0x74 + 4 * cl]`.
pub fn pc_source(mem: &Mem, pc: u32, cl: u32) -> u32 {
    mem.r32(pc.wrapping_add(cl << 2) + PC_SRC)
}

/// C: `FUN_overlay_d_93__021b9864`: index in party `cl` of the BattleMon whose id (`021d5440`) is `id`, else -1.
pub fn party_index_of_id<E: FlowEnv>(env: &mut E, mem: &Mem, pc: u32, cl: u32, id: u32) -> u32 {
    let party = pc
        .wrapping_add(PC_PARTIES)
        .wrapping_add(cl.wrapping_mul(PARTY_SIZE));
    let n = party_count(mem, party);
    for i in 0..n {
        let bm = party_get2(mem, party, i & 0xFF);
        if id == call(env, 0x021D_5440, &[bm]) {
            return i;
        }
    }
    u32::MAX
}

/// C: `FUN_overlay_d_93__021b98ac` (pc, pos): the BattleMon at battle position `pos`: client `c = 021b8cfc(M, pos)` (M =
/// `pc[0]`), `k` = number (u8) of the positions below `pos` (`(pos - 1) & 0xFF` down to 0) with client `c`; member `k` of party
/// `c`, or NULL when `k` is not below its count.
pub fn mon_at_position<E: FlowEnv>(env: &mut E, mem: &Mem, pc: u32, pos: u32) -> u32 {
    let m = mem.r32(pc + PC_M);
    let c = call(env, 0x021B_8CFC, &[m, pos]);
    let mut k = 0u32;
    let mut p = pos.wrapping_sub(1) & 0xFF;
    let mut prev = pos;
    while prev != 0 {
        if c == mem.r8(m.wrapping_add(p) + M_POS_CLIENT) {
            k = (k + 1) & 0xFF;
        }
        prev = p;
        p = p.wrapping_sub(1) & 0xFF;
    }
    let party = pc
        .wrapping_add(PC_PARTIES)
        .wrapping_add(c.wrapping_mul(PARTY_SIZE));
    if k >= party_count(mem, party) {
        return 0;
    }
    party_get2(mem, party, k)
}

/// C: `FUN_overlay_d_93__021b990c`: [`party_get`] on party `cl` of the pokecon (tail call).
pub fn pc_party_get(mem: &Mem, pc: u32, cl: u32, i: u32) -> u32 {
    party_get(
        mem,
        pc.wrapping_add(PC_PARTIES)
            .wrapping_add(cl.wrapping_mul(PARTY_SIZE)),
        i,
    )
}

/// C: `FUN_overlay_d_93__021b9920`: [`party_get2`] on party `cl` of the pokecon (tail call).
pub fn pc_party_get2(mem: &Mem, pc: u32, cl: u32, i: u32) -> u32 {
    party_get2(
        mem,
        pc.wrapping_add(PC_PARTIES)
            .wrapping_add(cl.wrapping_mul(PARTY_SIZE)),
        i,
    )
}

/// C: `FUN_overlay_d_93__021b9934`: the BattleMon of mon id `id`, `pc[0x84 + 4 * id]`.
pub fn pc_mon(mem: &Mem, pc: u32, id: u32) -> u32 {
    mem.r32(pc.wrapping_add(id << 2) + PC_MONS)
}

/// C: `FUN_overlay_d_93__021b9940`: same code as [`pc_mon`].
pub fn pc_mon2(mem: &Mem, pc: u32, id: u32) -> u32 {
    pc_mon(mem, pc, id)
}

// ---- main-module view objects / misc -------------------------------------------------------------------------------------

/// C: `FUN_021b994c`: the view object of the client at position `021b8ed8(M, x & 0xFF)` (client through the thunk 021b8de4 ->
/// 021b8cfc); if non-null it is stored to `*out` and 1 is returned, else 0.
pub fn view_obj_of<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32, x: u32, out: u32) -> u32 {
    let p = call(env, 0x021B_8ED8, &[m, x & 0xFF]);
    let c = call(env, 0x021B_8CFC, &[m, p]);
    let v = mem.r32(m.wrapping_add(c << 2) + M_VIEW_OBJS);
    if v != 0 {
        mem.w32(out, v);
        1
    } else {
        0
    }
}

/// C: `FUN_overlay_d_93__021b997c`: outside comm mode the view object `p = M[0x3E0 + 4 i]`: `02007B50(p) ? 02007C70(p) : 0`
/// (0 for NULL); in comm mode `setup[0x44 + i]`.
pub fn view_value<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, i: u32) -> u32 {
    if is_comm(mem, m) == 0 {
        let a = m.wrapping_add(M_VIEW_OBJS).wrapping_add(i << 2);
        let p = mem.r32(a);
        if p != 0 && call(env, 0x0200_7B50, &[p]) != 0 {
            return call(env, 0x0200_7C70, &[mem.r32(a)]);
        }
        0
    } else {
        mem.r8(setup(mem, m).wrapping_add(i) + SET_44)
    }
}

/// C: `FUN_overlay_d_93__021b99c0`: `r = 021b6994(M, x)`, plus 2 (u8) when `021b8588(M) == 3`.
pub fn adjusted_6994<E: FlowEnv>(env: &mut E, m: u32, x: u32) -> u32 {
    let mut r = call(env, 0x021B_6994, &[m, x]);
    if call(env, 0x021B_8588, &[m]) == 3 {
        r = (r + 2) & 0xFF;
    }
    r
}

/// C: `FUN_overlay_d_93__021b99e0`: address of record `i` of the 10-byte table at 0x021EFD70.
pub fn pos_list_record(i: u32) -> u32 {
    POS_LISTS_ADDR.wrapping_add(i.wrapping_mul(10))
}

/// C: `FUN_overlay_d_93__021b99f0`: `x == 2 || x == 3`.
pub fn is_pos_2_or_3(x: u32) -> u32 {
    u32::from(x == 2 || x == 3)
}

/// C: `sub_021b9a00`: unless `x` is 2 or 3: `*out = 021b8c20(2, x, (x >> 1) & 0xFF)` (a byte) and 1, else 0.
pub fn pos_8c20<E: FlowEnv>(env: &mut E, mem: &mut Mem, x: u32, out: u32) -> u32 {
    if is_pos_2_or_3(x) != 0 {
        return 0;
    }
    let v = call(env, 0x021B_8C20, &[2, x, (x >> 1) & 0xFF]);
    mem.w8(out, v);
    1
}

/// C: `sub_021b9a24`: the two mon ids are on the same side.
pub fn same_side(a: u32, b: u32) -> u32 {
    u32::from(side_of_id(a) == side_of_id(b))
}

/// C: `sub_021b9a40`: side of a BattleMon id (`id >= 12`).
pub fn side_of_id(id: u32) -> u32 {
    u32::from(id >= 12)
}

/// C: `FUN_overlay_d_93__021b9a50`: the other side of a mon id.
pub fn other_side_of_id(id: u32) -> u32 {
    is_zero(side_of_id(id))
}

/// C: `FUN_overlay_d_93__021b9a5c`: `x == 0`.
pub fn is_zero(x: u32) -> u32 {
    u32::from(x == 0)
}

/// C: `FUN_overlay_d_93__021b9a6c`: `v` is in list 0 (`n0` bytes from +4) or list 1 (`n1` bytes from +7) of record `i` of
/// 0x021EFD70.
pub fn pos_list_contains(mem: &Mem, i: u32, v: u32) -> u32 {
    let rec = pos_list_record(i);
    let n0 = mem.r16(rec);
    for j in 0..n0 {
        if v == mem.r8(rec + j + 4) {
            return 1;
        }
    }
    let mut j = 0;
    while j < mem.r16(rec + 2) {
        if v == mem.r8(rec + j + 7) {
            return 1;
        }
        j += 1;
    }
    0
}

/// C: `FUN_overlay_d_93__021b9ab0`: 2 -> 1, 3 -> 2, else 0.
pub fn swap_slot_a(x: u32) -> u32 {
    match x {
        2 => 1,
        3 => 2,
        _ => 0,
    }
}

/// C: `FUN_overlay_d_93__021b9ac8`: 2 -> 2, 3 -> 1, else 0.
pub fn swap_slot_b(x: u32) -> u32 {
    match x {
        2 => 2,
        3 => 1,
        _ => 0,
    }
}

/// C: `FUN_overlay_d_93__021b9ae0`: `x & 1`.
pub fn side_of_client(x: u32) -> u32 {
    x & 1
}

/// C: `FUN_overlay_d_93__021b9aec`: address of party container `cl` of the pokecon, `pc + 4 + cl * 0x1C`.
pub fn party_addr(pc: u32, cl: u32) -> u32 {
    pc.wrapping_add(PC_PARTIES)
        .wrapping_add(cl.wrapping_mul(PARTY_SIZE))
}

/// C: `FUN_overlay_d_93__021b9af8`: same code as [`party_addr`].
pub fn party_addr2(pc: u32, cl: u32) -> u32 {
    party_addr(pc, cl)
}

/// C: `FUN_overlay_d_93__021b9b04`: alive count of party `cl`.
pub fn pc_party_alive<E: FlowEnv>(env: &mut E, mem: &Mem, pc: u32, cl: u32) -> u32 {
    party_alive(env, mem, party_addr2(pc, cl))
}

/// C: `FUN_overlay_d_93__021b9b10`: if `M[0x46E] != 0`, the client exists (`021b8b9c(M, cl)`) and its object is non-null:
/// `021cd9f4(client, v)`.
pub fn client_call_d9f4<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, cl: u32, v: u32) {
    if mem.r8(m + M_46E) == 0 || call(env, 0x021B_8B9C, &[m, cl]) == 0 {
        return;
    }
    let c = client(mem, m, cl);
    if c != 0 {
        call(env, 0x021C_D9F4, &[c, v]);
    }
}

// ---- party container (0x1C bytes) ----------------------------------------------------------------------------------------

/// C: `FUN_overlay_d_93__021b9b40`: count 0, the 6 mon pointers NULL.
pub fn party_clear(mem: &mut Mem, p: u32) {
    mem.w8(p + PARTY_COUNT, 0);
    for i in 0..6 {
        mem.w32(p + i * 4, 0);
    }
}

/// C: `FUN_overlay_d_93__021b9b54`: `P.mon[P.count++] = mon` (no bound check; the count wraps as a byte).
pub fn party_append(mem: &mut Mem, p: u32, mon: u32) {
    let c = mem.r8(p + PARTY_COUNT);
    mem.w8(p + PARTY_COUNT, c + 1);
    mem.w32(p + c * 4, mon);
}

/// C: `FUN_overlay_d_93__021b9b60`: moves the dead mons (`021d5b7c(mon) == 0`) to the end, one 021b9c24 per dead slot, in
/// `count` steps (the index stays on a slot that was just replaced).
pub fn party_dead_to_end<E: FlowEnv>(env: &mut E, mem: &mut Mem, p: u32) {
    let mut i = 0u32;
    let mut k = 0u32;
    if mem.r8(p + PARTY_COUNT) == 0 {
        return;
    }
    loop {
        if call(env, 0x021D_5B7C, &[mem.r32(p + i * 4)]) != 0 {
            i += 1;
        } else {
            party_move_to_end(mem, p, i & 0xFF);
        }
        k += 1;
        if k >= mem.r8(p + PARTY_COUNT) {
            break;
        }
    }
}

/// C: `FUN_021B9B94`: the mon count.
pub fn party_count(mem: &Mem, p: u32) -> u32 {
    mem.r8(p + PARTY_COUNT)
}

/// C: `FUN_overlay_d_93__021b9b98`: number (u8) of mons with `021d5b7c(mon) != 0`.
pub fn party_alive<E: FlowEnv>(env: &mut E, mem: &Mem, p: u32) -> u32 {
    party_alive_from(env, mem, p, 0)
}

/// C: `FUN_overlay_d_93__021b9bc4`: number (u8) of mons from index `start` on with `021d5b7c(mon) != 0` (signed compare
/// with the count).
pub fn party_alive_from<E: FlowEnv>(env: &mut E, mem: &Mem, p: u32, start: u32) -> u32 {
    let mut n = 0u32;
    let mut i = start;
    while (i as i32) < mem.r8(p + PARTY_COUNT) as i32 {
        if call(env, 0x021D_5B7C, &[mem.r32(p.wrapping_add(i << 2))]) != 0 {
            n += 1;
        }
        i = i.wrapping_add(1);
    }
    n & 0xFF
}

/// C: `FUN_overlay_d_93__021b9bf0`: `P.mon[i]` if `i < count` (unsigned) else NULL.
pub fn party_get(mem: &Mem, p: u32, i: u32) -> u32 {
    if i < mem.r8(p + PARTY_COUNT) {
        mem.r32(p + (i << 2))
    } else {
        0
    }
}

/// C: `sub_021b9c00`: same code as [`party_get`].
pub fn party_get2(mem: &Mem, p: u32, i: u32) -> u32 {
    party_get(mem, p, i)
}

/// C: `BattleParty_SwapSlots` (`0x021B9C10`): swaps `P.mon[i]` and `P.mon[j]`.
pub fn party_swap(mem: &mut Mem, p: u32, i: u32, j: u32) {
    let a = p.wrapping_add(i << 2);
    let b = p.wrapping_add(j << 2);
    let x = mem.r32(a);
    let y = mem.r32(b);
    mem.w32(a, y);
    mem.w32(b, x);
}

/// C: `FUN_overlay_d_93__021b9c24`: moves the mon pointer of slot `i` of a party container to slot `count - 1`, shifting the
/// slots after it down by one. The loop compares signed (`count - 1` is -1 for an empty party: nothing moves, slot `i` is
/// written back unchanged) and keeps the index as a byte.
pub fn party_move_to_end(mem: &mut Mem, p: u32, i: u32) {
    let mut i = i;
    let moved = mem.r32(p + i * 4);
    let last = mem.r8(p + PARTY_COUNT) as i32 - 1;
    while (i as i32) < last {
        let next = mem.r32(p + i * 4 + 4);
        mem.w32(p + i * 4, next);
        i = (i + 1) & 0xFF;
    }
    mem.w32(p + i * 4, moved);
}

/// C: `FUN_overlay_d_93__021b9c50`: rotation for position `pos` (none for 0 / 1): with `a = 021b9ab0(pos)`, `b = 021b9ac8(pos)`:
/// `mon[0] <- mon[a]`, `mon[a] <- mon[b]`, `mon[b] <- old mon[0]`; the optional outs get the old and the new `mon[0]`.
pub fn party_rotate_for_pos(mem: &mut Mem, p: u32, pos: u32, out_old: u32, out_new: u32) {
    if pos == 0 || pos == 1 {
        return;
    }
    let old0 = mem.r32(p);
    let a = swap_slot_a(pos) << 2;
    let b = swap_slot_b(pos) << 2;
    let v = mem.r32(p + a);
    mem.w32(p, v);
    let v = mem.r32(p + b);
    mem.w32(p + a, v);
    mem.w32(p + b, old0);
    if out_old != 0 {
        mem.w32(out_old, old0);
    }
    if out_new != 0 {
        let v = mem.r32(p);
        mem.w32(out_new, v);
    }
}

/// C: `BattleParty_GetMonIndex` (`0x021B9C98`): index of the mon pointer `mon` in the party, or -1.
pub fn party_index_of(mem: &Mem, p: u32, mon: u32) -> u32 {
    let mut i = 0u32;
    while (i as i32) < mem.r8(p + PARTY_COUNT) as i32 {
        if mem.r32(p + (i << 2)) == mon {
            return i;
        }
        i += 1;
    }
    u32::MAX
}

/// C: `FUN_overlay_d_93__021b9cbc`: index of the mon whose id (`021d5440`) is `id`, or -1.
pub fn party_index_of_mon_id<E: FlowEnv>(env: &mut E, mem: &Mem, p: u32, id: u32) -> u32 {
    let mut i = 0u32;
    while (i as i32) < mem.r8(p + PARTY_COUNT) as i32 {
        if id == call(env, 0x021D_5440, &[mem.r32(p + (i << 2))]) {
            return i;
        }
        i += 1;
    }
    u32::MAX
}

/// C: `FUN_overlay_d_93__021b9ce8`: the first mon with `021d5b7c(mon) != 0`, or NULL.
pub fn party_first_alive<E: FlowEnv>(env: &mut E, mem: &Mem, p: u32) -> u32 {
    let mut i = 0u32;
    while (i as i32) < mem.r8(p + PARTY_COUNT) as i32 {
        let mon = mem.r32(p + (i << 2));
        if call(env, 0x021D_5B7C, &[mon]) != 0 {
            return mem.r32(p + (i << 2));
        }
        i += 1;
    }
    0
}

/// C: `FUN_overlay_d_93__021b9d10` (M, P, x): Illusion refresh of a party. `L` = last alive index (from the end, not below 1
/// unless the party has one mon); start = 3 when `021b8588(M) == 3`, else `021b8b78(M, x)`; for each member from start with
/// ability Illusion (`BattleMon_Get(mon, 0x11) == 0x95`): before `L` it disguises as the Pokemon of member `L`, otherwise its
/// disguise is cleared unless it shows `M[0x2C0]`; then `021d6c24(mon)` for every member.
pub fn party_refresh_illusion<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, p: u32, x: u32) {
    let mut last = mem.r8(p + PARTY_COUNT) as i32 - 1;
    while last > 0 {
        if call(
            env,
            0x021D_5B7C,
            &[mem.r32(p.wrapping_add((last as u32) << 2))],
        ) != 0
        {
            break;
        }
        last -= 1;
    }
    let mut i = if call(env, 0x021B_8588, &[m]) == 3 {
        3
    } else {
        call(env, 0x021B_8B78, &[m, x])
    };
    while (i as i32) < mem.r8(p + PARTY_COUNT) as i32 {
        let mon = mem.r32(p.wrapping_add(i << 2));
        if call(env, 0x021D_59B4, &[mon, 0x11]) == ABILITY_ILLUSION {
            if (i as i32) < last {
                let src = mem.r32(p.wrapping_add((last as u32) << 2));
                let pp = call(env, 0x021D_5844, &[src]);
                call(env, 0x021D_5848, &[mon, pp]);
            } else if call(env, 0x021D_5864, &[mon]) != mem.r32(m + M_ILLUSION_MON) {
                call(env, 0x021D_5854, &[mon]);
            }
        }
        i = i.wrapping_add(1);
    }
    let mut i = 0u32;
    while (i as i32) < mem.r8(p + PARTY_COUNT) as i32 {
        call(env, 0x021D_6C24, &[mem.r32(p + (i << 2))]);
        i += 1;
    }
}

// ---- player records in M (M + 0x20 + 0x28 * i) ---------------------------------------------------------------------------

/// C: `FUN_overlay_d_93__021b9db8`: clears the 4 records (+0x20 word, +0x2A u16, +0x2C word) and the 4 view-object words.
pub fn records_clear(mem: &mut Mem, m: u32) {
    for i in 0..4 {
        let r = m + i * REC_STRIDE;
        mem.w32(r + REC_PTR0, 0);
        mem.w16(r + REC_U16B, 0);
        mem.w32(r + REC_WORD, 0);
    }
    for i in 0..4 {
        mem.w32(m + i * 4 + M_VIEW_OBJS, 0);
    }
}

/// C: `FUN_overlay_d_93__021b9de8`: frees (`020307B0`) and nulls the non-null record pointers +0x20 / +0x24 and the view-object
/// words.
pub fn records_free<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32) {
    for i in 0..4 {
        let r = m + i * REC_STRIDE;
        for o in [REC_PTR0, REC_PTR1] {
            let v = mem.r32(r + o);
            if v != 0 {
                call(env, 0x0203_07B0, &[v]);
                mem.w32(r + o, 0);
            }
        }
    }
    for i in 0..4 {
        let a = m + i * 4 + M_VIEW_OBJS;
        let v = mem.r32(a);
        if v != 0 {
            call(env, 0x0203_07B0, &[v]);
            mem.w32(a, 0);
        }
    }
}

/// C: `FUN_overlay_d_93__021b9e38` (rec, a, b): `rec[0] = 0200846C(a)`, `02008494(b, rec[0])`, `rec+0xA = 0`,
/// `rec+8 = 02008550(rec[0]) != 0`, `rec[1] = 02008510(rec[0], 0x12)`, `0202428C(rec + 0x18)`, `0202428C(rec + 0x20)`.
pub fn record_fill_from<E: FlowEnv>(env: &mut E, mem: &mut Mem, rec: u32, a: u32, b: u32) {
    let v = call(env, 0x0200_846C, &[a]);
    mem.w32(rec, v);
    call(env, 0x0200_8494, &[b, v]);
    mem.w16(rec + 0xA, 0);
    let f = u32::from(call(env, 0x0200_8550, &[mem.r32(rec)]) != 0);
    mem.w16(rec + 8, f);
    let s = call(env, 0x0200_8510, &[mem.r32(rec), HEAP_BATTLE]);
    mem.w32(rec + 4, s);
    call(env, 0x0202_428C, &[rec + 0x18]);
    call(env, 0x0202_428C, &[rec + 0x20]);
}

/// C: `FUN_overlay_d_93__021b9e7c` (rec, src): `rec[0] = 0`; NULL src: empty record (u16s, `rec[1]`, the 4 u16 at +0x10
/// zero; `0202428C` on +0x18 / +0x20); else `+0xA = src[0]`, `+8 = src[1]`, `rec[1] = 02045874(src[0x14], 0x12)`,
/// `rec[3] = src[2]`, 8 bytes `src + 0xC -> rec + 0x10`, `02024570(rec + 0x18, src + 0x18)`, `02024570(rec + 0x20, src + 0x20)`.
pub fn record_fill_from_block<E: FlowEnv>(env: &mut E, mem: &mut Mem, rec: u32, src: u32) {
    mem.w32(rec, 0);
    if src == 0 {
        record_empty(env, mem, rec);
        return;
    }
    mem.w16(rec + 0xA, mem.r32(src));
    mem.w16(rec + 8, mem.r32(src + 4));
    let s = call(env, 0x0204_5874, &[mem.r32(src + 0x14), HEAP_BATTLE]);
    mem.w32(rec + 4, s);
    record_fill_tail(env, mem, rec, src);
}

fn record_empty<E: FlowEnv>(env: &mut E, mem: &mut Mem, rec: u32) {
    mem.w16(rec + 0xA, 0);
    mem.w16(rec + 8, 0);
    mem.w32(rec + 4, 0);
    for k in 0..4 {
        mem.w16(rec + 0x10 + 2 * k, 0);
    }
    call(env, 0x0202_428C, &[rec + 0x18]);
    call(env, 0x0202_428C, &[rec + 0x20]);
}

fn record_fill_tail<E: FlowEnv>(env: &mut E, mem: &mut Mem, rec: u32, src: u32) {
    mem.w32(rec + 0xC, mem.r32(src + 8));
    mem.copy8(src + 0xC, rec + 0x10, 8);
    call(env, 0x0202_4570, &[rec + 0x18, src + 0x18]);
    call(env, 0x0202_4570, &[rec + 0x20, src + 0x20]);
}

/// C: `FUN_overlay_d_93__021b9ef0`: as [`record_fill_from_block`], but `rec[1]` is a new StrBuf (`020457B0(0x20, 0x12)`)
/// filled by `02045A00(strbuf, src + 0x28, src[0x68] + 1)`.
pub fn record_fill_from_block_name<E: FlowEnv>(env: &mut E, mem: &mut Mem, rec: u32, src: u32) {
    mem.w32(rec, 0);
    if src == 0 {
        record_empty(env, mem, rec);
        return;
    }
    mem.w16(rec + 0xA, mem.r32(src));
    mem.w16(rec + 8, mem.r32(src + 4));
    let s = call(env, 0x0204_57B0, &[0x20, HEAP_BATTLE]);
    mem.w32(rec + 4, s);
    call(
        env,
        0x0204_5A00,
        &[s, src + 0x28, mem.r32(src + 0x68).wrapping_add(1)],
    );
    record_fill_tail(env, mem, rec, src);
}

fn rec(m: u32, i: u32) -> u32 {
    m.wrapping_add(i.wrapping_mul(REC_STRIDE))
}

/// C: `FUN_overlay_d_93__021b9f70`: record `i` is empty (`M[0x20 + 0x28 i] == NULL`).
pub fn record_empty_at(mem: &Mem, m: u32, i: u32) -> u32 {
    u32::from(mem.r32(rec(m, i) + REC_PTR0) == 0)
}

/// C: `FUN_overlay_d_93__021b9f84`: `u16 M[0x30 + 0x28 i + 2 k]` (k < 4) when [`record_empty_at`] says so, else 0.
pub fn record_u16x4(mem: &Mem, m: u32, i: u32, k: u32) -> u32 {
    if record_empty_at(mem, m, i) != 0 && k < 4 {
        mem.r16(rec(m, i) + 0x30 + (k << 1))
    } else {
        0
    }
}

/// C: `FUN_overlay_d_93__021b9fac`: `*out = u16 M[0x28 + 0x28 i]` (a word store); returns `M[0x24 + 0x28 i]`.
pub fn record_ptr1(mem: &mut Mem, m: u32, i: u32, out: u32) -> u32 {
    let r = rec(m, i);
    let v = mem.r16(r + REC_U16A);
    mem.w32(out, v);
    mem.r32(r + REC_PTR1)
}

/// C: `FUN_overlay_d_93__021b9fbc`: battle type 0 (`021b86b8`): 0x800 if `021ba1a8(M, 8)`, else 0x80 if `021b8588(M) == 1`,
/// else 0; other types: `M[0x2C + 0x28 i]` when [`record_empty_at`], else 0.
pub fn record_word<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, i: u32) -> u32 {
    if call(env, 0x021B_86B8, &[m]) == 0 {
        if setup_call_3158(env, mem, m, 8) != 0 {
            return 8 << 8;
        }
        if call(env, 0x021B_8588, &[m]) == 1 {
            return 0x80;
        }
        return 0;
    }
    if record_empty_at(mem, m, i) != 0 {
        mem.r32(rec(m, i) + REC_WORD)
    } else {
        0
    }
}

/// C: `FUN_overlay_d_93__021ba004`: `u16 M[0x2A + 0x28 i]` when [`record_empty_at`], else 0.
pub fn record_u16b(mem: &Mem, m: u32, i: u32) -> u32 {
    if record_empty_at(mem, m, i) != 0 {
        mem.r16(rec(m, i) + REC_U16B)
    } else {
        0
    }
}

/// C: `FUN_overlay_d_93__021ba020`: `u16 M[0x28 + 0x28 i]`.
pub fn record_u16a(mem: &Mem, m: u32, i: u32) -> u32 {
    mem.r16(rec(m, i) + REC_U16A)
}

/// C: `FUN_overlay_d_93__021ba02c`: battle type 2 and [`record_empty_at`]: `&M[0x40 + 0x28 i]` for `flag == 1`, else
/// `&M[0x38 + 0x28 i]`; otherwise NULL.
pub fn record_sub<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, i: u32, flag: u32) -> u32 {
    if call(env, 0x021B_86B8, &[m]) == 2 && record_empty_at(mem, m, i) != 0 {
        if flag == 1 {
            m.wrapping_add(0x40)
                .wrapping_add(i.wrapping_mul(REC_STRIDE))
        } else {
            m.wrapping_add(0x38)
                .wrapping_add(i.wrapping_mul(REC_STRIDE))
        }
    } else {
        0
    }
}

/// C: `FUN_overlay_d_93__021ba064`: 0 when [`record_empty_at`], else `M[0x20 + 0x28 i]`.
pub fn record_ptr0(mem: &Mem, m: u32, i: u32) -> u32 {
    if record_empty_at(mem, m, i) != 0 {
        0
    } else {
        mem.r32(rec(m, i) + REC_PTR0)
    }
}

/// C: `FUN_overlay_d_93__021ba080`: `0202E7DC(setup[0x70])` (tail call).
pub fn setup_call_e7dc<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32) -> u32 {
    call(env, 0x0202_E7DC, &[mem.r32(setup(mem, m) + SET_70)])
}

/// C: `FUN_overlay_d_93__021ba08c`: `M[0x2BC]`.
pub fn m_2bc(mem: &Mem, m: u32) -> u32 {
    mem.r32(m + M_2BC)
}

// ---- party copies M + 0x298.. --------------------------------------------------------------------------------------------

/// C: `FUN_overlay_d_93__021ba094`: allocates the parties `M[0x298 + 4 i]`, `M[0x2A8 + 4 i]` (i < 4, alternating) and
/// `M[0x2B8]` with `0201A920(0x12)`.
pub fn parties_alloc<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32) {
    for i in 0..4 {
        let a = call(env, 0x0201_A920, &[HEAP_BATTLE]);
        mem.w32(m + i * 4 + M_PARTY_A, a);
        let b = call(env, 0x0201_A920, &[HEAP_BATTLE]);
        mem.w32(m + i * 4 + M_PARTY_B, b);
    }
    let c = call(env, 0x0201_A920, &[HEAP_BATTLE]);
    mem.w32(m + M_PARTY_SCRATCH, c);
}

/// C: `FUN_overlay_d_93__021ba0cc`: frees (`020307B0`) and nulls the non-null party words allocated by [`parties_alloc`].
pub fn parties_free<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32) {
    for i in 0..4 {
        for o in [M_PARTY_A, M_PARTY_B] {
            let a = m + i * 4 + o;
            let v = mem.r32(a);
            if v != 0 {
                call(env, 0x0203_07B0, &[v]);
                mem.w32(a, 0);
            }
        }
    }
    let v = mem.r32(m + M_PARTY_SCRATCH);
    if v != 0 {
        call(env, 0x0203_07B0, &[v]);
        mem.w32(m + M_PARTY_SCRATCH, 0);
    }
}

/// C: `FUN_overlay_d_93__021ba128` (M, i, src): if party copy `M[0x298 + 4 i]` exists: both copies of client `i` cleared
/// (`0201A948`), `0201AC2C(src, copy)` into both, then `0201AB28(second, 0)` if the second is not empty.
pub fn parties_copy_in<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, i: u32, src: u32) {
    let ia = m.wrapping_add(i << 2).wrapping_add(M_PARTY_A);
    let ib = m.wrapping_add(i << 2).wrapping_add(M_PARTY_B);
    let a = mem.r32(ia);
    if a == 0 {
        return;
    }
    call(env, 0x0201_A948, &[a]);
    call(env, 0x0201_A948, &[mem.r32(ib)]);
    call(env, 0x0201_AC2C, &[src, mem.r32(ia)]);
    call(env, 0x0201_AC2C, &[src, mem.r32(ib)]);
    if call(env, 0x0201_AA18, &[mem.r32(ib)]) != 0 {
        call(env, 0x0201_AB28, &[mem.r32(ib), 0]);
    }
}

/// C: `FUN_overlay_d_93__021ba178` (M, i, flag): the party source of client `i`: with `flag`, the second copy
/// `M[0x2A8 + 4 i]` (after `0201AB28(p, 0)` when it is not empty), else `M[0x298 + 4 i]`.
pub fn party_source<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, i: u32, flag: u32) -> u32 {
    if flag != 0 {
        let a = m.wrapping_add(M_PARTY_B).wrapping_add(i << 2);
        if call(env, 0x0201_AA18, &[mem.r32(a)]) != 0 {
            call(env, 0x0201_AB28, &[mem.r32(a), 0]);
        }
        mem.r32(a)
    } else {
        mem.r32(m.wrapping_add(i << 2).wrapping_add(M_PARTY_A))
    }
}

/// C: `FUN_overlay_d_93__021ba1a8`: `02013158(*M, x)` (tail call).
pub fn setup_call_3158<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, x: u32) -> u32 {
    call(env, 0x0201_3158, &[setup(mem, m), x])
}

/// C: `FUN_overlay_d_93__021ba1b4`: outside comm mode `02008DF0(setup[0x6C], x)`.
pub fn setup_call_8df0<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, x: u32) {
    let s = setup(mem, m);
    if mem.r8(s + SET_COMM) == 0 {
        call(env, 0x0200_8DF0, &[mem.r32(s + SET_6C), x]);
    }
}

/// C: `FUN_overlay_d_93__021ba1cc`: outside comm mode `02008DAC(setup[0x6C], x, y)`.
pub fn setup_call_8dac<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, x: u32, y: u32) {
    let s = setup(mem, m);
    if mem.r8(s + SET_COMM) == 0 {
        call(env, 0x0200_8DAC, &[mem.r32(s + SET_6C), x, y]);
    }
}

/// C: `FUN_overlay_d_93__021ba1e4`: `&setup[0x78]`.
pub fn setup_78(mem: &Mem, m: u32) -> u32 {
    setup(mem, m).wrapping_add(0x78)
}

/// C: `FUN_overlay_d_93__021ba1ec`: `021b6818(setup) == 1 && setup[0x7F] != 0`.
pub fn setup_7f_flag<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32) -> u32 {
    let r = call(env, 0x021B_6818, &[setup(mem, m)]) == 1 && mem.r8(setup(mem, m) + SET_7F) != 0;
    u32::from(r)
}

/// C: `FUN_overlay_d_93__021ba20c`: `021b6818(setup) == 1` ? [`setup_7f_flag`] : battle type (`021b86b8`) 0 or 1.
pub fn setup_mode_flag<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32) -> u32 {
    if call(env, 0x021B_6818, &[setup(mem, m)]) == 1 {
        return setup_7f_flag(env, mem, m);
    }
    let t = call(env, 0x021B_86B8, &[m]);
    u32::from(t == 0 || t == 1)
}

/// C: `FUN_overlay_d_93__021ba238`: [`mon_pokemon_notify`]`(M, mon, 0)`, then `021bb8bc(S = M[8], mon)`.
pub fn mon_notify_server<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, mon: u32) {
    mon_pokemon_notify(env, mem, m, mon, 0);
    call(env, 0x021B_B8BC, &[mem.r32(m + M_SERVER), mon]);
}

/// C: `FUN_overlay_d_93__021ba250`: as [`mon_pokemon_notify`] with `0201B7B4`.
pub fn mon_pokemon_notify_b7b4<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, mon: u32, x: u32) {
    for_both_pokemon(env, mem, m, mon, x, 0x0201_B7B4);
}

/// C: `FUN_021ba2d4`: `S = M[8]`; non-null: `021bb8dc(S, x)`.
pub fn server_call_b8dc<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, x: u32) {
    let s = mem.r32(m + M_SERVER);
    if s != 0 {
        call(env, 0x021B_B8DC, &[s, x]);
    }
}

// ---- end of battle -------------------------------------------------------------------------------------------------------

/// C: `FUN_overlay_d_93__021ba2e4`: writes the battle's party results back into the setup (outside comm mode only).
/// Types 0 / 1: [`pokecon_sync_ordered`] of the own client in pokecon 1; type 1 with `021d7f60(record u16 of client 1)`:
/// `0201B818(party, 021b8714(M)[0xA], heap | 0x8000)`; type 1: per member the held item (`02017E1C(pp, 6, 0)`) of the setup's
/// party `setup[0x24]` is given to the battle party when `021d805c(item, 0x10) == 0`, else a different battle-party item is
/// cleared (`02017E40(pp, 6, ..)`); types 0 / 1: the party copied into `setup[0x24]` (0201AC2C) and every member touched
/// (0201AB28). Type 0: [`pokecon_sync_back`] of the partner client `021b9064(M, own, 0)` and its party copied into
/// `setup[0x28]`. Unless type 4, per relation `k = 021ba710(own, c)` (c < 4): `setup[0xC8 + 6 k ..+6]` zeroed and, when client
/// k exists, the status byte per mon (2 if `021d5b68`, 1 if `021d62a8`) and `setup[0xE0 + 4 k] = 100 * sum(Get 0xD) /
/// sum(Get 0xE)` over pokecon 0's party k (`_u32_div_f`: a zero divisor gives the dividend).
pub fn write_back_results<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32) {
    let s = setup(mem, m);
    if mem.r8(s + SET_COMM) != 0 {
        return;
    }
    if mem.r32(s + SET_TYPE) <= 1 {
        let pc1 = m + M_PC1;
        pokecon_sync_ordered(env, mem, pc1, m, own_client(mem, m));
        let party = pc_source(mem, pc1, own_client(mem, m));
        if mem.r32(setup(mem, m) + SET_TYPE) == 1 {
            let u = record_u16a(mem, m, 1) & 0xFFFF;
            if call(env, 0x021D_7F60, &[u]) != 0 {
                let st = call(env, 0x021B_8714, &[m]);
                call(
                    env,
                    0x0201_B818,
                    &[party, mem.r16(st + 0xA), heap_flag(mem, m)],
                );
            }
        }
        if mem.r32(setup(mem, m) + SET_TYPE) == 1 {
            let n = call(env, 0x0201_AA18, &[party]);
            for i in 0..n {
                let a = call(env, 0x0201_AB28, &[mem.r32(setup(mem, m) + SET_PARTIES), i]);
                let b = call(env, 0x0201_AB28, &[party, i]);
                let item = call(env, 0x0201_7E1C, &[a, 6, 0]) & 0xFFFF;
                if call(env, 0x021D_805C, &[item, 0x10]) == 0 {
                    call(env, 0x0201_7E40, &[b, 6, item]);
                } else if item != call(env, 0x0201_7E1C, &[b, 6, 0]) {
                    call(env, 0x0201_7E40, &[b, 6, 0]);
                }
            }
        }
        call(
            env,
            0x0201_AC2C,
            &[party, mem.r32(setup(mem, m) + SET_PARTIES)],
        );
        let mut i = 0u32;
        if call(env, 0x0201_AA18, &[party]) != 0 {
            loop {
                call(env, 0x0201_AB28, &[party, i]);
                i += 1;
                if i >= call(env, 0x0201_AA18, &[party]) {
                    break;
                }
            }
        }
    }
    if mem.r32(setup(mem, m) + SET_TYPE) == 0 {
        let k = pick_client(env, m, own_client(mem, m), 0) & 0xFF;
        let pc1 = m + M_PC1;
        pokecon_sync_back(env, mem, pc1, m, k, 1);
        let src = pc_source(mem, pc1, k);
        call(env, 0x0201_AC2C, &[src, mem.r32(setup(mem, m) + SET_28)]);
    }
    if mem.r32(setup(mem, m) + SET_TYPE) == 4 {
        return;
    }
    for c in 0..4u32 {
        let k = client_relation(own_client(mem, m), c & 0xFF);
        let row = k.wrapping_mul(6);
        for j in 0..6 {
            let s = setup(mem, m);
            mem.w8(s.wrapping_add(row).wrapping_add(j) + SET_STATUS, 0);
        }
        if call(env, 0x021B_8B9C, &[m, k & 0xFF]) == 0 {
            continue;
        }
        let party = party_addr(m + M_PC0, k);
        let n = party_count(mem, party);
        let base = mem.r8(FIRST_MON_SLOT_ADDR.wrapping_add(k));
        let mut s13 = 0u32;
        let mut s14 = 0u32;
        for i in 0..n {
            let mon = party_get(mem, party, i & 0xFF);
            let slot = call(env, 0x021D_5440, &[mon]).wrapping_sub(base) & 0xFF;
            s13 = s13.wrapping_add(call(env, 0x021D_59B4, &[mon, 0xD]));
            s14 = s14.wrapping_add(call(env, 0x021D_59B4, &[mon, 0xE]));
            let st = if call(env, 0x021D_5B68, &[mon]) != 0 {
                2
            } else if call(env, 0x021D_62A8, &[mon]) != 0 {
                1
            } else {
                continue;
            };
            let s = setup(mem, m);
            mem.w8(s.wrapping_add(row).wrapping_add(slot) + SET_STATUS, st);
        }
        let pct = crate::calc::u32_div(s13.wrapping_mul(100), s14);
        let s = setup(mem, m);
        mem.w32(s.wrapping_add(k << 2) + SET_E0, pct);
    }
}

/// C: `FUN_overlay_d_93__021ba534`: battle type (`021b86b8`) 2 or 3.
pub fn is_type_2_or_3<E: FlowEnv>(env: &mut E, m: u32) -> u32 {
    let t = call(env, 0x021B_86B8, &[m]);
    u32::from(t == 2 || t == 3)
}

/// C: `FUN_overlay_d_93__021ba54c`: the battle result code, also stored to `setup[0x8C]`: 1 for battle type 4, 6 when flag bit
/// 0 is set, 5 when `setup[0x90] != 6`, `021d8270(M + 0x448, own, type)` when `021d826c(M + 0x448)`, else `M[0x444]` (7 =
/// none becomes 1).
pub fn battle_result<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32) -> u32 {
    let r = if call(env, 0x021B_86B8, &[m]) == 4 {
        1
    } else if mem.r8(m + M_FLAGS) & 1 != 0 {
        6
    } else if mem.r8(setup(mem, m) + SET_90) != 6 {
        5
    } else if call(env, 0x021D_826C, &[m + M_448]) != 0 {
        let t = call(env, 0x021B_86B8, &[m]);
        call(env, 0x021D_8270, &[m + M_448, own_client(mem, m), t])
    } else {
        if mem.r32(m + M_RESULT) == 7 {
            mem.w32(m + M_RESULT, 1);
        }
        mem.r32(m + M_RESULT)
    };
    let s = setup(mem, m);
    mem.w32(s + SET_RESULT, r);
    r
}

/// C: `FUN_021ba5c4`: `u16 M[0x442]`.
pub fn m_442(mem: &Mem, m: u32) -> u32 {
    mem.r16(m + M_442)
}

/// C: `FUN_021ba5d0`: `u16 M[0x440]`.
pub fn m_440(mem: &Mem, m: u32) -> u32 {
    mem.r16(m + M_440)
}

/// C: `FUN_overlay_d_93__021ba5d8`: `021cdda0(own client object)` in comm mode, or outside it when `M[0x440] != 0`; else 0.
pub fn own_client_dda0<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32) -> u32 {
    if mem.r8(setup(mem, m) + SET_COMM) == 0 && mem.r16(m + M_440) == 0 {
        return 0;
    }
    call(env, 0x021C_DDA0, &[client(mem, m, own_client(mem, m))])
}

/// C: `FUN_overlay_d_93__021ba618`: in comm mode `r = 021cddcc(own client object)`; non-zero r sets `M[0x444] = 2`.
/// Returns r (0 outside comm mode).
pub fn own_client_ddcc<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32) -> u32 {
    if mem.r8(setup(mem, m) + SET_COMM) == 0 {
        return 0;
    }
    let r = call(env, 0x021C_DDCC, &[client(mem, m, own_client(mem, m))]);
    if r != 0 {
        mem.w32(m + M_RESULT, 2);
    }
    r
}

/// C: `FUN_overlay_d_93__021ba648`: `k = 021ba710(own, cl)`; `p = setup[0x24 + 4 k]` non-null: `0201AC2C(x, p)`.
pub fn copy_to_setup_party<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, cl: u32, x: u32) {
    let k = client_relation(own_client(mem, m), cl);
    let p = mem.r32(setup(mem, m).wrapping_add(k << 2) + SET_PARTIES);
    if p != 0 {
        call(env, 0x0201_AC2C, &[x, p]);
    }
}

/// C: `FUN_overlay_d_93__021ba670`: `k = 021ba710(own, cl)`; `p = setup[0x34 + 4 k]` non-null: `02008494(x, p)`.
pub fn copy_to_setup_34<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, cl: u32, x: u32) {
    let k = client_relation(own_client(mem, m), cl);
    let p = mem.r32(setup(mem, m).wrapping_add(k << 2) + SET_34);
    if p != 0 {
        call(env, 0x0200_8494, &[x, p]);
    }
}

/// C: `FUN_overlay_d_93__021ba698`: `setup[0x44 + 021ba710(own, cl)] = `[`view_value`]`(M, cl)`.
pub fn store_view_value<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32, cl: u32) {
    let k = client_relation(own_client(mem, m), cl);
    let v = view_value(env, mem, m, cl);
    let s = setup(mem, m);
    mem.w8(s.wrapping_add(k) + SET_44, v);
}

/// C: `FUN_overlay_d_93__021ba6c0`: when recording locally ([`records_locally`]) and `src = 021cd924(own client, &size)` is
/// non-null: `size` bytes from src to the record buffer `setup[0x94]` (MI_CpuCopy8), `setup[0x98] = size`, and the 0x18
/// bytes `M[0x3F0..]` to `setup[0x9C..]`.
pub fn save_record<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32) {
    if records_locally(mem, m) == 0 {
        return;
    }
    let mut size = [0u8; 4];
    let c = client(mem, m, own_client(mem, m));
    let src = env.call(0x021C_D924, &mut [Arg::V(c), Arg::Out(&mut size)]);
    if src == 0 {
        return;
    }
    let size = u32::from_le_bytes(size);
    let dst = mem.r32(setup(mem, m) + SET_RECORD);
    mem.copy8(src, dst, size);
    let s = setup(mem, m);
    mem.w32(s + SET_RECORD_SIZE, size);
    let s = setup(mem, m);
    for k in 0..6 {
        let v = mem.r32(m + M_3F0 + 4 * k);
        mem.w32(s + SET_9C + 4 * k, v);
    }
}

/// C: `FUN_overlay_d_93__021ba710`: relation of client `b` to client `a`: 0 the same client, 2 the same side (equal bit 0),
/// otherwise 1 when `b <= 1` and 3 when `b > 1`.
pub fn client_relation(a: u32, b: u32) -> u32 {
    if a == b {
        0
    } else if a & 1 == b & 1 {
        2
    } else if b <= 1 {
        1
    } else {
        3
    }
}

// ---- battle end / module teardown ----------------------------------------------------------------------------------------

/// C: `FUN_overlay_d_93__021ba73c` (M, x): rebuilds the battle for a new round: `st = 021b6818(setup)`, `021d787c(M + 0x3F0)`,
/// `021b68d8(M, setup)`, both pokecons freed (0 then 1); for every client c that exists: [`pokecon_setup_client`] in pokecon 1
/// and pokecon 0 and `021cd984(client, x)`; the view recreated (`021e9034(M[4])`, `021e8e28(0x13)`, `M[4] = 021e8f20(M, own
/// client, pokecon 0, st, 0x13)`), `021cd95c(own client)`, `021bab40(S)`.
pub fn rebuild_battle<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32, x: u32) {
    let st = call(env, 0x021B_6818, &[setup(mem, m)]);
    call(env, 0x021D_787C, &[m + M_3F0]);
    call(env, 0x021B_68D8, &[m, setup(mem, m)]);
    pokecon_free(env, mem, m + M_PC0);
    pokecon_free(env, mem, m + M_PC1);
    for c in 0..4 {
        if call(env, 0x021B_8B9C, &[m, c]) != 0 {
            pokecon_setup_client(env, mem, m + M_PC1, m, c);
            pokecon_setup_client(env, mem, m + M_PC0, m, c);
            call(env, 0x021C_D984, &[client(mem, m, c), x]);
        }
    }
    call(env, 0x021E_9034, &[mem.r32(m + 4)]);
    call(env, 0x021E_8E28, &[0x13]);
    let own = mem.r8(setup(mem, m) + SET_OWN_CLIENT);
    let v = call(
        env,
        0x021E_8F20,
        &[m, client(mem, m, own), m + M_PC0, st, 0x13],
    );
    mem.w32(m + 4, v);
    let own = mem.r8(setup(mem, m) + SET_OWN_CLIENT);
    call(env, 0x021C_D95C, &[client(mem, m, own)]);
    call(env, 0x021B_AB40, &[mem.r32(m + M_SERVER)]);
}

/// C: `FUN_overlay_d_93__021ba7f8`: `021cd9b0(client)` for every client that exists (`021b8b9c`).
pub fn clients_call_d9b0<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32) {
    for c in 0..4 {
        if call(env, 0x021B_8B9C, &[m, c]) != 0 {
            call(env, 0x021C_D9B0, &[client(mem, m, c)]);
        }
    }
}

/// C: `FUN_overlay_d_93__021ba820`: `021bb964(M[8])` (tail call; the server's flow work).
pub fn server_flow_work<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32) -> u32 {
    call(env, 0x021B_B964, &[mem.r32(m + M_SERVER)])
}

/// C: `FUN_overlay_d_93__021ba82c` (setup): clears the statistics bytes `setup[0xB6..=0xC0]` and `u16 setup[0xB4]`.
pub fn clear_stats(mem: &mut Mem, s: u32) {
    for o in 0xB6..=0xC0 {
        mem.w8(s + o, 0);
    }
    mem.w16(s + 0xB4, 0);
}

/// C: `FUN_overlay_d_93__021ba878`: statistics into the setup (nothing without a server object): from the flow work
/// `w = 021bb964(S)`: `setup[0xB6] = 021c8138(w)`, `[0xB7] = 021c8140(w, 0)`, `[0xB8..=0xBC] = 021c7cbc / 7cc8 / 7cd4 / 7cdc /
/// 7ce8(w)`; `[0xBD] = 1` when `setup[0x8C] == 1`; over pokecon 1's party 0: `[0xC0] += 021d54b4(mon)`, `u16 [0xB4] += Get(mon,
/// 0xD)`, `[0xBF] = count - alive`; `[0xBE] = count - alive` of party 1, plus that of party 3 when client 3 exists.
pub fn collect_stats<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32) {
    let srv = mem.r32(m + M_SERVER);
    if srv == 0 {
        return;
    }
    let w = call(env, 0x021B_B964, &[srv]);
    let s = setup(mem, m);
    let v = call(env, 0x021C_8138, &[w]);
    mem.w8(s + 0xB6, v);
    let v = call(env, 0x021C_8140, &[w, 0]);
    mem.w8(s + 0xB7, v);
    for (o, f) in [
        (0xB8, 0x021C_7CBC),
        (0xB9, 0x021C_7CC8),
        (0xBA, 0x021C_7CD4),
        (0xBB, 0x021C_7CDC),
        (0xBC, 0x021C_7CE8),
    ] {
        let v = call(env, f, &[w]);
        mem.w8(s + o, v);
    }
    if mem.r32(s + SET_RESULT) == 1 {
        mem.w8(s + 0xBD, 1);
    }
    let p0 = party_addr(m + M_PC1, 0);
    let n = party_count(mem, p0);
    let alive = party_alive(env, mem, p0);
    for i in 0..n {
        let mon = party_get(mem, p0, i & 0xFF);
        let v = call(env, 0x021D_54B4, &[mon]);
        let c0 = mem.r8(s + 0xC0);
        mem.w8(s + 0xC0, c0.wrapping_add(v));
        let hp = call(env, 0x021D_59B4, &[mon, 0xD]) & 0xFFFF;
        let b4 = mem.r16(s + 0xB4);
        mem.w16(s + 0xB4, b4 + hp);
    }
    mem.w8(s + 0xBF, n.wrapping_sub(alive));
    let p1 = party_addr(m + M_PC1, 1);
    let d = party_count(mem, p1).wrapping_sub(party_alive(env, mem, p1));
    mem.w8(s + 0xBE, d);
    if call(env, 0x021B_8B9C, &[m, 3]) == 0 {
        return;
    }
    let p3 = party_addr(m + M_PC1, 3);
    if p3 == 0 {
        return;
    }
    let d = party_count(mem, p3).wrapping_sub(party_alive(env, mem, p3)) & 0xFF;
    let v = mem.r8(s + 0xBE);
    mem.w8(s + 0xBE, v + d);
}

// ---- btl_server.c: the server object S (0xCC4 bytes) ---------------------------------------------------------------------

/// Size of the server object S.
const SERVER_SIZE: u32 = 0xCC4;

/// C: `FUN_overlay_d_93__021ba9c8` (M, src18, r2, r3, heap): creates the server object
/// `S = Heap_AllocDebug(heap, 0xCC4, 1, "btl_server.c", 0xAD)`: `S+0xC = M`, `S+0x10 = r2`, `u16 S+0xCC0 = heap`,
/// `S+0xCB0 = S + 0xF0`, `S+0xED = 0`, `S+0x44 = 0`, `S+0xCBA = 0`, `S+0xCBF = 0`, `S+0x4C = r3`, the 0x18 bytes of src18 at
/// `S+0x50`, `S+0xAC = 020457B0(0x20, heap)`; `021d787c(src18)`; the 4 client slots `021bb828(S + 0x14 + 12 i)`;
/// `S+0x44 = 021bb968(S, M, S[0x10], S + 0xF0, S[0x4C], heap)`; `S+0xB0 = 021bc550()`; `021babec(S, 0x021BAC05)`. Returns S.
pub fn server_create<E: FlowEnv>(
    env: &mut E,
    mem: &mut Mem,
    m: u32,
    src: u32,
    r2: u32,
    r3: u32,
    heap: u32,
) -> u32 {
    let heap = heap & 0xFFFF;
    let s = env.call(
        0x0203_0734,
        &mut [
            Arg::V(heap),
            Arg::V(SERVER_SIZE),
            Arg::V(1),
            // the heap-debug file name `"btl_server.c"` (0x021F3A60); the line number follows
            Arg::P,
            Arg::V(0xAD),
        ],
    );
    mem.w32(s + 0xC, m);
    mem.w32(s + 0x10, r2);
    mem.w16(s + SERVER_SIZE - 4, heap);
    mem.w32(s + SERVER_SIZE - 0x14, s + 0xF0);
    mem.w8(s + 0xED, 0);
    mem.w32(s + 0x44, 0);
    mem.w8(s + SERVER_SIZE - 0xA, 0);
    mem.w8(s + SERVER_SIZE - 5, 0);
    mem.w32(s + 0x4C, r3);
    for k in 0..6 {
        let v = mem.r32(src + 4 * k);
        mem.w32(s + 0x50 + 4 * k, v);
    }
    let sb = call(env, 0x0204_57B0, &[0x20, heap]);
    mem.w32(s + 0xAC, sb);
    call(env, 0x021D_787C, &[src]);
    for i in 0..4 {
        call(env, 0x021B_B828, &[s + 0x14 + 12 * i]);
    }
    let w = call(
        env,
        0x021B_B968,
        &[
            s,
            mem.r32(s + 0xC),
            mem.r32(s + 0x10),
            s + 0xF0,
            mem.r32(s + 0x4C),
            mem.r16(s + SERVER_SIZE - 4),
        ],
    );
    mem.w32(s + 0x44, w);
    let v = env.call(0x021B_C550, &mut []);
    mem.w32(s + 0xB0, v);
    call(env, 0x021B_ABEC, &[s, 0x021B_AC05]);
    s
}

#[cfg(test)]
mod tests;
