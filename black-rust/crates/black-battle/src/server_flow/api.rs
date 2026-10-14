//! Flow API used by the event handlers (0x021C7CBC-0x021C8810; B10c replaces the `FlowHost`
//! methods with these). Specs: `tools/rom-diff/svflow_specs/api.spec` (69 functions; the overlay-95
//! veneers and the five tail calls into overlay 95 that cannot run are listed in its header).
//!
//! B10bc-spec smoke ports: a getter (`7cbc`), the pokeID -> BattleMon lookup (`7cf4`), the
//! iterator-based id list (`7d54`), the action-record search (`804c`) and the 6-entry list append
//! (`831c`). B10c ported the other 64 specced functions. Callees outside this area go through
//! [`FlowEnv`]; callees inside it are called directly.
//!
//! `waddr` parameters are the address of the work itself (the original stores and compares
//! absolute pointers into it); pointers that a callee returns are turned into work offsets with
//! it (in the differential test those callees answer with addresses inside the work image).
//!
//! Not ported (tail calls / veneers into overlay 95, not loaded in the battle image): `7d24`,
//! `7d34`, `7d44`, `814c`, `82c4`, `82e4`, `82f4`, `845c`, `8578`, `87a0`..`8800`; they are
//! reached through [`FlowEnv`] at their own addresses.

use super::env::{Arg, FlowEnv, Lend};
use super::SvFlowWork;

/// Work offset of the main-module pointer.
const OFS_MAIN: usize = 4;
/// Work offset of the pokecon pointer handed to the BattleMon lookups.
const OFS_POKECON: usize = 8;
/// Nesting counter around the type checks `7dac` / `7e14`.
const OFS_CHECK_DEPTH: usize = 0x774;
/// Flag byte (bit 0, bit 3 once-per-battle, bits 3/4 copied from the move-record block, bit 5).
const OFS_FLAGS_78A: usize = 0x78A;
/// Count of the 0x18-entry pokeID set at [`OFS_SET_791`].
const OFS_SET_COUNT: usize = 0x785;
const OFS_SET_791: usize = 0x791;
/// Index of the current move-record block (0..6), see [`move_records_select`].
const OFS_RECORD_INDEX: usize = 0x1A64;

fn main_of(w: &SvFlowWork) -> u32 {
    w.u32_at(OFS_MAIN)
}

fn pokecon_of(w: &SvFlowWork) -> u32 {
    w.u32_at(OFS_POKECON)
}

/// `021b8d08(poke_id)` (through the thunk `021b8ddc`): the client of a pokeID.
fn client_of<E: FlowEnv>(env: &mut E, poke_id: u32) -> u32 {
    env.call(0x021B_8D08, &mut [Arg::V(poke_id)])
}

/// `021d5440(mon)`: the pokeID of a BattleMon.
fn id_of<E: FlowEnv>(env: &mut E, mon: u32) -> u32 {
    env.call(0x021D_5440, &mut [Arg::V(mon)])
}

/// `021b9934(pokecon, poke_id)`: the BattleMon of a pokeID.
fn mon_of<E: FlowEnv>(env: &mut E, w: &SvFlowWork, poke_id: u32) -> u32 {
    env.callwr(
        0x021B_9934,
        &mut [Arg::V(pokecon_of(w)), Arg::V(poke_id)],
        w,
    )
}

/// C: `FUN_overlay_d_93__021c7cbc`: the u16 at `work + 0x1F7C`.
pub fn word_1f7c(w: &SvFlowWork) -> u16 {
    w.u16_at(0x1F7C)
}

/// C: `FUN_overlay_d_93__021c7cc8`: the u16 at `work + 0x1F7E`.
pub fn word_1f7e(w: &SvFlowWork) -> u16 {
    w.u16_at(0x1F7E)
}

/// C: `FUN_overlay_d_93__021c7cd4`: the u16 at `work + 0x1F80`.
pub fn word_1f80(w: &SvFlowWork) -> u16 {
    w.u16_at(0x1F80)
}

/// C: `FUN_overlay_d_93__021c7cdc`: the u16 at `work + 0x1F82`.
pub fn word_1f82(w: &SvFlowWork) -> u16 {
    w.u16_at(0x1F82)
}

/// C: `FUN_overlay_d_93__021c7ce8`: the u16 at `work + 0x1F86`.
pub fn word_1f86(w: &SvFlowWork) -> u16 {
    w.u16_at(0x1F86)
}

/// C: `FUN_overlay_d_93__021c7cf4`: the BattleMon of `poke_id` (tail call
/// `021b9934(work[8], poke_id)`).
pub fn mon_by_id<E: FlowEnv>(env: &mut E, w: &SvFlowWork, poke_id: u32) -> u32 {
    env.callwr(
        0x021B_9934,
        &mut [Arg::V(w.u32_at(OFS_POKECON)), Arg::V(poke_id)],
        w,
    )
}

/// C: `FUN_overlay_d_93__021c7d00`: the battle position of `poke_id`: the index the overlay-95
/// veneer `7d24(work + 0x1AB8, poke_id)` finds, or 6 when `021b86a0(main, index)` rejects it.
pub fn poke_pos<E: FlowEnv>(env: &mut E, w: &SvFlowWork, poke_id: u32) -> u32 {
    let pos = env.callwr(
        0x021C_7D24,
        &mut [
            Arg::PHost(w.mem[0x1AB8..].as_ptr() as usize),
            Arg::V(poke_id),
        ],
        w,
    );
    if env.callwr(0x021B_86A0, &mut [Arg::V(main_of(w)), Arg::V(pos)], w) == 0 {
        return 6;
    }
    pos
}

