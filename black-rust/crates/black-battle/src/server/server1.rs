//! `battle_server.s` part 1 (0x021B6100-0x021B75A8): BattleProc init / main / exit, sub-proc selection, setup sub-procs (B11).
//!
//! Specs: `tools/rom-diff/svflow_specs/server1.spec` (`nowork`; M is `arg buf 0x1000`, its BATTLE_SETUP at arena 0x480).
//! Every function takes the address `m` of the main module M and works through a [`Mem`] view; callees in other areas
//! (server2 / server3 / server_side / client / adapter: `force` stubs) and outside the battle server (ARM9, view, battle_net,
//! BattleRecordReader) go through [`FlowEnv::call`]; calls inside this area are direct. Function-pointer fields (the sub-proc
//! slot `M + 0x450`, the main loop `M + 0x464`, the step tables at 0x021EFD00) are called as `env.call(ptr & !1, ..)`.
//!
//! Glue (GameProc / heap / overlay / hardware, D3): 6100 (heap creation `0203064C`, the M allocation `0203159C` + clear
//! `02082A7C`, display / sound setup), 6500 (fades, master brightness on the 2D engines `0207D534` / `0207D55C` with the
//! registers 0x0400006C / 0x0400106C, heap teardown `020306F0`, proc free `020315D4`): the logic is ported, the hardware and
//! heap calls stay behind the trait.

use super::server2::rom_word;
use super::Mem;
use crate::server_flow::env::{Arg, FlowEnv};

/// Functions of this area that are ported and checked by `tests::svflow_match_rom`.
pub const PORTED: &[u32] = &[
    0x021B_6100,
    0x021B_6434,
    0x021B_64CC,
    0x021B_6500,
    0x021B_66D4,
    0x021B_6804,
    0x021B_6818,
    0x021B_6848,
    0x021B_68D8,
    0x021B_6954,
    0x021B_6994,
    0x021B_69FC,
    0x021B_6A3C,
    0x021B_6A70,
    0x021B_6AAC,
    0x021B_6C08,
    0x021B_6C5C,
    0x021B_6DD0,
    0x021B_7050,
    0x021B_71D4,
    0x021B_734C,
    0x021B_73C4,
    0x021B_7488,
    0x021B_7518,
];

/// Thumb address of the exit sub-proc `021b6c08` (frees the battle objects).
const EXIT_SUBPROC: u32 = 0x021B_6C09;
/// Thumb address of the main-loop function 021b83f4 (local battles: server and clients on this console).
const MAIN_LOOP_LOCAL: u32 = 0x021B_83F5;

/// M: battle view.
const M_VIEW: u32 = 4;
/// M: server objects (server console / client console).
const M_SERVER: u32 = 8;
/// M: the 4 client objects.
const M_CLIENTS: u32 = 0x10;
/// M: word = setup + 0x34.
const M_C0: u32 = 0xC0;
/// M: 0x18-byte block handed to the server / client creates.
const M_BLOCK18: u32 = 0x3F0;
/// M: u8[6] battle position -> client id.
const M_POS_CLIENT: u32 = 0x428;
/// M: the sub-proc slot `{+0 fn, +4 fn 2, +8 arg, +0xC state}`.
const M_SLOT: u32 = 0x450;
/// M: sub-state / brightness word.
const M_460: u32 = 0x460;
/// M: main-loop function `fn(M)`.
const M_MAIN_FN: u32 = 0x464;
/// M: u16 heap id.
const M_HEAP: u32 = 0x468;
/// M: u8 client count.
const M_CLIENT_COUNT: u32 = 0x46B;
/// M: u8 own client id.
const M_OWN_ID: u32 = 0x46C;
/// M: u8 own client's position index (8bec).
const M_OWN_POS: u32 = 0x46D;
/// M: u8 "this console runs the server".
const M_IS_SERVER: u32 = 0x46E;
/// M: flag bits.
const M_FLAGS: u32 = 0x473;

/// The BATTLE_SETUP pointer `*M`.
fn setup(mem: &Mem, m: u32) -> u32 {
    mem.r32(m)
}

/// The u16 heap id `M[0x468]`.
fn heap(mem: &Mem, m: u32) -> u32 {
    u32::from(mem.r16(m + M_HEAP))
}

/// Runs the callback pair of the sub-proc slot `M + 0x450` once: the first function (`fn(M + 0x45C, M[0x458])`) while set
/// (done -> it and the state word are cleared; the pass returns false), then the second one; true once both are done.
fn run_slot<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32) -> bool {
    let f = mem.r32(m + M_SLOT);
    if f != 0 {
        let a = mem.r32(m + M_SLOT + 8);
        if env.call(f & !1, &mut [Arg::PAt(m + M_SLOT + 0xC), Arg::V(a)]) != 0 {
            mem.w32(m + M_SLOT, 0);
            mem.w32(m + M_SLOT + 0xC, 0);
        }
        return false;
    }
    let g = mem.r32(m + M_SLOT + 4);
    if g == 0 {
        return true;
    }
    let a = mem.r32(m + M_SLOT + 8);
    if env.call(g & !1, &mut [Arg::PAt(m + M_SLOT + 0xC), Arg::V(a)]) != 0 {
        mem.w32(m + M_SLOT + 4, 0);
        return true;
    }
    false
}

