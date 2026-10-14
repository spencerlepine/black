use super::*;
use crate::poke_tool::tests::Rom;
use black_fndiff::{run_suite, Arg, Case};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x020179B8, "5c54384e0f31e8bb"),
    (0x02017C50, "19e705e020f758a9"),
    (0x02017CC8, "60b41e352741b283"),
    (0x02017EB4, "b3684973b2c54af8"),
    (0x02017ED8, "842ac823499b5271"),
    (0x02017F08, "ae1d8605b568dbe6"),
    (0x02017F50, "10ef74043daff972"),
    (0x020185F8, "a66a2565c6e7b39e"),
    (0x0201862C, "9d82b4b7c3c72ce4"),
    (0x020186C8, "3ef7ec7bc378a0b1"),
    (0x0201AF6C, "e3c980c06be3a3d2"),
];
// fndiff-recorded-end

/// The callees of other areas, routed to the spec's stub lines (same model as the ROM side).
impl PidHooks for Rom<'_> {
    fn personal_record(&mut self, species: u32, form: u32) -> u32 {
        self.env
            .call(0x0201_9C54, &mut [Arg::V(species), Arg::V(form)])
    }
    fn personal_field(&mut self, rec: u32, field: u32) -> u32 {
        self.env
            .call(0x0201_AE1C, &mut [Arg::V(rec), Arg::V(field)])
    }
    fn ability_count(&mut self, rec: u32) -> u32 {
        self.env.call(0x0201_9C7C, &mut [Arg::V(rec)])
    }
    fn rand(&mut self, max: u32) -> u32 {
        self.env.call(0x0200_56EC, &mut [Arg::V(max)])
    }
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    match addr {
        0x0201_7ED8 => is_shiny(v[0], v[1]) as u64,
        0x0201_AF6C => is_fixed_gender(v[0]) as u64,
        0x0201_7F50 | 0x0201_85F8 | 0x0201_862C | 0x0201_86C8 => {
            let mut h = Rom::new(&mut c.env, 0);
            match addr {
                0x0201_7F50 => gender_from_ratio(v[0], v[1], v[2], &mut h) as u64,
                0x0201_85F8 => gender_threshold(v[0], v[1], v[2], &mut h) as u64,
                0x0201_862C => pid_without_rng(v[0], v[1], v[2], v[3], v[4], v[5], &mut h) as u64,
                _ => random_pid(v[0], v[1], v[2], v[3], v[4], v[5], &mut h) as u64,
            }
        }
        _ => {
            let mut mon = std::mem::take(&mut c.regions[0]);
            let mut h = Rom::new(&mut c.env, v[0]);
            let r = match addr {
                0x0201_7EB4 => is_shiny_mon(&mut mon, &mut h) as u64,
                0x0201_7F08 => gender_of(&mut mon, &mut h) as u64,
                0x0201_7C50 => {
                    update_ability_slot(&mut mon, v[1], &mut h);
                    0
                }
                0x0201_7CC8 => {
                    update_gender(&mut mon, v[1], v[2], &mut h);
                    0
                }
                0x0201_79B8 => set_form(&mut mon, v[1], &mut h) as u64,
                _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
            };
            c.regions[0] = mon;
            r
        }
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-pokemon pid`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &black_fndiff::Suite::new(
            "pid",
            include_str!("../../../../tools/rom-diff/specs/black-pokemon/pid.spec"),
            PORTED,
            RECORDED,
        ),
        &call,
    );
}
/// Scripted callees: a gender ratio per species, a queue of random answers (the argument is recorded).
struct Mock {
    ratio: u32,
    rands: Vec<u32>,
    maxes: Vec<u32>,
}

impl Mock {
    fn new(ratio: u32, rands: &[u32]) -> Self {
        Mock {
            ratio,
            rands: rands.to_vec(),
            maxes: Vec::new(),
        }
    }
}

impl PokeToolHooks for Mock {
    fn gender(&mut self, _: u32, _: u32, _: u32) -> u32 {
        unreachable!()
    }
    fn level_from_exp(&mut self, _: u32, _: u32, _: u32) -> u32 {
        unreachable!()
    }
    fn move_max_pp(&mut self, _: u32, _: u32) -> u32 {
        unreachable!()
    }
    fn personal_param(&mut self, _: u32, _: u32, _: u32) -> u32 {
        unreachable!()
    }
    fn strbuf_from_mon(&mut self, _: u32, _: usize) {
        unreachable!()
    }
    fn strbuf_to_mon(&mut self, _: u32, _: usize, _: u32) {
        unreachable!()
    }
    fn u16str_from_mon(&mut self, _: usize, _: u32, _: u32) {
        unreachable!()
    }
    fn u16str_to_mon(&mut self, _: u32, _: usize, _: u32) {
        unreachable!()
    }
    fn bad_egg_name_strbuf(&mut self, _: u32) {
        unreachable!()
    }
    fn bad_egg_name_u16(&mut self, _: u32, _: u32) {
        unreachable!()
    }
    fn species_name_to_scratch(&mut self, _: u32) {
        unreachable!()
    }
    fn scratch_equals_mon_string(&mut self, _: usize) -> u32 {
        unreachable!()
    }
    fn copy_ext_out(&mut self, _: usize, _: u32) {
        unreachable!()
    }
    fn copy_ext_in(&mut self, _: u32, _: usize) {
        unreachable!()
    }
}

impl PidHooks for Mock {
    fn personal_record(&mut self, species: u32, form: u32) -> u32 {
        species * 0x100 + form
    }
    fn personal_field(&mut self, _rec: u32, field: u32) -> u32 {
        assert_eq!(field, 0x14);
        self.ratio
    }
    fn ability_count(&mut self, _rec: u32) -> u32 {
        unreachable!()
    }
    fn rand(&mut self, max: u32) -> u32 {
        self.maxes.push(max);
        self.rands.remove(0)
    }
}

#[test]
fn fixed_gender_ratios_are_0_0xfe_0xff() {
    for r in [0, 0xFE, 0xFF] {
        assert!(is_fixed_gender(r));
    }
    for r in [1, 0x1F, 0x7F, 0xFD, 0x100, 0x1FE, 0xFFFF_FFFE] {
        assert!(!is_fixed_gender(r));
    }
}

#[test]
fn gender_follows_the_ratio_and_the_pid_low_byte() {
    let g = |ratio, pid| gender_from_ratio(1, 0, pid, &mut Mock::new(ratio, &[]));
    assert_eq!(g(0, 0), 0);
    assert_eq!(g(0, 0xFF), 0);
    assert_eq!(g(0xFE, 0xFF), 1);
    assert_eq!(g(0xFF, 0), 2);
    // 1:1 (ratio 0x7F): female when the low byte is below the ratio, the high bits do not count
    assert_eq!(g(0x7F, 0x7E), 1);
    assert_eq!(g(0x7F, 0x7F), 0);
    assert_eq!(g(0x7F, 0x1234_567E), 1);
    // only the low byte of the field counts
    assert_eq!(g(0x100, 5), 0);
}

#[test]
fn threshold_is_8_for_fixed_ratios() {
    let t = |ratio, flag| gender_threshold(1, 0, flag, &mut Mock::new(ratio, &[]));
    assert_eq!(t(0, 0), 8);
    assert_eq!(t(0xFE, 1), 8);
    assert_eq!(t(0xFF, 0), 8);
    assert_eq!(t(0x1F, 0), 0x1F);
    assert_eq!(t(0x1F, 1), 0x1E);
    assert_eq!(t(1, 1), 0);
}

#[test]
fn pid_without_rng_is_not_shiny_and_carries_the_ratio() {
    // not shiny: high halfword is (xor & 0xFF00) ^ 0xFF00, bit 16 = ability bit
    let mut h = Mock::new(0x7F, &[]);
    let ot = 0x1234_5678; // xor of the halves 0x444C
    let pid = pid_without_rng(ot, 1, 0, 0, 0, 0, &mut h);
    assert_eq!(pid, (0xBB00u32 << 16) | 0x7F);
    assert!(!is_shiny(ot, pid));
    let pid = pid_without_rng(ot, 1, 0, 1, 1, 0, &mut h);
    assert_eq!(pid, ((0xBB00u32 << 16) | 0x7E) ^ 0x10000);
    // fixed ratio: low byte stays 0
    let mut h = Mock::new(0xFF, &[]);
    assert_eq!(pid_without_rng(ot, 1, 0, 1, 0, 0, &mut h), 0xBB00 << 16);
    // shiny: the ids xor to the low byte
    let mut h = Mock::new(0x7F, &[]);
    let pid = pid_without_rng(ot, 1, 0, 0, 0, 1, &mut h);
    assert_eq!(pid & 0xFF, 0x7F);
    assert!(is_shiny(ot, pid));
    assert_eq!(pid & 0x10000, 0);
    let pid = pid_without_rng(ot, 1, 0, 0, 1, 1, &mut h);
    assert!(is_shiny(ot, pid));
    assert_eq!(pid & 0x10000, 0x10000);
}

#[test]
fn random_pid_draws_the_high_bits_then_the_gender_byte() {
    let ot = 0xABCD_1234;
    // genderless: one random word, then the shiny/ability fix-ups
    let mut h = Mock::new(0xFF, &[0x1234_5678]);
    assert_eq!(random_pid(ot, 1, 0, 2, 0, 2, &mut h), 0x1234_5678);
    assert_eq!(h.maxes, [0]);
    // male (kind 0), ratio 0x7F: low byte = 0x7F + rand(0xFE - 0x7F)
    let mut h = Mock::new(0x7F, &[0xAABB_CC00, 5]);
    let pid = random_pid(ot, 1, 0, 0, 0, 2, &mut h);
    assert_eq!(pid & 0xFF, 0x7F + 5);
    assert_eq!(pid >> 8 & 0xFF, 0xCC);
    assert_eq!(h.maxes, [0, 0xFE - 0x7F]);
    assert_eq!(pid & 0x10000, 0);
    // female (kind 1): low byte = rand(threshold - 1) + 1 where the threshold is ratio - 1
    let mut h = Mock::new(0x7F, &[0xAABB_CC00, 5]);
    let pid = random_pid(ot, 1, 0, 1, 1, 2, &mut h);
    assert_eq!(pid & 0xFF, 6);
    assert_eq!(h.maxes, [0, 0x7E]);
    assert_eq!(pid & 0x10000, 0x10000);
    // shiny 1: forced shiny; shiny 0: never shiny; shiny 3: unmodified
    let mut h = Mock::new(0x7F, &[0xAABB_CC00, 5]);
    assert!(is_shiny(ot, random_pid(ot, 1, 0, 0, 1, 1, &mut h)));
    let mut h = Mock::new(0x7F, &[0x0000_1100, 0]);
    let pid = random_pid(0, 1, 0, 0, 0, 0, &mut h);
    assert!(!is_shiny(0, pid));
    let mut h = Mock::new(0x7F, &[0xAABB_CC00, 5]);
    assert_eq!(random_pid(ot, 1, 0, 0, 1, 3, &mut h), 0xAABB_CC00 | 0x84);
}
