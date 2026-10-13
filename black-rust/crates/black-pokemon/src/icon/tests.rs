use super::*;
use black_fndiff::{run_suite, Arg, Case, StubEnv};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0201BA68, "8b3d763eaf590229"),
    (0x0201BABC, "4a5c370cdace61e7"),
    (0x0201BB5C, "45363505c909eea8"),
    (0x0201BBE8, "ccbad3ad58fba08f"),
    (0x0201BC3C, "c13e290302aab585"),
    (0x0201BC40, "145e08e173cd3b3f"),
    (0x0201BC78, "9b69447b315a9a39"),
    (0x0201BCB4, "fc0c101203b04322"),
    (0x0201BCEC, "bed6ced0be4c1938"),
];
// fndiff-recorded-end

/// The callees outside the area, routed to the spec's stub lines (same model as the ROM side).
struct Rom<'a> {
    env: &'a mut StubEnv,
}

impl IconHooks for Rom<'_> {
    fn arc_member_size(&mut self, arc: u32, member: u32) -> u32 {
        self.env
            .call(0x0204_90A8, &mut [Arg::V(arc), Arg::V(member)])
    }
}

impl PokeToolHooks for Rom<'_> {
    fn gender(&mut self, species: u32, form: u32, pid: u32) -> u32 {
        self.env.call(
            0x0201_7F50,
            &mut [Arg::V(species), Arg::V(form), Arg::V(pid)],
        )
    }
    fn personal_param(&mut self, species: u32, form: u32, param: u32) -> u32 {
        self.env.call(
            0x0201_9C90,
            &mut [Arg::V(species), Arg::V(form), Arg::V(param)],
        )
    }
    fn level_from_exp(&mut self, _: u32, _: u32, _: u32) -> u32 {
        unreachable!("not read by the icon routines")
    }
    fn move_max_pp(&mut self, _: u32, _: u32) -> u32 {
        unreachable!("not read by the icon routines")
    }
    fn strbuf_from_mon(&mut self, _: u32, _: usize) {
        unreachable!("not read by the icon routines")
    }
    fn strbuf_to_mon(&mut self, _: u32, _: usize, _: u32) {
        unreachable!("not read by the icon routines")
    }
    fn u16str_from_mon(&mut self, _: usize, _: u32, _: u32) {
        unreachable!("not read by the icon routines")
    }
    fn u16str_to_mon(&mut self, _: u32, _: usize, _: u32) {
        unreachable!("not read by the icon routines")
    }
    fn bad_egg_name_strbuf(&mut self, _: u32) {
        unreachable!("not read by the icon routines")
    }
    fn bad_egg_name_u16(&mut self, _: u32, _: u32) {
        unreachable!("not read by the icon routines")
    }
    fn species_name_to_scratch(&mut self, _: u32) {
        unreachable!("not read by the icon routines")
    }
    fn scratch_equals_mon_string(&mut self, _: usize) -> u32 {
        unreachable!("not read by the icon routines")
    }
    fn copy_ext_out(&mut self, _: usize, _: u32) {
        unreachable!("not read by the icon routines")
    }
    fn copy_ext_in(&mut self, _: u32, _: usize) {
        unreachable!("not read by the icon routines")
    }
}

fn reg(c: &Case) -> u32 {
    u32::from_le_bytes(c.globals[0][..4].try_into().unwrap())
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    let r = match addr {
        0x0201_BC3C => palette_member(),
        0x0201_BC40 | 0x0201_BC78 => cell_member(reg(c)),
        0x0201_BCB4 | 0x0201_BCEC => anim_member(reg(c)),
        0x0201_BABC => graphics_member(v[0], v[1], v[2], v[3], &mut Rom { env: &mut c.env }),
        0x0201_BB5C => {
            let table = c.globals[0].clone();
            palette_of(v[0], v[1], v[2], v[3], &table, &mut Rom { env: &mut c.env })
        }
        0x0201_BA68 => {
            let mut mon = c.region("mon").to_vec();
            let r = mon_graphics_member(&mut mon, &mut Rom { env: &mut c.env });
            c.region_mut("mon").copy_from_slice(&mon);
            r
        }
        0x0201_BBE8 => {
            let table = c.globals[0].clone();
            let mut mon = c.region("mon").to_vec();
            let r = mon_palette(&mut mon, &table, &mut Rom { env: &mut c.env });
            c.region_mut("mon").copy_from_slice(&mon);
            r
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    };
    r as u64
}

/// Every ported function against the original (`tools/check-fn.sh black-pokemon icon`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &black_fndiff::Suite::new(
            "icon",
            include_str!("../../../../tools/rom-diff/specs/black-pokemon/icon.spec"),
            PORTED,
            RECORDED,
        ),
        &call,
    );
}

