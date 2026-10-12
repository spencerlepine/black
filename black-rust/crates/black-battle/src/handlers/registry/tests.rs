//! Registration against the original code (`tools/rom-diff/handlers.py registration`). The
//! registry and the table-constant getters are read from the local build at run time; the
//! test is skipped when it is missing.

use super::*;
use crate::battle_mon::tests::{random_bpp, MockEnv};
use crate::event_factor::FACTOR_POOL;
use crate::event_var::EventVarStack;
use crate::testutil::{Fnv, Rng};

const NMON: usize = 8;
const NONE: u32 = 0xFF;
const OVY_BASE: u32 = 0x021B_6100;
const REGISTRY: u32 = 0x021F_0E74;
const REGISTRY_LEN: u32 = 0x9E;

struct RomRegistry {
    ovy: Vec<u8>,
}

impl RomRegistry {
    fn load() -> Option<Self> {
        let dir = std::env::var("BLACK_ROM_BUILD").unwrap_or_else(|_| {
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../../black-rom/build/black.us"
            )
            .to_string()
        });
        std::fs::read(format!("{dir}/OVY_93.sbin"))
            .ok()
            .map(|ovy| RomRegistry { ovy })
    }

    fn r16(&self, addr: u32) -> u32 {
        let o = (addr - OVY_BASE) as usize;
        u16::from_le_bytes([self.ovy[o], self.ovy[o + 1]]) as u32
    }

    fn r32(&self, addr: u32) -> u32 {
        self.r16(addr) | self.r16(addr + 2) << 16
    }

    /// `(ability, getter)` pairs.
    fn entries(&self) -> Vec<(u32, u32)> {
        (0..REGISTRY_LEN)
            .map(|i| (self.r32(REGISTRY + 8 * i), self.r32(REGISTRY + 8 * i + 4)))
            .collect()
    }
}

impl AbilityRegistry for RomRegistry {
    fn getter(&self, ability: u32) -> Option<(u32, u32)> {
        let (_, getter) = self.entries().into_iter().find(|e| e.0 == ability)?;
        if let Some(custom) = custom_getter(ability) {
            return Some(custom);
        }
        // `movs r1, #n; str r1, [r0]; ldr r0, [pc, #k]; bx lr`, then the table address.
        let g = getter & !1;
        let (mov, str, ldr, bx) = (
            self.r16(g),
            self.r16(g + 2),
            self.r16(g + 4),
            self.r16(g + 6),
        );
        assert!(mov >> 8 == 0x21 && str == 0x6001 && ldr >> 8 == 0x48 && bx == 0x4770);
        let table = self.r32(((g + 8) & !3) + (ldr & 0xFF) * 4);
        Some((table, mov & 0xFF))
    }
}

fn idx(f: Option<FactorId>) -> u32 {
    f.map_or(NONE, |v| v as u32)
}

fn hash_state(h: &mut Fnv, s: &EventSystem, env: &mut MockEnv) {
    for f in &s.pool {
        h.w(idx(f.prev));
        h.w(idx(f.next));
        h.w(f.table);
        h.w(f.skip_check);
        h.w(f.ftype);
        h.w(f.priority);
        h.w(f.flags);
        f.work.iter().for_each(|&w| h.w(w));
        h.w(f.sub_id as u32);
        h.b(f.owner);
        h.b(f.poke_id);
    }
    h.w(s.depth);
    h.w(idx(s.head));
    h.w(s.count);
    s.free.iter().for_each(|&f| h.w(f as u32));
    for e in env.log.drain(..) {
        e.iter().for_each(|&v| h.w(v));
    }
}

/// Registration, removal, disabling and swapping, hashed against the original code.
#[test]
fn registration_match_rom() {
    let Some(reg) = RomRegistry::load() else {
        return;
    };
    let abilities: Vec<u32> = reg.entries().iter().map(|e| e.0).collect();
    let mut h = Fnv::new();
    for seed in 0..120u32 {
        let mut r = Rng::new(0x8484 + seed);
        let mut mons: Vec<BattleMon> = (0..NMON).map(|_| random_bpp(&mut r)).collect();
        for m in mons.iter_mut() {
            let ability = match r.next() % 4 {
                0 => r.next() & 0xFFFF,
                _ => abilities[r.next() as usize % abilities.len()],
            };
            m.set_ability(ability as u16);
            m.set_u8(0x19, (r.next() % 4) as u8);
        }
        let mut env = MockEnv {
            field: r.next() & 1 != 0,
            ..Default::default()
        };
        let mut sys = EventSystem::new();
        sys.init(&mut EventVarStack::new());
        for _ in 0..30 {
            let o = r.next() % 5;
            let a = r.next() as usize % NMON;
            let b = r.next() as usize % NMON;
            match o {
                0 => h.w(idx(register_ability(&mut sys, &reg, &mons[a], &mut env))),
                1 => unregister_ability(&mut sys, &mons[a]),
                2 => disable_ability(&mut sys, &mons[a]),
                3 => enable_ability(&mut sys, &reg, &mons[a], &mut env),
                _ => {
                    let (x, y) = (mons[a].u16_at(0x13C), mons[b].u16_at(0x13C));
                    mons[a].set_ability(y);
                    mons[b].set_ability(x);
                    swap_abilities(&mut sys, &reg, &mons[a], &mons[b], &mut env);
                }
            }
            hash_state(&mut h, &sys, &mut env);
        }
        assert!(sys.count as usize <= FACTOR_POOL);
    }
    assert_eq!(h.hex(), "1279e193c3035c31");
}

#[test]
fn getter_words() {
    assert_eq!(split_getter_word(pack_u16_pair(0, 3)), (5, 3));
    assert_eq!(split_getter_word(pack_u16_pair(7, 1)), (7, 1));
    assert_eq!(getter_021d88d0(), (0x021F_05A0, 0x0007_0001));
    assert_eq!(getter_021dc564(), (0x021F_0600, 0x0004_0001));
    assert_eq!(custom_getter(1), None);
}
