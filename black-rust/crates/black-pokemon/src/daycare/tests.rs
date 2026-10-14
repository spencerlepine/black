use super::*;
use black_fndiff::{run_suite, Arg, Case, StubEnv};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0200BDB0, "fca582887c7d706e"),
    (0x021C31D4, "5134c888f16a7415"),
    (0x021C31FC, "9b9d16ee5d175e3a"),
    (0x021C3204, "668d6f88e7310f93"),
    (0x021C3248, "4f34bdce6a4ff519"),
    (0x021C3250, "afe797c6cedc556d"),
    (0x021C3270, "81edd3b08d48be1a"),
    (0x021C32BC, "81c2e96d6571900d"),
    (0x021C32F4, "9bf0e2eb99e21b74"),
    (0x021C335C, "37087b5d4753c477"),
    (0x021C3378, "dc30b9665484d197"),
    (0x021C33A4, "a25eda3cff98f070"),
    (0x021C33E0, "6b382fae47f26fdd"),
    (0x021C3400, "3e04ba3b7bce0e4a"),
    (0x021C340C, "5aea962809899364"),
    (0x021C341C, "7fc227823168f8cb"),
    (0x021C3490, "802a91ec45a6fe53"),
    (0x021C34B4, "609c010237ad368d"),
    (0x021C34C4, "7d555c0376e84ecb"),
    (0x021C34F8, "b48f862252ff6038"),
    (0x021C3510, "7a499fa541c93f00"),
    (0x021C3530, "c47429d6e304ac3e"),
    (0x021C360C, "d67f0fcad742b3e2"),
    (0x021C3670, "6408b7936d13254c"),
    (0x021C36B0, "9e9bbc921049c742"),
    (0x021C36EC, "09b2dac73f66e84b"),
    (0x021C37F8, "a447900deb6820d4"),
    (0x021C385C, "3067a64c89c884a2"),
    (0x021C3898, "893d36c92c7acc65"),
    (0x021C38DC, "3bf7de0abda17b07"),
];
// fndiff-recorded-end

/// The callees outside the area, routed to the spec's stub lines (same model as the ROM side). `bufs` /
/// `vals` are the case's pointer arguments: the event-work var lives there.
struct Rom<'a> {
    env: &'a mut StubEnv,
    bufs: &'a mut [Vec<u8>],
    vals: &'a [u32],
}

impl Rom<'_> {
    fn v(&mut self, addr: u32, args: &[u32]) -> u32 {
        let mut a: Vec<Arg> = args.iter().map(|&x| Arg::V(x)).collect();
        self.env.call(addr, &mut a)
    }
}

