## Proposed changes to section 6 (input for T2)

Sizes are asm fn counts of the previously unidentified code only (the labelled battle/field/UI files are separate). The P/S/F
logic that was feared to be "8-9k fns" is much smaller: in-scope game logic in the unidentified ARM9 code is about 1,300 fns, the rest
is gflib/UI plumbing (rewrite, D3), SDK/DWC (X1/X3) and out-of-scope overlays.

**P (Pokemon data), source ranges from this triage**
- **P1** split in three SEQ-then-PAR sub-items, all in `poke_tool.c` = `unk_02016BA0..unk_02019A50` (0x02016BA0-0x0201A26C, ~118 asm fns):
  P1a crypt + checksum (FUN_02019A50, FUN_02019A7C LCG32, FUN_02019A9C, FUN_02017DBC, FUN_02017DE4; trivial, port first, verify with the Unicorn harness);
  P1b `PokeParaGet`/`Put` (FUN_02017E88, FUN_02017E40 -> FUN_02018E34 / FUN_02019308: field-id switch over the decrypted 0x80-byte block);
  P1c create/init (FUN_020173F0, FUN_02017430 allocate 0xDC = 220 B POKEMON_PARAM, FUN_02018328 allocates 0x68).
  Deps: T2 only (A12 not needed for the byte layout).
- **P2**: `poke_personal.c`: FUN_0201AE1C (44-field getter), FUN_0201AFD4 (form index), FUN_0201ADA4/AD88/ADE4; ARC 016 (60 B record). ~12 fns.
- **P3 (EXP/level)**: not located. Add **T3** below. ARC 017 has no direct loader call; ARC 019 likewise.
- **P4**: `waza_tool.c` = `unk_0201BAE4` (6 asm fns, ARC 021 move data, 36 B) + learnset reader FUN_0201ADD0 (ARC 018).
- **P5**: `shinka_check.c` = `unk_0201AF6C` (FUN_0201B3B8, FUN_0201B406; 7 slots x 6 B per species, ARC 019). ~11 fns.
- **P6**: RNG is a core item, not Pokemon-specific: new **Z15 (PAR, small)** `gf_standard.c` RNG in `unk_0201FC1C`:
  MT19937 (FUN_0203F034 seed, FUN_0203F0A8 next/temper, matrix constant at 0x0203EFC4), LCG64 FUN_0203F258 / FUN_0203F2FC (0x5D588B656C078965 / 0x269EC3, seeded from boot 0x020056C8),
  LCG32 FUN_0203F388 via FUN_0203F328 (0x41C64E6D); other LCG64 users FUN_0202A630/FUN_0202A8BE (tr_tool/tpoke_data). P6 (PID/IV/nature/gender) then consumes Z15.
- **P7** (eggs): egg-move table is ARC 123 (650 records, read in overlay 21); baby forms ARC 020 (FUN_021C4504). Move to F (daycare code is field-side); A-reader for ARC 123 under A.

**S (storage, items, save)**
- **S1**: `pokeparty.c` = `unk_0201A858` head (FUN_0201A920, FUN_0201AB8C; 0xDC-byte party mon, count at +4). ~12 fns.
- **S2**: `box_savedata.c` = gap `unk_020071A8..unk_02007AA4` + FUN_02007AA4 (~23 fns, low confidence on the boundary). UI stays overlay_170.
- **S3**: `myitem_savedata.c` = `unk_02007F7C` (FUN_02008254, ~15 fns), `bag_param.c` FUN_0202E474, item record loader FUN_02020F34 (ARC 024/025) in `unk_0201FC1C`.
- **S5** (save): data model per block (own format, D4/S5): `save_control.c` (unk_02006D3C), per-block files `config/mystatus/playtime/mail/regulation/battle_rec/symbol_save/bsubway_savedata/battle_examination`
  (unk_02007F7C-unk_0200E4CC, ~265 fns); the DS save plumbing `savedata.c`/`backup_system.c`/`flash_access.c` (unk_0201FC1C 0x020316C8-0x02033BA4, ~77 fns) and the CARD backup SDK are replaced, not ported.
  D9 blocks `wifilist/mystery_data/musical_dist_save/gds_profile/net_save` (unk_02009690-unk_02010794) stay out of the model.

**F (field)**
- **F2**: add `zonedata.c` = `unk_02013980` (FUN_020139B0, 0x4C record; 28 fns). **F7**: `vm.c` allocation = FUN_0201121C (`unk_020111F8`), the VM is in overlay 10. **F9** (text): `strbuf.c`/`msgdata.c`
  (`unk_02043900`, FUN_020457B0, FUN_02045B38 (type, arc 2|3, bank), FUN_02045CAC), `wordset.c` (`unk_0201EC64`), `printsys.c`/`gf_font.c`/`systemfont.c`: ~210 fns of which gf_font/systemfont/printsys are render side.
  Game data: `game_event.c`/`game_data.c`/`field_status.c` (unk_02011D40-unk_02014128, ~126 fns, GAMEDATA alloc 0x6B0) -> new **F0 (SEQ)** "game data root + event flags" before F1.
- **F10** grows from 9 to ~40 overlays: add the **23 unnamed gimmick overlays** 36-65 (launched from overlay 21's proc table, 4 KB or less each), the 5 field-child overlays 24-28, ov55, and the 10 tiny gimmick children 73-83 (~280 fns total). Add whiteout/battle-return `overlay_205` (msg 2/37) to the ov092 job.
- **B**: `btl_setup.c` = `unk_02012CE0..unk_02013980` (FUN_02013090 allocs 0xF0 BATTLE_SETUP; ~41 fns) as an adjunct to B1/B11.

**U (screens), new rows identified by message banks**: Trainer Card (`overlay_111`, bank 187; trcard_sys is in overlay 10), Hall of Fame record (`overlay_171`, bank 85) and ceremony (`overlay_180`, bank 32),
move-teaching (`overlay_173`, banks 202-204), Staff credits (`overlay_181`, bank 177), Pokedex family (`overlay_209`, `overlay_211`; ARC 159), party/summary launcher (`overlay_134`, `overlay_90`).
Unidentified UI to keep as unknown until a human looks: `overlay_68`, `overlay_69`, `overlay_109`, `overlay_129`, `overlay_230`, `overlay_231`.

**X (out of scope)**: X1 gains DWC/GameSpy in main (`unk_020583CC..unk_02062634`, 261 fns) and overlays 0, 3, 4, 7, 8, 13, 15, 17, 99, 102-105 (IR), 122 (C-Gear), 128, 178, 219 (Geonet); X2 gains `overlay_184/185` (musical helpers).
X3 gains the SDK/NNS bands `unk_02062634..unk_0209CE10` (~918 fns) listed in the per-file table (bands, confidence low-med; they are only ever ported on demand).

**New items**
- **T3 (PAR, small)**: close the unlocated items: ARC 017/019 consumers (look for handle-based reads `FUN_0204944C`/`FUN_02049578` with arc ids from tables), TM/HM mapping (item record fields), move tutors, and tighten `between` boundaries by decompiling the `unk_*` gaps with the 'ELF + signatures' prototypes from this table.
- **Z15** RNG (above). **G-phase**: boot/system init = `unk_02004FD4..unk_02006118` (gfl_use.c, FUN_0200545C: heap, ArcSys_Init, overlay table) belongs to G1/G2.
- **Sound**: field ISS/BGM selection (`iss_*.c`, `field_sound_system.c`, `unk_0200674C`, big run in `unk_0201FC1C` 0x02027...0x0202B...) is a logic-ish subsystem (~300 fns): new **A9b** "zone -> BGM id selection" fed by zone headers.
