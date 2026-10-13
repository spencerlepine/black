//! Overlay 96 (the trainer AI, ai1): adapters transcribed from the area's differential-test
//! `call` table; the ports work on a `server3::Mem` over the object table.

use super::Env;
use crate::ai::ai1::*;
use crate::ai::AiEnv;
use crate::server_flow::env::{Arg, FlowEnv, Lent};

impl AiEnv for Env {
    /// `OS_GetTick` (0x020870D0): a deterministic clock (one tick per call).
    fn call64(&mut self, addr: u32, args: &mut [Arg<'_>]) -> u64 {
        match addr {
            0x0208_70D0 => {
                self.tick += 0x1000;
                self.tick
            }
            _ => self.call(addr, args) as u64,
        }
    }
}

/// The AI ports that are wired end to end (B16): object create / start / answers.
const WIRED: &[u32] = &[0x0218_7F00, 0x0218_80BC, 0x0218_8134, 0x0218_8140];

/// An AI function whose first argument is a VM the environment created: the VM is the current
/// one for the call (the AI ports pass the VM to the VM routines as a pass-through `P`; B16 gap).
pub fn call(env: &mut Env, lent: &mut Lent<'_>, addr: u32, args: &mut [Arg<'_>]) -> Option<u32> {
    // ai_delete from the client's destructor (`021cd8e0`) gets no argument from the port (the
    // original passes `c + 0xF0` in r0): the VM of the calling client
    if addr == 0x0218_8080 && args.is_empty() {
        let c = *env
            .client_stack
            .last()
            .expect("ai_delete outside a client port");
        let vm = env.r32(c + 0xF0);
        return call(env, lent, addr, &mut [Arg::V(vm)]);
    }
    let vm = match args.first() {
        Some(Arg::V(x)) | Some(Arg::PAt(x)) if env.vms.contains(x) => Some(*x),
        _ => None,
    };
    if let Some(vm) = vm {
        env.ai_vm.push(vm);
    }
    let r = if WIRED.contains(&addr) {
        call_inner(env, lent, addr, args)
    } else {
        // the AI ports pass the BattleMon / server / VM pointers they got from a previous call as
        // pass-through `P` (~150 sites, B16 gap): the rest of overlay 96 runs as original Thumb code
        let v: Vec<u32> = args
            .iter()
            .map(|a| match a {
                Arg::V(x) | Arg::PAt(x) => *x,
                _ => panic!(
                    "B16 gap: AI {addr:#x} (interpreted) gets a pointer the caller does not know"
                ),
            })
            .collect();
        Some(env.run_thumb(addr, &v))
    };
    if vm.is_some() {
        env.ai_vm.pop();
    }
    r
}

