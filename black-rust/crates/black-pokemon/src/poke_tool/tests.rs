use super::*;
use black_fndiff::{run_suite, Arg, Case, StubEnv};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02017DBC, "96b3bd57dca63081"),
    (0x02017DE4, "045b053496f05a59"),
    (0x02017E1C, "4862d7bd9e447e79"),
    (0x02017E40, "54ce36810e8ec237"),
    (0x02017E64, "9718f4c47845faff"),
    (0x02017E88, "93f67ef8c8437db0"),
    (0x02018D10, "dca0230754622b5a"),
    (0x02018D34, "3b7d9529dc863c6a"),
    (0x02018D6C, "3e719f38babfe4ec"),
    (0x02018D8C, "88f0773252508266"),
    (0x02018DB4, "10c304dd7839ad77"),
    (0x02018E34, "b09b1190bdb2d3da"),
    (0x02019308, "09417e2ad8179f47"),
    (0x02019388, "388e7fd6d05bb879"),
    (0x02019A50, "5f0bb333599315f5"),
    (0x02019ABC, "c0860ab7260df24a"),
    (0x02019BBC, "9ff38980fcbde3de"),
    (0x02019C38, "cf396eaff34eb050"),
    (0x02019CE0, "f3a33438223dc2ca"),
];
// fndiff-recorded-end

/// The unported callees, routed to the spec's stub lines (same model as the ROM side).
pub(crate) struct Rom<'a> {
    pub(crate) env: &'a mut StubEnv,
    /// Fixed emulator address of the record (`vals[0]`); `mon + mon_ofs` is what the original passes.
    pub(crate) mon: u32,
    /// Bytes of a heap block that a hook of the create/personal tests reads (the learnset block).
    pub(crate) aux: Vec<u8>,
    /// Open personal-data archive handle: `Some` makes `PersonalHooks::load_record` run the port of
    /// `FUN_0201ADA4` (the callee is real code in the `personal` stat-calculation spec), `None` stubs it.
    pub(crate) handle: Option<u32>,
}

impl<'a> Rom<'a> {
    pub(crate) fn new(env: &'a mut StubEnv, mon: u32) -> Self {
        Rom {
            env,
            mon,
            aux: Vec::new(),
            handle: None,
        }
    }
}

const SCRATCH: u32 = 0x0214_62F0;

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
    fn move_max_pp(&mut self, move_id: u32, pp_ups: u32) -> u32 {
        self.env
            .call(0x0201_C158, &mut [Arg::V(move_id), Arg::V(pp_ups)])
    }
    fn personal_param(&mut self, species: u32, form: u32, param: u32) -> u32 {
        self.env.call(
            0x0201_9C90,
            &mut [Arg::V(species), Arg::V(form), Arg::V(param)],
        )
    }
    fn strbuf_from_mon(&mut self, strbuf: u32, mon_ofs: usize) {
        let p = self.mon + mon_ofs as u32;
        self.env.call(0x0204_5924, &mut [Arg::V(strbuf), Arg::V(p)]);
    }
    fn strbuf_to_mon(&mut self, strbuf: u32, mon_ofs: usize, len: u32) {
        let p = self.mon + mon_ofs as u32;
        self.env
            .call(0x0204_5A38, &mut [Arg::V(strbuf), Arg::V(p), Arg::V(len)]);
    }
    fn u16str_from_mon(&mut self, mon_ofs: usize, dst: u32, len: u32) {
        let p = self.mon + mon_ofs as u32;
        self.env
            .call(0x0201_F41C, &mut [Arg::V(p), Arg::V(dst), Arg::V(len)]);
    }
    fn u16str_to_mon(&mut self, src: u32, mon_ofs: usize, len: u32) {
        let p = self.mon + mon_ofs as u32;
        self.env
            .call(0x0201_F41C, &mut [Arg::V(src), Arg::V(p), Arg::V(len)]);
    }
    fn bad_egg_name_strbuf(&mut self, strbuf: u32) {
        self.env
            .call(0x0204_5CAC, &mut [Arg::P, Arg::V(0x28B), Arg::V(strbuf)]);
    }
    fn bad_egg_name_u16(&mut self, dst: u32, len: u32) {
        self.env.call(
            0x0204_5F5C,
            &mut [Arg::P, Arg::V(0x28B), Arg::V(dst), Arg::V(len)],
        );
    }
    fn species_name_to_scratch(&mut self, species: u32) {
        self.env.call(
            0x0204_5F5C,
            &mut [Arg::P, Arg::V(species), Arg::V(SCRATCH), Arg::V(0x20)],
        );
    }
    fn scratch_equals_mon_string(&mut self, mon_ofs: usize) -> u32 {
        let p = self.mon + mon_ofs as u32;
        self.env
            .call(0x0201_F448, &mut [Arg::V(SCRATCH), Arg::V(p)])
    }
    fn copy_ext_out(&mut self, mon_ofs: usize, dst: u32) {
        let p = self.mon + mon_ofs as u32;
        self.env.call(0x0200_8F3C, &mut [Arg::V(p), Arg::V(dst)]);
    }
    fn copy_ext_in(&mut self, src: u32, mon_ofs: usize) {
        let p = self.mon + mon_ofs as u32;
        self.env.call(0x0200_8F3C, &mut [Arg::V(src), Arg::V(p)]);
    }
}