impl DaycareHooks for Rom<'_> {
    fn save_set_egg(&mut self, save: u32) {
        self.v(0x0200_BC10, &[save]);
    }
    fn save_clear_egg(&mut self, save: u32) {
        self.v(0x0200_BC20, &[save]);
    }
    fn save_mon(&mut self, save: u32, slot: u32) -> u32 {
        self.v(0x0200_BC2C, &[save, slot])
    }
    fn save_put(&mut self, save: u32, slot: u32, mon: u32) {
        self.v(0x0200_BC4C, &[save, slot, mon]);
    }
    fn save_take(&mut self, save: u32, slot: u32) {
        self.v(0x0200_BC88, &[save, slot]);
    }
    fn save_egg(&mut self, save: u32) -> u32 {
        self.v(0x0200_BCA0, &[save])
    }
    fn save_occupied(&mut self, save: u32, slot: u32) -> u32 {
        self.v(0x0200_BCA8, &[save, slot])
    }
    fn save_count(&mut self, save: u32) -> u32 {
        self.v(0x0200_BCB8, &[save])
    }
    fn save_exp(&mut self, save: u32, slot: u32) -> u32 {
        self.v(0x0200_BCD8, &[save, slot])
    }
    fn save_swap(&mut self, save: u32, a: u32, b: u32) {
        self.v(0x0200_BCEC, &[save, a, b]);
    }
    fn save_add_exp(&mut self, save: u32, n: u32) {
        self.v(0x0200_BD48, &[save, n]);
    }
    fn get(&mut self, mon: u32, id: u32) -> u32 {
        self.v(0x0201_7E1C, &[mon, id, 0])
    }
    fn put(&mut self, mon: u32, id: u32, value: u32) {
        self.v(0x0201_7E40, &[mon, id, value]);
    }
    fn party_mon(&mut self, party: u32, index: u32) -> u32 {
        self.v(0x0201_AB28, &[party, index])
    }
    fn party_remove(&mut self, party: u32, index: u32) {
        self.v(0x0201_A9C8, &[party, index]);
    }
    fn party_add(&mut self, party: u32, mon: u32) {
        self.v(0x0201_A98C, &[party, mon]);
    }
    fn party_count(&mut self, party: u32) -> u32 {
        self.v(0x0201_AA18, &[party])
    }
    fn party_max(&mut self, party: u32) -> u32 {
        self.v(0x0201_AA14, &[party])
    }
    fn restore_pp(&mut self, mon: u32) {
        self.v(0x0201_B9F0, &[mon]);
    }
    fn set_form(&mut self, mon: u32, form: u32) {
        self.v(0x0201_7988, &[mon, form]);
    }
    fn exp_for_level(&mut self, species: u32, form: u32, level: u32) -> u32 {
        self.v(0x0201_8584, &[species, form, level])
    }
    fn level_from_exp(&mut self, species: u32, form: u32, exp: u32) -> u32 {
        self.v(0x0201_850C, &[species, form, exp])
    }
    fn recalc(&mut self, mon: u32) {
        self.v(0x0201_85C4, &[mon]);
    }
    fn load_learnset(&mut self, species: u32, form: u32, out: &mut [u8; LEARNSET_BYTES]) {
        self.env.call(
            0x0201_ADD0,
            &mut [Arg::V(species), Arg::V(form), Arg::Out(&mut out[..])],
        );
    }
    fn set_move(&mut self, mon: u32, mv: u32, slot: u32) {
        self.v(0x0201_81FC, &[mon, mv, slot]);
    }
    fn push_move(&mut self, mon: u32, mv: u32) {
        self.v(0x0201_80FC, &[mon, mv]);
    }
    fn personal(&mut self, species: u32, form: u32, field: u32) -> u32 {
        self.v(0x0201_9C90, &[species, form, field])
    }
    fn rand(&mut self, n: u32) -> u32 {
        self.v(0x0200_5728, &[n])
    }
    fn field_gamesys(&mut self, fm: u32) -> u32 {
        self.v(0x0218_8C80, &[fm])
    }
    fn gamedata(&mut self, gs: u32) -> u32 {
        self.v(0x0201_20F4, &[gs])
    }
    fn event_work(&mut self, gd: u32) -> u32 {
        self.v(0x0201_296C, &[gd])
    }
    fn party(&mut self, gd: u32) -> u32 {
        self.v(0x0201_2934, &[gd])
    }
    fn pokedex(&mut self, gd: u32) -> u32 {
        self.v(0x0200_CA1C, &[gd])
    }
    fn profile(&mut self, gd: u32) -> u32 {
        self.v(0x0201_2944, &[gd])
    }
    fn gamedata_02012f98(&mut self, gd: u32) -> u32 {
        self.v(0x0201_2F98, &[gd])
    }
    fn season(&mut self, gd: u32) -> u32 {
        self.v(0x0201_2984, &[gd])
    }
    fn party_changed(&mut self, x: u32, party: u32) {
        self.v(0x0200_7C34, &[x, party]);
    }
    fn set_work_var(&mut self, ev: u32, id: u32, value: u16) {
        let p = self.v(0x0201_43CC, &[ev, id]);
        let i = (0..self.bufs.len())
            .find(|&i| {
                let b = &self.bufs[i];
                !b.is_empty() && p >= self.vals[i] && ((p - self.vals[i]) as usize) < b.len()
            })
            .expect("var address in no buffer argument");
        let o = (p - self.vals[i]) as usize;
        self.bufs[i][o..o + 2].copy_from_slice(&value.to_le_bytes());
    }
    fn dex_has(&mut self, dex: u32, species: u32) -> u32 {
        self.v(0x0200_CED4, &[dex, species])
    }
    fn dex_register(&mut self, dex: u32, mon: u32) {
        self.v(0x0200_CDE0, &[dex, mon]);
    }
    fn new_pokemon(&mut self, a: u32, b: u32, c: u32, d: u32, heap: u32) -> u32 {
        self.v(0x0201_73F0, &[a, b, c, d, heap])
    }
    fn alloc(&mut self, heap: u32, size: u32, flag: u32, file: u32, line: u32) -> u32 {
        self.v(0x0203_0734, &[heap, size, flag, file, line])
    }
    fn free(&mut self, block: u32) {
        self.v(0x0203_07B0, &[block]);
    }
    fn egg_received(&mut self) {
        self.v(0x0216_556C, &[]);
    }
    fn seasonal_forms(&mut self, gd: u32, party: u32, season: u32) {
        self.v(0x0216_AAE4, &[gd, party, season]);
    }
    fn generate_egg(&mut self, heap: u32, profile: u32, place: u32, p0: u32, p1: u32, egg: u32) {
        self.v(0x021C_38F4, &[heap, profile, place, p0, p1, egg]);
    }
    fn save_block(&mut self, save: u32, id: u32) -> u32 {
        self.v(0x0200_71CC, &[save, id])
    }
}