/// C: `FUN_overlay_93__021b6100`: BattleProc init (GameProc init: `proc`, `&seq`, the BATTLE_SETUP, M from the proc work).
/// seq 0: creates the heaps 0x12 / 0x14 / 0x13, allocates M (`0203159C(proc, 0x474, 0x12)`) and clears it, sets up the
/// view / sound side (`021e8dc4`, `021e88cc`, `020057c4` / `020057ac` -> flag bit 7), copies the setup fields into M (see the
/// module notes), inits both party containers, the comm block `M + 0x3F0` (`0203F258` or a copy of `setup + 0x9C`), the
/// client module (`021d7840`, `021cd6ec`), the per-client 0x44 table of local battles and the comm layer (`021d4150`,
/// `021d804c`); seq++. seq 1: comm battles with an error (`021d4270`) set flag bits 0 and 1 and return 1; when `021d4230()`:
/// adapter init (`021eea30(setup[0x20])`), the setup sub-proc (66d4), `M[0x460] = 0`, `021b9db8` / `021ba094` (server3);
/// seq++. seq 2: runs the sub-proc slot; when done: flag bit 0 -> bit 1 and 1, else (battle type != 4) `021ba1b4(M, 4)` and 1.
/// Returns 0 while not done.
pub fn proc_init<E: FlowEnv>(
    env: &mut E,
    mem: &mut Mem,
    proc: u32,
    seq: u32,
    setup_p: u32,
    m: u32,
) -> u32 {
    match mem.r32(seq) {
        0 => {
            env.call(0x0203_064C, &mut [Arg::V(1), Arg::V(0x12), Arg::V(0x20000)]);
            env.call(0x0203_064C, &mut [Arg::V(1), Arg::V(0x14), Arg::V(0x8000)]);
            env.call(0x0203_064C, &mut [Arg::V(1), Arg::V(0x13), Arg::V(0xAB100)]);
            let m = env.call(
                0x0203_159C,
                &mut [Arg::V(proc), Arg::V(0x474), Arg::V(0x12)],
            );
            env.call(0x0208_2A7C, &mut [Arg::V(0), Arg::P, Arg::V(0x474)]);
            env.call(0x021E_8DC4, &mut [Arg::V(0x13)]);
            env.call(0x021E_88CC, &mut []);
            if env.call(0x0200_57C4, &mut []) == 0 {
                env.call(0x0200_57AC, &mut []);
                let f = mem.r8(m + M_FLAGS);
                mem.w8(m + M_FLAGS, f | 0x80);
            }
            mem.w16(m + M_HEAP, 0x12);
            mem.w32(m, setup_p);
            mem.w8(setup_p + 0x90, 6);
            let v = mem.r32(setup(mem, m) + 0x34);
            mem.w32(m + M_C0, v);
            let v = mem.r16(setup(mem, m) + 0x74);
            mem.w16(m + 0x440, v);
            let v = mem.r16(setup(mem, m) + 0x76);
            mem.w16(m + 0x442, v);
            for bit in [8u8, 0x10, 1, 2, 0x20, 0x40] {
                let f = mem.r8(m + M_FLAGS);
                mem.w8(m + M_FLAGS, f & !bit);
            }
            mem.w8(m + 0x471, 0);
            mem.w32(m + M_MAIN_FN, 0);
            mem.w32(m + M_VIEW, 0);
            mem.w32(m + 0x444, 7);
            let w = mem.r32(setup(mem, m) + 0x5C);
            let v = env.call(0x0200_8374, &mut [Arg::V(w)]);
            mem.w32(m + 0x43C, v);
            let w = mem.r32(setup_p + 0x5C);
            let none = u8::from(env.call(0x0200_83AC, &mut [Arg::V(w)]) == 0);
            let f = mem.r8(m + M_FLAGS);
            mem.w8(m + M_FLAGS, (f & !4) | ((none & 1) << 2));
            let h = heap(mem, m);
            let v = env.call(0x021E_F958, &mut [Arg::V(h)]);
            mem.w32(m + 0x2BC, v);
            env.call(
                0x021B_93E0,
                &mut [Arg::PAt(m + 0xC8), Arg::PAt(m), Arg::V(0)],
            );
            env.call(
                0x021B_93E0,
                &mut [Arg::PAt(m + 0x1B0), Arg::PAt(m), Arg::V(1)],
            );
            env.call(0x021B_A82C, &mut [Arg::PAt(setup_p)]);
            mem.w32(m + 0x2C0, 0);
            mem.w32(m + 0x2C4, 0);
            if mem.r8(setup(mem, m) + 0x23) == 0 {
                env.call(0x0203_F258, &mut [Arg::PAt(m + M_BLOCK18)]);
            } else {
                let b = mem.read_vec(setup_p + 0x9C, 0x18);
                mem.write_bytes(m + M_BLOCK18, &b);
            }
            let h = heap(mem, m);
            env.call(0x021D_7840, &mut [Arg::PAt(m + M_BLOCK18), Arg::V(h)]);
            env.call(0x021C_D6EC, &mut []);
            let v = env.call(0x021D_7F88, &mut [Arg::PAt(setup_p)]);
            mem.w32(m + 0x430, v);
            mem.w32(m + 0x434, 0);
            mem.w32(m + 0x438, 0);
            if mem.r8(setup(mem, m) + 0x23) == 0 {
                // the stored value is the loop counter's start value, loaded once: all four bytes become 0
                for i in 0..4u32 {
                    let s = setup(mem, m);
                    mem.w8(s + i + 0x44, 0);
                }
            }
            env.call(0x021D_8230, &mut [Arg::PAt(m + 0x448)]);
            let w = mem.r32(setup_p + 0x5C);
            let v = u8::from(env.call(0x0200_83C8, &mut [Arg::V(w)]) != 0);
            mem.w8(m + 0x46F, v);
            let w = mem.r32(setup_p + 0x1C);
            let h = u32::from(mem.r16(setup_p + 0x84));
            env.call(0x021D_4150, &mut [Arg::V(w), Arg::V(h), Arg::V(0x14)]);
            env.call(0x021D_804C, &mut [Arg::V(0x12)]);
            next(mem, seq);
            0
        }
        1 => {
            // r0 still holds setup[0x1C] at the 021d4270 call and 0 at the 021d4230 call (both take no argument; the
            // stubs log r0)
            let w = mem.r32(setup(mem, m) + 0x1C);
            if w != 0 && env.call(0x021D_4270, &mut [Arg::V(w)]) != 0 {
                let f = mem.r8(m + M_FLAGS);
                mem.w8(m + M_FLAGS, (f & !1) | 1);
                let f = mem.r8(m + M_FLAGS);
                mem.w8(m + M_FLAGS, f | 2);
                return 1;
            }
            if env.call(0x021D_4230, &mut [Arg::V(0)]) == 0 {
                return 0;
            }
            let b = u32::from(mem.r8(setup(mem, m) + 0x20));
            env.call(0x021E_EA30, &mut [Arg::V(b)]);
            let s = setup(mem, m);
            select_subproc(mem, m + M_SLOT, m, s);
            mem.w32(m + M_460, 0);
            env.call(0x021B_9DB8, &mut [Arg::PAt(m)]);
            env.call(0x021B_A094, &mut [Arg::PAt(m)]);
            next(mem, seq);
            0
        }
        2 => {
            if !run_slot(env, mem, m) {
                return 0;
            }
            let f = mem.r8(m + M_FLAGS);
            if f & 1 != 0 {
                mem.w8(m + M_FLAGS, f | 2);
                return 1;
            }
            if mem.r32(setup(mem, m)) != 4 {
                env.call(0x021B_A1B4, &mut [Arg::PAt(m), Arg::V(4)]);
            }
            1
        }
        _ => 0,
    }
}

