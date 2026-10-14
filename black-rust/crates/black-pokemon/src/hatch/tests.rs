use super::*;
use black_fndiff::{run_suite, Arg, Case, StubEnv};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02017B24, "bc1bfffdd7f5374f"),
    (0x02017B4C, "a18f3f49c9ad902d"),
    (0x0218A9B0, "6d2ce4f8e4d6b0b2"),
    (0x0218AA2C, "d2d4be048a9a9b14"),
    (0x0218AA74, "967ca55f8d8a35f4"),
    (0x0218AAD0, "f6d858ab46d66c2c"),
    (0x0218AB0C, "d9a7a35369d4f9d7"),
    (0x0218AB50, "ad009ba1ceab3729"),
];
// fndiff-recorded-end

/// The callees outside the area, routed to the spec's stub lines (same model as the ROM side).
struct Rom<'a> {
    env: &'a mut StubEnv,
}

impl HatchHooks for Rom<'_> {
    fn get(&mut self, mon: u32, id: u32) -> u32 {
        self.env
            .call(0x0201_7E1C, &mut [Arg::V(mon), Arg::V(id), Arg::V(0)])
    }
    fn put(&mut self, mon: u32, id: u32, value: u32) {
        self.env
            .call(0x0201_7E40, &mut [Arg::V(mon), Arg::V(id), Arg::V(value)]);
    }
    fn party_count(&mut self, party: u32) -> u32 {
        self.env.call(0x0201_AA18, &mut [Arg::V(party)])
    }
    fn party_mon(&mut self, party: u32, index: u32) -> u32 {
        self.env
            .call(0x0201_AB28, &mut [Arg::V(party), Arg::V(index)])
    }
    fn fsys_save(&mut self, fsys: u32) -> u32 {
        self.env.call(0x0201_2EBC, &mut [Arg::V(fsys)])
    }
    fn step_block(&mut self, save: u32) -> u32 {
        self.env.call(0x0200_8854, &mut [Arg::V(save)])
    }
    fn counter_get(&mut self, block: u32) -> u32 {
        let mut b = [0u8; 4];
        self.env
            .call(0x0200_8970, &mut [Arg::V(block), Arg::Out(&mut b)]);
        u32::from_le_bytes(b)
    }
    fn counter_set(&mut self, block: u32, value: u32) {
        self.env
            .call(0x0200_896C, &mut [Arg::V(block), Arg::V(value)]);
    }
    fn scaled_step(&mut self, unit: u32) -> u32 {
        self.env.call(0x0201_0FF4, &mut [Arg::V(unit)])
    }
    fn work_half(&mut self, work: u32) -> u32 {
        self.env.call(0x0218_8C84, &mut [Arg::V(work)]) & 0xFFFF
    }
    fn work_daycare(&mut self, work: u32) -> u32 {
        self.env.call(0x0218_8D50, &mut [Arg::V(work)])
    }
    fn daycare_step(&mut self, daycare: u32) {
        self.env.call(0x021C_3378, &mut [Arg::V(daycare)]);
    }
    fn fsys_party(&mut self, fsys: u32) -> u32 {
        self.env.call(0x0201_2934, &mut [Arg::V(fsys)])
    }
    fn start_event(&mut self, fsys: u32, event: u32, zero: u32, arg: u32) -> u32 {
        self.env.call(
            0x0215_8768,
            &mut [Arg::V(fsys), Arg::V(event), Arg::V(zero), Arg::V(arg)],
        )
    }
    fn work_field(&mut self, work: u32) -> u32 {
        self.env.call(0x0218_8C80, &mut [Arg::V(work)])
    }
    fn field_fsys(&mut self, x: u32) -> u32 {
        self.env.call(0x0201_20F4, &mut [Arg::V(x)])
    }
    fn fast_handler(&mut self, work: u32, a: u32, fsys: u32) -> u32 {
        self.env
            .call(0x0218_AC84, &mut [Arg::V(work), Arg::V(a), Arg::V(fsys)])
    }
    fn friendship_step(&mut self, word08: u32, work: u32) {
        self.env
            .call(0x0218_AB9C, &mut [Arg::V(word08), Arg::V(work)]);
    }
    fn poison_step(&mut self, a: u32, work: u32, half: u32) -> u32 {
        self.env
            .call(0x0218_AC08, &mut [Arg::V(a), Arg::V(work), Arg::V(half)])
    }
    fn last_handler(&mut self, work: u32, a: u32, fsys: u32) -> u32 {
        self.env
            .call(0x0218_AC4C, &mut [Arg::V(work), Arg::V(a), Arg::V(fsys)])
    }
    fn unlock_box(&mut self, mon: u32) -> u32 {
        self.env.call(0x0201_7D30, &mut [Arg::V(mon)])
    }
    fn relock_box(&mut self, mon: u32, flag: u32) {
        self.env.call(0x0201_7D70, &mut [Arg::V(mon), Arg::V(flag)]);
    }
    fn unlock_party(&mut self, mon: u32) -> u32 {
        self.env.call(0x0201_7DBC, &mut [Arg::V(mon)])
    }
    fn relock_party(&mut self, mon: u32, flag: u32) {
        self.env.call(0x0201_7DE4, &mut [Arg::V(mon), Arg::V(flag)]);
    }
    fn box_get(&mut self, mon: u32, id: u32) -> u32 {
        self.env
            .call(0x0201_7E88, &mut [Arg::V(mon), Arg::V(id), Arg::V(0)])
    }
    fn box_put(&mut self, mon: u32, id: u32, value: u32) {
        self.env
            .call(0x0201_7E64, &mut [Arg::V(mon), Arg::V(id), Arg::V(value)]);
    }
    fn message_into(&mut self, message: u32, buf: u32, len: u32) {
        self.env.call(
            0x0204_5F5C,
            &mut [Arg::P, Arg::V(message), Arg::V(buf), Arg::V(len)],
        );
    }
    fn set_met(&mut self, mon: u32, kind: u32, a: u32, b: u32, c: u32) {
        self.env.call(
            0x0202_F1B4,
            &mut [Arg::V(mon), Arg::V(kind), Arg::V(a), Arg::V(b), Arg::V(c)],
        );
    }
}

