use super::*;
use crate::hooks::PokeToolHooks;
use black_fndiff::{run_suite, Arg, Case, StubEnv};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02017FB0, "b585fbc9f0268b64"),
    (0x02018094, "22c11c26f32f5d0a"),
    (0x020180FC, "2f3d3558abe9ed76"),
    (0x020181A0, "c3390494b85ff0a6"),
    (0x020181FC, "6c210d8c13cc0ac8"),
    (0x02018240, "c9087202178aedf0"),
    (0x020182A8, "21975a5bcfa33f6f"),
    (0x0201ADD0, "9117822534fb8695"),
    (0x0201B920, "184a615f8705b58f"),
    (0x0201B970, "239afaadf0ff89db"),
    (0x0201B9F0, "de5052d9e5bbcddf"),
    (0x0201BA34, "7f8343db52a12744"),
    (0x0201BD28, "a72dfd8348804bf1"),
    (0x0201BDB4, "8b1443b740d354e5"),
    (0x0201BEB8, "bc24aa8bacd2de99"),
    (0x0201BF44, "5f90aef8386d011c"),
    (0x0201BF58, "f0aec0b88b9c90fa"),
    (0x0201C020, "bafe45bcc701cbae"),
    (0x0201C0BC, "f1ff80b767951e0a"),
    (0x0201C158, "6f0afd665ba5c53d"),
    (0x0201C184, "24ba4159b5f8b461"),
    (0x0201C1BC, "b81b57b02340c6e0"),
    (0x0201C1C8, "2cd07c01ac8b1523"),
    (0x0201C1D8, "484993e01e1bdef8"),
    (0x0201C1E8, "90e3aeab4d1fc213"),
    (0x0201C1F4, "20e393038b9cd7d1"),
    (0x0201C284, "824153906dd21900"),
    (0x0201C314, "a8f49c1ed4436dd6"),
    (0x0201C328, "d9b23e19fa367f0b"),
    (0x0201C3C0, "2538d1458b25543d"),
    (0x0201C408, "1355386cc1ed8b5b"),
];
// fndiff-recorded-end

/// The callees outside the area, routed to the spec's stub lines (same model as the ROM side).
struct Rom<'a> {
    env: &'a mut StubEnv,
    /// Emulator address of the learnset buffer `load_learnset` gets (the stub logs it).
    dst: u32,
}

