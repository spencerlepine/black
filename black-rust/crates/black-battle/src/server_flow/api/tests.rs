use crate::server_flow::testkit::{run_area_suite, Case};

// Written by tools/check-svflow.sh only, after the Rust hashes matched the ROM run.
// svflow-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021C7CBC, "b88ee8e5c09d85ab"),
    (0x021C7CC8, "e49c2ef08a82440f"),
    (0x021C7CD4, "2a551cbea4b3ffa5"),
    (0x021C7CDC, "526a6355c2a60603"),
    (0x021C7CE8, "35d9f88f40ca0559"),
    (0x021C7CF4, "b37143d193e00aec"),
    (0x021C7D00, "66f74a99c9c02293"),
    (0x021C7D54, "7435a1b000af5fce"),
    (0x021C7DAC, "51995f6b9e00e592"),
    (0x021C7E14, "f388d8a7daaabab6"),
    (0x021C7EC4, "6b408e356eaea2d5"),
    (0x021C7ED8, "88ad6595a5cd2f15"),
    (0x021C7EEC, "db8aa00e6af2262b"),
    (0x021C7F04, "cd3cec92c968a361"),
    (0x021C7F24, "7cebf0c29bc2f3f7"),
    (0x021C7F4C, "fbb516c5fb9d84fa"),
    (0x021C7F68, "91f08ed764f59198"),
    (0x021C7FCC, "e2d1b920bac0fdad"),
    (0x021C7FE0, "7a07aea34e869091"),
    (0x021C8010, "23d69f578e65316d"),
    (0x021C8020, "4f5ec6d3d88ed0b7"),
    (0x021C8030, "fcce118ee515c5e1"),
    (0x021C804C, "c9ecbf9f96485d44"),
    (0x021C80AC, "38846e5e4a87c898"),
    (0x021C8114, "b39a23f500c0855b"),
    (0x021C8120, "e6741fa6fa941d09"),
    (0x021C812C, "e1e787843c4fe9e4"),
    (0x021C8130, "94a01117780b0a91"),
    (0x021C8138, "285dae81a96611bc"),
    (0x021C8140, "6f48f9b9963ab5ad"),
    (0x021C815C, "26822b975c02e2fd"),
    (0x021C8168, "340e82d0e74f0379"),
    (0x021C8174, "cdda66916abcdc5a"),
    (0x021C8180, "e67b5d77eeef04ab"),
    (0x021C818C, "4961b2e22b4c29b1"),
    (0x021C81C8, "3d658e720f1630aa"),
    (0x021C81D0, "89d4a8cdd5576b34"),
    (0x021C8234, "0dd51aa44d5331e4"),
    (0x021C824C, "2610bf53043a52c8"),
    (0x021C8288, "4cc62dba5749dcc4"),
    (0x021C8290, "9b32a7b5f06a1445"),
    (0x021C82B8, "2373f8e428521c2a"),
    (0x021C82D0, "95eaadbc39c707a4"),
    (0x021C8300, "e127edd7b8ffa388"),
    (0x021C830C, "aea60c92e3cdbc9f"),
    (0x021C831C, "93b0f4dbaf0773c3"),
    (0x021C8338, "f16a1fccd499b665"),
    (0x021C8348, "5c04c187831cabac"),
    (0x021C8374, "3c42e589111fe8d0"),
    (0x021C83A0, "90f5e1bb82d985af"),
    (0x021C83CC, "40d2134672d20233"),
    (0x021C83D4, "703d84480dd1074d"),
    (0x021C83DC, "9eb570cbafc91d6d"),
    (0x021C83E8, "f09731e64dc8791e"),
    (0x021C8408, "1a5efc896ce27933"),
    (0x021C8434, "6558784ff8075dd8"),
    (0x021C846C, "9442853c3ebe9021"),
    (0x021C8494, "3367b8c88fb40e2f"),
    (0x021C84AC, "e4dcd871facefd7b"),
    (0x021C84D0, "7da583e9c1d9acd6"),
    (0x021C84D8, "6fb3574ebb333ac5"),
    (0x021C84E0, "b2b1e1012e9196c5"),
    (0x021C84E8, "8378b3c9c37fc993"),
    (0x021C84F4, "75ad627fa25e642a"),
    (0x021C8534, "7805502148f405ba"),
    (0x021C8588, "6295d6a1c5d5b93a"),
    (0x021C859C, "61e4dc13da693636"),
    (0x021C85B8, "977c06c10ac0f2a8"),
    (0x021C85D4, "c1131dce6ba2ed7e"),
];
// svflow-recorded-end

