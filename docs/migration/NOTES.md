# Reference notes (accumulated research — append, don't delete)

Moved out of `MIGRATION-AGENT.md` (its old section 7) on 2026-10-13. Workers read only the named sub-heading they are pointed at; nobody reads this whole file.

### Code size (from the original status table; labelled functions only)

| subsystem                                            | files | fns                                        |
| ---------------------------------------------------- | ----- | ------------------------------------------ |
| battle core + AI (P1)                                | 11    | ~1,888                                     |
| field core / scripts / events (P6)                   | 11    | ~5,139                                     |
| PC box UI (P3) / bag UI (P4)                         | 3     | ~893                                       |
| battle UI, party/summary, evolution/egg, UI/app (P7) | 40    | ~4,413                                     |
| networking / C-Gear (out of scope)                   | 53    | ~10,252                                    |
| unidentified ARM9 + overlays                         | 281   | ~6,108                                     |
| **total**                                            | ~409  | **~28,700** + `.byte`-encoded handler code |

### Battle handler registries (battle_calc rodata)

Resolved by B4 (<HASH_UNKNOWN>). The only registries in overlay 93 are abilities (factor type 4), items (type 5) and moves (type 0); they are the sole callers of `EventSystem::add` (`021d6f0c`) there.

| table                 | entries               | contents                                                                                                                                                                                                                                                                               |
| --------------------- | --------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 0x021F0E74            | 0x9E                  | **ability registry**: ability ID → handler-table getter (156 standard, 2 custom: abilities 100, 143); 240 distinct handlers 0x021D85F4–0x021DCE9C (+ `0x021D88B8`, `0x021DC4D4` via the custom getters). **All 242 ported (B3).** `021d8484` registers them as event factors (type 4). |
| 0x021F1EA4            | 171 + 0,0 end         | **item registry**: `(u32 item ID, getter)`, keys start at 149; registered by `021dcec4(mon, item)` as factor type 5 (class 6).                                                                                                                                                         |
| 0x021F3030            | 258                   | **move registry**: `(u16 move ID @+0, getter @+4)`, stride 8; registered by `021e02dc(wk, move, speed)` as factor type 0 (class 0), deduped against the Pokémon's type-0 factors by `021e0354`. 4 getters (moves 87, 100, 217, 542) don't reference a pool table by literal.           |
| overlay 95 0x0689D650 | 8                     | **field-effect registry**: keys 0–7 = `btl_field.c` slots (`black-battle::field`); `FUN_overlay_95__06898020(key,0)` adds factor type 3; `FUN_06898080` removes. Slot names unverified (2 Gravity, 3 Imprison, 7 Magic Room).                                                          |
| overlay 95 0x0689D6C8 | 5                     | keys 0–4; `FUN_overlay_95__068981f0` adds type 1 after a duplicate check (`068982c0`). Side vs position effects: UNVERIFIED. A type-2 adder at 0x06898328 (registry not decoded).                                                                                                      |
| 0x021F05A0            | 187 pairs, 132 tables | **pool, not a registry**: ability pool A (0x021F05A0–0x021F0B78); first table = ability 100 via getter `021d88d0`.                                                                                                                                                                     |
| 0x021F0B94            | 92 pairs, 16 tables   | pool: ability pool B (0x021F0B94–0x021F0E74); first table = ability 54 (getter `021dadf8`).                                                                                                                                                                                            |
| 0x021F136C            | 359 pairs, 168 tables | pool: **held-item** handlers (0x021F136C–0x021F1EA4); first table = item 300 (getter `021df2bc`); code 0x021DD05C–0x021E02C0.                                                                                                                                                          |
| 0x021F240C            | 136 pairs, 136 tables | pool: **move** pool A (to 0x021F284C); first table shared by moves 234/235/236 (`021e6158`); 161 move IDs.                                                                                                                                                                             |
| 0x021F2888            | 132 pairs, 57 tables  | pool: move pool B (to 0x021F2CA8); move 156 (`021e43fc`); 69 move IDs. 0x021F2CA8–0x021F2D58 are small non-handler tables.                                                                                                                                                             |
| 0x021F2D58            | 91 pairs, 19 tables   | pool: move pool C (to 0x021F3030); moves 182/197 (`021e3b08`); 24 move IDs.                                                                                                                                                                                                            |

