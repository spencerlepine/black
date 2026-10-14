use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0219BDC4, "2949756a75ad8eac"),
    (0x0219BE0C, "6693d0043df50d3b"),
    (0x0219BE18, "ccba95c66bc894f9"),
    (0x0219BE24, "05eaadcd81c58789"),
    (0x0219BE30, "65ad5ca131d2e823"),
    (0x0219BE3C, "17903603fa1c66c4"),
    (0x0219BE60, "d897500eb05dd0d1"),
    (0x0219BE70, "db42e47ce528411c"),
    (0x0219BEB4, "27ee026ebd338317"),
    (0x0219BED0, "5c3bbf346edce080"),
    (0x0219BF00, "4447e96b9d735c2c"),
    (0x0219BF3C, "9cde8f866ae12dcc"),
    (0x0219BF48, "458f7e2be3817fed"),
    (0x0219BF54, "fc5c248f9db15331"),
    (0x0219BF60, "f2189c17855573e3"),
    (0x0219BF7C, "d363a89cd78d44e0"),
    (0x0219BF88, "f460d389d94ceda5"),
    (0x0219BF94, "f75428b977676328"),
    (0x0219BFA0, "40692e218a2e673f"),
    (0x0219BFAC, "0a32ce0fc364803f"),
    (0x0219BFB8, "cb04057d9109bdf0"),
    (0x0219BFC4, "467ca5c062bcdbb7"),
    (0x0219BFD0, "6ef11ac998232825"),
    (0x0219BFDC, "161c971f184f6bb0"),
    (0x0219C000, "96759417704b6180"),
    (0x0219C034, "390c50fb502af3a4"),
    (0x0219C04C, "16ca28d01880ee1f"),
    (0x0219C080, "4bb2f22e99d3d0cb"),
    (0x0219C0AC, "f74f30e0a6119bb9"),
    (0x0219C0D4, "fafca15e1a9642ba"),
    (0x0219C0E8, "f5a8bbc36d74d8f1"),
    (0x0219C0F4, "cdb48b7045acb02a"),
    (0x0219C100, "0674980922f50990"),
    (0x0219C10C, "9335bc7f1ad086cf"),
    (0x0219C118, "0073c4fc996c7d2b"),
    (0x0219C13C, "e4c9b549d616c7fb"),
    (0x0219C150, "711a4128bba933fc"),
    (0x0219C164, "2755a0335bc5d88c"),
    (0x0219C180, "428b6068f81b6475"),
    (0x0219C20C, "a2a79af88c511734"),
    (0x0219C298, "238daa5a2dd7e01d"),
    (0x0219C2BC, "4d39968ab91964e4"),
    (0x0219C2E0, "e6abc39e9f59f85f"),
    (0x0219C314, "6ffb9d8a799ed31d"),
    (0x0219C338, "4e84788288ba62c4"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/mvproc3d.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| {
        let m = v.first().copied().unwrap_or(0);
        let unit = |f: &mut dyn FnMut()| {
            f();
            0
        };
        match addr {
            0x0219_BDC4 => unit(&mut || effect_init(e, m, v[1])),
            0x0219_BE0C => unit(&mut || effect_init_33(e, m)),
            0x0219_BE18 => unit(&mut || effect_init_34(e, m)),
            0x0219_BE24 => unit(&mut || effect_init_35(e, m)),
            0x0219_BE30 => unit(&mut || effect_init_36(e, m)),
            0x0219_BE3C => unit(&mut || effect_move(e, m)),
            0x0219_BE60 => unit(&mut || effect_delete(e, m)),
            0x0219_BE70 => unit(&mut || effect_return(e, m)),
            0x0219_BEB4 => effect_add(e, m, v[1]) as u64,
            0x0219_BED0 => effect_start(e, m, v[1]) as u64,
            0x0219_BF00 => effect_keep(e, m, v[1]) as u64,
            0x0219_BF3C => unit(&mut || set_effect_handle(e, m, v[1])),
            0x0219_BF48 => effect_handle(e, m) as u64,
            0x0219_BF54 => unit(&mut || effect_done(e, m)),
            0x0219_BF60 => unit(&mut || mirror_init(e, m, v[1], v[2])),
            0x0219_BF7C => unit(&mut || mirror_init_37(e, m)),
            0x0219_BF88 => unit(&mut || mirror_init_38(e, m)),
            0x0219_BF94 => unit(&mut || mirror_init_39(e, m)),
            0x0219_BFA0 => unit(&mut || mirror_init_3a(e, m)),
            0x0219_BFAC => unit(&mut || mirror_init_3b(e, m)),
            0x0219_BFB8 => unit(&mut || mirror_init_3c(e, m)),
            0x0219_BFC4 => unit(&mut || mirror_init_3d(e, m)),
            0x0219_BFD0 => unit(&mut || mirror_init_3e(e, m)),
            0x0219_BFDC => unit(&mut || mirror_move(e, m)),
            0x0219_C000 => mirror_face(e, m, v[1]) as u64,
            0x0219_C034 => mirror_face_wait(e, m, v[1]) as u64,
            0x0219_C04C => mirror_copy(e, m, v[1]) as u64,
            0x0219_C080 => mirror_turn(e, m, v[1]) as u64,
            0x0219_C0AC => mirror_turn_wait(e, m, v[1]) as u64,
            0x0219_C0D4 => unit(&mut || wall_init(e, m, v[1], v[2])),
            0x0219_C0E8 => unit(&mut || wall_init_3f(e, m)),
            0x0219_C0F4 => unit(&mut || wall_init_40(e, m)),
            0x0219_C100 => unit(&mut || wall_init_41(e, m)),
            0x0219_C10C => unit(&mut || wall_init_42(e, m)),
            0x0219_C118 => unit(&mut || wall_move(e, m)),
            0x0219_C13C => wall_ready(e, m, v[1]) as u64,
            0x0219_C150 => wall_go(e, m, v[1]) as u64,
            0x0219_C164 => wall_wait(e, m, v[1]) as u64,
            0x0219_C180 => wall_beside(e, m, v[1], v[2], v[3]) as u64,
            0x0219_C20C => wall_ahead(e, m, v[1], v[2], v[3]) as u64,
            0x0219_C298 => wall_beside_here(e, m, v[1], v[2]) as u64,
            0x0219_C2BC => wall_ahead_here(e, m, v[1], v[2]) as u64,
            0x0219_C2E0 => wall_dir(e, m, v[1], v[2]) as u64,
            0x0219_C314 => wall_dir_hit(e, m, v[1], v[2]) as u64,
            0x0219_C338 => wall_step(e, m, v[1], v[2]) as u64,
            _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
        }
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field mvproc3d`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("mvproc3d", SPEC, PORTED, RECORDED), &call);
}

/// The state tables embedded here equal the ROM block.
#[test]
fn rom_tables_match() {
    for (base, t) in [
        (0x021D_537C, &EFFECT_STATES[..]),
        (0x021D_53A0, &MIRROR_STATES[..]),
        (0x021D_5394, &WALL_STATES[..]),
    ] {
        for (i, &a) in t.iter().enumerate() {
            assert_eq!(rom32(base + 4 * i as u32), a);
        }
    }
}
