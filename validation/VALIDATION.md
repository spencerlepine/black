# Validation findings (VALIDATION-AGENT)

Append-only ledger written by the validator (`VALIDATION-AGENT.md`), triaged by the coordinator (`MIGRATION-AGENT.md` §5b).
Line format: `- [STATUS] V-NNNN — severity — area — addr `rust_fn` — claim — found at <sha> (introduced <sha>) — repro: … — fix notes: … — run R-…`
Statuses: `[TODO]` (validator) → `[IMPORTED]` (coordinator put it in its task list) or `[REJECTED] — reason` (coordinator; not a bug).

IMPORTANT: delete Findings when they are complete. Otherwise this file will grow large!

## Findings

- [IMPORTED] V-0020 (filed as V-0001; renumbered by the coordinator, IDs V-0001..V-0019 were already used) — low — docs — - — SYMBOLS.md stale: missing 16 entries for newly ported functions in api (5), handex (5), hem (5), plus one unlabeled entry — found at 82df00c (introduced e757a48 B10bc-spec) — repro: `python3 -I tools/sync-docs.py && git status --porcelain -- black-rust/crates/black-battle/SYMBOLS.md` shows modified — fix notes: the file regenerates cleanly; commit the regenerated version or ensure sync-docs.py runs in the porting workflow — run R-20261013-0451
