use crate::server_flow::testkit::{run_area_suite, Case};

// Written by tools/check-svflow.sh only, after the Rust hashes matched the ROM run.
// svflow-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021C8918, "efccf1621de9b9af"),
    (0x021C8C6C, "3e88a1bd4453d2a7"),
    (0x021C8CBC, "0b7d620ec5bc8f30"),
    (0x021C8D0C, "54b983b3a0f8b5db"),
    (0x021C8D2C, "0be31746ccdc6d3d"),
    (0x021C8DE4, "f5c9bfcd3d0e60bc"),
    (0x021C8E58, "753d4cbef8b9f306"),
    (0x021C8F24, "402cc31d053cc778"),
    (0x021C8FA4, "ed913e59076c8416"),
    (0x021C9010, "5eaa718f4d292ae6"),
    (0x021C9068, "c4bad9627651c9e3"),
    (0x021C9178, "090cdfcb99ad283e"),
    (0x021C9250, "b115e81c3603242a"),
    (0x021C9394, "97bd96331a4cab0c"),
    (0x021C9448, "b5466414bba86f4a"),
    (0x021C947C, "4a513df484e55007"),
    (0x021C94C8, "51653c4f808b82bb"),
    (0x021C95D0, "8ffa5eae28bb59ec"),
    (0x021C962C, "00ebecdc4fc1d5c0"),
    (0x021C96D4, "e43ae02367e8a7ea"),
    (0x021C9728, "967343fd7a9e7107"),
    (0x021C9750, "32defbb9b7e27693"),
    (0x021C9778, "2757e264fb194827"),
    (0x021C97A4, "d87ddb80cb007d0f"),
    (0x021C97D0, "18215b53ecf2aaaa"),
    (0x021C9828, "7da9bedcf0849f15"),
    (0x021C9898, "194ebe4feb822512"),
    (0x021C98C4, "f7312ac846107f14"),
    (0x021C98E4, "12ef683902e57d76"),
    (0x021C99A0, "4144339c5a91325a"),
    (0x021C9A24, "c9b7e4d06fa0e1dd"),
    (0x021C9A60, "4d21bcd1ae2a6a91"),
    (0x021C9BB8, "ff71435dd7780961"),
    (0x021C9CC0, "9f043960ab0a0f44"),
    (0x021C9DCC, "e8ebb85618fa7886"),
    (0x021C9DE8, "35723960311d08f7"),
    (0x021C9E3C, "9c312d658b260d61"),
    (0x021C9ECC, "a0d217cd3460248b"),
    (0x021C9F0C, "fc50fbab0b2c5822"),
    (0x021C9F4C, "2e710c9ce6cd05e1"),
    (0x021C9F6C, "2f3b23b2ba878ced"),
    (0x021CA128, "9aee63af3baa6874"),
    (0x021CA158, "57df9ad4917e612e"),
    (0x021CA1D0, "83e56155aa7572b8"),
    (0x021CA248, "8b706ec00e883a81"),
    (0x021CA270, "b42e110da2c8cfd5"),
    (0x021CA2E8, "29d9d1fe39cff018"),
    (0x021CA318, "5e57e49acb3a3efe"),
    (0x021CA370, "0c7047bbefd0adaa"),
    (0x021CA394, "c28d93807bbc6dfb"),
    (0x021CA3AC, "69671a5b8accbdc4"),
    (0x021CA3D0, "f5edd1d72f137c9b"),
    (0x021CA47C, "2e57819e0a0aa0f0"),
    (0x021CA570, "7b6d4fdbfe32bc76"),
    (0x021CA5CC, "d1d938014cc21704"),
    (0x021CA6C4, "f5f7baf8b7ae7e00"),
    (0x021CA6FC, "c0297fdfa0f93fa5"),
    (0x021CA738, "cf0c7e8a9b80ccd4"),
    (0x021CA748, "caf0cee853dd856d"),
    (0x021CA7C4, "83d168221ad0d042"),
    (0x021CA7D4, "ccd83cb3350f1b51"),
    (0x021CA7F4, "a4a808cdf543d0b6"),
];
// svflow-recorded-end