/// `*seq += 1`.
fn next(mem: &mut Mem, seq: u32) {
    let s = mem.r32(seq);
    mem.w32(seq, s.wrapping_add(1));
}

/// C: `sub_021b6434`: BattleProc main (the first three arguments are unused): no main loop `M[0x464]` -> 1; `fn(M) == 0` -> 0.
/// Then, local battles (`021b9114(M) == 0`): `r = 021ba54c(M)` (battle result), `021ba6c0` / `021ba878` / `021ba2e4(M)`, and
/// `setup[0x88]` = `-M[0x438]` for results 0 / 2, `M[0x430] + M[0x434]` for 1; comm battles: bit 1 of `setup[0xC1]` = 64cc
/// (all record readers at their end). Returns 1.
pub fn proc_main<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32) -> u32 {
    let f = mem.r32(m + M_MAIN_FN);
    if f == 0 {
        return 1;
    }
    if env.call(f & !1, &mut [Arg::PAt(m)]) == 0 {
        return 0;
    }
    if env.call(0x021B_9114, &mut [Arg::PAt(m)]) == 0 {
        let r = env.call(0x021B_A54C, &mut [Arg::PAt(m)]);
        env.call(0x021B_A6C0, &mut [Arg::PAt(m)]);
        env.call(0x021B_A878, &mut [Arg::PAt(m)]);
        env.call(0x021B_A2E4, &mut [Arg::PAt(m)]);
        let v = match r {
            0 | 2 => Some(mem.r32(m + 0x438).wrapping_neg()),
            1 => Some(mem.r32(m + 0x430).wrapping_add(mem.r32(m + 0x434))),
            _ => None,
        };
        if let Some(v) = v {
            let s = setup(mem, m);
            mem.w32(s + 0x88, v);
        }
    } else {
        let v = readers_at_end(env, m) as u8;
        let s = setup(mem, m);
        let b = mem.r8(s + 0xC1);
        mem.w8(s + 0xC1, (b & !2) | ((v & 1) << 1));
    }
    1
}

/// C: `FUN_overlay_d_93__021b64cc`: 1 unless some present client id `i` (server2 8b9c) has a record reader that is not at its
/// end (`BattleRecordReader_IsAtEnd(M + 0x2C8, i) == 0`).
pub fn readers_at_end<E: FlowEnv>(env: &mut E, m: u32) -> u32 {
    for i in 0..4 {
        if env.call(0x021B_8B9C, &mut [Arg::PAt(m), Arg::V(i)]) != 0
            && env.call(0x021E_E894, &mut [Arg::PAt(m + 0x2C8), Arg::V(i)]) == 0
        {
            return 0;
        }
    }
    1
}

/// C: `sub_021b6500`: BattleProc exit (`proc`, `&seq`, unused, M). seq 0: fade out (`02005E30(0x1E)` unless flag bit 5),
/// brightness word `M[0x460] = 0`, or -16 with both 2D engines set to -16 when `0207D55C(0x0400006C)` (then seq += 2); seq 1:
/// `M[0x460]` steps down to -16 (each engine follows while its brightness is above it), then seq 2; seq 2: wait for the fade
/// (`02005E50`, skipped with flag bit 5), `02005D28()`; seq 3: frees `M[0x2BC]` / `M[0x2C0]` / `M[0x2C4]`, `021d7894`,
/// `021ba0cc` / `021b9de8(M)`, sets the exit sub-proc (6804 -> 6c08); seq 4: runs the sub-proc slot, then `021eea54`,
/// `021d4254`, `021d8058`; seq 5: `021e8df8`, `02005CA4(1, 0x3E)`, `02006740`, flag bit 7 -> `02029F6C(1)` and cleared, frees
/// the proc (`020315D4`) and the heaps 0x13 / 0x14 / 0x12, `021e88f8`; returns 1. Every other step returns 0.
pub fn proc_exit<E: FlowEnv>(env: &mut E, mem: &mut Mem, proc: u32, seq: u32, m: u32) -> u32 {
    const BRIGHT_MAIN: u32 = 0x0400_006C;
    const BRIGHT_SUB: u32 = 0x0400_106C;
    let s = mem.r32(seq);
    match s {
        0 => {
            if (mem.r8(m + M_FLAGS) >> 5) & 1 == 0 {
                env.call(0x0200_5E30, &mut [Arg::V(0x1E)]);
            }
            mem.w32(m + M_460, 0);
            if env.call(0x0207_D55C, &mut [Arg::V(BRIGHT_MAIN)]) != 0 {
                let v = 0u32.wrapping_sub(16);
                mem.w32(m + M_460, v);
                env.call(0x0207_D534, &mut [Arg::V(BRIGHT_MAIN), Arg::V(v)]);
                env.call(0x0207_D534, &mut [Arg::V(BRIGHT_SUB), Arg::V(v)]);
                let s = mem.r32(seq);
                mem.w32(seq, s.wrapping_add(2));
            } else {
                next(mem, seq);
            }
            0
        }
        1 => {
            let v = mem.r32(m + M_460) as i32;
            if v <= -16 {
                mem.w32(seq, s.wrapping_add(1));
                return 0;
            }
            mem.w32(m + M_460, (v - 1) as u32);
            let b = env.call(0x0207_D55C, &mut [Arg::V(BRIGHT_MAIN)]) as i32;
            let cur = mem.r32(m + M_460);
            if b > cur as i32 {
                env.call(0x0207_D534, &mut [Arg::V(BRIGHT_MAIN), Arg::V(cur)]);
            }
            let b = env.call(0x0207_D55C, &mut [Arg::V(BRIGHT_SUB)]) as i32;
            let cur = mem.r32(m + M_460);
            if b > cur as i32 {
                env.call(0x0207_D534, &mut [Arg::V(BRIGHT_SUB), Arg::V(cur)]);
            }
            0
        }
        2 => {
            if (mem.r8(m + M_FLAGS) >> 5) & 1 != 0 {
                mem.w32(seq, s.wrapping_add(1));
                return 0;
            }
            if env.call(0x0200_5E50, &mut []) != 0 {
                return 0;
            }
            env.call(0x0200_5D28, &mut []);
            next(mem, seq);
            0
        }
        3 => {
            for (ofs, free) in [
                (0x2BC, 0x021E_F980),
                (0x2C0, 0x0203_07B0),
                (0x2C4, 0x021D_4D5C),
            ] {
                let p = mem.r32(m + ofs);
                if p != 0 {
                    env.call(free, &mut [Arg::V(p)]);
                    mem.w32(m + ofs, 0);
                }
            }
            env.call(0x021D_7894, &mut []);
            env.call(0x021B_A0CC, &mut [Arg::PAt(m)]);
            env.call(0x021B_9DE8, &mut [Arg::PAt(m)]);
            set_exit_subproc(mem, m + M_SLOT, m);
            next(mem, seq);
            0
        }
        4 => {
            if !run_slot(env, mem, m) {
                return 0;
            }
            env.call(0x021E_EA54, &mut []);
            env.call(0x021D_4254, &mut []);
            env.call(0x021D_8058, &mut []);
            next(mem, seq);
            0
        }
        5 => {
            env.call(0x021E_8DF8, &mut []);
            env.call(0x0200_5CA4, &mut [Arg::V(1), Arg::V(0x3E)]);
            env.call(0x0200_6740, &mut []);
            if mem.r8(m + M_FLAGS) >> 7 != 0 {
                env.call(0x0202_9F6C, &mut [Arg::V(1)]);
                let f = mem.r8(m + M_FLAGS);
                mem.w8(m + M_FLAGS, f & !0x80);
            }
            env.call(0x0203_15D4, &mut [Arg::V(proc)]);
            for h in [0x13, 0x14, 0x12] {
                env.call(0x0203_06F0, &mut [Arg::V(h)]);
            }
            env.call(0x021E_88F8, &mut []);
            1
        }
        _ => 0,
    }
}