Pool format: flat array of `(u32 event ID, thumb fn)` pairs, no terminator. Getter protocol: `table = getter(&out)`, `out & 0xFFFF` = pair count; abilities also read `out >> 16` as the priority class (0 = 5). Sub ID is the ability/item/move ID (`u16` at factor +0x38). Event IDs are one shared namespace across categories. Why abilities and moves are split over several pools is unknown.

Handlers take `(factor, flow_work, poke_id, work)` and call: event vars (`021d775c` get, `021d76bc` rewrite, `021d7704` mul), `021c7cf4` (battle Pokémon by ID), HANDEX API (`021c88a4` push, `021c88c8` pop, `021c88b4` push+pop, `021c8838` string param, `021c8860`), BattleMon accessors.

### Ported-file status (carried over from the deleted PORTING-STATUS.md; non-todo rows only)

| source file                                 | subsystem       | status                                               | Rust                                                              |
| ------------------------------------------- | --------------- | ---------------------------------------------------- | ----------------------------------------------------------------- |
| `src/battle_record.c`, `src/unk_02008574.c` | battle core     | verified (differential vs host-compiled C)           | `black-core::{battle_record,unk_02008574}`                        |
| `asm/battle_record.s`                       | battle core     | partial (BattleRecordBuilder_Load)                   | `black-core::battle_record`                                       |
| `asm/battle_calc.s`                         | battle core     | partial, 93/169 labelled fns                         | `black-battle::{calc,event_var,calc_target,calc_action,handlers}` |
| `asm/battle_pokeparam.s`                    | battle core     | verified, 176/176                                    | `black-battle::{battle_mon,event_factor}`                         |
| `asm/battle_field.s`                        | battle core     | verified, 34/34                                      | `black-battle::field`                                             |
| `asm/battle_view_core.s`                    | battle UI       | partial, 22/78 (logic helpers 0x021E8940–0x021E8DA8) | `black-battle::{sick_cont,poke_type}`                             |
| `asm/unk_0209C2B0.s`                        | runtime library | partial                                              | `black-battle::calc::u32_div`                                     |

Everything else in the original 400-row file table was `todo`; T1/T2 regenerate the subsystem map. Per-function ticks: `black-rust/checklists/*.md` (battle_calc, battle_pokeparam, battle_field, battle_view_core); generate more with `python3 tools/mkchecklist.py asm/<file>.s`.

### Overlays (Z12)

- Overlay manager in main: `0x02034AC4` `overlay_load(id)` (checks overlap via `02034BB0`, panics through `FUN_02034714`; records `{id,1}` in an 8-byte slot table; calls `FS_LoadOverlay` = `FUN_02079264`); `0x02034A5C` `overlay_unload(id)` (→ `02034A3C` → `FS_UnloadOverlay` = `FUN_020792B8`); `FUN_02034D60(id)` region (1/2), `FUN_02034D08` slot count. 147 static load sites resolved, 24 table-driven ones not.
- Main loads 10 + 11 at boot (`0x02005240`) and keeps them resident (Wi-Fi mode loads 9/13/14/15/114 instead). 10 loads 20; 21 loads 18 (`021881A6`, unloaded `02188B48`). 10 and 11 also load 93, so they are resident during battle. Battle 93 loads 94, 95, 96, 196, then 197 (bag) or 198 (party).
- Apps in 0x021B6100–0x021EE740 overlap 21, so the field is unloaded while they run; most apps load 67 (ui-common). Overlay 10 also holds `trcard_sys` and `waza_oshie`; 21 holds field-battle and shop script commands.
- 57 overlays are 32-byte stubs (no code); 69 are network/D9.

