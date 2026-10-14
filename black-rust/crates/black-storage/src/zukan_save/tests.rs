use super::*;
use crate::le::{r16s, r32, w32};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0200C720, "af1d6dc3167516ea"),
    (0x0200C738, "2f3b749314aff5f3"),
    (0x0200C780, "f31bf4540df6b369"),
    (0x0200C7A0, "f33264c6db37e075"),
    (0x0200C7C0, "f582ced62c03ed38"),
    (0x0200C7F4, "63e9ea6b3389426a"),
    (0x0200C818, "9bac50ce92390864"),
    (0x0200C914, "21b9b2729885cec3"),
    (0x0200C9EC, "90ed52862b973b55"),
    (0x0200C9F4, "200d0605b002d249"),
    (0x0200CA28, "1a7bbab95b6a6c35"),
    (0x0200CA38, "0348e7dd53c54a8d"),
    (0x0200CA40, "fda16691531b2d7a"),
    (0x0200CA54, "cd2326611d587e93"),
    (0x0200CA5C, "1376b83877975953"),
    (0x0200CA70, "0caa23c96cd4bdb6"),
    (0x0200CA78, "83bffcec665b9367"),
    (0x0200CA8C, "5b99d0609db68034"),
    (0x0200CA98, "e30f35f0b18b660f"),
    (0x0200CC20, "41120c56df2be864"),
    (0x0200CC40, "a76fd0d608afbb82"),
    (0x0200CDE0, "7594e7393c21c7f5"),
    (0x0200CED4, "94069c07ef247f39"),
    (0x0200CF08, "dbbdd669e432554d"),
    (0x0200CF28, "c6b2b420fc3b82bd"),
    (0x0200CF54, "0537f74d400b1657"),
    (0x0200CFA0, "8677f176cad74d4f"),
    (0x0200D064, "d58add868a9710ee"),
    (0x0200D0AC, "cc076fef181b0ddf"),
    (0x0200D0CC, "1bdf4c8d563972b2"),
    (0x0200D0F8, "ae98f8fe90c911c1"),
    (0x0200D144, "b56845139b0d6aea"),
    (0x0200D288, "20789ab9900ec11d"),
    (0x0200D298, "4c1bd048037450b9"),
    (0x0200D2B0, "7c204bfd2cbf59cd"),
    (0x0200D2C4, "01487d15d95afdc8"),
    (0x0200D2D8, "3a1bc1b5a507ee27"),
    (0x0200D310, "a5f7741bd0a9d61b"),
    (0x0200D368, "036b991b42344396"),
    (0x0200D3C0, "a746e0bdb2b12f2c"),
];
// fndiff-recorded-end

/// Fixed emulator address of pointer argument 2 (the regional-table arena).
const ARG2: u32 = 0x02E3_0000;

/// The callees routed to the spec's stub lines.
struct Rom<'a> {
    env: &'a mut StubEnv,
    arena: Vec<u8>,
}

