use super::*;
use crate::hooks::Mem;
use crate::testenv::{buf, ofs_arg, ofs_work, run};
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021BB890, "6916cd9a24923505"),
    (0x021BB90C, "7df8070817a035d3"),
    (0x021BB948, "189477bc9fe3702b"),
    (0x021BB954, "9781f9c0fdd2756e"),
    (0x021BB960, "d6742990566991fe"),
    (0x021BB978, "d3b8b7184284d78d"),
    (0x021BB990, "6a0b76517bb7dce0"),
    (0x021BBA1C, "c7a064bdedaf0326"),
    (0x021BBA6C, "dcdb30033c75b10c"),
    (0x021BBA70, "ece8a3209de0fa9d"),
    (0x021BBBD8, "9e53e76363c37cdd"),
    (0x021BBC2C, "af66045ba72a20f7"),
    (0x021BBC58, "fe9ed87652c0940b"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/fieldmap_func.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x021B_B890 => ofs_work(create(e, v[0], v[1], v[2])),
        0x021B_B90C => {
            delete(e, v[0]);
            0
        }
        0x021B_B948 => {
            run_update(e, v[0]);
            0
        }
        0x021B_B954 => {
            run_draw(e, v[0]);
            0
        }
        0x021B_B960 => {
            task_update(e, v[0], v[1]);
            0
        }
        0x021B_B978 => {
            task_draw(e, v[0], v[1]);
            0
        }
        0x021B_B990 => ofs_work(start(e, v[0], v[1], v[2])),
        0x021B_BA1C => {
            slot_delete(e, v[0]);
            0
        }
        0x021B_BA6C => slot_work(&e.mem, v[0]) as u64,
        0x021B_BA70 => {
            ring_add(e, v[0], v[1], v[2], v[3]);
            0
        }
        // `ret=ofs:arg0`: the port reports the offset into the ring buffer
        0x021B_BBD8 => ofs_arg(ring_take(&mut e.mem, v[0]), 0),
        0x021B_BC2C => ofs_arg(ring_entry(&e.mem, v[0], v[1]), 0),
        0x021B_BC58 => ring_is_new(e, v[0], v[1], v[2]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field fieldmap_func`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("fieldmap_func", SPEC, PORTED, RECORDED), &call);
}

#[test]
fn ring_wraps_and_pushes_the_oldest() {
    let r = buf(0);
    let mut m = Mem::new(r, vec![0u8; 0x444]);
    assert_eq!(ring_take(&mut m, r), r);
    assert_eq!((m.r8(r + RING_NEXT), m.r8(r + RING_OLDEST)), (1, 0));
    m.w8(r + RING_NEXT, 29);
    assert_eq!(ring_take(&mut m, r), r + 29 * RING_ENTRY);
    assert_eq!(m.r8(r + RING_NEXT), 0);
    // full ring: next == oldest with entries pushes the oldest on
    m.w32(r + RING_COUNT, 30);
    m.w8(r + RING_OLDEST, 0);
    assert_eq!(ring_take(&mut m, r), r);
    assert_eq!((m.r8(r + RING_NEXT), m.r8(r + RING_OLDEST)), (1, 1));
}
