use crate::server::Mem;
use crate::server_flow::testkit::{run_area_suite, Case};

// Written by tools/check-svflow.sh only, after the Rust hashes matched the ROM run.
// svflow-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021B6100, "9c2f1b503ab60665"),
    (0x021B6434, "e92e40c9c9bd8935"),
    (0x021B64CC, "c4475b13425a3eda"),
    (0x021B6500, "de3692994cfc64df"),
    (0x021B66D4, "09763e7c9abcd8ed"),
    (0x021B6804, "956abe7a31829290"),
    (0x021B6818, "9a0b47e048505380"),
    (0x021B6848, "5a91d362c65383d4"),
    (0x021B68D8, "47d2d2feb0896539"),
    (0x021B6954, "094d202c5eeadc1d"),
    (0x021B6994, "979a864419942127"),
    (0x021B69FC, "3af84d440759be26"),
    (0x021B6A3C, "f3e4f14c652173ae"),
    (0x021B6A70, "2580bbceff6fbcc4"),
    (0x021B6AAC, "c700bd4142f7fff3"),
    (0x021B6C08, "92f0fa0be8b73cc8"),
    (0x021B6C5C, "b8f48b09874ce5dc"),
    (0x021B6DD0, "1ffec03204b91a4e"),
    (0x021B7050, "8b8d70cc29128438"),
    (0x021B71D4, "e5f5fd1b8e888860"),
    (0x021B734C, "a3fa642628fe05f0"),
    (0x021B73C4, "18816a9b98e5b330"),
    (0x021B7488, "b35b7424848fa87e"),
    (0x021B7518, "18283ea7bd3acb17"),
];
// svflow-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
    // r3 of the original's entry state (0 past the declared arguments, Z19): the sub-procs hand it on to 6848
    let r3 = c.word(3);
    let Case {
        vals, bufs, env, ..
    } = c;
    let v = |i: usize| vals[i];
    // every `buf` argument is a window of the address space at its fixed emulator address
    let mut mem = Mem::new();
    for (a, b) in vals.iter().zip(bufs.iter_mut()) {
        if !b.is_empty() {
            mem.add(*a, b);
        }
    }
    let mem = &mut mem;
    let r: u32 = match addr {
        0x021B_6100 => super::proc_init(env, mem, v(0), v(1), v(2), v(3)),
        0x021B_6434 => super::proc_main(env, mem, v(3)),
        0x021B_64CC => super::readers_at_end(env, v(0)),
        0x021B_6500 => super::proc_exit(env, mem, v(0), v(1), v(3)),
        0x021B_66D4 => {
            super::select_subproc(mem, v(0), v(1), v(2));
            0
        }
        0x021B_6804 => {
            super::set_exit_subproc(mem, v(0), v(1));
            0
        }
        0x021B_6818 => super::setup_type_is_3(mem, v(0)),
        0x021B_6848 => {
            super::own_client_init(env, mem, v(0), v(1), r3);
            0
        }
        0x021B_68D8 => {
            super::register_parties(env, mem, v(0), v(1));
            0
        }
        0x021B_6954 => {
            super::own_party(env, mem, v(0), v(1));
            0
        }
        0x021B_6994 => super::party_mode(env, mem, v(0), v(1)),
        0x021B_69FC => {
            super::own_parties(env, mem, v(0), v(1));
            0
        }
        0x021B_6A3C => {
            super::record_reader_init(env, mem, v(0), v(1));
            0
        }
        0x021B_6A70 => {
            super::view_create(env, mem, v(0), v(1), v(2));
            0
        }
        0x021B_6AAC => super::subproc_local_single(env, mem, v(1), r3),
        0x021B_6C08 => super::subproc_exit(env, mem, v(1)),
        0x021B_6C5C => super::subproc_local_double(env, mem, v(1), r3),
        0x021B_6DD0 => super::subproc_local_multi(env, mem, v(1)),
        0x021B_7050 => super::subproc_local_triple(env, mem, v(1), r3),
        0x021B_71D4 => super::subproc_local_rotation(env, mem, v(1), r3),
        0x021B_734C => super::subproc_comm_single(env, mem, v(0), v(1)),
        0x021B_73C4 => super::subproc_comm_double(env, mem, v(0), v(1)),
        0x021B_7488 => super::subproc_comm_rule3(env, mem, v(0), v(1)),
        0x021B_7518 => super::subproc_comm_rule2(env, mem, v(0), v(1)),
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    };
    r as u64
}

/// Every ported function against the original (`tools/rom-diff/svflow.py server1`).
#[test]
fn svflow_match_rom() {
    run_area_suite(
        "server1",
        include_str!("../../../../../tools/rom-diff/svflow_specs/server1.spec"),
        super::PORTED,
        RECORDED,
        call,
    );
}