fn call(addr: u32, c: &mut Case) -> u64 {
    // functions with their own region layout first
    match addr {
        0x0201_9A50 => {
            let (size, seed) = (c.vals[1], c.vals[2]);
            crypt(c.region_mut("data"), size, seed);
            return 0;
        }
        0x0201_9C38 => return block_offset(c.vals[1], c.vals[2]) as u64,
        0x0201_9BBC => return plate_type(c.vals[0]) as u64,
        0x0201_9CE0 => {
            let src = c.region("src").to_vec();
            copy_pokemon(&src, c.region_mut("dst"));
            return 0;
        }
        _ => {}
    }
    let mon_addr = c.vals[0];
    let mut mon = std::mem::take(&mut c.regions[0]);
    let (id, val) = (c.vals.get(1).copied(), c.vals.get(2).copied());
    let r = {
        let mut h = Rom::new(&mut c.env, mon_addr);
        let (id, val) = (id.unwrap_or(0), val.unwrap_or(0));
        match addr {
            0x0201_7DBC => decrypt_on_access(&mut mon) as u64,
            0x0201_7DE4 => reencrypt(&mut mon, id) as u64,
            0x0201_8D34 => box_enter(&mut mon) as u64,
            0x0201_8D8C => {
                box_exit(&mut mon);
                0
            }
            0x0201_8D10 => party_enter(&mut mon) as u64,
            0x0201_8D6C => {
                party_exit(&mut mon);
                0
            }
            0x0201_9ABC => is_nicknamed(&mon, &mut h) as u64,
            0x0201_8E34 => box_get(&mut mon, id, val, &mut h) as u64,
            0x0201_9388 => {
                box_put(&mut mon, id, val, &mut h);
                0
            }
            0x0201_8DB4 => party_get(&mut mon, id, val, &mut h) as u64,
            0x0201_9308 => {
                party_put(&mut mon, id, val, &mut h);
                0
            }
            0x0201_7E88 => box_get_value(&mut mon, id, val, &mut h) as u64,
            0x0201_7E64 => {
                box_set_value(&mut mon, id, val, &mut h);
                0
            }
            0x0201_7E1C => party_get_value(&mut mon, id, val, &mut h) as u64,
            0x0201_7E40 => {
                party_set_value(&mut mon, id, val, &mut h);
                0
            }
            _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
        }
    };
    c.regions[0] = mon;
    r
}

/// Every ported function against the original (`tools/check-fn.sh black-pokemon poke_tool`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &black_fndiff::Suite::new(
            "poke_tool",
            include_str!("../../../../tools/rom-diff/specs/black-pokemon/poke_tool.spec"),
            PORTED,
            RECORDED,
        ),
        &call,
    );
}

/// Records every hook call; results are fixed so the tests can follow them.
#[derive(Default)]
pub(crate) struct Rec {
    pub(crate) log: Vec<String>,
    pub(crate) gender: u32,
}