fn r16(b: &[u8], o: usize) -> u16 {
    u16::from_le_bytes([b[o], b[o + 1]])
}

fn r32(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}

/// The object image at the start of a buffer.
fn read_obj(b: &[u8]) -> DaycareObj {
    DaycareObj {
        heap: r16(b, 0),
        field: r32(b, 4),
        save: r32(b, 8),
        counter: r16(b, 0xC),
    }
}

fn write_obj(b: &mut [u8], dc: &DaycareObj) {
    b[0..2].copy_from_slice(&dc.heap.to_le_bytes());
    b[4..8].copy_from_slice(&dc.field.to_le_bytes());
    b[8..12].copy_from_slice(&dc.save.to_le_bytes());
    b[0xC..0xE].copy_from_slice(&dc.counter.to_le_bytes());
}

/// Run a function with the case's stubs and buffers (no object).
fn free_fn(c: &mut Case, f: impl FnOnce(&[u32], &mut Rom) -> u64) -> u64 {
    let vals = c.vals.clone();
    let mut h = Rom {
        env: &mut c.env,
        bufs: &mut c.bufs,
        vals: &vals,
    };
    f(&vals, &mut h)
}

/// Run an object function (the object is arg0, `arg buf 0x10`) and write the object back.
fn obj_fn(c: &mut Case, f: impl FnOnce(&mut DaycareObj, &[u32], &mut Rom) -> u64) -> u64 {
    let mut dc = read_obj(&c.bufs[0]);
    let r = free_fn(c, |v, h| f(&mut dc, v, h));
    write_obj(&mut c.bufs[0], &dc);
    r
}

fn call(addr: u32, c: &mut Case) -> u64 {
    match addr {
        0x0200_BDB0 => free_fn(c, |v, h| u64::from(save_block(v[0], h))),
        0x021C_31D4 => {
            let (block, dc) = free_fn_pair(c);
            let base = c.vals[3];
            write_obj(&mut c.bufs[3][(block - base) as usize..], &dc);
            u64::from(block - base)
        }
        0x021C_31FC => free_fn(c, |v, h| {
            delete(v[0], h);
            0
        }),
        0x021C_3204 => obj_fn(c, |dc, v, h| {
            deposit(dc, v[1], v[2], h);
            0
        }),
        0x021C_3248 => free_fn(c, |v, h| {
            restore_pp(v[0], h);
            0
        }),
        0x021C_3250 => free_fn(c, |v, h| {
            reset_form(v[0], h);
            0
        }),
        0x021C_3270 => obj_fn(c, |dc, v, h| {
            withdraw(dc, v[1], v[2], h);
            0
        }),
        0x021C_32BC => obj_fn(c, |dc, v, h| {
            register_dex(dc, v[1], h);
            0
        }),
        0x021C_32F4 => obj_fn(c, |dc, v, h| {
            take_egg(dc, v[1], h);
            0
        }),
        0x021C_335C => obj_fn(c, |dc, _, h| {
            refuse_egg(dc, h);
            0
        }),
        0x021C_3378 => obj_fn(c, |dc, _, h| u64::from(step(dc, h))),
        0x021C_33A4 => free_fn(c, |v, h| {
            set_egg_var(v[0], h);
            0
        }),
        0x021C_33E0 => free_fn(c, |v, h| {
            clear_egg_var(v[0], h);
            0
        }),
        0x021C_3400 => obj_fn(c, |dc, _, h| u64::from(egg_ready(dc, h))),
        0x021C_340C => obj_fn(c, |dc, _, h| u64::from(count(dc, h))),
        0x021C_341C => obj_fn(c, |dc, v, h| u64::from(level_now(dc, v[1], h))),
        0x021C_3490 => obj_fn(c, |dc, v, h| u64::from(levels_gained(dc, v[1], h))),
        0x021C_34B4 => obj_fn(c, |dc, v, h| u64::from(fee(dc, v[1], h))),
        0x021C_34C4 => obj_fn(c, |dc, _, h| u64::from(compatibility(dc, h))),
        0x021C_34F8 => obj_fn(c, |dc, v, h| u64::from(slot_mon(dc, v[1], h))),
        0x021C_3510 => u64::from(sat_add(c.vals[0], c.vals[1])),
        0x021C_3530 => free_fn(c, |v, h| {
            apply_exp(v[0], v[1], h);
            0
        }),
        0x021C_360C => free_fn(c, |v, h| {
            learn_move(v[0], v[1], h);
            0
        }),
        0x021C_3670 => free_fn(c, |v, h| u64::from(knows_move(v[0], v[1], h))),
        0x021C_36B0 => free_fn(c, |v, h| {
            compact(v[0], h);
            0
        }),
        0x021C_36EC => free_fn(c, |v, h| u64::from(compat_pair(v[0], v[1], h))),
        0x021C_37F8 => free_fn(c, |v, h| u64::from(compat_score(v[0], v[1], h))),
        0x021C_385C => obj_fn(c, |dc, _, h| u64::from(egg_roll(dc, h))),
        0x021C_3898 => obj_fn(c, |dc, v, h| {
            make_egg(dc, v[1], h);
            0
        }),
        0x021C_38DC => free_fn(c, |v, h| {
            party_changed(v[0], v[1], h);
            0
        }),
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// [`create`] with the case's arguments.
fn free_fn_pair(c: &mut Case) -> (u32, DaycareObj) {
    let vals = c.vals.clone();
    let mut h = Rom {
        env: &mut c.env,
        bufs: &mut c.bufs,
        vals: &vals,
    };
    create(vals[0], vals[1], vals[2], &mut h)
}

/// Every ported function against the original (`tools/check-fn.sh black-pokemon daycare`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &black_fndiff::Suite::new(
            "daycare",
            include_str!("../../../../tools/rom-diff/specs/black-pokemon/daycare.spec"),
            PORTED,
            RECORDED,
        ),
        &call,
    );
}

