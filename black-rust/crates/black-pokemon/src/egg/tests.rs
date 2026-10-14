use super::*;
use black_fndiff::{run_suite, Arg, Case, StubEnv};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021C38F4, "2f41df386f2c1b2a"),
    (0x021C3A00, "1477de52cba791a0"),
    (0x021C3A44, "d26172020cbfa445"),
    (0x021C3A70, "87fe32d72ca97499"),
    (0x021C3A7C, "da6da04e232b6e25"),
    (0x021C3A90, "c884c88dff23d524"),
    (0x021C3AAC, "fb134c57053248a5"),
    (0x021C3ADC, "9b15f298a0d311fb"),
    (0x021C3B0C, "f8a96bf0b1ee85dc"),
    (0x021C3B2C, "e7e722c674f14b2d"),
    (0x021C3BB0, "563542d78a0291e0"),
    (0x021C3BCC, "86c67ff07ba24197"),
    (0x021C3BD4, "0ee21b969605c148"),
    (0x021C3C04, "6298dceabf044ce1"),
    (0x021C3C30, "f7750013a5d3685b"),
    (0x021C3C5C, "794159f3afd1e6f7"),
    (0x021C3C78, "56c902713306a523"),
    (0x021C3C88, "d22c387075f95c14"),
    (0x021C3D1C, "f570956e284c0629"),
    (0x021C3DA8, "619c5836f09425bb"),
    (0x021C3DE4, "0d41b39fa33477e6"),
    (0x021C3E38, "10fb73aaa98f9c0d"),
    (0x021C3E64, "1d5ff785d4741335"),
    (0x021C3F60, "eff7a41e7c483c81"),
    (0x021C3FC0, "e5e0c1719f2fe15d"),
    (0x021C4018, "48fbc3e49c669c2a"),
    (0x021C4024, "dc217e7b8bd45180"),
    (0x021C4030, "c4fdbb9e6b941d6e"),
    (0x021C4078, "64e28f5165b9a39d"),
    (0x021C4084, "06efe401b56ddf45"),
    (0x021C40F8, "d6800a4988b8d107"),
    (0x021C4160, "cb293f07ab776ec1"),
    (0x021C4238, "6c429c5c1c89ea45"),
    (0x021C42BC, "5ad86098c7cb668d"),
    (0x021C42F4, "473a593dca4d598f"),
    (0x021C4340, "ec09ff527c2c9e2d"),
    (0x021C447C, "6404b16b38493592"),
    (0x021C4504, "c98a04ff3d2f2f0e"),
    (0x021C452C, "7424baa021917e7a"),
];
// fndiff-recorded-end

/// The callees outside the area, routed to the spec's stub lines (same model as the ROM side).
struct Rom<'a> {
    env: &'a mut StubEnv,
    /// The egg-move block the `FUN_02048F60` stub points into, and its address.
    block: Vec<u8>,
    block_base: u32,
    /// The `learnset_end` global.
    end: u32,
}

