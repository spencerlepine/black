use crate::server::Mem;
use crate::server_flow::testkit::{run_area_suite, Case};

// Written by tools/check-svflow.sh only, after the Rust hashes matched the ROM run.
// svflow-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021B75A8, "5673f1fb31b1743e"),
    (0x021B7758, "ae71151d321ba52b"),
    (0x021B7968, "c45ed70fa6243b2c"),
    (0x021B7BB0, "8920c3362b76829a"),
    (0x021B7CB4, "59ae49f1b198c114"),
    (0x021B7E18, "439bdc35bce3a681"),
    (0x021B8070, "e0daf44ed8e19e51"),
    (0x021B81DC, "4b371222ce155569"),
    (0x021B835C, "9272d61ce5e3a382"),
    (0x021B83F4, "327098e014fb0740"),
    (0x021B8434, "7bd323576a8e7577"),
    (0x021B84D8, "ba08c1517a1c4858"),
    (0x021B8548, "f702ad2522c0e5cc"),
    (0x021B8564, "a6ecf34d65d89d16"),
    (0x021B8588, "b23927669d831420"),
    (0x021B8590, "403fe0745cbd216a"),
    (0x021B85A0, "5226fa811a9e02d1"),
    (0x021B85A8, "2c0bcebe9ef60f0a"),
    (0x021B85DC, "c46220d304425e7b"),
    (0x021B85EC, "aa5a46b306ca859b"),
    (0x021B8630, "d12835d296df2122"),
    (0x021B8640, "a937de7bdd2bbef5"),
    (0x021B8670, "ccdb76e7d39a0391"),
    (0x021B86A0, "6b27667cd5486dbb"),
    (0x021B86B8, "d4f3c7a39f28dde7"),
    (0x021B86C0, "f8f5801e281ff444"),
    (0x021B86C8, "31ab85b441072823"),
    (0x021B86DC, "a7b9f8775c59b6a6"),
    (0x021B86E4, "43c2ce4deec2692c"),
    (0x021B8714, "bc51b30456c20726"),
    (0x021B871C, "c83e8bd7dbaef270"),
    (0x021B8724, "9285d6776bfc3096"),
    (0x021B873C, "63d4565a79917b39"),
    (0x021B8748, "5900701c5f4da587"),
    (0x021B8750, "d562bbd8eab733f6"),
    (0x021B8778, "2e5f623f6d8cb7a6"),
    (0x021B87E8, "d7a8637736446ba5"),
    (0x021B8834, "8fc85cd1fc96b205"),
    (0x021B88EC, "50855a3abc931e79"),
    (0x021B8B78, "10d2ebcfeca2829e"),
    (0x021B8B9C, "251d2250814c7913"),
    (0x021B8BC4, "5d5aaff7f8f22031"),
    (0x021B8BD0, "8c3e386acdc89169"),
    (0x021B8BEC, "00a0158b291fc50a"),
    (0x021B8C20, "d0cfb07ee0171d8b"),
    (0x021B8C38, "ba9d2812666db049"),
    (0x021B8C50, "f484422bc48c5609"),
    (0x021B8C5C, "8ca7404a6b271067"),
    (0x021B8CB0, "e19e7b875896b892"),
    (0x021B8CE8, "c9808fdebdbf29ea"),
    (0x021B8CFC, "1096681909091889"),
    (0x021B8D08, "44912e8ccf62549c"),
    (0x021B8D38, "3cb3b038fae3a807"),
    (0x021B8D8C, "41d8dd4f363dc346"),
    (0x021B8DC0, "a43ef86123fe72a5"),
    (0x021B8DEC, "d1a9b2e508784f77"),
    (0x021B8E28, "d0d530f490003676"),
    (0x021B8E70, "6b8d577a2bc4a507"),
    (0x021B8ED8, "4dc45f0d7a483c0b"),
    (0x021B8F38, "2e9f6c143011c107"),
    (0x021B8F5C, "cc4b9bba4af409b3"),
    (0x021B8F7C, "72485c3512c6550e"),
    (0x021B8FC0, "2804eff18ddac51e"),
    (0x021B8FE4, "4a5d640866417361"),
    (0x021B8FF4, "43f39128e615f3ea"),
    (0x021B9000, "87866e6229faa8bc"),
    (0x021B9010, "4269532c16d3cf9b"),
];
// svflow-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
    let Case {
        vals, bufs, env, ..
    } = c;
    let v = |i: usize| vals[i];
    // argument 0 is M's arena (`arg buf 0x1000`) for every function that takes M; the other `buf`s stay separate
    let (b0, rest) = bufs.split_at_mut(1);
    let mut mem = Mem::new();
    if !b0[0].is_empty() {
        mem.add(vals[0], &mut b0[0]);
    }
    let m = vals[0];
    let r: u32 = match addr {
        0x021B_75A8 | 0x021B_7758 | 0x021B_7968 | 0x021B_7BB0 | 0x021B_835C => {
            mem.add(vals[1], &mut rest[0]);
            let f = match addr {
                0x021B_75A8 => super::step_handshake,
                0x021B_7758 => super::step_party_sync,
                0x021B_7968 => super::step_record_exchange,
                0x021B_7BB0 => super::step_data_exchange,
                _ => super::step_view,
            };
            f(env, &mut mem, m, vals[1])
        }
        0x021B_7CB4 => super::step_create_pair(env, &mut mem, m),
        0x021B_7E18 => super::step_create_multi(env, &mut mem, m),
        0x021B_8070 => super::step_create_mode3(env, &mut mem, m),
        0x021B_81DC => super::step_create_many(env, &mut mem, m),
        0x021B_83F4 => super::frame_no_server(env, &mut mem, m),
        0x021B_8434 => super::main_loop_server(env, &mut mem, m),
        0x021B_84D8 => super::main_loop_client(env, &mut mem, m),
        0x021B_8548 => {
            super::quit_clients(env, &mem, m);
            0
        }
        0x021B_8564 => super::all_clients_done(&mem, m),
        0x021B_8588 => super::rule(&mem, m),
        0x021B_8590 => super::flag_bit2(&mem, m),
        0x021B_85A0 => super::setup_byte_80(&mem, m),
        0x021B_85A8 => super::is_plain_single(env, &mem, m),
        0x021B_85DC => super::call_02013080(env, &mem, m),
        0x021B_85EC => super::check_overlay_231(env, &mem, m),
        0x021B_8630 => super::setup_word_70(&mem, m),
        0x021B_8640 => super::rule_count_8640(&mem, m),
        0x021B_8670 => super::rule_positions_per_side(&mem, m),
        0x021B_86A0 => super::pos_in_range(&mem, m, v(1)),
        0x021B_86B8 => super::battle_type(&mem, m),
        0x021B_86C0 => super::setup_byte_20(&mem, m),
        0x021B_86C8 => super::setup_byte_22_set(&mem, m),
        0x021B_86DC => super::setup_u16_1a(&mem, m),
        0x021B_86E4 => super::type_class(&mem, m),
        // ret=ofs:arg0
        0x021B_8714 => super::setup_addr_8(&mem, m).wrapping_sub(m),
        0x021B_871C => super::word_c0(&mem, m),
        0x021B_8724 => super::call_0200ced4(env, &mem, m, v(1)),
        0x021B_873C => super::call_0200cf28(env, &mem, m),
        0x021B_8748 => super::setup_word_58(&mem, m),
        0x021B_8750 => {
            super::call_0200cfa0(env, &mem, m, v(1), v(2));
            0
        }
        0x021B_8778 => super::expand_target(env, &mem, m, v(1), &mut rest[1]),
        0x021B_87E8 => super::expand_single(&mem, m, v(1), v(2), &mut rest[2]),
        0x021B_8834 => super::expand_double(&mem, m, v(1), v(2), &mut rest[2]),
        0x021B_88EC => super::expand_triple(env, &mem, m, v(1), v(2), &mut rest[2]),
        0x021B_8B78 => super::client_pos_count(&mem, m, v(1)),
        0x021B_8B9C => super::client_present(&mem, m, v(1)),
        0x021B_8BC4 => super::client_side(v(1)),
        0x021B_8BD0 => super::is_own_side(&mem, m, v(1)),
        0x021B_8BEC => super::client_nth_pos(&mem, m, v(1), v(2)),
        0x021B_8C20 => super::pos_of(v(0), v(1), v(2)),
        0x021B_8C38 => super::side_slot(v(0), v(1)),
        0x021B_8C50 => super::opponent_slot(&mem, m, v(1), v(2)),
        0x021B_8C5C => super::facing_pos(v(0), v(1)),
        0x021B_8CB0 => super::partner_pos(&mem, m, v(1)),
        0x021B_8CE8 => super::opposing(v(1), v(2)),
        0x021B_8CFC => super::pos_client(&mem, m, v(1)),
        0x021B_8D08 => super::mon_id_client(v(0)),
        0x021B_8D38 => super::target_mon_ids(env, &mem, m, v(1), v(2), &mut rest[2]),
        0x021B_8D8C => super::mon_pos(env, &mem, m, v(1), v(2)),
        0x021B_8DC0 => super::mon_pos_class(env, &mem, m, v(1), v(2)),
        0x021B_8DEC => super::pos_client_slot(&mem, m, v(1)),
        0x021B_8E28 => {
            let (o1, o2) = rest.split_at_mut(2);
            super::pos_client_and_slot(&mem, m, v(1), &mut o1[1], &mut o2[0]);
            0
        }
        0x021B_8E70 => super::pos_class(&mem, m, v(1)),
        0x021B_8ED8 => super::class_pos(&mem, m, v(1)),
        0x021B_8F38 => super::multi_not_type01(&mem, m),
        0x021B_8F5C => super::pokecon_refresh(env, m, v(1)),
        0x021B_8F7C => super::partner_pokecon_refresh(env, &mem, m, v(1)),
        0x021B_8FC0 => super::is_multi_second_pair(&mem, m, v(1)),
        0x021B_8FE4 => super::own_is_multi_second_pair(&mem, m),
        0x021B_8FF4 => super::own_client_id(&mem, m),
        0x021B_9000 => super::own_partner_id(&mem, m),
        0x021B_9010 => super::partner_id(&mem, m, v(1)),
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    };
    r as u64
}

/// Every ported function against the original (`tools/rom-diff/svflow.py server2`).
#[test]
fn svflow_match_rom() {
    run_area_suite(
        "server2",
        include_str!("../../../../../tools/rom-diff/svflow_specs/server2.spec"),
        super::PORTED,
        RECORDED,
        call,
    );
}
