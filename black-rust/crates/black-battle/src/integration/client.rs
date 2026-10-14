//! `btl_client.c` (client1-5, 0x021CD6EC-0x021D3F04): adapters to the ports, transcribed from the
//! areas' differential-test `call` tables. The client object (0x1B8 bytes) is held by the port
//! that runs on it; a pass-through `P` first argument is the client object of the calling client
//! port (`Env::client_stack`), and the call site must lend it.

use super::{Bufs, Env};
use crate::client::{client1, client2, client3, client4, client5, ClientUi};
use crate::server_flow::env::{Arg, Lent};

impl ClientUi for Env {}

/// Offsets in the client object.
pub mod ofs {
    pub const STEP: u32 = 0x28;
    pub const ADAPTER: u32 = 0x50;
    pub const HANDLER: u32 = 0xC8;
    pub const HANDLER_STATE: u32 = 0xCC;
    pub const QUEUE: u32 = 0x134;
    pub const CMD_ARGS: u32 = 0x138;
    pub const CMD_HANDLER: u32 = 0x17C;
    pub const CMD_STATE: u32 = 0x180;
}

/// The client object address of argument 0 (for the other host modules).
pub fn client_addr_of(env: &Env, lent: &Lent<'_>, args: &[Arg<'_>]) -> u32 {
    client_addr(env, lent, args)
}

/// The client object address of argument 0.
fn client_addr(env: &Env, lent: &Lent<'_>, args: &[Arg<'_>]) -> u32 {
    match args.first() {
        Some(Arg::V(a)) | Some(Arg::PAt(a)) => *a,
        Some(Arg::PHost(h)) => env.host_addr(lent, *h),
        Some(Arg::P) | None => *env.client_stack.last().unwrap_or_else(|| {
            panic!(
                "B16: client P argument outside a client port ({})",
                env.backtrace()
            )
        }),
        Some(_) => panic!("B16: client object passed as a buffer"),
    }
}

