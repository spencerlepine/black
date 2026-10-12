use super::*;
use black_fndiff::{run_suite, Arg, Case, StubEnv};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0201850C, "327278397c9cbc3a"),
    (0x02018584, "9764044fddc75b57"),
    (0x02019B94, "36368534690a6aa5"),
    (0x02019BA4, "42a4888643dfcd65"),
];
// fndiff-recorded-end

/// The unported callees, routed to the spec's stub lines (same model as the ROM side). The curve cache
/// (`dst`) is the planted global: the ARC stub writes nothing, like the ROM stub.
struct Rom<'a> {
    env: &'a mut StubEnv,
}

impl GrowthHooks for Rom<'_> {
    fn personal_record(&mut self, species: u32, form: u32) -> u32 {
        self.env
            .call(0x0201_9C54, &mut [Arg::V(species), Arg::V(form)])
    }
    fn personal_field(&mut self, rec: u32, field: u32) -> u32 {
        self.env
            .call(0x0201_AE1C, &mut [Arg::V(rec), Arg::V(field)])
    }
    fn personal_param(&mut self, species: u32, form: u32, field: u32) -> u32 {
        self.env.call(
            0x0201_9C90,
            &mut [Arg::V(species), Arg::V(form), Arg::V(field)],
        )
    }
    fn load_growth_curve(&mut self, curve: u32, _dst: &mut [u32; LEVELS]) {
        self.env.call(
            0x0204_8F4C,
            &mut [Arg::P, Arg::V(GROWTH_ARC), Arg::V(curve)],
        );
    }
}

/// The curve cache global as the port's table, and back.
fn table_of(g: &[u8]) -> [u32; LEVELS] {
    let mut t = [0u32; LEVELS];
    for (i, w) in g.chunks(4).enumerate() {
        t[i] = u32::from_le_bytes(w.try_into().unwrap());
    }
    t
}

fn store(g: &mut [u8], t: &[u32; LEVELS]) {
    for (i, w) in t.iter().enumerate() {
        g[i * 4..i * 4 + 4].copy_from_slice(&w.to_le_bytes());
    }
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let mut t = table_of(&c.globals[0]);
    let v = c.vals.clone();
    let mut h = Rom { env: &mut c.env };
    let r = match addr {
        0x0201_9B94 => {
            load_curve(v[0], &mut t, &mut h);
            0
        }
        0x0201_9BA4 => exp_for_level(v[0], v[1], &mut t, &mut h),
        0x0201_850C => level_from_exp(v[0], v[1], v[2], &mut t, &mut h),
        0x0201_8584 => exp_for_species_level(v[0], v[1], v[2], &mut t, &mut h),
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    };
    store(&mut c.globals[0], &t);
    r as u64
}

/// Every ported function against the original (`tools/check-fn.sh black-pokemon growth`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &black_fndiff::Suite::new(
            "growth",
            include_str!("../../../../tools/rom-diff/specs/black-pokemon/growth.spec"),
            PORTED,
            RECORDED,
        ),
        &call,
    );
}

/// Records the calls; the personal field and the table come from the test.
struct Fixed {
    curve: u32,
    table: [u32; LEVELS],
    log: Vec<String>,
}

impl GrowthHooks for Fixed {
    fn personal_record(&mut self, species: u32, form: u32) -> u32 {
        self.log.push(format!("rec {species} {form}"));
        0x1234
    }
    fn personal_field(&mut self, rec: u32, field: u32) -> u32 {
        self.log.push(format!("field {rec:#x} {field:#x}"));
        self.curve
    }
    fn personal_param(&mut self, species: u32, form: u32, field: u32) -> u32 {
        self.log.push(format!("param {species} {form} {field:#x}"));
        self.curve
    }
    fn load_growth_curve(&mut self, curve: u32, dst: &mut [u32; LEVELS]) {
        self.log.push(format!("load {curve}"));
        *dst = self.table;
    }
}

/// A medium-fast-like table: level^3, 0 for level 0.
fn cubic() -> [u32; LEVELS] {
    let mut t = [0u32; LEVELS];
    for (l, v) in t.iter_mut().enumerate() {
        *v = (l * l * l) as u32;
    }
    t
}

fn fixed() -> Fixed {
    Fixed {
        curve: 0x1_0003,
        table: cubic(),
        log: Vec::new(),
    }
}

#[test]
fn level_from_exp_boundaries() {
    let mut h = fixed();
    let mut buf = [0u32; LEVELS];
    // exp 0 is below every table entry from level 1 on... table[1] = 1: level 0
    assert_eq!(level_from_exp(25, 0, 0, &mut buf, &mut h), 0);
    // an entry equal to exp counts as reached
    assert_eq!(level_from_exp(25, 0, 1, &mut buf, &mut h), 1);
    assert_eq!(level_from_exp(25, 0, 7, &mut buf, &mut h), 1);
    assert_eq!(level_from_exp(25, 0, 8, &mut buf, &mut h), 2);
    assert_eq!(level_from_exp(25, 0, 1_000_000 - 1, &mut buf, &mut h), 99);
    assert_eq!(level_from_exp(25, 0, 1_000_000, &mut buf, &mut h), 100);
    assert_eq!(level_from_exp(25, 0, u32::MAX, &mut buf, &mut h), 100);
    // the curve id is truncated to 16 bits and the loaded table stays in the buffer
    assert_eq!(h.log[..3], ["rec 25 0", "field 0x1234 0x17", "load 3"]);
    assert_eq!(buf, cubic());
}

#[test]
fn exp_lookups_load_the_curve_first() {
    let mut h = fixed();
    let mut buf = [0u32; LEVELS];
    assert_eq!(exp_for_level(5, 50, &mut buf, &mut h), 125_000);
    assert_eq!(h.log, ["load 5"]);
    h.log.clear();
    assert_eq!(
        exp_for_species_level(1, 2, 100, &mut buf, &mut h),
        1_000_000
    );
    assert_eq!(h.log, ["param 1 2 0x17", "load 65539"]);
}
