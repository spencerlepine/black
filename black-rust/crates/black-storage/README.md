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