### Asm reading notes

- `thumb_func_end` often sits at the first branch; a function runs to the next `func_start`.
- Thumb `add rX, pc` (hi-reg form) reads PC as instr+4, **unaligned**; `add pc, r0` jump tables branch to instr+4+offset.
- Much code is `.byte` blocks (e.g. 0x021D778C, handler code 0x021D85E8–~0x021E6xxx in battle_calc.s/battle_view_core.s); the decomp's file split there doesn't follow real source files.
- Runtime helpers: `FUN_0209C2B0` = `_u32_div_f` (returns dividend on /0, remainder in r1), `FUN_0209C0A4` = `_s32_div_f`, `FUN_0209C054` = `_ll_mul`, `FUN_0209B750` int→double, `0x0209B790` uint→double, `FUN_02082D44` = `MI_CpuCopy8(src,dst,n)`, `0x02082BCC` = `MI_CpuFill8`, `Heap_AllocDebug` = `0x02030734`, `Heap_Free` = `0x020307B0`.
- Debug filename strings in each file's `.data` identify the original source file (used for subsystem mapping).

### Verification methods

- Decompiled C: compile on host with shims; compare fuzz hashes.
- Asm only: `tools/rom-diff` runs original ARM9 + overlay code under Unicorn with Python stubs for unported callees; stateful code compared as whole memory images (e.g. 0x1F8-byte `BTL_POKEPARAM`) + log of stubbed calls; re-entrant callbacks via assembled Thumb stubs (`Emu.load_thumb`).
- UI rewrites (U-phase): side-by-side screenshot/timing comparison against the original running in an emulator (manual; human sign-off).

### Asset sources (ROM filesystem `black-rom/files/`)

- `a/0/0/*`, `a/0/1/*`, `a/0/2/*` — numbered NARCs (meaning to be mapped by A0).
- `wb_sound_data.sdat` (all music/SE/cries), `gfl_font.dat`, `skb.narc`, `soundstatus.narc`, `titledemo.narc`, `dl_rom/`, `dwc/` (network, out of scope).

### T1 / A0 / B4 / B9 / F1 results

(append tables here as those items finish)

**B9 (<HASH_UNKNOWN>): `battle_ungrouped.s` = btl_server_flow.c and neighbours, 0x021BAA94–0x021CD800 (1,013 Ghidra fns, 944 labelled).** Method: call graph from `index.jsonl` (`/tmp/blk/b9/scan.py`), filename-string refs (`btl_server_flow.c` used at 0x021BB9C4 by the work allocator; `btl_client.c` at 0x021CD8CC, so the client starts right after), HANDEX switch in `021c8918`. The internal call graph is one 849-fn component once 7 hubs are removed, so the split follows address order.