/// Stores `{f, 0, arg, 0}` into the sub-proc slot at `slot`.
fn set_slot(mem: &mut Mem, slot: u32, f: u32, arg: u32) {
    mem.w32(slot + 8, arg);
    mem.w32(slot, f);
    mem.w32(slot + 4, 0);
    mem.w32(slot + 0xC, 0);
}

/// C: `FUN_overlay_d_93__021b66d4`: picks the setup sub-proc: the position map `M[0x428..0x42D]` = 4 (none), then the slot gets
/// `{fn, 0, M, 0}` with `fn` by `setup[0x20]` (0 = local) and the rule: local 0 -> 6aac, 1 -> 6dd0 when `setup[0x22]` else
/// 6c5c, 2 -> 7050, 3 -> 71d4; comm 0 -> 734c, 1 -> 73c4, 2 -> 7518, 3 -> 7488; other rules 6aac.
pub fn select_subproc(mem: &mut Mem, slot: u32, m: u32, setup_p: u32) {
    for i in 0..6 {
        mem.w8(m + i + M_POS_CLIENT, 4);
    }
    let rule = mem.r32(setup_p + 4);
    let f = if mem.r8(setup_p + 0x20) == 0 {
        match rule {
            1 if mem.r8(setup_p + 0x22) != 0 => 0x021B_6DD1,
            1 => 0x021B_6C5D,
            2 => 0x021B_7051,
            3 => 0x021B_71D5,
            _ => 0x021B_6AAD,
        }
    } else {
        match rule {
            0 => 0x021B_734D,
            1 => 0x021B_73C5,
            2 => 0x021B_7519,
            3 => 0x021B_7489,
            _ => 0x021B_6AAD,
        }
    };
    set_slot(mem, slot, f, m);
}

/// C: `FUN_overlay_d_93__021b6804`: puts the exit sub-proc into a sub-proc slot `{+0 fn, +4 fn 2, +8 arg, +0xC state}`:
/// `{0x021B6C09, 0, arg, 0}`.
pub fn set_exit_subproc(mem: &mut Mem, slot: u32, arg: u32) {
    set_slot(mem, slot, EXIT_SUBPROC, arg);
}

/// C: `FUN_overlay_d_93__021b6818`: 1 if the battle type word at `+0` of a BATTLE_SETUP is 3, else 0 (a 4-case jump table
/// in the original; types above 3 take the default 0).
pub fn setup_type_is_3(mem: &Mem, setup_p: u32) -> u32 {
    u32::from(mem.r32(setup_p) == 3)
}

/// C: `FUN_overlay_d_93__021b6848`: own client: `M[0x46C] = setup[0x21]`, `M[0x46D] = 8bec(M, id, 0)` (server2),
/// registers the parties (68d8). Local battles (`021b9114(M) == 0`): `M[0x3E0] = 02007B20(M[0x468])` filled by
/// `02007C20(.., 02007B44(02012EBC(setup[0x58])))`, `setup[0x44] = 021b997c(M, 0)`. Comm battles: the 4 bytes `setup[0x44..]`
/// are permuted by side slot (`021ba710(own id, i)`) through a 4-byte stack buffer. That buffer is the slot where the
/// prologue `push {r3-r7, lr}` saved the caller's r3, so a slot no `021ba710` answer hits keeps a byte of the caller's r3:
/// `caller_r3`. At the five call sites (disassembly): 6dd0 has `r3 = 0x46C - 0x41 = 0x42B` (its last position-map store,
/// `sub r3, #0x41` at 0x021B6E02); 6aac / 6c5c / 7050 / 71d4 never write r3 (and 6818 does not touch it), so it is their own
/// entry r3: M at 6100's slot call (`blx r2` at 0x021B63AE, 6100's entry r3 = M unchanged on that path), 0 under the svflow
/// entry state (Z19).
pub fn own_client_init<E: FlowEnv>(
    env: &mut E,
    mem: &mut Mem,
    m: u32,
    setup_p: u32,
    caller_r3: u32,
) {
    let id = mem.r8(setup_p + 0x21);
    mem.w8(m + M_OWN_ID, id);
    let pos = env.call(
        0x021B_8BEC,
        &mut [Arg::PAt(m), Arg::V(u32::from(id)), Arg::V(0)],
    );
    mem.w8(m + M_OWN_POS, pos as u8);
    register_parties(env, mem, m, setup_p);
    if env.call(0x021B_9114, &mut [Arg::PAt(m)]) == 0 {
        let w = mem.r32(setup_p + 0x58);
        let a = env.call(0x0201_2EBC, &mut [Arg::V(w)]);
        let a = env.call(0x0200_7B44, &mut [Arg::V(a)]);
        let h = heap(mem, m);
        let buf = env.call(0x0200_7B20, &mut [Arg::V(h)]);
        mem.w32(m + 0x3E0, buf);
        env.call(0x0200_7C20, &mut [Arg::V(buf), Arg::V(a)]);
        let v = env.call(0x021B_997C, &mut [Arg::PAt(m), Arg::V(0)]);
        let s = setup(mem, m);
        mem.w8(s + 0x44, v as u8);
        return;
    }
    let mut tmp = caller_r3.to_le_bytes();
    for i in 0..4u32 {
        let own = u32::from(mem.r8(m + M_OWN_ID));
        let k = env.call(0x021B_A710, &mut [Arg::V(own), Arg::V(i)]);
        let s = setup(mem, m);
        tmp[k as usize] = mem.r8(s + i + 0x44);
    }
    for (j, &b) in tmp.iter().enumerate() {
        let s = setup(mem, m);
        mem.w8(s + j as u32 + 0x44, b);
    }
}

