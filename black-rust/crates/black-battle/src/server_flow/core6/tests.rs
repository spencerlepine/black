use crate::server_flow::testkit::{run_area_suite, Case};

// Written by tools/check-svflow.sh only, after the Rust hashes matched the ROM run.
// svflow-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021C65A8, "011b70caf07e4703"),
    (0x021C6668, "dc3c69a0b837724e"),
    (0x021C66AC, "fe72449db37965c0"),
    (0x021C6720, "59b478e37acd0c16"),
    (0x021C676C, "fb05cefd1982f776"),
    (0x021C6848, "217fe94a3d3714a7"),
    (0x021C6878, "67b3421d4b0e1c36"),
    (0x021C68A8, "5839a15cac2d25b9"),
    (0x021C6948, "26830c3e9143eef7"),
    (0x021C69B8, "36c54e6368dfbf20"),
    (0x021C6A04, "4b621e5893e8f6a5"),
    (0x021C6A9C, "3924ba794846a939"),
    (0x021C6B5C, "a5fe6ce8fc7a826c"),
    (0x021C6BB8, "2c6f3eaae872cb31"),
    (0x021C6BF8, "0a25cc9d017fa5c8"),
    (0x021C6CAC, "b5309326d63d60b4"),
    (0x021C6DD0, "df75bfb39653c523"),
    (0x021C6E00, "f4a08692e872411f"),
    (0x021C6E3C, "a95fb83022db5dae"),
    (0x021C6F0C, "406536a6c34bb010"),
    (0x021C6F34, "e8816180034be8be"),
    (0x021C6FC0, "7f95a395a84a0d53"),
    (0x021C7008, "c0f13813ceed04d5"),
    (0x021C7038, "49b0aea8221e5014"),
    (0x021C70E4, "681c68004f9da06f"),
    (0x021C7194, "4f731e0b7d2585b3"),
    (0x021C71E0, "dbd1c16bd13088b5"),
    (0x021C728C, "085540b14237d194"),
    (0x021C7330, "1d06d70e5152c00a"),
    (0x021C7374, "fa4f4e9235e2559c"),
    (0x021C7474, "81a56de2fa5790c5"),
    (0x021C75EC, "16c8f6ea1f2a0e8b"),
    (0x021C7664, "643306c378c1c558"),
    (0x021C76F4, "d348d6ac6eac22fd"),
    (0x021C7780, "0fee06ab35a8cfd8"),
    (0x021C77B0, "d0ab183daef1c65e"),
    (0x021C7820, "90bfcf1540719685"),
    (0x021C785C, "c39a9393dcee74ae"),
    (0x021C78A8, "27c510c9e336bbb1"),
    (0x021C78F4, "60d5361a0c58b4bc"),
    (0x021C7998, "da360892b9e4fb4c"),
    (0x021C79E0, "b8c308883a534b7f"),
    (0x021C7A54, "8c894e77387d19c6"),
    (0x021C7AA0, "c5f8af1b19bc7ac9"),
    (0x021C7AD0, "6dc8c93c5651f84b"),
    (0x021C7B0C, "f61a9222ab12ab3c"),
    (0x021C7B3C, "5729e10d67e2f507"),
    (0x021C7B7C, "8a434033acf09f9c"),
    (0x021C7BA8, "c92c79ac3f49809a"),
    (0x021C7BF8, "1b850c2c85543e11"),
    (0x021C7C18, "a15af322f4ec4195"),
    (0x021C7C24, "6c2dbc75e57ee922"),
];
// svflow-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    match addr {
        0x021C_65A8 => {
            super::list_round_trip(&mut c.env, &mut c.regions[0], v[1], v[2], v[3]) as u64
        }
        0x021C_6668 => super::message_2d36(&mut c.env, v[1]) as u64,
        0x021C_66AC => super::message_2d4f(&mut c.env, v[1], v[2]) as u64,
        0x021C_6720 => super::message_2d66(&mut c.env, v[1], v[2]) as u64,
        0x021C_676C => {
            let (a, b) = c.bufs.split_at_mut(5);
            super::message_2d80(&mut c.env, v[1], v[2], &mut a[4], &mut b[0]) as u64
        }
        0x021C_6848 => {
            super::message_2dac(&mut c.env, v[1]);
            0
        }
        0x021C_6878 => {
            super::message_2dbd(&mut c.env, v[1]);
            0
        }
        0x021C_68A8 => super::message_2dd3(&mut c.env, v[1], v[2]) as u64,
        0x021C_6948 => super::message_2df5(&mut c.env, v[1], v[2]) as u64,
        0x021C_69B8 => super::message_2e15(&mut c.env, v[1], v[3]) as u64,
        0x021C_6A04 => super::message_2e2f(&mut c.env, v[1], v[2], &c.bufs[3]) as u64,
        0x021C_6A9C => super::priority_roll(&mut c.env, v[1], v[2], v[3]) as u64,
        0x021C_6B5C => super::message_2e83(&mut c.env, v[1], v[2], &c.bufs[3]) as u64,
        0x021C_6BB8 => {
            super::message_2e98(&mut c.env, v[1], v[2]);
            0
        }
        0x021C_6BF8 => super::message_2eb0(&mut c.env, &c.work, v[1], v[2], v[3]) as u64,
        0x021C_6CAC => super::scaled_message(&mut c.env, v[1], v[2], v[3], &mut c.bufs[4]) as u64,
        0x021C_6DD0 => {
            super::message_2f2d(&mut c.env, v[1]);
            0
        }
        0x021C_6E00 => {
            super::message_2f3d(&mut c.env, v[1], v[2]);
            0
        }
        0x021C_6E3C => {
            super::type_check(&mut c.env, &mut c.work, v[1], v[2], &c.bufs[3], v[4]) as u64
        }
        0x021C_6F0C => super::pair_resists(&mut c.env, v[1], v[2]) as u64,
        0x021C_6F34 => super::resist_message(&mut c.env, v[1], v[2]) as u64,
        0x021C_6FC0 => {
            super::bracketed_messages(&mut c.env, &mut c.work, v[1]);
            0
        }
        0x021C_7008 => {
            super::message_1b(&mut c.env, v[1]);
            0
        }
        0x021C_7038 => super::type_question(&mut c.env, v[1], v[2], v[3], v[4]) as u64,
        0x021C_70E4 => {
            super::emit_move_command(
                &mut c.env,
                &mut c.regions[0],
                &mut c.globals[0],
                v[1],
                v[2],
                v[3],
                v[4],
                v[5],
            );
            0
        }
        0x021C_7194 => super::message_306f(&mut c.env, v[1], v[2]) as u64,
        0x021C_71E0 => super::power_message(&mut c.env, v[1], v[2], &mut c.bufs[3]) as u64,
        0x021C_728C => super::move_record_message(&mut c.env, v[1], v[2], &c.bufs[3]) as u64,
        0x021C_7330 => super::message_30e9(&mut c.env, v[1]) as u64,
        0x021C_7374 => {
            super::damage_message(&mut c.env, &c.work, v[1], v[2], &c.bufs[3], v[4]) as u64
        }
        0x021C_7474 => super::multiplier_message(&mut c.env, v[1], v[2], &c.bufs[3], v[4]) as u64,
        0x021C_75EC => super::boost_message(&mut c.env, v[1], v[2]) as u64,
        0x021C_7664 => super::message_319d(&mut c.env, v[1], v[2]) as u64,
        0x021C_76F4 => super::message_31c9(&mut c.env, v[1], v[2]) as u64,
        0x021C_7780 => {
            super::message_31ea(&mut c.env, v[1]);
            0
        }
        0x021C_77B0 => super::message_3201(&mut c.env, v[1], v[2], v[3], v[4], v[5]) as u64,
        0x021C_7820 => {
            super::message_321a(&mut c.env, v[1], v[2]);
            0
        }
        0x021C_785C => {
            super::message_322e(&mut c.env, v[1], v[2], v[3], v[4]);
            0
        }
        0x021C_78A8 => {
            super::message_3244(&mut c.env, v[1], v[2], v[3], v[4]);
            0
        }
        0x021C_78F4 => super::message_325e(&mut c.env, v[1], v[2], v[3]) as u64,
        0x021C_7998 => {
            super::message_3282(&mut c.env, v[1]);
            0
        }
        0x021C_79E0 => super::message_329c(&mut c.env, v[1], v[2]) as u64,
        0x021C_7A54 => super::message_9a(&mut c.env, v[1], v[2]) as u64,
        0x021C_7AA0 => {
            super::message_32d9(&mut c.env, v[1]);
            0
        }
        0x021C_7AD0 => {
            super::message_32e9(&mut c.env, v[1], v[2]);
            0
        }
        0x021C_7B0C => {
            super::message_32fa(&mut c.env, v[1]);
            0
        }
        0x021C_7B3C => {
            super::message_330c(&mut c.env, v[1], v[2], v[3]);
            0
        }
        0x021C_7B7C => {
            super::message_331d(&mut c.env, v[1]);
            0
        }
        0x021C_7BA8 => {
            super::message_9f(&mut c.env, v[1], v[2], v[3], &mut c.bufs[4]);
            0
        }
        0x021C_7BF8 => {
            super::message_3343(&mut c.env);
            0
        }
        0x021C_7C18 => {
            super::clear_12(&mut c.bufs[0]);
            0
        }
        0x021C_7C24 => {
            super::count_statistic(&mut c.env, &mut c.bufs[0], v[2], v[3], v[4] as i32);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/rom-diff/svflow.py core6`).
#[test]
fn svflow_match_rom() {
    run_area_suite(
        "core6",
        include_str!("../../../../../tools/rom-diff/svflow_specs/core6.spec"),
        super::PORTED,
        RECORDED,
        call,
    );
}