fn vals(args: &[Arg<'_>]) -> Vec<u32> {
    args.iter()
        .map(|a| match a {
            Arg::V(v) | Arg::PAt(v) => *v,
            _ => 0,
        })
        .chain(std::iter::repeat(0))
        .take(10)
        .collect()
}

/// Runs a client port: `bufs` = (argument index, length) of its pointer arguments (fetched before
/// and stored back after), `f(env, c, c_addr, vals, bufs)`.
fn run<R>(
    env: &mut Env,
    lent: &mut Lent<'_>,
    args: &mut [Arg<'_>],
    bufs: &[(usize, usize)],
    f: impl FnOnce(&mut Env, &mut [u8], u32, &[u32], &mut [Vec<u8>]) -> R,
) -> R {
    let a = client_addr(env, lent, args);
    let v = vals(args);
    let mut bv: Vec<Vec<u8>> = Vec::new();
    {
        let mut b = Bufs::new(args);
        for &(i, n) in bufs {
            bv.push(b.get(env, lent, i, n));
        }
    }
    // a state word that lives in the client object itself (`c + 0xCC` / `0xD4` / `0x180`): the
    // port gets it as a separate buffer, so a write through `c` (`set_callback` resets the
    // callback state) and one through the buffer are merged below (B16 adapter aliasing)
    let inside: Vec<Option<usize>> = bufs
        .iter()
        .map(|&(i, _)| {
            let p = match args.get(i) {
                Some(Arg::V(x)) | Some(Arg::PAt(x)) => Some(*x),
                Some(Arg::PHost(h)) => Some(env.host_addr(lent, *h)),
                _ => None,
            }?;
            (p >= a && p < a + client1::CLIENT_SIZE).then(|| (p - a) as usize)
        })
        .collect();
    let orig = bv.clone();
    env.client_stack.push(a);
    let mut c_after: Vec<Option<Vec<u8>>> = vec![None; bufs.len()];
    let r = env.with_obj(lent, a, |env, c| {
        let r = f(env, c, a, &v, &mut bv);
        for (k, o) in inside.iter().enumerate() {
            if let Some(o) = *o {
                c_after[k] = Some(c[o..o + bv[k].len()].to_vec());
            }
        }
        r
    });
    env.client_stack.pop();
    let mut b = Bufs::new(args);
    for (k, &(i, _)) in bufs.iter().enumerate() {
        if let Some(cv) = &c_after[k] {
            if *cv != orig[k] {
                // written through the client object (last): that value stands
                if bv[k] != orig[k] && bv[k] != *cv {
                    env.note("client state buffer written both directly and through the object (keeping the object's)".to_string());
                }
                continue;
            }
        }
        b.put(env, lent, i, &bv[k]);
    }
    r
}

pub fn call(env: &mut Env, lent: &mut Lent<'_>, addr: u32, args: &mut [Arg<'_>]) -> Option<u32> {
    // functions that do not take the client object
    match addr {
        0x021C_D6EC => return Some(0),
        // 021cf470(c, mon, rec): the client is unused; the record is in the lent client object
        0x021C_F470 => {
            let v = vals(args);
            return Some(client2::mon_answered(env, v[1], v[2]));
        }
        // 021cf58c(rec, c, mon)
        0x021C_F58C => {
            let v = vals(args);
            // the client (argument 1) may be the calling client port's pass-through `P`
            let ca = match args.get(1) {
                Some(Arg::P) => *env
                    .client_stack
                    .last()
                    .expect("client P outside a client port"),
                _ => v[1],
            };
            env.client_stack.push(ca);
            env.with_obj(lent, ca, |e, c| client2::struggle(e, v[0], c, v[2]));
            env.client_stack.pop();
            return Some(0);
        }
        0x021C_D6F0 => {
            let v = vals(args);
            let heap = match args.get(10) {
                Some(Arg::V(h)) => *h,
                _ => 0,
            };
            let ca = client1::CreateArgs {
                id: v[4],
                b11c: v[5],
                mode: v[6],
                b1aa: v[7],
                no_ai: v[8],
                heap,
            };
            let block = Bufs::new(args).get(env, lent, 9, 0x18);
            // the object the port allocates first is pre-allocated so it can write into it
            let pre = env
                .objs
                .borrow_mut()
                .alloc(client1::CLIENT_SIZE, "client object");
            env.pending_alloc = Some((pre, client1::CLIENT_SIZE));
            env.client_stack.push(pre);
            let r = env.with_obj(lent, pre, |e, c| {
                client1::create(e, v[0], v[1], v[2], v[3], &ca, &block, c)
            });
            env.client_stack.pop();
            assert!(
                env.pending_alloc.is_none() && r == pre,
                "B16: client create did not allocate its object first"
            );
            return Some(r);
        }
        _ => {}
    }
    let s = |n: usize| (n, 4usize);
    Some(match addr {
        // ---- client1 ----
        0x021C_D8E0 => run(env, lent, args, &[], |e, c, a, _, _| {
            client1::free(e, c, a);
            0
        }),
        0x021C_D914 => run(env, lent, args, &[], |_, c, _, _, _| {
            client1::set_done(c);
            0
        }),
        0x021C_D924 => run(env, lent, args, &[], |e, c, _, _, _| {
            client1::record_data(e, c)
        }),
        0x021C_D938 => run(env, lent, args, &[], |e, c, _, v, _| {
            client1::set_replay(e, c, v[1]);
            0
        }),
        0x021C_D95C => run(env, lent, args, &[], |_, c, _, v, _| {
            client1::set_view(c, v[1]);
            0
        }),
        0x021C_D960 => run(env, lent, args, &[], |_, c, _, v, _| {
            client1::set_a0(c, v[1]);
            0
        }),
        0x021C_D978 => run(env, lent, args, &[], |_, c, _, _, _| client1::adapter(c)),
        0x021C_D97C => run(env, lent, args, &[], |e, c, _, _, _| {
            client1::run_step(e, c)
        }),
        0x021C_D984 => run(env, lent, args, &[], |e, c, _, v, _| {
            client1::replay_restart(e, c, v[1]);
            0
        }),
        0x021C_D9B0 => run(env, lent, args, &[], |e, c, _, _, _| {
            client1::restart_main(e, c);
            0
        }),
        0x021C_D9C8 => run(env, lent, args, &[], |_, c, _, _, _| {
            client1::is_replay(c) as u32
        }),
        0x021C_D9DC => run(env, lent, args, &[], |_, c, _, _, _| {
            client1::cursor_u16(c) as u32
        }),
        0x021C_D9E0 => run(env, lent, args, &[], |_, c, _, _, _| {
            client1::is_waiting(c) as u32
        }),
        0x021C_D9F4 => run(env, lent, args, &[], |_, c, _, v, _| {
            client1::counter_add(c, v[1]);
            0
        }),
        0x021C_DA0C => run(env, lent, args, &[], |_, c, _, v, _| {
            client1::set_step(c, v[1]);
            0
        }),
        0x021C_DA18 => run(env, lent, args, &[], |e, c, a, _, _| {
            client1::main_step(e, c, a)
        }),
        0x021C_DC0C => run(env, lent, args, &[], |e, c, a, _, _| {
            client1::wait_step(e, c, a)
        }),
        0x021C_DD34 => run(env, lent, args, &[], |_, c, a, _, _| {
            client1::result_clear(c, a);
            0
        }),
        0x021C_DD50 => {
            let has = !matches!(args.get(2), Some(Arg::V(0)) | None);
            let bufs: &[(usize, usize)] = if has { &[(2, 4)] } else { &[] };
            run(env, lent, args, bufs, |_, c, _, v, b| {
                if has {
                    let mut aux = u32::from_le_bytes(b[0][..4].try_into().unwrap());
                    let r = client1::request_handler(c, v[1], Some(&mut aux));
                    b[0][..4].copy_from_slice(&aux.to_le_bytes());
                    r
                } else {
                    client1::request_handler(c, v[1], None)
                }
            })
        }
        0x021C_DD94 => run(env, lent, args, &[(1, 8)], |_, c, _, _, b| {
            client1::request_pair(c, &mut b[0]);
            0
        }),
        0x021C_DDA0 => run(env, lent, args, &[], |e, c, _, _, _| {
            client1::effect_pending(e, c)
        }),
        0x021C_DDCC => run(env, lent, args, &[], |_, c, _, _, _| {
            client1::flag_20(c) as u32
        }),
        // ---- client2 (request steps: (c, &state)) ----
        0x021C_DDDC => run(env, lent, args, &[s(1)], |e, c, _, _, b| {
            client2::req1_step(e, c, &mut b[0])
        }),
        0x021C_DE8C => run(env, lent, args, &[s(1)], |e, c, _, _, b| {
            client2::req1_replay(e, c, &mut b[0])
        }),
        0x021C_DECC => run(env, lent, args, &[], |e, c, _, _, _| {
            client2::store_active_hp(e, c);
            0
        }),
        0x021C_DEE8 => run(env, lent, args, &[], |e, c, _, _, _| {
            client2::hp_drop_percent(e, c)
        }),
        0x021C_DF1C => run(env, lent, args, &[], |e, c, _, _, _| {
            client2::active_mon(e, c)
        }),
        0x021C_DF58 => run(env, lent, args, &[s(1)], |e, c, a, _, b| {
            client2::req2_step(e, c, a, &mut b[0])
        }),
        0x021C_E044 => run(env, lent, args, &[], |e, c, a, _, _| {
            client2::req2_auto(e, c, a)
        }),
        0x021C_E040 => client2::req2_replay(),
        0x021C_E068 => run(env, lent, args, &[], |e, c, _, _, _| {
            client2::fade_start(e, c);
            0
        }),
        0x021C_E08C => run(env, lent, args, &[], |e, c, _, _, _| {
            client2::fade_poll(e, c)
        }),
        0x021C_E0BC => run(env, lent, args, &[], |e, c, _, v, _| {
            client2::fade_poll_then(e, c, v[1])
        }),
        0x021C_E0E0 => run(env, lent, args, &[], |e, c, _, _, _| {
            client2::fade_end(e, c);
            0
        }),
        0x021C_E0F8 => run(env, lent, args, &[], |e, c, _, v, _| {
            client2::sent_out_message(e, c, v[1], v[2]);
            0
        }),
        0x021C_E150 => run(env, lent, args, &[], |_, c, _, v, _| {
            client2::set_callback(c, v[1]);
            0
        }),
        0x021C_E160 => run(env, lent, args, &[], |e, c, _, _, _| {
            client2::run_callback(e, c)
        }),
        0x021C_E170 => run(env, lent, args, &[s(1)], |e, c, _, _, b| {
            client2::req3_step(e, c, &mut b[0])
        }),
        0x021C_E2E8 => run(env, lent, args, &[], |e, c, a, _, _| {
            client2::answer_skip(e, c, a);
            0
        }),
        0x021C_E308 => run(env, lent, args, &[], |e, c, _, _, _| {
            client2::select_start(e, c)
        }),
        0x021C_E3B4 => run(env, lent, args, &[], |e, c, _, _, _| {
            client2::print_message(e, c);
            0
        }),
        0x021C_E3C8 => run(env, lent, args, &[s(1)], |e, c, a, _, b| {
            client2::select_finish(e, c, a, &mut b[0])
        }),
        0x021C_E4D0 => run(env, lent, args, &[s(1)], |e, c, a, _, b| {
            client2::select_loop(e, c, a, &mut b[0])
        }),
        0x021C_E7D4 => run(env, lent, args, &[s(1)], |e, c, _, _, b| {
            client2::item_anim(e, c, &mut b[0])
        }),
        0x021C_EA94 => run(env, lent, args, &[s(1)], |e, c, a, _, b| {
            client2::fight_menu(e, c, a, &mut b[0])
        }),
        0x021C_ED4C => run(env, lent, args, &[s(1)], |e, c, _, _, b| {
            client2::target_page(e, c, &mut b[0])
        }),
        0x021C_EDC0 => run(env, lent, args, &[s(1)], |e, c, _, _, b| {
            client2::party_page(e, c, &mut b[0])
        }),
        0x021C_EEB8 => run(env, lent, args, &[s(1)], |e, c, _, _, b| {
            client2::bag_page(e, c, &mut b[0])
        }),
        0x021C_EFFC => run(env, lent, args, &[], |e, c, _, _, _| {
            client2::single_live_mon(e, c)
        }),
        0x021C_F078 => run(env, lent, args, &[s(1)], |e, c, a, _, b| {
            client2::run_menu(e, c, a, &mut b[0])
        }),
        0x021C_F220 => run(env, lent, args, &[s(1)], |e, c, a, _, b| {
            client2::select_next(e, c, a, &mut b[0])
        }),
        0x021C_F2F0 => run(env, lent, args, &[s(1)], |e, c, _, _, b| {
            client2::select_end(e, c, &mut b[0])
        }),
        0x021C_F360 => run(env, lent, args, &[], |_, c, _, _, _| {
            client2::clear_1a0(c);
            0
        }),
        0x021C_F37C => run(env, lent, args, &[], |e, c, _, v, _| {
            client2::set_1a0_slot(e, c, v[1], v[2]);
            0
        }),
        0x021C_F3B8 => run(env, lent, args, &[], |e, c, _, v, _| {
            client2::clear_1a0_slot(e, c, v[1]);
            0
        }),
        0x021C_F3F8 => run(env, lent, args, &[], |e, c, _, _, _| {
            client2::clear_1a0_all(e, c);
            0
        }),
        0x021C_F41C => run(env, lent, args, &[], |_, c, _, _, _| {
            client2::clear_130(c);
            0
        }),
        0x021C_F430 => run(env, lent, args, &[], |_, c, _, v, _| {
            client2::set_130(c, v[1], v[2]);
            0
        }),
        0x021C_F43C => run(env, lent, args, &[], |_, c, _, v, _| {
            client2::get_130(c, v[1]) as u32
        }),
        0x021C_F448 => run(env, lent, args, &[], |_, c, _, _, _| client2::sum_130(c)),
        0x021C_F524 => run(env, lent, args, &[], |e, c, _, v, _| {
            client2::struggle_if_stuck(e, c, v[1], v[2])
        }),
        // ---- client3 ----
        0x021C_F5B0 => run(env, lent, args, &[], |e, c, _, v, _| {
            client3::move_blocked(e, c, v[1], v[2], v[3])
        }),
        0x021C_F89C => run(env, lent, args, &[], |e, c, _, v, _| {
            client3::can_use(e, c, v[1])
        }),
        0x021D_0B10 => run(env, lent, args, &[], |e, c, _, v, _| {
            client3::setup_party_block(e, c, v[1], v[2], v[3], v[4]);
            0
        }),
        0x021D_0770 => run(env, lent, args, &[s(1)], |e, c, a, _, b| {
            client3::ai_step(e, c, a, &mut b[0])
        }),
        0x021D_0CC4 => run(env, lent, args, &[s(1)], |e, c, a, _, b| {
            client3::yes_no_step(e, c, a, &mut b[0])
        }),
        // ---- client4 / client5 command handlers: (c, &state, args) ----
        0x021D_1A74 | 0x021D_1A60 => return play_commands(env, lent, addr, args),
        _ => return client45(env, lent, addr, args),
    })
}

/// The command player 021d1a74 / 021d1a60 (request 8): `(c, &state)`; the command queue
/// (`c + 0x134` points at it) and the adapter receive memory are separate objects.
fn play_commands(
    env: &mut Env,
    lent: &mut Lent<'_>,
    addr: u32,
    args: &mut [Arg<'_>],
) -> Option<u32> {
    let a = client_addr(env, lent, args);
    Some(run(env, lent, args, &[(1, 4)], |e, c, _, _, b| {
        let q = u32::from_le_bytes(c[ofs::QUEUE as usize..][..4].try_into().unwrap());
        let adapter = u32::from_le_bytes(c[ofs::ADAPTER as usize..][..4].try_into().unwrap());
        let qbase = e.objs.borrow().base_of(q).expect("command queue object");
        let dbase = e.objs.borrow().base_of(adapter).expect("adapter object");
        let dlen = e.objs.borrow().blocks[&dbase].len;
        let data = e.objs.borrow_mut().blocks[&dbase].data.clone();
        let _ = (a, dlen);
        let mut queue = e.objs.borrow_mut().take(qbase);
        let r = if addr == 0x021D_1A60 {
            client4::play_commands_mode2(e, c, &mut b[0], &mut queue, qbase, &data, dbase)
        } else {
            client4::play_commands(e, c, &mut b[0], &mut queue, qbase, &data, dbase)
        };
        e.objs.borrow_mut().put(qbase, queue);
        r
    }))
}

fn client45(env: &mut Env, lent: &mut Lent<'_>, addr: u32, args: &mut [Arg<'_>]) -> Option<u32> {
    Some(match addr {
        0x021D_36CC => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_damage(e, c, &b[0])
        }),
        0x021D_36EC => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_heal(e, c, &b[0])
        }),
        0x021D_370C => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_spend_pp(e, c, &b[0])
        }),
        0x021D_3734 => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_spend_orig_pp(e, c, &b[0])
        }),
        0x021D_375C => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_mark_used(e, c, &b[0])
        }),
        0x021D_377C => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_faint(e, c, &b[0])
        }),
        0x021D_3794 => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_restore_pp(e, c, &b[0])
        }),
        0x021D_37B8 => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_restore_orig_pp(e, c, &b[0])
        }),
        0x021D_37DC => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_raise_stage(e, c, &b[0])
        }),
        0x021D_37FC => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_lower_stage(e, c, &b[0])
        }),
        0x021D_381C => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_set_stages(e, c, &b[0])
        }),
        0x021D_3894 => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_clear_negative_stages(e, c, &b[0])
        }),
        0x021D_38AC => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_reset_stages(e, c, &b[0])
        }),
        0x021D_38C4 => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_add_crit_boost(e, c, &b[0])
        }),
        0x021D_38E0 => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_set_sick(e, c, &b[0])
        }),
        0x021D_3900 => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_cure_major_status(e, c, &b[0])
        }),
        0x021D_3918 => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_cure_sick(e, c, &b[0])
        }),
        0x021D_3934 => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_switch_in(e, c, &b[0])
        }),
        0x021D_3988 => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_set_type_pair(e, c, &b[0])
        }),
        0x021D_39A8 => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_tick_sicks(e, c, &b[0])
        }),
        0x021D_3A20 => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_set_u14(e, c, &b[0])
        }),
        0x021D_3A40 => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_record_move(e, c, &b[0])
        }),
        0x021D_3A7C => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_set_flag_b(e, c, &b[0])
        }),
        0x021D_3A98 => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_clear_flag_b(e, c, &b[0])
        }),
        0x021D_3AB4 => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_set_flag_a(e, c, &b[0])
        }),
        0x021D_3AD0 => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_clear_flag_a(e, c, &b[0])
        }),
        0x021D_3AEC => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_set_ability(e, c, &b[0])
        }),
        0x021D_3B08 => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_set_item(e, c, &b[0])
        }),
        0x021D_3B28 => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_move_replace(e, c, &b[0])
        }),
        0x021D_3B5C => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_reset_on_battle_end(e, c, &b[0])
        }),
        0x021D_3B74 => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_view_side_ef9f4(e, c, &b[0])
        }),
        0x021D_3B88 => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_view_side_efad8(e, c, &b[0])
        }),
        0x021D_3B9C => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_view_side_efb34(e, c, &b[0])
        }),
        0x021D_3BB0 => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_view_side_efa94(e, c, &b[0])
        }),
        0x021D_3BC0 => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_set_u8_157(e, c, &b[0])
        }),
        0x021D_3BE0 => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_copy_passable(e, c, &b[0])
        }),
        0x021D_3C0C => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_make_substitute(e, c, &b[0])
        }),
        0x021D_3C2C => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_clear_substitute(e, c, &b[0])
        }),
        0x021D_3C70 => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_party_9d10(e, c, &b[0])
        }),
        0x021D_3C90 => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_6738(e, c, &b[0])
        }),
        0x021D_3CA8 => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_cure_linked(e, c, &b[0])
        }),
        0x021D_3CC8 => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::relay_mon_block(e, c, &b[0])
        }),
        0x021D_3CFC => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_turn_start(e, c, &b[0])
        }),
        0x021D_3D2C => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_set_stat(e, c, &b[0])
        }),
        0x021D_3D50 => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_mon_set_weight(e, c, &b[0])
        }),
        0x021D_2FD8 => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client5::cmd_exp_gain(e, c, &mut s[0], &a[0])
        }),
        0x021D_3138 => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client5::cmd_gauge_message(e, c, &mut s[0], &a[0])
        }),
        0x021D_31A0 => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client5::cmd_party_slot(e, c, &mut s[0], &a[0])
        }),
        0x021D_324C => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client5::cmd_ability_change(e, c, &mut s[0], &a[0])
        }),
        0x021D_344C => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client5::cmd_illusion_break(e, c, &mut s[0], &a[0])
        }),
        0x021D_34E8 => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client5::cmd_position_anim(e, c, &mut s[0], &a[0])
        }),
        0x021D_3568 => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client5::cmd_pair_anim(e, c, &mut s[0], &a[0])
        }),
        0x021D_35C8 => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client5::cmd_form_change(e, c, &mut s[0], &a[0])
        }),
        0x021D_3630 => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client5::cmd_ability_window_open(e, c, &mut s[0], &a[0])
        }),
        0x021D_3688 => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client5::cmd_ability_window_close(e, c, &mut s[0], &a[0])
        }),
        0x021D_1BD0 => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client4::cmd_hp_message(e, c, &mut s[0], &a[0])
        }),
        0x021D_1CAC => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client4::cmd_hp_gauge(e, c, &mut s[0], &a[0])
        }),
        0x021D_1CF4 => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client4::cmd_send_out(e, c, &mut s[0], &a[0])
        }),
        0x021D_1E6C => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client4::cmd_message(e, c, &mut s[0], &a[0])
        }),
        0x021D_1EB4 => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client4::cmd_message_se(e, c, &mut s[0], &a[0])
        }),
        0x021D_1F10 => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client4::cmd_message_alt(e, c, &mut s[0], &a[0])
        }),
        0x021D_1F58 => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client4::cmd_message_alt_se(e, c, &mut s[0], &a[0])
        }),
        0x021D_1FB4 => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client4::cmd_message_6f4(e, c, &mut s[0], &a[0])
        }),
        0x021D_2030 => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client4::cmd_move_anim(e, c, &mut s[0], &a[0])
        }),
        0x021D_20BC => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client4::cmd_status_anim(e, c, &mut s[0], &a[0])
        }),
        0x021D_210C => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client4::cmd_effect_anim(e, c, &mut s[0], &a[0])
        }),
        0x021D_21D4 => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client4::cmd_slot_anim_message(e, c, &mut s[0], &a[0])
        }),
        0x021D_2234 => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client4::cmd_fx_81d0(e, c, &mut s[0], &a[0])
        }),
        0x021D_2278 => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client4::cmd_hit_anim(e, c, &mut s[0], &a[0])
        }),
        0x021D_2320 => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client4::cmd_faint_anim(e, c, &mut s[0], &a[0])
        }),
        0x021D_2374 => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client4::cmd_slot_anim_438(e, c, &mut s[0], &a[0])
        }),
        0x021D_23B8 => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client4::cmd_position_anim_7c8(e, c, &mut s[0], &a[0])
        }),
        0x021D_2424 => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client4::cmd_position_anim_7d8(e, c, &mut s[0], &a[0])
        }),
        0x021D_2490 => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client4::cmd_field_effect(e, c, &mut s[0], &a[0])
        }),
        0x021D_2510 => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client4::cmd_field_end(e, c, &mut s[0], &a[0])
        }),
        0x021D_2580 => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client4::cmd_slot_anim_41(e, c, &mut s[0], &a[0])
        }),
        0x021D_25C4 => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client4::cmd_slot_anim_838(e, c, &mut s[0], &a[0])
        }),
        0x021D_2628 => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client4::cmd_slot_anim_43(e, c, &mut s[0], &a[0])
        }),
        0x021D_266C => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client4::cmd_swap_slots(e, c, &mut s[0], &a[0])
        }),
        0x021D_2700 => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client4::cmd_rotate(e, c, &mut s[0], &a[0])
        }),
        0x021D_2814 => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client4::cmd_fx_7e94(e, c, &mut s[0], &a[0])
        }),
        0x021D_284C => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client4::cmd_fx_7ea4(e, c, &mut s[0], &a[0])
        }),
        0x021D_28AC => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client4::cmd_transform(e, c, &mut s[0], &a[0])
        }),
        0x021D_2930 => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let (s, a) = b.split_at_mut(1);
            client4::cmd_effectiveness_anim(e, c, &mut s[0], &a[0])
        }),
        // command 0x33: pops its list from the command queue (`c + 0x134` points into it)
        0x021D_215C => run(env, lent, args, &[(1, 4), (2, 0x40)], |e, c, _, _, b| {
            let q = rd32(c, ofs::QUEUE as usize);
            let qbase = e.objs.borrow().base_of(q).expect("command queue object");
            let mut queue = e.objs.borrow_mut().take(qbase);
            let (s, a) = b.split_at_mut(1);
            let r = client4::cmd_targets_anim(e, c, &mut s[0], &a[0], &mut queue, qbase);
            e.objs.borrow_mut().put(qbase, queue);
            r
        }),
        0x021D_127C => d3(env, lent, args, client4::req_battle_result_bgm_mode2),
        0x021D_1290 => d3(env, lent, args, client4::req_battle_result_bgm),
        0x021D_13FC => d3(env, lent, args, client4::req_battle_end_mode2),
        0x021D_1410 => d3(env, lent, args, client4::req_battle_end),
        0x021D_1708 => d3(env, lent, args, client4::req_trainer_message_mode2),
        0x021D_171C => d3(env, lent, args, client4::req_trainer_message),
        0x021D_3D9C => run(env, lent, args, &[], |e, c, _, _, _| {
            client5::load_trainer_items(e, c);
            0
        }),
        0x021D_3D70 => run(env, lent, args, &[], |_, c, _, _, _| {
            client5::client_id(c) as u32
        }),
        0x021D_3D7C => run(env, lent, args, &[], |_, c, _, _, _| client5::party(c)),
        0x021D_3D84 => run(env, lent, args, &[], |e, c, _, _, _| {
            client5::view_side_ef9a8(e, c)
        }),
        0x021D_3D90 => run(env, lent, args, &[], |_, c, _, _, _| {
            client5::counter(c) as u32
        }),
        0x021D_3DC4 => run(env, lent, args, &[], |e, c, _, v, _| {
            client5::pick_trainer_item(e, c, v[1], v[2]) as u32
        }),
        0x021D_3E70 => client5::item_boosts_stat(env, vals(args)[0], vals(args)[1]),
        0x021D_3EC8 => client5::item_cures_status(env, vals(args)[0], vals(args)[1]),
        0x021D_3C44 => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client5::cmd_counter_add(e, c, &b[0])
        }),
        0x021D_3D1C => run(env, lent, args, &[], |e, c, _, _, _| {
            client5::cmd_view_side_efc78(e, c)
        }),
        0x021D_34B0 => {
            let mut b = Bufs::new(args);
            let (mut st, a) = (b.get(env, lent, 1, 4), b.get(env, lent, 2, 0x40));
            let r = client5::cmd_play_se(env, vals(args)[0], &mut st, &a);
            Bufs::new(args).put(env, lent, 1, &st);
            r
        }
        0x021D_2990 => {
            let a = Bufs::new(args).get(env, lent, 2, 0x40);
            client4::play_sound(env, &a)
        }
        0x021D_299C => run(env, lent, args, &[(1, 4)], |e, c, _, _, b| {
            client4::view_effect_step(e, c, &mut b[0])
        }),
        0x021D_29CC => run(env, lent, args, &[(2, 0x40)], |e, c, _, _, b| {
            client4::cmd_add_evs(e, c, &b[0])
        }),
        0x021D_2C7C => client4::window_sound_event(env, vals(args)[0]),
        0x021D_1DF4 => run(env, lent, args, &[], |e, c, _, _, _| {
            client4::own_send_out_message(e, c)
        }),
        0x021D_2000 => run(env, lent, args, &[], |e, c, _, v, _| {
            client4::move_anim_param(e, c, v[1], v[2])
        }),
        0x021D_185C => run(env, lent, args, &[], |e, c, _, v, _| {
            client4::trainer_speech(e, c, v[1], v[2]);
            0
        }),
        0x021D_18EC => run(env, lent, args, &[(1, 4)], |e, c, _, _, b| {
            client4::req_money_message(e, c, &mut b[0])
        }),
        0x021D_1960 => run(env, lent, args, &[(1, 4)], |e, c, _, _, b| {
            client4::req_lost_message(e, c, &mut b[0])
        }),
        0x021D_1A18 => run(env, lent, args, &[(1, 4)], |e, c, _, _, b| {
            client4::req_message_b7(e, c, &mut b[0])
        }),
        0x021D_123C => run(env, lent, args, &[], |e, c, _, _, _| {
            client4::req_record_data(e, c)
        }),
        _ => return None,
    })
}

/// `fn(env, c, state, data, data_base)`: a request step reading the adapter's message.
type ReqStep = fn(&mut Env, &[u8], &mut [u8], &[u8], u32) -> u32;

/// A request step `(c, &state)` that reads the adapter's received message: `data` = the adapter
/// channel object (`c + 0x50`), `data_base` its address.
fn d3(env: &mut Env, lent: &mut Lent<'_>, args: &mut [Arg<'_>], f: ReqStep) -> u32 {
    run(env, lent, args, &[(1, 4)], |e, c, _, _, b| {
        let ad = super::client::rd32(c, ofs::ADAPTER as usize);
        let data = e.obj_copy(ad);
        f(e, c, &mut b[0], &data, ad)
    })
}

pub fn rd32(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}
