# T3 findings: ARC 017/019/123 consumers, TM/HM table, tutors, egg moves

Method: Thumb `BL`/`BLX` and `ldr r3,=thunk; bx r3` tail-call scan of `main.sbin` (base 0x02004000) and every `OVY_n.sbin`
(`/tmp/blk/t3_scan.py`, `t3_tail.py`; not committed), then rizin (`pd @b:16`, `/v4`) on the `field` ELF and the Ghidra pseudo-C.
No game text is quoted. Move/species numbers are the numeric IDs used by the ROM tables.

Tip for later sessions: rizin shows many main-RAM Thumb functions as ARM garbage; use `pd N @ addr @b:16`.

## ARC loader API (main, ARM thunks over Thumb workers)

ARC id = index into the NitroFS archive table at `*DAT_02048e30` (`a/0/1/7` = id 17 = 0x11, `a/1/2/3` = id 123 = 0x7B).

| addr | role | notes |
|---|---|---|
| 0x02048DDC | open archive `arc` into a 72-byte handle | `FUN_02048DDC(handle, arc)`; reads the NARC through `FUN_02078C18` or the card path |
| 0x02048E34 | member size/offset resolver | parses NARC header (BTAF at +0x0C), bounds check -> panic `FUN_02034734` |
| 0x02048D00, thunk 0x02048F4C | `ArcLoadMember(dst, arc, member, [off=0, size=0 => whole])` | thunk zeroes the 5th arg. Used as `F4C(buf, arc, member)` |
| 0x02048F74 | same as F4C with explicit `(dst, arc, member, off, size)` | partial reads (e.g. baby-form u16) |
| 0x02048D54, thunk 0x02048F60 | `ArcLoadMemberAlloc(arc, member, heap, off, size)` -> heap ptr | caller frees with `Heap_Free` 0x020307B0 |
| 0x020490A8 | `ArcMemberSize(arc, member)` | |
| 0x02049030 | `ArcMemberCount(arc)` | |
| 0x020490F4 | alloc-load variant with `(arc, member)`; thunk at 0x0201B70C passes arc 0x13 | |
| 0x0204944C / 0x02049578 | **handle-based** reads on an already-open handle (+0x48 data base, +0x4C FATB size, +0x4E count) | used for graphics tables (overlay 10/21) and the personal-data cache (see below); not the consumers of ARC 17/19 |

## (a) ARC 017 growth (EXP curves)

- **`FUN_02019B94(curve, dst)` @ 0x02019B94**: `ArcLoadMember(dst, 0x11, curve)`: loads the whole 101 x u32 table (404 B) into `dst`. Used with the shared cache buffer `0x02146398`.
- **`FUN_02019BA4(curve, level)` @ 0x02019BA4**: loads the curve into 0x02146398 and returns `table[level]` (EXP needed for `level`, level 0..100).
- **`FUN_0201850C(species, form, exp)` @ 0x0201850C: level-from-EXP.** Gets the personal record (`FUN_02019C54(species, form)`), reads personal field 0x17 (growth-curve id) through `FUN_0201AE1C(rec, 0x17)`, loads the curve, then
  `lvl = 1; while lvl < 101 && table[lvl] <= exp { lvl++ }; return lvl - 1` (so the result is capped at 100; strict `exp < table[lvl]` breaks). Callers: 0x02018478 (FUN_02018438), 0x02019044 (FUN_02019040), ov21 0x021C3486.
- Personal-record helpers (all main): `FUN_02019C54` @0x02019C54 (cache keyed by `(form,species)` at 0x02146270, record at 0x02146274), `FUN_0201ADA4` @0x0201ADA4 (loads one 0x3C-byte personal record from the global open handle at 0x0214652C, using the handle-based reads above), `FUN_0201AFD4` @0x0201AFD4 (maps `species`+`form` to the alternate-form record index; returns `species` when form 0 or no form table), `FUN_0201AE1C` @0x0201AE1C (personal field getter, 44 fields via jump table; field 0x17 = growth id, 0x2B = tutor bitfield), `FUN_02019C90(species, form, field)` = record + getter.
- EXP gain formulas (P3) are elsewhere in the battle set (not found by this task). The EXP-for-level getter above is what they should call.

Proposed Rust shape (P3): `fn exp_for_level(growth: &[[u32;101];8], curve: u8, level: u8) -> u32` and `fn level_from_exp(...)` exactly as above.

## (a) ARC 019 evolution