impl PokeToolHooks for Rec {
    fn gender(&mut self, species: u32, form: u32, pid: u32) -> u32 {
        self.log.push(format!("gender {species} {form} {pid:#x}"));
        self.gender
    }
    fn level_from_exp(&mut self, species: u32, form: u32, exp: u32) -> u32 {
        self.log.push(format!("level {species} {form} {exp}"));
        7
    }
    fn move_max_pp(&mut self, move_id: u32, pp_ups: u32) -> u32 {
        self.log.push(format!("maxpp {move_id} {pp_ups}"));
        35
    }
    fn personal_param(&mut self, species: u32, form: u32, param: u32) -> u32 {
        self.log.push(format!("personal {species} {form} {param}"));
        11
    }
    fn strbuf_from_mon(&mut self, strbuf: u32, mon_ofs: usize) {
        self.log.push(format!("sb_from {strbuf} {mon_ofs:#x}"));
    }
    fn strbuf_to_mon(&mut self, strbuf: u32, mon_ofs: usize, len: u32) {
        self.log.push(format!("sb_to {strbuf} {mon_ofs:#x} {len}"));
    }
    fn u16str_from_mon(&mut self, mon_ofs: usize, dst: u32, len: u32) {
        self.log.push(format!("u16_from {mon_ofs:#x} {dst} {len}"));
    }
    fn u16str_to_mon(&mut self, src: u32, mon_ofs: usize, len: u32) {
        self.log.push(format!("u16_to {src} {mon_ofs:#x} {len}"));
    }
    fn bad_egg_name_strbuf(&mut self, strbuf: u32) {
        self.log.push(format!("badegg_sb {strbuf}"));
    }
    fn bad_egg_name_u16(&mut self, dst: u32, len: u32) {
        self.log.push(format!("badegg_u16 {dst} {len}"));
    }
    fn species_name_to_scratch(&mut self, species: u32) {
        self.log.push(format!("name {species}"));
    }
    fn scratch_equals_mon_string(&mut self, mon_ofs: usize) -> u32 {
        self.log.push(format!("cmp {mon_ofs:#x}"));
        1
    }
    fn copy_ext_out(&mut self, mon_ofs: usize, dst: u32) {
        self.log.push(format!("ext_out {mon_ofs:#x} {dst}"));
    }
    fn copy_ext_in(&mut self, src: u32, mon_ofs: usize) {
        self.log.push(format!("ext_in {src} {mon_ofs:#x}"));
    }
}

/// A plain record: PID with shuffle row 6 (blocks B, A, C, D), box-plain flag set.
fn sample_mon() -> Vec<u8> {
    let mut m = vec![0u8; PARTY_MON_SIZE];
    m[..4].copy_from_slice(&(6u32 << 13).to_le_bytes());
    m[4] = FLAG_BOX_PLAIN as u8;
    m
}

#[test]
fn shuffle_rows_are_permutations() {
    for row in SHUFFLE.chunks(4) {
        let mut s = row.to_vec();
        s.sort();
        assert_eq!(s, [0, 32, 64, 96]);
    }
    for r in 0..8 {
        assert_eq!(
            SHUFFLE[r * 4..r * 4 + 4],
            SHUFFLE[(r + 24) * 4..(r + 24) * 4 + 4]
        );
    }
    // row 6 = B, A, C, D in storage order
    assert_eq!(block_offset(6 << 13, 0), 8 + 32);
    assert_eq!(block_offset(6 << 13, 1), 8);
    assert_eq!(
        block_offset(0x3E000 | 0x1234, 3),
        8 + SHUFFLE[31 * 4 + 3] as usize
    );
}

#[test]
fn crypt_is_an_involution_and_keeps_odd_tail() {
    let orig: Vec<u8> = (0..0x21u32).map(|i| (i * 7 + 3) as u8).collect();
    let mut d = orig.clone();
    crypt(&mut d, 0x21, 0xBEEF);
    assert_ne!(d[..0x20], orig[..0x20]);
    assert_eq!(d[0x20], orig[0x20]);
    crypt(&mut d, 0x21, 0xBEEF);
    assert_eq!(d, orig);
    // first key word of seed 0: (0 * 0x41C64E6D + 0x6073) >> 16 = 0
    let mut z = [0u8; 4];
    crypt(&mut z, 4, 0);
    assert_eq!(z, [0, 0, 0x7E, 0xE9]);
}