/// C: `FUN_overlay_d_93__021b68d8`: registers the parties of the sides of clients 0..3 (`k = 021ba710(M[0x46C], i)`):
/// `021ba128(M, i, setup[0x24 + 4 k])` for 0 and 1, for 2 and 3 only when that pointer is non-null.
pub fn register_parties<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, setup_p: u32) {
    let side = |env: &mut E, i: u32| {
        let own = u32::from(mem.r8(m + M_OWN_ID));
        env.call(0x021B_A710, &mut [Arg::V(own), Arg::V(i)])
    };
    let party = |k: u32| mem.r32(setup_p.wrapping_add(k.wrapping_mul(4)) + 0x24);
    let a = side(env, 0);
    let b = side(env, 1);
    env.call(0x021B_A128, &mut [Arg::PAt(m), Arg::V(0), Arg::V(party(a))]);
    env.call(0x021B_A128, &mut [Arg::PAt(m), Arg::V(1), Arg::V(party(b))]);
    let c = side(env, 2);
    let d = side(env, 3);
    let p = party(c);
    if p != 0 {
        env.call(0x021B_A128, &mut [Arg::PAt(m), Arg::V(2), Arg::V(p)]);
    }
    let p = party(d);
    if p != 0 {
        env.call(0x021B_A128, &mut [Arg::PAt(m), Arg::V(3), Arg::V(p)]);
    }
}

/// C: `FUN_overlay_d_93__021b6954`: player record `k` (`M + 0x20 + 0x28 k`) of the own side's slot `s = 021ba710(M[0x46C],
/// k)`: `p = setup[0x48 + 4 s]`; `*p != 0` -> `021b9e7c(rec, p)`, else `021b9e38(rec, M[0x468], setup[0x34 + 4 s])`.
pub fn own_party<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, k: u32) {
    let s = setup(mem, m);
    let own = u32::from(mem.r8(m + M_OWN_ID));
    let side = env.call(0x021B_A710, &mut [Arg::V(own), Arg::V(k)]);
    let base = s.wrapping_add(side.wrapping_mul(4));
    let p = mem.r32(base + 0x48);
    let rec = m + 0x20 + 0x28 * k;
    if mem.r32(p) != 0 {
        env.call(0x021B_9E7C, &mut [Arg::PAt(rec), Arg::V(p)]);
    } else {
        let h = heap(mem, m);
        let w = mem.r32(base + 0x34);
        env.call(0x021B_9E38, &mut [Arg::PAt(rec), Arg::V(h), Arg::V(w)]);
    }
}

/// C: `FUN_overlay_d_93__021b6994`: battle-mode code of client `k` by rule: 0 / 3 / other -> 1, 2 -> 3; rule 1: no multi
/// (`setup[0x22] == 0`) -> 2, battle type 3 -> 1, else 1 when the partner's party `setup[0x24 + 4 ((k + 2) & 3)]` is non-null
/// and `0201AA18(party) > 0` (signed), otherwise 2.
pub fn party_mode<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, k: u32) -> u32 {
    let s = setup(mem, m);
    match mem.r32(s + 4) {
        2 => 3,
        1 => {
            if mem.r8(s + 0x22) == 0 {
                return 2;
            }
            if mem.r32(s) == 3 {
                return 1;
            }
            let p = mem.r32(s + ((k.wrapping_add(2) & 3) << 2) + 0x24);
            if p != 0 && (env.call(0x0201_AA18, &mut [Arg::V(p)]) as i32) > 0 {
                1
            } else {
                2
            }
        }
        _ => 1,
    }
}

/// C: `FUN_overlay_d_93__021b69fc`: own player records: local (`setup[0x23] == 0`): `021b9e38(M + 0x20, M[0x468], M[0xC0])`,
/// `021b9e7c(M + 0x48, setup[0x4C])`; comm: 6954 for 0 and 1.
pub fn own_parties<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, setup_p: u32) {
    if mem.r8(setup_p + 0x23) == 0 {
        let h = heap(mem, m);
        let w = mem.r32(m + M_C0);
        env.call(0x021B_9E38, &mut [Arg::PAt(m + 0x20), Arg::V(h), Arg::V(w)]);
        let p = mem.r32(setup_p + 0x4C);
        env.call(0x021B_9E7C, &mut [Arg::PAt(m + 0x48), Arg::V(p)]);
    } else {
        own_party(env, mem, m, 0);
        own_party(env, mem, m, 1);
    }
}

/// C: `FUN_overlay_d_93__021b6a3c`: comm battles only (`setup[0x23]`): `BattleRecordReader_Init(M + 0x2C8, setup[0x94],
/// setup[0x98])` and the reader attached to clients `M[0x10]` / `M[0x14]` (`021cd938`).
pub fn record_reader_init<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, setup_p: u32) {
    if mem.r8(setup_p + 0x23) == 0 {
        return;
    }
    let a = mem.r32(setup_p + 0x94);
    let b = mem.r32(setup_p + 0x98);
    env.call(0x021E_E644, &mut [Arg::P, Arg::V(a), Arg::V(b)]);
    for ofs in [0x10, 0x14] {
        let c = mem.r32(m + ofs);
        env.call(0x021C_D938, &mut [Arg::V(c), Arg::P]);
    }
}

