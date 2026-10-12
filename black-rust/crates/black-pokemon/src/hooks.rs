//! Undecompiled routines called by the ported Pokémon code, modelled as traits.

/// Callees of `poke_tool.c` ([`crate::poke_tool`]) that are not ported yet (personal data, message
/// and string-buffer routines, move data) or that are outside the Pokémon record (memcpy).
///
/// Pointers into the Pokémon record are passed as byte offsets from its start (`mon_ofs`, the
/// original passes `mon + mon_ofs`); every other pointer argument is an opaque `u32` handle the
/// caller got from the game (a string buffer, a `u16` output array, ...). The ROM differential test
/// (`tools/check-fn.sh black-pokemon poke_tool`) stubs all of these identically on both sides, so
/// only the arguments are verified, not the callees' own behaviour.
pub trait PokeToolHooks {
    /// Callee `FUN_02017F50(species, form, pid)`: gender from personal data and the PID (UNVERIFIED: P2/P6).
    /// Callers use the low 2 bits.
    fn gender(&mut self, species: u32, form: u32, pid: u32) -> u32;

    /// Callee `FUN_0201850C(species, form, exp)`: level from experience via the growth table (UNVERIFIED: P3).
    fn level_from_exp(&mut self, species: u32, form: u32, exp: u32) -> u32;

    /// Callee `FUN_0201C158(move, pp_ups)`: maximum PP of a move with the given PP-up count (UNVERIFIED: P4).
    fn move_max_pp(&mut self, move_id: u32, pp_ups: u32) -> u32;

    /// Callee `FUN_02019C90(species, form, param)`: personal-data field `param` (UNVERIFIED: P2). Get ids
    /// 0xAE/0xAF call it with param 6/7 (type 1/2).
    fn personal_param(&mut self, species: u32, form: u32, param: u32) -> u32;

    /// Callee `FUN_02045924(strbuf, mon + mon_ofs)`: copy the 0xFFFF-terminated `u16` string at
    /// `mon_ofs` into a string buffer.
    fn strbuf_from_mon(&mut self, strbuf: u32, mon_ofs: usize);

    /// Callee `FUN_02045A38(strbuf, mon + mon_ofs, len)`: copy a string buffer's text into the
    /// record, at most `len` `u16`s.
    fn strbuf_to_mon(&mut self, strbuf: u32, mon_ofs: usize, len: u32);

    /// Callee `FUN_0201F41C(mon + mon_ofs, dst, len)`: copy at most `len` `u16`s from the record to a
    /// `u16` array, terminating it.
    fn u16str_from_mon(&mut self, mon_ofs: usize, dst: u32, len: u32);

    /// Callee `FUN_0201F41C(src, mon + mon_ofs, len)`: copy at most `len` `u16`s from a `u16` array
    /// into the record, terminating it.
    fn u16str_to_mon(&mut self, src: u32, mon_ofs: usize, len: u32);

    /// Callee `FUN_02045CAC(*0x020A7370, 0x28B, strbuf)`: message 0x28B (the bad-egg name) into a
    /// string buffer.
    fn bad_egg_name_strbuf(&mut self, strbuf: u32);

    /// Callee `FUN_02045F5C(*0x020A7370, 0x28B, dst, len)`: message 0x28B into a `u16` array.
    fn bad_egg_name_u16(&mut self, dst: u32, len: u32);

    /// Callee `FUN_02045F5C(*0x020A7370, species, 0x021462F0, 0x20)`: the species name into the global
    /// scratch array at 0x021462F0.
    fn species_name_to_scratch(&mut self, species: u32);

    /// Callee `FUN_0201F448(0x021462F0, mon + mon_ofs)`: nonzero if the scratch array equals the
    /// string at `mon_ofs`.
    fn scratch_equals_mon_string(&mut self, mon_ofs: usize) -> u32;

    /// Callee `FUN_02008F3C(mon + mon_ofs, dst)`: copy 0x38 bytes out of the party extension.
    fn copy_ext_out(&mut self, mon_ofs: usize, dst: u32);

