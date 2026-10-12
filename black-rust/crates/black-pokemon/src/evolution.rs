//! Evolution check (P5; `shinka_check.c`, `unk_0201AF6C.s` 0x0201B2B0-0x0201B7A0, T3-FINDINGS.md (a)).
//!
//! `FUN_0201B2B0` walks the seven 6-byte records of a species' ARC 19 member (`u16 method`, `u16 param`,
//! `u16 target`) and returns the target species of the first record whose method case holds. The method
//! switch has 29 cases (jump table 0x0201B3D4, code 0x0201B3B8..0x0201B5C4); trade (mode 1) and item use
//! (modes 2/3) have their own loops. [`evolution_check`] is the whole function; `FUN_0201B3B8` and
//! `FUN_0201B406` that Ghidra lists are the loop head and a case label inside it.
//!
//! The record loaders `0201B70C` / `B718` / `B764` and the friendship routines `0201B7A0`, `B7B4`, `B7EC`,
//! `B818` (callers of `B850`) and `B850` itself ([`friendship_adjust`]). Ghidra's pseudo-C of `0201B850` is
//! wrong (it shows a 30-byte tail call); the port follows the disassembly (rizin).
//!
//! Deviations: the four callers of `B850` take the mon as an address and call it through
//! [`EvolutionHooks::friendship_adjust_at`] (their specs stub `B850`, so the port must go through the same
//! call); [`record_field`] returns 0 for an index past the record array instead of reading the original's
//! stack past its end (no caller does that).
//!
//! Verified per function with `tools/check-fn.sh black-pokemon evolution --per-function`. The Pokemon
//! getters are [`crate::poke_tool`]; the data and time services come in through [`EvolutionHooks`], so the
//! crate stays free of `nds-rom` / `black-data` (D6).

use crate::hooks::PokeToolHooks;
use crate::poke_tool::{party_get_value, party_set_value};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0201_B2B0,
    0x0201_B764,
    0x0201_B774,
    0x0201_B70C,
    0x0201_B718,
    0x0201_B7A0,
    0x0201_B7B4,
    0x0201_B7EC,
    0x0201_B818,
    0x0201_B850,
];

/// Delta table of the friendship routines: the address of a constant table in the ROM (passed as a pointer) or
/// the three bytes the caller built on its stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeltaTable {
    Addr(u32),
    Bytes([u8; 3]),
}

/// ROM tables of the friendship routines (`0201B7A0`: `0x0209E45C + 3 * idx`, `B7EC`: `0x0209E465`,
/// `B818`: `0x0209E462`).
pub const FRIENDSHIP_TABLE_BASE: u32 = 0x0209_E45C;
pub const FRIENDSHIP_TABLE_RANDOM: u32 = 0x0209_E465;
pub const FRIENDSHIP_TABLE_PARTY: u32 = 0x0209_E462;
/// Species the friendship routine skips (species 0 and 0x28A).
pub const SPECIES_NO_FRIENDSHIP: u32 = 0x28A;
/// Held-item effect that multiplies a positive delta by 1.5 (`FUN_0209C0A4(delta * 150, 100)`).
pub const HOLD_EFFECT_FRIENDSHIP_BOOST: u32 = 0x35;
/// Ball id (field 0x98) that adds one to a positive delta.
pub const BALL_ADDS_FRIENDSHIP: u32 = 11;

/// Records per ARC 19 member.
pub const RECORDS: usize = 7;
/// Bytes per record (`u16` method, `u16` param, `u16` target).
pub const RECORD_SIZE: usize = 6;
/// Size of an ARC 19 member (and of the heap block `FUN_0201B2B0` allocates for it).
pub const MEMBER_SIZE: usize = RECORDS * RECORD_SIZE;
/// ARC id of the evolution records.
pub const EVOLUTION_ARC: u32 = 0x13;
/// Address of the `"shinka_check.c"` string passed to the allocator.
pub const FILE_SHINKA_CHECK: u32 = 0x020A_7330;
/// Line number `FUN_0201B2B0` passes to the allocator.
pub const ALLOC_LINE: u32 = 0x6A;
/// Held-item effect that blocks evolution (Everstone), result of `FUN_02020FE4(item, 1, heap)`.
pub const HOLD_EFFECT_NO_EVOLVE: u32 = 0x40;
/// Species the Everstone block does not apply to (64).
pub const SPECIES_EXEMPT_FROM_BLOCK: u32 = 0x40;
/// Friendship needed by methods 1-3.
pub const FRIENDSHIP_EVOLVE: u32 = 220;

