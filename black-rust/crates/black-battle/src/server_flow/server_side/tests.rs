use crate::server_flow::testkit::{run_area_suite, Case};

// Written by tools/check-svflow.sh only, after the Rust hashes matched the ROM run.
// svflow-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021BAA94, "4f563424c663c73c"),
    (0x021BAACC, "9a5c7f42a63b00d3"),
    (0x021BAB10, "cce6edc607e5a870"),
    (0x021BAB40, "fd11c4ca6cfcb59f"),
    (0x021BAB5C, "6649d56fb7bf6079"),
    (0x021BAB9C, "368385a8bb25c429"),
    (0x021BABDC, "f321d25ebc1341e4"),
    (0x021BABEC, "6c30e7a1811e15d9"),
    (0x021BABF4, "72415c9fe6304595"),
    (0x021BAC04, "88d2542bec5c29a1"),
    (0x021BACAC, "e3c55da17c26b027"),
    (0x021BAE5C, "debf610d000ff115"),
    (0x021BAE60, "d905797c4644d6e4"),
    (0x021BAEF8, "6f80624c02530b6f"),
    (0x021BB040, "b345a0ecfcde8f39"),
    (0x021BB0A0, "ea3370545a1e8b49"),
    (0x021BB0E0, "8c1143a3633cbf47"),
    (0x021BB208, "60c3f5f0104d1fa3"),
    (0x021BB280, "2bd1b4e13d574d9f"),
    (0x021BB320, "18ac16a6a7da610a"),
    (0x021BB368, "f193dbee3144c218"),
    (0x021BB3B0, "ef9f986860f9c060"),
    (0x021BB3F8, "c5674b1a61700c54"),
    (0x021BB43C, "ec0eb04c971063a0"),
    (0x021BB484, "7f475bae23a0ccbf"),
    (0x021BB4E0, "8897e59b15f4a1f3"),
    (0x021BB56C, "9280e7dec0439b85"),
    (0x021BB570, "d9a5066f015502b6"),
    (0x021BB5FC, "86681e70f4eee1f3"),
    (0x021BB628, "67e083ff7cb33b0c"),
    (0x021BB650, "87bc8927f7aa42a5"),
    (0x021BB6A8, "241c7ba068ef0d70"),
    (0x021BB718, "7771390aa88ecbfe"),
    (0x021BB724, "e7554336254231e7"),
    (0x021BB768, "80deb04f1996eeb5"),
    (0x021BB798, "9c1129c24de6e2fc"),
    (0x021BB7D0, "eb9962fe3d3075ed"),
    (0x021BB7FC, "c8975c781fa24ef7"),
    (0x021BB828, "a56568da90eceff5"),
    (0x021BB830, "c4d8ded1cde72ec3"),
    (0x021BB840, "f28e4f63032574c6"),
    (0x021BB848, "79fd2f4634274826"),
    (0x021BB85C, "80ff071c0c4bc6cf"),
    (0x021BB878, "1237bacb18894dd8"),
    (0x021BB890, "b9da10caee5c090e"),
    (0x021BB8A8, "c18a19fa75699b6c"),
    (0x021BB8B0, "841f236526873c45"),
    (0x021BB8BC, "92639671d71a5dbd"),
    (0x021BB8DC, "2e1c046de510fc58"),
    (0x021BB914, "7f4f8d2a84237001"),
    (0x021BB920, "538afda005f7e009"),
    (0x021BB930, "d96a68ca5d9f0ebb"),
    (0x021BB964, "8714deed11f419eb"),
];
// svflow-recorded-end

/// Runs a state function `f(env, s_addr, S, state)`; the state word is buffer 1 of the case.
fn run_state_fn(
    c: &mut Case,
    f: impl FnOnce(
        &mut crate::server_flow::testkit::TestEnv,
        u32,
        &mut [u8],
        &mut super::StateWord,
    ) -> u32,
) -> u64 {
    let (a, b) = c.bufs.split_at_mut(1);
    let mut st = super::StateWord::Separate(&mut b[0]);
    f(&mut c.env, c.vals[0], &mut a[0], &mut st) as u64
}