    /// Callee `FUN_02008F3C(src, mon + mon_ofs)`: copy 0x38 bytes into the party extension.
    fn copy_ext_in(&mut self, src: u32, mon_ofs: usize);
}

/// Callees of the growth routines ([`crate::growth`]) that belong to other areas or to the archive loader.
///
/// As with [`PokeToolHooks`], the differential test stubs them identically on both sides, so only their
/// arguments are verified here; `personal` and the ARC loader are verified (or ported) on their own.
pub trait GrowthHooks {
    /// Callee `FUN_02019C54(species, form)`: the personal-data record of the species/form (the original
    /// returns the address of its record cache; here an opaque handle for [`GrowthHooks::personal_field`]).
    fn personal_record(&mut self, species: u32, form: u32) -> u32;

    /// Callee `FUN_0201AE1C(rec, field)`: field `field` of a personal record (field 0x17 = growth curve id).
    fn personal_field(&mut self, rec: u32, field: u32) -> u32;

    /// Callee `FUN_02019C90(species, form, field)`: `personal_record` + `personal_field` in one call.
    fn personal_param(&mut self, species: u32, form: u32, field: u32) -> u32;

    /// Callee `ArcLoadMember(dst, 0x11, curve)` (`FUN_02048F4C`): load ARC 17 member `curve` (101 `u32`,
    /// the EXP needed per level) into `dst`.
    fn load_growth_curve(&mut self, curve: u32, dst: &mut [u32; 101]);
}

/// Callees of the personal-data routines ([`crate::personal`]): the NitroFS member reader on the open
/// personal-data archive handle (global 0x0214652C), and the record loader `FUN_0201ADA4`.
///
/// Records are 0x3C bytes. The differential test stubs these identically on both sides (the read fills
/// `dst` with the stub's bytes), so only the arguments are verified here. Data enters the game through
/// these hooks (D6): `black-pokemon` itself never touches the archive.
pub trait PersonalHooks {
    /// Callee `FUN_02049578(handle, member)`: byte offset of an archive member (the first step of a read).
    fn arc_member_offset(&mut self, handle: u32, member: u32) -> u32;

    /// Callee `FUN_0204960C(handle, offset)`: position the archive reader.
    fn arc_seek(&mut self, handle: u32, offset: u32);

    /// Callee `FUN_020495F4(handle, size, dst)`: read `size` bytes at the current position into `dst`.
    fn arc_read(&mut self, handle: u32, size: u32, dst: &mut [u8]);

    /// Callee `FUN_020490F4(0x10, arg)`: open the personal-data archive (ARC id 0x10); returns the handle.
    fn open_archive(&mut self, id: u32, arg: u32) -> u32;

    /// Callee `FUN_0201ADA4(species, form, dst)`: read the 0x3C-byte record of a species/form into `dst`.
    fn load_record(&mut self, species: u32, form: u32, dst: &mut [u8]);

    /// Callee `FUN_02030734(heap, size, align, file, line)` with `file` = the address of the
    /// `"poke_personal.c"` string (0x020A7310, not passed): allocate a block; 0 on failure.
    fn heap_alloc(&mut self, heap: u32, size: u32, align: u32, line: u32) -> u32;
}

/// Callees of the PokeParam allocation routine ([`crate::create`]).
///
/// A `mon` handle is the address of a heap block the game owns, opaque to this crate; the differential
/// test stubs both callees identically on both sides, so only the arguments are verified here.
pub trait CreateHooks {
    /// Callee `FUN_02030734(heap, size, align, file, line)`: allocate a block on heap `heap`; 0 on failure.
    /// `file` is always the address of the `"poke_tool.c"` string (0x020A72E4) and is not passed.
    fn heap_alloc(&mut self, heap: u32, size: u32, align: u32, line: u32) -> u32;

    /// Callee `FUN_0201756C(mon, species, level, a2, a3)`: initialise a freshly allocated Pokemon: the
    /// 8-argument initialiser `FUN_02017588` called with `(.., -1, 0, -1)` for its last three arguments.
    fn init_pokemon(&mut self, mon: u32, species: u32, level: u32, a2: u32, a3: u32);
}

