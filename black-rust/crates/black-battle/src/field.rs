//! Battlefield-wide state (asm/battle_field.s, `btl_field.c`, overlay 93).
//!
//! One [`Field`] (0x168 bytes, a static at `0x021F63F4` in the original) holds the weather and
//! eight "field effect" slots. Slot 0 is unused by the effect timer; slot 2 and slot 3 are
//! checked by the move-failure code (flag 9 of the move data, and the Pokemon-list/move check
//! [`Field::foe_knows_move`]), which makes them Gravity and Imprison. The other slot meanings
//! are not identified.
//!
//! The original exposes the global through thin wrappers (`021ef874`..`021ef948`) that load
//! `0x021F63F4` into `r0` and tail-call the real routine; each wrapper maps to the method named
//! in its doc comment. Routines in other overlays (the field-effect handler registry in overlay 95) and the
//! graphics calls are modelled by [`FieldHost`].
//!
//! Layout of the original struct (offsets used by [`Field::image`]):
//!
//! | offset | size | field |
//! |---|---|---|
//! | 0x000 | `u32` | weather kind (low byte used; 0 = none) |
//! | 0x004 | `u32` | weather turns left (0xFF = unlimited) |
//! | 0x008 | `[u32; 8]` | event factor of the slot's handlers (0 = none) |
//! | 0x028 | `[u32; 8]` | [`SickCont`] per slot |
//! | 0x048 | `[u32; 8]` | elapsed turns per slot |
//! | 0x068 | `[[u32; 6]; 8]` | Pokemon IDs per slot (0x1F = none) |
//! | 0x128 | `[u32; 8]` | number of Pokemon IDs per slot |
//! | 0x148 | `[u32; 8]` | slot active flag |

use crate::hooks::BattleMon;
use crate::main_util::same_side;
use crate::sick_cont::SickCont;

/// Number of effect slots.
pub const SLOTS: usize = 8;
/// Pokemon IDs a slot can hold.
pub const SLOT_POKES: usize = 6;
/// "No Pokemon" ID stored in empty list entries.
pub const NO_POKE: u32 = 0x1F;
/// Size of the original struct.
pub const SIZE: usize = 0x168;

/// Undecompiled routines called by this module.
pub trait FieldHost {
    /// C: `FUN_021EFA84`, `thunk_EXT_FUN_06898020`: the thunk into `FUN_06898020(slot, 0)`
    /// (overlay 95): looks `slot` up in the overlay-95 field-effect registry (keys 0..7) and
    /// adds its handler table as an event factor of type 3; returns the factor (0 on failure).
    fn effect_create(&mut self, slot: u32) -> u32;
    /// C: `FUN_021EFAC8`, `FUN_021EFC0C`, `FUN_021EFCD4`, `thunk_EXT_FUN_06898080`: the three
    /// identical thunks into `FUN_06898080` (overlay 95), which tail-calls
    /// `EventSystem::remove` (`021d7040`) on the factor.
    fn effect_destroy(&mut self, factor: u32);
    /// C: `FUN_overlay_d_93__021b9940`: container lookup of the Pokemon with ID `poke_id`, then
    /// the "knows `mv`" check (ported as `BattleMon::knows_move`, `021d5750`).
    fn poke_knows_move(&mut self, poke_id: u32, mv: u32) -> bool;
    /// C: `FUN_02045CAC`.
    fn fun_02045cac(&mut self, a: u32, b: u32, c: u32);
    /// C: `FUN_0201EFA4`: called as `(a, index, value, mode, 0, 1)`.
    fn fun_0201efa4(&mut self, a: u32, index: u32, value: u32, mode: u32, x: u32, y: u32);
    /// C: `FUN_0201F250`.
    fn fun_0201f250(&mut self, a: u32, b: u32, c: u32);
}

/// The three words the graphics routines read from the global at `0x021F6398`
/// (`+0x1C`, `+0x08` and `+0x0C`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GfxGlobals {
    pub p1c: u32,
    pub p08: u32,
    pub p0c: u32,
}

/// The `btl_field.c` state: the static at 0x021F63F4, allocated by `new`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    weather: u32,
    weather_turns: u32,
    handle: [u32; SLOTS],
    cont: [SickCont; SLOTS],
    turns: [u32; SLOTS],
    pokes: [[u32; SLOT_POKES]; SLOTS],
    count: [u32; SLOTS],
    active: [u32; SLOTS],
}

impl Field {
    /// C: `FUN_overlay_d_93__021ef958`: the allocation is not modelled; runs `init(0)`.
    pub fn new() -> Self {
        let mut f = Field {
            weather: 0,
            weather_turns: 0,
            handle: [0; SLOTS],
            cont: [SickCont::NONE; SLOTS],
            turns: [0; SLOTS],
            pokes: [[NO_POKE; SLOT_POKES]; SLOTS],
            count: [0; SLOTS],
            active: [0; SLOTS],
        };
        f.init(0);
        f
    }

