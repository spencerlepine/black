# black-battle

Safe Rust port of battle logic from overlay 93 (`asm/battle_*.s`; only assembly exists). The
original symbol is in each item's doc comment. Undecompiled callees are traits in `hooks.rs`:
`BattleMon` (`BattleMon_Get`, `HasType`, poke ID, species), `BattleParty`, `PersonalData`
(`FUN_02019C90`) and `Arm9Misc` (`FUN_0202A3FC`). `Heap_AllocDebug`/`Heap_Free` use
`black_core::heap::Heap`.

## Verification

Every function in the tables except the four marked † is hashed against the **original machine
code** run under an emulator (`tools/rom-diff`). The Rust test and the Python harness share the
same input generator, and each stub mirrors its Rust mock. Hand-checked unit tests cover known
game behaviour (type chart, damage examples, stage ratios, hit-count distribution). Mutating the
port (e.g. the rounding boundary in `fx_mul`, or signed vs unsigned division) breaks the hashes.

## battle_calc.s: `calc`

| asm | Rust |
|---|---|
| `FUN_overlay_d_93__021d7840` † | `calc::BattleCalc::init` |
| `FUN_overlay_d_93__021d787c` † | `BattleCalc::set_rng` |
| `FUN_overlay_d_93__021d7894` † | `BattleCalc::free` |
| `FUN_overlay_d_93__021d804c` † | `BattleCalc::set_unk_00` |
| `FUN_overlay_d_93__021d78ac` | `BattleCalc::rand` (`MathRandContext32::next`) |
| `FUN_overlay_d_93__021d7930` | `BattleCalc::roll_critical` |
| `FUN_overlay_d_93__021d7af0` | `BattleCalc::roll_percent` |
| `FUN_overlay_d_93__021d7b8c` | `BattleCalc::rand_range` |
| `FUN_overlay_d_93__021d7ba4` | `BattleCalc::roll_hit_count` |
| `FUN_overlay_d_93__021d7d18` | `BattleCalc::make_default_sick_cont` |
| `FUN_overlay_d_93__021d7dc0` | `BattleCalc::make_sick_cont` |
| `FUN_overlay_d_93__021d7c70` | `BattleCalc::make_move_sick_cont` |
| `FUN_overlay_d_93__021d78e4` | `calc::apply_stat_stage` |
| `FUN_overlay_d_93__021d7908` | `calc::apply_accuracy_stage` |
| `FUN_overlay_d_93__021d794c` | `calc::type_effectiveness` |
| `FUN_overlay_d_93__021d799c` | `calc::type_effectiveness_vs_pair` |
| `FUN_overlay_d_93__021d79e8` | `calc::combine_effectiveness` |
| `CalcBaseDamage` | `calc::calc_base_damage` |
| `ApplyTypeEffectiveness` | `calc::apply_type_effectiveness` |
| `FUN_overlay_d_93__021d7a9c` | `calc::resisting_types` |
| `GetTypeEffectivenessClass` | `calc::effectiveness_class` |
| `FUN_overlay_d_93__021d7b10` / `7b2c` | `calc::fx_mul` / `fx_mul_min1` |
| `FUN_overlay_d_93__021d7b3c` | `calc::percent_mul` |
| `FUN_overlay_d_93__021d7b60` / `7b74` | `calc::max_hp_div` / `max_hp_div_min1` |
| `FUN_overlay_d_93__021d7bd4` | `calc::weather_residual_damage` |
| `FUN_overlay_d_93__021d7c38` | `calc::weather_power_modifier` |
| `FUN_overlay_d_93__021d7eb4` | `calc::exp_yield` |
| `FUN_overlay_d_93__021d7ef0` | `calc::is_not_0_or_3` |
| `FUN_overlay_d_93__021d7f00` / `7f2c` | `calc::fun_021d7f00` / `fun_021d7f2c` |
| `FUN_overlay_d_93__021d7f38` / `7f4c` / `7f60` | `calc::fun_021d7f38` / `fun_021d7f4c` / `fun_021d7f60` |
| `FUN_overlay_d_93__021d7ff8` | `calc::prize_money` |
| `FUN_overlay_d_93__021d806c` / `8090` | `calc::pack_5bit_x6` / `unpack_5bit_x6` |
| `FUN_overlay_d_93__021d80a8` | `calc::fun_021d80a8` |
| `FUN_0209C2B0` (`_u32_div_f`) | `calc::u32_div` / `u32_divmod` |
| `battle_calc_rodata` (0x021F041C, first 0x181 bytes) | `calc::BATTLE_CALC_RODATA` |