impl EggHooks for Rom<'_> {
    fn get(&mut self, mon: u32, id: u32) -> u32 {
        self.env
            .call(0x0201_7E1C, &mut [Arg::V(mon), Arg::V(id), Arg::V(0)])
    }
    fn put(&mut self, mon: u32, id: u32, value: u32) {
        self.env
            .call(0x0201_7E40, &mut [Arg::V(mon), Arg::V(id), Arg::V(value)]);
    }
    fn rand(&mut self, max: u32) -> u32 {
        self.env.call(0x0200_5728, &mut [Arg::V(max)])
    }
    fn alloc_record(&mut self, species: u32, form: u32, heap: u32) -> u32 {
        self.env.call(
            0x0201_ADE4,
            &mut [Arg::V(species), Arg::V(form), Arg::V(heap)],
        )
    }
    fn record_field(&mut self, record: u32, field: u32) -> u32 {
        self.env
            .call(0x0201_AE1C, &mut [Arg::V(record), Arg::V(field)])
    }
    fn free_block(&mut self, block: u32) {
        self.env.call(0x0203_07B0, &mut [Arg::V(block)]);
    }
    fn read_learnset(&mut self, species: u32, form: u32, dst: &mut [u8]) {
        self.env.call(
            0x0201_ADD0,
            &mut [Arg::V(species), Arg::V(form), Arg::Out(dst)],
        );
    }
    fn learnset_end(&mut self) -> u32 {
        self.end
    }
    fn arc_alloc(&mut self, arc: u32, member: u32, heap: u32) -> u32 {
        self.env.call(
            0x0204_8F60,
            &mut [Arg::V(arc), Arg::V(member), Arg::V(heap)],
        )
    }
    fn block_u16(&mut self, block: u32, index: u32) -> u32 {
        let o = (block - self.block_base + 2 * index) as usize;
        u32::from(u16::from_le_bytes([self.block[o], self.block[o + 1]]))
    }
    fn arc_u16(&mut self, arc: u32, member: u32, offset: u32) -> u32 {
        let mut b = [0u8; 2];
        self.env.call(
            0x0204_8F74,
            &mut [
                Arg::Out(&mut b),
                Arg::V(arc),
                Arg::V(member),
                Arg::V(offset),
                Arg::V(2),
            ],
        );
        u32::from(u16::from_le_bytes(b))
    }
    fn item_is_tm(&mut self, item: u32) -> u32 {
        self.env.call(0x0202_1288, &mut [Arg::V(item)])
    }
    fn tm_index(&mut self, item: u32) -> u32 {
        self.env.call(0x0202_130C, &mut [Arg::V(item)]) & 0xFF
    }
    fn tm_move(&mut self, item: u32) -> u32 {
        self.env.call(0x0202_12B0, &mut [Arg::V(item)]) & 0xFFFF
    }
    fn trainer_id(&mut self, profile: u32) -> u32 {
        self.env.call(0x0200_8530, &mut [Arg::V(profile)])
    }
    fn init_pokemon(
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
    fn set_hidden_ability(&mut self, mon: u32, species: u32, form: u32) {
        self.env.call(
            0x0201_8A08,
            &mut [Arg::V(mon), Arg::V(species), Arg::V(form)],
        );
    }
    fn trainer_name(&mut self, profile: u32, heap: u32) -> u32 {
        self.env
            .call(0x0200_8510, &mut [Arg::V(profile), Arg::V(heap)])
    }
    fn free_strbuf(&mut self, strbuf: u32) {
        self.env.call(0x0204_5808, &mut [Arg::V(strbuf)]);
    }
    fn message(&mut self, message: u32) -> u32 {
        self.env.call(0x0204_5EC0, &mut [Arg::P, Arg::V(message)])
    }
    fn set_met_data(&mut self, mon: u32, kind: u32, profile: u32, place: u32, heap: u32) {
        self.env.call(
            0x0202_F190,
            &mut [
                Arg::V(mon),
                Arg::V(kind),
                Arg::V(profile),
                Arg::V(place),
                Arg::V(heap),
            ],
        );
    }
    fn restore_pp(&mut self, mon: u32) {
        self.env.call(0x0201_81A0, &mut [Arg::V(mon)]);
    }
    fn recalc(&mut self, mon: u32) {
        self.env.call(0x0201_85C4, &mut [Arg::V(mon)]);
    }
    fn restore_mon(&mut self, mon: u32) {
        self.env.call(0x0201_B9F0, &mut [Arg::V(mon)]);
    }
}

fn words<const N: usize>(b: &[u8]) -> [u32; N] {
    let mut out = [0u32; N];
    for (i, w) in out.iter_mut().enumerate() {
        *w = u32::from_le_bytes(b[i * 4..i * 4 + 4].try_into().unwrap());
    }
    out
}

fn put_words(b: &mut [u8], w: &[u32]) {
    for (i, v) in w.iter().enumerate() {
        b[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
    }
}

/// The driver and the egg-move function read the `buf` argument `block_arg` the ARC stub points into.
fn rom<'a>(c: &'a mut Case, block_arg: Option<usize>) -> (Rom<'a>, Vec<u32>) {
    let v = c.vals.clone();
    let end = c.globals.first().map_or(0xFFFF, |g| words::<1>(g)[0]);
    let (block, block_base) = match block_arg {
        Some(k) => (c.bufs[k].clone(), v[k]),
        None => (Vec::new(), 0),
    };
    (
        Rom {
            env: &mut c.env,
            block,
            block_base,
            end,
        },
        v,
    )
}