/// C: `FUN_overlay_d_93__021b6a70`: battle view create: client `c` = `M[0x10 + 4 setup[0x21]]` in comm battles, else `M[0x10]`;
/// `M[4] = 021e8f20(M, c, M + 0xC8, mode, 0x13)`, `021cd95c(c, M[4])`.
pub fn view_create<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32, setup_p: u32, mode: u32) {
    let c = if mem.r8(setup_p + 0x23) != 0 {
        mem.r32(m + 4 * u32::from(mem.r8(setup_p + 0x21)) + M_CLIENTS)
    } else {
        mem.r32(m + M_CLIENTS)
    };
    let v = env.call(
        0x021E_8F20,
        &mut [
            Arg::PAt(m),
            Arg::V(c),
            Arg::PAt(m + 0xC8),
            Arg::V(mode),
            Arg::V(0x13),
        ],
    );
    mem.w32(m + M_VIEW, v);
    env.call(0x021C_D95C, &mut [Arg::V(c), Arg::V(v)]);
}

/// The party-container setup shared by the local sub-procs: `021b93e0(M + 0xC8, M, 0)`, `021b9460(.., 0 / 1)`, then the same
/// for `M + 0x1B0` with side 1.
fn pokecon_init<E: FlowEnv>(env: &mut E, m: u32) {
    for side in 0..2 {
        let pc = m + if side == 0 { 0xC8 } else { 0x1B0 };
        env.call(0x021B_93E0, &mut [Arg::PAt(pc), Arg::PAt(m), Arg::V(side)]);
        env.call(0x021B_9460, &mut [Arg::PAt(pc), Arg::PAt(m), Arg::V(0)]);
        env.call(0x021B_9460, &mut [Arg::PAt(pc), Arg::PAt(m), Arg::V(1)]);
    }
}

/// `M[8] = 021ba9c8(M, M + 0x3F0, M + 0x1B0, mode, M[0x468])` (server3: the server create).
fn server_create<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32, mode: u32) {
    let h = heap(mem, m);
    let sv = env.call(
        0x021B_A9C8,
        &mut [
            Arg::PAt(m),
            Arg::PAt(m + M_BLOCK18),
            Arg::PAt(m + 0x1B0),
            Arg::V(mode),
            Arg::V(h),
        ],
    );
    mem.w32(m + M_SERVER, sv);
}

/// `021cd6f0(M, M + 0xC8, 0, setup[0x1C], id, cmode, a6, mode, setup[0x23], M + 0x3F0, M[0x468])` (client1: the client create
/// of a local battle).
fn client_create<E: FlowEnv>(
    env: &mut E,
    mem: &Mem,
    m: u32,
    id: u32,
    cmode: u32,
    a6: u32,
    mode: u32,
) -> u32 {
    let s = setup(mem, m);
    let w = mem.r32(s + 0x1C);
    let comm = u32::from(mem.r8(s + 0x23));
    let h = heap(mem, m);
    env.call(
        0x021C_D6F0,
        &mut [
            Arg::PAt(m),
            Arg::PAt(m + 0xC8),
            Arg::V(0),
            Arg::V(w),
            Arg::V(id),
            Arg::V(cmode),
            Arg::V(a6),
            Arg::V(mode),
            Arg::V(comm),
            Arg::PAt(m + M_BLOCK18),
            Arg::V(h),
        ],
    )
}

/// `021baa94(M[8], 021cd978(M[0x10 + 4 i]), i, k)` (server_side: attach a client to a server slot).
fn attach<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, i: u32, k: u32) {
    let c = mem.r32(m + M_CLIENTS + 4 * i);
    let ch = env.call(0x021C_D978, &mut [Arg::V(c)]);
    let sv = mem.r32(m + M_SERVER);
    env.call(
        0x021B_AA94,
        &mut [Arg::V(sv), Arg::V(ch), Arg::V(i), Arg::V(k)],
    );
}

/// The tail of the two-client local sub-procs: clients 0 / 1 created with `(cmode, 0 / 1)` into `M[0x10]` / `M[0x14]`, the
/// record readers (6a3c), the view (6a70), both attached (`k = cmode`), the server started (`021bab40`), and the main loop
/// `M[0x464]` = 83f4. Returns 1.
fn local_pair_tail<E: FlowEnv>(
    env: &mut E,
    mem: &mut Mem,
    m: u32,
    s: u32,
    mode: u32,
    cmode: u32,
) -> u32 {
    let c = client_create(env, mem, m, 0, cmode, 0, mode);
    mem.w32(m + M_CLIENTS, c);
    let c = client_create(env, mem, m, 1, cmode, 1, mode);
    mem.w32(m + M_CLIENTS + 4, c);
    record_reader_init(env, mem, m, s);
    view_create(env, mem, m, s, mode);
    attach(env, mem, m, 0, cmode);
    attach(env, mem, m, 1, cmode);
    let sv = mem.r32(m + M_SERVER);
    env.call(0x021B_AB40, &mut [Arg::V(sv)]);
    mem.w32(m + M_MAIN_FN, MAIN_LOOP_LOCAL);
    1
}

/// Writes `ids` into the position map `M[0x428..]`.
fn set_positions(mem: &mut Mem, m: u32, ids: &[u8]) {
    for (i, &v) in ids.iter().enumerate() {
        mem.w8(m + M_POS_CLIENT + i as u32, v);
    }
}

/// C: `sub_021b6aac`: setup sub-proc of a local single battle (`(state, M)`, state unused): `mode = 6818(setup)`, 2 clients,
/// positions {0, 1}, 6848, the party containers, the server create (`M[0x46E] = 1`), 69fc, clients (1, 0 / 1), 6a3c, 6a70,
/// attach (k 1), server start, main loop 83f4. Returns 1.
/// `entry_r3`: the r3 the sub-proc was entered with, which reaches 6848 untouched (see [`own_client_init`]).
pub fn subproc_local_single<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32, entry_r3: u32) -> u32 {
    let s = setup(mem, m);
    let mode = setup_type_is_3(mem, s);
    mem.w8(m + M_CLIENT_COUNT, 2);
    set_positions(mem, m, &[0, 1]);
    own_client_init(env, mem, m, s, entry_r3);
    pokecon_init(env, m);
    server_create(env, mem, m, mode);
    mem.w8(m + M_IS_SERVER, 1);
    own_parties(env, mem, m, s);
    local_pair_tail(env, mem, m, s, mode, 1)
}