- **`FUN_0201B2B0` @ 0x0201B2B0 (file string `shinka_check` @0x020A7330, line 0x6A)**: the evolution check. Prologue `push {r4-r7,lr}; sub sp,0x4c`. Arguments (UNVERIFIED names): r0 = context/bag ptr (saved at sp+4), r1 = mon, r2 = **mode 0..3** (jump table at 0x0201B374; mode > 3 returns 0), r3 = mode parameter (trade partner / used item), stack `+0x64` = out ptr for the evolution kind (defaults to a local), stack `+0x68` = heap id. Returns target species (0 = none).
  - Early outs: mon field 0x4C (egg flag) nonzero -> 0; species 0x40 (64) with a form flag != 0x40 and mode != 3 -> 0 (an item-gated species check).
  - Allocates 0x2A bytes (7 x 6) on the heap (`FUN_02030734`, size 0x2A), loads the species' evolution member with **`FUN_0201B764` @ 0x0201B764 = `ArcLoadMember(dst, 0x13, species)`**, loops `i = 0..6` over 6-byte records and `switch (u16 method)` with **29 cases (0..28)** (jump table at 0x0201B3CA); the matching case sets `target = rec.u16@4`; the loop stops at the first match; heap freed at 0x0201B700.
  - Record layout (confirmed by the code: `ldrh [r5]` method, `[r5+2]` param, `[r5+4]` target): `u16 method, u16 param, u16 target`, 7 records per member = 42 B, matches `black-data::evolution`.
  - Method semantics seen in the switch (partial, from the first cases): 0xDC = 220 is the friendship threshold (cases 1-2 = friendship, day / night via `FUN_02011410`); param level compared with mon level (`[sp+0x14]`, from field 0x9E); stat comparison via fields 0xA2/0xA3 (Attack vs Defense: > / = / < variants); personality-based split (`sp+8` = `(personality >> 16) % 10` via `FUN_0209C0A4`, threshold 5); item held (field 0x6E, held-item id compared with param); known-move check (`FUN_0201B774`); party-species check (`FUN_0201AC44`). A full method table is a P4 job; the code range is 0x0201B3B8..0x0201B700 (switch body 0x0201B406 onward).
  - Sub-blocks: `FUN_0201B718` @ 0x0201B718 reads a record field out of an evolution member with `(handle?, member, idx, kind)` and returns `u16` method/param/target by `kind` 0/1/2 (loads 0x2A bytes through `FUN_0204932C`; arc/handle origin UNVERIFIED). `FUN_0201B70C` @ 0x0201B70C = alloc-load `(0x13, species)`.
- Baby-form (ARC 020): `FUN_021C4504` (ov21) = `if species < 650: species = ArcRead(dst=u16, arc 0x14, member=species, off 0, size 2)`. T1/P7 already knew this.

## (b) TM/HM -> move table

- **Table: main `0x0209EAA0`, 101 x u16** (move ids), file offset `0x0209EAA0 - 0x02004000 = 0x9AAA0` in `main.sbin`. Order is **storage order**: `[0..91]` = TM01..TM92, `[92..97]` = HM01..HM06, `[98..100]` = TM93..TM95. First entries (numeric): 468, 337, 473, 347, 46, 92, 258, 339, 474, 237 ...; HM entries 15, 19, 57, 70, 127, 291; last three 528, 249, 555. Just before it sit `901..904` (unrelated), after it `0, 1001, 1002`.
- Item id mapping (functions around 0x02021288): `FUN_02021288` is-TM/HM item predicate: item in `[328, 425]` or `[618, 620]`. `FUN_020212B0(item)` -> move: `idx = item - 328` for items < 618, `idx = item - 520` for items >= 618, `move = table[idx]` (0 when not a TM/HM). `FUN_0202130C(item)` returns the UI TM number (0-based): `item - 328` (TM01-92), `item - 526` (TM93-95 -> 92..94), `item - 325` (HM -> 95..100); 0xFF otherwise. `FUN_02021350` returns the HM index 0..5 for items 420..425 (0xFF otherwise). `sub_020212E4(move)` tests whether `move` equals one of the six HM moves (reads `table + 0xB8`, 6 entries).
- Learnability: personal record `+0x28 .. +0x34` (13 bytes, little-endian bit i = table index i above, i.e. same storage order, 101 used bits). Checked on three records against the table (consistent). Tutor flags are separate (below).
- Decode in Rust: trivial (`u16` array at the fixed main offset); not added to `black-data` (needs `main.sbin`, not an NARC member). Suggest a `TmTable::from_main(main_sbin: &[u8])` in A11 when the loader for main exists.

## (c) Move tutors

