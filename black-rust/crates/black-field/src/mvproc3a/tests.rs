use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0219A098, "bd984c8e3c605e68"),
    (0x0219A0CC, "58807c955155a546"),
    (0x0219A0D8, "90e30746a36acb8b"),
    (0x0219A0E4, "ff9996e0b7220925"),
    (0x0219A0F0, "e57351534c9c7604"),
    (0x0219A0FC, "bc60d43535c543bc"),
    (0x0219A108, "45133c261ec501d1"),
    (0x0219A114, "f88f28c97e253b0e"),
    (0x0219A120, "5c220aa5a0da9b56"),
    (0x0219A12C, "4c78886685304f34"),
    (0x0219A138, "1c5522a72a025576"),
    (0x0219A144, "7f1fccfdc58dfe16"),
    (0x0219A150, "deef783c3f4fe05c"),
    (0x0219A1E8, "95cf090568566ca4"),
    (0x0219A24C, "bdea77bd1c21550c"),
    (0x0219A250, "9ead8fbfd4ffb7fa"),
    (0x0219A260, "db3052ba2f05685a"),
    (0x0219A270, "e41afa16bbe18921"),
    (0x0219A280, "86724b9561299ff9"),
    (0x0219A290, "a968dbc62a836474"),
    (0x0219A2B8, "d214abf7476acce5"),
    (0x0219A3D8, "1d25d1c4e7077c51"),
    (0x0219A4CC, "e6ba3599f0ec7438"),
    (0x0219A528, "837dc0cb022442d4"),
    (0x0219A54C, "cc1e36dc00095413"),
    (0x0219A574, "44b4f35d195a6505"),
    (0x0219A580, "59568d7d603ef155"),
    (0x0219A58C, "7de0d3062de85067"),
    (0x0219A598, "57897c64b0750378"),
    (0x0219A5A4, "6c48381e0b755a8f"),
    (0x0219A5C8, "4bd558c66949bd9d"),
    (0x0219A5D4, "dd4816b0e128e80f"),
    (0x0219A5E0, "20014bc9f7352d91"),
    (0x0219A604, "72120c93aa15a50b"),
    (0x0219A638, "ba77790dcd7fd84a"),
    (0x0219A660, "8c2b460d8302159c"),
    (0x0219A6CC, "2db04e7896d2d2ff"),
    (0x0219A750, "99e7faa97856d5fa"),
    (0x0219A75C, "5144f246e505461d"),
    (0x0219A780, "17a129a64606ab1e"),
    (0x0219A7A8, "b23d0c57dc5fbc61"),
    (0x0219A7C4, "ee4fa05f545e9254"),
    (0x0219A7DC, "d77d920ba306a6a7"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/mvproc3a.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| {
        let m = v.first().copied().unwrap_or(0);
        let unit = |f: &mut dyn FnMut()| {
            f();
            0
        };
        match addr {
            0x0219_A098 => unit(&mut || look_init(e, m, v[1])),
            0x0219_A0CC => unit(&mut || look_init_all(e, m)),
            0x0219_A0D8 => unit(&mut || look_init_06(e, m)),
            0x0219_A0E4 => unit(&mut || look_init_07(e, m)),
            0x0219_A0F0 => unit(&mut || look_init_08(e, m)),
            0x0219_A0FC => unit(&mut || look_init_09(e, m)),
            0x0219_A108 => unit(&mut || look_init_0a(e, m)),
            0x0219_A114 => unit(&mut || look_init_0b(e, m)),
            0x0219_A120 => unit(&mut || look_init_0c(e, m)),
            0x0219_A12C => unit(&mut || look_init_0d(e, m)),
            0x0219_A138 => unit(&mut || look_init_2d(e, m)),
            0x0219_A144 => unit(&mut || look_init_2e(e, m)),
            0x0219_A150 => unit(&mut || look_move(e, m)),
            0x0219_A1E8 => next_dir(v[0], v[1]) as u64,
            0x0219_A24C => unit(&mut || look_delete(e, m)),
            0x0219_A250 => unit(&mut || rand_walk_init_03(e, m)),
            0x0219_A260 => unit(&mut || rand_walk_init_04(e, m)),
            0x0219_A270 => unit(&mut || rand_walk_init_05(e, m)),
            0x0219_A280 => unit(&mut || rand_walk_init_43(e, m)),
            0x0219_A290 => unit(&mut || rand_walk_init(e, m, v[1], v[2], v[3])),
            0x0219_A2B8 => unit(&mut || rand_walk_move(e, m)),
            0x0219_A3D8 => unit(&mut || limit_rect(e, m, v[1])),
            0x0219_A4CC => in_limits(e, m, v[1]) as u64,
            0x0219_A528 => unit(&mut || face_init(e, m, v[1])),
            0x0219_A54C => unit(&mut || face_move(e, m)),
            0x0219_A574 => unit(&mut || face_init_up(e, m)),
            0x0219_A580 => unit(&mut || face_init_down(e, m)),
            0x0219_A58C => unit(&mut || face_init_left(e, m)),
            0x0219_A598 => unit(&mut || face_init_right(e, m)),
            0x0219_A5A4 => unit(&mut || spin_init(e, m, v[1])),
            0x0219_A5C8 => unit(&mut || spin_init_12(e, m)),
            0x0219_A5D4 => unit(&mut || spin_init_13(e, m)),
            0x0219_A5E0 => unit(&mut || spin_move(e, m)),
            0x0219_A604 => spin_face(e, m, v[1]) as u64,
            0x0219_A638 => spin_face_wait(e, m, v[1]) as u64,
            0x0219_A660 => spin_watch(e, m, v[1]) as u64,
            0x0219_A6CC => spin_turn(e, m, v[1]) as u64,
            0x0219_A750 => unit(&mut || spin_init_31(e, m)),
            0x0219_A75C => unit(&mut || spin2_move(e, m)),
            0x0219_A780 => spin2_face(e, m, v[1]) as u64,
            0x0219_A7A8 => spin2_face_wait(e, m, v[1]) as u64,
            0x0219_A7C4 => spin_wait(e, v[1]) as u64,
            0x0219_A7DC => spin2_turn(e, m, v[1]) as u64,
            _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
        }
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field mvproc3a`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("mvproc3a", SPEC, PORTED, RECORDED), &call);
}

/// The ROM state tables and turn orders embedded here equal the build's (skipped without the ROM build unless `REQUIRE_ROM`).
#[test]
fn rom_tables_match() {
    let Some(rd) = crate::testenv::overlay21() else {
        return;
    };
    for (i, &a) in SPIN_STATES.iter().enumerate() {
        assert_eq!(rd(0x021D_4DE8 + 4 * i as u32), a);
    }
    for (i, &a) in SPIN2_STATES.iter().enumerate() {
        assert_eq!(rd(0x021D_4DB8 + 4 * i as u32), a);
    }
    for (base, row) in [
        (0x021D_5024, 0),
        (0x021D_5038, 1),
        (0x021D_504C, 0),
        (0x021D_5060, 1),
    ] {
        for (i, &d) in SPIN_ORDERS[row].iter().enumerate() {
            assert_eq!(rd(base + 4 * i as u32), d);
        }
    }
    for (r, row) in TURN_ORDERS.iter().enumerate() {
        for (i, &h) in row.iter().enumerate() {
            let a = 0x021D_5074 + 0xC * r as u32 + 2 * i as u32;
            assert_eq!((rd(a & !3) >> (8 * (a & 3))) & 0xFFFF, u32::from(h));
        }
    }
}
