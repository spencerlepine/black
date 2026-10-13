use super::*;
use black_fndiff::{run_suite, Arg, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[(0x0216E680, "1125c1d4801c0dd1")];
// fndiff-recorded-end

struct H<'a>(&'a mut Case);

impl EventBattleHooks for H<'_> {
    fn call_callback(&mut self, f: u32) {
        self.0.env.call(f & !1, &mut [Arg::V(f)]);
    }
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    match addr {
        0x0216_E680 => run_callback(&mut H(c), v[0]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-field event_battle_head`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "event_battle_head",
            include_str!("../../../../tools/rom-diff/specs/black-field/event_battle_head.spec"),
            PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn null_callback_is_skipped() {
    struct Panics;
    impl EventBattleHooks for Panics {
        fn call_callback(&mut self, _: u32) {
            panic!("called");
        }
    }
    assert_eq!(run_callback(&mut Panics, 0), u32::MAX);
}
