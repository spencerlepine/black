use super::*;
use crate::testutil::{Fnv, Rng};

struct Host {
    h: Fnv,
    n: u32,
    /// Four moves per Pokemon ID (0..64).
    moves: Vec<[u16; 4]>,
}

/// The moves table shared with `tools/rom-diff/gen.py` (`field`).
fn moves_table() -> Vec<[u16; 4]> {
    let mut r = Rng::new(0xF1E1D);
    (0..64)
        .map(|_| [0; 4].map(|_| (r.next() % 9) as u16))
        .collect()
}

impl Host {
    fn new() -> Self {
        Host {
            h: Fnv::new(),
            n: 0,
            moves: moves_table(),
        }
    }
}

impl FieldHost for Host {
    fn effect_create(&mut self, slot: u32) -> u32 {
        self.n += 1;
        let handle = if self.n.is_multiple_of(5) {
            0
        } else {
            0x1000 + self.n * 16 + slot
        };
        self.h.w(0xA1);
        self.h.w(slot);
        self.h.w(handle);
        handle
    }
    fn effect_destroy(&mut self, handle: u32) {
        self.h.w(0xA2);
        self.h.w(handle);
    }
    fn poke_knows_move(&mut self, poke_id: u32, mv: u32) -> bool {
        self.moves[poke_id as usize].iter().any(|m| *m as u32 == mv)
    }
    fn fun_02045cac(&mut self, a: u32, b: u32, c: u32) {
        for v in [0xB0, a, b, c] {
            self.h.w(v);
        }
    }
    fn fun_0201efa4(&mut self, a: u32, index: u32, value: u32, mode: u32, x: u32, y: u32) {
        for v in [0xB1, a, index, value, mode, x, y] {
            self.h.w(v);
        }
    }
    fn fun_0201f250(&mut self, a: u32, b: u32, c: u32) {
        for v in [0xB2, a, b, c] {
            self.h.w(v);
        }
    }
}

struct Mon(u8);

impl BattleMon for Mon {
    fn get(&self, _id: u32) -> u32 {
        0
    }
    fn has_type(&self, _ty: u32) -> bool {
        false
    }
    fn poke_id(&self) -> u8 {
        self.0
    }
    fn species(&self) -> u16 {
        0
    }
}

fn word(img: &[u8; SIZE], o: usize) -> u32 {
    u32::from_le_bytes(img[o..o + 4].try_into().unwrap())
}

#[test]
fn weather_counts_down() {
    let mut f = Field::new();
    assert_eq!(f.weather(), 0);
    assert_eq!(f.weather_turns(), 0);
    f.set_weather(2, 2);
    assert_eq!(f.weather(), 2);
    assert_eq!(f.weather_turns(), 2);
    assert_eq!(f.tick_weather(), 0);
    assert_eq!(f.tick_weather(), 2);
    assert_eq!(f.weather(), 0);
    f.set_weather(4, 0xFF);
    assert_eq!(f.tick_weather(), 0);
    assert_eq!(f.weather_turns(), 0xFF);
    f.clear_weather();
    assert_eq!(f.weather(), 0);
}

#[test]
fn new_image_layout() {
    let img = Field::new().image();
    assert_eq!(word(&img, 0), 0);
    assert_eq!(word(&img, 4), 0xFF);
    assert_eq!(word(&img, 0x68), 0x1F);
    assert_eq!(word(&img, 0x124), 0x1F);
    assert_eq!(word(&img, 0x148), 0);
}

#[test]
fn effect_lifecycle() {
    let mut host = Host::new();
    let mut f = Field::new();
    // Slot 2 lasting 2 turns, linked to Pokemon 5.
    let cont = SickCont::poke_turn(5, 2);
    assert!(f.add_effect(&mut host, 2, cont, true));
    assert_eq!(f.is_active(2), 1);
    assert!(!f.add_effect(&mut host, 2, cont, true));
    let img = f.image();
    assert_eq!(word(&img, 0x08 + 8), 0x1000 + 16 + 2);
    assert_eq!(word(&img, 0x68 + 0x18 * 2), 5);
    assert_eq!(word(&img, 0x128 + 8), 1);
    assert!(f.add_poke(2, 9));
    assert!(!f.add_poke(2, 9));
    assert!(!f.add_poke(3, 9));
    // Removing the linked Pokemon re-links the slot to the next one.
    f.remove_poke(&mut host, 5);
    assert_eq!(f.cont[2].linked_poke_id(), 9);
    assert_eq!(f.count[2], 1);
    let mut ended = vec![];
    let mut cb = |slot: u32, ctx: u32| ended.push((slot, ctx));
    f.tick_effects(&mut host, Some(&mut cb), 77);
    f.tick_effects(&mut host, Some(&mut cb), 77);
    assert_eq!(ended, vec![(2, 77)]);
    assert_eq!(f.is_active(2), 0);
    assert_eq!(f, {
        let mut g = Field::new();
        g.reset_slot(2);
        g
    });
}