#[test]
fn box_enter_exit_round_trip_and_bad_checksum() {
    let mut m = sample_mon();
    m[4] = 0;
    // build a valid encrypted record from a plaintext block set
    for i in 0..0x80 {
        m[8 + i] = (i * 5 + 1) as u8;
    }
    let plain = m.clone();
    let sum = checksum(&m[8..], 0x80);
    m[6..8].copy_from_slice(&sum.to_le_bytes());
    crypt(&mut m[8..], 0x80, sum as u32);
    let enc = m.clone();
    assert_eq!(box_enter(&mut m), 1);
    assert_eq!(m[8..0x88], plain[8..0x88]);
    assert_eq!(rd16(&m, 4), 0);
    box_exit(&mut m);
    assert_eq!(m, enc);
    // wrong checksum: enter fails, flags get the bad-egg bit
    m[6] ^= 1;
    assert_eq!(box_enter(&mut m), 0);
    assert_eq!(rd16(&m, 4), FLAG_BAD_EGG as u32);
}

#[test]
fn decrypt_on_access_and_reencrypt() {
    let mut m = sample_mon();
    m[4] = 0;
    for i in 0..0x80 {
        m[8 + i] = i as u8;
    }
    let plain = m.clone();
    let sum = checksum(&m[8..], 0x80);
    m[6..8].copy_from_slice(&sum.to_le_bytes());
    crypt(&mut m[8..], 0x80, sum as u32);
    let enc = m.clone();
    assert!(decrypt_on_access(&mut m));
    assert_eq!(m[8..0x88], plain[8..0x88]);
    assert!(!decrypt_on_access(&mut m));
    assert_eq!(reencrypt(&mut m, 0), 0);
    assert_eq!(reencrypt(&mut m, 1), 1);
    assert_eq!(m, enc);
    assert_eq!(reencrypt(&mut m, 1), 0);
}

#[test]
fn put_then_get_round_trips_fields() {
    let mut m = sample_mon();
    let mut h = Rec::default();
    box_put(&mut m, id::SPECIES, 495, &mut h);
    assert_eq!(box_get(&mut m, id::SPECIES, 0, &mut h), 495);
    box_put(&mut m, id::HELD_ITEM, 0x272, &mut h);
    box_put(&mut m, id::HELD_ITEM, 0x273, &mut h);
    assert_eq!(box_get(&mut m, id::HELD_ITEM, 0, &mut h), 0x272);
    for (i, iv) in [31, 0, 17, 5, 30, 12].iter().enumerate() {
        box_put(&mut m, id::IV_HP + i as u32, *iv, &mut h);
    }
    for (i, iv) in [31, 0, 17, 5, 30, 12].iter().enumerate() {
        assert_eq!(box_get(&mut m, id::IV_HP + i as u32, 0, &mut h), *iv);
    }
    box_put(&mut m, id::MOVE1 + 2, 33, &mut h);
    box_put(&mut m, id::PP_UPS1 + 2, 3, &mut h);
    assert_eq!(box_get(&mut m, id::MAX_PP1 + 2, 0, &mut h), 35);
    assert_eq!(h.log.last().unwrap(), "maxpp 33 3");
    assert_eq!(box_get(&mut m, id::MAX_PP1, 0, &mut h), 0);
    // ribbons and flag words
    box_put(&mut m, id::RIBBON_FIRST + 28, 1, &mut h);
    assert_eq!(box_get(&mut m, id::RIBBON_FIRST + 28, 0, &mut h), 1);
    assert_eq!(box_get(&mut m, id::RIBBON_FIRST + 27, 0, &mut h), 0);
    box_put(&mut m, id::FLAGS3_LAST, 5, &mut h);
    assert_eq!(box_get(&mut m, id::FLAGS3_LAST, 0, &mut h), 1);
    box_put(&mut m, id::FLAGS3_LAST, 0, &mut h);
    assert_eq!(box_get(&mut m, id::FLAGS3_LAST, 0, &mut h), 0);
    // unknown ids
    assert_eq!(box_get(&mut m, 0xB0, 0, &mut h), 0);
    assert_eq!(box_get(&mut m, 0x1_0000, 0, &mut h), 0);
}