    /// C: `FUN_overlay_d_93__021ef988`, wrapper `FUN_overlay_d_93__021ef874`: resets every
    /// slot, then sets the weather word and an unlimited weather duration.
    pub fn init(&mut self, weather: u32) {
        for slot in 0..SLOTS {
            self.reset_slot(slot);
        }
        self.weather = weather;
        self.weather_turns = 0xFF;
    }

    /// C: `FUN_overlay_d_93__021ef83c`: clears one slot (the event factor is
    /// only forgotten, not removed).
    pub fn reset_slot(&mut self, slot: usize) {
        self.handle[slot] = 0;
        self.cont[slot] = SickCont::none();
        self.turns[slot] = 0;
        self.count[slot] = 0;
        self.active[slot] = 0;
        self.pokes[slot] = [NO_POKE; SLOT_POKES];
    }

    /// C: `LAB_overlay_d_93__021ef9a8`, wrapper `FUN_overlay_d_93__021ef884`: weather kind.
    pub fn weather(&self) -> u32 {
        self.weather & 0xFF
    }

    /// C: `LAB_overlay_d_93__021ef9b0`, wrapper `FUN_overlay_d_93__021ef894`: weather turns
    /// left, or 0 while there is no weather.
    pub fn weather_turns(&self) -> u32 {
        if self.weather != 0 {
            self.weather_turns
        } else {
            0
        }
    }

    /// C: `FUN_overlay_d_93__021ef9c0`, wrapper `FUN_overlay_d_93__021ef8a4`: sets both words.
    pub fn set_weather(&mut self, kind: u32, turns: u32) {
        self.weather = kind;
        self.weather_turns = turns;
    }

    /// C: unlabeled routine at `0x021EF9C8`: clears both weather words.
    pub fn clear_weather(&mut self) {
        self.weather = 0;
        self.weather_turns = 0;
    }

    /// C: `LAB_overlay_d_93__021ef9d0`, wrapper `FUN_overlay_d_93__021ef8b8`: counts the
    /// weather down (unless unlimited). Returns the kind when it just ended, else 0.
    pub fn tick_weather(&mut self) -> u32 {
        if self.weather == 0 || self.weather_turns == 0xFF {
            return 0;
        }
        self.weather_turns = self.weather_turns.wrapping_sub(1);
        if self.weather_turns != 0 {
            return 0;
        }
        let kind = self.weather & 0xFF;
        self.weather = 0;
        kind
    }

    /// C: `FUN_overlay_d_93__021efce4`, wrapper `FUN_overlay_d_93__021ef948`: slot active flag.
    pub fn is_active(&self, slot: usize) -> u32 {
        self.active[slot]
    }

    /// C: `FUN_overlay_d_93__021ef9f4`, wrapper `FUN_overlay_d_93__021ef8c8` (`with_handlers` =
    /// true): activates a slot that is
    /// not active yet: optionally registers its handlers (failing if that returns 0), stores
    /// `cont` and registers the Pokemon `cont` links to.
    pub fn add_effect<H: FieldHost>(
        &mut self,
        host: &mut H,
        slot: usize,
        cont: SickCont,
        with_handlers: bool,
    ) -> bool {
        if self.active[slot] != 0 {
            return false;
        }
        if with_handlers {
            let handle = host.effect_create(slot as u32);
            self.handle[slot] = handle;
            if handle == 0 {
                return false;
            }
        }
        self.active[slot] = 1;
        self.cont[slot] = cont;
        self.turns[slot] = 0;
        self.count[slot] = 0;
        self.pokes[slot] = [NO_POKE; SLOT_POKES];
        let linked = cont.linked_poke_id();
        if linked != NO_POKE {
            self.add_poke(slot, linked);
        }
        true
    }

    /// C: `LAB_overlay_d_93__021efa94`, wrapper `FUN_overlay_d_93__021ef8e8`: deactivates a
    /// slot, removing its event factor. Returns false if it was not active.
    pub fn remove_effect<H: FieldHost>(&mut self, host: &mut H, slot: usize) -> bool {
        if self.active[slot] == 0 {
            return false;
        }
        if self.handle[slot] != 0 {
            host.effect_destroy(self.handle[slot]);
            self.handle[slot] = 0;
        }
        self.reset_slot(slot);
        true
    }

    /// C: `FUN_overlay_d_93__021efad8`, wrapper `FUN_overlay_d_93__021ef8f8`: appends a
    /// Pokemon ID to an active slot's list (false if inactive, full or already listed).
    pub fn add_poke(&mut self, slot: usize, poke_id: u32) -> bool {
        if self.active[slot] == 0 {
            return false;
        }
        let n = self.count[slot];
        if n >= SLOT_POKES as u32 {
            return false;
        }
        for i in 0..n as usize {
            if self.pokes[slot][i] == poke_id {
                return false;
            }
        }
        self.pokes[slot][n as usize] = poke_id;
        self.count[slot] = n + 1;
        true
    }

