//! PokeParam creation (P1c; `poke_tool.c` 0x02017430-0x02017D70): clearing a record, the plain-state
//! enter routine and the allocating constructor. Layout: see [`crate::poke_tool`].
//!
//! The initialisers (`FUN_0201756C`, `FUN_02017588`, `FUN_0201761C`, `FUN_02017430`, `FUN_02017600`), the
//! party copy `FUN_0201747C`, the stat recalculation `FUN_020185C4`, the party exit `FUN_02017D70` and the
//! level-up move scan `FUN_02018328` (with its callees `FUN_0201808C` and `FUN_020181FC`, inlined as
//! private helpers) are specified in `tools/rom-diff/specs/black-pokemon/create.spec`.
//!
//! Verified per function with `tools/check-fn.sh black-pokemon create --per-function`. A wrapper and the
//! function it calls cannot both be in [`PORTED`]: the spec stubs the callee, and a callee that is
//! ported runs as real code on the ROM side (see the notes at [`PORTED`]).

use crate::hooks::{CreateHooks, InitHooks, PokeToolHooks};
use crate::poke_tool::{
    box_get_value, box_set_value, crypt, decrypt_on_access, party_get_value, party_set_value,
    reencrypt, BLOCKS_OFS, BLOCKS_SIZE, BOX_MON_SIZE, FLAG_BOX_PLAIN, FLAG_PARTY_PLAIN,
    PARTY_EXT_OFS, PARTY_EXT_SIZE, PARTY_MON_SIZE,
};
use black_core::poke_crypt::checksum;

/// Functions of this area that are ported and recorded (read by `tools/rom-diff/svflow.py --ported-src`).
///
/// A wrapper whose callee is also ported stubs it with the `force` flag in the spec (the ROM side then keeps
/// the stub, as the port calls the callee through a hook); see `tools/rom-diff/specs/black-pokemon/create.spec`.
pub const PORTED: &[u32] = &[
    0x0201_73F0,
    0x0201_7430,
    0x0201_747C,
    0x0201_7528,
    0x0201_7550,
    0x0201_756C,
    0x0201_7588,
    0x0201_7600,
    0x0201_761C,
    0x0201_7D30,
    0x0201_7D70,
    0x0201_8328,
    0x0201_85C4,
];

/// Line number `FUN_020173F0` passes to the allocator (for its debug message).
pub const NEW_POKEMON_LINE: u32 = 0xA0;

fn rd16(b: &[u8], o: usize) -> u32 {
    u16::from_le_bytes([b[o], b[o + 1]]) as u32
}

fn rd32(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]])
}

/// C: `FUN_020173F0`: allocate a `Pokemon` (0xDC bytes, align flag 1) on heap `heap` (low 16 bits used)
/// and, if that worked, initialise it with `FUN_0201756C(mon, species, level, a2, a3)`. Returns the
/// block, or 0.
pub fn new_pokemon(
    species: u32,
    level: u32,
    a2: u32,
    a3: u32,
    heap: u32,
    h: &mut impl CreateHooks,
) -> u32 {
    let mon = h.heap_alloc(heap & 0xFFFF, PARTY_MON_SIZE as u32, 1, NEW_POKEMON_LINE);
    if mon != 0 {
        h.init_pokemon(mon, species, level, a2, a3);
    }
    mon
}

/// C: `FUN_02017528`: zero a `Pokemon` (0xDC bytes), then run the cipher over the blocks (seed: the
/// checksum halfword at 6) and the party extension (seed: the PID at 0), both zero by now: the record
/// becomes the encrypted image of an all-zero one.
pub fn clear_pokemon(mon: &mut [u8]) {
    mon[..PARTY_MON_SIZE].fill(0);
    let seed = rd16(mon, 6);
    crypt(&mut mon[BLOCKS_OFS..], BLOCKS_SIZE as u32, seed);
    let pid = rd32(mon, 0);
    crypt(&mut mon[PARTY_EXT_OFS..], PARTY_EXT_SIZE as u32, pid);
}

/// C: `FUN_02017550`: [`clear_pokemon`] for a `BoxPokemon` (0x88 bytes, no extension).
pub fn clear_box_pokemon(mon: &mut [u8]) {
    mon[..BOX_MON_SIZE].fill(0);
    let seed = rd16(mon, 6);
    crypt(&mut mon[BLOCKS_OFS..], BLOCKS_SIZE as u32, seed);
}