/// C: `FUN_021c7d54`: walks the mons of the work (core2 iterator `021bd2b4` / `021bd34c`) and
/// writes the pokeID of every mon that is not on the side of `poke_id` (`021b9a24`) to `out`;
/// returns how many (u8 count).
pub fn list_foes<E: FlowEnv>(env: &mut E, poke_id: u32, out: &mut [u8]) -> u32 {
    // the iterator state lives in the caller's frame
    let mut it = [0u8; 4];
    env.call(0x021B_D2B4, &mut [Arg::PBuf(&mut it), Arg::PWork(0)]);
    let mut n = 0u32;
    loop {
        let mut m = [0u8; 4];
        if env.call(
            0x021B_D34C,
            &mut [Arg::PBuf(&mut it), Arg::PWork(0), Arg::Out(&mut m)],
        ) == 0
        {
            break;
        }
        let mon = u32::from_le_bytes(m);
        let id = env.call(0x021D_5440, &mut [Arg::V(mon)]);
        if env.call(0x021B_9A24, &mut [Arg::V(poke_id), Arg::V(id)]) == 0 {
            let id = env.call(0x021D_5440, &mut [Arg::V(mon)]);
            out[n as usize] = id as u8;
            n = (n + 1) & 0xFF;
        }
    }
    n
}

/// The defender after `021d6c24(def)` / `021b937c(main, pokecon, def)` (a substitute target).
fn redirect_def<E: FlowEnv>(env: &mut E, w: &SvFlowWork, def: u32) -> u32 {
    if env.callwr(0x021D_6C24, &mut [Arg::V(def)], w) != 0 {
        return env.callwr(
            0x021B_937C,
            &mut [Arg::V(main_of(w)), Arg::V(pokecon_of(w)), Arg::V(def)],
            w,
        );
    }
    def
}

fn depth_add(w: &mut SvFlowWork, d: u32) {
    let v = w.u32_at(OFS_CHECK_DEPTH).wrapping_add(d);
    w.set_u32_at(OFS_CHECK_DEPTH, v);
}

/// C: `FUN_021c7dac(work, atk, def, move)`: with the nesting counter `work[0x774]` raised, fills a
/// 0x14-byte move record (core5 `021c64b4(work, move, atk_mon, &rec)`) and returns the core6 type
/// check `021c6e3c(work, atk_mon, def_mon, &rec, 0)`.
pub fn move_type_check<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    atk: u32,
    def: u32,
    mv: u32,
) -> u32 {
    let atk_mon = mon_of(env, w, atk);
    let def_mon = mon_of(env, w, def);
    depth_add(w, 1);
    let def_mon = redirect_def(env, w, def_mon);
    let mut rec = [0u8; 0x14];
    env.callw(
        0x021C_64B4,
        &mut [Arg::P, Arg::V(mv), Arg::V(atk_mon), Arg::Out(&mut rec)],
        w,
    );
    let r = env.callw(
        0x021C_6E3C,
        &mut [
            Arg::PWork(0),
            Arg::V(atk_mon),
            Arg::V(def_mon),
            Arg::In(&rec[..0x10]),
            Arg::V(0),
        ],
        w,
    );
    depth_add(w, 0xFFFF_FFFF);
    r
}

/// C: `FUN_021c7e14(work, atk, def, move, use_check, flag)` (two stack arguments): 0 for move 0 or
/// when `0201c314(move)` is 0; else, with `work[0x774]` raised, the u16 that core4
/// `021c1e74(work, atk_mon, def_mon, &rec, eff, 0x1000, 0, flag == 0, &out)` writes, `eff` being
/// [`move_type_check`] when `use_check` is set and 3 otherwise.
#[allow(clippy::too_many_arguments)]
pub fn move_damage_value<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    atk: u32,
    def: u32,
    mv: u32,
    use_check: u32,
    flag: u32,
) -> u16 {
    if mv == 0 || env.callw(0x0201_C314, &mut [Arg::V(mv)], w) == 0 {
        return 0;
    }
    let atk_mon = mon_of(env, w, atk);
    let def_mon = mon_of(env, w, def);
    depth_add(w, 1);
    let def_mon = redirect_def(env, w, def_mon);
    let eff = if use_check != 0 {
        move_type_check(env, w, atk, def, mv)
    } else {
        3
    };
    let mut rec = [0u8; 0x14];
    env.callw(
        0x021C_64B4,
        &mut [Arg::P, Arg::V(mv), Arg::V(atk_mon), Arg::Out(&mut rec)],
        w,
    );
    let mut out = [0u8; 2];
    env.callw(
        0x021C_1E74,
        &mut [
            Arg::PWork(0),
            Arg::V(atk_mon),
            Arg::V(def_mon),
            Arg::In(&rec[..0x10]),
            Arg::V(eff),
            Arg::V(0x1000),
            Arg::V(0),
            Arg::V((flag == 0) as u32),
            Arg::Out(&mut out),
        ],
        w,
    );
    depth_add(w, 0xFFFF_FFFF);
    u16::from_le_bytes(out)
}

/// C: `FUN_021c7ec4`: the nesting counter `work[0x774]` is nonzero.
pub fn in_type_check(w: &SvFlowWork) -> bool {
    w.u32_at(OFS_CHECK_DEPTH) != 0
}

/// C: `FUN_021c7ed8`: `021b997c(main, client(poke_id))`.
pub fn client_value_997c<E: FlowEnv>(env: &mut E, w: &SvFlowWork, poke_id: u32) -> u32 {
    let client = client_of(&mut Lend::new_ref(env, w), poke_id);
    env.callwr(0x021B_997C, &mut [Arg::V(main_of(w)), Arg::V(client)], w)
}

