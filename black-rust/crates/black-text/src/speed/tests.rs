use super::*;
use black_fndiff::{run_suite, Arg, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02013020, "87253d84669eafba"),
    (0x02013050, "9cc6a5af33643a88"),
    (0x02013080, "ccadeba627108544"),
    (0x0201CDF4, "197a2a54d9caa473"),
];
// fndiff-recorded-end

/// `FUN_0200873C(*0x020A7298)` then `FUN_02008374(result)`: the save option, from the stubs.
fn option(c: &mut Case) -> u32 {
    let p = c.env.call(0x0200_873C, &mut [Arg::V(0)]);
    c.env.call(0x0200_8374, &mut [Arg::V(p)])
}

fn call(addr: u32, c: &mut Case) -> u64 {
    match addr {
        0x0201_3080 => speed_value(c.vals[0]) as u32 as u64,
        0x0201_3020 => {
            let o = option(c);
            option_speed(o) as u32 as u64
        }
        0x0201_3050 => {
            let o = option(c);
            option_speed_alt(o) as u32 as u64
        }
        0x0201_CDF4 => {
            let p = pace(c.vals[0] as i32);
            c.bufs[1][0] = p.delay;
            c.bufs[1][1] = p.per_tick;
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-text speed`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "speed",
            include_str!("../../../../tools/rom-diff/specs/black-text/speed.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn pace_and_pacer() {
    assert_eq!(
        pace(3),
        Pace {
            delay: 3,
            per_tick: 1
        }
    );
    assert_eq!(
        pace(-4),
        Pace {
            delay: 0,
            per_tick: 4
        }
    );
    assert_eq!(option_speed(0), 1);
    assert_eq!(option_speed(2), -4);
    assert_eq!(option_speed_alt(0), 6);
    let mut p = Pacer::new(pace(1));
    assert_eq!([p.tick(), p.tick(), p.tick(), p.tick()], [1, 0, 1, 0]);
    let mut p = Pacer::new(pace(-2));
    assert_eq!([p.tick(), p.tick()], [2, 2]);
}
