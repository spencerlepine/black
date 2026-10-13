use super::Mem;
use crate::server_flow::testkit::{run_area_suite, Case};

// Written by tools/check-svflow.sh only, after the Rust hashes matched the ROM run.
// svflow-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021B902C, "4d46ae5ca8991551"),
    (0x021B904C, "f2ac987d4b296d83"),
    (0x021B9064, "9f32c5a805fd1aad"),
    (0x021B9090, "1df540a3b119c2f8"),
    (0x021B90BC, "10ca94d069b90927"),
    (0x021B90E8, "25285d7cd377108c"),
    (0x021B90F0, "ef1a40479a932e8a"),
    (0x021B90F8, "bc81c962485c7688"),
    (0x021B9114, "25935dad491ad7e5"),
    (0x021B911C, "c86bdd2468b601a5"),
    (0x021B9120, "c610c4f26f16a967"),
    (0x021B916C, "12d55c97497557e5"),
    (0x021B9184, "660eb74c4b744655"),
    (0x021B919C, "1deea09993209437"),
    (0x021B91E4, "f9c8cac9c7885de6"),
    (0x021B9210, "2ad733ce1bdcac67"),
    (0x021B9234, "695c978b09f14ac2"),
    (0x021B9244, "77ac0f97223d25b1"),
    (0x021B92A0, "79b79fd0658940c8"),
    (0x021B92B0, "aa3143d8236d620a"),
    (0x021B92D4, "e012bd83b1d19593"),
    (0x021B9358, "8061719b167bf9a1"),
    (0x021B937C, "b280f90f9ae20307"),
    (0x021B93E0, "1f60da5e00c92f26"),
    (0x021B9424, "ce669a4cbb57ccf6"),
    (0x021B9460, "933a26e153e9979a"),
    (0x021B9634, "a889f916497ebce4"),
    (0x021B9674, "78bb790ac37b91c6"),
    (0x021B969C, "6296de45c83d53e3"),
    (0x021B97A0, "e5e1f33645d16fd3"),
    (0x021B985C, "214f6ed49e6d7c97"),
    (0x021B9864, "d472d2158df507c0"),
    (0x021B98AC, "51143c871e2d3bad"),
    (0x021B990C, "788203c08ab33119"),
    (0x021B9920, "47c8543add18050d"),
    (0x021B9934, "23ac5c944609d118"),
    (0x021B9940, "c5fda6f1bb234917"),
    (0x021B994C, "f80e4c3397d6ff62"),
    (0x021B997C, "665cb3b3deca447b"),
    (0x021B99C0, "637570ca029e3129"),
    (0x021B99E0, "33cb992187bebe2b"),
    (0x021B99F0, "07073d86dddb0166"),
    (0x021B9A00, "ce10f6570ea89b72"),
    (0x021B9A24, "236ee85fff54befd"),
    (0x021B9A40, "b25f62194f9b74a9"),
    (0x021B9A50, "2d85adbe9806f215"),
    (0x021B9A5C, "ec455bd98d35fcc5"),
    (0x021B9A6C, "9dfe9f55a4526a8b"),
    (0x021B9AB0, "6ca1da2f2b4be446"),
    (0x021B9AC8, "8a1bf44ab26831bd"),
    (0x021B9AE0, "1ba338bb4e5d13aa"),
    (0x021B9AEC, "46496fe4755669a6"),
    (0x021B9AF8, "2f01966523257f7e"),
    (0x021B9B04, "9036bfa773d0f91b"),
    (0x021B9B10, "99bd49dea807a248"),
    (0x021B9B40, "04472f02f378005a"),
    (0x021B9B54, "874b6ae249c23f92"),
    (0x021B9B60, "5194648dc995a00d"),
    (0x021B9B94, "898af7502990c210"),
    (0x021B9B98, "7aee09c60df3de94"),
    (0x021B9BC4, "e468cdd316da53b4"),
    (0x021B9BF0, "2151785ec9b37ecd"),
    (0x021B9C00, "20a32a5a16588255"),
    (0x021B9C10, "ca0b94bc3066e328"),
    (0x021B9C24, "a01f2ec8dc6186bd"),
    (0x021B9C50, "86714d52078fe548"),
    (0x021B9C98, "d9e050ddeca82ad0"),
    (0x021B9CBC, "3acb4170166d9952"),
    (0x021B9CE8, "2f910bdfefdaee0b"),
    (0x021B9D10, "d930c56ab5b7f0fc"),
    (0x021B9DB8, "403c4e2c6fca55d6"),
    (0x021B9DE8, "234bf8751d9bd3f8"),
    (0x021B9E38, "293bcaef124c7f8a"),
    (0x021B9E7C, "c5c72b0840bf2e38"),
    (0x021B9EF0, "b82bbc8296034343"),
    (0x021B9F70, "4732faa6fe95de42"),
    (0x021B9F84, "f9fc1bd9f52fa68c"),
    (0x021B9FAC, "46d3a10571159c9b"),
    (0x021B9FBC, "9c9d7db5bdca6068"),
    (0x021BA004, "c92c259ff2901168"),
    (0x021BA020, "e7f4dded8d76d317"),
    (0x021BA02C, "f764eee4ed5ad368"),
    (0x021BA064, "284714bc58a5d475"),
    (0x021BA080, "2321eb449e491181"),
    (0x021BA08C, "8715470dbd8aa2e0"),
    (0x021BA094, "4bd849415002dc7a"),
    (0x021BA0CC, "ed97837cfcfdc229"),
    (0x021BA128, "8849169eb4b7c33b"),
    (0x021BA178, "7eca59b8e545e56d"),
    (0x021BA1A8, "fe3dcb7e00fd126c"),
    (0x021BA1B4, "8edf956cd4ec8826"),
    (0x021BA1CC, "f6a217bafe5a2717"),
    (0x021BA1E4, "e8495faf7fcf5c57"),
    (0x021BA1EC, "4f39775b567108a9"),
    (0x021BA20C, "c103665e73582e98"),
    (0x021BA238, "409276ed14e0b438"),
    (0x021BA250, "16d9b16b8105e721"),
    (0x021BA2D4, "00570c90a943d751"),
    (0x021BA2E4, "e426c44234577ebc"),
    (0x021BA534, "2811a9ab0bfbf03d"),
    (0x021BA54C, "fdf3e0f91bf442e3"),
    (0x021BA5C4, "77a166d2f88d816b"),
    (0x021BA5D0, "a0ecd485aedcb48f"),
    (0x021BA5D8, "1acc4efc67cbfd43"),
    (0x021BA618, "83e75f3adcd152e5"),
    (0x021BA648, "d89b41a41fcb6529"),
    (0x021BA670, "3e6d6638e319527a"),
    (0x021BA698, "169c6e7e08173a1c"),
    (0x021BA6C0, "5e625af6e8093901"),
    (0x021BA710, "92a928e0955bdc88"),
    (0x021BA73C, "24916de24cf6ad31"),
    (0x021BA7F8, "f39daafea38d3bf3"),
    (0x021BA820, "841c5c99fffbc699"),
    (0x021BA82C, "b0ddd42a9806695f"),
    (0x021BA878, "daadbe81bcf830e2"),
    (0x021BA9C8, "9434c086a6ac0ed1"),
];
// svflow-recorded-end

