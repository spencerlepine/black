# Validation findings (VALIDATION-AGENT)

Append-only ledger written by the validator (`VALIDATION-AGENT.md`), triaged by the coordinator (`MIGRATION-AGENT.md` §5b).
Line format: `- [STATUS] V-NNNN — severity — area — addr `rust_fn` — claim — found at <sha> (introduced <sha>) — repro: … — fix notes: … — run R-…`
Statuses: `[TODO]` (validator) → `[IMPORTED]` (coordinator put it in its task list) or `[REJECTED] — reason` (coordinator; not a bug).

IMPORTANT: delete Findings when they are complete. Otherwise this file will grow large!

## Findings

- [TODO] V-0019 — low — docs — black-storage/box_data (`tools/check-fn.sh`) — `tools/check-fn.sh black-storage box_data --per-function` (the convention in VALIDATION-AGENT §2 and the PORTED enumeration) exits 2 "no tools/rom-diff/specs/black-storage/box_data.spec" because the spec file is named `box.spec`; box_data.rs docs (lines 2-5) say to run `check-fn.sh black-storage box` with FNDIFF_SRC/TESTS/FILTER env, but only FNDIFF_SPEC is actually needed, so the docs are stale/over-complicated and the area is skipped by anyone using the standard enumeration — found at b171e89 (introduced 729e65e) — repro: `tools/check-fn.sh black-storage box_data --per-function` -> exit 2; `FNDIFF_SPEC=tools/rom-diff/specs/black-storage/box.spec tools/check-fn.sh black-storage box_data --per-function` -> 33/33 match, rom=44c0c98734f3630c rust=44c0c98734f3630c, MATCH: recorded — fix notes: rename box.spec -> box_data.spec (and the include_str! in box_data/tests.rs:245) or add a note to VALIDATION-AGENT §2 / box_data.rs docs; hash equals the one in task-file S2b — run R-20261013-0140
