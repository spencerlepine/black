use super::*;
use black_fndiff::{run_suite, Arg, Case, StubEnv};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0201B2B0, "555d3c264f36910c"),
    (0x0201B70C, "d2269b55576ad815"),
    (0x0201B718, "e1f2709ec3cc2e69"),
    (0x0201B764, "36296c1d52393195"),
    (0x0201B774, "e3342a9e31d2bfe2"),
    (0x0201B7A0, "738f88a63e6dd207"),
    (0x0201B7B4, "09a0755327df14d4"),
    (0x0201B7EC, "3f4936da7d3dfaf6"),
    (0x0201B818, "533df6c5ba43da5c"),
    (0x0201B850, "d9a9d58217bea6e5"),
];
// fndiff-recorded-end

/// Emulator address of pointer argument 7 (the heap block of `FUN_0201B2B0`'s spec).
const BLOCK_ARG: usize = 7;

/// The callees outside the crate, routed to the spec's stub lines (same model as the ROM side). The
/// Pokemon getters run as real code ([`crate::poke_tool`]); only the gender routine is a stub. Every other
/// [`PokeToolHooks`] callee is unreachable for the field ids the evolution check reads.
struct Rom<'a> {
    env: &'a mut StubEnv,
    /// What the (writes-nothing) ARC stub leaves in the heap block: the planted records.
    block: [u8; MEMBER_SIZE],
}