#[test]
fn ev_total_is_capped_at_510() {
    let mut m = sample_mon();
    let mut h = Rec::default();
    box_put(&mut m, id::EV_HP, 255, &mut h);
    box_put(&mut m, id::EV_HP + 1, 255, &mut h);
    box_put(&mut m, id::EV_HP + 2, 100, &mut h);
    assert_eq!(box_get(&mut m, id::EV_HP + 2, 0, &mut h), 0);
    box_put(&mut m, id::EV_HP, 10, &mut h);
    box_put(&mut m, id::EV_HP + 2, 100, &mut h);
    assert_eq!(box_get(&mut m, id::EV_HP + 2, 0, &mut h), 100);
    // total is now 365: only 145 more fit
    box_put(&mut m, id::EV_SPD, 255, &mut h);
    assert_eq!(box_get(&mut m, id::EV_SPD, 0, &mut h), 145);
    // a request above 255 is clamped to 255 when there is room
    box_put(&mut m, id::EV_HP + 2, 0, &mut h);
    box_put(&mut m, id::EV_HP + 1, 0, &mut h);
    box_put(&mut m, id::EV_HP + 3, 0x1234, &mut h);
    assert_eq!(box_get(&mut m, id::EV_HP + 3, 0, &mut h), 255);
}

#[test]
fn packed_ivs_put_uses_the_gen5_order() {
    let mut m = sample_mon();
    let mut h = Rec::default();
    // HP 1, Atk 2, Def 3, SpA 4, SpD 5, Spe 6
    let v = 1 | (2 << 5) | (3 << 10) | (4 << 15) | (5 << 20) | (6 << 25);
    box_put(&mut m, id::IVS_PACKED, v, &mut h);
    let got: Vec<u32> = (0..6)
        .map(|i| box_get(&mut m, id::IV_HP + i, 0, &mut h))
        .collect();
    // storage order HP, Atk, Def, Spe, SpA, SpD
    assert_eq!(got, [1, 2, 3, 6, 4, 5]);
    // get 172 is the raw IV word without the egg/nickname bits (not the inverse of put)
    assert_eq!(
        box_get(&mut m, id::IVS_PACKED, 0, &mut h),
        1 | (2 << 5) | (3 << 10) | (6 << 15) | (4 << 20) | (5 << 25)
    );
}

#[test]
fn bad_egg_reports_the_bad_egg_species_and_name() {
    let mut m = sample_mon();
    let mut h = Rec::default();
    box_put(&mut m, id::SPECIES, 25, &mut h);
    box_put(&mut m, id::BAD_EGG_FLAG, 1, &mut h);
    assert_eq!(box_get(&mut m, id::SPECIES, 0, &mut h), BAD_EGG_SPECIES);
    assert_eq!(
        box_get(&mut m, id::SPECIES_OR_EGG, 0, &mut h),
        BAD_EGG_SPECIES
    );
    assert_eq!(box_get(&mut m, id::IS_EGG, 0, &mut h), 1);
    box_get(&mut m, id::NICKNAME_STRBUF, 77, &mut h);
    assert_eq!(h.log.last().unwrap(), "badegg_sb 77");
    box_put(&mut m, id::BAD_EGG_FLAG, 0, &mut h);
    box_get(&mut m, id::NICKNAME_STRBUF, 77, &mut h);
    // block C of row 6 is the third block in storage order after B, A: 8 + 64
    assert_eq!(h.log.last().unwrap(), "sb_from 77 0x48");
}

#[test]
fn nickname_put_updates_the_nicknamed_flag() {
    let mut m = sample_mon();
    let mut h = Rec::default();
    box_put(&mut m, id::NICKNAME_U16, 9, &mut h);
    // Rec::scratch_equals_mon_string returns 1 (equal to the default name): not nicknamed
    assert_eq!(box_get(&mut m, id::IS_NICKNAMED, 0, &mut h), 0);
    assert_eq!(&h.log[..3], ["u16_to 9 0x48 11", "name 0", "cmp 0x48"]);
}