/// Evolution modes (argument 3 of `FUN_0201B2B0`).
pub mod mode {
    pub const LEVEL_UP: u32 = 0;
    pub const TRADE: u32 = 1;
    pub const ITEM: u32 = 2;
    /// Item use that ignores the held-item block.
    pub const ITEM_FORCED: u32 = 3;
}

/// Evolution methods (`u16` at the start of a record): the 29 cases of the switch.
pub mod method {
    pub const NONE: u32 = 0;
    pub const FRIENDSHIP: u32 = 1;
    pub const FRIENDSHIP_DAY: u32 = 2;
    pub const FRIENDSHIP_NIGHT: u32 = 3;
    pub const LEVEL: u32 = 4;
    pub const TRADE: u32 = 5;
    pub const TRADE_HELD_ITEM: u32 = 6;
    pub const TRADE_FOR_SPECIES: u32 = 7;
    pub const USE_ITEM: u32 = 8;
    pub const LEVEL_ATK_GT_DEF: u32 = 9;
    pub const LEVEL_ATK_EQ_DEF: u32 = 10;
    pub const LEVEL_ATK_LT_DEF: u32 = 11;
    pub const LEVEL_PERSONALITY_LOW: u32 = 12;
    pub const LEVEL_PERSONALITY_HIGH: u32 = 13;
    pub const LEVEL_EXTRA: u32 = 14;
    /// Sets the out kind to 15 (the free-slot clone case) but never matches in the level-up loop.
    pub const SPAWN_MARKER: u32 = 15;
    pub const BEAUTY: u32 = 16;
    pub const USE_ITEM_MALE: u32 = 17;
    pub const USE_ITEM_FEMALE: u32 = 18;
    pub const HELD_ITEM_DAY: u32 = 19;
    pub const HELD_ITEM_NIGHT: u32 = 20;
    pub const KNOWS_MOVE: u32 = 21;
    pub const PARTY_SPECIES: u32 = 22;
    pub const LEVEL_MALE: u32 = 23;
    pub const LEVEL_FEMALE: u32 = 24;
    pub const LOCATION_A: u32 = 25;
    pub const LOCATION_B: u32 = 26;
    pub const LOCATION_C: u32 = 27;
    pub const LOCATION_D: u32 = 28;
    /// Number of cases of the switch; larger values match nothing.
    pub const COUNT: u32 = 29;
}

/// Location ids of the four location methods (`u16` tables at 0x020A7324, 0x020A7322, 0x020A7320, 0x020A732A).
pub const LOCATIONS_A: [u32; 3] = [195, 196, 197];
pub const LOCATIONS_B: [u32; 1] = [155];
pub const LOCATIONS_C: [u32; 1] = [203];
pub const LOCATIONS_D: [u32; 3] = [195, 196, 197];

/// Species checked by the trade-for-species method (species ids 588 and 616; the swap-evolution pair).
pub const TRADE_SPECIES_A: u32 = 0x24C;
pub const TRADE_SPECIES_B: u32 = 0x268;