/// Personal data with forms for species 479 (first form index 30, 6 forms) and none elsewhere; every
/// female icon exists.
struct Fake;

impl IconHooks for Fake {
    fn arc_member_size(&mut self, arc: u32, _member: u32) -> u32 {
        assert_eq!(arc, ICON_ARC);
        0x200
    }
}

impl PokeToolHooks for Fake {
    fn personal_param(&mut self, species: u32, _form: u32, field: u32) -> u32 {
        match (species, field) {
            (479, personal_field::FIRST_FORM) => 30,
            (479, personal_field::FORM_COUNT) => 6,
            (_, personal_field::FORM_COUNT) => 1,
            _ => 0,
        }
    }
    fn gender(&mut self, _: u32, _: u32, _: u32) -> u32 {
        0
    }
    fn level_from_exp(&mut self, _: u32, _: u32, _: u32) -> u32 {
        0
    }
    fn move_max_pp(&mut self, _: u32, _: u32) -> u32 {
        0
    }
    fn strbuf_from_mon(&mut self, _: u32, _: usize) {}
    fn strbuf_to_mon(&mut self, _: u32, _: usize, _: u32) {}
    fn u16str_from_mon(&mut self, _: usize, _: u32, _: u32) {}
    fn u16str_to_mon(&mut self, _: u32, _: usize, _: u32) {}
    fn bad_egg_name_strbuf(&mut self, _: u32) {}
    fn bad_egg_name_u16(&mut self, _: u32, _: u32) {}
    fn species_name_to_scratch(&mut self, _: u32) {}
    fn scratch_equals_mon_string(&mut self, _: usize) -> u32 {
        0
    }
    fn copy_ext_out(&mut self, _: usize, _: u32) {}
    fn copy_ext_in(&mut self, _: u32, _: usize) {}
}

#[test]
fn graphics_members() {
    let h = &mut Fake;
    // Bulbasaur: 1 * 2 + 7, the female variant one later
    assert_eq!(graphics_member(1, 0, 0, 0, h), 9);
    assert_eq!(graphics_member(1, 0, 1, 0, h), 10);
    assert_eq!(graphics_member(1, 0, 2, 0, h), 9);
    // eggs: 0x28A * 2 + 7, Manaphy's egg the next pair
    assert_eq!(graphics_member(25, 0, 0, 1, h), 1307);
    assert_eq!(graphics_member(MANAPHY, 0, 0, 1, h), 1309);
    // forms 1..5 of 479 from 0x51F + (30 + form - 1) * 2; form 6 is past the count: the base icon
    assert_eq!(graphics_member(479, 1, 0, 0, h), 0x51F + 60);
    assert_eq!(graphics_member(479, 6, 0, 0, h), 479 * 2 + 7);
    // a form's female variant
    assert_eq!(graphics_member(479, 5, 1, 0, h), 0x51F + 68 + 1);
}

#[test]
fn palettes_and_display_modes() {
    let h = &mut Fake;
    let mut t = [0u8; PALETTE_TABLE_LEN];
    t[1] = 0x21;
    t[0x28B] = 0x02;
    t[0x28B + 31] = 0x10;
    assert_eq!(palette_of(1, 0, 0, 0, &t, h), 1);
    assert_eq!(palette_of(1, 0, 1, 0, &t, h), 2);
    assert_eq!(palette_of(MANAPHY, 0, 0, 1, &t, h), 2);
    assert_eq!(palette_of(479, 1, 1, 0, &t, h), 1);
    assert_eq!(palette_member(), 0);
    assert_eq!(
        [0x10, 0x0010_0010, 0x0020_0010, 0x0030_0010, 0].map(cell_member),
        [2, 4, 6, 6, 6]
    );
    assert_eq!(
        [0x10, 0x0010_0010, 0x0020_0010, 0x0030_0010, 0].map(anim_member),
        [1, 3, 5, 5, 5]
    );
}