/// Base address of `region` k of a spec block (README: `0x02F1_0000 + k * 0x10000`).
fn region_addr(k: usize) -> u32 {
    0x02F1_0000 + k as u32 * 0x1_0000
}

/// Bytes per argument / region slot of the emulator layout; the memory after a buffer up to the next slot is mapped and
/// zero there (a fresh image per function), so a buffer is mapped here padded with zeros to the full slot.
const SLOT: usize = 0x1_0000;

fn padded(mut b: Vec<u8>) -> Vec<u8> {
    b.resize(SLOT, 0);
    b
}

/// The case's `arg buf`s (at their fixed addresses `vals[i]`) and regions as one [`Mem`] (plus the ROM tables). Returns the
/// buffer sizes (0 = scalar argument).
fn take_mem(c: &mut Case) -> (Mem, Vec<usize>) {
    let mut mem = Mem::new();
    let sizes: Vec<usize> = c.bufs.iter().map(Vec::len).collect();
    for (i, b) in c.bufs.iter_mut().enumerate() {
        if !b.is_empty() {
            mem.map(c.vals[i], padded(std::mem::take(b)));
        }
    }
    for (k, r) in c.regions.iter_mut().enumerate() {
        mem.map(region_addr(k), padded(std::mem::take(r)));
    }
    (mem, sizes)
}