/// Pokemon field ids used by the evolution check (see [`crate::poke_tool::id`]).
mod fid {
    pub const PID: u32 = 0;
    pub const SPECIES: u32 = 5;
    pub const ITEM: u32 = 6;
    pub const FRIENDSHIP: u32 = 9;
    pub const BEAUTY: u32 = 0x14;
    pub const MOVE0: u32 = 0x36;
    pub const IS_EGG: u32 = 0x4C;
    pub const GENDER: u32 = 0x6E;
    pub const LEVEL: u32 = 0x9E;
    pub const ATTACK: u32 = 0xA2;
    pub const DEFENSE: u32 = 0xA3;
    pub const REAL_SPECIES: u32 = 0xAB;
}

/// Callees of the evolution routines that belong to other areas or to the archive/heap/time services.
///
/// The differential test stubs them identically on both sides (the spec's stub lines), so only their
/// arguments are verified here. The Pokemon getters are real code ([`crate::poke_tool`]).
pub trait EvolutionHooks {
    /// Callee `FUN_02020FE4(item, 1, heap)`: the held-item effect id of `item` (item data param 1).
    fn hold_effect(&mut self, item: u32, heap: u32) -> u32;

    /// Callee `FUN_02011410(tod)`: 1 during the night-like time-of-day periods (3 and 4), else 0.
    fn is_night(&mut self, tod: u32) -> u32;

    /// Callee `FUN_0201AC44(party, species)`: 1 if some member of `party` is of `species`.
    fn party_has_species(&mut self, party: u32, species: u32) -> u32;

    /// Callee `FUN_02030734(heap, size, align, file, line)`: allocate a block, returns its address.
    fn heap_alloc(&mut self, heap: u32, size: u32, align: u32, file: u32, line: u32) -> u32;

    /// Callee `FUN_020307B0(block)`: free a block.
    fn heap_free(&mut self, block: u32);

    /// Callee `ArcLoadMember(dst, 0x13, species)` (`FUN_02048F4C`): load ARC 19 member `species` into the block
    /// at address `dst`. The block is opaque here, so the callee hands the 42 bytes back.
    fn load_evolution(&mut self, dst: u32, species: u32) -> [u8; MEMBER_SIZE];

    /// Callee `FUN_02013E4C(location)`: the map-location id the location resolves to (compared with a mon's met
    /// location in [`friendship_adjust`]; nonzero = it applies).
    fn location_met(&mut self, location: u32) -> u32;

    /// Callee `FUN_02011014(delta)`: the event friendship bonus added to the delta.
    fn event_bonus(&mut self, delta: u32) -> u32;

    /// Callee `FUN_02020FE4(item, param, heap)`: item data parameter `param` of `item` (`param` 1 is the
    /// held-item effect, 0x3C..0x3E the friendship deltas).
    fn item_param(&mut self, item: u32, param: u32, heap: u32) -> u32;

    /// Callee `FUN_0203F0A8()`: a random word (the friendship routine takes its top bit).
    fn random_word(&mut self) -> u32;

    /// Callee `FUN_020490F4(arc, member)`: allocate and load an archive member; returns the block address.
    fn alloc_load(&mut self, arc: u32, member: u32) -> u32;

    /// Callee `FUN_0204932C(handle, member, out)`: read the 42-byte member `member` of archive `handle`.
    fn read_member(&mut self, handle: u32, member: u32) -> [u8; MEMBER_SIZE];

    /// Callee `FUN_0201AA18(party)`: number of party members.
    fn party_count(&mut self, party: u32) -> u32;

    /// Callee `FUN_0201AB28(party, idx)`: address of party member `idx`.
    fn party_member(&mut self, party: u32, idx: u32) -> u32;

    /// Callee `FUN_0201B850(mon, table, location, heap)` as the callers see it (their specs stub it): the
    /// production side runs [`friendship_adjust`] on the mon at `mon`.
    fn friendship_adjust_at(&mut self, mon: u32, table: DeltaTable, location: u32, heap: u32);
}

