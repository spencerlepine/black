use crate::server::Mem;
use crate::server_flow::testkit::{run_area_suite, Case};

// Written by tools/check-svflow.sh only, after the Rust hashes matched the ROM run.
// svflow-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021B6100, "d3f77870467a28df"),
    (0x021B6434, "977301d3eae445f3"),
    (0x021B64CC, "b77f77422aa25d9d"),
    (0x021B6500, "af66c6906fdf7a70"),
    (0x021B66D4, "09763e7c9abcd8ed"),
    (0x021B6804, "956abe7a31829290"),
    (0x021B6818, "9a0b47e048505380"),
    (0x021B6848, "6e17b1f0ead1eb0a"),
    (0x021B68D8, "4cdbdfb5e8482cfd"),
    (0x021B6954, "19b7d083d7d478f2"),
    (0x021B6994, "979a864419942127"),
    (0x021B69FC, "1fdee3280ee3ae13"),
    (0x021B6A3C, "64946c0fb790970e"),
    (0x021B6A70, "2580bbceff6fbcc4"),
    (0x021B6AAC, "8fcd6723997f84f8"),
    (0x021B6C08, "d5c3cc02ad386b98"),
    (0x021B6C5C, "7da03fee458e3753"),
    (0x021B6DD0, "c279dbd50d2c34a0"),
    (0x021B7050, "4156f6a3ac776774"),
    (0x021B71D4, "6cb24b060ffb4b1e"),
    (0x021B734C, "c1e1e166181c5fa4"),
    (0x021B73C4, "f58c8ad626ad5462"),
    (0x021B7488, "9ff7d95649750d59"),
    (0x021B7518, "974ede421c4eb419"),
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
        0x021B_64CC => super::readers_at_end(env),
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
