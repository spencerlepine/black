# black-storage

Party, PC boxes, bag/items and the save data model.

Nothing ported yet. Undecompiled callees go in `hooks` as traits (§4 porting pattern).

| C | Rust |
|---|------|

Deliberate deviations: none yet.

## S4: item data and classification (`item`, `item.c` of the main ARM9)

Verified per function with `tools/check-fn.sh black-storage item --per-function` (spec `tools/rom-diff/specs/black-storage/item.spec`).
Rodata tables (TM/HM moves 0x0209EAA0, mails 0x0209E8EC, pairs 0x0209E904, berries 0x0209E968, graphics index 0x0209EB6C)
are slice parameters. The item-use effects themselves (party overlay 91, bag overlay 70) are in no decompiled set.

| C | Rust |
|---|------|
| `FUN_02020F34` | `item::load_item_asset` (`load_item_data` for kind 0) |
| `FUN_02020FE4` | `item::item_param` |
| `FUN_02021014` (+ tail `020210AA`) | `item::record_param` (`usage_byte`) |
| `FUN_020210C8` | `item::usage_param` |
| `FUN_020212B0` / `sub_020212E4` | `item::tm_move` / `item::is_hm_move` |
| `FUN_02021350` | `item::hm_index` |
| `FUN_0202136C` / `FUN_0202138C` / `FUN_020213AC` | `item::is_mail` / `item::mail_index` / `item::mail_item` |
| `FUN_020213C0` | `item::is_berry` |
| `FUN_020213E0` | `item::outside_1ec_range` |
| `sub_020213F8` | `item::pair_lookup` |

Deviation: `FUN_020210AA` is not a function of its own (the tail of `FUN_02021014`), so it is covered through `record_param`.

## S5: save data model and persistence (`save` + one area per save block)

`save::SaveData` is the port's own save (serde, not the DS layout): one field per modelled block
of the original's block table (`savedata.c`, rodata 0x0209DA28, 69 entries `(id, size fn, init
fn)`). Each block is a plain serde type with `from_bytes` / `to_bytes` that convert losslessly
from / to the original block image; the ported accessors are its methods, and the differential
tests run them through that conversion (image -> model -> method -> image), so the hashes cover
the model's layout too. Party, boxes and bag (S1-S3 ports work on byte images) are kept as
images (hex strings in RON). `SaveData::new_game` runs every block's init (verified per area).

Persistence: `save::encode` / `decode` wrap the RON payload in an envelope (`b"BLKSAVE\0"`,
format version, payload length, CRC-32); `decode` rejects damage (`Corrupt`), newer formats
(`TooNew`) and unknown ones; a later format adds a migration step there. `save::Storage` stores
envelope bytes by slot key (`load` / `save` / `remove`, keys `[A-Za-z0-9_-]{1,64}`, errors as
`SaveError::Storage(String)`); `FileStorage` (native, `cfg(not(target_arch = "wasm32"))`,
`<dir>/<slot>.sav` via a temp file and rename), `MemoryStorage`; the browser implementation
belongs to `black-game` (G9). The crate builds for `wasm32-unknown-unknown`.

Verified per area with `tools/check-fn.sh black-storage <area> --per-function`:

| area | file / block | fns | area hash |
|---|---|---|---|
| `config` | `config.c`, in block 27 | 17 | 9605e23b2dde887d |
| `mystatus` | `mystatus.c`, in block 27 | 21 | 52b6255ea7a5d888 |
| `playtime` | `playtime.c`, in block 27 | 14 | 5dda302c9706f76e |
| `save_info` | block 27 (0x02008700..0x02008828) | 14 | 8db2fa8c380aaeaa |
| `mail` | `mail.c`, block 40 | 26 | fc5737239eef5298 |
| `regulation` | `regulation.c`, block 47 | 20 | a0251b23daa1c8d7 |
| `battle_rec` | `battle_rec.c`, block 48 + header helpers | 12 | f8023f1633f00e11 |
| `misc` | block 52 (money, badges; 0x0200C054..0x0200C3F4) | 47 | 675855371b7c5318 |
| `bsubway` | `bsubway_savedata.c`, blocks 57 / 58 + block-60 counters | 39 | 2504bff4b02ff8a6 |
| `symbol_save` | `symbol_save.c`, block 61 | 10 | 4664d2a0aefb017b |
| `battle_examination` | `battle_examination.c`, extra area 5 | 7 | 4d522b8b0b9fb51c |
| `save_control` | `save_control.c` status bytes | 3 | 42591b529ab90eb6 |

Every ported function's `/// C:` head names its symbol (or `unlabeled routine at 0x...`).
Each area's module doc lists the functions of its file that are not ported and why.