/// C: `sub_021b6c08`: the exit sub-proc: frees the view `M[4]` (`021e9034`, cleared), the clients (`021cd8e0`), both party
/// containers (`021b9674`) and the server objects `M[8]` / `M[0xC]` (`021bab5c`). Returns 1.
pub fn subproc_exit<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32) -> u32 {
    let v = mem.r32(m + M_VIEW);
    if v != 0 {
        env.call(0x021E_9034, &mut [Arg::V(v)]);
        mem.w32(m + M_VIEW, 0);
    }
    for i in 0..4 {
        let c = mem.r32(m + M_CLIENTS + 4 * i);
        if c != 0 {
            env.call(0x021C_D8E0, &mut [Arg::V(c)]);
        }
    }
    env.call(0x021B_9674, &mut [Arg::PAt(m + 0xC8)]);
    env.call(0x021B_9674, &mut [Arg::PAt(m + 0x1B0)]);
    for ofs in [8, 0xC] {
        let sv = mem.r32(m + ofs);
        if sv != 0 {
            env.call(0x021B_AB5C, &mut [Arg::V(sv)]);
        }
    }
    1
}

/// C: `sub_021b6c5c`: setup sub-proc of a local double battle: own id 0, 2 clients, positions {0, 1, 0, 1}, 6848, party
/// containers, 69fc before the server create, clients (2, 0 / 1), attach with k 2. Returns 1.
/// `entry_r3`: the r3 the sub-proc was entered with, which reaches 6848 untouched (see [`own_client_init`]).
pub fn subproc_local_double<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32, entry_r3: u32) -> u32 {
    let s = setup(mem, m);
    let mode = setup_type_is_3(mem, s);
    mem.w8(m + M_OWN_ID, 0);
    mem.w8(m + M_CLIENT_COUNT, 2);
    set_positions(mem, m, &[0, 1, 0, 1]);
    own_client_init(env, mem, m, s, entry_r3);
    pokecon_init(env, m);
    own_parties(env, mem, m, s);
    server_create(env, mem, m, mode);
    mem.w8(m + M_IS_SERVER, 1);
    local_pair_tail(env, mem, m, s, mode, 2)
}

/// C: `sub_021b6dd0`: setup sub-proc of a local multi battle (rule 1 with `setup[0x22]`): positions {0, 1, 0, 1}, then by
/// `setup[0x22]`: 1..3 -> positions 2 / 3 = 2 / 3 and 4 clients, 4 -> position 3 = 3 and 3 clients; 6848; both containers and
/// `021b9460` for every present client (server2 8b9c); player records (local: `021b9e38` / `021b9e7c` for 0, 1 and present 2 /
/// 3; comm: 6954 per present client); server create; one client per present id (`021cd6f0` with the client's position count
/// 8b78 and `id != 0`); comm record readers; 6a70; attach per present id (k = position count); server start; main loop 83f4.
pub fn subproc_local_multi<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32) -> u32 {
    let s = setup(mem, m);
    let mode = setup_type_is_3(mem, s);
    mem.w8(m + M_OWN_ID, 0);
    mem.w8(m + M_CLIENT_COUNT, 2);
    set_positions(mem, m, &[0, 1, 0, 1]);
    match mem.r8(s + 0x22) {
        1..=3 => {
            mem.w8(m + M_POS_CLIENT + 2, 2);
            mem.w8(m + M_POS_CLIENT + 3, 3);
            mem.w8(m + M_CLIENT_COUNT, 4);
        }
        4 => {
            mem.w8(m + M_POS_CLIENT + 3, 3);
            mem.w8(m + M_CLIENT_COUNT, 3);
        }
        _ => {}
    }
    own_client_init(env, mem, m, s, 0x42B);
    // the two party containers M + 0xC8 / M + 0x1B0, the player records M + 0x20 + 0x28 k
    env.call(
        0x021B_93E0,
        &mut [Arg::PAt(m + 0xC8), Arg::PAt(m), Arg::V(0)],
    );
    env.call(
        0x021B_93E0,
        &mut [Arg::PAt(m + 0x1B0), Arg::PAt(m), Arg::V(1)],
    );
    let present = |env: &mut E, i: u32| env.call(0x021B_8B9C, &mut [Arg::PAt(m), Arg::V(i)]) != 0;
    for i in 0..4 {
        if present(env, i) {
            env.call(
                0x021B_9460,
                &mut [Arg::PAt(m + 0xC8), Arg::PAt(m), Arg::V(i)],
            );
            env.call(
                0x021B_9460,
                &mut [Arg::PAt(m + 0x1B0), Arg::PAt(m), Arg::V(i)],
            );
        }
    }
    if mem.r8(s + 0x23) == 0 {
        let h = heap(mem, m);
        let w = mem.r32(m + M_C0);
        env.call(0x021B_9E38, &mut [Arg::PAt(m + 0x20), Arg::V(h), Arg::V(w)]);
        let p = mem.r32(s + 0x4C);
        env.call(0x021B_9E7C, &mut [Arg::PAt(m + 0x48), Arg::V(p)]);
        if present(env, 2) {
            let p = mem.r32(s + 0x50);
            env.call(0x021B_9E7C, &mut [Arg::PAt(m + 0x70), Arg::V(p)]);
        }
        if present(env, 3) {
            let p = mem.r32(s + 0x54);
            env.call(0x021B_9E7C, &mut [Arg::PAt(m + 0x98), Arg::V(p)]);
        }
    } else {
        for i in 0..4 {
            if present(env, i) {
                own_party(env, mem, m, i);
            }
        }
    }
    server_create(env, mem, m, mode);
    mem.w8(m + M_IS_SERVER, 1);
    let count = |env: &mut E, i: u32| env.call(0x021B_8B78, &mut [Arg::PAt(m), Arg::V(i)]) & 0xFF;
    for i in 0..4 {
        if present(env, i) {
            let n = count(env, i);
            let c = client_create(env, mem, m, i & 0xFFFF, n, u32::from(i != 0), mode);
            mem.w32(m + M_CLIENTS + 4 * i, c);
        }
    }
    if mem.r8(s + 0x23) != 0 {
        let a = mem.r32(s + 0x94);
        let b = mem.r32(s + 0x98);
        env.call(0x021E_E644, &mut [Arg::P, Arg::V(a), Arg::V(b)]);
        for i in 0..4 {
            if present(env, i) {
                let c = mem.r32(m + M_CLIENTS + 4 * i);
                env.call(0x021C_D938, &mut [Arg::V(c), Arg::P]);
            }
        }
    }
    view_create(env, mem, m, s, mode);
    for i in 0..4 {
        if present(env, i) {
            let n = count(env, i);
            attach(env, mem, m, i, n);
        }
    }
    let sv = mem.r32(m + M_SERVER);
    env.call(0x021B_AB40, &mut [Arg::V(sv)]);
    mem.w32(m + M_MAIN_FN, MAIN_LOOP_LOCAL);
    1
}

