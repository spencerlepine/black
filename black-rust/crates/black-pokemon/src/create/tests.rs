use super::*;
use black_core::poke_crypt::checksum;
use black_fndiff::{run_suite, Arg, Case};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x020173F0, "e8f9da7f472fcf28"),
    (0x02017430, "54a2e6724c81ea31"),
    (0x0201747C, "d0f89b0e62b513af"),
    (0x02017528, "a2e52794efa8b265"),
    (0x02017550, "1b2e086b3ac6d96e"),
    (0x0201756C, "ac6751ed9177763d"),
    (0x02017588, "2c56b8306709e654"),
    (0x02017600, "2752dfbfe1938fc1"),
    (0x0201761C, "76b6846b4c681aaf"),
    (0x02017D30, "cfc7016ee54658a6"),
    (0x02017D70, "f0da9be486e76544"),
    (0x02018328, "54639507ec764f0a"),
    (0x020185C4, "ef539ce108088b0b"),
];
// fndiff-recorded-end

use crate::poke_tool::tests::Rom;

/// Address of the `"poke_tool.c"` string the original passes to the allocator.
const FILE_POKE_TOOL: u32 = 0x020A_72E4;

/// The bytes at 0x0209DF0C and 0x0209DF08 in the ROM image (language 2, version 0x15): the original reads
/// them from RAM, and the emulator's image holds the ROM's initial values.
const GAME_LANGUAGE: u32 = 2;
const GAME_VERSION: u32 = 0x15;

/// The unported callees, routed to the spec's stub lines (same model as the ROM side).
impl CreateHooks for Rom<'_> {
    fn heap_alloc(&mut self, heap: u32, size: u32, align: u32, line: u32) -> u32 {
        self.env.call(
            0x0203_0734,
            &mut [
                Arg::V(heap),
                Arg::V(size),
                Arg::V(align),
                Arg::V(FILE_POKE_TOOL),
                Arg::V(line),
            ],
        )
    }
    fn init_pokemon(&mut self, mon: u32, species: u32, level: u32, a2: u32, a3: u32) {
        self.env.call(
            0x0201_756C,
            &mut [
                Arg::V(mon),
                Arg::V(species),
                Arg::V(level),
                Arg::V(a2),
                Arg::V(a3),
            ],
        );
    }
}

