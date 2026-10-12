use super::*;
use black_fndiff::{run_suite, Arg, Case};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x020185A0, "8fe6aa9aaa41c1bc"),
    (0x02018A7C, "0be55fcba4b60809"),
    (0x02019B04, "0d0277048f5a80e4"),
    (0x02019C54, "b8fc30321477b8f7"),
    (0x02019C7C, "02ab405375e1f12a"),
    (0x02019C90, "24597344817bba9b"),
    (0x0201AD88, "8f2f902b9380e9d8"),
    (0x0201ADA4, "2617e83c349d3427"),
    (0x0201ADE4, "fcf604ada3988cd4"),
    (0x0201AE1C, "d2dc8a56ebc923ba"),
    (0x0201AFD4, "b113fb51ac383552"),
];
// fndiff-recorded-end

use crate::poke_tool::tests::Rom;

/// Address of the `"poke_personal.c"` string the original passes to the allocator.
const FILE_POKE_PERSONAL: u32 = 0x020A_7310;

/// The unported callees, routed to the spec's stub lines (same model as the ROM side).
impl PersonalHooks for Rom<'_> {
    fn arc_member_offset(&mut self, handle: u32, member: u32) -> u32 {
        self.env
            .call(0x0204_9578, &mut [Arg::V(handle), Arg::V(member)])
    }
    fn arc_seek(&mut self, handle: u32, offset: u32) {
        self.env
            .call(0x0204_960C, &mut [Arg::V(handle), Arg::V(offset)]);
    }
    fn arc_read(&mut self, handle: u32, size: u32, dst: &mut [u8]) {
        self.env.call(
            0x0204_95F4,
            &mut [Arg::V(handle), Arg::V(size), Arg::Out(dst)],
        );
    }
    fn open_archive(&mut self, id: u32, arg: u32) -> u32 {
        self.env.call(0x0204_90F4, &mut [Arg::V(id), Arg::V(arg)])
    }
    fn load_record(&mut self, species: u32, form: u32, dst: &mut [u8]) {
        match self.handle {
            // the record reader is real code in the stat-calculation spec
            Some(handle) => read_record(species, form, handle, dst, self),
            None => {
                self.env.call(
                    0x0201_ADA4,
                    &mut [Arg::V(species), Arg::V(form), Arg::Out(dst)],
                );
            }
        }
    }
    fn heap_alloc(&mut self, heap: u32, size: u32, align: u32, line: u32) -> u32 {
        self.env.call(
            0x0203_0734,
            &mut [
                Arg::V(heap),
                Arg::V(size),
                Arg::V(align),
                Arg::V(FILE_POKE_PERSONAL),
                Arg::V(line),
            ],
        )
    }
}

