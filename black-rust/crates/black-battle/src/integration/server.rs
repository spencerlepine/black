//! `battle_server.s` (server1-3, 0x021B6100-0x021BAA94): adapters to the ports, transcribed from
//! the areas' differential-test `call` tables. The ports work on a [`crate::server::Mem`] over the
//! object table. A pass-through (`P`) argument of these functions is the main module M unless an
//! adapter says otherwise.

use super::{Bufs, Env};
use crate::server::{server1, server2, server3};
use crate::server_flow::env::{Arg, Lent};

/// Argument values with `P` = M.
fn vals(env: &Env, args: &[Arg<'_>]) -> Vec<u32> {
    args.iter()
        .map(|a| match a {
            Arg::V(v) | Arg::PAt(v) => *v,
            Arg::P => env.main_module,
            _ => 0,
        })
        .chain(std::iter::repeat(0))
        .take(8)
        .collect()
}

pub fn call(env: &mut Env, lent: &mut Lent<'_>, addr: u32, args: &mut [Arg<'_>]) -> Option<u32> {
    let v = vals(env, args);
    let m = v[0];
    let e = env;
    if let Some(r) = server1_call(e, addr, &v) {
        return Some(r);
    }
    if let Some(r) = server2_call(e, lent, addr, &v, args) {
        return Some(r);
    }
    server3_call(e, addr, &v, m)
}

fn server1_call(e: &mut Env, addr: u32, v: &[u32]) -> Option<u32> {
    let mut mem = e.mem();
    let mem = &mut mem;
    // sub-procs run as fn(M + 0x45C, M[0x458]): v[1] = M; their entry r3 is M in the game (B11c)
    Some(match addr {
        0x021B_6100 => server1::proc_init(e, mem, v[0], v[1], v[2], v[3]),
        0x021B_6434 => server1::proc_main(e, mem, v[3]),
        0x021B_64CC => server1::readers_at_end(e),
        0x021B_6500 => server1::proc_exit(e, mem, v[0], v[1], v[3]),
        0x021B_66D4 => {
            server1::select_subproc(mem, v[0], v[1], v[2]);
            0
        }
        0x021B_6804 => {
            server1::set_exit_subproc(mem, v[0], v[1]);
            0
        }
        0x021B_6818 => server1::setup_type_is_3(mem, v[0]),
        0x021B_6848 => {
            let r3 = e.main_module;
            server1::own_client_init(e, mem, v[0], v[1], r3);
            0
        }
        0x021B_68D8 => {
            server1::register_parties(e, mem, v[0], v[1]);
            0
        }
        0x021B_6954 => {
            server1::own_party(e, mem, v[0], v[1]);
            0
        }
        0x021B_6994 | 0x021B_99B8 => server1::party_mode(e, mem, v[0], v[1]),
        0x021B_69FC => {
            server1::own_parties(e, mem, v[0], v[1]);
            0
        }
        0x021B_6A3C => {
            server1::record_reader_init(e, mem, v[0], v[1]);
            0
        }
        0x021B_6A70 => {
            server1::view_create(e, mem, v[0], v[1], v[2]);
            0
        }
        0x021B_6AAC => server1::subproc_local_single(e, mem, v[1], v[1]),
        0x021B_6C08 => server1::subproc_exit(e, mem, v[1]),
        0x021B_6C5C => server1::subproc_local_double(e, mem, v[1], v[1]),
        0x021B_6DD0 => server1::subproc_local_multi(e, mem, v[1]),
        0x021B_7050 => server1::subproc_local_triple(e, mem, v[1], v[1]),
        0x021B_71D4 => server1::subproc_local_rotation(e, mem, v[1], v[1]),
        0x021B_734C => server1::subproc_comm_single(e, mem, v[0], v[1]),
        0x021B_73C4 => server1::subproc_comm_double(e, mem, v[0], v[1]),
        0x021B_7488 => server1::subproc_comm_rule3(e, mem, v[0], v[1]),
        0x021B_7518 => server1::subproc_comm_rule2(e, mem, v[0], v[1]),
        _ => return None,
    })
}