/// C: `FUN_0201B718(handle, member, idx, kind)`: field `kind` of record `idx` of an archive member read through
/// `FUN_0204932C`: the `u16` method (kind 0), param (1) or target (2); 0 for any other kind.
pub fn record_field(
    handle: u32,
    member: u32,
    idx: u32,
    kind: u32,
    h: &mut impl EvolutionHooks,
) -> u32 {
    let buf = h.read_member(handle, member);
    if kind > 2 {
        return 0;
    }
    // the original reads its 44-byte stack array past the record for idx > 6 (never called that way); 0 here
    let k = (idx * 3 + kind) as usize;
    buf.get(2 * k..2 * k + 2)
        .map_or(0, |b| u16::from_le_bytes([b[0], b[1]]) as u32)
}

/// C: `FUN_0201B70C(species)`: alloc-load ARC 19 member `species` (tail call `FUN_020490F4(0x13, species)`).
pub fn alloc_load_record(species: u32, h: &mut impl EvolutionHooks) -> u32 {
    h.alloc_load(EVOLUTION_ARC, species)
}

/// C: `FUN_0201B850(mon, table, location, heap)`: the friendship adjustment. The delta is the table byte for
/// the friendship level (`< 100`, `< 200`, `>= 200`); +1 for a positive delta when the ball is id 11 (field
/// 0x98), +1 for a positive delta when the mon's met location (0x96) equals `FUN_02013E4C(location)` (nonzero);
/// with the held effect 0x35 a positive delta becomes `delta * 150 / 100`; then `FUN_02011014(delta)` is added
/// to the friendship (field 9), clamped to 0..=255 and stored. Species 0 and 0x28A are skipped.
pub fn friendship_adjust<H: EvolutionHooks + PokeToolHooks>(
    mon: &mut [u8],
    table: &[u8],
    location: u32,
    heap: u32,
    h: &mut H,
) {
    let met = h.location_met(location);
    let species = party_get_value(mon, fid::REAL_SPECIES, 0, h) & 0xFFFF;
    if species == 0 || species == SPECIES_NO_FRIENDSHIP {
        return;
    }
    let item = party_get_value(mon, fid::ITEM, 0, h) & 0xFFFF;
    let hold = h.hold_effect(item, heap) & 0xFF;
    let friendship = party_get_value(mon, fid::FRIENDSHIP, 0, h);
    let mut idx = 0;
    if friendship >= 100 {
        idx += 1;
    }
    if friendship >= 200 {
        idx += 1;
    }
    let mut delta = table[idx] as i8 as i32;
    if delta > 0 && party_get_value(mon, 0x98, 0, h) == BALL_ADDS_FRIENDSHIP {
        delta += 1;
    }
    if met != 0 && delta > 0 && party_get_value(mon, 0x96, 0, h) == met {
        delta += 1;
    }
    if delta > 0 && hold == HOLD_EFFECT_FRIENDSHIP_BOOST {
        delta = (150 * delta) / 100;
    }
    let bonus = h.event_bonus(delta as u32);
    let f = friendship.wrapping_add(bonus) as i32;
    let f = f.clamp(0, 255);
    party_set_value(mon, fid::FRIENDSHIP, f as u32, h);
}

/// C: `FUN_0201B7A0(mon, idx, location, heap)`: [`friendship_adjust`] with the delta table `0x0209E45C + 3 * idx`
/// (the table is the callee's argument; the callee is a stub in the spec).
pub fn friendship_by_index(
    mon: u32,
    idx: u32,
    location: u32,
    heap: u32,
    h: &mut impl EvolutionHooks,
) {
    let table = FRIENDSHIP_TABLE_BASE.wrapping_add(idx.wrapping_mul(3));
    h.friendship_adjust_at(mon, DeltaTable::Addr(table), location, heap);
}

/// C: `FUN_0201B7B4(mon, item, location, heap)`: the three deltas are the item's parameters `0x3C..=0x3E`
/// (kept as bytes), then [`friendship_adjust`] with them.
pub fn friendship_by_item(
    mon: u32,
    item: u32,
    location: u32,
    heap: u32,
    h: &mut impl EvolutionHooks,
) {
    let mut t = [0u8; 3];
    for (i, b) in t.iter_mut().enumerate() {
        *b = h.item_param(item, (0x3C + i as u32) & 0xFFFF, heap) as u8;
    }
    h.friendship_adjust_at(mon, DeltaTable::Bytes(t), location, heap);
}

