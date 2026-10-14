use crate::server_flow::testkit::{run_area_suite, Case};

// Written by tools/check-svflow.sh only, after the Rust hashes matched the ROM run.
// svflow-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021CDDDC, "9554eeadd4da61e4"),
    (0x021CDE8C, "ba8b3c71b30522f2"),
    (0x021CDECC, "d17275d0d645ca62"),
    (0x021CDEE8, "77a4fffe83da090e"),
    (0x021CDF1C, "80dec792b2b41d6e"),
    (0x021CDF58, "76ddcb9e3fb7a5f0"),
    (0x021CDFE4, "3caf40b1bcc91fbf"),
    (0x021CE040, "debf610d000ff115"),
    (0x021CE044, "52806be3c14f28b3"),
    (0x021CE068, "9185f31a5db52bbd"),
    (0x021CE08C, "0ddbcc975dae779f"),
    (0x021CE0BC, "2a11afd4e1683fa8"),
    (0x021CE0E0, "bd8c6f9e137170f8"),
    (0x021CE0F8, "5a3a118e14893ff4"),
    (0x021CE150, "08c8233f658a0c9d"),
    (0x021CE160, "010f263c37c06a8e"),
    (0x021CE170, "1d89931cf8b248da"),
    (0x021CE1B0, "3d2d11f19263d79c"),
    (0x021CE2E8, "5e42850c8f3bd662"),
    (0x021CE308, "40d0637df0d56a37"),
    (0x021CE3B4, "36a6e3f4090f33c8"),
    (0x021CE3C8, "54f6ac920b0d6353"),
    (0x021CE4D0, "0969733607bc3c6e"),
    (0x021CE7D4, "31001349c1338bde"),
    (0x021CE8F0, "7981d226fc3abe02"),
    (0x021CEA94, "d63ad388c95bbbc7"),
    (0x021CECB4, "6b284c1a66107f80"),
    (0x021CED4C, "5634265fe3861a6a"),
    (0x021CEDC0, "0e7aed5a7b9d8e92"),
    (0x021CEEB8, "dfe49dc2a84d4c6d"),
    (0x021CEFFC, "324ec68d6bb52705"),
    (0x021CF078, "22690d9665886b1a"),
    (0x021CF220, "ce4827925c4ea1e9"),
    (0x021CF2F0, "8270eb328dbfc326"),
    (0x021CF360, "c958536c3ace52d8"),
    (0x021CF37C, "b928c6f57f7dd467"),
    (0x021CF3B8, "de3d0823d869473f"),
    (0x021CF3F8, "2fc9d92585e3a6c9"),
    (0x021CF41C, "8ede79334d90cbba"),
    (0x021CF430, "19d7906b6c117366"),
    (0x021CF43C, "b622fbd723089816"),
    (0x021CF448, "3a8fb19ddd3260b5"),
    (0x021CF470, "d9e36c2deb6d5975"),
    (0x021CF524, "091736cad1972d71"),
    (0x021CF58C, "63674693a181dc61"),
];
// svflow-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
    let a0 = c.vals.first().copied().unwrap_or(0);
    let (env, vals, bufs) = (&mut c.env, &c.vals, &mut c.bufs);
    match addr {
        0x021C_E040 => super::req2_replay() as u64,
        0x021C_DFE4 => super::random_choice(env, vals[0]) as u64,
        0x021C_F470 => super::mon_answered(env, vals[1], vals[2]) as u64,
        0x021C_F58C => {
            super::struggle(env, vals[0], &mut bufs[1], vals[2]);
            0
        }
        _ => {
            let (o, rest) = bufs.split_at_mut(1);
            let o = &mut o[0];
            match addr {
                0x021C_DDDC => super::req1_step(env, o, &mut rest[0]) as u64,
                0x021C_DE8C => super::req1_replay(env, o, &mut rest[0]) as u64,
                0x021C_DECC => {
                    super::store_active_hp(env, o);
                    0
                }
                0x021C_DEE8 => super::hp_drop_percent(env, o) as u64,
                0x021C_DF1C => super::active_mon(env, o) as u64,
                0x021C_DF58 => super::req2_step(env, o, a0, &mut rest[0]) as u64,
                0x021C_E044 => super::req2_auto(env, o, a0) as u64,
                0x021C_E068 => {
                    super::fade_start(env, o);
                    0
                }
                0x021C_E08C => super::fade_poll(env, o) as u64,
                0x021C_E0BC => super::fade_poll_then(env, o, vals[1]) as u64,
                0x021C_E0E0 => {
                    super::fade_end(env, o);
                    0
                }
                0x021C_E150 => {
                    super::set_callback(o, vals[1]);
                    0
                }
                0x021C_E160 => super::run_callback(env, o) as u64,
                0x021C_E170 => super::req3_step(env, o, &mut rest[0]) as u64,
                0x021C_E1B0 => {
                    let (st, recs) = rest.split_at_mut(1);
                    super::req3_replay(env, o, a0, &mut st[0], &recs[0], vals[2]) as u64
                }
                0x021C_E2E8 => {
                    super::answer_skip(env, o, a0);
                    0
                }
                0x021C_E308 => super::select_start(env, o) as u64,
                0x021C_E3B4 => {
                    super::print_message(env, o);
                    0
                }
                0x021C_E3C8 => super::select_finish(env, o, a0, &mut rest[0]) as u64,
                0x021C_E4D0 => super::select_loop(env, o, a0, &mut rest[0]) as u64,
                0x021C_E7D4 => super::item_anim(env, o, &mut rest[0]) as u64,
                0x021C_E8F0 => match super::item_shortcut(env, o) {
                    Some(item) => {
                        rest[0][..2].copy_from_slice(&item.to_le_bytes());
                        1
                    }
                    None => 0,
                },
                0x021C_EA94 => super::fight_menu(env, o, a0, &mut rest[0]) as u64,
                0x021C_ED4C => super::target_page(env, o, &mut rest[0]) as u64,
                0x021C_EDC0 => super::party_page(env, o, &mut rest[0]) as u64,
                0x021C_EEB8 => super::bag_page(env, o, &mut rest[0]) as u64,
                0x021C_EFFC => super::single_live_mon(env, o) as u64,
                0x021C_F078 => super::run_menu(env, o, a0, &mut rest[0]) as u64,
                0x021C_F220 => super::select_next(env, o, a0, &mut rest[0]) as u64,
                0x021C_F2F0 => super::select_end(env, o, &mut rest[0]) as u64,
                0x021C_F360 => {
                    super::clear_1a0(o);
                    0
                }
                0x021C_F37C => {
                    super::set_1a0_slot(env, o, vals[1], vals[2]);
                    0
                }
                0x021C_F3B8 => {
                    super::clear_1a0_slot(env, o, vals[1]);
                    0
                }
                0x021C_F3F8 => {
                    super::clear_1a0_all(env, o);
                    0
                }
                0x021C_F41C => {
                    super::clear_130(o);
                    0
                }
                0x021C_F430 => {
                    super::set_130(o, vals[1], vals[2]);
                    0
                }
                0x021C_F43C => super::get_130(o, vals[1]) as u64,
                0x021C_F448 => super::sum_130(o) as u64,
                0x021C_E0F8 => {
                    super::sent_out_message(env, o, vals[1], vals[2]);
                    0
                }
                0x021C_ECB4 => {
                    super::fight_table(env, o, &mut rest[0]);
                    0
                }
                0x021C_F524 => super::struggle_if_stuck(env, o, vals[1], vals[2]) as u64,
                _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
            }
        }
    }
}

/// Every ported function against the original (`tools/rom-diff/svflow.py client2`).
#[test]
fn svflow_match_rom() {
    run_area_suite(
        "client2",
        include_str!("../../../../../tools/rom-diff/svflow_specs/client2.spec"),
        super::PORTED,
        RECORDED,
        call,
    );
}