fn give_mem(c: &mut Case, mut mem: Mem, sizes: &[usize], region_sizes: &[usize]) {
    for (i, &n) in sizes.iter().enumerate().take(c.bufs.len()) {
        if n != 0 {
            let mut b = mem.unmap(c.vals[i]);
            b.truncate(n);
            c.bufs[i] = b;
        }
    }
    for (k, &n) in region_sizes.iter().enumerate() {
        let mut r = mem.unmap(region_addr(k));
        r.truncate(n);
        c.regions[k] = r;
    }
}

/// `ret=ofs:argK`: a returned pointer as an offset into argument K's buffer (0 stays 0).
fn ofs(addr: u32, base: u32) -> u64 {
    if addr == 0 {
        0
    } else {
        u64::from(addr.wrapping_sub(base))
    }
}

fn call(addr: u32, c: &mut Case) -> u64 {
    use super::*;
    let region_sizes: Vec<usize> = c.regions.iter().map(Vec::len).collect();
    let (mut mem, sizes) = take_mem(c);
    let v = c.vals.clone();
    let e = &mut c.env;
    let m = &mut mem;
    let r: u64 = match addr {
        0x021B_902C => same_or_partner(v[0], v[1]).into(),
        0x021B_904C => pick_own_side_client(e, m, v[0], v[1]).into(),
        0x021B_9064 => pick_client(e, v[0], v[1], v[2]).into(),
        0x021B_9090 => {
            own_view_call_7f1c(e, m, v[0], v[1], v[2]);
            0
        }
        0x021B_90BC => {
            own_view_call_7e50(e, m, v[0], v[1], v[2]);
            0
        }
        0x021B_90E8 => setup_60(m, v[0]).into(),
        0x021B_90F0 => setup_64(m, v[0]).into(),
        0x021B_90F8 => records_locally(m, v[0]).into(),
        0x021B_9114 => is_comm(m, v[0]).into(),
        0x021B_911C => zero().into(),
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
        0x021B_919C => finalize_430(e, m, v[0]).into(),
        0x021B_91E4 => {
            add_money(m, v[0], v[1]);
            0
        }
        0x021B_9210 => money(m, v[0]).into(),
        0x021B_9234 => {
            set_doubling(m, v[0]);
            0
        }
        0x021B_9244 => {
            compute_438(e, m, v[0]);
            0
        }
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
        0x021B_9358 => compare_pc_mons(e, m, v[0], v[1]).into(),
        0x021B_937C => illusion_source(e, m, v[0], v[1], v[2]).into(),
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
        0x021B_9634 => party_last_eligible(e, v[0]).into(),
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
        0x021B_985C => pc_source(m, v[0], v[1]).into(),
        0x021B_9864 => party_index_of_id(e, m, v[0], v[1], v[2]).into(),
        0x021B_98AC => mon_at_position(e, m, v[0], v[1]).into(),
        0x021B_990C => pc_party_get(m, v[0], v[1], v[2]).into(),
        0x021B_9920 => pc_party_get2(m, v[0], v[1], v[2]).into(),
        0x021B_9934 => pc_mon(m, v[0], v[1]).into(),
        0x021B_9940 => pc_mon2(m, v[0], v[1]).into(),
        0x021B_994C => view_obj_of(e, m, v[0], v[1], v[2]).into(),
        0x021B_997C => view_value(e, m, v[0], v[1]).into(),
        0x021B_99C0 => adjusted_6994(e, v[0], v[1]).into(),
        0x021B_99E0 => pos_list_record(v[0]).into(),
        0x021B_99F0 => is_pos_2_or_3(v[0]).into(),
        0x021B_9A00 => pos_8c20(e, m, v[0], v[1]).into(),
        0x021B_9A24 => same_side(v[0], v[1]).into(),
        0x021B_9A40 => side_of_id(v[0]).into(),
        0x021B_9A50 => other_side_of_id(v[0]).into(),
        0x021B_9A5C => is_zero(v[0]).into(),
        // arg 2 is test data only (the block a stubbed 99e0 would point into)
        0x021B_9A6C => pos_list_contains(m, v[0], v[1]).into(),
        0x021B_9AB0 => swap_slot_a(v[0]).into(),
        0x021B_9AC8 => swap_slot_b(v[0]).into(),
        0x021B_9AE0 => side_of_client(v[0]).into(),
        0x021B_9AEC => party_addr(v[0], v[1]).into(),
        0x021B_9AF8 => party_addr2(v[0], v[1]).into(),
        0x021B_9B04 => pc_party_alive(e, m, v[0], v[1]).into(),
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
        0x021B_9B94 => party_count(m, v[0]).into(),
        0x021B_9B98 => party_alive(e, m, v[0]).into(),
        0x021B_9BC4 => party_alive_from(e, m, v[0], v[1]).into(),
        0x021B_9BF0 => party_get(m, v[0], v[1]).into(),
        0x021B_9C00 => party_get2(m, v[0], v[1]).into(),
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
        0x021B_9C98 => party_index_of(m, v[0], v[1]).into(),
        0x021B_9CBC => party_index_of_mon_id(e, m, v[0], v[1]).into(),
        0x021B_9CE8 => party_first_alive(e, m, v[0]).into(),
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
        0x021B_9F70 => record_empty_at(m, v[0], v[1]).into(),
        0x021B_9F84 => record_u16x4(m, v[0], v[1], v[2]).into(),
        0x021B_9FAC => record_ptr1(m, v[0], v[1], v[2]).into(),
        0x021B_9FBC => record_word(e, m, v[0], v[1]).into(),
        0x021B_A004 => record_u16b(m, v[0], v[1]).into(),
        0x021B_A020 => record_u16a(m, v[0], v[1]).into(),
        0x021B_A02C => ofs(record_sub(e, m, v[0], v[1], v[2]), v[0]),
        0x021B_A064 => record_ptr0(m, v[0], v[1]).into(),
        0x021B_A080 => setup_call_e7dc(e, m, v[0]).into(),
        0x021B_A08C => m_2bc(m, v[0]).into(),
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
        0x021B_A178 => party_source(e, m, v[0], v[1], v[2]).into(),
        0x021B_A1A8 => setup_call_3158(e, m, v[0], v[1]).into(),
        0x021B_A1B4 => {
            setup_call_8df0(e, m, v[0], v[1]);
            0
        }
        0x021B_A1CC => {
            setup_call_8dac(e, m, v[0], v[1], v[2]);
            0
        }
        0x021B_A1E4 => setup_78(m, v[0]).into(),
        0x021B_A1EC => setup_7f_flag(e, m, v[0]).into(),
        0x021B_A20C => setup_mode_flag(e, m, v[0]).into(),
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
        0x021B_A534 => is_type_2_or_3(e, v[0]).into(),
        0x021B_A54C => battle_result(e, m, v[0]).into(),
        0x021B_A5C4 => m_442(m, v[0]).into(),
        0x021B_A5D0 => m_440(m, v[0]).into(),
        0x021B_A5D8 => own_client_dda0(e, m, v[0]).into(),
        0x021B_A618 => own_client_ddcc(e, m, v[0]).into(),
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
        0x021B_A710 => client_relation(v[0], v[1]).into(),
        0x021B_A73C => {
            rebuild_battle(e, m, v[0], v[1]);
            0
        }
        0x021B_A7F8 => {
            clients_call_d9b0(e, m, v[0]);
            0
        }
        0x021B_A820 => server_flow_work(e, m, v[0]).into(),
        0x021B_A82C => {
            clear_stats(m, v[0]);
            0
        }
        0x021B_A878 => {
            collect_stats(e, m, v[0]);
            0
        }
        // arg 5 is test data only: the heap block the stubbed allocator returns
        0x021B_A9C8 => ofs(server_create(e, m, v[0], v[1], v[2], v[3], v[4]), v[5]),
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    };
    give_mem(c, mem, &sizes, &region_sizes);
    r
}

/// Every ported function against the original (`tools/rom-diff/svflow.py server3`).
#[test]
fn svflow_match_rom() {
    run_area_suite(
        "server3",
        include_str!("../../../../../tools/rom-diff/svflow_specs/server3.spec"),
        super::PORTED,
        RECORDED,
        call,
    );
}