/// C: `FUN_0201B7EC(mon, location, heap)`: with probability 1/2 (the top bit of [`EvolutionHooks::random_word`]
/// clear) [`friendship_adjust`] with the table `0x0209E465`.
pub fn friendship_random(mon: u32, location: u32, heap: u32, h: &mut impl EvolutionHooks) {
    if (h.random_word() as i32) > -1 {
        h.friendship_adjust_at(
            mon,
            DeltaTable::Addr(FRIENDSHIP_TABLE_RANDOM),
            location,
            heap,
        );
    }
}

/// C: `FUN_0201B818(party, location, heap)`: [`friendship_adjust`] with the table `0x0209E462` on every party
/// member; the count is read again after each member, as the original does.
pub fn friendship_party(party: u32, location: u32, heap: u32, h: &mut impl EvolutionHooks) {
    let mut i = 0;
    let mut n = h.party_count(party) as i32;
    while i < n {
        let m = h.party_member(party, i as u32);
        h.friendship_adjust_at(m, DeltaTable::Addr(FRIENDSHIP_TABLE_PARTY), location, heap);
        i += 1;
        n = h.party_count(party) as i32;
    }
}

fn rd16(b: &[u8], o: usize) -> u32 {
    u16::from_le_bytes([b[o], b[o + 1]]) as u32
}

/// Result of [`evolution_check`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Evolution {
    /// Target species, 0 if none.
    pub target: u32,
    /// What the original stores through its out pointer, if anything: the winning method, or
    /// [`method::SPAWN_MARKER`] when that case was met (the last store wins).
    pub kind: Option<u32>,
}

/// C: `FUN_0201B764`: `ArcLoadMember(dst, 0x13, species)`: load the evolution member of `species` into the block
/// at `dst` (tail call; the arguments are `(species, dst)`).
pub fn load_record(species: u32, dst: u32, h: &mut impl EvolutionHooks) -> [u8; MEMBER_SIZE] {
    h.load_evolution(dst, species)
}

/// C: `FUN_0201B774`: 1 if one of the four move slots (field ids 0x36..=0x39) equals `mv`, else 0.
pub fn knows_move(mon: &mut [u8], mv: u32, h: &mut dyn PokeToolHooks) -> u32 {
    for id in fid::MOVE0..fid::MOVE0 + 4 {
        if mv == party_get_value(mon, id, 0, h) {
            return 1;
        }
    }
    0
}

/// Arguments of [`evolution_check`] besides the mon and the hooks.
#[derive(Debug, Clone, Copy)]
pub struct EvolutionArgs {
    /// The party (for method 22), an opaque address; 0 = none.
    pub party: u32,
    /// [`mode`].
    pub mode: u32,
    /// Mode 0: location id; modes 2/3: the used item; mode 1: unused (the partner is passed separately).
    pub param: u32,
    /// Time-of-day word handed to `FUN_02011410`.
    pub tod: u32,
    /// Heap id.
    pub heap: u32,
}

fn location_hit(table: &[u32], param: u32) -> bool {
    table.contains(&param)
}

