//! `btl_pokeparam.c` (asm/battle_pokeparam.s 0x021D4D5C-0x021D6EAC): the battle Pokemon. The
//! integration keeps every `BTL_POKEPARAM` as a typed [`BattleMon`] in [`Env::mons`] (by its
//! address); the accessors run the verified `battle_mon` ports on it. Argument mapping from the
//! call sites (argument 0 = the battle Pokemon's address).

use super::{Bufs, Env};
use crate::battle_mon::BattleMon;
use crate::server_flow::env::{Arg, Lent};
use crate::sick_cont::SickCont;

impl Env {
    /// Runs `f` on the battle Pokemon at `addr` (removed from the table while it runs).
    pub fn with_mon<R>(&mut self, addr: u32, f: impl FnOnce(&mut Env, &mut BattleMon) -> R) -> R {
        let mut m = self.mons.remove(&addr).unwrap_or_else(|| {
            panic!(
                "B16: no battle Pokemon at {addr:#x} (or it is in use) ({})",
                self.backtrace()
            )
        });
        let r = f(self, &mut m);
        self.mons.insert(addr, m);
        r
    }
}

fn gap(addr: u32) -> ! {
    panic!("B16 gap: BattleMon {addr:#010x}: argument mapping not wired")
}

pub fn call(env: &mut Env, lent: &mut Lent<'_>, addr: u32, args: &mut [Arg<'_>]) -> Option<u32> {
    let v: Vec<u32> = (0..6)
        .map(|i| match args.get(i) {
            Some(Arg::V(x)) | Some(Arg::PAt(x)) => *x,
            _ => 0,
        })
        .collect();
    let mon = v[0];
    let mut unp = env.unp();
    macro_rules! m {
        (|$b:ident| $e:expr) => {
            env.with_mon(mon, |_, $b| $e)
        };
    }
    Some(match addr {
        // ---- create / free ----
        0x021D_4D84 => {
            let a = env
                .objs
                .borrow_mut()
                .alloc(crate::battle_mon::BPP_SIZE as u32, "BattleMon");
            // the bytes live in `Env::mons`; the block only reserves the address
            let _ = env.objs.borrow_mut().take(a);
            let b = BattleMon::new(v[0], v[1] as u8, &mut env.calc, &mut unp);
            env.mons.insert(a, b);
            a
        }
        0x021D_4D5C | 0x021D_53B8 => {
            env.mons.remove(&mon).expect("free of an unknown BattleMon");
            env.objs.borrow_mut().blocks.remove(&mon);
            0
        }
        // ---- plain getters ----
        0x021D_5440 => m!(|b| b.poke_id() as u32),
        0x021D_5444 => m!(|b| b.species() as u32),
        0x021D_5448 => m!(|b| b.move_count() as u32),
        0x021D_5450 => m!(|b| b.orig_move_count() as u32),
        0x021D_5478 => m!(|b| b.used_move_count() as u32),
        0x021D_54B4 => m!(|b| b.distinct_moves_used() as u32),
        0x021D_54C0 => m!(|b| b.move_id(v[1]) as u32),
        0x021D_54D0 => m!(|b| b.orig_move_id(v[1]) as u32),
        0x021D_54E0 => m!(|b| b.move_used_flag(v[1]) as u32),
        0x021D_554C => m!(|b| b.move_pp_missing(v[1]) as u32),
        0x021D_5568 => m!(|b| b.orig_move_pp_missing(v[1]) as u32),
        0x021D_5584 => m!(|b| b.move_pp(v[1]) as u32),
        0x021D_55A0 => m!(|b| b.move_pp_of(v[1]) as u32),
        0x021D_55D8 => m!(|b| b.move_pp_full(v[1], v[2] != 0) as u32),
        0x021D_5750 => m!(|b| b.knows_move(v[1]) as u32),
        0x021D_5778 => m!(|b| b.move_slot_of(v[1]) as u32),
        0x021D_57FC => m!(|b| b.type_pair() as u32),
        0x021D_5814 => m!(|b| b.has_type(v[1]) as u32),
        0x021D_5844 => m!(|b| b.get_pokemon()),
        0x021D_5864 => m!(|b| b.visible_mon()),
        0x021D_58A8 => m!(|b| b.get_unstaged(v[1], &mut unp)),
        0x021D_59B4 => m!(|b| b.get(v[1], &mut unp)),
        0x021D_5AD8 => m!(|b| b.get_for_crit(v[1], &mut unp)),
        0x021D_5B40 | 0x021D_5B44 => m!(|b| b.held_item() as u32),
        0x021D_5B48 => m!(|b| b.is_full_hp(&mut unp) as u32),
        0x021D_5B68 => m!(|b| b.is_fainted(&mut unp) as u32),
        0x021D_5B7C => m!(|b| b.is_alive(&mut unp) as u32),
        0x021D_5BA4 => m!(|b| b.u16_146() as u32),
        0x021D_5BB0 => m!(|b| b.flag_a(v[1]) as u32),
        0x021D_5BDC => m!(|b| b.flag_b(v[1]) as u32),
        0x021D_5C08 => m!(|b| b.first_flag_b_3546()),
        0x021D_5C30 => m!(|b| b.any_flag_b_3546() as u32),
        0x021D_5C44 => m!(|b| b.hp_percent_fx() as u32),
        0x021D_5D14 => m!(|b| b.can_change_stage(v[1], v[2] as i32) as u32),
        0x021D_5D50 => m!(|b| b.stage_headroom(v[1]) as u32),
        0x021D_5D68 => m!(|b| b.has_negative_stage() as u32),
        0x021D_5F14 => m!(|b| b.crit_stage() as u32),
        0x021D_62A8 => m!(|b| b.major_status()),
        0x021D_62C4 => m!(|b| b.has_sick(v[1]) as u32),
        0x021D_62D8 => m!(|b| b.sick_param(v[1]) as u32),
        0x021D_6314 => m!(|b| b.get_sick_cont(v[1]).0),
        0x021D_631C => m!(|b| b.sick_turns(v[1]) as u32),
        0x021D_6324 => m!(|b| b.sick_damage(v[1], &mut unp)),
        0x021D_679C => m!(|b| b.move_streak() as u32),
        0x021D_67A8 => m!(|b| b.last_move() as u32),
        0x021D_67B0 => m!(|b| b.last_move_type() as u32),
        0x021D_67B8 => m!(|b| b.last_move_orig() as u32),
        0x021D_67C4 => m!(|b| b.last_move_target() as u32),
        0x021D_67D0 => m!(|b| b.flag_1b_bit7() as u32),
        0x021D_67E8 => m!(|b| b.weight() as u32),
        0x021D_6920 => m!(|b| b.history_count(v[1]) as u32),
        0x021D_69AC => m!(|b| b.u8_157(v[1]) as u32),
        0x021D_6B6C => m!(|b| b.exp_for_max_level(&mut unp)),
        0x021D_6C24 => m!(|b| b.is_illusion_active() as u32),
        0x021D_6D54 => m!(|b| b.is_transformed() as u32),
        0x021D_6D7C => m!(|b| b.has_substitute() as u32),
        0x021D_6D90 => m!(|b| b.substitute_hp() as u32),
        0x021D_6E04 => m!(|b| b.faced_count() as u32),
        0x021D_6E0C => m!(|b| b.faced(v[1]) as u32),
        0x021D_6E74 => m!(|b| b.has_1f6() as u32),
        // ---- setters ----
        0x021D_5848 => m!(|b| {
            b.set_illusion_mon(v[1]);
            0
        }),
        0x021D_5854 | 0x021D_6C2C => m!(|b| {
            b.clear_illusion_mon();
            0
        }),
        0x021D_51CC => m!(|b| {
            b.move_clear_used(v[1]);
            0
        }),
        0x021D_5364 => m!(|b| {
            b.reset_switch_state();
            0
        }),
        0x021D_539C => m!(|b| {
            b.clear_157();
            0
        }),
        0x021D_5150 => m!(|b| {
            b.moves_restore();
            0
        }),
        0x021D_505C => m!(|b| {
            b.moves_store(&mut unp);
            0
        }),
        0x021D_50B4 => m!(|b| {
            b.moves_reload(&mut unp);
            0
        }),
        0x021D_5608 => m!(|b| {
            b.move_spend_pp(v[1], v[2], &mut unp);
            0
        }),
        0x021D_5640 => m!(|b| {
            b.move_spend_orig_pp(v[1], v[2], &mut unp);
            0
        }),
        0x021D_5674 => m!(|b| b.move_restore_pp(v[1], v[2]) as u32),
        0x021D_56A0 => m!(|b| b.move_restore_orig_pp(v[1], v[2]) as u32),
        0x021D_56CC => m!(|b| {
            b.move_mark_used(v[1]);
            0
        }),
        0x021D_5970 => m!(|b| {
            b.set_stat(v[1], v[2] as u16, &mut unp);
            0
        }),
        0x021D_5DC0 => m!(|b| b.raise_stage(v[1], v[2])),
        0x021D_5E38 => m!(|b| b.lower_stage(v[1], v[2])),
        0x021D_5EA8 => m!(|b| {
            b.set_stage(v[1], v[2]);
            0
        }),
        0x021D_5EFC => m!(|b| b.clear_negative_stages() as u32),
        0x021D_5F08 => m!(|b| {
            b.reset_stages();
            0
        }),
        0x021D_5F38 => m!(|b| b.add_crit_boost(v[1] as i32) as u32),
        0x021D_5F90 => m!(|b| {
            b.damage(v[1]);
            0
        }),
        0x021D_5FA4 => m!(|b| {
            b.heal(v[1]);
            0
        }),
        0x021D_5FB8 => m!(|b| {
            b.faint();
            0
        }),
        0x021D_5FC0 => m!(|b| {
            b.set_flag_a(v[1]);
            0
        }),
        0x021D_5FE4 => m!(|b| {
            b.set_flag_b(v[1]);
            0
        }),
        0x021D_6008 => m!(|b| {
            b.clear_flag_b(v[1]);
            0
        }),
        0x021D_6034 => m!(|b| {
            b.set_sick(v[1], SickCont(v[2]));
            0
        }),
        0x021D_6130 => m!(|b| b.tick_sleep() as u32),
        0x021D_61A8 => m!(|b| b.tick_confusion() as u32),
        0x021D_61F0 => m!(|b| {
            b.cure_major_status();
            0
        }),
        0x021D_6238 => m!(|b| {
            b.cure_sick(v[1]);
            0
        }),
        0x021D_6264 => m!(|b| {
            b.cure_sicks_linked_to(v[1]);
            0
        }),
        0x021D_63B8 => m!(|b| {
            b.clear_sicks(v[1] != 0);
            0
        }),
        0x021D_63EC => m!(|b| {
            b.set_148(v[1] as u16);
            0
        }),
        0x021D_6408 => m!(|b| {
            b.on_turn_start();
            0
        }),
        0x021D_6440 => m!(|b| {
            b.clear_flag_a(v[1]);
            0
        }),
        0x021D_646C => m!(|b| {
            b.reset_on_switch_out(&mut unp);
            0
        }),
        0x021D_64F0 => m!(|b| {
            b.reset_on_battle_end(&mut unp);
            0
        }),
        0x021D_6580 => m!(|b| {
            b.reset_on_replace(&mut unp);
            0
        }),
        0x021D_66B0 => m!(|b| {
            b.set_type_pair(v[1] as u16);
            0
        }),
        0x021D_66D0 => m!(|b| {
            b.set_ability(v[1] as u16);
            0
        }),
        0x021D_66D8 => m!(|b| {
            b.change_form(v[1] as u8, &mut unp);
            0
        }),
        0x021D_6730 => m!(|b| {
            b.set_u14(v[1] as u16);
            0
        }),
        0x021D_6738 => m!(|b| {
            b.clear_u14();
            0
        }),
        0x021D_67D8 => m!(|b| {
            b.set_weight(v[1] as u16);
            0
        }),
        0x021D_67F4 => m!(|b| {
            b.clear_history();
            0
        }),
        0x021D_6824 => m!(|b| {
            b.advance_history();
            0
        }),
        0x021D_69A0 => m!(|b| {
            b.set_u8_157(v[1], v[2] as u8);
            0
        }),
        0x021D_6B84 => m!(|b| {
            b.store_to_party(v[1] != 0, &mut unp);
            0
        }),
        0x021D_6BFC => m!(|b| {
            b.refresh_from_party(&mut unp);
            0
        }),
        0x021D_6C3C => m!(|b| {
            b.set_pokemon(v[1], &mut unp);
            0
        }),
        0x021D_6D5C => m!(|b| {
            b.make_substitute(v[1] as u16);
            0
        }),
        0x021D_6DC4 => m!(|b| {
            b.clear_faced();
            0
        }),
        0x021D_6DCC => m!(|b| {
            b.add_faced(v[1] as u8);
            0
        }),
        0x021D_6E24 => m!(|b| {
            b.set_pp_0x98(v[1], &mut unp);
            0
        }),
        0x021D_6E40 => m!(|b| {
            b.set_1f6(v[1] as u8, v[2] as u16);
            0
        }),
        0x021D_6E88 => m!(|b| {
            b.clear_1f6();
            0
        }),
        0x021D_6EA0 => m!(|b| {
            b.recalc_party_stats(&mut unp);
            0
        }),
        0x021D_6D9C => {
            let mut d = Bufs::new(args).get(env, lent, 1, 2);
            let mut dmg = u16::from_le_bytes([d[0], d[1]]);
            let r = env.with_mon(mon, |_, b| b.hit_substitute(&mut dmg));
            d[..2].copy_from_slice(&dmg.to_le_bytes());
            Bufs::new(args).put(env, lent, 1, &d);
            r as u32
        }
        // two battle Pokemon
        0x021D_6C50 => {
            let t = env.mons.get(&v[1]).expect("transform target").clone();
            env.with_mon(mon, |_, b| b.transform_into(&t, &mut unp)) as u32
        }
        // 021d6744(mon, target, consecutive, type, move, orig move (stack words))
        0x021D_6744 => {
            m!(|b| b.record_move(v[1] as u8, v[2] != 0, v[3] as u8, v[4] as u16, v[5] as u16));
            0
        }
        // 021d6064(mon, sick, &prev cont, &ended): end-of-turn tick of one condition
        0x021D_6064 => {
            let want = |i: usize| !matches!(args.get(i), Some(Arg::V(0)) | None);
            let (w2, w3) = (want(2), want(3));
            let (mut prev, mut ended) = (SickCont(0), 0u32);
            let sick = v[1];
            let r = env.with_mon(mon, |_, b| {
                b.tick_sick(sick, w2.then_some(&mut prev), w3.then_some(&mut ended))
            });
            if w2 {
                Bufs::new(args).put(env, lent, 2, &prev.0.to_le_bytes());
            }
            if w3 {
                Bufs::new(args).put(env, lent, 3, &ended.to_le_bytes());
            }
            r as u32
        }
        // 021d6848(mon, &record): appends an 8-byte action record to this turn's history
        0x021D_6848 => {
            // the caller's 7-byte local (byte 7 is uninitialised stack in the original: 0)
            let mut b = Bufs::new(args).get(env, lent, 1, 8);
            b.resize(8, 0);
            let r: crate::battle_mon::ActionRecord =
                std::array::from_fn(|i| u16::from_le_bytes([b[2 * i], b[2 * i + 1]]));
            m!(|bm| bm.push_history(&r));
            0
        }
        0x021D_54F4 | 0x021D_65C8 | 0x021D_6940 | 0x021D_69B8 | 0x021D_590C | 0x021D_5528
        | 0x021D_51DC | 0x021D_5734 | 0x021D_5244 | 0x021D_52B4 | 0x021D_533C | 0x021D_4EFC
        | 0x021D_4F84 | 0x021D_6E50 => gap(addr),
        _ => return None,
    })
}
