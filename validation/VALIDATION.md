# Validation findings (VALIDATION-AGENT)

Append-only ledger written by the validator (`VALIDATION-AGENT.md`), triaged by the coordinator (`MIGRATION-AGENT.md` §5b).
Line format: `- [STATUS] V-NNNN — severity — area — addr `rust_fn` — claim — found at <sha> (introduced <sha>) — repro: … — fix notes: … — run R-…`
Statuses: `[TODO]` (validator) → `[IMPORTED]` (coordinator put it in its task list) or `[REJECTED] — reason` (coordinator; not a bug).

IMPORTANT: delete Findings when they are complete. Otherwise this file will grow large!

## Findings

- [TODO] V-0003 — low — harness — 021d74f8 `EventVarStack::push` (+ 021d77c4 `slot_for_set`) — the stock event_var fuzz (`gen.py event_var` / `event_var::tests::matches_rom_fuzz`, 300 seeds × 60 ops, labels 2..7, mul 0..0x3FFF, clamp bounds 0..299) can never reach the push-overflow branch at 021d74f8 (0x021d752a: FUN_02034714 then return without writing) or the full-stack fallback of 021d77c4 (returns slot 0, so a set overwrites the separator): at most 60 labels are ever written, fewer than the 0x5F needed; labels 0/1, labels > 7, negative multipliers and negative/inverted clamp bounds are also never generated — found at c7cc534 (introduced 13d722d) — repro: scratch widening (1000 seeds × 200 ops, push weight 4/14, label = 25% full u16 else 0..7, mul 50% full-range i32, clamp bounds 50% signed full-range) gives ROM `event_var_wide 923d9ae82203fab5` = Rust `923d9ae82203fab5`, with 342 push overflows, 178 full-stack sets, 3602 negative multipliers hit — fix notes: no behaviour bug (Rust matches the original on all of the above); adopt the widened generator next to the stock one in tools/rom-diff/gen.py and event_var.rs tests so the overflow / full-stack / sign paths stay pinned; same RNG draw order on both sides (label draw: `r%4==0 ? r&0xFFFF : r%8`; op 6 and op 8 draw a selector before their values) — run R-20261012-2212