/// C: `FUN_0201B2B0` (with the walk `FUN_0201B3B8` inside it): the target species `mon` evolves into, 0 if it
/// does not, for [`mode`] `a.mode`. `partner` is the trade partner (mode 1 only).
///
/// Order of the Pokemon reads is the original's, since the record cipher state is shared between reads:
/// egg flag, species, held item, PID, beauty, then (mode 0) level and friendship.
pub fn evolution_check<H: EvolutionHooks + PokeToolHooks>(
    mon: &mut [u8],
    partner: Option<&mut [u8]>,
    a: EvolutionArgs,
    h: &mut H,
) -> Evolution {
    let none = Evolution::default();
    if party_get_value(mon, fid::IS_EGG, 0, h) != 0 {
        return none;
    }
    let species = party_get_value(mon, fid::SPECIES, 0, h) & 0xFFFF;
    let item = party_get_value(mon, fid::ITEM, 0, h) & 0xFFFF;
    let pid = party_get_value(mon, fid::PID, 0, h);
    let beauty = party_get_value(mon, fid::BEAUTY, 0, h) & 0xFF;
    let pid_hi = pid >> 16;
    let hold = h.hold_effect(item, a.heap) & 0xFF;
    // the Everstone block: species 64 is exempt, and so is item use mode 3
    if species != SPECIES_EXEMPT_FROM_BLOCK
        && hold == HOLD_EFFECT_NO_EVOLVE
        && a.mode != mode::ITEM_FORCED
    {
        return none;
    }
    let blk = h.heap_alloc(a.heap, MEMBER_SIZE as u32, 0, FILE_SHINKA_CHECK, ALLOC_LINE);
    let rec = load_record(species, blk, h);
    let mut out = none;
    if a.mode <= mode::ITEM_FORCED {
        match a.mode {
            mode::LEVEL_UP => out = walk_level_up(mon, &rec, pid_hi, beauty, item, &a, h),
            mode::TRADE => {
                let partner = partner.expect("trade mode needs the partner");
                out = walk_trade(mon, partner, &rec, item, &a, h);
            }
            _ => out = walk_item(mon, &rec, &a, h),
        }
    }
    h.heap_free(blk);
    out
}

/// The record walk of mode 0 (level-up family): the 29-case switch of `FUN_0201B3B8`.
fn walk_level_up<H: EvolutionHooks + PokeToolHooks>(
    mon: &mut [u8],
    rec: &[u8; MEMBER_SIZE],
    pid_hi: u32,
    beauty: u32,
    item: u32,
    a: &EvolutionArgs,
    h: &mut H,
) -> Evolution {
    let level = party_get_value(mon, fid::LEVEL, 0, h) & 0xFF;
    let friendship = party_get_value(mon, fid::FRIENDSHIP, 0, h) & 0xFFFF;
    let rem = pid_hi % 10;
    let mut kind = None;
    for i in 0..RECORDS {
        let m = rd16(rec, i * RECORD_SIZE);
        let p = rd16(rec, i * RECORD_SIZE + 2);
        let t = rd16(rec, i * RECORD_SIZE + 4);
        let hit = match m {
            method::FRIENDSHIP => friendship >= FRIENDSHIP_EVOLVE,
            method::FRIENDSHIP_DAY => h.is_night(a.tod) == 0 && friendship >= FRIENDSHIP_EVOLVE,
            method::FRIENDSHIP_NIGHT => h.is_night(a.tod) == 1 && friendship >= FRIENDSHIP_EVOLVE,
            method::LEVEL | method::LEVEL_EXTRA => p <= level,
            method::LEVEL_ATK_GT_DEF => {
                p <= level && {
                    let atk = party_get_value(mon, fid::ATTACK, 0, h);
                    let def = party_get_value(mon, fid::DEFENSE, 0, h);
                    atk > def
                }
            }
            method::LEVEL_ATK_EQ_DEF => {
                p <= level && {
                    let atk = party_get_value(mon, fid::ATTACK, 0, h);
                    let def = party_get_value(mon, fid::DEFENSE, 0, h);
                    atk == def
                }
            }
            method::LEVEL_ATK_LT_DEF => {
                p <= level && {
                    let atk = party_get_value(mon, fid::ATTACK, 0, h);
                    let def = party_get_value(mon, fid::DEFENSE, 0, h);
                    atk < def
                }
            }
            method::LEVEL_PERSONALITY_LOW => p <= level && rem < 5,
            method::LEVEL_PERSONALITY_HIGH => p <= level && rem >= 5,
            method::SPAWN_MARKER => {
                kind = Some(15);
                false
            }
            method::BEAUTY => p <= beauty,
            method::HELD_ITEM_DAY => h.is_night(a.tod) == 0 && item == p,
            method::HELD_ITEM_NIGHT => h.is_night(a.tod) == 1 && item == p,
            method::KNOWS_MOVE => knows_move(mon, p, h) == 1,
            method::PARTY_SPECIES => a.party != 0 && h.party_has_species(a.party, p) == 1,
            method::LEVEL_MALE => party_get_value(mon, fid::GENDER, 0, h) == 0 && p <= level,
            method::LEVEL_FEMALE => party_get_value(mon, fid::GENDER, 0, h) == 1 && p <= level,
            method::LOCATION_A => location_hit(&LOCATIONS_A, a.param),
            method::LOCATION_B => location_hit(&LOCATIONS_B, a.param),
            method::LOCATION_C => location_hit(&LOCATIONS_C, a.param),
            method::LOCATION_D => location_hit(&LOCATIONS_D, a.param),
            _ => false,
        };
        if hit && t != 0 {
            return Evolution {
                target: t,
                kind: Some(m),
            };
        }
    }
    Evolution { target: 0, kind }
}