† Unit-tested only. These are state setters, plus the `Heap` call whose implementation is a
trait.

## battle_calc.s: `event_var`

| asm | Rust |
|---|---|
| global `0x021F3ECC` | `event_var::EventVarStack` |
| `FUN_overlay_d_93__021d74f8` / `7540` | `EventVarStack::push` / `pop` |
| `FUN_overlay_d_93__021d75a8` / `75ec` / `7630` / `7674` | `set_value` / `set_const_value` / `set_rewrite_once_value` / `set_mul_value` |
| `FUN_overlay_d_93__021d76bc` | `rewrite_value` |
| `FUN_overlay_d_93__021d7704` | `mul_value` |
| `FUN_overlay_d_93__021d775c` | `get_value` |
| unlabeled Thumb at `0x021D778C` (data bytes in the .s) | `get_value_if_exists` |
| `FUN_overlay_d_93__021d77c4` / `77e8` / `780c` | `slot_for_set` / `find` / `clamp` |

## battle_view_core.s: `sick_cont`, `poke_type`

| asm | Rust |
|---|---|
| `FUN_overlay_d_93__021e8940` / `896c` | `SickCont::turn` / `turn_param` |
| `FUN_overlay_d_93__021e899c` / `89c0` / `89d4` / `89f8` | `SickCont::poke` / `permanent` / `permanent_param` / `permanent_count` |
| `FUN_overlay_d_93__021e8a28` / `8a58` / `8a78` | `SickCont::poke_turn` / `badly_poisoned` / `is_badly_poisoned` |
| `FUN_overlay_d_93__021e8ce0` / `8cf0` | `poke_type::make_pair` / `make_mono_pair` |
| `FUN_overlay_d_93__021e8cfc` / `8d04` / `8d0c` | `pair_type1` / `pair_type2` / `split_pair` |
| `FUN_overlay_d_93__021e8d24` / `8da8` | `pair_has_type` / `pair_is_mono` |

## handlers::effects (B7: overlay-95 registries)

Decoded by B7h; the full table and evidence are in the module doc of
`src/handlers/effects.rs`. Summary: overlay 95 (linked at 0x06898020) has three registries with
the same getter protocol as abilities/items/moves: field effects 0x0689D650 (8 keys, factor
type 3, owner 0), **position effects** 0x0689D6C8 (5 keys, type 1, owner = battle position;
Wish, Lunar Dance, Healing Wish, Future Sight / Doom Desire, Baton Pass) and **side effects**
0x0689D780 (14 keys with a max layer count, type 2, owner = side; Reflect, Light Screen,
Safeguard, Mist, Tailwind, Lucky Chant, Spikes, Toxic Spikes, Stealth Rock, Wide Guard, Quick
Guard, 3 Pledge combos). The "type-2 adder at 0x06898328" is the Wish handler; the adder is
`06898c10`. 30 distinct handlers (`tools/rom-diff/handlers.py effects --list`), checked with
`tools/check-handlers.sh effects`.

## Deviations

- Function and argument types are narrowed to what callers pass (`u8` types, `u16` type pairs
  and move condition params). Table reads index `BATTLE_CALC_RODATA`, so any index inside the
  table reads the same bytes as the original; reads past the end panic instead of reading
  neighbouring memory.
- `EventVarStack::push` returns `Err(StackOverflow)` where the original calls `FUN_02034714`
  (undecompiled diagnostic) and returns. The state change is identical.
- `BattleCalc::init` drops a previously allocated work buffer. The original overwrites the
  pointer, leaking it.
- `SickCont` constructors use plain ORs. The original's per-field clear masks act on a zeroed
  word with non-overlapping fields, so the results are identical (hash-verified).
- Not ported: `FUN_overlay_d_93__021d7e10` (calls into the ARM overlay at `0x0689C9E4`),
  `021d7f88`/`021d7fbc` (four undecompiled ARM9 item/save calls), `021d805c` (tail call to
  `FUN_02020FE4`), `021d8058` (empty), and two unreferenced data-encoded duplicates of
  `SickCont::turn`/`poke` at `0x021D7D78`.

## Inferred names (UNVERIFIED semantics, verified behaviour)

`bpp::{MAX_HP, LEVEL, FORM}`, the `weather` and `sick` ID constants, and personal param 9 as the
"base EXP yield" are named from usage. The code behaves identically whatever they mean.
