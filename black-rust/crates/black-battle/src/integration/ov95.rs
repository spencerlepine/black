//! Overlay 95 (linked at 0x06898020; `bx pc` veneers in overlay 93 reach it): host models of the
//! routines the ported code calls, written from the disassembly of `OVY_95.sbin` (UNVERIFIED
//! unless stated). The effect handlers and the side-effect adder / remover are ported
//! (`handlers::effects`); everything else here is a model.

use super::field::factor_addr;
use super::Env;

/// Field-effect registry: 8 `(key, getter)` entries, stride 8.
const FIELD_REGISTRY: u32 = 0x0689_D650;

/// Model of `06898020(key, owner)` (thunk `021EFA84`): finds `key` in the field-effect registry and
/// registers its handler table as an event factor (`021d6f0c(3, key, 3, 0, owner, table,
/// count)`); returns the factor pointer, 0 when the key is unknown or the pool is full.
pub fn field_effect_add(env: &mut Env, key: u32) -> u32 {
    field_effect_add_owner(env, key, 0)
}

pub fn field_effect_add_owner(env: &mut Env, key: u32, owner: u32) -> u32 {
    let rom = env.rom.clone();
    for i in 0..8 {
        if rom.r32(FIELD_REGISTRY + 8 * i) == key {
            let (table, n) = rom.getter(rom.r32(FIELD_REGISTRY + 8 * i + 4));
            let f = env
                .events_mut()
                .add(3, key as u16, 3, 0, owner as u8, table, n as u16);
            return factor_addr(f);
        }
    }
    0
}

use super::field::factor_of;
use super::thumb::{OVY95_BSS, OVY95_BSS_SIZE};
use crate::handlers::effects::{SideEffectEntry, SideRegistry, SIDE_ENTRIES};
use crate::server_flow::env::Arg;

/// The side-effect registry 0x0689D780 read from the ROM.
struct SideReg<'a>(&'a super::rom::Rom);

impl SideRegistry for SideReg<'_> {
    fn side_entry(&self, i: u32) -> (u32, u32) {
        let a = 0x0689_D780 + 12 * i;
        (self.0.r32(a), self.0.r32(a + 8))
    }
    fn side_handlers(&self, i: u32) -> (u32, u32) {
        self.0.getter(self.0.r32(0x0689_D780 + 12 * i + 4))
    }
}

impl Env {
    /// Copies the typed side-effect layers into the overlay-95 BSS image (for interpreted code).
    fn side_effects_to_bss(&mut self) {
        if self.objs.borrow().find_block(OVY95_BSS, 1).is_none() {
            self.objs.borrow_mut().map(
                OVY95_BSS,
                vec![0; OVY95_BSS_SIZE as usize],
                "overlay 95 BSS",
            );
        }
        let mut b = vec![0u8; SIDE_ENTRIES * 16];
        for (i, e) in self.side_effects.entries.iter().enumerate() {
            let w = [factor_addr(e.factor), e.cont, e.w8, e.layers];
            for (k, x) in w.iter().enumerate() {
                b[16 * i + 4 * k..16 * i + 4 * k + 4].copy_from_slice(&x.to_le_bytes());
            }
        }
        self.objs.borrow_mut().write(OVY95_BSS, &b);
    }

    fn side_effects_from_bss(&mut self) {
        let mut b = vec![0u8; SIDE_ENTRIES * 16];
        self.objs.borrow_mut().read(OVY95_BSS, &mut b);
        for (i, e) in self.side_effects.entries.iter_mut().enumerate() {
            let w = |k: usize| {
                u32::from_le_bytes(b[16 * i + 4 * k..16 * i + 4 * k + 4].try_into().unwrap())
            };
            *e = SideEffectEntry {
                factor: factor_of(w(0)),
                cont: w(1),
                w8: w(2),
                layers: w(3),
            };
        }
    }
}

/// An overlay-95 routine `target` (reached through a veneer at `addr`).
pub fn call(env: &mut Env, addr: u32, target: u32, args: &mut [Arg<'_>]) -> u32 {
    // veneer call sites whose port passes the work (or a struct in it) through without holding
    // the work (from the disassembly of the call sites)
    let p0 = match addr {
        0x021C_845C => Some(env.waddr + 0x1AB8),
        0x021B_D89C | 0x021C_6FB0 | 0x021C_70D4 => Some(env.waddr),
        _ => None,
    };
    // a caller's local buffer is mapped for the call (and copied back)
    let mut temps: Vec<(usize, u32)> = Vec::new();
    let mut vals: Vec<u32> = Vec::new();
    for (i, a) in args.iter().enumerate() {
        vals.push(match a {
            Arg::V(v) | Arg::PAt(v) => *v,
            Arg::P if i == 0 && p0.is_some() => p0.unwrap(),
            Arg::Out(b) | Arg::PBuf(b) => {
                let t = env.objs.borrow_mut().alloc(b.len().max(4) as u32, "overlay-95 call local");
                env.objs.borrow_mut().write(t, b);
                temps.push((i, t));
                t
            }
            Arg::In(b) => {
                let t = env.objs.borrow_mut().alloc(b.len().max(4) as u32, "overlay-95 call local");
                env.objs.borrow_mut().write(t, b);
                temps.push((usize::MAX, t));
                t
            }
            _ => panic!(
                "B16 gap: overlay-95 routine {target:#x} (veneer {addr:#x}) gets a pass-through argument {i} ({})",
                env.backtrace()
            ),
        });
    }
    let r = call_vals(env, target, &vals);
    for (i, t) in temps {
        if i != usize::MAX {
            if let Some(Arg::Out(b) | Arg::PBuf(b)) = args.get_mut(i) {
                env.objs.borrow_mut().read(t, b);
            }
        }
        env.objs.borrow_mut().free(t);
    }
    r
}

fn call_vals(env: &mut Env, target: u32, vals: &[u32]) -> u32 {
    match target {
        // the ported side-effect adder / remover (B7b2)
        0x0689_8C10 => {
            let rom = env.rom.clone();
            let mut sys = env.events.take().expect("event system");
            let f = env
                .side_effects
                .add(&mut sys, &SideReg(&rom), vals[0], vals[1], vals[2]);
            env.events = Some(sys);
            factor_addr(f)
        }
        0x0689_8D10 => {
            let mut sys = env.events.take().expect("event system");
            let r = env.side_effects.remove(&mut sys, vals[0], vals[1]);
            env.events = Some(sys);
            r as u32
        }
        0x0689_8020 => field_effect_add_owner(env, vals[0], vals[1]),
        0x0689_8080 => {
            let f = factor_of(vals[0]).expect("null factor");
            env.events_mut().remove(f);
            0
        }
        _ => {
            env.side_effects_to_bss();
            let r = env.run_thumb(target, vals);
            env.side_effects_from_bss();
            r
        }
    }
}