fn server2_call(
    e: &mut Env,
    lent: &mut Lent<'_>,
    addr: u32,
    v: &[u32],
    args: &mut [Arg<'_>],
) -> Option<u32> {
    let mut mem = e.mem();
    let m = v[0];
    let v = |i: usize| v[i];
    Some(match addr {
        0x021B_75A8 => server2::step_handshake(e, &mut mem, m, v(1)),
        0x021B_7758 => server2::step_party_sync(e, &mut mem, m, v(1)),
        0x021B_7968 => server2::step_record_exchange(e, &mut mem, m, v(1)),
        0x021B_7BB0 => server2::step_data_exchange(e, &mut mem, m, v(1)),
        0x021B_835C => server2::step_view(e, &mut mem, m, v(1)),
        0x021B_7CB4 => server2::step_create_pair(e, &mut mem, m),
        0x021B_7E18 => server2::step_create_multi(e, &mut mem, m),
        0x021B_8070 => server2::step_create_mode3(e, &mut mem, m),
        0x021B_81DC => server2::step_create_many(e, &mut mem, m),
        0x021B_83F4 => server2::frame_no_server(e, &mut mem, m),
        0x021B_8434 => server2::main_loop_server(e, &mut mem, m),
        0x021B_84D8 => server2::main_loop_client(e, &mut mem, m),
        0x021B_8548 => {
            server2::quit_clients(e, &mem, m);
            0
        }
        0x021B_8564 => server2::all_clients_done(&mem, m),
        0x021B_8588 => server2::rule(&mem, m),
        0x021B_8590 => server2::flag_bit2(&mem, m),
        0x021B_85A0 => server2::setup_byte_80(&mem, m),
        0x021B_85A8 => server2::is_plain_single(e, &mem, m),
        0x021B_85DC => server2::call_02013080(e, &mem, m),
        0x021B_85EC => server2::check_overlay_231(e, &mem, m),
        0x021B_8630 => server2::setup_word_70(&mem, m),
        0x021B_8640 => server2::rule_count_8640(&mem, m),
        0x021B_8670 => server2::rule_positions_per_side(&mem, m),
        0x021B_86A0 => server2::pos_in_range(&mem, m, v(1)),
        0x021B_86B8 => server2::battle_type(&mem, m),
        0x021B_86C0 => server2::setup_byte_20(&mem, m),
        0x021B_86C8 => server2::setup_byte_22_set(&mem, m),
        0x021B_86DC => server2::setup_u16_1a(&mem, m),
        0x021B_86E4 => server2::type_class(&mem, m),
        0x021B_8714 => server2::setup_addr_8(&mem, m),
        0x021B_871C => server2::word_c0(&mem, m),
        0x021B_8724 => server2::call_0200ced4(e, &mem, m, v(1)),
        0x021B_873C => server2::call_0200cf28(e, &mem, m),
        0x021B_8748 => server2::setup_word_58(&mem, m),
        0x021B_8750 => {
            server2::call_0200cfa0(e, &mem, m, v(1), v(2));
            0
        }
        0x021B_8778 => {
            let mut b = Bufs::new(args);

            b.with(e, lent, 2, 6, |e, out| {
                server2::expand_target(e, &mem, m, v(1), out)
            })
        }
        0x021B_87E8 => Bufs::new(args).with(e, lent, 3, 6, |_, out| {
            server2::expand_single(&mem, m, v(1), v(2), out)
        }),
        0x021B_8834 => Bufs::new(args).with(e, lent, 3, 6, |_, out| {
            server2::expand_double(&mem, m, v(1), v(2), out)
        }),
        0x021B_88EC => Bufs::new(args).with(e, lent, 3, 6, |e, out| {
            server2::expand_triple(e, &mem, m, v(1), v(2), out)
        }),
        0x021B_8B78 => server2::client_pos_count(&mem, m, v(1)),
        0x021B_8B9C => server2::client_present(&mem, m, v(1)),
        0x021B_8BC4 => server2::client_side(v(1)),
        0x021B_8BD0 => server2::is_own_side(&mem, m, v(1)),
        0x021B_8BEC => server2::client_nth_pos(&mem, m, v(1), v(2)),
        0x021B_8C20 => server2::pos_of(v(0), v(1), v(2)),
        0x021B_8C38 => server2::side_slot(v(0), v(1)),
        0x021B_8C50 => server2::opponent_slot(&mem, m, v(1), v(2)),
        0x021B_8C5C => server2::facing_pos(v(0), v(1)),
        0x021B_8CB0 => server2::partner_pos(&mem, m, v(1)),
        0x021B_8CE8 => server2::opposing(v(1), v(2)),
        0x021B_8CFC | 0x021B_8DE4 => server2::pos_client(&mem, m, v(1)),
        0x021B_8D08 | 0x021B_8DDC => server2::mon_id_client(v(0)),
        0x021B_8D38 => Bufs::new(args).with(e, lent, 3, 6, |e, out| {
            server2::target_mon_ids(e, &mem, m, v(1), v(2), out)
        }),
        0x021B_8D8C => server2::mon_pos(e, &mem, m, v(1), v(2)),
        0x021B_8DC0 => server2::mon_pos_class(e, &mem, m, v(1), v(2)),
        0x021B_8DEC => server2::pos_client_slot(&mem, m, v(1)),
        0x021B_8E28 => {
            let mut b = Bufs::new(args);
            let (mut o1, mut o2) = (b.get(e, lent, 2, 1), b.get(e, lent, 3, 1));
            server2::pos_client_and_slot(&mem, m, v(1), &mut o1, &mut o2);
            b.put(e, lent, 2, &o1);
            b.put(e, lent, 3, &o2);
            0
        }
        0x021B_8E70 => server2::pos_class(&mem, m, v(1)),
        0x021B_8ED8 => server2::class_pos(&mem, m, v(1)),
        0x021B_8F38 => server2::multi_not_type01(&mem, m),
        0x021B_8F5C => server2::pokecon_refresh(e, m, v(1)),
        0x021B_8F7C => server2::partner_pokecon_refresh(e, &mem, m, v(1)),
        0x021B_8FC0 => server2::is_multi_second_pair(&mem, m, v(1)),
        0x021B_8FE4 => server2::own_is_multi_second_pair(&mem, m),
        0x021B_8FF4 => server2::own_client_id(&mem, m),
        0x021B_9000 => server2::own_partner_id(&mem, m),
        0x021B_9010 => server2::partner_id(&mem, m, v(1)),
        _ => return None,
    })
}

fn server3_call(e: &mut Env, addr: u32, v: &[u32], m0: u32) -> Option<u32> {
    use server3::*;
    let mut mem = e.mem3();
    let m = &mut mem;
    let _ = m0;
    Some(match addr {
        0x021B_902C => same_or_partner(v[0], v[1]),
        0x021B_904C => pick_own_side_client(e, m, v[0], v[1]),
        0x021B_9064 => pick_client(e, v[0], v[1], v[2]),
        0x021B_9090 => {
            own_view_call_7f1c(e, m, v[0], v[1], v[2]);
            0
        }
        0x021B_90BC => {
            own_view_call_7e50(e, m, v[0], v[1], v[2]);
            0
        }
        0x021B_90E8 => setup_60(m, v[0]),
        0x021B_90F0 => setup_64(m, v[0]),
        0x021B_90F8 => records_locally(m, v[0]),
        0x021B_9114 => is_comm(m, v[0]),
        0x021B_911C => zero(),
        0x021B_9120 => {
            count_positions_of(e, m, v[0], v[1]);
            0
        }
        0x021B_916C => {
            set_result_once(m, v[0], v[1]);
            0
        }
        0x021B_9184 => {
            set_setup_c1_bit0(m, v[0]);
            0
        }
        0x021B_919C => finalize_430(e, m, v[0]),
        0x021B_91E4 => {
            add_money(m, v[0], v[1]);
            0
        }
        0x021B_9210 => money(m, v[0]),
        0x021B_9234 => {
            set_doubling(m, v[0]);
            0
        }
        0x021B_9244 => compute_438(e, m, v[0]),
        0x021B_92A0 => {
            set_flag5(m, v[0]);
            0
        }
        0x021B_92B0 => {
            mon_pokemon_notify_local(e, m, v[0], v[1], v[2]);
            0
        }
        0x021B_92D4 => {
            mon_pokemon_notify(e, m, v[0], v[1], v[2]);
            0
        }
        0x021B_9358 => compare_pc_mons(e, m, v[0], v[1]),
        0x021B_937C => illusion_source(e, m, v[0], v[1], v[2]),
        0x021B_93E0 => {
            pokecon_init(m, v[0], v[1], v[2]);
            0
        }
        0x021B_9424 => {
            pokecon_free(e, m, v[0]);
            0
        }
        0x021B_9460 => {
            pokecon_setup_client(e, m, v[0], v[1], v[2]);
            0
        }
        0x021B_9634 => party_last_eligible(e, v[0]),
        0x021B_9674 => {
            pokecon_free_mons(e, m, v[0]);
            0
        }
        0x021B_969C => {
            pokecon_sync_back(e, m, v[0], v[1], v[2], v[3]);
            0
        }
        0x021B_97A0 => {
            pokecon_sync_ordered(e, m, v[0], v[1], v[2]);
            0
        }
        0x021B_985C => pc_source(m, v[0], v[1]),
        0x021B_9864 => party_index_of_id(e, m, v[0], v[1], v[2]),
        0x021B_98AC | 0x021B_98A4 => mon_at_position(e, m, v[0], v[1]),
        0x021B_990C => pc_party_get(m, v[0], v[1], v[2]),
        0x021B_9920 => pc_party_get2(m, v[0], v[1], v[2]),
        0x021B_9934 => pc_mon(m, v[0], v[1]),
        0x021B_9940 => pc_mon2(m, v[0], v[1]),
        0x021B_994C => view_obj_of(e, m, v[0], v[1], v[2]),
        0x021B_997C => view_value(e, m, v[0], v[1]),
        0x021B_99C0 => adjusted_6994(e, v[0], v[1]),
        0x021B_99E0 => pos_list_record(v[0]),
        0x021B_99F0 => is_pos_2_or_3(v[0]),
        0x021B_9A00 => pos_8c20(e, m, v[0], v[1]),
        0x021B_9A24 => same_side(v[0], v[1]),
        0x021B_9A40 => side_of_id(v[0]),
        0x021B_9A50 => other_side_of_id(v[0]),
        0x021B_9A5C => is_zero(v[0]),
        0x021B_9A6C => pos_list_contains(m, v[0], v[1]),
        0x021B_9AB0 => swap_slot_a(v[0]),
        0x021B_9AC8 => swap_slot_b(v[0]),
        0x021B_9AE0 => side_of_client(v[0]),
        0x021B_9AEC => party_addr(v[0], v[1]),
        0x021B_9AF8 => party_addr2(v[0], v[1]),
        0x021B_9B04 => pc_party_alive(e, m, v[0], v[1]),
        0x021B_9B10 => {
            client_call_d9f4(e, m, v[0], v[1], v[2]);
            0
        }
        0x021B_9B40 => {
            party_clear(m, v[0]);
            0
        }
        0x021B_9B54 => {
            party_append(m, v[0], v[1]);
            0
        }
        0x021B_9B60 => {
            party_dead_to_end(e, m, v[0]);
            0
        }
        0x021B_9B94 => party_count(m, v[0]),
        0x021B_9B98 => party_alive(e, m, v[0]),
        0x021B_9BC4 => party_alive_from(e, m, v[0], v[1]),
        0x021B_9BF0 => party_get(m, v[0], v[1]),
        0x021B_9C00 => party_get2(m, v[0], v[1]),
        0x021B_9C10 => {
            party_swap(m, v[0], v[1], v[2]);
            0
        }
        0x021B_9C24 => {
            party_move_to_end(m, v[0], v[1]);
            0
        }
        0x021B_9C50 => {
            party_rotate_for_pos(m, v[0], v[1], v[2], v[3]);
            0
        }
        0x021B_9C98 => party_index_of(m, v[0], v[1]),
        0x021B_9CBC => party_index_of_mon_id(e, m, v[0], v[1]),
        0x021B_9CE8 => party_first_alive(e, m, v[0]),
        0x021B_9D10 => {
            party_refresh_illusion(e, m, v[0], v[1], v[2]);
            0
        }
        0x021B_9DB8 => {
            records_clear(m, v[0]);
            0
        }
        0x021B_9DE8 => {
            records_free(e, m, v[0]);
            0
        }
        0x021B_9E38 => {
            record_fill_from(e, m, v[0], v[1], v[2]);
            0
        }
        0x021B_9E7C => {
            record_fill_from_block(e, m, v[0], v[1]);
            0
        }
        0x021B_9EF0 => {
            record_fill_from_block_name(e, m, v[0], v[1]);
            0
        }
        0x021B_9F70 => record_empty_at(m, v[0], v[1]),
        0x021B_9F84 => record_u16x4(m, v[0], v[1], v[2]),
        0x021B_9FAC => record_ptr1(m, v[0], v[1], v[2]),
        // the AI mask of a client (`021b9fbc(M, id)`, read only for AI clients); the B17 test
        // forces it to run the trainer-AI scripts in this wild battle
        0x021B_9FBC => match e.ai_mask {
            Some(x) => x,
            None => record_word(e, m, v[0], v[1]),
        },
        0x021B_A004 => record_u16b(m, v[0], v[1]),
        0x021B_A020 => record_u16a(m, v[0], v[1]),
        0x021B_A02C => record_sub(e, m, v[0], v[1], v[2]),
        0x021B_A064 => record_ptr0(m, v[0], v[1]),
        0x021B_A080 => setup_call_e7dc(e, m, v[0]),
        0x021B_A08C => m_2bc(m, v[0]),
        0x021B_A094 => {
            parties_alloc(e, m, v[0]);
            0
        }
        0x021B_A0CC => {
            parties_free(e, m, v[0]);
            0
        }
        0x021B_A128 => {
            parties_copy_in(e, m, v[0], v[1], v[2]);
            0
        }
        0x021B_A178 => party_source(e, m, v[0], v[1], v[2]),
        0x021B_A1A8 => setup_call_3158(e, m, v[0], v[1]),
        0x021B_A1B4 => {
            setup_call_8df0(e, m, v[0], v[1]);
            0
        }
        0x021B_A1CC => {
            setup_call_8dac(e, m, v[0], v[1], v[2]);
            0
        }
        0x021B_A1E4 => setup_78(m, v[0]),
        0x021B_A1EC => setup_7f_flag(e, m, v[0]),
        0x021B_A20C => setup_mode_flag(e, m, v[0]),
        0x021B_A238 => {
            mon_notify_server(e, m, v[0], v[1]);
            0
        }
        0x021B_A250 => {
            mon_pokemon_notify_b7b4(e, m, v[0], v[1], v[2]);
            0
        }
        0x021B_A2D4 => {
            server_call_b8dc(e, m, v[0], v[1]);
            0
        }
        0x021B_A2E4 => {
            write_back_results(e, m, v[0]);
            0
        }
        0x021B_A534 => is_type_2_or_3(e, v[0]),
        0x021B_A54C | 0x021B_917C => battle_result(e, m, v[0]),
        0x021B_A5C4 => m_442(m, v[0]),
        0x021B_A5D0 => m_440(m, v[0]),
        0x021B_A5D8 => own_client_dda0(e, m, v[0]),
        0x021B_A618 => own_client_ddcc(e, m, v[0]),
        0x021B_A648 => {
            copy_to_setup_party(e, m, v[0], v[1], v[2]);
            0
        }
        0x021B_A670 => {
            copy_to_setup_34(e, m, v[0], v[1], v[2]);
            0
        }
        0x021B_A698 => {
            store_view_value(e, m, v[0], v[1]);
            0
        }
        0x021B_A6C0 => {
            save_record(e, m, v[0]);
            0
        }
        0x021B_A710 => client_relation(v[0], v[1]),
        0x021B_A73C => {
            rebuild_battle(e, m, v[0], v[1]);
            0
        }
        0x021B_A7F8 => {
            clients_call_d9b0(e, m, v[0]);
            0
        }
        0x021B_A820 => server_flow_work(e, m, v[0]),
        0x021B_A82C => {
            clear_stats(m, v[0]);
            0
        }
        0x021B_A878 => {
            collect_stats(e, m, v[0]);
            0
        }
        0x021B_A9C8 => server_create(e, m, v[0], v[1], v[2], v[3], v[4]),
        _ => return None,
    })
}