fn call(addr: u32, c: &mut Case) -> u64 {
    match addr {
        0x021B_B4E0 => {
            super::load_record_stream(&mut c.env, c.vals[0], &mut c.bufs[0], c.vals[1], c.vals[2]);
            0
        }
        0x021B_B6A8 => {
            super::collect_slot_records(&mut c.env, c.vals[0], &mut c.bufs[0]);
            0
        }
        0x021B_B828 => {
            super::slot_clear(&mut c.bufs[0]);
            0
        }
        0x021B_B830 => super::slot_used(&c.bufs[0]) as u64,
        0x021B_B840 => {
            super::slot_set(&mut c.bufs[0], c.vals[1] as u8, c.vals[2], c.vals[3] as u8);
            0
        }
        0x021B_B848 => {
            super::slot_set_data(&mut c.env, &mut c.bufs[0], c.vals[1], c.vals[2] as u8);
            0
        }
        0x021B_B85C => super::slot_checked(&c.bufs[0], c.vals[1]).unwrap_or(0) as u64,
        0x021B_B878 => super::slot_unchecked(&c.bufs[0], c.vals[1] as i32).unwrap_or(0) as u64,
        0x021B_B890 => super::slot_used_idx(&c.bufs[0], c.vals[1]) as u64,
        0x021B_B8A8 => super::byte_0x30(&c.bufs[0], c.vals[1] as i32) as u64,
        0x021B_B8B0 => super::word_at(&c.bufs[0], c.vals[1] as i32, c.vals[2] as i32) as u64,
        0x021B_B964 => super::ptr_0x44(&c.bufs[0]) as u64,
        0x021B_B920 => {
            super::list_clear(&mut c.bufs[0]);
            0
        }
        0x021B_B930 => {
            super::list_add(&mut c.bufs[0], c.vals[1]);
            0
        }
        0x021B_ABDC => {
            let (a, b) = c.bufs.split_at_mut(1);
            super::copy_pair(&a[0], c.vals[0], &mut b[0]);
            0
        }
        0x021B_ABEC => {
            super::set_state(&mut c.bufs[0], c.vals[1]);
            0
        }
        0x021B_AE5C => super::always_true() as u64,
        0x021B_B56C => {
            super::nop();
            0
        }
        0x021B_B724 => {
            super::broadcast(&mut c.env, &mut c.bufs[0], c.vals[1], c.vals[2], c.vals[3]);
            0
        }
        0x021B_B718 => {
            super::broadcast_code(&mut c.env, &mut c.bufs[0], c.vals[1]);
            0
        }
        0x021B_B768 => {
            super::send_one(
                &mut c.env,
                &mut c.bufs[0],
                c.vals[1],
                c.vals[2],
                c.vals[3],
                c.vals[4],
            );
            0
        }
        0x021B_B798 => super::all_ready(&mut c.env, &c.bufs[0]) as u64,
        0x021B_B7D0 => {
            super::each_eeb70(&mut c.env, &c.bufs[0]);
            0
        }
        0x021B_B7FC => {
            super::each_eead0(&mut c.env, &c.bufs[0]);
            0
        }
        0x021B_AA94 => {
            super::setup_slot_handle(&mut c.env, &mut c.bufs[0], c.vals[1], c.vals[2], c.vals[3]);
            0
        }
        0x021B_AACC => {
            super::setup_slot_created(&mut c.env, &mut c.bufs[0], c.vals[2], c.vals[3], c.vals[4]);
            0
        }
        0x021B_AB10 => {
            super::setup_slot_empty(&mut c.env, &mut c.bufs[0], c.vals[1], c.vals[2]);
            0
        }
        0x021B_AB40 => {
            super::teardown(&mut c.env, &mut c.bufs[0]);
            0
        }
        0x021B_AB5C => {
            super::destroy(&mut c.env, c.vals[0], &c.bufs[0]);
            0
        }
        0x021B_ABF4 => {
            super::set_state_acad(&mut c.bufs[0]);
            0
        }
        0x021B_B914 => {
            super::call_9234(&mut c.env, &c.bufs[0]);
            0
        }
        0x021B_B8BC => {
            super::pokemon_set_73(&mut c.env, &c.bufs[0], c.vals[1]);
            0
        }
        0x021B_B8DC => {
            super::pokemon_set_5(&mut c.env, &c.bufs[0], c.vals[1]);
            0
        }
        0x021B_B5FC => super::builder_load(&mut c.env, c.vals[0], &mut c.bufs[0], c.vals[1]) as u64,
        0x021B_B650 => super::builder_finish(
            &mut c.env,
            c.vals[0],
            &mut c.bufs[0],
            c.vals[1],
            c.vals[2],
            super::FinishOut::Ptr(c.vals[3]),
        ) as u64,
        0x021B_B628 => {
            super::builder_flush(&mut c.env, c.vals[0], &mut c.bufs[0], c.vals[1], c.vals[2]) as u64
        }
        0x021B_AC04 => run_state_fn(c, super::state_ac04),
        0x021B_ACAC => run_state_fn(c, super::state_acac),
        0x021B_AE60 => run_state_fn(c, super::state_ae60),
        0x021B_AEF8 => run_state_fn(c, super::state_aef8),
        0x021B_B0E0 => run_state_fn(c, super::state_b0e0),
        0x021B_B208 => run_state_fn(c, super::state_b208),
        0x021B_B368 => run_state_fn(c, super::state_b368),
        0x021B_B320 => run_state_fn(c, |e, _, s, st| super::state_b320(e, s, st)),
        0x021B_B3B0 => run_state_fn(c, super::state_b3b0),
        0x021B_B3F8 => run_state_fn(c, super::state_b3f8),
        0x021B_B43C => run_state_fn(c, super::state_b43c),
        0x021B_B484 => run_state_fn(c, super::state_b484),
        0x021B_AB9C => super::run_state(&mut c.env, c.vals[0], &mut c.bufs[0]) as u64,
        0x021B_B570 => super::compare_header(
            &mut c.env,
            c.vals[0],
            &mut c.bufs[0],
            c.vals[1],
            c.vals[2],
            c.vals[3],
        ) as u64,
        0x021B_B040 => super::party_ready(&mut c.env, &c.bufs[0]) as u64,
        0x021B_B0A0 => super::slot1_value(&mut c.env, c.vals[0], &c.bufs[0]) as u64,
        0x021B_B280 => super::pick_next_state(&mut c.env, &mut c.bufs[0]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/rom-diff/svflow.py server_side`).
#[test]
fn svflow_match_rom() {
    run_area_suite(
        "server_side",
        include_str!("../../../../../tools/rom-diff/svflow_specs/server_side.spec"),
        super::PORTED,
        RECORDED,
        call,
    );
}