impl WazaHooks for Rom<'_> {
    fn load_move(&mut self, move_id: u32, dst: &mut [u8; RECORD_SIZE]) {
        self.env.call(
            0x0204_8F4C,
            &mut [Arg::Out(dst), Arg::V(MOVE_ARC), Arg::V(move_id)],
        );
    }
    fn move_param(&mut self, move_id: u32, param: u32) -> u32 {
        self.env
            .call(0x0201_BD28, &mut [Arg::V(move_id), Arg::V(param)])
    }
    fn form_record_index(&mut self, species: u32, form: u32) -> u32 {
        self.env
            .call(0x0201_AFD4, &mut [Arg::V(species), Arg::V(form)])
    }
    fn load_learnset(&mut self, index: u32, _dst: &mut [u8]) {
        // the stub writes nothing (`args=v,v,v`): only the call is compared
        self.env.call(
            0x0204_8F4C,
            &mut [Arg::V(self.dst), Arg::V(LEARNSET_ARC), Arg::V(index)],
        );
    }
    fn learnset_read(&mut self, species: u32, form: u32, dst: u32, _buf: &mut [u8]) {
        // the ADD0 stub writes nothing: the learnset is the planted global
        self.env.call(
            0x0201_ADD0,
            &mut [Arg::V(species), Arg::V(form), Arg::V(dst)],
        );
    }
    fn free(&mut self, block: u32) {
        self.env.call(0x0203_07B0, &mut [Arg::V(block)]);
    }
    fn alloc(&mut self, heap: u32, size: u32, line: u32) -> u32 {
        self.env.call(
            0x0203_0734,
            &mut [Arg::V(heap), Arg::V(size), Arg::V(0), Arg::P, Arg::V(line)],
        )
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
    fn move_max_pp(&mut self, move_id: u32, pp_ups: u32) -> u32 {
        self.env
            .call(0x0201_C158, &mut [Arg::V(move_id), Arg::V(pp_ups)])
    }
    fn personal_param(&mut self, _: u32, _: u32, _: u32) -> u32 {
        unreachable!("not read by the move routines")
    }
    fn strbuf_from_mon(&mut self, _: u32, _: usize) {
        unreachable!("not read by the move routines")
    }
    fn strbuf_to_mon(&mut self, _: u32, _: usize, _: u32) {
        unreachable!("not read by the move routines")
    }
    fn u16str_from_mon(&mut self, _: usize, _: u32, _: u32) {
        unreachable!("not read by the move routines")
    }
    fn u16str_to_mon(&mut self, _: u32, _: usize, _: u32) {
        unreachable!("not read by the move routines")
    }
    fn bad_egg_name_strbuf(&mut self, _: u32) {
        unreachable!("not read by the move routines")
    }
    fn bad_egg_name_u16(&mut self, _: u32, _: u32) {
        unreachable!("not read by the move routines")
    }
    fn species_name_to_scratch(&mut self, _: u32) {
        unreachable!("not read by the move routines")
    }
    fn scratch_equals_mon_string(&mut self, _: usize) -> u32 {
        unreachable!("not read by the move routines")
    }
    fn copy_ext_out(&mut self, _: usize, _: u32) {
        unreachable!("not read by the move routines")
    }
    fn copy_ext_in(&mut self, _: u32, _: usize) {
        unreachable!("not read by the move routines")
    }
}

/// The move cache as planted in the regions: global 0x02146530 holds the cache struct's address or 0; the
/// struct holds the bufs and ids addresses and the count at +8; the static record is global 0x02146534.
struct Planted {
    has_cache: bool,
    cache: MoveCache,
    scratch: [u8; RECORD_SIZE],
}

fn plant(c: &Case) -> Planted {
    let has_cache = c.globals[0][..4] != [0; 4];
    let count = u32::from_le_bytes(c.region("cache")[8..12].try_into().unwrap()) as usize;
    let mut scratch = [0u8; RECORD_SIZE];
    scratch.copy_from_slice(&c.globals[1][..RECORD_SIZE]);
    let mut cache = MoveCache::new(count);
    for i in 0..count {
        let ids = c.region("ids");
        cache.ids[i] = u16::from_le_bytes([ids[2 * i], ids[2 * i + 1]]);
        cache.bufs[i].copy_from_slice(c.region(&format!("rec{i}")));
    }
    Planted {
        has_cache,
        cache,
        scratch,
    }
}

fn unplant(c: &mut Case, p: &Planted) {
    c.globals[1][..RECORD_SIZE].copy_from_slice(&p.scratch);
    for i in 0..p.cache.ids.len() {
        c.region_mut("ids")[2 * i..2 * i + 2].copy_from_slice(&p.cache.ids[i].to_le_bytes());
        c.region_mut(&format!("rec{i}"))
            .copy_from_slice(&p.cache.bufs[i]);
    }
}

/// Runs `f` with the planted cache (or none) and the static record, then writes the state back.
fn with_cache(
    c: &mut Case,
    f: impl FnOnce(Option<&mut MoveCache>, &mut [u8; RECORD_SIZE], &mut Rom) -> u64,
) -> u64 {
    let mut p = plant(c);
    let r = {
        let mut h = Rom {
            env: &mut c.env,
            dst: 0,
        };
        let cache = if p.has_cache {
            Some(&mut p.cache)
        } else {
            None
        };
        f(cache, &mut p.scratch, &mut h)
    };
    unplant(c, &p);
    r
}