/// A scripted day care for the plain-Rust behaviour tests: two Pokemon (handles 0 / 1) given as (species,
/// OT id, gender, egg group 1, egg group 2), both slots occupied.
struct Pair {
    mons: [(u32, u32, u32, u32, u32); 2],
    rolls: Vec<u32>,
}

impl DaycareHooks for Pair {
    fn save_mon(&mut self, _: u32, slot: u32) -> u32 {
        slot.min(1)
    }
    fn save_count(&mut self, _: u32) -> u32 {
        2
    }
    fn get(&mut self, mon: u32, id: u32) -> u32 {
        let m = self.mons[mon as usize];
        match id {
            5 => m.0,
            7 => m.1,
            0x6E => m.2,
            _ => 0,
        }
    }
    fn personal(&mut self, species: u32, _form: u32, field: u32) -> u32 {
        let m = self.mons.iter().find(|m| m.0 == species).unwrap();
        if field == 0x18 {
            m.3
        } else {
            m.4
        }
    }
    fn rand(&mut self, n: u32) -> u32 {
        self.rolls.push(n);
        0
    }
    fn save_set_egg(&mut self, _: u32) {}
    fn save_clear_egg(&mut self, _: u32) {}
    fn save_put(&mut self, _: u32, _: u32, _: u32) {}
    fn save_take(&mut self, _: u32, _: u32) {}
    fn save_egg(&mut self, _: u32) -> u32 {
        0
    }
    fn save_occupied(&mut self, _: u32, _: u32) -> u32 {
        1
    }
    fn save_exp(&mut self, _: u32, _: u32) -> u32 {
        0
    }
    fn save_swap(&mut self, _: u32, _: u32, _: u32) {}
    fn save_add_exp(&mut self, _: u32, _: u32) {}
    fn put(&mut self, _: u32, _: u32, _: u32) {}
    fn party_mon(&mut self, _: u32, _: u32) -> u32 {
        0
    }
    fn party_remove(&mut self, _: u32, _: u32) {}
    fn party_add(&mut self, _: u32, _: u32) {}
    fn party_count(&mut self, _: u32) -> u32 {
        0
    }
    fn party_max(&mut self, _: u32) -> u32 {
        6
    }
    fn restore_pp(&mut self, _: u32) {}
    fn set_form(&mut self, _: u32, _: u32) {}
    fn exp_for_level(&mut self, _: u32, _: u32, _: u32) -> u32 {
        0
    }
    fn level_from_exp(&mut self, _: u32, _: u32, _: u32) -> u32 {
        0
    }
    fn recalc(&mut self, _: u32) {}
    fn load_learnset(&mut self, _: u32, _: u32, out: &mut [u8; LEARNSET_BYTES]) {
        out.fill(0xFF);
    }
    fn set_move(&mut self, _: u32, _: u32, _: u32) {}
    fn push_move(&mut self, _: u32, _: u32) {}
    fn field_gamesys(&mut self, _: u32) -> u32 {
        0
    }
    fn gamedata(&mut self, _: u32) -> u32 {
        0
    }
    fn event_work(&mut self, _: u32) -> u32 {
        0
    }
    fn party(&mut self, _: u32) -> u32 {
        0
    }
    fn pokedex(&mut self, _: u32) -> u32 {
        0
    }
    fn profile(&mut self, _: u32) -> u32 {
        0
    }
    fn gamedata_02012f98(&mut self, _: u32) -> u32 {
        0
    }
    fn season(&mut self, _: u32) -> u32 {
        0
    }
    fn party_changed(&mut self, _: u32, _: u32) {}
    fn set_work_var(&mut self, _: u32, _: u32, _: u16) {}
    fn dex_has(&mut self, _: u32, _: u32) -> u32 {
        1
    }
    fn dex_register(&mut self, _: u32, _: u32) {}
    fn new_pokemon(&mut self, _: u32, _: u32, _: u32, _: u32, _: u32) -> u32 {
        0
    }
    fn alloc(&mut self, _: u32, _: u32, _: u32, _: u32, _: u32) -> u32 {
        0
    }
    fn free(&mut self, _: u32) {}
    fn egg_received(&mut self) {}
    fn seasonal_forms(&mut self, _: u32, _: u32, _: u32) {}
    fn generate_egg(&mut self, _: u32, _: u32, _: u32, _: u32, _: u32, _: u32) {}
    fn save_block(&mut self, _: u32, _: u32) -> u32 {
        0
    }
}