    /// C: `FUN_overlay_d_93__021efb34`, wrapper `FUN_overlay_d_93__021ef90c`: removes a
    /// Pokemon from every active slot's list. A slot whose list becomes empty ends (factor
    /// removed, slot cleared); otherwise, if the removed Pokemon was the one the slot's
    /// [`SickCont`] links to, it is re-linked to the new first entry.
    pub fn remove_poke<H: FieldHost>(&mut self, host: &mut H, poke_id: u32) {
        for slot in 0..SLOTS {
            if self.active[slot] == 0 || self.count[slot] == 0 {
                continue;
            }
            let mut i = 0usize;
            let mut found = false;
            while i < self.count[slot] as usize {
                if self.pokes[slot][i] == poke_id {
                    found = true;
                    break;
                }
                i += 1;
            }
            if !found {
                continue;
            }
            while i < SLOT_POKES - 1 {
                self.pokes[slot][i] = self.pokes[slot][i + 1];
                i += 1;
            }
            self.pokes[slot][i] = NO_POKE;
            self.count[slot] -= 1;
            if self.count[slot] == 0 {
                if self.handle[slot] != 0 {
                    host.effect_destroy(self.handle[slot]);
                    self.handle[slot] = 0;
                }
                self.reset_slot(slot);
            } else if self.cont[slot].linked_poke_id() == poke_id {
                let first = self.pokes[slot][0] & 0xFF;
                self.cont[slot].set_linked_poke_id(first);
            }
        }
    }

    /// C: `FUN_overlay_d_93__021efc1c`, wrapper `FUN_overlay_d_93__021ef91c`: for slot 3 (Imprison), true if any listed Pokemon on the
    /// other side from `mon` knows `mv`.
    pub fn foe_knows_move<H: FieldHost, M: BattleMon>(
        &self,
        host: &mut H,
        mon: &M,
        mv: u32,
    ) -> bool {
        let own = mon.poke_id() as u32;
        for i in 0..self.count[3] as usize {
            let id = self.pokes[3][i] & 0xFF;
            if !same_side(own, id) && host.poke_knows_move(id, mv) {
                return true;
            }
        }
        false
    }

    /// C: `FUN_overlay_d_93__021efc78`, wrapper `FUN_overlay_d_93__021ef934`: advances the turn counter of slots 1..8 that have a turn
    /// limit; a slot reaching it is ended (factor removed, slot cleared) and `callback(slot,
    /// ctx)` runs.
    pub fn tick_effects<H: FieldHost>(
        &mut self,
        host: &mut H,
        mut callback: Option<&mut dyn FnMut(u32, u32)>,
        ctx: u32,
    ) {
        for slot in 1..SLOTS {
            if self.active[slot] == 0 {
                continue;
            }
            let limit = self.cont[slot].turns();
            if limit == 0 {
                continue;
            }
            self.turns[slot] = self.turns[slot].wrapping_add(1);
            if self.turns[slot] < limit {
                continue;
            }
            if self.handle[slot] != 0 {
                host.effect_destroy(self.handle[slot]);
                self.handle[slot] = 0;
            }
            self.reset_slot(slot);
            if let Some(cb) = callback.as_mut() {
                cb(slot as u32, ctx);
            }
        }
    }

    /// The original byte image (see the module layout table), for differential tests.
    pub fn image(&self) -> [u8; SIZE] {
        let mut out = [0u8; SIZE];
        let mut put = |o: usize, v: u32| out[o..o + 4].copy_from_slice(&v.to_le_bytes());
        put(0, self.weather);
        put(4, self.weather_turns);
        for s in 0..SLOTS {
            put(0x08 + 4 * s, self.handle[s]);
            put(0x28 + 4 * s, self.cont[s].0);
            put(0x48 + 4 * s, self.turns[s]);
            for p in 0..SLOT_POKES {
                put(0x68 + 0x18 * s + 4 * p, self.pokes[s][p]);
            }
            put(0x128 + 4 * s, self.count[s]);
            put(0x148 + 4 * s, self.active[s]);
        }
        out
    }
}

impl Default for Field {
    fn default() -> Self {
        Self::new()
    }
}

/// C: `FUN_overlay_d_93__021ef71c`: loads six values through the
/// graphics routines with mode 2 (`FUN_02045CAC` index 0x10), then commits.
pub fn load_gfx_a<H: FieldHost>(host: &mut H, g: GfxGlobals, a: u32, vals: [u32; 6]) {
    load_gfx(host, g, a, 0x10, 2, vals)
}

/// C: `FUN_overlay_d_93__021ef7ac`: as [`load_gfx_a`] with index 0x11 and mode 3.
pub fn load_gfx_b<H: FieldHost>(host: &mut H, g: GfxGlobals, a: u32, vals: [u32; 6]) {
    load_gfx(host, g, a, 0x11, 3, vals)
}

fn load_gfx<H: FieldHost>(
    host: &mut H,
    g: GfxGlobals,
    a: u32,
    index: u32,
    mode: u32,
    vals: [u32; 6],
) {
    host.fun_02045cac(g.p1c, index, g.p0c);
    for (i, v) in vals.iter().enumerate() {
        host.fun_0201efa4(g.p08, i as u32, *v, mode, 0, 1);
    }
    host.fun_0201f250(g.p08, a, g.p0c);
}

#[cfg(test)]
mod tests;