/// Runs `f` on the planted `mon` region (written back afterwards).
fn with_mon(c: &mut Case, f: impl FnOnce(&mut [u8], &mut Rom) -> u64) -> u64 {
    let mut mon = c.region("mon").to_vec();
    let r = {
        let mut h = Rom {
            env: &mut c.env,
            dst: 0,
        };
        f(&mut mon, &mut h)
    };
    c.region_mut("mon").copy_from_slice(&mon);
    r
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    match addr {
        0x0201_BDB4 => record_param(c.region("rec"), v[1]) as u64,
        0x0201_BF44 => flag_test(c.region("rec"), v[1]) as u64,
        0x0201_C158 => {
            let mut h = Rom {
                env: &mut c.env,
                dst: 0,
            };
            max_pp(v[0], v[1], &mut h) as u64
        }
        0x0201_ADD0 => {
            let mut h = Rom {
                env: &mut c.env,
                dst: v[2],
            };
            // `dst` is a plain scalar in the spec: the stubbed load writes nothing, so any buffer will do
            read_learnset(v[0], v[1], &mut [0u8; 4], &mut h);
            0
        }
        0x0201_C328 => {
            // global 0x02146530 = the cache pointer; the allocator hands out blocks of the arena `arg buf` (bufs[2])
            let had = c.globals[0][..4] != [0; 4];
            let mut cache = had.then(|| MoveCache::new(0));
            let blocks = {
                let mut h = Rom {
                    env: &mut c.env,
                    dst: 0,
                };
                cache_create(&mut cache, v[0], v[1], &mut h)
            };
            if let Some(b) = blocks {
                let base = 0x02E1_0000 + 2 * 0x10000;
                let arena = &mut c.bufs[2];
                let mut put = |addr: u32, bytes: &[u8]| {
                    let o = (addr - base) as usize;
                    arena[o..o + bytes.len()].copy_from_slice(bytes);
                };
                put(b.cache, &b.bufs.to_le_bytes());
                put(b.cache + 4, &b.ids.to_le_bytes());
                let n = cache.as_ref().unwrap().ids.len() as u32;
                put(b.cache + 8, &n.to_le_bytes());
                for (i, &r) in b.recs.iter().enumerate() {
                    put(b.bufs + 4 * i as u32, &r.to_le_bytes());
                    put(
                        b.ids + 2 * i as u32,
                        &cache.as_ref().unwrap().ids[i].to_le_bytes(),
                    );
                }
                c.globals[0][..4].copy_from_slice(&b.cache.to_le_bytes());
            }
            0
        }
        0x0201_BD28 => with_cache(c, |cache, s, h| move_param(v[0], v[1], cache, s, h) as u64),
        0x0201_BEB8 => with_cache(c, |cache, s, h| record_flag(v[0], v[1], cache, s, h) as u64),
        0x0201_BF58 => with_cache(c, |cache, s, h| {
            record_field16(v[0], v[3], cache, s, h) as u64
        }),
        0x0201_C020 => with_cache(c, |cache, s, h| {
            record_pp_ups_count(v[0], cache, s, h) as u64
        }),
        0x0201_C0BC => {
            let mut out = c.bufs[2].clone();
            let r = with_cache(c, |cache, s, h| {
                record_signed_if_set(v[0], v[1], &mut out, cache, s, h);
                0
            });
            c.bufs[2] = out;
            r
        }
        0x0201_C1F4 => with_cache(c, |cache, s, h| record_is_101(v[0], cache, s, h) as u64),
        0x0201_C284 => with_cache(c, |cache, s, h| record_is_6(v[0], cache, s, h) as u64),
        0x0201_C1BC | 0x0201_C1C8 | 0x0201_C1D8 | 0x0201_C1E8 | 0x0201_C314 => {
            let mut h = Rom {
                env: &mut c.env,
                dst: 0,
            };
            let r = match addr {
                0x0201_C1BC => param1(v[0], &mut h),
                0x0201_C1C8 => param0_u8(v[0], &mut h),
                0x0201_C1D8 => param3_u16(v[0], &mut h),
                0x0201_C1E8 => param2(v[0], &mut h),
                _ => param3_nonzero(v[0], &mut h),
            };
            r as u64
        }
        0x0201_C184 => class_of(v[0]) as u64,
        0x0201_C3C0 => {
            let cache_addr = u32::from_le_bytes(c.globals[0][..4].try_into().unwrap());
            let has_cache = cache_addr != 0;
            let count = u32::from_le_bytes(c.region("cache")[8..12].try_into().unwrap()) as usize;
            let st = c.region("cache");
            let blocks = CacheBlocks {
                cache: cache_addr,
                ids: u32::from_le_bytes(st[4..8].try_into().unwrap()),
                bufs: u32::from_le_bytes(st[0..4].try_into().unwrap()),
                recs: {
                    let b = c.region("bufs");
                    (0..count)
                        .map(|i| u32::from_le_bytes(b[4 * i..4 * i + 4].try_into().unwrap()))
                        .collect()
                },
            };
            let mut opt = has_cache.then(|| MoveCache::new(count));
            {
                let mut h = Rom {
                    env: &mut c.env,
                    dst: 0,
                };
                cache_free(&mut opt, &blocks, &mut h);
            }
            c.globals[0][..4].copy_from_slice(&0u32.to_le_bytes());
            0
        }
        0x0201_C408 => cache_exists(&if c.globals[0][..4] != [0; 4] {
            Some(MoveCache::new(1))
        } else {
            None
        }) as u64,
        0x0201_B920 => with_mon(c, |mon, h| slot_below_max(mon, v[1], h) as u64),
        0x0201_B970 => with_mon(c, |mon, h| raise_pp(mon, v[1], v[2], h) as u64),
        0x0201_B9F0 => with_mon(c, |mon, h| {
            restore_mon(mon, h);
            0
        }),
        0x0201_BA34 => {
            let mut party = c.region("party").to_vec();
            {
                let mut h = Rom {
                    env: &mut c.env,
                    dst: 0,
                };
                restore_party(&mut party, &mut h);
            }
            c.region_mut("party").copy_from_slice(&party);
            0
        }
        0x0201_7FB0 => {
            let mut ls = c.globals[0][..0x100].to_vec();
            let r = with_mon(c, |mon, h| {
                learn_all(mon, &mut ls, h);
                0
            });
            c.globals[0][..0x100].copy_from_slice(&ls);
            r
        }
        0x0201_8094 => with_mon(c, |mon, h| learn_move(mon, v[1], h) as u64),
        0x0201_80FC => with_mon(c, |mon, h| {
            learn_shift(mon, v[1], h);
            0
        }),
        0x0201_81A0 => with_mon(c, |mon, h| {
            refill_pp(mon, h);
            0
        }),
        0x0201_81FC => with_mon(c, |mon, h| {
            learn_into_slot(mon, v[1], v[2], h);
            0
        }),
        0x0201_8240 => with_mon(c, |mon, h| learn_move_replacing(mon, v[1], h) as u64),
        0x0201_82A8 => with_mon(c, |mon, h| {
            replace_slot(mon, v[1], v[2], h);
            0
        }),
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-pokemon waza`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &black_fndiff::Suite::new(
            "waza",
            include_str!("../../../../tools/rom-diff/specs/black-pokemon/waza.spec"),
            PORTED,
            RECORDED,
        ),
        &call,
    );
}

/// A record whose byte i is i + 1 (so byte 0x12 = 0x13 ...).
fn numbered() -> [u8; RECORD_SIZE] {
    let mut r = [0u8; RECORD_SIZE];
    for (i, b) in r.iter_mut().enumerate() {
        *b = i as u8 + 1;
    }
    r
}

#[test]
fn getter_reads_the_documented_bytes() {
    let r = numbered();
    // bytes 0-3, then special-cased 4 / 6 / 7 (0xE)
    assert_eq!(
        (0..4).map(|p| record_param(&r, p)).collect::<Vec<_>>(),
        [1, 2, 3, 4]
    );
    assert_eq!(record_param(&r, 4), 5);
    assert_eq!(record_param(&r, 5), 6);
    assert_eq!(record_param(&r, 6), 7);
    assert_eq!(record_param(&r, 7), 0x0F);
    // nibbles of byte 7 (= 8)
    assert_eq!((record_param(&r, 8), record_param(&r, 9)), (0, 8));
    assert_eq!(record_param(&r, 10), 0x10);
    assert_eq!(record_param(&r, 11), 0x0A09);
    assert_eq!(
        (12..16).map(|p| record_param(&r, p)).collect::<Vec<_>>(),
        [0x0B, 0x0C, 0x0D, 0x0E]
    );
    // the stat-change groups of three: byte 0x15+3k, signed byte 0x18+3k, byte 0x1B+3k
    for k in 0..3u32 {
        assert_eq!(record_param(&r, 16 + k * 3), 0x16 + 3 * k);
        assert_eq!(record_param(&r, 17 + k * 3), 0x19 + 3 * k);
        assert_eq!(record_param(&r, 18 + k * 3), 0x1C + 3 * k);
    }
    assert_eq!(record_param(&r, 25), 0x13);
    assert_eq!(record_param(&r, 26), 0x14);
    assert_eq!(record_param(&r, 27), 0x15);
    assert_eq!(record_param(&r, 28), 0x1211);
    assert_eq!(record_param(&r, 29), 0);
    assert_eq!((record_param(&r, 30), record_param(&r, 31)), (0, 0));
    assert_eq!(record_param(&r, 32), 0);
    assert_eq!(record_param(&r, u32::MAX), 0);
}

#[test]
fn getter_special_values_and_signs() {
    let mut r = [0u8; RECORD_SIZE];
    r[4] = 101;
    r[0xE] = 6;
    r[6] = 0xFE;
    r[0x12] = 0xFD; // -3
    r[0x13] = 0x05;
    r[0x18] = 0x80;
    assert_eq!(record_param(&r, 4), 100);
    assert_eq!(record_param(&r, 7), 0);
    assert_eq!(record_param(&r, 6), 0xFFFF_FFFE);
    assert_eq!(record_param(&r, 17), 0xFFFF_FF80);
    // 25/26 keep the positive part only, 30/31 the magnitude of the negative part
    assert_eq!((record_param(&r, 25), record_param(&r, 26)), (0, 5));
    assert_eq!((record_param(&r, 30), record_param(&r, 31)), (3, 0));
    r[4] = 100;
    assert_eq!(record_param(&r, 4), 100);
    r[4] = 102;
    assert_eq!(record_param(&r, 4), 102);
}

#[test]
fn flag_test_is_a_register_shift() {
    let mut r = [0u8; RECORD_SIZE];
    r[0x20..0x24].copy_from_slice(&0x8000_0001u32.to_le_bytes());
    assert_eq!(flag_test(&r, 0), 1);
    assert_eq!(flag_test(&r, 1), 0);
    assert_eq!(flag_test(&r, 31), 1);
    // 32..=255 give 0; only the low byte of the argument counts
    assert_eq!(flag_test(&r, 32), 0);
    assert_eq!(flag_test(&r, 255), 0);
    assert_eq!(flag_test(&r, 256), 1);
    assert_eq!(flag_test(&r, 0x11F), 1);
}

/// Loads a record whose byte 5 (base PP) is the move id, and records the loads.
struct Fake {
    loads: Vec<u32>,
}

impl WazaHooks for Fake {
    fn load_move(&mut self, move_id: u32, dst: &mut [u8; RECORD_SIZE]) {
        self.loads.push(move_id);
        *dst = [0; RECORD_SIZE];
        dst[5] = move_id as u8;
    }
    fn move_param(&mut self, move_id: u32, _param: u32) -> u32 {
        move_id
    }
    fn form_record_index(&mut self, species: u32, form: u32) -> u32 {
        species * 10 + form
    }
    fn load_learnset(&mut self, index: u32, dst: &mut [u8]) {
        self.loads.push(index);
        dst[..4].copy_from_slice(&[0x21, 0, 1, 0]);
    }
    fn learnset_read(&mut self, _: u32, _: u32, _: u32, _: &mut [u8]) {}
    fn free(&mut self, _: u32) {}
    fn alloc(&mut self, _heap: u32, size: u32, line: u32) -> u32 {
        self.loads.push(line);
        0x0230_0000 + 0x100 * self.loads.len() as u32 + size
    }
}

#[test]
fn cache_hits_free_slots_and_the_full_cache_fallback() {
    let mut h = Fake { loads: vec![] };
    let mut cache = MoveCache::new(2);
    let mut scratch = [0u8; RECORD_SIZE];
    let p = param::BASE_PP;
    // free slots are filled in order, a repeat is a hit
    assert_eq!(
        move_param(20, p, Some(&mut cache), &mut scratch, &mut h),
        20
    );
    assert_eq!(
        move_param(30, p, Some(&mut cache), &mut scratch, &mut h),
        30
    );
    assert_eq!(
        move_param(20, p, Some(&mut cache), &mut scratch, &mut h),
        20
    );
    assert_eq!(h.loads, [20, 30]);
    assert_eq!(cache.ids, [20, 30]);
    // full: the LAST slot is reused, slot 0 stays
    assert_eq!(
        move_param(40, p, Some(&mut cache), &mut scratch, &mut h),
        40
    );
    assert_eq!(cache.ids, [20, 40]);
    assert_eq!(h.loads, [20, 30, 40]);
    // no cache: loaded into the scratch record every time
    assert_eq!(move_param(7, p, None, &mut scratch, &mut h), 7);
    assert_eq!(move_param(7, p, None, &mut scratch, &mut h), 7);
    assert_eq!(h.loads[3..], [7, 7]);
    assert_eq!(scratch[5], 7);
}

#[test]
fn max_pp_formula() {
    let mut h = Fake { loads: vec![] };
    // the fake's "base PP" is the move id
    assert_eq!(max_pp(35, 0, &mut h), 35);
    assert_eq!(max_pp(35, 3, &mut h), 56);
    assert_eq!(max_pp(35, 9, &mut h), 56);
    assert_eq!(max_pp(10, 1, &mut h), 12);
    assert_eq!(max_pp(5, 3, &mut h), 8);
    // u8 arithmetic: base 255 + 255 * 20 * 3 / 100 = 255 + 153 wraps
    assert_eq!(max_pp(255, 3, &mut h), (255 + 153) & 0xFF);
}

#[test]
fn learnset_reader_goes_through_the_form_index() {
    let mut h = Fake { loads: vec![] };
    let mut buf = [0xFFu8; 8];
    read_learnset(25, 3, &mut buf, &mut h);
    assert_eq!(h.loads, [253]);
    assert_eq!(buf, [0x21, 0, 1, 0, 0xFF, 0xFF, 0xFF, 0xFF]);
}

#[test]
fn cache_constructor_allocates_once() {
    let mut h = Fake { loads: vec![] };
    let mut cache = None;
    let b = cache_create(&mut cache, 2, 0, &mut h).unwrap();
    assert_eq!(h.loads, [0x241, 0x242, 0x243, 0x248, 0x248]);
    assert_eq!(b.recs.len(), 2);
    assert_eq!(cache, Some(MoveCache::new(2)));
    // a second call finds the cache and does nothing
    assert!(cache_create(&mut cache, 5, 0, &mut h).is_none());
    assert_eq!(h.loads.len(), 5);
}
