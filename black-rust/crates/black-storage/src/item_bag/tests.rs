use super::*;
use crate::item::RECORD_SIZE;
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021B619C, "abe7e1597c1e4ac8"),
    (0x021BA1D0, "e503eedb6b6704c4"),
];
// fndiff-recorded-end

/// The loader callees, routed to the spec's stub lines (same model as the ROM side).
struct Rom<'a> {
    env: &'a mut StubEnv,
    rec: [u8; RECORD_SIZE],
}

impl ItemHooks for Rom<'_> {
    fn archive_load(&mut self, archive: u32, member: u32, heap: u32, r3: u32) -> u32 {
        self.env.call(
            0x0204_8F60,
            &mut [Arg::V(archive), Arg::V(member), Arg::V(heap), Arg::V(r3)],
        )
    }
    fn block(&mut self, _handle: u32) -> [u8; RECORD_SIZE] {
        self.rec
    }
    fn free(&mut self, handle: u32) {
        self.env.call(0x0203_07B0, &mut [Arg::V(handle)]);
    }
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    match addr {
        0x021B_619C => {
            let rec: [u8; RECORD_SIZE] = c.bufs[3][..RECORD_SIZE].try_into().unwrap();
            let mut h = Rom {
                env: &mut c.env,
                rec,
            };
            sell_price(&mut h, v[0], v[1] as i32, v[2] as u16) as u64
        }
        0x021B_A1D0 => {
            let table: Vec<u16> = c.globals[0]
                .as_chunks::<2>()
                .0
                .iter()
                .map(|b| u16::from_le_bytes(*b))
                .collect();
            is_field_hm_item(v[0], &table) as u64
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-storage item_bag`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "item_bag",
            include_str!("../../../../tools/rom-diff/specs/black-storage/item_bag.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}