/// C: `FUN_02017D30`: if the "party plain" flag (flags bit 0) is clear, set bits 0 and 1 and decrypt the
/// extension (PID seed) and the blocks (checksum seed, not verified); returns whether it did so.
pub fn decrypt_party(mon: &mut [u8]) -> bool {
    let flags = rd16(mon, 4) as u16;
    if flags & FLAG_PARTY_PLAIN != 0 {
        return false;
    }
    let flags = (flags & !FLAG_PARTY_PLAIN) | FLAG_PARTY_PLAIN | FLAG_BOX_PLAIN;
    mon[4..6].copy_from_slice(&flags.to_le_bytes());
    let pid = rd32(mon, 0);
    crypt(&mut mon[PARTY_EXT_OFS..], PARTY_EXT_SIZE as u32, pid);
    let seed = rd16(mon, 6);
    crypt(&mut mon[BLOCKS_OFS..], BLOCKS_SIZE as u32, seed);
    true
}

/// Line number `FUN_02017430` passes to the allocator (for its debug message).
pub const NEW_POKEMON8_LINE: u32 = 0xBA;
/// Size of the learnset block `FUN_02018328` allocates.
pub const LEARNSET_BLOCK_SIZE: u32 = 0x68;
/// Line number `FUN_02018328` passes to the allocator.
pub const LEARNSET_BLOCK_LINE: u32 = 0x644;
/// The global scratch array 0x021462F0 (the species name is written there, then copied into the nickname).
pub const NAME_SCRATCH: u32 = 0x0214_62F0;

/// Id of the first move slot (ids 0x36-0x39), its PP (0x3A-0x3D) and PP-ups (0x3E-0x41).
const MOVE1: u32 = 0x36;

fn wr16(b: &mut [u8], o: usize, v: u32) {
    b[o..o + 2].copy_from_slice(&(v as u16).to_le_bytes());
}

/// C: `FUN_02017430(species, level, a2, a3, a4, a5, a6, heap)`: [`new_pokemon`] with the 8-argument
/// initialiser `FUN_02017588` (allocator line 0xBA); its last arguments are passed on as given.
/// Returns the block, or 0.
#[allow(clippy::too_many_arguments)]
pub fn new_pokemon8(
    species: u32,
    level: u32,
    a2: u32,
    a3: u32,
    a4: u32,
    a5: u32,
    a6: u32,
    heap: u32,
    h: &mut impl InitHooks,
) -> u32 {
    let mon = h.heap_alloc(heap & 0xFFFF, PARTY_MON_SIZE as u32, 1, NEW_POKEMON8_LINE);
    if mon != 0 {
        h.init_pokemon8(mon, species, level, a2, a3, a4, a5, a6);
    }
    mon
}

/// C: `FUN_0201756C(mon, species, level, a3, a4)`: `FUN_02017588` with `(a5, a6, a7) = (-1, 0, -1)`.
pub fn init_pokemon(mon: u32, species: u32, level: u32, a3: u32, a4: u32, h: &mut impl InitHooks) {
    h.init_pokemon8(mon, species, level, a3, a4, 0xFFFF_FFFF, 0, 0xFFFF_FFFF);
}

/// C: `FUN_02017600(mon, species, level, a3, a4)`: `FUN_0201761C` with `(a5, a6, a7) = (-1, 0, -1)`.
pub fn core_init_default(
    mon: u32,
    species: u32,
    level: u32,
    a3: u32,
    a4: u32,
    h: &mut impl InitHooks,
) {
    h.core_init_at(mon, species, level, a3, a4, 0xFFFF_FFFF, 0, 0xFFFF_FFFF);
}