The blocks S5 left unmodelled are covered by S5b (below). Out of scope (D9): 30 `wifilist.c`, 34
`mystery_data.c`, 42 (musical), 56 (`gds_profile.c`), 59 (Battle Subway online), and the S5b list.

Deliberate deviations: the block fetch `FUN_020071CC(sav, id)` is ownership (the tests call its
stub where the original does); the symbol block is kept in plaintext (its at-rest cipher
`FUN_0200E024` / `FUN_0200E06C` is the `SymbolCrypt` hook, a no-op in the game); the inlined
easy-chat helpers `FUN_0202428C` / `FUN_02024570` (`pms_word.c`) are field writes in
`mail::Pms`; index arguments outside an array (a negative mailbox slot, a mode index above 8, a
bit index past a regulation bit set) panic where the original reads or writes neighbouring
memory.

## S5b: the remaining save blocks

Every in-scope block of the table is a `SaveData` field now (S5b fields carry `#[serde(default)]`
= the block's init state, so a format-1 payload written before S5b still decodes; the envelope
stays format 1). Identification (from the init / accessors and the debug source names of their
callers):

| block | purpose | area / model | fns | area hash |
|---|---|---|---|---|
| 28 | situation: location records, saved player position, egg-step / script counters | `situation` | 23 | 8b3cd181e582ddd6 |
| 33 | trainer card: signature, badge entries + time stamp | `trcard` | 16 | 656b888890ca7961 |
| 36 | `perapvoice.c`: Chatot's cry | `perapvoice` | 11 | 925dda879068b7ae |
| 37 | system info: RTC offset, MAC, birthday, game clock (RTC-change penalty) | `sysinfo` | 11 | ae4ee14994e88288 |
| 38 | game records (trainer-card counters, caps, at-rest cipher as hooks) | `record` | 17 | 804241b85770928f |
| 39 + 62 | easy chat: unlocked word groups, 20 sentences; the player's 5 messages | `pms_save` | 10 | ab633be5fad18c86 |
| 41 | field objects: 64 MMDL save records (pack / unpack) | `mmdl_save` | 4 | c741a3fb0ef9713c |
| 43 | White Forest / Black City town data | `wfbc_save` | 6 | 28912182e303057a |
| 45 | event flags / vars (accessors: `black_field::event_work`) | `field_save` (model) | 0 | - |
| 49 | Battle Box | `battle_box` | 10 | 106d5678e61c80ad |
| 50 | daycare | `daycare` | 13 | 761d2cb81ea63ee7 |
| 51 | kept MMDL positions (accessors: `black_field::fldmmdl`) | `field_save` (model) | 0 | - |
| 53 | Entralink | `intrude_save` | 18 | 4c29e941c014efe3 |
| 54 | registered shortcuts | `shortcut` | 8 | b3a70054d44643e9 |
| 55 | Pokedex (`zukan`) | `zukan_save` | 40 | ebbde1e72424b154 |
| 60 + 65 | DS logs (block 60 = Wi-Fi match log + the S5 Subway counters; 65 = Trial House) | `mac_log` | 13 | fce35828371dae38 |
| 66 | a Hall of Fame record | `dendou_save` | 6 | 9816c1e7b7c268b5 |
| 67 | two Pokemon records (trainer card / event battle) | `poke_records` | 11 | 37e4dec9c7d6ca8a |