| area             | range             | fns | what                                                                                                                                                                                                                 | evidence                            |
| ---------------- | ----------------- | --- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------- |
| `server_side`    | 021BAA94–021BB968 | 69  | entry points called from battle_server.s (`021b6aac`…)                                                                                                                                                               | 82 externally called fns, most here |
| `core1`..`core6` | 021BB968–021C7CBC | 577 | flow core; `021bb968` = create (`Heap_AllocDebug(heap, 0x212C, 1, "btl_server_flow.c", 0x245)`, stores +0 server, +4, +8 pokecon, +0xC, +0x18, heap ID u16 @+0x780, +0x4A4 = `FUN_0201B70C()`)                       |                                     |
| `api`            | 021C7CBC–021C8810 | 81  | flow API for event handlers (`021c7cf4` battle mon by ID: 284 external callers; `021c7d00`, `021c7f68`, `021c8288`…)                                                                                                 | callers in handler code 0x021D85F4+ |
| `hem`            | 021C8810–021C8918 | 10  | HANDEX manager: `021c88a4` push, `021c88c8` pop, `021c88b4`, `021c8838`, `021c8860`                                                                                                                                  |                                     |
| `handex`         | 021C8918–021CA8CC | 90  | `021c8918` executes one HANDEX: `switch(*work & 0xFF)` over types 0x00–0x3B (0x0D absent), procs 0x021C8C6C–0x021CA7D4                                                                                               | full case list in pseudo-C          |
| `events`         | 021CA8CC–021CC8D8 | 98  | flow-owned helpers + an event pool at 0x021EFDE4 (23 `(event, fn)` pairs: events 0x12–0x29, 0x10263–0x10266 → fns 0x021CBF31–0x021CC5F5), referenced at 0x021CBA0C                                                   |                                     |
| `cmd_queue`      | 021CC8D8–021CD800 | 28  | server command queue: `021cd4b0` put (arg count = low nibble of table at `DAT_021cd4f4`[cmd]), `021cd64c` put message (varargs to terminator), `021cc8d8`/`021cc95c` reserve/write-at, `021ccae0`/`021ccb98` writers |                                     |

**A3 (<HASH_UNKNOWN>): text banks** (ARC 002, by bank number): species names 70 (652; also 253, with 254/255 FR/DE → 253–258 per-language), move names 203 (560), item names 54 (627), ability names 182 (165), type names 199 (17), trainer classes 191 (105). Control codes: `F000 id n params…` commands (0100 trainer/player name, 0101 species, 0102 nickname, 0103 type, 0105 place, 0107 move, 0109 item, 010B box, 010D stat, 010E trainer class, 0110 title, 02xx numbers; BE00 wait button, BE01 page clear, BE02(n) wait n frames, FF00(n) colour, BDxx layout), FFFE newline, FFFF end, F100 = 9-bit compressed entry.

Hubs (most in-range callers): `021cd4b0` (80), `021cc8d8`/`021cc95c` (64 each), `021cd64c` (47), `021c99a0` (34, message HANDEX helper), `021c195c` (14), `021c88e4` (13).

**A0 (2026-10-11): ROM archives.** 235 NARCs `files/a/X/Y/Z`; ARC ID = decimal `XYZ` (path table of 235 pointers at main 0x020A6C58, identity order; passed to an init in `FUN_0200545c`). Full per-file inventory: `python3 -I tools/narc-inventory.py` (member counts, sizes, magics; its LZ10 counts include raw records whose first byte is 0x10). Identified:

| ARC                      | members   | content                                                                   | evidence                                                     |
| ------------------------ | --------- | ------------------------------------------------------------------------- | ------------------------------------------------------------ |
| 002                      | 288       | system message banks                                                      | count/size; A3 confirms                                      |
| 003                      | 472       | story message banks                                                       | count/size; A3 confirms                                      |
| 004                      | 14,285    | Pokémon battle sprites (NCGR/NCLR/NCER/NANR, LZ11)                        | counts per species                                           |
| 052                      | 14,285    | second Pokémon sprite set (same layout as 004)                            | UNVERIFIED role                                              |
| 007                      | 1,431     | Pokémon icons (NCGR)                                                      | count                                                        |
| 008                      | 649       | map containers (per-map model + permissions + building placement)         | sizes; A6 confirms                                           |
| 009                      | 255       | map matrices                                                              | count                                                        |
| 012                      | 1         | zone headers (20,496 B = 427 × 48)                                        | size                                                         |
| 014                      | 282       | map textures (BTX0)                                                       | magic                                                        |
| 016                      | 669       | **personal data** (60 B; stats HP/Atk/Def/Spe/SpA/SpD at +0, types +6/+7) | #1 = 45/49/49/45/65/65, types 11/3; #495 = 45/45/55/63/45/55 |
| 017                      | 8         | **EXP growth tables** (101 × u32)                                         | table 0 level 100 = 1,000,000                                |
| 018                      | 668       | **level-up learnsets** (`u16 move, u16 level` pairs)                      | #1: Tackle@1, Growl@3, Leech Seed@7, Vine Whip@9             |
| 019                      | 668       | **evolutions** (7 × `u16 method, u16 param, u16 target`)                  | #495: method 4 (level) 17 → 496                              |
| 020                      | 650       | baby forms (u16)                                                          | size                                                         |
| 021                      | 560       | **move data** (36 B)                                                      | #33 Tackle: type 0, power 50, acc 100, PP 35 at +3/+4/+5     |
| 024                      | 627       | item data (36 B)                                                          | count/size                                                   |
| 057                      | 899       | field scripts                                                             | count; F7 confirms                                           |
| 092 / 093                | 616 / 616 | trainer data (20 B) / trainer parties                                     | paired counts                                                |
| 125                      | 428       | zone events (NPCs, warps, triggers, signs)                                | ≈ zone count + 1                                             |
| 126                      | 112       | wild encounters (232 B)                                                   | size                                                         |
| 111–114, 140             | —         | small SSEQ/SBNK/SWAR / SDAT sets (non-main audio)                         | magic                                                        |
| 11, 75, 103, 115, 160, … | —         | 3D models/anims (BMD0/BCA0/BTA0/BTP0/BVA0)                                | magic                                                        |

