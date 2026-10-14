use super::*;
use crate::item::RECORD_SIZE;
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02017F90, "5309796e719bc856"),
    (0x02019D00, "c5d02c45d2238066"),
    (0x021BBE74, "40646b7910a51d26"),
    (0x021BC17C, "ef054947dd0ff11d"),
    (0x021BC1B4, "c85cc48c467eebc4"),
    (0x021BC228, "78c0594e115eddc8"),
    (0x021BC24C, "132949ccb4c381ed"),
    (0x021BC288, "65a7bd912fd33ec8"),
    (0x021BDD78, "77bda323c4286001"),
    (0x021BE2B4, "65fa7e39dada769c"),
    (0x021BE874, "aec5cd803ba930aa"),
    (0x021BE914, "1b13127432912a55"),
    (0x021BE948, "77e439f37030644d"),
    (0x021BE9BC, "24c7c0ac9f7b0310"),
    (0x021BEA48, "ccd1809ab8b59863"),
];
// fndiff-recorded-end

/// The callees outside the area, routed to the spec's stub lines (same model as the ROM side). `rec` is
/// the record the stubbed archive loader "returns". The Pokemon getters run as real code
/// (`black_pokemon::poke_tool`); the callees of the getters are not reached by the ids the item code reads.
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

impl PokeToolHooks for Rom<'_> {
    fn gender(&mut self, species: u32, form: u32, pid: u32) -> u32 {
        self.env.call(
            0x0201_7F50,
            &mut [Arg::V(species), Arg::V(form), Arg::V(pid)],
        )
    }
    fn level_from_exp(&mut self, species: u32, form: u32, exp: u32) -> u32 {
        self.env.call(
            0x0201_850C,
            &mut [Arg::V(species), Arg::V(form), Arg::V(exp)],
        )
    }
    fn move_max_pp(&mut self, mv: u32, ups: u32) -> u32 {
        self.env.call(0x0201_C158, &mut [Arg::V(mv), Arg::V(ups)])
    }
    fn personal_param(&mut self, species: u32, form: u32, param: u32) -> u32 {
        self.env.call(
            0x0201_9C90,
            &mut [Arg::V(species), Arg::V(form), Arg::V(param)],
        )
    }
    fn strbuf_from_mon(&mut self, _: u32, _: usize) {
        unreachable!("not read by the item routines")
    }
    fn strbuf_to_mon(&mut self, _: u32, _: usize, _: u32) {
        unreachable!("not read by the item routines")
    }
    fn u16str_from_mon(&mut self, _: usize, _: u32, _: u32) {
        unreachable!("not read by the item routines")
    }
    fn u16str_to_mon(&mut self, _: u32, _: usize, _: u32) {
        unreachable!("not read by the item routines")
    }
    fn bad_egg_name_strbuf(&mut self, _: u32) {
        unreachable!("not read by the item routines")
    }
    fn bad_egg_name_u16(&mut self, _: u32, _: u32) {
        unreachable!("not read by the item routines")
    }
    fn species_name_to_scratch(&mut self, _: u32) {
        unreachable!("not read by the item routines")
    }
    fn scratch_equals_mon_string(&mut self, _: usize) -> u32 {
        unreachable!("not read by the item routines")
    }
    fn copy_ext_out(&mut self, _: usize, _: u32) {
        unreachable!("not read by the item routines")
    }
    fn copy_ext_in(&mut self, _: u32, _: usize) {
        unreachable!("not read by the item routines")
    }
}

impl ItemUseHooks for Rom<'_> {
    fn recalc(&mut self, _mon: &mut [u8]) {
        self.env.call(0x0201_85C4, &mut [Arg::P]);
    }
    fn exp_for_species_level(&mut self, species: u32, form: u32, level: u32) -> u32 {
        self.env.call(
            0x0201_8584,
            &mut [Arg::V(species), Arg::V(form), Arg::V(level)],
        )
    }
    fn slot_below_max(&mut self, _mon: &mut [u8], slot: u32) -> u32 {
        self.env.call(0x0201_B920, &mut [Arg::P, Arg::V(slot)])
    }
    fn raise_pp(&mut self, _mon: &mut [u8], slot: u32, amount: u32) -> u32 {
        self.env
            .call(0x0201_B970, &mut [Arg::P, Arg::V(slot), Arg::V(amount)])
    }
    fn friendship_by_item(&mut self, _mon: &mut [u8], item: u32, location: u32, heap: u32) {
        self.env.call(
            0x0201_B7B4,
            &mut [Arg::P, Arg::V(item), Arg::V(location), Arg::V(heap)],
        );
    }
    fn set_form(&mut self, _mon: &mut [u8], form: u32) {
        self.env.call(0x0201_7988, &mut [Arg::P, Arg::V(form)]);
    }
}