fn call_inner(env: &mut Env, lent: &mut Lent<'_>, addr: u32, args: &mut [Arg<'_>]) -> Option<u32> {
    // ai_start(vm, usable, pos): the usable-move flags are a local of the caller; mapped for the call
    if addr == 0x0218_80BC && matches!(args.get(1), Some(Arg::PBuf(_)) | Some(Arg::Out(_))) {
        let b = super::Bufs::new(args).get(env, lent, 1, 4);
        let tmp = env
            .objs
            .borrow_mut()
            .alloc(4, "ai_start usable flags (caller local)");
        env.objs.borrow_mut().write(tmp, &b);
        let (vm, pos) = (super::val(args, 0), super::val(args, 2));
        let mut a2 = [Arg::V(vm), Arg::V(tmp), Arg::V(pos)];
        let r = call_inner(env, lent, addr, &mut a2);
        env.objs.borrow_mut().free(tmp);
        return r;
    }
    let v: Vec<u32> = (0..8)
        .map(|i| match args.get(i) {
            Some(Arg::V(x)) | Some(Arg::PAt(x)) => *x,
            Some(Arg::P) | None => 0,
            Some(_) => panic!("B16: ai1 {addr:#x} gets a buffer argument"),
        })
        .collect();
    let w1 = v[1];
    let mut mem = env.mem3();
    let e = env;
    let m = &mut mem;
    let (vm, ai) = (v[0], v[1]);
    let k = v[2];
    let _ = (w1, vm, ai, k);
    let r: u32 = match addr {
        // shared helpers
        0x0218_A0F4 => branch_if(e, m, v[0], v[1], v[2], v[3], v[4]),
        0x0218_A160 => pos_of(e, m, v[0], v[1]),
        0x0218_A220 => mon_trait(e, m, v[0], v[1], v[2]),
        0x0218_A2FC => mon_at(e, m, v[0], w1),
        0x0218_A308 => party_mon(e, v[0], v[1]),
        0x0218_A310 => reset_scores(m, v[0]),
        0x0218_A324 => note_last_move(e, m, v[0]),
        0x0218_A378 => waza_param(e, v[0], v[1], v[2]),
        0x0218_A384 => waza_flag(e, v[0], v[1], v[2]),
        0x0218_A390 => best_score(e, m, v[0], v[1], v[2], v[3]),
        0x0218_A3F4 => table_lookup(e, m, v[0], v[1], v[2]),
        0x0218_A41C => table_create(e, m, v[0]),
        0x0218_A458 => table_free(e, m),
        0x0218_A494 => script_acquire(e, m, v[0]),
        0x0218_A528 => script_release(e, m, v[0]),
        // create / delete / start / think (the think functions read the OS tick through `AiEnv::call64`)
        0x0218_7F00 => ai_create(e, m, v[0], v[1], v[2], v[3], v[4]),
        0x0218_7FB0 => ai_think(e, m, vm),
        0x0218_814C => think_single(e, m, vm, ai),
        0x0218_8284 => think_multi(e, m, vm, ai),
        0x0218_85BC => run_moves(e, m, vm, ai),
        0x0218_8080 => ai_delete(e, m, vm),
        0x0218_80BC => ai_start(e, m, vm, v[1], v[2]),
        0x0218_8134 => chosen_move(e, m),
        0x0218_8140 => chosen_target(e, m),
        // helpers of the command wrappers: (vm, ai, kind)
        0x0218_875C => cmd_if_rand(e, m, vm, k),
        0x0218_87F4 => if_hp(e, m, vm, ai, k),
        0x0218_885C => if_status(e, m, vm, ai, k),
        0x0218_88BC => if_cond(e, m, vm, ai, k),
        0x0218_8924 => if_cond5(e, m, vm, ai, k),
        0x0218_89A0 => if_5bdc(e, m, vm, ai, k),
        0x0218_8A08 => if_side(e, m, vm, ai, k),
        0x0218_8AB4 => if_result(e, m, vm, ai, k),
        0x0218_8AF8 => if_move(e, m, vm, ai, k),
        0x0218_8FCC => if_party_status(e, m, vm, ai, k),
        0x0218_908C => if_move_effect(e, m, vm, ai, k),
        0x0218_9100 => if_value(e, m, vm, ai, k),
        0x0218_9170 => if_hp_vs_damage(e, m, vm, ai, k),
        0x0218_91F4 => if_knows_move(e, m, vm, ai, k),
        0x0218_92F8 => if_knows_effect(e, m, vm, ai, k),
        0x0218_9640 => if_target_cond_b(e, m, vm, ai, k),
        0x0218_9E90 => if_fainted(e, m, vm, ai, k),
        0x0218_9F98 => if_turn_rand(e, m, vm, ai, k),
        0x0218_A0A4 => if_pos_vs_value(e, m, vm, ai, k),
        // command handlers: (vm, ai)
        0x0218_871C => cmd_00(e, m, vm, ai),
        0x0218_872C => cmd_01(e, m, vm, ai),
        0x0218_873C => cmd_02(e, m, vm, ai),
        0x0218_874C => cmd_03(e, m, vm, ai),
        0x0218_878C => cmd_add_score(e, m, ai),
        0x0218_87B4 => cmd_05(e, m, vm, ai),
        0x0218_87C4 => cmd_06(e, m, vm, ai),
        0x0218_87D4 => cmd_07(e, m, vm, ai),
        0x0218_87E4 => cmd_08(e, m, vm, ai),
        0x0218_883C => cmd_09(e, m, vm, ai),
        0x0218_884C => cmd_10(e, m, vm, ai),
        0x0218_889C => cmd_11(e, m, vm, ai),
        0x0218_88AC => cmd_12(e, m, vm, ai),
        0x0218_8904 => cmd_13(e, m, vm, ai),
        0x0218_8914 => cmd_14(e, m, vm, ai),
        0x0218_8980 => cmd_15(e, m, vm, ai),
        0x0218_8990 => cmd_16(e, m, vm, ai),
        0x0218_89E8 => cmd_17(e, m, vm, ai),
        0x0218_89F8 => cmd_18(e, m, vm, ai),
        0x0218_8A54 => cmd_19(e, m, vm, ai),
        0x0218_8A64 => cmd_20(e, m, vm, ai),
        0x0218_8A74 => cmd_21(e, m, vm, ai),
        0x0218_8A84 => cmd_22(e, m, vm, ai),
        0x0218_8A94 => cmd_23(e, m, vm, ai),
        0x0218_8AA4 => cmd_24(e, m, vm, ai),
        0x0218_8AD8 => cmd_25(e, m, vm, ai),
        0x0218_8AE8 => cmd_26(e, m, vm, ai),
        0x0218_8B1C => cmd_if_result_in(e, m, vm, ai),
        0x0218_8B64 => cmd_if_result_not_in(e, m, vm, ai),
        0x0218_8BB0 => cmd_if_has_damaging(e, m, vm, ai),
        0x0218_8C1C => cmd_if_no_damaging(e, m, vm, ai),
        0x0218_8C88 => cmd_get_8138(e, m, ai),
        0x0218_8CA0 => cmd_get_type(e, m, ai),
        0x0218_8D48 => cmd_move_1d8(e, m, ai),
        0x0218_8D5C => cmd_cmp_scores(e, m, ai),
        0x0218_8E00 => cmd_last_move(e, m, ai),
        0x0218_8E24 => cmd_cmp_speed(e, m, vm, ai),
        0x0218_8E90 => cmd_count_party_5b7c(e, m, ai),
        0x0218_8F08 => cmd_result_from_score(m, ai),
        0x0218_8F14 => cmd_move_effect(e, m, ai),
        0x0218_8F2C => cmd_trait(e, m, ai),
        0x0218_8F50 => cmd_status(m, ai),
        0x0218_8F58 => cmd_if_effectiveness(e, m, vm, ai),
        0x0218_8FAC => cmd_45(e, m, vm, ai),
        0x0218_8FBC => cmd_46(e, m, vm, ai),
        0x0218_905C => cmd_get_f884(e, m, ai),
        0x0218_906C => cmd_48(e, m, vm, ai),
        0x0218_907C => cmd_49(e, m, vm, ai),
        0x0218_90C0 => cmd_50(e, m, vm, ai),
        0x0218_90D0 => cmd_51(e, m, vm, ai),
        0x0218_90E0 => cmd_52(e, m, vm, ai),
        0x0218_90F0 => cmd_53(e, m, vm, ai),
        0x0218_9150 => cmd_54(e, m, vm, ai),
        0x0218_9160 => cmd_55(e, m, vm, ai),
        0x0218_91D4 => cmd_56(e, m, vm, ai),
        0x0218_91E4 => cmd_57(e, m, vm, ai),
        0x0218_92D8 => cmd_58(e, m, vm, ai),
        0x0218_92E8 => cmd_59(e, m, vm, ai),
        0x0218_93AC | 0x0218_93CC | 0x0218_93D4 | 0x0218_94F8 | 0x0218_9D30 | 0x0218_9D38
        | 0x0218_9D40 => cmd_nop(m, ai),
        0x0218_93B4 => cmd_set_abort(m, ai),
        0x0218_93DC => cmd_ability(e, m, ai),
        0x0218_9400 => cmd_ability_lookup(e, m, ai),
        0x0218_9434 => cmd_value12(e, m, ai),
        0x0218_945C => cmd_5bdc0_zero(e, m, ai),
        0x0218_948C => cmd_69ac(e, m, ai),
        0x0218_94B4 => cmd_result_from_98(m, ai),
        0x0218_94C4 => cmd_result_from_9c(m, ai),
        0x0218_94D4 => cmd_6740(e, m, ai),
        0x0218_9500 => cmd_result_param3(e, m, ai),
        0x0218_9524 => cmd_result_effect(e, m, ai),
        0x0218_9548 => cmd_protect_count(e, m, ai),
        0x0218_9590 => cmd_goto(e, m, vm, ai),
        0x0218_95AC => cmd_end(e, m, ai),
        0x0218_95C4 => cmd_cmp_value_f(e, m, vm, ai),
        0x0218_9620 => cmd_79(e, m, vm, ai),
        0x0218_9630 => cmd_80(e, m, vm, ai),
        0x0218_9668 => cmd_if_same_side(e, m, vm, ai),
        0x0218_9694 => cmd_has_type(e, m, ai),
        0x0218_96E0 => cmd_trait_is(e, m, ai),
        0x0218_9718 => cmd_if_5bdc_d(e, m, vm, ai),
        0x0218_9758 => cmd_if_ability(e, m, vm, ai),
        0x0218_979C => cmd_if_f948(e, m, vm, ai),
        0x0218_97CC => cmd_side(e, m, ai),
        0x0218_9800 => cmd_if_party_hp(e, m, vm, ai),
        0x0218_989C => cmd_if_party_no_pp(e, m, vm, ai),
        0x0218_9978 => cmd_ability_lookup_a(e, m, ai),
        0x0218_99C4 => cmd_slot_usable(e, m, ai),
        0x0218_99DC => cmd_if_5478(e, m, vm, ai),
        0x0218_9A2C => cmd_move_param2(e, m, ai),
        0x0218_9A44 => cmd_target_last_param2(e, m, ai),
        0x0218_9A6C => cmd_81d0(e, m, ai),
        0x0218_9A9C => cmd_5ba4(e, m, ai),
        0x0218_9AC0 => cmd_if_party_better(e, m, vm, ai),
        0x0218_9B78 => cmd_if_super_effective(e, m, vm, ai),
        0x0218_9BFC => cmd_if_last_move_beats(e, m, vm, ai),
        0x0218_9C8C => cmd_sum_boosts(e, m, ai),
        0x0218_9CEC => cmd_value_diff(e, m, ai),
        0x0218_9D48 => cmd_party_outscores(e, m, ai),
        0x0218_9E70 => cmd_106(e, m, vm, ai),
        0x0218_9E80 => cmd_107(e, m, vm, ai),
        0x0218_9ED0 => cmd_value11(e, m, ai),
        0x0218_9EF8 => cmd_if_6d7c(e, m, vm, ai),
        0x0218_9F34 => cmd_species(e, m, ai),
        0x0218_9F58 => cmd_111(e, m, vm, ai),
        0x0218_9F68 => cmd_112(e, m, vm, ai),
        0x0218_9F78 => cmd_113(e, m, vm, ai),
        0x0218_9F88 => cmd_114(e, m, vm, ai),
        0x0218_9FC0 => cmd_switch_effect(e, m, vm, ai),
        0x0218_A034 => cmd_if_82f4(e, m, vm, ai),
        0x0218_A074 => cmd_117(e, m, vm, ai),
        0x0218_A084 => cmd_118(e, m, vm, ai),
        0x0218_A094 => cmd_119(e, m, vm, ai),
        _ => return None,
    };
    Some(r)
}