/// The record walk of mode 1 (trade): methods 5, 6 and 7.
fn walk_trade<H: EvolutionHooks + PokeToolHooks>(
    mon: &mut [u8],
    partner: &mut [u8],
    rec: &[u8; MEMBER_SIZE],
    item: u32,
    a: &EvolutionArgs,
    h: &mut H,
) -> Evolution {
    for i in 0..RECORDS {
        let m = rd16(rec, i * RECORD_SIZE);
        let p = rd16(rec, i * RECORD_SIZE + 2);
        let t = rd16(rec, i * RECORD_SIZE + 4);
        let hit = match m {
            method::TRADE => true,
            method::TRADE_HELD_ITEM => item == p,
            method::TRADE_FOR_SPECIES => {
                let partner_item = party_get_value(partner, fid::ITEM, 0, h) & 0xFFFF;
                if h.hold_effect(partner_item, a.heap) == HOLD_EFFECT_NO_EVOLVE {
                    false
                } else if party_get_value(mon, fid::REAL_SPECIES, 0, h) == TRADE_SPECIES_A
                    && party_get_value(partner, fid::REAL_SPECIES, 0, h) == TRADE_SPECIES_B
                {
                    true
                } else {
                    party_get_value(partner, fid::REAL_SPECIES, 0, h) == TRADE_SPECIES_A
                        && party_get_value(mon, fid::REAL_SPECIES, 0, h) == TRADE_SPECIES_B
                }
            }
            _ => false,
        };
        if hit && t != 0 {
            return Evolution {
                target: t,
                kind: Some(m),
            };
        }
    }
    Evolution::default()
}

/// The record walk of modes 2 and 3 (item use): methods 8, 17 and 18 with `a.param` = the used item.
fn walk_item<H: EvolutionHooks + PokeToolHooks>(
    mon: &mut [u8],
    rec: &[u8; MEMBER_SIZE],
    a: &EvolutionArgs,
    h: &mut H,
) -> Evolution {
    for i in 0..RECORDS {
        let m = rd16(rec, i * RECORD_SIZE);
        let p = rd16(rec, i * RECORD_SIZE + 2);
        let t = rd16(rec, i * RECORD_SIZE + 4);
        let mut r = 0;
        if m == method::USE_ITEM && a.param == p {
            r = t;
        }
        if m == method::USE_ITEM_MALE
            && party_get_value(mon, fid::GENDER, 0, h) == 0
            && a.param == p
        {
            r = t;
        }
        if m == method::USE_ITEM_FEMALE
            && party_get_value(mon, fid::GENDER, 0, h) == 1
            && a.param == p
        {
            r = t;
        }
        if r != 0 {
            return Evolution {
                target: r,
                kind: Some(m),
            };
        }
    }
    Evolution::default()
}

#[cfg(test)]
mod tests;