/// C: `FUN_02017588(mon, species, level, a3, a4, a5, a6, a7)`: initialise a `Pokemon`: clear the record
/// ([`clear_pokemon`]), run the core initialiser `FUN_0201761C`, re-encrypt the extension with a zero
/// and then the PID seed, store the level (id 0x9E), put a fresh extension block (id 0xA7, allocated by
/// `FUN_02008F08(0x8000)` and freed again) and recalculate the stats (`FUN_020185C4`).
#[allow(clippy::too_many_arguments)]
pub fn init_pokemon_full(
    mon: &mut [u8],
    species: u32,
    level: u32,
    a3: u32,
    a4: u32,
    a5: u32,
    a6: u32,
    a7: u32,
    h: &mut impl InitHooks,
) {
    clear_pokemon(mon);
    h.core_init(species, level, a3, a4, a5, a6, a7);
    crypt(&mut mon[PARTY_EXT_OFS..], PARTY_EXT_SIZE as u32, 0);
    let pid = rd32(mon, 0);
    crypt(&mut mon[PARTY_EXT_OFS..], PARTY_EXT_SIZE as u32, pid);
    party_set_value(mon, 0x9E, level, h);
    let ext = h.ext_alloc(0x8000);
    party_set_value(mon, 0xA7, ext, h);
    h.heap_free(ext);
    h.recalc();
}

/// C: `FUN_02017F08`: the gender of a record, from its species, form and PID (callee `FUN_02017F50`,
/// [`PokeToolHooks::gender`]).
pub fn gender_of(mon: &mut [u8], h: &mut impl InitHooks) -> u32 {
    let enter = decrypt_on_access(mon);
    let species = box_get_value(mon, 5, 0, h) & 0xFFFF;
    let form = box_get_value(mon, 0x6F, 0, h);
    let pid = box_get_value(mon, 0, 0, h);
    reencrypt(mon, enter as u32);
    h.gender(species, form & 0xFFFF, pid)
}

/// C: `FUN_02017ED8(ot_id, pid)`: whether the ids make the Pokemon shiny: the xor of the four
/// halfwords is below 8.
pub fn is_shiny(ot_id: u32, pid: u32) -> bool {
    ((ot_id & 0xFFFF) ^ (ot_id >> 16) ^ (pid >> 16) ^ (pid & 0xFFFF)) < 8
}

/// C: `FUN_0201761C(mon, species, level, a3, a4, ivs, a6, a7)`: the core initialiser. Starts from a
/// cleared box record ([`clear_box_pokemon`]) and sets, through `box_set_value`: the PID (id 0; `a6` as
/// given, random for `(a7:a6) = (-1:0)`, `a3` for `(-1:1)`), the trainer ID (id 7; `a3` as given,
/// random for `(a4:a3) = (-1:-1)`, `pid + 1` skipping shiny ids for `(-1:0)`), the language (0xC),
/// species (5), the species name as the nickname (0x74), EXP for the level (8), base friendship (9),
/// the level met (0x99), origin game (0x77), met location (0x98 = 4), the six IVs (ids 0x46-0x4B: the
/// 5-bit fields of `ivs`, or random for `-1`), ability slot (10; the second ability for a PID with
/// bit 16 set if the species has two), gender (0x6E), nature (0x70: `rand(25)`), then learns the
/// starting moves (`FUN_02017FB0`).
#[allow(clippy::too_many_arguments)]
pub fn init_core(
    mon: &mut [u8],
    species: u32,
    level: u32,
    a3: u32,
    a4: u32,
    ivs: u32,
    a6: u32,
    a7: u32,
    h: &mut impl InitHooks,
) {
    clear_box_pokemon(mon);
    let enter = decrypt_on_access(mon);
    let rec = h.personal_record(species, 0);
    let mut pid = a6;
    if a6 == 0 && a7 == 0xFFFF_FFFF {
        pid = h.rand(0);
        h.mt_next();
    } else if a6 == 1 && a7 == 0xFFFF_FFFF {
        pid = a3;
    }
    box_set_value(mon, 0, pid, h);
    let mut ot = a3;
    if a4 == 0xFFFF_FFFF && a3 == 0xFFFF_FFFF {
        ot = h.rand(0);
    } else if a4 == 0xFFFF_FFFF && a3 == 0 {
        ot = pid.wrapping_add(1);
        while is_shiny(ot, pid) {
            ot = ot.wrapping_add(1);
        }
    }
    box_set_value(mon, 7, ot, h);
    let language = h.game_language();
    box_set_value(mon, 0xC, language, h);
    box_set_value(mon, 5, species, h);
    h.species_name_to_scratch(species);
    box_set_value(mon, 0x74, NAME_SCRATCH, h);
    let exp = h.exp_for_level(species, 0, level);
    box_set_value(mon, 8, exp, h);
    let friendship = h.personal_field(rec, 0x16);
    box_set_value(mon, 9, friendship, h);
    box_set_value(mon, 0x99, level, h);
    let version = h.game_version();
    box_set_value(mon, 0x77, version, h);
    box_set_value(mon, 0x98, 4, h);
    if ivs == 0xFFFF_FFFF {
        for id in [0x46, 0x47, 0x48, 0x4A, 0x4B, 0x49] {
            let r = h.mt_next();
            box_set_value(mon, id, r >> 27, h);
        }
    } else {
        for (id, shift) in [
            (0x46, 0),
            (0x47, 5),
            (0x48, 10),
            (0x4A, 15),
            (0x4B, 20),
            (0x49, 25),
        ] {
            box_set_value(mon, id, (ivs >> shift) & 0x1F, h);
        }
    }
    // C: `FUN_02019C7C` inline: two ability slots if field 0x1B is nonzero
    let two_abilities = h.personal_field(rec, 0x1B) != 0;
    let field = if two_abilities && pid & 0x1_0000 != 0 {
        0x1B
    } else {
        0x1A
    };
    let ability = h.personal_field(rec, field);
    box_set_value(mon, 10, ability, h);
    let gender = gender_of(mon, h);
    box_set_value(mon, 0x6E, gender, h);
    let nature = h.rand(0x19);
    box_set_value(mon, 0x70, nature, h);
    h.learn_start_moves();
    reencrypt(mon, enter as u32);
}