fn word(b: &[u8], ofs: usize) -> u32 {
    u32::from_le_bytes(b[ofs..ofs + 4].try_into().unwrap())
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    match addr {
        0x0218_A9B0 => {
            let cfg = StepCfg {
                word08: word(&c.regions[0], 8),
                half10: word(&c.regions[0], 0x10) & 0xFFFF,
                flags: word(&c.regions[0], 0x3C),
            };
            let mut h = Rom { env: &mut c.env };
            u64::from(step_hook(&cfg, v[1], &mut h))
        }
        0x0218_AA2C => u64::from(step_decrement(v[0], &mut Rom { env: &mut c.env })),
        0x0218_AA74 => {
            decrement_eggs(v[0], &mut Rom { env: &mut c.env });
            0
        }
        0x0218_AAD0 => {
            step_counter(v[0], v[1], &mut Rom { env: &mut c.env });
            0
        }
        0x0218_AB0C => u64::from(hatch_ready(v[0], &mut Rom { env: &mut c.env })),
        0x0218_AB50 => u64::from(egg_step(v[1], v[2], v[3], &mut Rom { env: &mut c.env })),
        0x0201_7B24 => {
            hatch(v[0], v[1], v[2], v[3], &mut Rom { env: &mut c.env });
            0
        }
        0x0201_7B4C => {
            hatch_body(v[0], v[1], v[2], v[3], &mut Rom { env: &mut c.env });
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-pokemon hatch`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &black_fndiff::Suite::new(
            "hatch",
            include_str!("../../../../tools/rom-diff/specs/black-pokemon/hatch.spec"),
            PORTED,
            RECORDED,
        ),
        &call,
    );
}

/// A scripted party for the plain-Rust behaviour tests.
struct Party {
    /// (ability, egg flag, friendship) per member.
    mons: Vec<(u32, u32, u32)>,
    counter: u32,
    step: u32,
    events: Vec<(u32, u32)>,
}

impl HatchHooks for Party {
    fn get(&mut self, mon: u32, id: u32) -> u32 {
        let m = self.mons[mon as usize];
        match id {
            0xA => m.0,
            0x4C => m.1,
            9 => m.2,
            _ => 0,
        }
    }
    fn put(&mut self, mon: u32, id: u32, value: u32) {
        assert_eq!(id, 9);
        self.mons[mon as usize].2 = value;
    }
    fn party_count(&mut self, _: u32) -> u32 {
        self.mons.len() as u32
    }
    fn party_mon(&mut self, _: u32, i: u32) -> u32 {
        i
    }
    fn fsys_save(&mut self, f: u32) -> u32 {
        f
    }
    fn step_block(&mut self, s: u32) -> u32 {
        s
    }
    fn counter_get(&mut self, _: u32) -> u32 {
        self.counter
    }
    fn counter_set(&mut self, _: u32, v: u32) {
        self.counter = v;
    }
    fn scaled_step(&mut self, unit: u32) -> u32 {
        unit * self.step / 100
    }
    fn work_half(&mut self, _: u32) -> u32 {
        0
    }
    fn work_daycare(&mut self, _: u32) -> u32 {
        0
    }
    fn daycare_step(&mut self, _: u32) {}
    fn fsys_party(&mut self, _: u32) -> u32 {
        0
    }
    fn start_event(&mut self, f: u32, e: u32, _: u32, _: u32) -> u32 {
        self.events.push((f, e));
        1
    }
    fn work_field(&mut self, _: u32) -> u32 {
        0
    }
    fn field_fsys(&mut self, _: u32) -> u32 {
        0
    }
    fn fast_handler(&mut self, _: u32, _: u32, _: u32) -> u32 {
        0
    }
    fn friendship_step(&mut self, _: u32, _: u32) {}
    fn poison_step(&mut self, _: u32, _: u32, _: u32) -> u32 {
        0
    }
    fn last_handler(&mut self, _: u32, _: u32, _: u32) -> u32 {
        0
    }
    fn unlock_box(&mut self, _: u32) -> u32 {
        0
    }
    fn relock_box(&mut self, _: u32, _: u32) {}
    fn unlock_party(&mut self, _: u32) -> u32 {
        0
    }
    fn relock_party(&mut self, _: u32, _: u32) {}
    fn box_get(&mut self, _: u32, _: u32) -> u32 {
        0
    }
    fn box_put(&mut self, _: u32, _: u32, _: u32) {}
    fn message_into(&mut self, _: u32, _: u32, _: u32) {}
    fn set_met(&mut self, _: u32, _: u32, _: u32, _: u32, _: u32) {}
}

#[test]
fn flame_body_halves_the_cycles_only_from_a_non_egg() {
    let mut p = Party {
        mons: vec![(0x31, 1, 5), (0, 1, 5)],
        counter: 0,
        step: 100,
        events: Vec::new(),
    };
    // the Flame Body Pokemon is itself an egg: it does not count
    assert_eq!(step_decrement(0, &mut p), 1);
    p.mons[0].1 = 0;
    assert_eq!(step_decrement(0, &mut p), 2);
    p.mons[0].0 = 0x28;
    assert_eq!(step_decrement(0, &mut p), 2);
    decrement_eggs(0, &mut p);
    assert_eq!(p.mons[1].2, 3);
    assert_eq!(p.mons[0].2, 5);
    p.mons[1].2 = 1;
    decrement_eggs(0, &mut p);
    assert_eq!(p.mons[1].2, 0);
}

#[test]
fn counter_rolls_over_above_0x10000_and_hatches_at_zero() {
    let mut p = Party {
        mons: vec![(0, 1, 1)],
        counter: 0xFF00,
        step: 100,
        events: Vec::new(),
    };
    // 0xFF00 + 0x100 = 0x10000: not above the limit
    step_counter(0, 0, &mut p);
    assert_eq!(p.counter, 0x10000);
    assert_eq!(hatch_ready(0, &mut p), 0);
    step_counter(0, 0, &mut p);
    assert_eq!(p.counter, 0);
    assert_eq!(p.mons[0].2, 0);
    assert_eq!(hatch_ready(0, &mut p), 1);
    p.mons[0].1 = 0;
    assert_eq!(hatch_ready(0, &mut p), 0);
    p.mons[0].1 = 1;
    assert_eq!(egg_step(0, 7, 0, &mut p), 1);
    assert_eq!(p.events, vec![(7, HATCH_EVENT)]);
}