impl InitHooks for Rom<'_> {
    fn init_pokemon8(
        &mut self,
        mon: u32,
        species: u32,
        level: u32,
        a3: u32,
        a4: u32,
        a5: u32,
        a6: u32,
        a7: u32,
    ) {
        self.env.call(
            0x0201_7588,
            &mut [
                Arg::V(mon),
                Arg::V(species),
                Arg::V(level),
                Arg::V(a3),
                Arg::V(a4),
                Arg::V(a5),
                Arg::V(a6),
                Arg::V(a7),
            ],
        );
    }
    fn core_init_at(
        &mut self,
        mon: u32,
        species: u32,
        level: u32,
        a3: u32,
        a4: u32,
        a5: u32,
        a6: u32,
        a7: u32,
    ) {
        self.env.call(
            0x0201_761C,
            &mut [
                Arg::V(mon),
                Arg::V(species),
                Arg::V(level),
                Arg::V(a3),
                Arg::V(a4),
                Arg::V(a5),
                Arg::V(a6),
                Arg::V(a7),
            ],
        );
    }
    fn core_init(&mut self, species: u32, level: u32, a3: u32, a4: u32, a5: u32, a6: u32, a7: u32) {
        let mon = self.mon;
        self.core_init_at(mon, species, level, a3, a4, a5, a6, a7);
    }
    fn recalc(&mut self) {
        let mon = self.mon;
        self.recalc_at(mon);
    }
    fn recalc_at(&mut self, mon: u32) {
        self.env.call(0x0201_85C4, &mut [Arg::V(mon)]);
    }
    fn recalc_stats(&mut self) {
        let mon = self.mon;
        self.env.call(0x0201_8A7C, &mut [Arg::V(mon)]);
    }
    fn ext_alloc(&mut self, heap: u32) -> u32 {
        self.env.call(0x0200_8F08, &mut [Arg::V(heap)])
    }
    fn heap_free(&mut self, block: u32) {
        self.env.call(0x0203_07B0, &mut [Arg::V(block)]);
    }
    fn personal_record(&mut self, species: u32, form: u32) -> u32 {
        self.env
            .call(0x0201_9C54, &mut [Arg::V(species), Arg::V(form)])
    }
    fn personal_field(&mut self, rec: u32, field: u32) -> u32 {
        self.env
            .call(0x0201_AE1C, &mut [Arg::V(rec), Arg::V(field)])
    }
    fn rand(&mut self, max: u32) -> u32 {
        self.env.call(0x0200_56EC, &mut [Arg::V(max)])
    }
    fn mt_next(&mut self) -> u32 {
        self.env.call(0x0203_F0A8, &mut [])
    }
    fn exp_for_level(&mut self, species: u32, form: u32, level: u32) -> u32 {
        self.env.call(
            0x0201_8584,
            &mut [Arg::V(species), Arg::V(form), Arg::V(level)],
        )
    }
    fn learn_start_moves(&mut self) {
        let mon = self.mon;
        self.env.call(0x0201_7FB0, &mut [Arg::V(mon)]);
    }
    fn game_language(&mut self) -> u32 {
        GAME_LANGUAGE
    }
    fn game_version(&mut self) -> u32 {
        GAME_VERSION
    }
    fn load_learnset(&mut self, species: u32, form: u32, _block: u32) {
        self.env
            .call(0x0201_ADD0, &mut [Arg::V(species), Arg::V(form), Arg::P]);
    }
    fn learnset_entry(&mut self, _block: u32, index: u32) -> (u32, u32) {
        let o = index as usize * 4;
        let w = |i: usize| u16::from_le_bytes([self.aux[i], self.aux[i + 1]]) as u32;
        (w(o), w(o + 2))
    }
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    // the `buf` argument that is the heap block of the wrappers (a spec `ofs:argN` return)
    let block_arg = |n: usize| v[n];
    match addr {
        0x0201_7528 => {
            clear_pokemon(c.region_mut("mon"));
            0
        }
        0x0201_7550 => {
            clear_box_pokemon(c.region_mut("mon"));
            0
        }
        0x0201_7D30 => decrypt_party(c.region_mut("mon")) as u64,
        0x0201_7D70 => encrypt_party(c.region_mut("mon"), v[1]) as u64,
        0x0201_73F0 => {
            let mut h = Rom::new(&mut c.env, 0);
            // the block comes back as an address inside `buf` argument 5; the spec compares its offset
            match new_pokemon(v[0], v[1], v[2], v[3], v[4], &mut h) {
                0 => 0,
                a => (a - block_arg(5)) as u64,
            }
        }
        0x0201_7430 => {
            let mut h = Rom::new(&mut c.env, 0);
            match new_pokemon8(v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], &mut h) {
                0 => 0,
                a => (a - block_arg(8)) as u64,
            }
        }
        0x0201_756C => {
            let mut h = Rom::new(&mut c.env, 0);
            init_pokemon(v[0], v[1], v[2], v[3], v[4], &mut h);
            0
        }
        0x0201_7600 => {
            let mut h = Rom::new(&mut c.env, 0);
            core_init_default(v[0], v[1], v[2], v[3], v[4], &mut h);
            0
        }
        0x0201_7588 | 0x0201_761C | 0x0201_85C4 | 0x0201_747C | 0x0201_8328 => {
            let mut mon = std::mem::take(&mut c.regions[0]);
            let mut ret = 0;
            {
                let mut h = Rom::new(&mut c.env, v[0]);
                match addr {
                    0x0201_7588 => init_pokemon_full(
                        &mut mon, v[1], v[2], v[3], v[4], v[5], v[6], v[7], &mut h,
                    ),
                    0x0201_761C => {
                        init_core(&mut mon, v[1], v[2], v[3], v[4], v[5], v[6], v[7], &mut h)
                    }
                    0x0201_85C4 => recalc(&mut mon, &mut h),
                    0x0201_747C => {
                        // the hooks' record base is the new block
                        h.mon = block_arg(2);
                        let mut new = std::mem::take(&mut c.bufs[2]);
                        let a = copy_pokemon_new(&mut mon, v[1], &mut new, &mut h);
                        c.bufs[2] = new;
                        ret = (a - block_arg(2)) as u64;
                    }
                    _ => {
                        h.aux = c.bufs[3].clone();
                        let mut cursor = u32::from_le_bytes(c.regions[1][..4].try_into().unwrap());
                        ret = next_level_up_move(&mut mon, &mut cursor, v[2], &mut h) as u64;
                        c.regions[1][..4].copy_from_slice(&cursor.to_le_bytes());
                    }
                }
            }
            c.regions[0] = mon;
            ret
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-pokemon create`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &black_fndiff::Suite::new(
            "create",
            include_str!("../../../../tools/rom-diff/specs/black-pokemon/create.spec"),
            PORTED,
            RECORDED,
        ),
        &call,
    );
}

/// Records the allocator and initialiser calls; the allocator answers with a fixed block or fails.
struct Rec {
    log: Vec<String>,
    block: u32,
}

impl CreateHooks for Rec {
    fn heap_alloc(&mut self, heap: u32, size: u32, align: u32, line: u32) -> u32 {
        self.log
            .push(format!("alloc {heap:#x} {size:#x} {align} {line:#x}"));
        self.block
    }
    fn init_pokemon(&mut self, mon: u32, species: u32, level: u32, a2: u32, a3: u32) {
        self.log
            .push(format!("init {mon:#x} {species} {level} {a2} {a3}"));
    }
}

#[test]
fn new_pokemon_allocates_then_initialises() {
    let mut h = Rec {
        log: Vec::new(),
        block: 0x0230_0000,
    };
    assert_eq!(new_pokemon(495, 5, 1, 2, 0x1_8003, &mut h), 0x0230_0000);
    // the heap id is a halfword
    assert_eq!(
        h.log,
        ["alloc 0x8003 0xdc 1 0xa0", "init 0x2300000 495 5 1 2"]
    );
    // an allocation failure skips the initialiser and returns 0
    let mut h = Rec {
        log: Vec::new(),
        block: 0,
    };
    assert_eq!(new_pokemon(495, 5, 1, 2, 3, &mut h), 0);
    assert_eq!(h.log, ["alloc 0x3 0xdc 1 0xa0"]);
}

/// A record with the given PID, flags and checksum, and recognisable bytes elsewhere.
fn sample(flags: u16) -> Vec<u8> {
    let mut m = vec![0u8; PARTY_MON_SIZE];
    for (i, b) in m.iter_mut().enumerate().skip(8) {
        *b = (i * 7 + 1) as u8;
    }
    m[..4].copy_from_slice(&0x1234_5678u32.to_le_bytes());
    m[4..6].copy_from_slice(&flags.to_le_bytes());
    m[6..8].copy_from_slice(&0xBEEFu16.to_le_bytes());
    m
}

#[test]
fn clear_pokemon_leaves_an_encrypted_zero_record() {
    let mut m = sample(0);
    clear_pokemon(&mut m);
    // header zero, blocks and extension are the key streams of seed 0 (blocks: the zero checksum, ext: zero PID)
    assert!(m[..8].iter().all(|&b| b == 0));
    let mut want = vec![0u8; PARTY_MON_SIZE];
    crypt(&mut want[BLOCKS_OFS..], BLOCKS_SIZE as u32, 0);
    crypt(&mut want[PARTY_EXT_OFS..], PARTY_EXT_SIZE as u32, 0);
    assert_eq!(m, want);
    assert_eq!(&m[8..12], [0, 0, 0x7E, 0xE9]);
    // the box variant stops at 0x88
    let mut b = sample(0);
    clear_box_pokemon(&mut b);
    assert_eq!(b[..BOX_MON_SIZE], want[..BOX_MON_SIZE]);
    assert_eq!(b[BOX_MON_SIZE..], sample(0)[BOX_MON_SIZE..]);
}

#[test]
fn decrypt_party_marks_the_record_plain_once() {
    let mut m = sample(0);
    let plain = m.clone();
    // encrypt as the exit routine would
    let pid = rd32(&m, 0);
    crypt(&mut m[PARTY_EXT_OFS..], PARTY_EXT_SIZE as u32, pid);
    let sum = checksum(&m[BLOCKS_OFS..], BLOCKS_SIZE as u32) as u16;
    m[6..8].copy_from_slice(&sum.to_le_bytes());
    crypt(&mut m[BLOCKS_OFS..], BLOCKS_SIZE as u32, sum as u32);
    // decrypt_party does not look at the checksum value, only uses it as the seed
    let enc = m.clone();
    assert!(decrypt_party(&mut m));
    assert_eq!(rd16(&m, 4), 3);
    assert_eq!(m[8..], {
        let mut p = plain.clone();
        p[6..8].copy_from_slice(&sum.to_le_bytes());
        p[8..].to_vec()
    });
    // already plain: nothing happens, whatever the other flag bits
    let again = m.clone();
    assert!(!decrypt_party(&mut m));
    assert_eq!(m, again);
    let mut f = enc.clone();
    f[4] = 1;
    let before = f.clone();
    assert!(!decrypt_party(&mut f));
    assert_eq!(f, before);
}

#[test]
fn encrypt_party_undoes_decrypt_party() {
    let mut m = sample(0);
    let pid = rd32(&m, 0);
    crypt(&mut m[PARTY_EXT_OFS..], PARTY_EXT_SIZE as u32, pid);
    let sum = checksum(&m[BLOCKS_OFS..], BLOCKS_SIZE as u32) as u16;
    m[6..8].copy_from_slice(&sum.to_le_bytes());
    crypt(&mut m[BLOCKS_OFS..], BLOCKS_SIZE as u32, sum as u32);
    let enc = m.clone();
    assert!(decrypt_party(&mut m));
    // only with flag 1
    assert_eq!(encrypt_party(&mut m, 0), 0);
    assert_eq!(encrypt_party(&mut m, 2), 0);
    assert_eq!(rd16(&m, 4), 3);
    assert_eq!(encrypt_party(&mut m, 1), 1);
    assert_eq!(m, enc);
    assert_eq!(rd16(&m, 4), 0);
    // already encrypted: nothing happens
    assert_eq!(encrypt_party(&mut m, 1), 0);
    assert_eq!(m, enc);
}

#[test]
fn shiny_needs_the_halfword_xor_below_8() {
    assert!(is_shiny(0, 0));
    // 0x5678 ^ 0x1234 ^ 0x1234 ^ 0x5678 = 0: shiny; the xor is 7 (shiny) or 8 (not) when the low byte differs
    assert!(is_shiny(0x1234_5678, 0x1234_5678));
    assert!(is_shiny(0x1234_5678, 0x1234_567F));
    assert!(!is_shiny(0x1234_5678, 0x1234_5670));
}
