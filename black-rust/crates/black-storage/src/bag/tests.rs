use super::*;
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02007D00, "97076e9e5f15815a"),
    (0x02007D6C, "7116bcaf122ffe5a"),
    (0x02007D94, "d2aec2d4cb797b2f"),
    (0x02007DF8, "8413cd73d4055e22"),
    (0x02007E3C, "a3591990b8cd197b"),
    (0x02007E50, "ed2b71bef1b24d22"),
    (0x02007ECC, "393989e942e09a0a"),
    (0x02007EF8, "ee942a5ee6db4442"),
    (0x02007F1C, "2e5b3147e07c32a5"),
    (0x02007F68, "39d11acfdaa12241"),
    (0x02007F7C, "157f9287a4363f04"),
    (0x02007FA0, "a24edc7e42de6c6a"),
    (0x02007FB8, "9e19f60f522be932"),
    (0x02007FD0, "f8d4311f26687b40"),
    (0x02007FF4, "330a621d6c904b10"),
    (0x02008038, "3a93c585d2bbbe69"),
    (0x02008058, "26b729dbeba376c8"),
    (0x020080A0, "6b4256c7c654d234"),
    (0x020080FC, "de22f409deaa97e6"),
    (0x0200816C, "c07d493f3071a898"),
    (0x020081DC, "eea5d25fccdf7516"),
    (0x02008216, "8b724253592237d3"),
    (0x0200823C, "69e372c9740c5471"),
    (0x02021288, "f5b1f32a63d8dd16"),
    (0x0202130C, "c3af361844b60467"),
    (0x0202E474, "b4062f09453bfced"),
];
// fndiff-recorded-end

/// Fixed emulator address of the bag (pointer argument 0).
const BAG_ADDR: u32 = 0x02E1_0000;

/// `FUN_02020FE4` routed to the spec's stub line (same model as the ROM side).
struct Rom<'a>(&'a mut StubEnv);

impl BagHooks for Rom<'_> {
    fn pocket_of(&mut self, item: u32, heap: Heap) -> u32 {
        self.0.call(
            0x0202_0FE4,
            &mut [Arg::V(item), Arg::V(5), Arg::V(heap as u32)],
        )
    }
}

/// The `bag_param.c` callees, routed to their spec stubs.
struct RomParam<'a>(&'a mut StubEnv);

impl BagParamHooks for RomParam<'_> {
    fn alloc(&mut self, heap: u32, size: u32, align: u32, file: u32, line: u32) -> u32 {
        self.0.call(
            0x0203_0734,
            &mut [
                Arg::V(heap),
                Arg::V(size),
                Arg::V(align),
                Arg::V(file),
                Arg::V(line),
            ],
        )
    }
    fn field_71cc(&mut self, x: u32, sel: u32) -> u32 {
        self.0.call(0x0200_71CC, &mut [Arg::V(x), Arg::V(sel)])
    }
    fn gamedata_2ebc(&mut self, g: u32) -> u32 {
        self.0.call(0x0201_2EBC, &mut [Arg::V(g)])
    }
    fn gamedata_2944(&mut self, g: u32) -> u32 {
        self.0.call(0x0201_2944, &mut [Arg::V(g)])
    }
    fn gamedata_2924(&mut self, g: u32) -> u32 {
        self.0.call(0x0201_2924, &mut [Arg::V(g)])
    }
    fn gamedata_292c(&mut self, g: u32) -> u32 {
        self.0.call(0x0201_292C, &mut [Arg::V(g)])
    }
    fn gamedata_27a4(&mut self, g: u32) -> u32 {
        self.0.call(0x0201_27A4, &mut [Arg::V(g)])
    }
    fn gamedata_2b28(&mut self, x: u32) -> u32 {
        self.0.call(0x0201_2B28, &mut [Arg::V(x)])
    }
}

fn put32(b: &mut [u8], v: u32) {
    b[..4].copy_from_slice(&v.to_le_bytes());
}

/// Writes a pocket lookup to the C out-parameters: base pointer (bag address + offset, or 0 for
/// an invalid pocket) and, for a valid pocket, its slot count.
fn put_pocket(c: &mut Case, loc: Option<(usize, u32)>) {
    match loc {
        Some((ofs, n)) => {
            put32(&mut c.bufs[2], BAG_ADDR + ofs as u32);
            put32(&mut c.bufs[3], n);
        }
        None => put32(&mut c.bufs[2], 0),
    }
}