pub(crate) fn call<E: crate::server_flow::env::FlowEnv>(addr: u32, c: &mut Case<E>) -> u64 {
    let v = c.vals.clone();
    let wa = v[0];
    let e = &mut c.env;
    let w = &mut c.work;
    match addr {
        0x021C_7CBC => super::word_1f7c(w) as u64,
        0x021C_7CC8 => super::word_1f7e(w) as u64,
        0x021C_7CD4 => super::word_1f80(w) as u64,
        0x021C_7CDC => super::word_1f82(w) as u64,
        0x021C_7CE8 => super::word_1f86(w) as u64,
        0x021C_7CF4 => super::mon_by_id(e, w, v[1]) as u64,
        0x021C_7D00 => super::poke_pos(e, w, v[1]) as u64,
        0x021C_7D54 => super::list_foes(e, v[1], &mut c.bufs[2]) as u64,
        0x021C_7DAC => super::move_type_check(e, w, v[1], v[2], v[3]) as u64,
        0x021C_7E14 => super::move_damage_value(e, w, v[1], v[2], v[3], v[4], v[5]) as u64,
        0x021C_7EC4 => super::in_type_check(w) as u64,
        0x021C_7ED8 => super::client_value_997c(e, w, v[1]) as u64,
        0x021C_7EEC => super::client_byte9(e, w, wa, v[1]) as u64,
        0x021C_7F04 => super::client_count(e, w, wa, v[1]) as u64,
        0x021C_7F24 => super::party_check(e, w, wa, v[1]) as u64,
        0x021C_7F4C => super::take_flag_78a_bit3(w) as u64,
        0x021C_7F68 => super::expand_poke_ids(e, w, v[1], &mut c.bufs[2]) as u64,
        0x021C_7FCC => super::party_of(e, w, v[1]) as u64,
        0x021C_7FE0 => super::partner_party_of(e, w, v[1]) as u64,
        0x021C_8010 => super::poke_pos_in_battle(e, w, v[1]) as u64,
        0x021C_8020 => super::poke_id_at_pos(e, w, v[1]) as u64,
        0x021C_8030 => super::mon_query_80bc(e, w, v[1], v[2]) as u64,
        0x021C_804C => super::find_action(e, w, v[1], &mut c.bufs[2]) as u64,
        0x021C_80AC => super::others_actions_done(e, w, v[1]) as u64,
        0x021C_8114 => super::word_1f74(w) as u64,
        0x021C_8120 => super::ofs_1fec() as u64,
        0x021C_812C => super::ofs_1c() as u64,
        0x021C_8130 => super::ofs_3e0() as u64,
        0x021C_8138 => super::turn_count(w) as u64,
        0x021C_8140 => super::byte_7d9(w, v[1]) as u64,
        0x021C_815C => super::rule(e, w) as u64,
        0x021C_8168 => super::main_86b8(e, w) as u64,
        0x021C_8174 => super::main_8714_word4(e, w, wa) as u64,
        0x021C_8180 => super::main_8670(e, w) as u64,
        0x021C_818C => super::species_check(e, w, v[1]) as u64,
        0x021C_81C8 => super::mon_stat(e, v[1], v[2]) as u64,
        0x021C_81D0 => super::count_faster(e, v[1], v[2]) as u64,
        0x021C_8234 => super::mon_query_6f0c(e, w, v[1]) as u64,
        0x021C_824C => super::mon_check_824c(e, w, v[1]) as u64,
        0x021C_8288 => super::weather(e) as u64,
        0x021C_8290 => super::mon_pair_bfa00(e, w, v[1], v[2], v[3]) as u64,
        0x021C_82B8 => super::main_911c(e) as u64,
        0x021C_82D0 => super::side_condition(e, v[1], v[2]) as u64,
        0x021C_8300 => super::main_9010(e, w, v[1]) as u64,
        0x021C_830C => super::flag_78a_bit0(w) as u64,
        0x021C_831C => {
            super::list6_push(w, v[1]);
            0
        }
        0x021C_8338 => super::mon_query_5c30(e, w, v[1]) as u64,
        0x021C_8348 => super::mon_field4_in_range(e, w, v[1]) as u64,
        0x021C_8374 => super::mon_check_8374(e, w, v[1]) as u64,
        0x021C_83A0 => super::mon_weight(e, w, v[1]) as u64,
        0x021C_83CC => super::counter_2e4c(e) as u64,
        0x021C_83D4 => super::core1_bc2b0(e) as u64,
        0x021C_83DC => {
            // region 0 = the per-turn record behind work[0x4A8]
            super::set_turn_rec_byte4(&mut c.regions[0], v[1]);
            0
        }
        0x021C_83E8 => super::server_bb908(e, w, v[1]) as u64,
        0x021C_8408 => {
            super::server_bb914(e, w, v[1]);
            0
        }
        0x021C_8434 => {
            super::bounce_move(e, w, v[1], v[2]);
            0
        }
        0x021C_846C => super::mon_pair_bfeac(e, w, v[1], v[2], v[3]) as u64,
        0x021C_8494 => {
            super::mon_bff60(e, w, v[1]);
            0
        }
        0x021C_84AC => super::call_2128_flag(e, w, v[1], &mut c.bufs[2]) as u64,
        0x021C_84D0 => {
            super::core2_bdbb8(e, v[1]);
            0
        }
        0x021C_84D8 => super::events_cc668(e, v[1]) as u64,
        0x021C_84E0 => {
            super::set_add_thunk(w, v[1]);
            0
        }
        0x021C_84E8 => {
            super::set_clear(w);
            0
        }
        0x021C_84F4 => {
            super::set_add(w, v[1]);
            0
        }
        0x021C_8534 => super::set_any_8578(e, w) as u64,
        0x021C_8588 => {
            super::move_records_reset(e, w, wa);
            0
        }
        0x021C_859C => {
            super::move_records_push(e, w, wa);
            0
        }
        0x021C_85B8 => {
            super::move_records_pop(e, w, wa);
            0
        }
        0x021C_85D4 => {
            super::move_records_select(e, w, wa, v[1], v[2]);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/rom-diff/svflow.py api`).
#[test]
fn svflow_match_rom() {
    run_area_suite(
        "api",
        include_str!("../../../../../tools/rom-diff/svflow_specs/api.spec"),
        super::PORTED,
        RECORDED,
        call,
    );
}