impl ZukanHooks for Rom<'_> {
    fn lock(&mut self, mon: u32) -> u32 {
        self.env.call(0x0201_7D30, &mut [Arg::V(mon)])
    }
    fn unlock(&mut self, mon: u32, state: u32) {
        self.env
            .call(0x0201_7D70, &mut [Arg::V(mon), Arg::V(state)]);
    }
    fn param(&mut self, mon: u32, id: u32) -> u32 {
        self.env
            .call(0x0201_7E1C, &mut [Arg::V(mon), Arg::V(id), Arg::V(0)])
    }
    fn gender(&mut self, mon: u32) -> u32 {
        self.env.call(0x0201_7F00, &mut [Arg::V(mon)])
    }
    fn shiny(&mut self, mon: u32) -> u32 {
        self.env.call(0x0201_7EAC, &mut [Arg::V(mon)])
    }
    fn regional_table(&mut self, heap: u32) -> (Vec<u16>, u32) {
        let p = self.env.call(0x0201_AF80, &mut [Arg::V(heap), Arg::V(0)]);
        let o = (p - ARG2) as usize;
        (r16s::<650>(&self.arena, o).to_vec(), p)
    }
    fn free(&mut self, p: u32) {
        self.env.call(0x0203_07B0, &mut [Arg::V(p)]);
    }
    fn personal_open(&mut self, species: u32, form: u32, heap: u32) -> u32 {
        self.env.call(
            0x0201_ADE4,
            &mut [Arg::V(species), Arg::V(form), Arg::V(heap)],
        )
    }
    fn personal_param(&mut self, p: u32, id: u32) -> u32 {
        self.env.call(0x0201_AE1C, &mut [Arg::V(p), Arg::V(id)])
    }
    fn personal_close(&mut self, p: u32) {
        self.env.call(0x0201_AE14, &mut [Arg::V(p)]);
    }
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    match addr {
        0x0200_C720 => return Zukan::is_invalid_species(v[0]) as u64,
        0x0200_C780 => return Zukan::counts_national(v[0]) as u64,
        0x0200_C7A0 => return Zukan::counts_regional(v[0]) as u64,
        0x0200_C7C0 => return Zukan::form_start(v[0]) as u64,
        0x0200_C7F4 => return Zukan::form_index(v[0]) as u64,
        0x0200_C9EC => return Zukan::size() as u64,
        0x0200_CC20 => return Zukan::form_count(v[0]) as u64,
        _ => {}
    }
    let mut z = Zukan::from_bytes(&c.bufs[0]);
    let arena = c.bufs.get(2).cloned().unwrap_or_default();
    let h = &mut Rom {
        env: &mut c.env,
        arena,
    };
    let ret: u64 = match addr {
        0x0200_C738 => {
            z.set_lang(v[1], v[2]);
            0
        }
        0x0200_C818 => {
            z.set_seen(v[1], v[2], v[3]);
            0
        }
        0x0200_C914 => {
            z.set_form_seen(v[1], v[3], v[4]);
            0
        }
        0x0200_C9F4 => {
            z.init();
            0
        }
        0x0200_CA28 => {
            z.set_national();
            0
        }
        0x0200_CA38 => z.national() as u64,
        0x0200_CA40 => {
            z.set_field_1();
            0
        }
        0x0200_CA54 => z.field_1() as u64,
        0x0200_CA5C => {
            z.set_bit_11(v[1]);
            0
        }
        0x0200_CA70 => z.bit_11() as u64,
        0x0200_CA78 => {
            z.set_field_12(v[1]);
            0
        }
        0x0200_CA8C => z.field_12() as u64,
        0x0200_CA98 => {
            z.set_shown(v[1], v[2], v[3], v[4]);
            0
        }
        0x0200_CC40 => {
            let cur = [r32(&c.bufs[2], 0), r32(&c.bufs[3], 0), r32(&c.bufs[4], 0)];
            let out = z.shown_kind(v[1], v[5], cur, h);
            for (k, w) in out.iter().enumerate() {
                w32(&mut c.bufs[2 + k], 0, *w);
            }
            0
        }
        0x0200_CDE0 => {
            z.register_caught(v[1], h);
            0
        }
        0x0200_CED4 => z.caught(v[1]) as u64,
        0x0200_CF08 => z.caught_count(v[1], h) as u64,
        0x0200_CF28 => z.caught_count_national() as u64,
        0x0200_CF54 => z.caught_count_regional(v[1], h) as u64,
        0x0200_CFA0 => {
            z.register_seen(v[1], h);
            0
        }
        0x0200_D064 => z.seen(v[1]) as u64,
        0x0200_D0AC => z.seen_count(v[1], h) as u64,
        0x0200_D0CC => z.seen_count_national() as u64,
        0x0200_D0F8 => z.seen_count_regional(v[1], h) as u64,
        0x0200_D144 => z.seen_kind(v[1], v[2], v[3], v[4]) as u64,
        0x0200_D288 => z.spinda(v[1]) as u64,
        0x0200_D298 => z.national_complete() as u64,
        0x0200_D2B0 => z.regional_caught_complete(v[1], h) as u64,
        0x0200_D2C4 => z.regional_seen_complete(v[1], h) as u64,
        0x0200_D2D8 => z.national_caught() as u64,
        0x0200_D310 => z.regional_caught(v[1], h) as u64,
        0x0200_D368 => z.regional_seen(v[1], h) as u64,
        0x0200_D3C0 => z.lang_seen(v[1], v[2]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    };
    c.bufs[0] = z.to_bytes();
    ret
}

/// Every ported function against the original (`tools/check-fn.sh black-storage zukan_save`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "zukan_save",
            include_str!("../../../../tools/rom-diff/specs/black-storage/zukan_save.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

/// A Pokemon whose parameters come from a fixed list.
struct Mon {
    species: u32,
    form: u32,
    gender: u32,
    shiny: u32,
}

impl ZukanHooks for Mon {
    fn lock(&mut self, _mon: u32) -> u32 {
        0
    }
    fn unlock(&mut self, _mon: u32, _state: u32) {}
    fn param(&mut self, _mon: u32, id: u32) -> u32 {
        match id {
            5 => self.species,
            111 => self.form,
            12 => 2,
            0 => 0x1234,
            _ => 0,
        }
    }
    fn gender(&mut self, _mon: u32) -> u32 {
        self.gender
    }
    fn shiny(&mut self, _mon: u32) -> u32 {
        self.shiny
    }
    fn regional_table(&mut self, _heap: u32) -> (Vec<u16>, u32) {
        (vec![1; 650], 0)
    }
    fn free(&mut self, _p: u32) {}
    fn personal_open(&mut self, _species: u32, _form: u32, _heap: u32) -> u32 {
        0
    }
    fn personal_param(&mut self, _p: u32, _id: u32) -> u32 {
        0
    }
    fn personal_close(&mut self, _p: u32) {}
}

#[test]
fn register_and_query() {
    let mut z = Zukan::default();
    z.init();
    let mut m = Mon {
        species: 201,
        form: 5,
        gender: 2,
        shiny: 1,
    };
    z.register_caught(0, &mut m);
    assert!(z.caught(201) && z.seen(201) && z.lang_seen(201, 2));
    assert_eq!(z.seen_kind(201, 0, 1, 5), 1);
    assert_eq!(z.shown_kind(201, 0, [9, 9, 9], &mut m), [0, 1, 5]);
    m.species = 327;
    z.register_seen(0, &mut m);
    assert_eq!(z.spinda(0), 0x1234);
    assert_eq!(z.caught_count_national(), 1);
    assert_eq!(z.seen_count(0, &mut m), 2);
    assert_eq!(Zukan::from_bytes(&z.to_bytes()), z);
}