fn call(addr: u32, c: &mut Case) -> u64 {
    match addr {
        // base = arg4 (fixed address), the slots live in `bufs[4]`
        0x0200_7ECC => find(&c.bufs[4], c.vals[1], c.vals[2], c.vals[3]).unwrap_or(0) as u64,
        // base is 0 or the fixed address of arg2
        0x0200_7F7C => {
            if c.vals[0] == 0 {
                0
            } else {
                count_used(&c.bufs[2], c.vals[1]) as u64
            }
        }
        0x0200_7FD0 => {
            let (a, b) = c.bufs.split_at_mut(1);
            swap(&mut a[0], &mut b[0]);
            0
        }
        0x0200_8038 => {
            let n = c.vals[1];
            clear_empty_counts(&mut c.bufs[0], n);
            0
        }
        0x0200_8216 => slot_ofs(c.vals[1], c.vals[2], c.vals[3]).unwrap_or(0) as u64,
        0x0200_823C => max_stack(c.vals[0]) as u64,
        0x0202_1288 => is_tm(c.vals[0]) as u64,
        0x0202_130C => tm_index(c.vals[0]) as u64,
        // FUN_02007D00(bag, pocket, &base, &count): the pocket id is returned
        0x0200_7D00 => {
            put_pocket(c, pocket(c.vals[1]));
            c.vals[1] as u64
        }
        // FUN_02007D6C(bag, item, &base, &count, heap16): the pocket id is returned
        0x0200_7D6C => {
            let (p, loc) = locate(&mut Rom(&mut c.env), c.vals[1], c.vals[4] as u16);
            put_pocket(c, loc);
            p as u64
        }
        // FUN_02007D94(base = bufs[5], n, item, add, max16): the slot offset, 0 for null
        0x0200_7D94 => slot_for_add(
            &c.bufs[5],
            c.vals[1],
            c.vals[2],
            c.vals[3],
            c.vals[4] as u16,
        )
        .unwrap_or(0) as u64,
        0x0200_7DF8 => add_slot(
            &mut Rom(&mut c.env),
            &c.bufs[0],
            c.vals[1],
            c.vals[2],
            c.vals[3] as u16,
        )
        .unwrap_or(0) as u64,
        0x0200_7E3C => can_add(
            &mut Rom(&mut c.env),
            &c.bufs[0],
            c.vals[1],
            c.vals[2],
            c.vals[3] as u16,
        ) as u64,
        0x0200_7E50 => add_item(
            &mut Rom(&mut c.env),
            &mut c.bufs[0],
            c.vals[1],
            c.vals[2],
            c.vals[3] as u16,
        ) as u64,
        0x0200_7EF8 => held_slot(
            &mut Rom(&mut c.env),
            &c.bufs[0],
            c.vals[1],
            c.vals[2],
            c.vals[3] as u16,
        )
        .unwrap_or(0) as u64,
        0x0200_7F1C => remove_item(
            &mut Rom(&mut c.env),
            &mut c.bufs[0],
            c.vals[1],
            c.vals[2],
            c.vals[3] as u16,
        ) as u64,
        0x0200_7F68 => has_held(
            &mut Rom(&mut c.env),
            &c.bufs[0],
            c.vals[1],
            c.vals[2],
            c.vals[3] as u16,
        ) as u64,
        0x0200_7FA0 => pocket_used(&c.bufs[0], c.vals[1]) as u64,
        0x0200_7FB8 => item_count(
            &mut Rom(&mut c.env),
            &c.bufs[0],
            c.vals[1],
            c.vals[2] as u16,
        ) as u64,
        0x0200_7FF4 => {
            shift_front(&mut c.bufs[0], c.vals[1]);
            0
        }
        0x0200_8058 => {
            compact(&mut c.bufs[0], c.vals[1]);
            0
        }
        0x0200_80A0 => {
            sort_by_id(&mut c.bufs[0], c.vals[1]);
            0
        }
        0x0200_80FC => {
            sort_by_tm(&mut c.bufs[0], c.vals[1]);
            0
        }
        0x0200_816C => {
            compact_all(&mut c.bufs[0]);
            0
        }
        0x0200_81DC => pocket_slot(c.vals[1], c.vals[2], c.vals[3]).unwrap_or(0) as u64,
        // bag_param constructor: block = bufs[4] (fixed address 0x02E5_0000), src = bufs[5]
        0x0202_E474 => {
            let src = (c.vals[1] != 0).then(|| c.bufs[5].clone());
            let ret = new_param(
                &mut RomParam(&mut c.env),
                &mut c.bufs[4],
                0x02E5_0000,
                c.vals[0],
                src.as_deref(),
                c.vals[2],
                c.vals[3],
            );
            ret as u64
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-storage bag`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "bag",
            include_str!("../../../../tools/rom-diff/specs/black-storage/bag.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn tm_tests() {
    assert!(is_tm(0x148) && is_tm(0x1A9) && is_tm(0x26C));
    assert!(!is_tm(0x147) && !is_tm(0x1AA) && !is_tm(0x26D));
    assert_eq!(tm_index(0x148), 0);
    assert_eq!(tm_index(0x26A), 0x5C);
    assert_eq!(max_stack(0x148), 1);
    assert_eq!(max_stack(4), 999);
}

#[test]
fn find_needs_min_count() {
    let slots = [5, 0, 3, 0, 9, 0, 1, 0];
    assert_eq!(find(&slots, 2, 9, 1), Some(4));
    assert_eq!(find(&slots, 2, 9, 2), None);
    assert_eq!(find(&slots, 1, 9, 1), None);
}