Unknown roles (structure only): 006 (733, ` APS`), 108 (282 × 28 B, pairs with 014), 134 (670, LZ11 ~30 KB), 178 (649 × 249 B), 194 (652 × 4 B), 123 (650 × 2 B), 169/214 (16-B records), 229/230 (52/64 large bin), 212 (2 RIFF).

**F1 (<HASH_UNKNOWN>): field core split** (full table `black-rust/tools/re/F1-SPLIT.md`, reproducible with `tools/re/f1split.py --md`). Field set = 7,210 fns: ov10 1,835 (script/event side), 11 241 + 18 281 (intrude/comm, X1), 20 182 (event*mapchange, WFBC (X1), el_scoreboard), 21 4,671 (field core; code ends 0x021D3594). Main areas: script_sys/work ov10 02158804 (F7); scrcmd ov10 02159204/0215A514/0215C63C + ov21 021B28E0 + unanchored clusters 021AE948/021B0D14/021CF7DC/021D1670 (F8); map_matrix ov10 02159B48 + fieldmap ov21 02187F00 (F2a); buildmodel 0218B754, scenearea 021AB5F4 (F2b); field_player\* 021A2E50–021A7356, rail 021B7884/0219EF44 (F3); fldmmdl ov10 0216CC58 (F4a), fldmmdl_acmd 02197D3C (F4b), trainer_eye 021AE454 (F4c); field_encount 021A9ED8, effect_encount 021AA77C, move_pokemon ov10 0215E2B8, symbol_map 021664CC, event_battle 0216E95C (F5); eventdata_system ov10 02161D98 (F6a), event_mapchange ov20 02183B60 (F6b); sodateya 021C31D4 (F13); field_task 021C6C70, proclink ov10 02160A94 (F14); calender/weather/field_light (F11). Render → G8 (~420 fns: g3d_mapper, camera, fog, 3dbg, ground_anime) and U2 (~920: fldeff*_, field_effect, blact, particles, place name, subscreen). Hubs are 1-line getters/veneers: 0215A5F4/5DC/5E4 (script ctx), 02159B10/AE8, 021A6D9C (Heap_Free veneer), 02188CB4 (`_(arg+0x8C)`), 0216D67C/5F8 (fldmmdl).

---

### Effort estimate (baseline 2026-10-11; moved from the old section 2 on 2026-10-13)

Since 2026-10-13 the completion percentage is **computed** by `tools/progress.py` (`PROGRESS.json`), using the weights below (agent-hour midpoints: logic 675, assets 90, Bevy shell 60, screens 225, integration 150 → logic 0.56, A 0.075, G 0.05, U 0.19, H+R 0.125). Re-baseline the weights there, not in prose.