impl EvolutionHooks for Rom<'_> {
    fn hold_effect(&mut self, item: u32, heap: u32) -> u32 {
        self.env
            .call(0x0202_0FE4, &mut [Arg::V(item), Arg::V(1), Arg::V(heap)])
    }
    fn is_night(&mut self, tod: u32) -> u32 {
        self.env.call(0x0201_1410, &mut [Arg::V(tod)])
    }
    fn party_has_species(&mut self, party: u32, species: u32) -> u32 {
        self.env
            .call(0x0201_AC44, &mut [Arg::V(party), Arg::V(species)])
    }
    fn heap_alloc(&mut self, heap: u32, size: u32, align: u32, file: u32, line: u32) -> u32 {
        self.env.call(
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
    fn heap_free(&mut self, block: u32) {
        self.env.call(0x0203_07B0, &mut [Arg::V(block)]);
    }
    fn load_evolution(&mut self, dst: u32, species: u32) -> [u8; MEMBER_SIZE] {
        self.env.call(
            0x0204_8F4C,
            &mut [Arg::V(dst), Arg::V(EVOLUTION_ARC), Arg::V(species)],
        );
        self.block
    }
    fn location_met(&mut self, location: u32) -> u32 {
        self.env.call(0x0201_3E4C, &mut [Arg::V(location)])
    }
    fn event_bonus(&mut self, delta: u32) -> u32 {
        self.env.call(0x0201_1014, &mut [Arg::V(delta)])
    }
    fn item_param(&mut self, item: u32, param: u32, heap: u32) -> u32 {
        self.env.call(
            0x0202_0FE4,
            &mut [Arg::V(item), Arg::V(param), Arg::V(heap)],
        )
    }
    fn random_word(&mut self) -> u32 {
        self.env.call(0x0203_F0A8, &mut [])
    }
    fn alloc_load(&mut self, arc: u32, member: u32) -> u32 {
        self.env
            .call(0x0204_90F4, &mut [Arg::V(arc), Arg::V(member)])
    }
    fn read_member(&mut self, handle: u32, member: u32) -> [u8; MEMBER_SIZE] {
        let mut buf = [0u8; MEMBER_SIZE];
        self.env.call(
            0x0204_932C,
            &mut [Arg::V(handle), Arg::V(member), Arg::Out(&mut buf)],
        );
        buf
    }
    fn party_count(&mut self, party: u32) -> u32 {
        self.env.call(0x0201_AA18, &mut [Arg::V(party)])
    }
    fn party_member(&mut self, party: u32, idx: u32) -> u32 {
        self.env
            .call(0x0201_AB28, &mut [Arg::V(party), Arg::V(idx)])
    }
    fn friendship_adjust_at(&mut self, mon: u32, table: DeltaTable, location: u32, heap: u32) {
        match table {
            DeltaTable::Addr(a) => {
                self.env.call(
                    0x0201_B850,
                    &mut [Arg::V(mon), Arg::V(a), Arg::V(location), Arg::V(heap)],
                );
            }
            DeltaTable::Bytes(b) => {
                self.env.call(
                    0x0201_B850,
                    &mut [Arg::V(mon), Arg::In(&b), Arg::V(location), Arg::V(heap)],
                );
            }
        }
    }
}

impl PokeToolHooks for Rom<'_> {
    fn gender(&mut self, species: u32, form: u32, pid: u32) -> u32 {
        self.env.call(
            0x0201_7F50,
            &mut [Arg::V(species), Arg::V(form), Arg::V(pid)],
        )
    }
    fn level_from_exp(&mut self, _: u32, _: u32, _: u32) -> u32 {
        unreachable!("not read by the evolution check")
    }
    fn move_max_pp(&mut self, _: u32, _: u32) -> u32 {
        unreachable!("not read by the evolution check")
    }
    fn personal_param(&mut self, _: u32, _: u32, _: u32) -> u32 {
        unreachable!("not read by the evolution check")
    }
    fn strbuf_from_mon(&mut self, _: u32, _: usize) {
        unreachable!("not read by the evolution check")
    }
    fn strbuf_to_mon(&mut self, _: u32, _: usize, _: u32) {
        unreachable!("not read by the evolution check")
    }
    fn u16str_from_mon(&mut self, _: usize, _: u32, _: u32) {
        unreachable!("not read by the evolution check")
    }
    fn u16str_to_mon(&mut self, _: u32, _: usize, _: u32) {
        unreachable!("not read by the evolution check")
    }
    fn bad_egg_name_strbuf(&mut self, _: u32) {
        unreachable!("not read by the evolution check")
    }
    fn bad_egg_name_u16(&mut self, _: u32, _: u32) {
        unreachable!("not read by the evolution check")
    }
    fn species_name_to_scratch(&mut self, _: u32) {
        unreachable!("not read by the evolution check")
    }
    fn scratch_equals_mon_string(&mut self, _: usize) -> u32 {
        unreachable!("not read by the evolution check")
    }
    fn copy_ext_out(&mut self, _: usize, _: u32) {
        unreachable!("not read by the evolution check")
    }
    fn copy_ext_in(&mut self, _: u32, _: usize) {
        unreachable!("not read by the evolution check")
    }
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    let mut block = [0u8; MEMBER_SIZE];
    if let Some(b) = c.bufs.get(BLOCK_ARG) {
        let n = b.len().min(MEMBER_SIZE);
        block[..n].copy_from_slice(&b[..n]);
    }
    match addr {
        0x0201_B2B0 => {
            let mut mon = c.region("mon").to_vec();
            let mut partner = c.region("partner").to_vec();
            let a = EvolutionArgs {
                party: v[0],
                mode: v[2],
                param: v[3],
                tod: v[4],
                heap: v[6],
            };
            let mut h = Rom {
                env: &mut c.env,
                block,
            };
            let p = if a.mode == mode::TRADE {
                Some(&mut partner[..])
            } else {
                None
            };
            let e = evolution_check(&mut mon, p, a, &mut h);
            c.region_mut("mon").copy_from_slice(&mon);
            c.region_mut("partner").copy_from_slice(&partner);
            // the out pointer (arg 5) is a store of the winning method
            if v[5] != 0 {
                if let Some(k) = e.kind {
                    c.region_mut("out")[..4].copy_from_slice(&k.to_le_bytes());
                }
            }
            e.target as u64
        }
        0x0201_B764 => {
            let mut h = Rom {
                env: &mut c.env,
                block,
            };
            load_record(v[0], v[1], &mut h);
            0
        }
        0x0201_B774 => {
            let mut mon = c.region("mon").to_vec();
            let mut h = Rom {
                env: &mut c.env,
                block,
            };
            let r = knows_move(&mut mon, v[1], &mut h) as u64;
            c.region_mut("mon").copy_from_slice(&mon);
            r
        }
        0x0201_B70C => {
            let mut h = Rom {
                env: &mut c.env,
                block,
            };
            alloc_load_record(v[0], &mut h) as u64
        }
        0x0201_B718 => {
            let mut h = Rom {
                env: &mut c.env,
                block,
            };
            record_field(v[0], v[1], v[2], v[3], &mut h) as u64
        }
        0x0201_B7A0 => {
            let mut h = Rom {
                env: &mut c.env,
                block,
            };
            friendship_by_index(v[0], v[1], v[2], v[3], &mut h);
            0
        }
        0x0201_B7B4 => {
            let mut h = Rom {
                env: &mut c.env,
                block,
            };
            friendship_by_item(v[0], v[1], v[2], v[3], &mut h);
            0
        }
        0x0201_B7EC => {
            let mut h = Rom {
                env: &mut c.env,
                block,
            };
            friendship_random(v[0], v[1], v[2], &mut h);
            0
        }
        0x0201_B818 => {
            let mut h = Rom {
                env: &mut c.env,
                block,
            };
            friendship_party(v[0], v[1], v[2], &mut h);
            0
        }
        0x0201_B850 => {
            let mut mon = c.region("mon").to_vec();
            let table = c.bufs[1].clone();
            let mut h = Rom {
                env: &mut c.env,
                block,
            };
            friendship_adjust(&mut mon, &table, v[2], v[3], &mut h);
            c.region_mut("mon").copy_from_slice(&mon);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-pokemon evolution`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &black_fndiff::Suite::new(
            "evolution",
            include_str!("../../../../tools/rom-diff/specs/black-pokemon/evolution.spec"),
            PORTED,
            RECORDED,
        ),
        &call,
    );
}

// ---- hand-checked cases -------------------------------------------------------------------------

/// A plain (decrypted) party record with shuffle row 0: blocks A B C D at 0x08 0x28 0x48 0x68.
fn plain_mon(species: u16, item: u16, friendship: u8, level: u8) -> Vec<u8> {
    let mut m = vec![0u8; 0xDC];
    m[4] = 3; // party and box "plain" flags: no cipher, no checksum
    m[0x08..0x0A].copy_from_slice(&species.to_le_bytes());
    m[0x0A..0x0C].copy_from_slice(&item.to_le_bytes());
    m[0x14] = friendship;
    m[0x8C] = level;
    m
}

/// Records: `(method, param, target)` x up to 7.
fn records(rs: &[(u16, u16, u16)]) -> [u8; MEMBER_SIZE] {
    let mut b = [0u8; MEMBER_SIZE];
    for (i, (m, p, t)) in rs.iter().enumerate() {
        b[i * 6..i * 6 + 2].copy_from_slice(&m.to_le_bytes());
        b[i * 6 + 2..i * 6 + 4].copy_from_slice(&p.to_le_bytes());
        b[i * 6 + 4..i * 6 + 6].copy_from_slice(&t.to_le_bytes());
    }
    b
}

/// Fixed answers; the log records the order of the outside calls.
struct Fake {
    rec: [u8; MEMBER_SIZE],
    /// The held-item effect returned for the item `hold_item` (other items have none).
    hold: u32,
    hold_item: u32,
    night: u32,
    has_species: u32,
    gender: u32,
    log: Vec<String>,
}

impl Fake {
    fn new(rs: &[(u16, u16, u16)]) -> Self {
        Fake {
            rec: records(rs),
            hold: 0,
            hold_item: u32::MAX,
            night: 0,
            has_species: 0,
            gender: 0,
            log: Vec::new(),
        }
    }
}

impl EvolutionHooks for Fake {
    fn location_met(&mut self, _: u32) -> u32 {
        unreachable!("not used by the hand-checked evolution_check cases")
    }
    fn event_bonus(&mut self, _: u32) -> u32 {
        unreachable!("not used by the hand-checked evolution_check cases")
    }
    fn item_param(&mut self, _: u32, _: u32, _: u32) -> u32 {
        unreachable!("not used by the hand-checked evolution_check cases")
    }
    fn random_word(&mut self) -> u32 {
        unreachable!("not used by the hand-checked evolution_check cases")
    }
    fn alloc_load(&mut self, _: u32, _: u32) -> u32 {
        unreachable!("not used by the hand-checked evolution_check cases")
    }
    fn read_member(&mut self, _: u32, _: u32) -> [u8; MEMBER_SIZE] {
        unreachable!("not used by the hand-checked evolution_check cases")
    }
    fn party_count(&mut self, _: u32) -> u32 {
        unreachable!("not used by the hand-checked evolution_check cases")
    }
    fn party_member(&mut self, _: u32, _: u32) -> u32 {
        unreachable!("not used by the hand-checked evolution_check cases")
    }
    fn friendship_adjust_at(&mut self, _: u32, _: DeltaTable, _: u32, _: u32) {
        unreachable!("not used by the hand-checked evolution_check cases")
    }
    fn hold_effect(&mut self, item: u32, heap: u32) -> u32 {
        self.log.push(format!("hold {item} {heap}"));
        if item == self.hold_item {
            self.hold
        } else {
            0
        }
    }
    fn is_night(&mut self, tod: u32) -> u32 {
        self.log.push(format!("night {tod}"));
        self.night
    }
    fn party_has_species(&mut self, party: u32, species: u32) -> u32 {
        self.log.push(format!("party {party:#x} {species}"));
        self.has_species
    }
    fn heap_alloc(&mut self, heap: u32, size: u32, align: u32, file: u32, line: u32) -> u32 {
        self.log
            .push(format!("alloc {heap} {size} {align} {file:#x} {line:#x}"));
        0x1000
    }
    fn heap_free(&mut self, block: u32) {
        self.log.push(format!("free {block:#x}"));
    }
    fn load_evolution(&mut self, dst: u32, species: u32) -> [u8; MEMBER_SIZE] {
        self.log.push(format!("load {dst:#x} {species}"));
        self.rec
    }
}

impl PokeToolHooks for Fake {
    fn gender(&mut self, _: u32, _: u32, _: u32) -> u32 {
        self.gender
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

fn args(mode: u32, param: u32) -> EvolutionArgs {
    EvolutionArgs {
        party: 0,
        mode,
        param,
        tod: 2,
        heap: 7,
    }
}

#[test]
fn level_method_needs_the_level() {
    let mut h = Fake::new(&[(method::LEVEL as u16, 16, 5)]);
    let mut m = plain_mon(4, 0, 0, 15);
    assert_eq!(
        evolution_check(&mut m, None, args(mode::LEVEL_UP, 0), &mut h),
        Evolution::default()
    );
    let mut m = plain_mon(4, 0, 0, 16);
    assert_eq!(
        evolution_check(&mut m, None, args(mode::LEVEL_UP, 0), &mut h),
        Evolution {
            target: 5,
            kind: Some(method::LEVEL)
        }
    );
    // the allocation and the load use the species, the block is freed on every path
    assert_eq!(
        h.log[..2],
        [
            "hold 0 7".to_string(),
            "alloc 7 42 0 0x20a7330 0x6a".to_string()
        ]
    );
    assert_eq!(h.log[2], "load 0x1000 4");
    assert_eq!(h.log[3], "free 0x1000");
}

#[test]
fn friendship_boundary_and_time_of_day() {
    for (friend, ok) in [(219u8, false), (220, true)] {
        let mut h = Fake::new(&[(method::FRIENDSHIP as u16, 0, 9)]);
        let mut m = plain_mon(1, 0, friend, 1);
        let e = evolution_check(&mut m, None, args(mode::LEVEL_UP, 0), &mut h);
        assert_eq!(e.target != 0, ok, "friendship {friend}");
    }
    // day variant needs is_night == 0, night variant needs 1
    let mut m = plain_mon(1, 0, 255, 1);
    let mut h = Fake::new(&[(method::FRIENDSHIP_DAY as u16, 0, 9)]);
    assert_eq!(evolution_check(&mut m, None, args(0, 0), &mut h).target, 9);
    h.night = 1;
    assert_eq!(evolution_check(&mut m, None, args(0, 0), &mut h).target, 0);
    let mut h = Fake::new(&[(method::FRIENDSHIP_NIGHT as u16, 0, 9)]);
    h.night = 1;
    assert_eq!(evolution_check(&mut m, None, args(0, 0), &mut h).target, 9);
}

#[test]
fn held_item_blocks_unless_species_64_or_mode_3() {
    let rs = [(method::USE_ITEM as u16, 80, 7)];
    let mut m = plain_mon(10, 0x50, 0, 1);
    let mut h = Fake::new(&rs);
    h.hold = HOLD_EFFECT_NO_EVOLVE;
    h.hold_item = 0x50;
    // blocked before the allocation: only the hold-effect call is made
    assert_eq!(
        evolution_check(&mut m, None, args(mode::ITEM, 80), &mut h).target,
        0
    );
    assert_eq!(h.log, ["hold 80 7"]);
    // mode 3 ignores the block
    assert_eq!(
        evolution_check(&mut m, None, args(mode::ITEM_FORCED, 80), &mut h).target,
        7
    );
    // species 64 is exempt
    let mut m = plain_mon(64, 0x50, 0, 1);
    assert_eq!(
        evolution_check(&mut m, None, args(mode::ITEM, 80), &mut h).target,
        7
    );
}

#[test]
fn item_use_mode_matches_item_and_gender() {
    let rs = [
        (method::USE_ITEM_MALE as u16, 81, 21),
        (method::USE_ITEM_FEMALE as u16, 81, 22),
    ];
    let mut m = plain_mon(1, 0, 0, 1);
    let mut h = Fake::new(&rs);
    h.gender = 1;
    assert_eq!(
        evolution_check(&mut m, None, args(mode::ITEM, 81), &mut h),
        Evolution {
            target: 22,
            kind: Some(method::USE_ITEM_FEMALE)
        }
    );
    assert_eq!(
        evolution_check(&mut m, None, args(mode::ITEM, 80), &mut h).target,
        0
    );
}

#[test]
fn trade_cases() {
    let mut m = plain_mon(1, 0x50, 0, 1);
    let mut partner = plain_mon(2, 0, 0, 1);
    let mut h = Fake::new(&[(method::TRADE_HELD_ITEM as u16, 0x50, 3)]);
    assert_eq!(
        evolution_check(&mut m, Some(&mut partner), args(mode::TRADE, 0), &mut h).target,
        3
    );
    let mut h = Fake::new(&[(method::TRADE as u16, 0, 4)]);
    assert_eq!(
        evolution_check(&mut m, Some(&mut partner), args(mode::TRADE, 0), &mut h).target,
        4
    );
    // the swap pair: this mon is 588 and the partner 616 (field 0xAB = species while not an egg)
    let mut a = plain_mon(0x24C, 0, 0, 1);
    let mut b = plain_mon(0x268, 0, 0, 1);
    let mut h = Fake::new(&[(method::TRADE_FOR_SPECIES as u16, 0, 5)]);
    assert_eq!(
        evolution_check(&mut a, Some(&mut b), args(mode::TRADE, 0), &mut h).target,
        5
    );
    // an evolution-blocking partner item cancels it
    h.hold = HOLD_EFFECT_NO_EVOLVE;
    h.hold_item = 0x51;
    let mut a = plain_mon(0x24C, 0, 0, 1);
    let mut b = plain_mon(0x268, 0x51, 0, 1);
    assert_eq!(
        evolution_check(&mut a, Some(&mut b), args(mode::TRADE, 0), &mut h).target,
        0
    );
}

#[test]
fn location_methods_use_the_param_table() {
    for (m, loc, ok) in [
        (method::LOCATION_A, 196, true),
        (method::LOCATION_A, 155, false),
        (method::LOCATION_B, 155, true),
        (method::LOCATION_C, 203, true),
        (method::LOCATION_C, 195, false),
        (method::LOCATION_D, 197, true),
    ] {
        let mut h = Fake::new(&[(m as u16, 0, 9)]);
        let mut mon = plain_mon(1, 0, 0, 1);
        let e = evolution_check(&mut mon, None, args(mode::LEVEL_UP, loc), &mut h);
        assert_eq!(e.target != 0, ok, "method {m} location {loc}");
    }
}

#[test]
fn spawn_marker_sets_the_kind_without_matching() {
    let mut h = Fake::new(&[(method::SPAWN_MARKER as u16, 0, 9)]);
    let mut m = plain_mon(1, 0, 0, 1);
    assert_eq!(
        evolution_check(&mut m, None, args(0, 0), &mut h),
        Evolution {
            target: 0,
            kind: Some(15)
        }
    );
}

#[test]
fn unknown_methods_and_modes_match_nothing() {
    let mut h = Fake::new(&[(29, 0, 9), (0xFFFF, 0, 9), (0, 0, 9)]);
    let mut m = plain_mon(1, 0, 255, 100);
    assert_eq!(evolution_check(&mut m, None, args(0, 0), &mut h).target, 0);
    let mut h = Fake::new(&[(method::LEVEL as u16, 1, 9)]);
    assert_eq!(evolution_check(&mut m, None, args(4, 0), &mut h).target, 0);
    // the record is still loaded and freed for a mode past 3
    assert_eq!(h.log.len(), 4);
}

#[test]
fn zero_target_does_not_match() {
    let mut h = Fake::new(&[(method::LEVEL as u16, 1, 0), (method::LEVEL as u16, 1, 6)]);
    let mut m = plain_mon(1, 0, 0, 50);
    assert_eq!(evolution_check(&mut m, None, args(0, 0), &mut h).target, 6);
}

#[test]
fn knows_move_checks_the_four_slots() {
    let mut m = plain_mon(1, 0, 0, 1);
    m[0x28 + 6..0x28 + 8].copy_from_slice(&33u16.to_le_bytes());
    let mut h = Fake::new(&[]);
    assert_eq!(knows_move(&mut m, 33, &mut h), 1);
    assert_eq!(knows_move(&mut m, 34, &mut h), 0);
}