- Flags: personal record **`+0x38`, u32 bitfield**, read by personal field 0x2B (`FUN_02018948` @ 0x02018948, species 650 excluded; `FUN_0201891C(mon, bit)` @ 0x0201891C = test bit `bit` for the mon's (species, form)). Used bits 0..6 only. Population: bits 0,1,2 = 15 species each (the three starter lines of each generation, grass/fire/water), bits 3,4,5 = 5 species each (final evolutions of the same families), bit 6 = 30 species (fully evolved dragon-type families).
- Tutor move table is **code, not data**: `FUN_overlay_d_21__021B5F24(mon, tutor)` @ ov21 0x021B5F24:
  - tutor 0 -> move 434 (always; no bit test)
  - tutor 1 -> bit 3 -> 338, else bit 4 -> 307, else bit 5 -> 308 (0x134), else 307
  - tutor 2 -> bit 0 -> 520 (0x208), else bit 1 -> 519 (0x207), else bit 2 -> 518 (0x206), else 307
  So bits 3-5 select the three "ultimate" starter-type moves, bits 0-2 the three pledge moves, bit 6 the dragon move (tutor 0). Callers at ov21 0x021B5FCC.. 0x021B6034 (the same bit tests, other tutor-screen paths) and 0x021B6156 / 0x021B61B0 / 0x021B61D4.
- No other tutor tables found (no u16 array containing these moves exists in main or any overlay besides the code literals). Costs/shards, if any, are in field scripts (ARC 57), which is F7's scope.

## (c) Egg moves (ARC 123)

- **`FUN_overlay_d_21__021C40F8(egg_ctx, egg_work, heap)` @ ov21 0x021C40F8**: `buf = ArcLoadMemberAlloc(0x7B, member = base_species(*egg_work), heap)`; the buffer is then handed to the move-picker (`sub_02017e1c`). Part of the egg generation pipeline `FUN_overlay_d_21__021C38F4` @ 0x021C38F4 (calls 0x021C3A00 .. 0x021C4340 in sequence: 0x021C40F8 = egg moves, 0x021C4160 next step, 0x021C4238, 0x021C42BC, 0x021C4340 = final write to the egg; baby-form lookup 0x021C4504 is called by the same pipeline).
- **Member layout (verified on the ROM):** `u16 count; u16 moves[count]`. 650 members, member index = species id; 389 members are exactly 2 bytes (count 0), the rest are 18..30 bytes (8..14 moves). Example member 1: count 14. Members for species with no egg moves have count 0.
- Format for A7: `Vec<u16>` per species. Decoder not added (trivial).

## (d) T1 boundary tightening (no TSV edit)

T1 "between anchors"/label notes are consistent with what was found, with these refinements (could be applied to `T1-overrides.tsv` by the coordinator):
- `unk_0201AF6C.s` (0x0201AF6C-0x0201BAE4): contains `poke_personal` (0x0201AF80 .. 0x0201B1xx: `FUN_0201AFD4` form-record lookup, party/box bit helpers at 0x0201B100-0x0201B280) **and** `shinka_check` from **0x0201B2B0 to ~0x0201B774** (the evolution switch, 0x0201B3B8-0x0201B700, plus ARC-19 thunks 0x0201B70C/0x0201B718/0x0201B764). 0x0201B774-0x0201BAE4 are further evolution helpers (`FUN_0201B774` known-move check).
- `unk_020185B4.s` (0x020185B4-0x02018D10): holds the level-from-EXP function 0x0201850C-0x02018540 (boundary 0x020185B4 is just after it, in `unk_02017FB0.s` 0x02017FB0-0x020185B4) and the tutor-bit test 0x0201891C/0x02018948. `unk_02019A50.s` holds the EXP-table loaders 0x02019B94/0x02019BA4 and the personal cache 0x02019C54-0x02019C90.
- The TM/HM helpers (0x02021288-0x02021370) and the TM table (0x0209EAA0) sit in `unk_0201FC1C.s` (0x0201FC1C-0x02040618, T1 label "networking / ... + sound/ISS": too coarse). The TM helpers are `item`-module code (tag it with the items subsystem, A9/A7 scope).
- `unk_0201BAE4.s` waza_tool: ARC 21 (move data) member loaders (`F4C(dst, 0x15, move)`) at 0x0201BD58-0x0201C2F8, ~20 accessors; that is the `waza_tool.c` getter set A7 already decoded.

## Not found / open

- The EXP-gain formula and where `FUN_0201850C` / `FUN_02019BA4` are called in battle (P3 should grep callers of these two in the battle set).
- Origin of the global personal-record handle at 0x0214652C (opened once at boot; the four readers are 0x0201ADA4/0x0201AFD4); ARC 16 id inferred, not seen as an immediate.
- Evolution method table (29 cases): semantics partly read; P4 should port from 0x0201B3B8.
- Item data field holding "TM" flags: none found; TM->move mapping is table-only (above), not an item field.