fn call(addr: u32, c: &mut Case) -> u64 {
    match addr {
        // (par) / (par, egg[, heap[, block]]): regions 0 and 1
        0x021C_3A00 | 0x021C_3A44 | 0x021C_3A70 | 0x021C_3BB0 | 0x021C_3C5C | 0x021C_3A90
        | 0x021C_3AAC | 0x021C_3ADC | 0x021C_3B0C | 0x021C_3B2C | 0x021C_3BD4 | 0x021C_3C04
        | 0x021C_3C30 | 0x021C_3C88 | 0x021C_3D1C | 0x021C_3DA8 | 0x021C_3E64 | 0x021C_3FC0
        | 0x021C_4030 | 0x021C_40F8 | 0x021C_4160 | 0x021C_4238 | 0x021C_42BC => {
            let mut par: Parents = words(&c.regions[0]);
            let has_egg = !matches!(
                addr,
                0x021C_3A00 | 0x021C_3A44 | 0x021C_3A70 | 0x021C_3BB0 | 0x021C_3C5C
            );
            let mut egg: Egg = if has_egg {
                words(&c.regions[1])
            } else {
                [0; 35]
            };
            let block_arg = (addr == 0x021C_40F8).then_some(3);
            let (mut h, v) = rom(c, block_arg);
            let mut ret = 0;
            match addr {
                0x021C_3A00 => order_by_sex(&mut par, &mut h),
                0x021C_3A44 => order_ditto(&mut par, &mut h),
                0x021C_3A70 => swap_parents(&mut par),
                0x021C_3BB0 | 0x021C_3C5C => ret = base_parent(&par, &mut h) as u64,
                0x021C_3A90 => set_baby_species(&par, &mut egg, &mut h),
                0x021C_3AAC => nidoran_gender(&par, &mut egg, &mut h),
                0x021C_3ADC => volbeat_illumise(&par, &mut egg, &mut h),
                0x021C_3B0C => phione(&par, &mut egg, &mut h),
                0x021C_3B2C => incense_baby(&par, &mut egg, &mut h),
                0x021C_3BD4 => form_shellos(&par, &mut egg, &mut h),
                0x021C_3C04 => form_burmy(&par, &mut egg, &mut h),
                0x021C_3C30 => form_basculin(&par, &mut egg, &mut h),
                0x021C_3C88 => everstone_nature(&par, &mut egg, &mut h),
                0x021C_3D1C => roll_ability(&par, &mut egg, v[2], &mut h),
                0x021C_3DA8 => ditto_ability(&par, &mut egg, &mut h),
                0x021C_3E64 => power_item_ivs(&par, &mut egg, &mut h),
                0x021C_3FC0 => inherit_ivs(&par, &mut egg, &mut h),
                0x021C_4030 => masuda(&par, &mut egg, &mut h),
                0x021C_40F8 => egg_moves(&par, &mut egg, v[2], &mut h),
                0x021C_4160 => common_moves(&par, &mut egg, &mut h),
                0x021C_4238 => tm_moves(&par, &mut egg, v[2], &mut h),
                _ => volt_tackle(&par, &mut egg, &mut h),
            }
            put_words(&mut c.regions[0], &par);
            if has_egg {
                put_words(&mut c.regions[1], &egg);
            }
            ret
        }
        // (x, egg[, ...]): egg is region 0
        0x021C_3A7C | 0x021C_3BCC | 0x021C_3C78 | 0x021C_3DE4 | 0x021C_3E38 | 0x021C_3F60
        | 0x021C_4018 | 0x021C_4024 | 0x021C_4084 | 0x021C_42F4 | 0x021C_4340 => {
            let mut egg: Egg = words(&c.regions[0]);
            let (mut h, v) = rom(c, None);
            match addr {
                0x021C_3A7C => init_egg(&mut egg),
                0x021C_3BCC => clear_form(&mut egg),
                0x021C_3C78 => roll_nature(&mut egg, &mut h),
                0x021C_3DE4 => pick_ability(&mut egg, v[2], &mut h),
                0x021C_3E38 => egg_cycles(&mut egg, v[1], &mut h),
                0x021C_3F60 => random_ivs(&mut egg, &mut h),
                0x021C_4018 => set_ot_id(v[0], &mut egg, &mut h),
                0x021C_4024 => roll_pid(&mut egg, &mut h),
                0x021C_4084 => level1_moves(&mut egg, &mut h),
                0x021C_42F4 => add_move(&mut egg, v[1]),
                _ => build_mon(v[0], &egg, v[2], v[3], v[4], &mut h),
            }
            put_words(&mut c.regions[0], &egg);
            0
        }
        0x021C_4078 => {
            let (mut h, _) = rom(c, None);
            rand32(&mut h) as u64
        }
        0x021C_447C => {
            let (mut h, v) = rom(c, None);
            can_learn_tm(v[0], v[1], v[2], v[3], &mut h) as u64
        }
        0x021C_4504 => {
            let (mut h, v) = rom(c, None);
            baby_species(v[0], &mut h) as u64
        }
        0x021C_452C => {
            let (mut h, v) = rom(c, None);
            ability_slot(v[0], v[1], v[2], v[3], &mut h) as u64
        }
        0x021C_38F4 => {
            let (mut h, v) = rom(c, Some(6));
            generate(v[0], v[1], v[2], v[3], v[4], v[5], &mut h);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-pokemon egg`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &black_fndiff::Suite::new(
            "egg",
            include_str!("../../../../tools/rom-diff/specs/black-pokemon/egg.spec"),
            PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn add_move_dedups_and_shifts() {
    let mut e: Egg = [0; 35];
    for m in [0, 5, 5, 6, 7, 8] {
        add_move(&mut e, m);
    }
    assert_eq!(e[w::MOVE_COUNT], 4);
    assert_eq!(&e[w::MOVES..w::MOVES + 4], &[5, 6, 7, 8]);
    // a full list drops its first move
    add_move(&mut e, 9);
    assert_eq!(&e[w::MOVES..w::MOVES + 4], &[6, 7, 8, 9]);
    assert_eq!(e[w::MOVE_COUNT], 4);
}

#[test]
fn init_egg_sets_level_one() {
    let mut e: Egg = [7; 35];
    init_egg(&mut e);
    assert_eq!(e[w::LEVEL], 1);
    assert_eq!(e.iter().sum::<u32>(), 1);
}