Out of scope (D9) and not modelled: 29 (Wi-Fi history / Unity Tower countries; 0x02009454), 31
(players met through GTS negotiation / Wi-Fi; 0x02009D9C), 32 (C-Gear panel layout; 0x020090B0),
35 (Dream World; 0x02009228), 44 (IR feeling check; 0x0200AB24), 46 (GTS deposit; 0x0200AD78), 63
(research radar / survey; 0x0200E368), 64 (Unity Tower search snapshot; 0x0200E874), 68 (players
met over wireless; 0x0200ED38).

New-game inits that need the outside world are `NewGameHooks` methods with defaults: `rtc_now`
(block 37's clock), `mac_address` (block 60), `intrude_power_init` (overlay-11 `FUN_021763BC`,
block 53); the Battle Box name is `box_name(0)` (the same bank 9 entry 6). Deviations: the record
block keeps plaintext (cipher = `record::RecordCrypt`, `Plain` in the game; init key from the
V-count is 0); arguments the original does not check and that reach past a block (a shortcut
position above 25, an Entralink flag id from 0x80000021, Pokedex language 0 with species 0 / 1,
a Pokedex form index past the form bits, record ids from 167) panic. The OBJ-data load of
`mmdl_save::unpack` (`FUN_0216E2E8`), the Entralink power init and the personal / party / string
callees are hooks.
## S4b: item use on a party Pokemon (`item_use`) and bag helpers (`item_bag`)

Logic only (D3). Party-summary overlay 91 (memory set `party-summary`; `tools/check-fn.sh black-storage item_use --per-function`, spec
`tools/rom-diff/specs/black-storage/item_use.spec`) and bag overlay 70 (set `bag`; `item_bag`, spec `item_bag.spec`). The Pokemon is a
party record slice read and written through `black_pokemon::poke_tool::{party_get_value, party_set_value}` (run as real code on the ROM
side); the item through `item::record_param` on a record from `item::load_item_data`. Callees of other areas are the `ItemUseHooks` methods
(stubbed identically on both sides; only their arguments are checked).

| C | Rust |
|---|------|
| `FUN_overlay_d_91__021bbe74` | `item_use::use_kind` (classifier: which party-menu behaviour an item has) |
| `021bc17c` / `021bc1b4` / `021bc228` | `item_use::needs_move_slot` / `has_param_1b` / `move_prompt_id` |
| `021bdd78` | `item_use::can_use` (the "can this item be used on this Pokemon" check) |
| `021be2b4` | `item_use::apply_item` (cures, HP/revive, level up, PP, EVs, friendship) |
| `021be874` / `021be914` / `021be948` | `item_use::add_pp_ups` / `heal` / `ev_after` |
| `021be9bc` / `021bea48` | `item_use::friendship_raises` / `apply_friendship` |
| `021bc24c` / `021bc288` | `item_use::apply_plate_form` / `apply_drive_form` (Arceus plate / Genesect drive form) |
| `FUN_02017F90` / `FUN_02019D00` | `item_use::status_of` / `item_use::drive_form` |
| `021b619c` / `021ba1d0` (bag) | `item_bag::sell_price` / `item_bag::is_field_hm_item` |

Deviations: the party screen's work struct is not modelled, the callers pass the pieces the originals read from it (the heap id, the
move slot). `apply_item` returns 1 only for a change other than friendship (the original returns the same flag; a friendship-only item
reports 0 but still applies its friendship deltas). UI glue not ported: `021bc1d0` (find the first party member the item can be used
on, reads the screen work struct), `021b822c` / `021b8280` / `021ba218` / `021b6ec8` / `021b690c` / `021b7f54` and the labels
`021b6f08` / `021b7146` (bag menu building and state machine), `021bc2c8..` (screen transitions).