fn record(c: &Case, arg: usize) -> [u8; RECORD_SIZE] {
    c.bufs[arg][..RECORD_SIZE].try_into().unwrap()
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    match addr {
        0x021B_BE74 | 0x021B_C228 | 0x021B_C17C | 0x021B_C1B4 => {
            let rec = record(c, 3);
            let heap = match c.bufs[0].as_slice() {
                [a, b, ..] => u16::from_le_bytes([*a, *b]) as u32,
                _ => 0,
            };
            let mut h = Rom {
                env: &mut c.env,
                rec,
            };
            match addr {
                0x021B_BE74 => use_kind(&mut h, v[0]) as u64,
                0x021B_C228 => move_prompt_id(&mut h, v[1]) as u64,
                0x021B_C17C => needs_move_slot(&mut h, heap, v[1]) as u64,
                _ => has_param_1b(&mut h, heap, v[1]) as u64,
            }
        }
        0x021B_DD78 => {
            let rec = record(c, 4);
            let mut mon = c.region("mon").to_vec();
            let mut h = Rom {
                env: &mut c.env,
                rec,
            };
            let r = can_use(&mut h, &mut mon, v[1], v[2], v[3]);
            c.region_mut("mon").copy_from_slice(&mon);
            r as u64
        }
        0x021B_E2B4 => {
            let rec = record(c, 5);
            let mut mon = c.region("mon").to_vec();
            let mut h = Rom {
                env: &mut c.env,
                rec,
            };
            let r = apply_item(&mut h, &mut mon, v[1], v[2], v[3], v[4]);
            c.region_mut("mon").copy_from_slice(&mon);
            r as u64
        }
        0x021B_E874 | 0x021B_E914 | 0x021B_EA48 | 0x021B_E9BC => {
            let mut mon = c.region("mon").to_vec();
            let mut h = Rom {
                env: &mut c.env,
                rec: [0; RECORD_SIZE],
            };
            let r = match addr {
                0x021B_E874 => add_pp_ups(&mut h, &mut mon, v[1], v[2]) as u64,
                0x021B_E914 => {
                    heal(&mut h, &mut mon, v[1], v[2], v[3]);
                    0
                }
                0x021B_EA48 => {
                    apply_friendship(&mut h, &mut mon, v[1], v[2] as i32, v[3], v[4], v[5]) as u64
                }
                _ => {
                    let rec = c.bufs[1].clone();
                    friendship_raises(&mut h, &mut mon, &rec) as u64
                }
            };
            c.region_mut("mon").copy_from_slice(&mon);
            r
        }
        0x021B_C24C | 0x021B_C288 | 0x0201_7F90 => {
            let mut mon = c.region("mon").to_vec();
            let mut h = Rom {
                env: &mut c.env,
                rec: [0; RECORD_SIZE],
            };
            let r = match addr {
                0x021B_C24C => {
                    apply_plate_form(&mut h, &mut mon, v[2]);
                    0
                }
                0x021B_C288 => {
                    apply_drive_form(&mut h, &mut mon, v[2]);
                    0
                }
                _ => status_of(&mut h, &mut mon) as u64,
            };
            c.region_mut("mon").copy_from_slice(&mon);
            r
        }
        0x0201_9D00 => drive_form(v[0]) as u64,
        0x021B_E948 => ev_after(v[0] as i32, v[1] as i32, v[2] as i32, v[3] as i32) as u32 as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-storage item_use`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "item_use",
            include_str!("../../../../tools/rom-diff/specs/black-storage/item_use.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn ev_amounts() {
    // blocked: nothing to take, already 255, 100 without the flag, total at 510
    assert_eq!(ev_after(0, 0, -10, 0), -1);
    assert_eq!(ev_after(255, 0, 10, 1), -1);
    assert_eq!(ev_after(100, 0, 10, 0), -1);
    assert_eq!(ev_after(50, 460, 10, 1), -1);
    // capped at 100 without the flag, at the total of 510 with it, never below 0
    assert_eq!(ev_after(95, 0, 10, 0), 100);
    assert_eq!(ev_after(95, 410, 10, 1), 100);
    assert_eq!(ev_after(60, 440, 20, 1), 70);
    assert_eq!(ev_after(5, 0, -10, 0), 0);
}

#[test]
fn drive_forms() {
    assert_eq!(
        [0x73, 0x74, 0x75, 0x76, 0x77, 0x78].map(drive_form),
        [0, 1, 2, 3, 4, 0]
    );
}
