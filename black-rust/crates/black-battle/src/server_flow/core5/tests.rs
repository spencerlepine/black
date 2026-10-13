use crate::server_flow::core2::Cx;
use crate::server_flow::testkit::{run_area_suite, Case};

// Written by tools/check-svflow.sh only, after the Rust hashes matched the ROM run.
// svflow-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021C4838, "d0bdc94aedac1386"),
    (0x021C4910, "5303d6f27e1364ba"),
    (0x021C493C, "1d975797d5ce5f11"),
    (0x021C4A64, "7407233c92541c49"),
    (0x021C4B3C, "f5ca5e20f4110a13"),
    (0x021C4B9C, "8bea30f19c798298"),
    (0x021C4BAC, "4bf79666edac27d1"),
    (0x021C4C24, "ee3e0ba34ab2c68d"),
    (0x021C4C40, "12bf456725eca877"),
    (0x021C4C50, "3649b64c9867924f"),
    (0x021C4D1C, "278513ab56386f22"),
    (0x021C4E38, "99e493dda6d6d2fa"),
    (0x021C4EA4, "d27289ae21129014"),
    (0x021C4F64, "b127bd4c0ccae528"),
    (0x021C5098, "9bef5f193839011e"),
    (0x021C5108, "a5309ccc02b41980"),
    (0x021C51D0, "cbcc786b5e8b49ef"),
    (0x021C5200, "aeb7d81af49284dd"),
    (0x021C5308, "309de17d002364e6"),
    (0x021C5350, "36f13433f4360b8f"),
    (0x021C5418, "3c56e5cb584cc632"),
    (0x021C5474, "f6e77b393403afa3"),
    (0x021C5480, "80b9d7a0d04d65ae"),
    (0x021C54BC, "1ca489fdf6807429"),
    (0x021C54C8, "cb3e4e8eb7a6b31b"),
    (0x021C54FC, "3631849b201d25cd"),
    (0x021C5540, "a9206ead8c5f2d7f"),
    (0x021C557C, "c2d897726ddf986d"),
    (0x021C5604, "4758da294f76cbcf"),
    (0x021C5620, "af9a02d04d81c653"),
    (0x021C56A8, "bfb9b4770b22d047"),
    (0x021C5718, "ba014fcdb4302d34"),
    (0x021C572C, "7855552eccb36c5f"),
    (0x021C5750, "9dde03335672c1bd"),
    (0x021C5774, "d643fcf7569e4722"),
    (0x021C5798, "fe653540e8f19b99"),
    (0x021C5840, "99f0c4930e831c0a"),
    (0x021C59C4, "93c17e0cc6107697"),
    (0x021C5A1C, "81b2d87f481a5033"),
    (0x021C5A34, "851f63fdeed31847"),
    (0x021C5A4C, "2eadb13f2cee3420"),
    (0x021C5A8C, "35717dc89482ee69"),
    (0x021C5B74, "fe2858e203e2281a"),
    (0x021C5BE4, "0c1aef21354ec4cc"),
    (0x021C5C1C, "5a40af48ce555784"),
    (0x021C5C74, "c85a7a8c911139d3"),
    (0x021C5D88, "50f6b4164888b8ad"),
    (0x021C5DAC, "5880ccbe5ed15ce6"),
    (0x021C5E48, "3dc1b2c1f6d78370"),
    (0x021C5EFC, "83905b1e298d8a55"),
    (0x021C5F98, "40b53a4580606f06"),
    (0x021C604C, "7233735018106562"),
    (0x021C6080, "83e4c0bea804d855"),
    (0x021C60D4, "be10b9fd36f5fb19"),
    (0x021C60EC, "0dfb3df9abb15f4e"),
    (0x021C6110, "6b67443c522c6ba0"),
    (0x021C6134, "2240ef6d0c31e3d9"),
    (0x021C6158, "d88e693304924623"),
    (0x021C6190, "49ca73c2e4c55bd2"),
    (0x021C62D8, "9fccd2785b250157"),
    (0x021C6350, "058ce8b874ad5b59"),
    (0x021C6394, "cf3768749a348a2a"),
    (0x021C63D0, "a60ed4056d33e119"),
    (0x021C6458, "e7be92c8a1e24c6a"),
    (0x021C64B4, "ec9e8f20d5622251"),
    (0x021C6564, "d08745a15caac139"),
];
// svflow-recorded-end