**Measured baseline:** one session verified 21 `battle_calc` functions (Rust port + rom-diff) in ≤1 h (<HASH_UNKNOWN>). That's simple leaf code; assume **~20 fns/agent-hour for leaf logic and ~5–10 for stateful flow code** (server flow, script VM). Session 4 (Opus workers, 2026-10-13): spec jobs of 100–250 fns take 1.5–3 h and 300–700k tokens; port jobs of 100–250 fns 1–2.5 h and 300–500k tokens.

| work                                                         | size                       | agent-hours    | notes                                                                                                                                                                               |
| ------------------------------------------------------------ | -------------------------- | -------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Logic port (B, P, S, F; in-scope share of unidentified code) | ~8–9k fns                  | 450–900        | biggest bucket; mostly PAR. T1: in-scope logic in the unidentified ARM9 code is only ~1.3k fns (P ≈ 143, S ≈ 369, F ≈ 670); the rest is gflib/UI plumbing (rewrite) or out of scope |
| Asset readers + extractor + music render (A)                 | ~12 items                  | 60–120         | PAR; A9 by ear needs human                                                                                                                                                          |
| Bevy shell (G)                                               | 9 items                    | 40–80          | SEQ critical path                                                                                                                                                                   |
| Screens (U)                                                  | ~17 screens + battle anims | 150–300        | PAR; each needs human visual sign-off (now: `tools/ui-diff` scores + sign-off)                                                                                                     |
| Integration + full playthrough (R)                           | —                          | 100–200        | SEQ; human play-test paced                                                                                                                                                          |
| **Total**                                                    |                            | **~800–1,600** | plus ~30% rework/review overhead                                                                                                                                                    |

