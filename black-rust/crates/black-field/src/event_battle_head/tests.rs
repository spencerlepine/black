use super::*;
use crate::hooks::{CArg, Mem, MemEnv};
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0216E62C, "861046f3b9de187a"),
    (0x0216E680, "1125c1d4801c0dd1"),
    (0x0216E68C, "646946caf0cc7689"),
    (0x0216E7C8, "7a4045915f3db8ee"),
    (0x0216E818, "022573158812b626"),
    (0x0216E8CC, "acb3bbf78dcb2834"),
    (0x0216E910, "504c1f68cf141f9a"),
];
// fndiff-recorded-end

const SPEC: &str =
    include_str!("../../../../tools/rom-diff/specs/black-field/event_battle_head.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x0216_E62C => {
            burmy_form(e, v[0], v[1], v[2]);
            0
        }
        0x0216_E680 => run_callback(e, v[0]) as u64,
        0x0216_E68C => battle_call_proc(e, v[0], v[1], v[2]) as u64,
        0x0216_E7C8 => battle_call_new(e, v[0], v[1], v[2], v[3], v[4]) as u64,
        0x0216_E818 => battle_event_new(e, v[0], v[1], v[2], v[3], v[4], v[5], v[6]) as u64,
        0x0216_E8CC => battle_event_new_a(e, v[0], v[1], v[2]) as u64,
        0x0216_E910 => battle_event_new_b(e, v[0], v[1], v[2]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field event_battle_head`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new("event_battle_head", SPEC, PORTED, RECORDED),
        &call,
    );
}

/// An environment whose callees must not be reached.
struct Panics(Mem);

impl MemEnv for Panics {
    fn mem(&mut self) -> &mut Mem {
        &mut self.0
    }
}

impl FieldEnv for Panics {
    fn call(&mut self, addr: u32, _: &mut [CArg<'_>]) -> u32 {
        panic!("called {addr:#010x}");
    }
}

#[test]
fn null_callback_is_skipped() {
    assert_eq!(run_callback(&mut Panics(Mem::default()), 0), u32::MAX);
}

#[test]
fn unknown_step_does_nothing() {
    // step 4 of the battle call: no callee but the GAMEDATA getter, result 0, step unchanged
    struct Gd(Mem);
    impl MemEnv for Gd {
        fn mem(&mut self) -> &mut Mem {
            &mut self.0
        }
    }
    impl FieldEnv for Gd {
        fn call(&mut self, addr: u32, _: &mut [CArg<'_>]) -> u32 {
            assert_eq!(addr, 0x0201_20F4);
            0
        }
    }
    let mut env = Gd(Mem::new(0x100, vec![0; 0x20]));
    env.w32(0x100, 4);
    assert_eq!(battle_call_proc(&mut env, 0, 0x100, 0x110), 0);
    assert_eq!(env.r32(0x100), 4);
}