/// C: `sub_021b7050`: setup sub-proc of a local rule-2 battle (triple?): 2 clients, server flag, positions {0, 1, 0, 1, 0, 1},
/// as 6aac with 69fc before the server create and clients / attach with mode 3. Returns 1.
/// `entry_r3`: the r3 the sub-proc was entered with, which reaches 6848 untouched (see [`own_client_init`]).
pub fn subproc_local_triple<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32, entry_r3: u32) -> u32 {
    let s = setup(mem, m);
    let mode = setup_type_is_3(mem, s);
    mem.w8(m + M_CLIENT_COUNT, 2);
    mem.w8(m + M_IS_SERVER, 1);
    set_positions(mem, m, &[0, 1, 0, 1, 0, 1]);
    own_client_init(env, mem, m, s, entry_r3);
    pokecon_init(env, m);
    own_parties(env, mem, m, s);
    server_create(env, mem, m, mode);
    mem.w8(m + M_IS_SERVER, 1);
    local_pair_tail(env, mem, m, s, mode, 3)
}

/// C: `sub_021b71d4`: setup sub-proc of a local rule-3 battle (rotation?): positions {0, 1, 0, 1, 0, 1}, as 6aac (69fc after
/// the server create, clients / attach with mode 1). Returns 1.
/// `entry_r3`: the r3 the sub-proc was entered with, which reaches 6848 untouched (see [`own_client_init`]).
pub fn subproc_local_rotation<E: FlowEnv>(
    env: &mut E,
    mem: &mut Mem,
    m: u32,
    entry_r3: u32,
) -> u32 {
    let s = setup(mem, m);
    let mode = setup_type_is_3(mem, s);
    mem.w8(m + M_CLIENT_COUNT, 2);
    set_positions(mem, m, &[0, 1, 0, 1, 0, 1]);
    own_client_init(env, mem, m, s, entry_r3);
    pokecon_init(env, m);
    server_create(env, mem, m, mode);
    mem.w8(m + M_IS_SERVER, 1);
    own_parties(env, mem, m, s);
    local_pair_tail(env, mem, m, s, mode, 1)
}

/// The comm sub-proc driver: state 0 runs `init` then `M[0x46C] = setup[0x21]`, `M[0x46D] = 8bec(M, id, 0)` (server2) and
/// state 1; states 1..6 run the step `table[state](M, M + 0x460)` (server2) and advance when it returns nonzero
/// (`M[0x460] = 0`); returns 0, or 1 from state 7 on.
fn comm_subproc<E: FlowEnv>(
    env: &mut E,
    mem: &mut Mem,
    state: u32,
    m: u32,
    table: u32,
    init: impl FnOnce(&mut Mem),
) -> u32 {
    let st = mem.r32(state);
    if st == 0 {
        init(mem);
        let s = setup(mem, m);
        let id = mem.r8(s + 0x21);
        mem.w8(m + M_OWN_ID, id);
        let id = u32::from(mem.r8(m + M_OWN_ID));
        let pos = env.call(0x021B_8BEC, &mut [Arg::PAt(m), Arg::V(id), Arg::V(0)]);
        mem.w8(m + M_OWN_POS, pos as u8);
        next(mem, state);
        return 0;
    }
    if st >= 7 {
        return 1;
    }
    let f = rom_word(table + 4 * st);
    if env.call(f & !1, &mut [Arg::PAt(m), Arg::PAt(m + M_460)]) != 0 {
        mem.w32(m + M_460, 0);
        next(mem, state);
    }
    0
}

/// C: `sub_021b734c`: comm single-battle sub-proc: state 0: 2 clients, positions {0, 1}; steps from the table 0x021EFD1C (75a8
/// 7758 7968 7bb0 7cb4 835c).
pub fn subproc_comm_single<E: FlowEnv>(env: &mut E, mem: &mut Mem, state: u32, m: u32) -> u32 {
    comm_subproc(env, mem, state, m, 0x021E_FD1C, |mem| {
        mem.w8(m + M_CLIENT_COUNT, 2);
        set_positions(mem, m, &[0, 1]);
    })
}

/// C: `sub_021b73c4`: comm double / multi sub-proc: state 0 by `setup[0x22]`: 1 -> 4 clients at positions {0, 1, 2, 3}; 2 ->
/// the same and `M[0x471] = 2`; other -> 2 clients at {0, 1, 0, 1}; steps from 0x021EFD00 (.. 7e18 ..).
pub fn subproc_comm_double<E: FlowEnv>(env: &mut E, mem: &mut Mem, state: u32, m: u32) -> u32 {
    comm_subproc(env, mem, state, m, 0x021E_FD00, |mem| {
        let s = setup(mem, m);
        let v = mem.r8(s + 0x22);
        if v == 1 || v == 2 {
            if v == 2 {
                mem.w8(m + 0x471, 2);
            }
            mem.w8(m + M_CLIENT_COUNT, 4);
            set_positions(mem, m, &[0, 1, 2, 3]);
        } else {
            mem.w8(m + M_CLIENT_COUNT, 2);
            set_positions(mem, m, &[0, 1, 0, 1]);
        }
    })
}

/// C: `sub_021b7488`: comm rule-3 sub-proc: 2 clients at positions {0, 1, 0, 1, 0, 1}; steps from 0x021EFD38 (.. 81dc ..).
pub fn subproc_comm_rule3<E: FlowEnv>(env: &mut E, mem: &mut Mem, state: u32, m: u32) -> u32 {
    comm_subproc(env, mem, state, m, 0x021E_FD38, |mem| {
        mem.w8(m + M_CLIENT_COUNT, 2);
        set_positions(mem, m, &[0, 1, 0, 1, 0, 1]);
    })
}

/// C: `sub_021b7518`: comm rule-2 sub-proc: 2 clients at positions {0, 1, 0, 1, 0, 1}; steps from 0x021EFD54 (.. 8070 ..).
pub fn subproc_comm_rule2<E: FlowEnv>(env: &mut E, mem: &mut Mem, state: u32, m: u32) -> u32 {
    comm_subproc(env, mem, state, m, 0x021E_FD54, |mem| {
        mem.w8(m + M_CLIENT_COUNT, 2);
        set_positions(mem, m, &[0, 1, 0, 1, 0, 1]);
    })
}

#[cfg(test)]
mod tests;