**Calendar (wall-clock, 2026-10-11 guess):** 3 parallel agents, ~8–10 h/day: ~5–9 months; 6–8 agents: ~3–5 months (the logic phase scales; the SEQ critical path doesn't). Floor ~6–8 weeks of SEQ critical path however many agents run: Z → B2 → B9 → B11 → B16, F1 → F7 → F12, G1 → G2, R1 → R2. Human time (reviews/merges, visual sign-off per screen, the full playthrough R2) is the real bottleneck late on.

### Early done list (2026-10-11 snapshot, superseded by the checklist and `PROGRESS.json`)

- `black-core`: `src/battle_record.c`, `src/unk_02008574.c` (23 fns, differential vs host-compiled C).
- `black-battle` (per-function ticks in `black-rust/checklists/`): `battle_calc.s` 93/169 (89 verified, 4 unit-tested only); `battle_pokeparam.s` 176/176; `battle_field.s` 34/34; `battle_view_core.s` 22/78 (logic helpers only, rest is UI); all 242 ability handlers + registration helpers (`handlers::abilities`, `handlers::registry`).
- `nds-rom` + `black-cli`: ROM parse/extract/build/compare, BLZ, overlays, FNT, banner.
- `black-battle-wasm` + `web/`: browser playground running ported calc functions (proves wasm path).

---

### Lessons (coordinator) — the stories behind the rules in `MIGRATION-AGENT.md` §5

- **Plan-file size (2026-10-13).** The single plan file reached 262 KB. Two edits on 2026-10-13 silently lost data because the file was too large to re-read before editing: "re-add 9 session notes that were silently dropped (wrong anchor)" (07:34Z) and "Running line rewritten (was stale since wave 4: unchecked replace)" (13:05Z). Hence the split into `docs/migration/`, the size caps in `tools/check-docsize.py`, and the rule to append to `OPEN.md`/`LOG.md` only at the end.
- **Worktrees from `origin/main` (found 2026-10-12).** Several workers started from local `main` without unpushed merges and rebased onto stale bases. Rule 4: create worktrees from `origin/main`; push verified `main` before spawning dependent workers.
- **Auto-removed worktrees (found 2026-10-13).** An Agent-tool worktree with no changes is auto-removed when the worker stops; a worker resumed with `SendMessage` then silently works in the main checkout. B11-spec and B13-spec2 did this; their edits were moved to `/tmp/blk/wt-B11` and `/tmp/blk/wt-B13s2` before anything reached a commit. Rule 4: create the worktree yourself before resuming, and check `git status` on `main` after every worker report.
- **`check-force.py` after every port merge (found 2026-10-13).** B11a's `force` edits predated B13-spec2's new client specs, leaving 118 unforced stub lines; the tool now catches it (rule 6). Related: the PORTED-by-address collision (ai1 02188990 vs fieldmap ov21) led to Z20 (PORTED keyed per memory set); a port that calls an in-area callee directly bypasses another area's `force` stub of that callee (B13a merge); two gate-green branches together can leave dead code (B11a + B11b, clippy on `main`) — the post-merge gate is required.
- **Disk (2026-10-12 macOS, 2026-10-13 Linux).** A Bevy-including target dir is ~10–14 GB; the macOS data volume filled up with 8 per-worker targets (329 MB free, A5's gate failed with ENOSPC), and on the Linux host one full-gate target alone filled the 16 GB `/tmp` tmpfs (`/tmp/blk` is now a symlink to `~/blk`). A **shared** target dir was tried and is unsafe: cargo artifacts of the same crates from different worktrees collide, so a check can run another worktree's code (main's check-fn run picked up F9-worker functions). Rule 10: per-worker targets, `GATE_NO_GAME=1` for workers, full gate only in `/tmp/blk/target-coord`.
- **Sandbox (2026-10-13).** Workers cannot `export`, `.`, heredoc, or loop over shell variables; env vars never reached the gate and 4 Sonnet workers reported "BLOCKED: sandbox" or returned partials. Fix: `env VAR=… command` prefixes in `.claude/settings.json` and the worker template; prompts tell workers to report refusals instead of routing commands through wrapper scripts.
- **Sonnet quitting early (session 4).** Spec/harness jobs of 100+ functions on Sonnet came back after a few minutes with partial work ("too big", strict-callee warnings unresolved, empty-suite hashes). Opus finishes them on the first report at roughly 3–10× the tokens. Routing for such jobs: Opus (table in `WORKER-PROMPT.md`).
- **Why model tiers need rules (Z14 pilot, 2026-10-11).** The Unicorn harness runs the original code and the Rust port on the same random inputs and compares memory + the stubbed-call log. It is not a byte-for-byte proof: a wrong port still passes when inputs miss a branch (rare event ID, edge value), when stubs hide behaviour (callees stubbed identically on both sides), or when the porting agent weakens its own test (narrower inputs, extra stubbing, re-recorded hash) — weaker models under "make it match" pressure do this more. So retry-until-match is only safe when the harness isn't written or edited by the agent doing the port; that is why Z9 (tool-generated stubs, per-function hashes) came before cheap-model bulk work. Pilot data (Z14, 20 item handlers, same packets): Haiku 133k tokens / ~6 min / 2 check runs, Sonnet 104k tokens / ~4.4 min / 1 run, identical module hash; Haiku had 2 wrong helper descriptions. Decision: Haiku for leaf handler batches with an existing harness, with a 2–3 helper spot audit per batch; Sonnet for new helpers with non-obvious semantics, harness jobs, stateful flow code. Re-check per verified function (not per call) every few waves and record it in `LOG.md`.
- **UI drift (2026-10-13).** Phase U had no automatic check (only "side-by-side screenshot comparison, human sign-off"), and three `/tmp/blk/g*/\*.png` requests sat unanswered in the human queue. `tools/ui-diff/` (U0) scores port frames against human-captured emulator goldens locally; agents read numbers and open a picture only when a number is red (at most 3 per item).