#[test]
fn party_ids_read_and_write_the_extension() {
    let mut m = sample_mon();
    let mut h = Rec::default();
    party_put(&mut m, id::HP, 123, &mut h);
    party_put(&mut m, id::LEVEL, 0x1_05, &mut h);
    assert_eq!(party_get(&mut m, id::HP, 0, &mut h), 123);
    assert_eq!(m[0x8E], 123);
    assert_eq!(party_get(&mut m, id::LEVEL, 0, &mut h), 5);
    assert_eq!(party_get(&mut m, id::PARTY_A8, 0, &mut h), 1);
    // the box-level level (id 158 without the party switch) is computed from EXP
    assert_eq!(box_get(&mut m, id::LEVEL, 0, &mut h), 7);
    assert_eq!(party_get(&mut m, id::PARTY_EXT_BYTES, 99, &mut h), 1);
    assert_eq!(h.log.last().unwrap(), "ext_out 0x9c 99");
    assert_eq!(party_get(&mut m, id::STATUS, 0, &mut h), 0);
}

#[test]
fn party_wrappers_encrypt_around_the_access() {
    let mut m = sample_mon();
    m[4] = 0;
    for (i, b) in m.iter_mut().enumerate().skip(8) {
        *b = (i * 3) as u8;
    }
    let sum = checksum(&m[8..], 0x80);
    m[6..8].copy_from_slice(&sum.to_le_bytes());
    let pid = rd32(&m, 0);
    let plain = m.clone();
    crypt(&mut m[8..], 0x80, sum as u32);
    crypt(&mut m[0x88..], 0x54, pid);
    let enc = m.clone();
    let mut h = Rec::default();
    let hp = party_get_value(&mut m, id::HP, 0, &mut h);
    assert_eq!(hp, u16::from_le_bytes([plain[0x8E], plain[0x8F]]) as u32);
    assert_eq!(m, enc);
    party_set_value(&mut m, id::HP, 777, &mut h);
    assert_ne!(m, enc);
    assert_eq!(party_get_value(&mut m, id::HP, 0, &mut h), 777);
    // a corrupted checksum blocks the put
    let mut bad = enc.clone();
    bad[6] ^= 0xFF;
    let before = bad.clone();
    party_set_value(&mut bad, id::HP, 5, &mut h);
    assert_eq!(rd16(&bad, 4) & FLAG_BAD_EGG as u32, FLAG_BAD_EGG as u32);
    assert_ne!(bad, before);
    // flags bit 0 set: all enter/exit work is skipped
    let mut p = plain.clone();
    p[4] = FLAG_PARTY_PLAIN as u8;
    assert_eq!(party_enter(&mut p), 1);
    party_exit(&mut p);
    assert_eq!(p[8..], plain[8..]);
}

#[test]
fn gender_get_writes_back_and_checksums() {
    let mut m = sample_mon();
    let mut h = Rec {
        gender: 2,
        ..Rec::default()
    };
    assert_eq!(box_get(&mut m, id::GENDER, 0, &mut h), 2);
    assert_eq!(h.log[0], "gender 0 0 0xc000");
    let b = block_offset(6 << 13, 1);
    assert_eq!(m[b + 0x18], 4);
    assert_eq!(
        rd16(&m, 6),
        checksum(&m[BLOCKS_OFS..], BLOCKS_SIZE as u32) as u32
    );
}

#[test]
fn plate_types() {
    assert_eq!(plate_type(0x12A), 9);
    assert_eq!(plate_type(0x138), 16);
    assert_eq!(plate_type(0x139), 8);
    assert_eq!(plate_type(0x129), 0);
    assert_eq!(plate_type(0x13A), 0);
}

#[test]
fn arceus_type_uses_the_plate_only_with_multitype() {
    let mut m = sample_mon();
    let mut h = Rec::default();
    box_put(&mut m, id::SPECIES, 493, &mut h);
    box_put(&mut m, id::HELD_ITEM, 0x12B, &mut h);
    box_put(&mut m, id::ABILITY, 0x79, &mut h);
    assert_eq!(box_get(&mut m, id::TYPE1, 0, &mut h), 10);
    box_put(&mut m, id::ABILITY, 0x78, &mut h);
    assert_eq!(box_get(&mut m, id::TYPE2, 0, &mut h), 11);
    assert_eq!(h.log.last().unwrap(), "personal 493 0 7");
}