/// C: `FUN_overlay_d_93__021c7eec`: byte +9 of the client record server_side
/// `021bb85c(work[0], client(poke_id))` returns (`waddr`: see the module doc).
pub fn client_byte9<E: FlowEnv>(env: &mut E, w: &SvFlowWork, waddr: u32, poke_id: u32) -> u8 {
    let client = client_of(&mut Lend::new_ref(env, w), poke_id);
    let rec = env.callwr(0x021B_B85C, &mut [Arg::PAt(w.u32_at(0)), Arg::V(client)], w);
    w.u8_at(rec.wrapping_sub(waddr) as usize + 9)
}

/// C: `FUN_overlay_d_93__021c7f04`: 3 when the battle rule `021b8588(main)` is 3, else
/// [`client_byte9`].
pub fn client_count<E: FlowEnv>(env: &mut E, w: &SvFlowWork, waddr: u32, poke_id: u32) -> u32 {
    if env.callwr(0x021B_8588, &mut [Arg::V(main_of(w))], w) == 3 {
        return 3;
    }
    client_byte9(env, w, waddr, poke_id) as u32
}

/// C: `FUN_overlay_d_93__021c7f24`: `021b9bc4(party_of(poke_id), client_count(poke_id)) != 0`.
pub fn party_check<E: FlowEnv>(env: &mut E, w: &SvFlowWork, waddr: u32, poke_id: u32) -> bool {
    let party = party_of(env, w, poke_id);
    let n = client_count(env, w, waddr, poke_id);
    env.callwr(0x021B_9BC4, &mut [Arg::V(party), Arg::V(n)], w) != 0
}

/// C: unlabeled routine at `0x021C7F4C`: test-and-set of bit 3 of `work[0x78A]`: false if it was
/// set, else sets it and returns true.
pub fn take_flag_78a_bit3(w: &mut SvFlowWork) -> bool {
    let b = w.u8_at(OFS_FLAGS_78A);
    if (b >> 3) & 1 != 0 {
        return false;
    }
    w.set_u8_at(OFS_FLAGS_78A, b | 8);
    true
}

/// C: `FUN_overlay_d_93__021c7f68(work, code, out)`: the positions `021b8778(main, code, &pos)`
/// expands `code` into -> the pokeIDs of the mons there (`021b98ac(pokecon, pos)`) that exist and
/// are not fainted (`021d5b68` = 0), written to `out`; returns how many.
pub fn expand_poke_ids<E: FlowEnv>(env: &mut E, w: &SvFlowWork, code: u32, out: &mut [u8]) -> u32 {
    let mut pos = [0u8; 8];
    let n = env.callwr(
        0x021B_8778,
        &mut [Arg::V(main_of(w)), Arg::V(code), Arg::Out(&mut pos)],
        w,
    );
    let mut i = 0u8;
    let mut k = 0u8;
    if n != 0 {
        loop {
            let mon = env.callwr(
                0x021B_98AC,
                &mut [Arg::V(pokecon_of(w)), Arg::V(pos[i as usize] as u32)],
                w,
            );
            let alive = mon != 0 && env.callwr(0x021D_5B68, &mut [Arg::V(mon)], w) == 0;
            if alive {
                let id = id_of(&mut Lend::new_ref(env, w), mon);
                out[k as usize] = id as u8;
                k = k.wrapping_add(1);
            }
            i = i.wrapping_add(1);
            if (i as u32) >= n {
                break;
            }
        }
    }
    k as u32
}

/// C: `FUN_overlay_d_93__021c7fcc`: `021b9af8(pokecon, client(poke_id))`, the party of the
/// pokeID's client.
pub fn party_of<E: FlowEnv>(env: &mut E, w: &SvFlowWork, poke_id: u32) -> u32 {
    let client = client_of(&mut Lend::new_ref(env, w), poke_id);
    env.callwr(0x021B_9AF8, &mut [Arg::V(pokecon_of(w)), Arg::V(client)], w)
}

/// C: `FUN_overlay_d_93__021c7fe0`: the party of the partner client `021b9010(main, client)`
/// when `021b8b9c(main, partner)` holds, else 0.
pub fn partner_party_of<E: FlowEnv>(env: &mut E, w: &SvFlowWork, poke_id: u32) -> u32 {
    let client = client_of(&mut Lend::new_ref(env, w), poke_id);
    let partner = env.callwr(0x021B_9010, &mut [Arg::V(main_of(w)), Arg::V(client)], w);
    if env.callwr(0x021B_8B9C, &mut [Arg::V(main_of(w)), Arg::V(partner)], w) == 0 {
        return 0;
    }
    env.callwr(
        0x021B_9AF8,
        &mut [Arg::V(pokecon_of(w)), Arg::V(partner)],
        w,
    )
}

/// C: `FUN_overlay_d_93__021c8010`: tail call `021b8d8c(main, pokecon, poke_id)` (the position,
/// 6 when not in battle).
pub fn poke_pos_in_battle<E: FlowEnv>(env: &mut E, w: &SvFlowWork, poke_id: u32) -> u32 {
    env.callwr(
        0x021B_8D8C,
        &mut [Arg::V(main_of(w)), Arg::V(pokecon_of(w)), Arg::V(poke_id)],
        w,
    )
}

/// C: `sub_021c8020`: the pokeID of the mon at `pos` (`021d5440(021b98ac(pokecon, pos))`).
pub fn poke_id_at_pos<E: FlowEnv>(env: &mut E, w: &SvFlowWork, pos: u32) -> u32 {
    let mon = env.callwr(0x021B_98AC, &mut [Arg::V(pokecon_of(w)), Arg::V(pos)], w);
    id_of(&mut Lend::new_ref(env, w), mon)
}