#[test]
fn last_poke_ends_effect_and_view_failure() {
    let mut host = Host::new();
    let mut f = Field::new();
    assert!(f.add_effect(&mut host, 3, SickCont::poke(4), true));
    f.remove_poke(&mut host, 4);
    assert_eq!(f.is_active(3), 0);
    // The fifth view creation fails and leaves the slot inactive.
    host.n = 4;
    assert!(!f.add_effect(&mut host, 3, SickCont::poke(4), true));
    assert_eq!(f.is_active(3), 0);
    // Without a view the handle is not touched.
    assert!(f.add_effect(&mut host, 3, SickCont::poke(4), false));
    assert!(f.remove_effect(&mut host, 3));
    assert!(!f.remove_effect(&mut host, 3));
}

#[test]
fn pokes_list_capacity() {
    let mut host = Host::new();
    let mut f = Field::new();
    assert!(f.add_effect(&mut host, 1, SickCont::permanent(), false));
    for id in 0..6 {
        assert!(f.add_poke(1, id));
    }
    assert!(!f.add_poke(1, 6));
    f.remove_poke(&mut host, 0);
    assert_eq!(f.pokes[1], [1, 2, 3, 4, 5, NO_POKE]);
}

#[test]
fn imprison_check() {
    let mut host = Host::new();
    let mut f = Field::new();
    let mv = host.moves[14][0] as u32;
    assert!(f.add_effect(&mut host, 3, SickCont::poke(14), false));
    // Pokemon 14 is on side 1, so it blocks Pokemon 2 (side 0) but not Pokemon 13.
    assert!(f.foe_knows_move(&mut host, &Mon(2), mv));
    assert!(!f.foe_knows_move(&mut host, &Mon(13), mv));
    assert!(!f.foe_knows_move(&mut host, &Mon(2), 99));
}

/// Hash of every routine over random call sequences, matching the original machine code run
/// under an emulator (`tools/rom-diff/gen.py field`).
#[test]
fn matches_rom_fuzz() {
    let mut host = Host::new();
    let mut hashes = Fnv::new();
    for seed in 0..1000u32 {
        let mut r = Rng::new(seed);
        let mon = Mon((r.next() % 24) as u8);
        host.n = 0;
        host.h = Fnv::new();
        let mut f = Field::new();
        let w = r.next();
        f.init(w);
        for _ in 0..30 {
            let op = r.next() % 15;
            let a = r.next();
            let b = r.next();
            let cont = SickCont((b & !0x1FF) | ((a >> 20) % 8) << 3 | (1 + (b ^ (a >> 16)) % 4));
            let mut v = 0u32;
            match op {
                0 => f.init(a),
                1 => f.set_weather(a % 8, if b.is_multiple_of(3) { 0xFF } else { b % 6 }),
                2 => v = f.tick_weather(),
                3 => v = f.weather().wrapping_mul(31).wrapping_add(f.weather_turns()),
                4 => v = f.add_effect(&mut host, (a % 8) as usize, cont, true) as u32,
                5 => v = f.add_effect(&mut host, (a % 8) as usize, cont, false) as u32,
                6 => v = f.remove_effect(&mut host, (a % 8) as usize) as u32,
                7 => v = f.add_poke((a % 8) as usize, b % 64) as u32,
                8 => f.remove_poke(&mut host, b % 64),
                9 => v = f.foe_knows_move(&mut host, &mon, a % 9) as u32,
                10 => {
                    let mut log = vec![];
                    let mut cb = |slot: u32, ctx: u32| log.extend([0xC0, slot, ctx]);
                    let cbr: Option<&mut dyn FnMut(u32, u32)> =
                        if a & 1 == 1 { Some(&mut cb) } else { None };
                    f.tick_effects(&mut host, cbr, b);
                    for x in log {
                        host.h.w(x);
                    }
                }
                11 => v = f.is_active((a % 8) as usize),
                12 | 13 => {
                    let g = GfxGlobals {
                        p1c: r.next(),
                        p08: r.next(),
                        p0c: r.next(),
                    };
                    let vals = [0; 6].map(|_| r.next());
                    if op == 12 {
                        load_gfx_a(&mut host, g, a, vals)
                    } else {
                        load_gfx_b(&mut host, g, a, vals)
                    }
                }
                _ => f.clear_weather(),
            }
            host.h.w(v);
            for c in f.image().chunks(4) {
                host.h.w(u32::from_le_bytes(c.try_into().unwrap()));
            }
        }
        hashes.w(host.h.0 as u32);
        hashes.w((host.h.0 >> 32) as u32);
    }
    println!("field hash {}", hashes.hex());
    assert_eq!(hashes.hex(), "8d7cc23e6321c3c6");
}