/// C: `FUN_02017D70(mon, flag)`: the inverse of [`decrypt_party`]: only if the "party plain" flag is set
/// and `flag == 1`, clear both plain flags, encrypt the extension (PID seed), store the checksum of the
/// blocks and encrypt them with it. Returns 1 if it did, else 0.
pub fn encrypt_party(mon: &mut [u8], flag: u32) -> u32 {
    let flags = rd16(mon, 4);
    if flags & 1 != 1 || flag != 1 {
        return 0;
    }
    let flags = flags & !(FLAG_PARTY_PLAIN as u32);
    let flags = flags & !(FLAG_BOX_PLAIN as u32);
    wr16(mon, 4, flags);
    let pid = rd32(mon, 0);
    crypt(&mut mon[PARTY_EXT_OFS..], PARTY_EXT_SIZE as u32, pid);
    let sum = checksum(&mon[BLOCKS_OFS..], BLOCKS_SIZE as u32) as u32;
    wr16(mon, 6, sum);
    crypt(&mut mon[BLOCKS_OFS..], BLOCKS_SIZE as u32, sum);
    1
}

/// C: `FUN_02018438`: the level of a record from its species, form and EXP (callee `FUN_0201850C`,
/// [`PokeToolHooks::level_from_exp`]).
pub fn level_of(mon: &mut [u8], h: &mut impl PokeToolHooks) -> u32 {
    let enter = decrypt_on_access(mon);
    let species = box_get_value(mon, 5, 0, h);
    let form = box_get_value(mon, 0x6F, 0, h);
    let exp = box_get_value(mon, 8, 0, h);
    reencrypt(mon, enter as u32);
    h.level_from_exp(species & 0xFFFF, form & 0xFFFF, exp)
}

/// C: `FUN_020185C4(mon)`: after a change of EXP: enter ([`decrypt_party`]), store the level from the EXP
/// (id 0x9E), recalculate the stats (`FUN_02018A7C`) and leave ([`encrypt_party`]).
pub fn recalc(mon: &mut [u8], h: &mut impl InitHooks) {
    let entered = decrypt_party(mon);
    let level = level_of(mon, h);
    party_set_value(mon, 0x9E, level, h);
    h.recalc_stats();
    encrypt_party(mon, entered as u32);
}