/// C: `FUN_overlay_d_93__021c8030`: `021d80bc(main, pokecon, mon(poke_id), x)`.
pub fn mon_query_80bc<E: FlowEnv>(env: &mut E, w: &SvFlowWork, poke_id: u32, x: u32) -> u32 {
    let mon = mon_of(env, w, poke_id);
    env.callwr(
        0x021D_80BC,
        &mut [
            Arg::V(main_of(w)),
            Arg::V(pokecon_of(w)),
            Arg::V(mon),
            Arg::V(x),
        ],
        w,
    )
}

/// C: `FUN_overlay_d_93__021c804c`: searches the `work[0x782]` action records (0x10 bytes each,
/// BattleMon at +0x7E0, action word at +0x7E4) for one of `poke_id` whose kind (`word & 0xF`) is
/// not 6; writes its word to `out` and returns 1, or returns 0.
pub fn find_action<E: FlowEnv>(env: &mut E, w: &SvFlowWork, poke_id: u32, out: &mut [u8]) -> u32 {
    let mut i = 0usize;
    while i < w.u8_at(0x782) as usize {
        let mon = w.u32_at(0x7E0 + 16 * i);
        if poke_id == env.callwr(0x021D_5440, &mut [Arg::V(mon)], w) {
            let word = w.u32_at(0x7E4 + 16 * i);
            if word & 0xF != 6 {
                out[0..4].copy_from_slice(&word.to_le_bytes());
                return 1;
            }
        }
        i += 1;
    }
    0
}

/// C: `FUN_overlay_d_93__021c80ac`: true when every action record (see [`find_action`]) of a mon
/// other than `poke_id` has its byte +0x7ED set.
pub fn others_actions_done<E: FlowEnv>(env: &mut E, w: &SvFlowWork, poke_id: u32) -> bool {
    let mut done = 0u32;
    let mut others = 0u32;
    let mut i = 0usize;
    while i < w.u8_at(0x782) as usize {
        let mon = w.u32_at(0x7E0 + 16 * i);
        if poke_id != id_of(&mut Lend::new_ref(env, w), mon) {
            others = others.wrapping_add(1);
            if w.u8_at(0x7ED + 16 * i) != 0 {
                done = done.wrapping_add(1);
            }
        }
        i += 1;
    }
    others == done
}

/// C: `FUN_021c8114`: the u16 at `work + 0x1F74`.
pub fn word_1f74(w: &SvFlowWork) -> u16 {
    w.u16_at(0x1F74)
}

/// C: `FUN_021c8120`: `work + 0x1FEC` (returned as a work offset).
pub fn ofs_1fec() -> usize {
    0x1FEC
}

/// C: `FUN_021c812c`: `work + 0x1C`, the move-history ring (returned as a work offset).
pub fn ofs_1c() -> usize {
    0x1C
}

/// C: `FUN_021c8130`: `work + 0x3E0`, the side party records (returned as a work offset).
pub fn ofs_3e0() -> usize {
    0x3E0
}

/// C: `FUN_overlay_d_93__021c8138`: the turn counter, low half of `work[0x10]`.
pub fn turn_count(w: &SvFlowWork) -> u16 {
    w.u32_at(0x10) as u16
}

/// C: `FUN_overlay_d_93__021c8140`: the byte `work[0x7D9 + i]`.
pub fn byte_7d9(w: &SvFlowWork, i: u32) -> u8 {
    w.u8_at(0x7D9usize.wrapping_add(i as usize))
}

/// C: `FUN_overlay_d_93__021c815c`: tail call `021b8588(main)` (the battle rule).
pub fn rule<E: FlowEnv>(env: &mut E, w: &SvFlowWork) -> u32 {
    env.callwr(0x021B_8588, &mut [Arg::V(main_of(w))], w)
}

/// C: `FUN_overlay_d_93__021c8168`: tail call `021b86b8(main)`.
pub fn main_86b8<E: FlowEnv>(env: &mut E, w: &SvFlowWork) -> u32 {
    env.callwr(0x021B_86B8, &mut [Arg::V(main_of(w))], w)
}

/// C: `FUN_021c8174`: word +4 of the record `021b8714(main)` returns (`waddr`: see the module
/// doc).
pub fn main_8714_word4<E: FlowEnv>(env: &mut E, w: &SvFlowWork, waddr: u32) -> u32 {
    let rec = env.callwr(0x021B_8714, &mut [Arg::V(main_of(w))], w);
    w.u32_at(rec.wrapping_sub(waddr) as usize + 4)
}

/// C: `FUN_021c8180`: tail call `021b8670(main)`.
pub fn main_8670<E: FlowEnv>(env: &mut E, w: &SvFlowWork) -> u32 {
    env.callwr(0x021B_8670, &mut [Arg::V(main_of(w))], w)
}

/// C: `FUN_021c818c`: true when `0201b718(work[0x4A4], species(mon), 0, i)` holds for some
/// `i < 7` (`021d5444` gives the value of the pokeID's mon).
pub fn species_check<E: FlowEnv>(env: &mut E, w: &SvFlowWork, poke_id: u32) -> bool {
    let mon = mon_of(env, w, poke_id);
    let sp = env.callwr(0x021D_5444, &mut [Arg::V(mon)], w);
    let mut i = 0u16;
    loop {
        let r = env.callwr(
            0x0201_B718,
            &mut [
                Arg::V(w.u32_at(0x4A4)),
                Arg::V(sp),
                Arg::V(0),
                Arg::V(i as u32),
            ],
            w,
        );
        if r != 0 {
            return true;
        }
        i += 1;
        if i >= 7 {
            return false;
        }
    }
}

