use super::*;
use crate::testenv::{overlay21, run, Env};
use black_fndiff::{run_suite, Arg, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0209BBB0, "23eb587ae7591d9a"),
    (0x021C6F00, "6ba8cb2eb77eb97a"),
    (0x021C6F30, "d55f302030bb5740"),
    (0x021C701C, "d07f526f049f063c"),
    (0x021C7058, "e176294f06917efc"),
    (0x021C7094, "b90ede0787c2cc17"),
    (0x021C70DC, "98ce28e024cff1a0"),
    (0x021C71A4, "a24910fafa9242f4"),
    (0x021C71D4, "5c19e8608c9af3da"),
    (0x021C7220, "2018a6de4f13c2cd"),
    (0x021C7260, "e919275680540f43"),
    (0x021C72AC, "ce99f6e45871e5d1"),
    (0x021C72F8, "2b7511a580d7185b"),
    (0x021C734C, "7c8a1ed4172e4e98"),
    (0x021C73B4, "5820e25d8a748c0b"),
    (0x021C7418, "aa81471ccf60993c"),
    (0x021C748C, "067ad2a206bfee73"),
    (0x021C7500, "e56fbe8ccd8533db"),
    (0x021C754C, "cd436a1969ffde2c"),
    (0x021C7584, "9e1d3862fd72a8b0"),
    (0x021C7684, "82ca6d617147be95"),
    (0x021C76C4, "69949a9a48babf17"),
    (0x021C7730, "5f4e3b6892af05d5"),
    (0x021C77D8, "61a3c747e5bb9c66"),
    (0x021C77FC, "364c0658603660bf"),
    (0x021C7814, "7d07944cb8fbb236"),
    (0x021C7844, "cb0b17bf2f2d97b1"),
    (0x021C7858, "52f406cf100dc929"),
    (0x021C7878, "29eba143c8245708"),
    (0x021C78DC, "df95efbfe8d822bb"),
    (0x021C7940, "45bc92b594985232"),
    (0x021C7A20, "06c5f724b6643693"),
    (0x021C7A7C, "7dfee046116a6a39"),
    (0x021C7AE4, "1fc4af61a54595fa"),
    (0x021C7B1C, "bd60bc8607ac76d6"),
    (0x021C7C10, "f8ce428e6d266ff6"),
    (0x021C7C74, "94ed552cea1df1df"),
    (0x021C7D28, "8de3f12a4c5550f6"),
    (0x021C7D78, "ab0c917505c393a5"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/field_tasks.spec");

/// The `ret=u64` stubs (the double helpers of [`curve_c`]).
impl FieldEnv64 for Env<'_> {
    fn call64(&mut self, addr: u32, args: &[u32]) -> u64 {
        let mut a: Vec<Arg<'_>> = args.iter().map(|&v| Arg::V(v)).collect();
        self.stubs.call64(addr, &mut a)
    }
}

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x0209_BBB0 => u64::from(fcmpgt(v[0], v[1])),
        0x021C_6F00 => u64::from(zoom_new(e, v[0], v[1], v[2])),
        0x021C_6F30 => u64::from(zoom_proc(e, v[0])),
        0x021C_701C => u64::from(rot_new(e, v[0], v[1], v[2], 0)),
        0x021C_7058 => u64::from(rot_new(e, v[0], v[1], v[2], 1)),
        0x021C_7094 => {
            rot_init(e, v[0]);
            0
        }
        0x021C_70DC => {
            rot_step(e, v[0]);
            0
        }
        0x021C_71A4 => u64::from(rot_proc(e, v[0])),
        0x021C_71D4 => u64::from(spin_new_player(e, v[0], v[1], v[2], 0x021C_7419)),
        0x021C_7220 => u64::from(spin_new_obj(e, v[0], v[1], v[2], v[3])),
        0x021C_7260 => u64::from(spin_new_player(e, v[0], v[1], v[2], 0x021C_748D)),
        0x021C_72AC => u64::from(spin_new_end(e, v[0], v[1], v[2], v[3])),
        0x021C_72F8 => u64::from(curve_a(e, v[0], v[1], v[2])),
        0x021C_734C => u64::from(curve_b(e, v[0], v[1], v[2])),
        0x021C_73B4 => u64::from(curve_c(e, v[0], v[1], v[2])),
        0x021C_7418 => u64::from(spin_proc(e, v[0], 0)),
        0x021C_748C => u64::from(spin_proc(e, v[0], 1)),
        0x021C_7500 => u64::from(spin_proc_end(e, v[0])),
        0x021C_754C => u64::from(move_new(e, v[0], v[1], v[2], 0x021C_7585)),
        0x021C_7584 => u64::from(move_proc(e, v[0])),
        0x021C_7684 => u64::from(shake_new(e, v[0], v[1], v[2], v[3])),
        0x021C_76C4 => {
            shake_calc(e, v[0], v[1], v[2], v[3]);
            0
        }
        0x021C_7730 => u64::from(shake_proc(e, v[0])),
        0x021C_77D8 => u64::from(wait_new(e, v[0], v[1])),
        0x021C_77FC => u64::from(wait_proc(e, v[0])),
        0x021C_7814 => u64::from(call_new(e, v[0], v[1], v[2], v[3], v[4])),
        0x021C_7844 => u64::from(call_proc(e, v[0])),
        0x021C_7858 => u64::from(offset_new_player(e, v[0], v[1], v[2])),
        0x021C_7878 => u64::from(offset_new(e, v[0], v[1], v[2], v[3], 0x021C_7A21)),
        0x021C_78DC => u64::from(offset_new(e, v[0], v[1], v[2], v[3], 0x021C_7A7D)),
        0x021C_7940 => {
            lerp(e, v[0], v[1], v[2], v[3]);
            0
        }
        0x021C_7A20 => u64::from(offset_proc(e, v[0], false)),
        0x021C_7A7C => u64::from(offset_proc(e, v[0], true)),
        0x021C_7AE4 => u64::from(move_new(e, v[0], v[1], v[2], 0x021C_7B1D)),
        0x021C_7B1C => u64::from(campos_proc(e, v[0])),
        0x021C_7C10 => u64::from(cam_new(e, v[0], v[1], v[2], v[3])),
        0x021C_7C74 => u64::from(cam_proc(e, v[0])),
        0x021C_7D28 => u64::from(periodic_new(e, v[0], v[2], v[3])),
        0x021C_7D78 => u64::from(periodic_proc(e, v[0])),
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field field_tasks`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("field_tasks", SPEC, PORTED, RECORDED), &call);
}

/// The three direction tables equal the ROM halfwords (overlay 21).
#[test]
fn dir_tables_match_rom() {
    let Some(rd) = overlay21() else { return };
    for (addr, t) in [
        (0x021D_AAC0, DIR_TABLE_A),
        (0x021D_AAB8, DIR_TABLE_B),
        (0x021D_AAC8, DIR_TABLE_C),
    ] {
        let lo = rd(addr);
        let hi = rd(addr + 4);
        let got = [lo as u16, (lo >> 16) as u16, hi as u16, (hi >> 16) as u16];
        assert_eq!(got, t, "{addr:#010x}");
    }
}
