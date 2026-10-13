use crate::server_flow::testkit::{run_area_suite, Case};

// Written by tools/check-svflow.sh only, after the Rust hashes matched the ROM run.
// svflow-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021C8918, "3037b3918013273e"),
    (0x021C8C6C, "dc98ced5cc764699"),
    (0x021C8CBC, "18ae8073b0c3b7d3"),
    (0x021C8D0C, "c33aadb4ce8322f4"),
    (0x021C8D2C, "54c7262509670ac2"),
    (0x021C8DE4, "90d73ae5e4a2a0f0"),
    (0x021C8E58, "b27c9993e989563a"),
    (0x021C8F24, "9366a7b2794dab67"),
    (0x021C8FA4, "71fa12867fcac945"),
    (0x021C9010, "dc475f11808d4ff0"),
    (0x021C9068, "428da1d6d707b158"),
    (0x021C9178, "5ac7bfd47595dd51"),
    (0x021C9250, "8b16e40e98a8b45d"),
    (0x021C9394, "97bd96331a4cab0c"),
    (0x021C9448, "b5466414bba86f4a"),
    (0x021C947C, "4a513df484e55007"),
    (0x021C94C8, "8fc78fab7b087025"),
    (0x021C95D0, "762c4ef0cb67e313"),
    (0x021C962C, "00ebecdc4fc1d5c0"),
    (0x021C96D4, "5dc2f4d6374c4f6c"),
    (0x021C9728, "967343fd7a9e7107"),
    (0x021C9750, "32defbb9b7e27693"),
    (0x021C9778, "94468a59761f4bc5"),
    (0x021C97A4, "f7173c54bb5e068a"),
    (0x021C97D0, "4b93707108e8d29b"),
    (0x021C9828, "37a5db1259038626"),
    (0x021C9898, "d069dce7b25c93e0"),
    (0x021C98C4, "73330d5f5e0cc1dd"),
    (0x021C98E4, "7d11df08445a99bd"),
    (0x021C99A0, "84d81fafa561c198"),
    (0x021C9A24, "c9b7e4d06fa0e1dd"),
    (0x021C9A60, "6b34f38a614269b9"),
    (0x021C9BB8, "583b34fbc21462b2"),
    (0x021C9CC0, "66c90d8c13790b24"),
    (0x021C9DCC, "fad0d6d29ab8bcef"),
    (0x021C9DE8, "f2a694630fde6622"),
    (0x021C9E3C, "9a529bb87a3a9080"),
    (0x021C9ECC, "95e8275c564aa87f"),
    (0x021C9F0C, "fc50fbab0b2c5822"),
    (0x021C9F4C, "8b9ed8a6d864fce2"),
    (0x021C9F6C, "a53bdf281c82cdd2"),
    (0x021CA128, "78d3da356a7e0e57"),
    (0x021CA158, "6bf3b4e0b3ffaac5"),
    (0x021CA1D0, "ff1e852dc1b5e99a"),
    (0x021CA248, "782a7afa765de107"),
    (0x021CA270, "a12a8e976abeddff"),
    (0x021CA2E8, "e5d2fd7c877173a4"),
    (0x021CA318, "a59ccc914350940a"),
    (0x021CA370, "4bcc30fc7585bb27"),
    (0x021CA394, "b1239eb058582499"),
    (0x021CA3AC, "50a45fb900afd81f"),
    (0x021CA3D0, "c694df562358a18a"),
    (0x021CA47C, "2301251266ecb0fd"),
    (0x021CA570, "ea44cf31ee63124b"),
    (0x021CA5CC, "ea072a2cf47fa58c"),
    (0x021CA6C4, "f6407295682e2737"),
    (0x021CA6FC, "05ebe8fd38419ac0"),
    (0x021CA738, "cf0c7e8a9b80ccd4"),
    (0x021CA748, "84a5def056989a4e"),
    (0x021CA7C4, "83d168221ad0d042"),
    (0x021CA7D4, "ccd83cb3350f1b51"),
    (0x021CA7F4, "d56e2725658a6f6c"),
];
// svflow-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
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