/// C: `sub_021c81c8`: thunk to core1 `021bc948(work, mon, flag)` (a stat of a BattleMon).
pub fn mon_stat<E: FlowEnv>(env: &mut E, mon: u32, flag: u32) -> u32 {
    env.call(0x021B_C948, &mut [Arg::PWork(0), Arg::V(mon), Arg::V(flag)])
}

/// C: `FUN_021c81d0(work, mon, flag)`: how many other mons (core2 iterator `021bd2b4` /
/// `021bd34c`) have a [`mon_stat`] above that of `mon` (u16 count).
pub fn count_faster<E: FlowEnv>(env: &mut E, mon: u32, flag: u32) -> u32 {
    let base = mon_stat(env, mon, flag);
    let id = id_of(env, mon);
    let mut n = 0u16;
    // the iterator state lives in the caller's frame
    let mut it = [0u8; 4];
    env.call(0x021B_D2B4, &mut [Arg::PBuf(&mut it), Arg::PWork(0)]);
    loop {
        let mut m = [0u8; 4];
        if env.call(
            0x021B_D34C,
            &mut [Arg::PBuf(&mut it), Arg::PWork(0), Arg::Out(&mut m)],
        ) == 0
        {
            break;
        }
        let other = u32::from_le_bytes(m);
        if id != id_of(env, other) && mon_stat(env, other, flag) > base {
            n = n.wrapping_add(1);
        }
    }
    n as u32
}

/// C: `FUN_overlay_d_93__021c8234`: core6 `021c6f0c(work, mon(poke_id), 1)`.
pub fn mon_query_6f0c<E: FlowEnv>(env: &mut E, w: &SvFlowWork, poke_id: u32) -> u32 {
    let mon = mon_of(env, w, poke_id);
    env.callwr(0x021C_6F0C, &mut [Arg::P, Arg::V(mon), Arg::V(1)], w)
}

/// C: `FUN_overlay_d_93__021c824c`: false when the mon's field 0x11 (`BattleMon_Get`) is 0x67 or
/// it has condition 0x13 (`021d62c4`); else `021ef948(7) == 0`.
pub fn mon_check_824c<E: FlowEnv>(env: &mut E, w: &SvFlowWork, poke_id: u32) -> bool {
    let mon = mon_of(env, w, poke_id);
    if env.callwr(0x021D_59B4, &mut [Arg::V(mon), Arg::V(0x11)], w) == 0x67 {
        return false;
    }
    if env.callwr(0x021D_62C4, &mut [Arg::V(mon), Arg::V(0x13)], w) != 0 {
        return false;
    }
    env.callwr(0x021E_F948, &mut [Arg::V(7)], w) == 0
}

/// C: `thunk_FUN_overlay_d_93__021c2ab0` (`0x021C8288`): thunk to core4 `021c2ab0(work)` (the
/// weather).
pub fn weather<E: FlowEnv>(env: &mut E) -> u32 {
    env.call(0x021C_2AB0, &mut [Arg::PWork(0)])
}

/// C: `FUN_overlay_d_93__021c8290`: core3 `021bfa00(work, mon(atk), mon(def), x)`.
pub fn mon_pair_bfa00<E: FlowEnv>(env: &mut E, w: &SvFlowWork, atk: u32, def: u32, x: u32) -> u32 {
    let a = mon_of(env, w, atk);
    let d = mon_of(env, w, def);
    env.callwr(
        0x021B_FA00,
        &mut [Arg::PWork(0), Arg::V(a), Arg::V(d), Arg::V(x)],
        w,
    )
}

/// C: `FUN_overlay_d_93__021c82b8`: tail call `021b911c(main, x)` (a ROM stub that returns 0 and
/// reads nothing).
pub fn main_911c<E: FlowEnv>(env: &mut E) -> u32 {
    env.call(0x021B_911C, &mut [])
}

/// C: `FUN_021c82d0(_, x, kind)` (r0 not read): the overlay-95 side-condition veneer
/// `82e4(021b9ae0(x), kind)`.
pub fn side_condition<E: FlowEnv>(env: &mut E, x: u32, kind: u32) -> u32 {
    let side = env.call(0x021B_9AE0, &mut [Arg::V(x)]);
    env.call(0x021C_82E4, &mut [Arg::V(side), Arg::V(kind)])
}

/// C: `FUN_021c8300`: tail call `021b9010(main, x)`.
pub fn main_9010<E: FlowEnv>(env: &mut E, w: &SvFlowWork, x: u32) -> u32 {
    env.callwr(0x021B_9010, &mut [Arg::V(main_of(w)), Arg::V(x)], w)
}

/// C: `FUN_021c830c`: bit 0 of `work[0x78A]`.
pub fn flag_78a_bit0(w: &SvFlowWork) -> u32 {
    (w.u8_at(OFS_FLAGS_78A) & 1) as u32
}

/// C: unlabeled routine at `0x021C831C`: appends the byte `v` to the 6-entry list at
/// `work + 0x78B` (count `work[0x788]`); no-op when it is full.
pub fn list6_push(w: &mut SvFlowWork, v: u32) {
    let n = w.u8_at(0x788);
    if n < 6 {
        w.set_u8_at(0x788, n + 1);
        w.set_u8_at(0x78B + n as usize, v as u8);
    }
}