/// The command queue context: region 0 = the queue buffer (Q), global 0 = the put_cmd scratch
/// words at 0x021F3B44 (a macro so the borrows of `c.env` / `c.work` stay disjoint).
macro_rules! cx {
    ($c:ident) => {
        Cx {
            q: &mut $c.regions[0],
            scratch: &mut $c.globals[0],
            w_addr: $c.vals[0],
        }
    };
}

fn call(addr: u32, c: &mut Case) -> u64 {
    match addr {
        0x021C_4910 => {
            super::mon_message(&mut c.env, c.vals[1], c.vals[2]);
            0
        }
        0x021C_51D0 => {
            super::mon_message_a3(&mut c.env, c.vals[1]);
            0
        }
        0x021C_4E38 => super::message_26e8(&mut c.env, c.vals[1], c.vals[2], c.vals[3]) as u64,
        0x021C_4EA4 => {
            super::hem_damage_messages(&mut c.env, c.vals[1], c.vals[2], c.vals[3] as i32);
            0
        }
        0x021C_5108 => {
            let mut cx = cx!(c);
            super::mon_pass(&mut c.env, &mut cx, c.vals[1], c.vals[2]);
            0
        }
        0x021C_4F64 => {
            let mut cx = cx!(c);
            super::mon_once(&mut c.env, &mut c.work, &mut cx, c.vals[1]) as u64
        }
        0x021C_5350 => {
            let mut cx = cx!(c);
            let rec = (c.vals[2] - 0x02E0_0000) as usize;
            super::record_messages(&mut c.env, &c.work, &mut cx, c.vals[1], rec) as u64
        }
        0x021C_5308 => {
            let mut cx = cx!(c);
            super::species_gate(&mut c.env, &c.work, &mut cx, c.vals[1]) as u64
        }
        0x021C_5200 => {
            let mut cx = cx!(c);
            super::end_of_turn_pass(&mut c.env, &mut c.work, &mut cx) as u64
        }
        0x021C_4838 => {
            let mut cx = cx!(c);
            super::walk_list(&mut c.env, &mut c.work, &mut cx, c.vals[1], c.vals[2]) as u64
        }
        0x021C_4D1C => {
            let mut cx = cx!(c);
            super::walk_list2(&mut c.env, &mut c.work, &mut cx, c.vals[1]) as u64
        }
        0x021C_4C24 => {
            let mut cx = cx!(c);
            super::register_callback(&mut c.env, &mut cx);
            0
        }
        0x021C_4C50 => {
            let mut cx = cx!(c);
            super::sel_message(&mut c.env, &c.work, &mut cx, c.vals[1]);
            0
        }
        0x021C_4C40 => {
            let mut cx = cx!(c);
            super::sel_message_cb(&mut c.env, &c.work, &mut cx, c.vals[0]);
            0
        }
        0x021C_4B9C => {
            super::put_selector_message_cb(&mut c.env, &mut c.regions[0], c.vals[0], c.vals[1]);
            0
        }
        0x021C_5A1C => {
            // region 0 = the command queue buffer (work + 0xC), global 0 = the put_cmd scratch words
            super::cmd_57(&mut c.env, &mut c.regions[0], &mut c.globals[0], c.vals[1]);
            0
        }
        0x021C_56A8 => super::decode_event_var(&mut c.env, c.vals[0], &mut c.bufs[1]) as u64,
        0x021C_5098 => super::max_level(&mut c.env, &c.work) as u64,
        0x021C_4BAC => {
            super::put_selector_message(&mut c.env, &mut c.regions[0], c.vals[1], c.vals[2]);
            0
        }
        0x021C_4B3C => {
            super::message_round_trip(&mut c.env, c.vals[1], c.vals[2], c.vals[3]) as u64
        }
        0x021C_493C => {
            let mut cx = cx!(c);
            super::pair_pass(&mut c.env, &mut c.work, &mut cx, c.vals[1]) as u64
        }
        0x021C_4A64 => {
            let mut cx = cx!(c);
            super::bracketed_effect(
                &mut c.env,
                &mut c.work,
                &mut cx,
                c.vals[1],
                c.vals[2],
                c.vals[3],
            );
            0
        }
        0x021C_5418 => {
            let mut cx = cx!(c);
            super::apply_status(&mut c.env, &mut cx, c.vals[1], c.vals[2]) as u64
        }
        0x021C_5474 => {
            super::clear_220(&mut c.work);
            0
        }
        0x021C_5480 => {
            let mut cx = cx!(c);
            super::put_slot_message(&mut c.env, &mut cx, c.vals[1], c.vals[2]);
            0
        }
        0x021C_54BC => {
            let mut cx = cx!(c);
            super::put_message_350(&mut c.env, &mut cx, c.vals[1]);
            0
        }
        0x021C_54C8 => {
            let mut cx = cx!(c);
            super::put_slot_message_25b(&mut c.env, &mut cx, c.vals[1]);
            0
        }
        0x021C_54FC => {
            let mut cx = cx!(c);
            super::put_slot_message_25c(&mut c.env, &mut cx, c.vals[1]);
            0
        }
        0x021C_5540 => {
            let mut cx = cx!(c);
            super::damage_commands(&mut c.env, &c.work, &mut cx, c.vals[1], c.vals[2] as i32);
            0
        }
        0x021C_557C => {
            let mut cx = cx!(c);
            super::cmd_10_35(&mut c.env, &mut cx, c.vals[1], c.vals[2], c.vals[3]);
            0
        }
        0x021C_5604 => {
            let mut cx = cx!(c);
            super::cmd_59(&mut c.env, &mut cx, c.vals[1], c.vals[2]);
            0
        }
        0x021C_5750 => {
            let mut cx = cx!(c);
            super::message_d2(&mut c.env, &mut cx, c.vals[1]);
            0
        }
        0x021C_5774 => {
            let mut cx = cx!(c);
            super::message_d8(&mut c.env, &mut cx, c.vals[1]);
            0
        }
        0x021C_5840 => {
            let mut cx = cx!(c);
            super::count_messages(
                &mut c.env, &mut cx, c.vals[1], &c.bufs[2], &c.bufs[3], c.vals[4],
            );
            0
        }
        0x021C_59C4 => {
            let mut cx = cx!(c);
            super::flagged_messages(
                &mut c.env, &mut cx, c.vals[1], &c.bufs[2], &c.bufs[3], c.vals[4],
            );
            0
        }
        0x021C_5A34 => {
            let mut cx = cx!(c);
            super::cmd_58(&mut c.env, &mut cx, c.vals[1]);
            0
        }
        0x021C_5A4C => {
            let mut cx = cx!(c);
            super::message_99_ae(&mut c.env, &mut cx, c.vals[1], c.vals[2], c.vals[3] as i32);
            0
        }
        0x021C_5A8C => {
            let mut cx = cx!(c);
            super::stat_change(
                &mut c.env,
                &mut cx,
                c.vals[1],
                c.vals[2],
                c.vals[3] as i32,
                c.vals[4],
                c.vals[5],
            );
            0
        }
        0x021C_5B74 => {
            let mut cx = cx!(c);
            super::hp_change(
                &mut c.env,
                &c.work,
                &mut cx,
                c.vals[1],
                c.vals[2] as i32,
                c.vals[3],
            );
            0
        }
        0x021C_5BE4 => {
            let mut cx = cx!(c);
            super::cmd_03_43(&mut c.env, &mut cx, c.vals[1], c.vals[2]);
            0
        }
        0x021C_5C1C => {
            let mut cx = cx!(c);
            super::cmd_0f(&mut c.env, &mut cx, c.vals[1], c.vals[2], c.vals[3]);
            0
        }
        0x021C_5C74 => {
            let mut cx = cx!(c);
            super::mode_message(
                &mut c.env,
                &mut c.work,
                &mut cx,
                c.vals[1],
                c.vals[2],
                c.vals[3],
            );
            0
        }
        0x021C_5D88 => {
            let mut cx = cx!(c);
            super::put_mon_message(&mut c.env, &mut cx, c.vals[1], c.vals[2]);
            0
        }
        0x021C_5E48 => {
            let mut cx = cx!(c);
            super::message_5c_words(
                &mut c.env, &mut cx, c.vals[1], c.vals[2], c.vals[3], &c.bufs[4],
            );
            0
        }
        0x021C_5EFC => {
            let mut cx = cx!(c);
            super::message_5b_words(&mut c.env, &mut cx, c.vals[1], c.vals[2], &c.bufs[3]);
            0
        }
        0x021C_5F98 => {
            let mut cx = cx!(c);
            super::message_5d_words(
                &mut c.env, &mut cx, c.vals[1], c.vals[2], c.vals[3], &c.bufs[4],
            );
            0
        }
        0x021C_604C => {
            let mut cx = cx!(c);
            super::cmd_01(&mut c.env, &c.work, &mut cx, c.vals[1], c.vals[2]);
            0
        }
        0x021C_6080 => {
            let mut cx = cx!(c);
            super::cmd_07_08(
                &mut c.env, &mut cx, c.vals[1], c.vals[2], c.vals[3], c.vals[4],
            );
            0
        }
        0x021C_60D4 => {
            let mut cx = cx!(c);
            super::cmd_42(&mut c.env, &mut cx, c.vals[1]);
            0
        }
        0x021C_60EC => {
            let mut cx = cx!(c);
            super::cmd_19(&mut c.env, &mut cx, c.vals[1], c.vals[2]);
            0
        }
        0x021C_6110 => {
            let mut cx = cx!(c);
            super::cmd_1a(&mut c.env, &mut cx, c.vals[1], c.vals[2]);
            0
        }
        0x021C_6134 => {
            let mut cx = cx!(c);
            super::cmd_1b(&mut c.env, &mut cx, c.vals[1], c.vals[2]);
            0
        }
        0x021C_6158 => {
            let mut cx = cx!(c);
            super::list_mons(&mut c.env, &c.work, &mut cx, &c.bufs[1]);
            0
        }
        0x021C_6190 => super::mark_species_entries(&mut c.env, &mut c.work, c.vals[1]) as u64,
        // ---- ported, but outside PORTED until B10j-fix (the callers' stubs in core1-4 are wrong) ----
        0x021C_5620 => {
            // region 0 = OUT (the u32 r3 points at), region 1 = the command queue Q
            let (out, q) = c.regions.split_at_mut(1);
            let mut cx = Cx {
                q: &mut q[0],
                scratch: &mut c.globals[0],
                w_addr: c.vals[0],
            };
            let out = if c.vals[3] == 0 {
                None
            } else {
                Some(&mut out[0][..])
            };
            super::set_status_value(&mut c.env, &mut cx, c.vals[1], c.vals[2], out);
            0
        }
        0x021C_5718 => {
            let mut cx = cx!(c);
            super::message_47(&mut c.env, &mut cx);
            0
        }
        0x021C_572C => {
            let mut cx = cx!(c);
            super::message_d5(&mut c.env, &mut cx, c.vals[1]);
            0
        }
        0x021C_5798 => {
            let mut cx = cx!(c);
            super::entry_commands(
                &mut c.env, &c.work, &mut cx, &c.bufs[1], c.vals[2], &c.bufs[3], &c.bufs[4],
                &c.bufs[5],
            );
            0
        }
        0x021C_5DAC => {
            let mut cx = cx!(c);
            super::message_5a_words(&mut c.env, &mut cx, c.vals[1], c.vals[2], &c.bufs[3]);
            0
        }
        0x021C_62D8 => {
            super::round_trip_2c3f(&mut c.env, c.vals[1], c.vals[3], &mut c.bufs[4]) as u64
        }
        0x021C_6350 => super::round_trip_2c5d(&mut c.env, c.vals[1]) as u64,
        0x021C_6394 => {
            super::message_2c72(&mut c.env, c.vals[1], c.vals[2]);
            0
        }
        0x021C_63D0 => {
            let mut cx = cx!(c);
            super::base_damage_effect(&mut c.env, &c.work, &mut cx, c.vals[1]) as u64
        }
        0x021C_6458 => super::round_trip_2cb0(&mut c.env, c.vals[1], c.vals[3], c.vals[4]) as u64,
        0x021C_64B4 => {
            super::round_trip_2cc8(&mut c.env, c.vals[1], c.vals[2], &mut c.bufs[3]);
            0
        }
        0x021C_6564 => {
            super::message_2cf4(&mut c.env, c.vals[1], c.vals[2], c.vals[3]);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/rom-diff/svflow.py core5`).
#[test]
fn svflow_match_rom() {
    run_area_suite(
        "core5",
        include_str!("../../../../../tools/rom-diff/svflow_specs/core5.spec"),
        super::PORTED,
        RECORDED,
        call,
    );
}