/// C: `FUN_0201747C(mon, heap)`: a copy of a Pokemon. `new` is the memory of the block the allocator
/// returns (it stands for that block: the original allocates it with [`new_pokemon`]). A new Pokemon with
/// the species, level (id 0x9E, read through the box accessor) and trainer ID of `mon` is created, the
/// first 0x88 bytes of `mon` are copied over it, ids 0x9D/0xA0/0xA1 are zeroed, a fresh extension block
/// is put (id 0xA7) and the stats recalculated (`FUN_020185C4`). Returns the block's handle.
///
/// The hooks' record base is the new block (the only record they get offsets into). An allocation
/// failure is not modelled (the original would copy to address 0): it panics.
pub fn copy_pokemon_new(mon: &mut [u8], heap: u32, new: &mut [u8], h: &mut impl InitHooks) -> u32 {
    let enter = decrypt_on_access(mon);
    let species = box_get_value(mon, 5, 0, h) & 0xFFFF;
    let level = box_get_value(mon, 0x9E, 0, h) & 0xFFFF;
    let ot = box_get_value(mon, 7, 0, h);
    let block = new_pokemon(species, level, ot, 0, heap, h);
    assert_ne!(block, 0, "allocation failure is not modelled");
    reencrypt(mon, enter as u32);
    new[..BOX_MON_SIZE].copy_from_slice(&mon[..BOX_MON_SIZE]);
    party_set_value(new, 0x9D, 0, h);
    party_set_value(new, 0xA0, 0, h);
    party_set_value(new, 0xA1, 0, h);
    let ext = h.ext_alloc(((heap & 0x7FFF) | 0x8000) & 0xFFFF);
    party_set_value(new, 0xA7, ext, h);
    h.heap_free(ext);
    h.recalc_at(block);
    block
}

/// C: `FUN_020181FC(mon, move, slot)`: put `move` in move slot `slot` with no PP-ups and full PP.
fn set_move_slot(mon: &mut [u8], mv: u32, slot: u32, h: &mut impl InitHooks) {
    box_set_value(mon, MOVE1 + slot, mv, h);
    box_set_value(mon, MOVE1 + 8 + slot, 0, h);
    let pp = if mv != 0 { h.move_max_pp(mv, 0) } else { 0 };
    box_set_value(mon, MOVE1 + 4 + slot, pp, h);
}

/// C: `FUN_0201808C` (thunk of `FUN_02018094`): teach `mv` into the first empty move slot (returns `mv`),
/// or 0xFFFE if the record knows it already, or 0xFFFF if all four slots hold other moves.
fn learn_into_free_slot(mon: &mut [u8], mv: u32, h: &mut impl InitHooks) -> u32 {
    let mut result = 0xFFFF;
    let enter = decrypt_on_access(mon);
    for slot in 0..4 {
        let known = box_get_value(mon, MOVE1 + slot, 0, h) & 0xFFFF;
        if known == 0 {
            set_move_slot(mon, mv, slot, h);
            result = mv;
            break;
        }
        if known == mv {
            result = 0xFFFE;
            break;
        }
    }
    reencrypt(mon, enter as u32);
    result
}

/// C: `FUN_02018328(mon, cursor, heap)`: the next level-up move. Loads the learnset of the Pokemon's
/// species and form into a freshly allocated 0x68-byte block (pairs of (move, level) halfwords ending
/// in (0xFFFF, 0xFFFF)) and walks it from `*cursor`, advancing the cursor: entries of another level are
/// skipped; for an entry of the Pokemon's level the move is taught into a free slot
/// ([`learn_into_free_slot`]): taught (returns the move) or all slots full (returns the move with bit
/// 15 set) end the scan, an already known move continues it. Returns 0 at the end of the list (or
/// 0xFFFE if the list ended right after an already known move).
pub fn next_level_up_move(
    mon: &mut [u8],
    cursor: &mut u32,
    heap: u32,
    h: &mut impl InitHooks,
) -> u32 {
    let block = h.heap_alloc(
        ((heap & 0x7FFF) | 0x8000) & 0xFFFF,
        LEARNSET_BLOCK_SIZE,
        0,
        LEARNSET_BLOCK_LINE,
    );
    let species = party_get_value(mon, 5, 0, h) & 0xFFFF;
    let form = party_get_value(mon, 0x6F, 0, h);
    let level = party_get_value(mon, 0x9E, 0, h) & 0xFF;
    h.load_learnset(species, form & 0xFFFF, block);
    let mut result = 0;
    loop {
        let (mv, lvl) = h.learnset_entry(block, *cursor);
        if mv == 0xFFFF && lvl == 0xFFFF {
            break;
        }
        if level == lvl {
            result = learn_into_free_slot(mon, mv, h);
            if result == 0xFFFF {
                result = mv | 0x8000;
                *cursor = cursor.wrapping_add(1);
                break;
            }
            if result != 0xFFFE {
                *cursor = cursor.wrapping_add(1);
                break;
            }
        }
        *cursor = cursor.wrapping_add(1);
    }
    h.heap_free(block);
    result
}

#[cfg(test)]
mod tests;