/// C: `FUN_overlay_d_93__021c8338`: `021d5c30(mon(poke_id))`.
pub fn mon_query_5c30<E: FlowEnv>(env: &mut E, w: &SvFlowWork, poke_id: u32) -> u32 {
    let mon = mon_of(env, w, poke_id);
    env.callwr(0x021D_5C30, &mut [Arg::V(mon)], w)
}

/// C: `FUN_overlay_d_93__021c8348`: `x = 021d69ac(mon(poke_id), 4)` is nonzero and
/// `(u8)(x - 1) < 0x18`.
pub fn mon_field4_in_range<E: FlowEnv>(env: &mut E, w: &SvFlowWork, poke_id: u32) -> bool {
    let mon = mon_of(env, w, poke_id);
    let x = env.callwr(0x021D_69AC, &mut [Arg::V(mon), Arg::V(4)], w);
    let mut v = 0x1F;
    if x != 0 {
        let y = x.wrapping_sub(1) & 0xFF;
        if y < 0x18 {
            v = y;
        }
    }
    v != 0x1F
}

/// C: `FUN_overlay_d_93__021c8374`: [`mon_field4_in_range`] or condition 0x21 (`021d62c4`).
pub fn mon_check_8374<E: FlowEnv>(env: &mut E, w: &SvFlowWork, poke_id: u32) -> bool {
    if mon_field4_in_range(env, w, poke_id) {
        return true;
    }
    let mon = mon_of(env, w, poke_id);
    env.callwr(0x021D_62C4, &mut [Arg::V(mon), Arg::V(0x21)], w) != 0
}

/// C: `FUN_overlay_d_93__021c83a0`: `max(1, 021d7b10(021d67e8(mon), core4 021c2af0(work, mon)))`
/// (a weight-like value).
pub fn mon_weight<E: FlowEnv>(env: &mut E, w: &SvFlowWork, poke_id: u32) -> u32 {
    let mon = mon_of(env, w, poke_id);
    let m = env.callwr(0x021C_2AF0, &mut [Arg::PWork(0), Arg::V(mon)], w);
    let base = env.callwr(0x021D_67E8, &mut [Arg::V(mon)], w);
    let r = env.callwr(0x021D_7B10, &mut [Arg::V(base), Arg::V(m)], w);
    if r < 1 {
        1
    } else {
        r
    }
}

/// C: `FUN_overlay_d_93__021c2e4c` (thunk at `0x021C83CC`): thunk to core4 `021c2e4c(work)`.
pub fn counter_2e4c<E: FlowEnv>(env: &mut E) -> u32 {
    env.call(0x021C_2E4C, &mut [Arg::PWork(0)])
}

/// C: `thunk_FUN_overlay_d_93__021bc2b0` (`0x021C83D4`): thunk to core1 `021bc2b0(work)`.
pub fn core1_bc2b0<E: FlowEnv>(env: &mut E) -> u32 {
    env.call(0x021B_C2B0, &mut [Arg::PWork(0)])
}

/// C: `FUN_overlay_d_93__021c83dc`: byte +4 of the per-turn record behind `work[0x4A8]` := `v`
/// (`rec` = that record).
pub fn set_turn_rec_byte4(rec: &mut [u8], v: u32) {
    rec[4] = v as u8;
}

/// C: `FUN_021c83e8`: when `021b86b8(main) <= 1`, server_side `021bb908(work[0], v)` and true;
/// else false.
pub fn server_bb908<E: FlowEnv>(env: &mut E, w: &SvFlowWork, v: u32) -> bool {
    if env.callwr(0x021B_86B8, &mut [Arg::V(main_of(w))], w) > 1 {
        return false;
    }
    env.callwr(0x021B_B908, &mut [Arg::P, Arg::V(v)], w);
    true
}

/// C: `sub_021c8408`: server_side `021bb914(work[0])` when `021b86b8(main) <= 1` and the pokeID's
/// client is `021b8ff4(main)`.
pub fn server_bb914<E: FlowEnv>(env: &mut E, w: &SvFlowWork, poke_id: u32) {
    let mode = env.callwr(0x021B_86B8, &mut [Arg::V(main_of(w))], w);
    let client = client_of(&mut Lend::new_ref(env, w), poke_id);
    if mode <= 1 && client == env.callwr(0x021B_8FF4, &mut [Arg::V(main_of(w))], w) {
        env.callwr(0x021B_B914, &mut [Arg::PAt(w.u32_at(0))], w);
    }
}

/// C: `FUN_overlay_d_93__021c8434(work, a, b)`: core2 `021be52c(work[0x4B0], a, b, i)` with `i`
/// = the overlay-95 veneer `845c(work + 0x1AB8, b)` (Magic Bounce).
pub fn bounce_move<E: FlowEnv>(env: &mut E, w: &SvFlowWork, a: u32, b: u32) {
    let i = env.callwr(0x021C_845C, &mut [Arg::P, Arg::V(b)], w);
    env.callwr(
        0x021B_E52C,
        &mut [Arg::PAt(w.u32_at(0x4B0)), Arg::V(a), Arg::V(b), Arg::V(i)],
        w,
    );
}

/// C: `FUN_021c846c`: core3 `021bfeac(work, mon(a), mon(b), out)` (`out` passed through).
pub fn mon_pair_bfeac<E: FlowEnv>(env: &mut E, w: &SvFlowWork, a: u32, b: u32, out: u32) -> u32 {
    let ma = mon_of(env, w, a);
    let mb = mon_of(env, w, b);
    env.callwr(
        0x021B_FEAC,
        &mut [Arg::PWork(0), Arg::V(ma), Arg::V(mb), Arg::PAt(out)],
        w,
    )
}