/// Callees of the PokeParam initialisers and the level-up move scan ([`crate::create`]) beyond the
/// allocator and the 5-argument initialiser of [`CreateHooks`], plus the `poke_tool` accessors of
/// [`PokeToolHooks`] they call.
///
/// Where the original passes the record being worked on (`mon`) to a callee the hook has no argument
/// for it: it is the record of the call. A `mon` that is not that record (a wrapper's plain pointer
/// argument, a freshly allocated block) is an explicit `u32` handle. The differential test stubs every
/// one of these identically on both sides, so only the arguments are verified.
pub trait InitHooks: CreateHooks + PokeToolHooks {
    /// Callee `FUN_02017588(mon, species, level, a3, a4, a5, a6, a7)` on an allocated block `mon`
    /// (what `FUN_0201756C` and `FUN_02017430` call).
    #[allow(clippy::too_many_arguments)]
    fn init_pokemon8(
        &mut self,
        mon: u32,
        species: u32,
        level: u32,
        a3: u32,
        a4: u32,
        a5: u32,
        a6: u32,
        a7: u32,
    );

    /// Callee `FUN_0201761C(mon, species, level, a3, a4, a5, a6, a7)` for a plain pointer `mon`
    /// (`FUN_02017600`).
    #[allow(clippy::too_many_arguments)]
    fn core_init_at(
        &mut self,
        mon: u32,
        species: u32,
        level: u32,
        a3: u32,
        a4: u32,
        a5: u32,
        a6: u32,
        a7: u32,
    );

    /// Callee `FUN_0201761C(mon, species, level, a3, a4, a5, a6, a7)` on the record of the call
    /// (`FUN_02017588`).
    #[allow(clippy::too_many_arguments)]
    fn core_init(&mut self, species: u32, level: u32, a3: u32, a4: u32, a5: u32, a6: u32, a7: u32);

    /// Callee `FUN_020185C4(mon)`: recalculate the stats of the record of the call.
    fn recalc(&mut self);

    /// Callee `FUN_020185C4(mon)` on an allocated block (`FUN_0201747C`'s copy).
    fn recalc_at(&mut self, mon: u32);

    /// Callee `FUN_02018A7C(mon)`: the stat calculation on the record of the call.
    fn recalc_stats(&mut self);

    /// Callee `FUN_02008F08(heap)`: allocate a party-extension block (0x38 bytes); returns its handle.
    fn ext_alloc(&mut self, heap: u32) -> u32;

    /// Callee `FUN_020307B0(block)`: free a block.
    fn heap_free(&mut self, block: u32);

    /// Callee `FUN_02019C54(species, form)`: the personal record of a species/form (an opaque handle).
    fn personal_record(&mut self, species: u32, form: u32) -> u32;

    /// Callee `FUN_0201AE1C(rec, field)`: field `field` of a personal record handle.
    fn personal_field(&mut self, rec: u32, field: u32) -> u32;

    /// Callee `FUN_020056EC(max)`: random number below `max` (0: the full 32-bit range).
    fn rand(&mut self, max: u32) -> u32;

    /// Callee `FUN_0203F0A8()`: next output of the Mersenne Twister.
    fn mt_next(&mut self) -> u32;

    /// Callee `FUN_02018584(species, form, level)`: EXP needed for `level` on the species' growth curve.
    fn exp_for_level(&mut self, species: u32, form: u32, level: u32) -> u32;

    /// Callee `FUN_02017FB0(mon)`: learn the level-up moves a new Pokemon starts with (the record of the call).
    fn learn_start_moves(&mut self);

    /// The byte at 0x0209DF0C (the game language), stored as the language of a new Pokemon.
    fn game_language(&mut self) -> u32;

    /// The byte at 0x0209DF08 (the game version), stored as the origin game of a new Pokemon.
    fn game_version(&mut self) -> u32;

    /// Callee `FUN_0201ADD0(species, form, block)`: load the level-up learnset into a block.
    fn load_learnset(&mut self, species: u32, form: u32, block: u32);

    /// Entry `index` of a learnset block: the (move, level) halfwords at offset `4 * index`.
    fn learnset_entry(&mut self, block: u32, index: u32) -> (u32, u32);
}