fn global_u32(c: &Case, k: usize) -> u32 {
    u32::from_le_bytes(c.globals[k][..4].try_into().unwrap())
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    match addr {
        0x0201_AE1C => field(c.region("rec"), v[1]) as u64,
        0x0201_9C7C => ability_count(c.region("rec")) as u64,
        0x0201_9B04 => nature_modifier(v[0], v[1], v[2]) as u64,
        0x0201_AFD4 => {
            let handle = global_u32(c, 0);
            let mut h = Rom::new(&mut c.env, 0);
            form_record_index(v[0], v[1], handle, &mut h) as u64
        }
        0x0201_AD88 => {
            let mut handle = global_u32(c, 0);
            let mut h = Rom::new(&mut c.env, 0);
            open_archive_once(v[0], &mut handle, &mut h);
            c.globals[0][..4].copy_from_slice(&handle.to_le_bytes());
            0
        }
        0x0201_9C54 => {
            let mut cache = RecordCache::from_bytes(&c.globals[0]);
            let mut h = Rom::new(&mut c.env, 0);
            load_cached_record(&mut cache, v[0], v[1], &mut h);
            cache.write_to(&mut c.globals[0]);
            RecordCache::RECORD_ADDR as u64
        }
        0x0201_9C90 => {
            let mut cache = RecordCache::from_bytes(&c.globals[0]);
            let mut h = Rom::new(&mut c.env, 0);
            let r = cached_field(&mut cache, v[0], v[1], v[2], &mut h);
            cache.write_to(&mut c.globals[0]);
            r as u64
        }
        0x0201_ADA4 => {
            let handle = global_u32(c, 0);
            let mut rec = std::mem::take(&mut c.regions[0]);
            let mut h = Rom::new(&mut c.env, 0);
            read_record(v[0], v[1], handle, &mut rec, &mut h);
            c.regions[0] = rec;
            0
        }
        0x0201_ADE4 => {
            let handle = global_u32(c, 0);
            let mut block = std::mem::take(&mut c.bufs[3]);
            let mut h = Rom::new(&mut c.env, 0);
            let a = alloc_record(v[0], v[1], v[2], handle, &mut block, &mut h);
            c.bufs[3] = block;
            (a - v[3]) as u64
        }
        0x0201_85A0 => {
            let mut mon = std::mem::take(&mut c.regions[0]);
            let mut h = Rom::new(&mut c.env, v[0]);
            let r = nature_of(&mut mon, &mut h);
            c.regions[0] = mon;
            r as u64
        }
        0x0201_8A7C => {
            let mut cache = RecordCache::from_bytes(&c.globals[0]);
            let handle = global_u32(c, 1);
            let mut mon = std::mem::take(&mut c.regions[0]);
            {
                let mut h = Rom::new(&mut c.env, v[0]);
                h.handle = Some(handle);
                recalc_stats(&mut mon, &mut cache, &mut h);
            }
            c.regions[0] = mon;
            cache.write_to(&mut c.globals[0]);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-pokemon personal`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &black_fndiff::Suite::new(
            "personal",
            include_str!("../../../../tools/rom-diff/specs/black-pokemon/personal.spec"),
            PORTED,
            RECORDED,
        ),
        &call,
    );
}

/// A record whose byte i is i + 1, so a field's source is readable from its value.
fn numbered() -> [u8; RECORD_SIZE] {
    let mut r = [0u8; RECORD_SIZE];
    for (i, b) in r.iter_mut().enumerate() {
        *b = i as u8 + 1;
    }
    r
}

#[test]
fn field_reads_the_documented_offsets() {
    let r = numbered();
    // plain bytes 0-8, then the halfword at 0x22
    assert_eq!(field(&r, 0), 1);
    assert_eq!(field(&r, 8), 9);
    assert_eq!(field(&r, 9), 0x2423);
    // bytes 0x12..=0x1B are ids 0x14..=0x1D; id 0x24 is byte 9
    assert_eq!(field(&r, 0x14), 0x13);
    assert_eq!(field(&r, 0x17), 0x16);
    assert_eq!(field(&r, 0x1D), 0x1C);
    assert_eq!(field(&r, 0x24), 10);
    // halfwords and words
    assert_eq!(field(&r, 0x1E), 0x1E1D);
    assert_eq!(field(&r, 0x26), 0x2827);
    assert_eq!(field(&r, 0x27), 0x2C2B2A29);
    assert_eq!(field(&r, 0x2B), 0x3C3B3A39);
    // byte 0x21 = 0x22: low 6 bits = 0x22, bit 6 = 0, bit 7 = 0
    assert_eq!(field(&r, 0x21), 0x22);
    assert_eq!((field(&r, 0x22), field(&r, 0x23)), (0, 0));
    let mut r2 = r;
    r2[0x21] = 0xC1;
    assert_eq!(
        (field(&r2, 0x21), field(&r2, 0x22), field(&r2, 0x23)),
        (1, 1, 1)
    );
    // EV yields: two bits per stat from bit 0 of the halfword at 0x0A (0x0C0B here)
    r2[0x0A] = 0b1110_0100;
    r2[0x0B] = 0b0001_0011;
    let ev: Vec<u32> = (0xA..=0x10).map(|i| field(&r2, i)).collect();
    assert_eq!(ev, [0, 1, 2, 3, 3, 0, 1]);
    // out of range
    assert_eq!(field(&r, 0x2C), 0);
    assert_eq!(field(&r, u32::MAX), 0);
}

/// Archive stand-in: the record read returns a fixed record; calls are logged.
struct Archive {
    rec: [u8; RECORD_SIZE],
    log: Vec<String>,
}

impl PersonalHooks for Archive {
    fn arc_member_offset(&mut self, handle: u32, member: u32) -> u32 {
        self.log.push(format!("offset {handle:#x} {member}"));
        member * 0x3C
    }
    fn arc_seek(&mut self, handle: u32, offset: u32) {
        self.log.push(format!("seek {handle:#x} {offset}"));
    }
    fn arc_read(&mut self, handle: u32, size: u32, dst: &mut [u8]) {
        self.log.push(format!("read {handle:#x} {size}"));
        dst.copy_from_slice(&self.rec);
    }
    fn open_archive(&mut self, id: u32, arg: u32) -> u32 {
        self.log.push(format!("open {id:#x} {arg}"));
        0x77
    }
    fn load_record(&mut self, species: u32, form: u32, dst: &mut [u8]) {
        self.log.push(format!("load {species} {form}"));
        dst.copy_from_slice(&self.rec);
    }
    fn heap_alloc(&mut self, heap: u32, size: u32, align: u32, line: u32) -> u32 {
        self.log
            .push(format!("alloc {heap:#x} {size:#x} {align} {line:#x}"));
        0x0230_0000
    }
}

#[test]
fn form_index_follows_the_alternate_form_table() {
    let mut rec = [0u8; RECORD_SIZE];
    rec[0x1C..0x1E].copy_from_slice(&0x2A0u16.to_le_bytes());
    rec[0x20] = 3;
    let mut a = Archive {
        rec,
        log: Vec::new(),
    };
    // form 0 and forms past the count keep the species
    assert_eq!(form_record_index(487, 0, 9, &mut a), 487);
    assert_eq!(form_record_index(487, 3, 9, &mut a), 487);
    // forms 1 and 2 are the first form record and the one after it
    assert_eq!(form_record_index(487, 1, 9, &mut a), 0x2A0);
    assert_eq!(form_record_index(487, 2, 9, &mut a), 0x2A1);
    assert_eq!(
        a.log[..3],
        ["offset 0x9 487", "seek 0x9 29220", "read 0x9 60"]
    );
    // a negative form passes the signed compare
    assert_eq!(form_record_index(487, 0xFFFF_FFFF, 9, &mut a), 0x29E);
    // species 0x28A/0x28B (also with high bits set) are index 0 and touch nothing
    a.log.clear();
    assert_eq!(form_record_index(0x28A, 1, 9, &mut a), 0);
    assert_eq!(form_record_index(0x1_028B, 1, 9, &mut a), 0);
    assert!(a.log.is_empty());
    // no form table: species unchanged
    a.rec[0x1C] = 0;
    a.rec[0x1D] = 0;
    assert_eq!(form_record_index(487, 1, 9, &mut a), 487);
}

#[test]
fn cache_reloads_only_on_a_new_pair() {
    let mut a = Archive {
        rec: numbered(),
        log: Vec::new(),
    };
    let mut c = RecordCache {
        form: 0,
        species: 0,
        record: [0; RECORD_SIZE],
    };
    // species 0 form 0 is the empty cache's own pair: nothing is loaded
    load_cached_record(&mut c, 0, 0, &mut a);
    assert!(a.log.is_empty());
    load_cached_record(&mut c, 25, 1, &mut a);
    load_cached_record(&mut c, 25, 1, &mut a);
    assert_eq!(a.log, ["load 25 1"]);
    assert_eq!((c.species, c.form), (25, 1));
    assert_eq!(c.record, numbered());
    // 32-bit arguments never equal a cached halfword pair when they have high bits
    load_cached_record(&mut c, 0x1_0019, 1, &mut a);
    assert_eq!(a.log.len(), 2);
    assert_eq!(c.species, 0x19);
    // image round trip
    let mut img = [0u8; RecordCache::SIZE];
    c.write_to(&mut img);
    assert_eq!(RecordCache::from_bytes(&img), c);
    assert_eq!(&img[..4], [1, 0, 0x19, 0]);
}

#[test]
fn archive_opens_once() {
    let mut a = Archive {
        rec: numbered(),
        log: Vec::new(),
    };
    let mut handle = 0;
    open_archive_once(5, &mut handle, &mut a);
    open_archive_once(6, &mut handle, &mut a);
    assert_eq!(handle, 0x77);
    assert_eq!(a.log, ["open 0x10 5"]);
}

#[test]
fn nature_modifier_follows_the_table() {
    // every row raises one stat and lowers the same stat, or none
    for row in NATURE_TABLE.chunks(5) {
        assert_eq!(row.iter().map(|&x| x as i32).sum::<i32>(), 0);
    }
    // row 3 (Adamant) raises Atk and lowers SpA; entries are in the order Atk, Def, SpA, SpD, Spe
    assert_eq!(NATURE_TABLE[15..20], [1, 0, -1, 0, 0]);
    assert_eq!(nature_modifier(3, 100, 1), 110);
    assert_eq!(nature_modifier(3, 100, 3), 90);
    assert_eq!(nature_modifier(3, 100, 2), 100);
    // the product is cut to 16 bits before the division by 100: 0x10000 * 110 -> 0
    assert_eq!(nature_modifier(3, 0x10000, 1), 0);
    // neutral row, stat 0 and stats above 5 give the value back (unmasked)
    assert_eq!(nature_modifier(0, 0x1_2345, 1), 0x1_2345);
    assert_eq!(nature_modifier(3, 0x1_2345, 6), 0x1_2345);
    // past the 125 table bytes the original reads the three padding bytes and then the shuffle table
    assert_eq!(nature_modifier(25, 100, 1), 100);
    assert_eq!(nature_modifier(25, 100, 4), 100);
}

#[test]
fn ability_count_is_two_for_a_second_ability() {
    let mut r = [0u8; RECORD_SIZE];
    assert_eq!(ability_count(&r), 1);
    r[0x19] = 7;
    assert_eq!(ability_count(&r), 2);
}

#[test]
fn recalc_stats_computes_stats_and_shifts_hp() {
    use crate::poke_tool::tests::Rec;
    use crate::poke_tool::{party_get_value, party_set_value};

    struct H(Rec, [u8; RECORD_SIZE]);
    impl PokeToolHooks for H {
        fn gender(&mut self, a: u32, b: u32, c: u32) -> u32 {
            self.0.gender(a, b, c)
        }
        fn level_from_exp(&mut self, a: u32, b: u32, c: u32) -> u32 {
            self.0.level_from_exp(a, b, c)
        }
        fn move_max_pp(&mut self, a: u32, b: u32) -> u32 {
            self.0.move_max_pp(a, b)
        }
        fn personal_param(&mut self, a: u32, b: u32, c: u32) -> u32 {
            self.0.personal_param(a, b, c)
        }
        fn strbuf_from_mon(&mut self, a: u32, b: usize) {
            self.0.strbuf_from_mon(a, b)
        }
        fn strbuf_to_mon(&mut self, a: u32, b: usize, c: u32) {
            self.0.strbuf_to_mon(a, b, c)
        }
        fn u16str_from_mon(&mut self, a: usize, b: u32, c: u32) {
            self.0.u16str_from_mon(a, b, c)
        }
        fn u16str_to_mon(&mut self, a: u32, b: usize, c: u32) {
            self.0.u16str_to_mon(a, b, c)
        }
        fn bad_egg_name_strbuf(&mut self, a: u32) {
            self.0.bad_egg_name_strbuf(a)
        }
        fn bad_egg_name_u16(&mut self, a: u32, b: u32) {
            self.0.bad_egg_name_u16(a, b)
        }
        fn species_name_to_scratch(&mut self, a: u32) {
            self.0.species_name_to_scratch(a)
        }
        fn scratch_equals_mon_string(&mut self, a: usize) -> u32 {
            self.0.scratch_equals_mon_string(a)
        }
        fn copy_ext_out(&mut self, a: usize, b: u32) {
            self.0.copy_ext_out(a, b)
        }
        fn copy_ext_in(&mut self, a: u32, b: usize) {
            self.0.copy_ext_in(a, b)
        }
    }
    impl PersonalHooks for H {
        fn arc_member_offset(&mut self, _: u32, _: u32) -> u32 {
            unreachable!()
        }
        fn arc_seek(&mut self, _: u32, _: u32) {
            unreachable!()
        }
        fn arc_read(&mut self, _: u32, _: u32, _: &mut [u8]) {
            unreachable!()
        }
        fn open_archive(&mut self, _: u32, _: u32) -> u32 {
            unreachable!()
        }
        fn load_record(&mut self, _: u32, _: u32, dst: &mut [u8]) {
            dst.copy_from_slice(&self.1);
        }
        fn heap_alloc(&mut self, _: u32, _: u32, _: u32, _: u32) -> u32 {
            unreachable!()
        }
    }

    // base stats HP 45, Atk 49, Def 49, Spe 45, SpA 65, SpD 65
    let mut rec = [0u8; RECORD_SIZE];
    rec[..6].copy_from_slice(&[45, 49, 49, 45, 65, 65]);
    let mut h = H(Rec::default(), rec);
    // a plain record (both plain flags), PID 0 (blocks in storage order), species 1, level 50, all IVs 31
    let mut mon = vec![0u8; crate::poke_tool::PARTY_MON_SIZE];
    mon[4] = 3;
    fn set(mon: &mut [u8], id: u32, v: u32, h: &mut H) {
        party_set_value(mon, id, v, h);
    }
    fn get(mon: &mut [u8], id: u32, h: &mut H) -> u32 {
        party_get_value(mon, id, 0, h)
    }
    set(&mut mon, 5, 1, &mut h);
    set(&mut mon, 0x9E, 50, &mut h);
    for iv in 0x46..=0x4B {
        set(&mut mon, iv, 31, &mut h);
    }
    let mut cache = RecordCache {
        form: 0xFFFF,
        species: 0xFFFF,
        record: [0; RECORD_SIZE],
    };
    recalc_stats(&mut mon, &mut cache, &mut h);
    let stats: Vec<u32> = (0xA1..=0xA6).map(|id| get(&mut mon, id, &mut h)).collect();
    // HP (90 + 31) * 50 / 100 + 50 + 10; others ((2 * base + 31) * 50 / 100 + 5), order Atk Def Spe SpA SpD
    assert_eq!(stats, [120, 69, 69, 65, 85, 85]);
    // fainted (HP 0) with max HP 0 before: HP becomes the new max HP
    assert_eq!(get(&mut mon, 0xA0, &mut h), 120);
    // raise EVs and set an Adamant nature: Atk +10%, SpA -10%; HP shifts by the max HP change (+0 here)
    set(&mut mon, 0x70, 3, &mut h);
    set(&mut mon, 0xE, 252, &mut h);
    set(&mut mon, 0xD, 252, &mut h);
    set(&mut mon, 0xA0, 100, &mut h);
    recalc_stats(&mut mon, &mut cache, &mut h);
    // HP: (90 + 31 + 63) * 50 / 100 + 60 = 152; cur 100 + (152 - 120)
    assert_eq!(get(&mut mon, 0xA1, &mut h), 152);
    assert_eq!(get(&mut mon, 0xA0, &mut h), 132);
    // Atk: ((98 + 31 + 63) * 50 / 100 + 5) = 101 -> 111; SpA 85 -> 76
    assert_eq!(get(&mut mon, 0xA2, &mut h), 111);
    assert_eq!(get(&mut mon, 0xA5, &mut h), 76);
    // a lowered max HP caps the current HP
    set(&mut mon, 0xD, 0, &mut h);
    set(&mut mon, 0xA0, 152, &mut h);
    recalc_stats(&mut mon, &mut cache, &mut h);
    assert_eq!(get(&mut mon, 0xA1, &mut h), 120);
    assert_eq!(get(&mut mon, 0xA0, &mut h), 120);
    // Shedinja (0x124) always has 1 HP
    set(&mut mon, 5, 0x124, &mut h);
    recalc_stats(&mut mon, &mut cache, &mut h);
    assert_eq!(get(&mut mon, 0xA1, &mut h), 1);
    assert_eq!(get(&mut mon, 0xA0, &mut h), 1);
}