fn pair(mons: [(u32, u32, u32, u32, u32); 2]) -> Pair {
    Pair {
        mons,
        rolls: Vec::new(),
    }
}

#[test]
fn compatibility_table() {
    // Same species, same OT: 50; other OT: 70.
    assert_eq!(
        compat_pair(0, 1, &mut pair([(1, 7, 0, 1, 1), (1, 7, 1, 1, 1)])),
        50
    );
    assert_eq!(
        compat_pair(0, 1, &mut pair([(1, 7, 0, 1, 1), (1, 8, 1, 1, 1)])),
        70
    );
    // Different species sharing an egg group (group 2 of one, group 1 of the other): 20 (same OT) / 50.
    assert_eq!(
        compat_pair(0, 1, &mut pair([(1, 7, 0, 3, 4), (2, 7, 1, 4, 5)])),
        20
    );
    assert_eq!(
        compat_pair(0, 1, &mut pair([(1, 7, 0, 3, 4), (2, 9, 1, 4, 5)])),
        50
    );
    // No shared group, same gender, genderless, Undiscovered: 0.
    assert_eq!(
        compat_pair(0, 1, &mut pair([(1, 7, 0, 3, 4), (2, 7, 1, 5, 6)])),
        0
    );
    assert_eq!(
        compat_pair(0, 1, &mut pair([(1, 7, 1, 1, 1), (2, 7, 1, 1, 1)])),
        0
    );
    assert_eq!(
        compat_pair(0, 1, &mut pair([(1, 7, 2, 1, 1), (2, 7, 1, 1, 1)])),
        0
    );
    assert_eq!(
        compat_pair(0, 1, &mut pair([(1, 7, 0, 15, 15), (2, 7, 1, 1, 1)])),
        0
    );
    // Ditto pairs with a genderless Pokemon of another group, not with another Ditto.
    assert_eq!(
        compat_pair(0, 1, &mut pair([(DITTO, 7, 2, 13, 13), (2, 8, 2, 1, 1)])),
        50
    );
    assert_eq!(
        compat_pair(
            0,
            1,
            &mut pair([(DITTO, 7, 2, 13, 13), (DITTO, 8, 2, 13, 13)])
        ),
        0
    );
}

#[test]
fn egg_roll_every_sixteen_steps() {
    let mut p = pair([(1, 7, 0, 1, 1), (1, 8, 1, 1, 1)]);
    let mut dc = DaycareObj::default();
    // 16 steps raise the counter to 0x100; the 17th resets it and rolls rand(1600) (0 < 70: an egg).
    let eggs: Vec<u32> = (0..17).map(|_| egg_roll(&mut dc, &mut p)).collect();
    assert_eq!(eggs.iter().sum::<u32>(), 1);
    assert_eq!(eggs[16], 1);
    assert_eq!(dc.counter, 0);
    assert_eq!(p.rolls, [1600]);
}

#[test]
fn saturating_add() {
    assert_eq!(sat_add(u32::MAX - 1, 5), u32::MAX);
    assert_eq!(sat_add(3, 4), 7);
}