/// C: `FUN_overlay_d_93__021c8494`: core3 `021bff60(work, mon(poke_id), 1)`.
pub fn mon_bff60<E: FlowEnv>(env: &mut E, w: &SvFlowWork, poke_id: u32) {
    let mon = mon_of(env, w, poke_id);
    env.callwr(0x021B_FF60, &mut [Arg::PWork(0), Arg::V(mon), Arg::V(1)], w);
}

/// C: `FUN_021c84ac(work, x, out)`: clears bit 5 of `work[0x78A]`, calls core4
/// `021c2128(work, x)`, then `*out` (byte) = bit 5 as it is afterwards; returns 2128's result.
pub fn call_2128_flag<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, x: u32, out: &mut [u8]) -> u32 {
    let b = w.u8_at(OFS_FLAGS_78A) & !0x20;
    w.set_u8_at(OFS_FLAGS_78A, b);
    let r = env.callw(0x021C_2128, &mut [Arg::PWork(0), Arg::V(x)], w);
    out[0] = (w.u8_at(OFS_FLAGS_78A) >> 5) & 1;
    r
}

/// C: `FUN_overlay_d_93__021bdbb8` (thunk at `0x021C84D0`): thunk to core2 `021bdbb8(work, x)`.
pub fn core2_bdbb8<E: FlowEnv>(env: &mut E, x: u32) {
    env.call(0x021B_DBB8, &mut [Arg::PWork(0), Arg::V(x)]);
}

/// C: `thunk_FUN_overlay_d_93__021cc668` (`0x021C84D8`): thunk to events `021cc668(work, mon)`.
pub fn events_cc668<E: FlowEnv>(env: &mut E, mon: u32) -> u32 {
    env.call(0x021C_C668, &mut [Arg::PWork(0), Arg::V(mon)])
}

/// C: `sub_021c84f4` (thunk at `0x021C84E0`): thunk to [`set_add`].
pub fn set_add_thunk(w: &mut SvFlowWork, v: u32) {
    set_add(w, v);
}

/// C: `FUN_overlay_d_93__021c84e8`: empties the pokeID set (`work[0x785] := 0`).
pub fn set_clear(w: &mut SvFlowWork) {
    w.set_u8_at(OFS_SET_COUNT, 0);
}

/// C: `sub_021c84f4`: adds the byte `v` to the set at `work + 0x791` (count `work[0x785]`, at
/// most 0x18) unless it is there (compared as a full word against each byte).
pub fn set_add(w: &mut SvFlowWork, v: u32) {
    let n = w.u8_at(OFS_SET_COUNT) as usize;
    let mut i = 0usize;
    while i < n {
        if v == w.u8_at(OFS_SET_791 + i) as u32 {
            return;
        }
        i += 1;
    }
    if i < 0x18 {
        w.set_u8_at(OFS_SET_791 + i, v as u8);
        let c = w.u8_at(OFS_SET_COUNT).wrapping_add(1);
        w.set_u8_at(OFS_SET_COUNT, c);
    }
}

/// C: `FUN_overlay_d_93__021c8534`: true when the overlay-95 veneer
/// `8578(work + 0x1AB8, client(set[i]), sp)` finds something for some `i < work[0x785]`.
///
/// The original hands its own stack slot to `0689d558` as a 6-byte out buffer (latent bug: a
/// 5-6 entry result would overwrite the caller's saved r4); the port passes a pointer the callee
/// only receives, like the spec.
pub fn set_any_8578<E: FlowEnv>(env: &mut E, w: &SvFlowWork) -> bool {
    let n = w.u8_at(OFS_SET_COUNT) as usize;
    if n == 0 {
        return false;
    }
    let mut i = 0usize;
    loop {
        let id = w.u8_at(OFS_SET_791 + i) as u32;
        let client = client_of(&mut Lend::new_ref(env, w), id);
        if env.callwr(
            0x021C_8578,
            &mut [
                Arg::PHost(w.mem[0x1AB8..].as_ptr() as usize),
                Arg::V(client),
                Arg::P,
            ],
            w,
        ) != 0
        {
            return true;
        }
        i += 1;
        if i >= w.u8_at(OFS_SET_COUNT) as usize {
            return false;
        }
    }
}

/// C: `FUN_overlay_d_93__021c8588`: `work[0x1A64] := 0`; tail call
/// [`move_records_select`]`(work, 0, 1)`.
pub fn move_records_reset<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, waddr: u32) {
    w.set_u32_at(OFS_RECORD_INDEX, 0);
    move_records_select(env, w, waddr, 0, 1);
}

/// C: `FUN_overlay_d_93__021c859c`: if `work[0x1A64] < 6`, increments it and selects (and
/// clears) that block.
pub fn move_records_push<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, waddr: u32) {
    let n = w.u32_at(OFS_RECORD_INDEX);
    if n < 6 {
        w.set_u32_at(OFS_RECORD_INDEX, n + 1);
        move_records_select(env, w, waddr, n + 1, 1);
    }
}

/// C: `FUN_overlay_d_93__021c85b8`: if `work[0x1A64] != 0`, decrements it and selects that block
/// without clearing it.
pub fn move_records_pop<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, waddr: u32) {
    let n = w.u32_at(OFS_RECORD_INDEX);
    if n != 0 {
        w.set_u32_at(OFS_RECORD_INDEX, n - 1);
        move_records_select(env, w, waddr, n - 1, 0);
    }
}

/// Clears `len` bytes at the work pointer stored at `field` (`MI_CpuFill8(*field, 0, len)`).
fn fill_at(w: &mut SvFlowWork, waddr: u32, field: usize, len: usize) {
    let o = w.u32_at(field).wrapping_sub(waddr) as usize;
    w.mem[o..o + len].fill(0);
}