pub(crate) fn call<E: crate::server_flow::env::FlowEnv>(addr: u32, c: &mut Case<E>) -> u64 {
    use super::*;
    let v = c.vals.clone();
    let env = &mut c.env;
    let w = &mut c.work;
    // queue procs: region Q = regions[0], put_cmd scratch = globals[0]
    macro_rules! q {
        ($f:ident) => {
            $f(env, w, &c.bufs[1], &mut c.regions[0], &mut c.globals[0]) as u64
        };
    }
    match addr {
        0x021C_8918 => {
            let (q, d) = c.regions.split_at_mut(1);
            execute(
                env,
                w,
                &c.bufs[1],
                v[1],
                &mut q[0],
                &mut c.globals[0],
                &mut d[0],
            );
            0
        }
        0x021C_9DE8 => proc_00(env, w, &c.bufs[1]) as u64,
        0x021C_8C6C => proc_01(env, w, &c.bufs[1]) as u64,
        0x021C_8CBC => proc_02(env, w, &c.bufs[1]) as u64,
        0x021C_8D0C => proc_03(env, w, &c.bufs[1]) as u64,
        0x021C_96D4 => proc_04(env, w, &c.bufs[1]) as u64,
        0x021C_8D2C => proc_05(env, w, &c.bufs[1], v[2], &mut c.regions[0]) as u64,
        0x021C_8DE4 => proc_06(env, w, &c.bufs[1]) as u64,
        0x021C_8E58 => proc_07(env, w, &c.bufs[1]) as u64,
        0x021C_8F24 => proc_08(env, w, &c.bufs[1]) as u64,
        0x021C_8FA4 => proc_09(env, w, &c.bufs[1]) as u64,
        0x021C_9010 => proc_0a(env, w, &c.bufs[1]) as u64,
        0x021C_9068 => proc_0b(env, w, &c.bufs[1], v[2]) as u64,
        0x021C_9178 => proc_0c(env, w, &c.bufs[1], v[1]) as u64,
        0x021C_9250 => proc_0e(env, w, &c.bufs[1], v[2]) as u64,
        0x021C_9394 => q!(proc_0f),
        0x021C_947C => q!(proc_10),
        0x021C_94C8 => q!(proc_11),
        0x021C_9448 => q!(proc_12),
        0x021C_95D0 => proc_13(env, w, &c.bufs[1]) as u64,
        0x021C_962C => q!(proc_14),
        0x021C_9728 => proc_15(env, w, &c.bufs[1]) as u64,
        0x021C_9750 => proc_16(env, w, &c.bufs[1]) as u64,
        0x021C_9778 => proc_17(env, w, &c.bufs[1]) as u64,
        0x021C_97A4 => proc_18(env, w, &c.bufs[1]) as u64,
        0x021C_97D0 => proc_19(env, &c.bufs[1]) as u64,
        0x021C_9828 => proc_1a(env, &c.bufs[1]) as u64,
        0x021C_9898 => proc_1b(env, &c.bufs[1]) as u64,
        0x021C_98C4 => proc_1c(env, &c.bufs[1]) as u64,
        0x021C_98E4 => proc_1d(env, w, &c.bufs[1]) as u64,
        0x021C_9A24 => proc_1e(env, &c.bufs[1]) as u64,
        0x021C_9A60 => q!(proc_1f),
        0x021C_9BB8 => q!(proc_20),
        0x021C_9DCC => proc_21(env, w, &c.bufs[1]) as u64,
        0x021C_9E3C => q!(proc_22),
        0x021C_9ECC => proc_23(env, w, &c.bufs[1]) as u64,
        0x021C_9CC0 => proc_24(env, w, &c.bufs[1]) as u64,
        0x021C_9F0C => q!(proc_25),
        0x021C_9F4C => proc_26(env, w, &c.bufs[1]) as u64,
        0x021C_9F6C => proc_27(env, w, &c.bufs[1], &mut c.regions[0]) as u64,
        0x021C_A128 => proc_28(env, w, &c.bufs[1]) as u64,
        0x021C_A158 => proc_29(env, w, &c.bufs[1]) as u64,
        0x021C_A1D0 => q!(proc_2a),
        0x021C_A248 => proc_2b(env, w, &c.bufs[1]) as u64,
        0x021C_A270 => q!(proc_2c),
        0x021C_A2E8 => q!(proc_2d),
        0x021C_A318 => proc_2e(env, w, &c.bufs[1]) as u64,
        0x021C_A370 => proc_2f(env, &c.bufs[1]) as u64,
        0x021C_A394 => proc_30(env, &c.bufs[1]) as u64,
        0x021C_A3AC => proc_31(env, &c.bufs[1]) as u64,
        0x021C_A3D0 => proc_32(env, w, &c.bufs[1]) as u64,
        0x021C_A47C => q!(proc_33),
        0x021C_A570 => q!(proc_34),
        0x021C_A5CC => proc_35(env, w, &c.bufs[1], &mut c.regions[0]) as u64,
        0x021C_A6C4 => proc_36(env, w, &c.bufs[1]) as u64,
        0x021C_A6FC => proc_37(env, &c.bufs[1], &mut c.regions[0], &mut c.globals[0]) as u64,
        0x021C_A738 => proc_38(env, &mut c.regions[0], &mut c.globals[0]) as u64,
        0x021C_A748 => q!(proc_39),
        0x021C_A7C4 => proc_3a(&c.bufs[1], &mut c.regions[0]) as u64,
        0x021C_A7D4 => proc_3b(&mut c.regions[0]) as u64,
        0x021C_99A0 => message(env, &c.bufs[1]) as u64,
        0x021C_A7F4 => rule_dispatch(env, w, v[1], v[2], &c.bufs[3], v[3], v[4]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/rom-diff/svflow.py handex`).
#[test]
fn svflow_match_rom() {
    run_area_suite(
        "handex",
        include_str!("../../../../../tools/rom-diff/svflow_specs/handex.spec"),
        super::PORTED,
        RECORDED,
        call,
    );
}