/// C: `FUN_overlay_d_93__021c85d4(work, idx, clear)`: points the move-record pointers at block
/// `idx` (`b = work + 0x874 + idx * 0x290`): `+0x850..+0x868 = b + 0x48 * k` (k = 0..6),
/// `+0x4A8 = b + 0x1F8`, `+0x1AB0 / +0x1AB4 = b + 0x200 / 0x214`, `+0x4B4 / +0x4AC / +0x4B0 =
/// b + 0x228 / 0x22E / 0x241`, `+0x86C / +0x870 = b + 0x254 / 0x270`; copies `b[0x28C]` to `work[0x77F]`
/// and bit 0 of `b[0x28D]` / `b[0x28E]` to bits 3 / 4 of `work[0x78A]`. With `clear`: clears the
/// seven 0x48-byte records (overlay-95 veneers `87a0`..`8800`), core2 `021bd1e0(*0x4A8)`, zero
/// fills of the smaller ones, `work[0x77F] := 6` and bits 3 / 4 of `work[0x78A]` cleared.
pub fn move_records_select<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    waddr: u32,
    idx: u32,
    clear: u32,
) {
    let b = 0x874u32.wrapping_add(idx.wrapping_mul(0x290));
    let a = waddr.wrapping_add(b);
    for k in 0..7u32 {
        w.set_u32_at(0x850 + 4 * k as usize, a.wrapping_add(0x48 * k));
    }
    w.set_u32_at(0x4A8, a + 0x1F8);
    w.set_u32_at(0x1AB0, a + 0x200);
    w.set_u32_at(0x1AB4, a + 0x214);
    w.set_u32_at(0x4B4, a + 0x228);
    w.set_u32_at(0x4AC, a + 0x22E);
    w.set_u32_at(0x4B0, a + 0x241);
    w.set_u32_at(0x86C, a + 0x254);
    w.set_u32_at(0x870, a + 0x270);
    let b = b as usize;
    w.set_u8_at(0x77F, w.u8_at(b + 0x28C));
    let f = (w.u8_at(OFS_FLAGS_78A) & !8) | ((w.u8_at(b + 0x28D) & 1) << 3);
    w.set_u8_at(OFS_FLAGS_78A, f);
    let f = (w.u8_at(OFS_FLAGS_78A) & !0x10) | ((w.u8_at(b + 0x28E) & 1) << 4);
    w.set_u8_at(OFS_FLAGS_78A, f);
    if clear == 0 {
        return;
    }
    for (k, veneer) in [
        0x021C_87A0u32,
        0x021C_87B0,
        0x021C_87C0,
        0x021C_87D0,
        0x021C_87E0,
        0x021C_87F0,
        0x021C_8800,
    ]
    .into_iter()
    .enumerate()
    {
        // record k (`work[0x850 + 4 k]`)
        let rec = w.u32_at(0x850 + 4 * k);
        env.callw(veneer, &mut [Arg::PAt(rec)], w);
    }
    let rec = w.u32_at(0x4A8);
    env.callw(0x021B_D1E0, &mut [Arg::PAt(rec)], w);
    fill_at(w, waddr, 0x1AB0, 0x14);
    fill_at(w, waddr, 0x1AB4, 0x14);
    fill_at(w, waddr, 0x4B4, 6);
    fill_at(w, waddr, 0x4AC, 0x13);
    fill_at(w, waddr, 0x4B0, 0x13);
    fill_at(w, waddr, 0x86C, 0x1C);
    fill_at(w, waddr, 0x870, 0x1C);
    w.set_u8_at(0x77F, 6);
    let f = w.u8_at(OFS_FLAGS_78A) & !8;
    w.set_u8_at(OFS_FLAGS_78A, f);
    let f = w.u8_at(OFS_FLAGS_78A) & !0x10;
    w.set_u8_at(OFS_FLAGS_78A, f);
}

/// Addresses of ported functions; stubs at these addresses become inactive.
pub const PORTED: &[u32] = &[
    0x021C7CBC, 0x021C7CC8, 0x021C7CD4, 0x021C7CDC, 0x021C7CE8, 0x021C7CF4, 0x021C7D00, 0x021C7D54,
    0x021C7DAC, 0x021C7E14, 0x021C7EC4, 0x021C7ED8, 0x021C7EEC, 0x021C7F04, 0x021C7F24, 0x021C7F4C,
    0x021C7F68, 0x021C7FCC, 0x021C7FE0, 0x021C8010, 0x021C8020, 0x021C8030, 0x021C804C, 0x021C80AC,
    0x021C8114, 0x021C8120, 0x021C812C, 0x021C8130, 0x021C8138, 0x021C8140, 0x021C815C, 0x021C8168,
    0x021C8174, 0x021C8180, 0x021C818C, 0x021C81C8, 0x021C81D0, 0x021C8234, 0x021C824C, 0x021C8288,
    0x021C8290, 0x021C82B8, 0x021C82D0, 0x021C8300, 0x021C830C, 0x021C831C, 0x021C8338, 0x021C8348,
    0x021C8374, 0x021C83A0, 0x021C83CC, 0x021C83D4, 0x021C83DC, 0x021C83E8, 0x021C8408, 0x021C8434,
    0x021C846C, 0x021C8494, 0x021C84AC, 0x021C84D0, 0x021C84D8, 0x021C84E0, 0x021C84E8, 0x021C84F4,
    0x021C8534, 0x021C8588, 0x021C859C, 0x021C85B8, 0x021C85D4,
];

#[cfg(test)]
pub(crate) mod tests;
