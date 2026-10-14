# Worker prompt template

Moved out of `MIGRATION-AGENT.md` §5 on 2026-10-13. Fill the `<>`; keep it self-contained — workers read only the files named here, never the whole plan. Rules, model routing and the definition of done are in [§5](../MIGRATION-AGENT.md).

## Porting / spec / harness job

```
You are a worker on the Pokémon Black Rust rewrite. Item: <ID> — <checklist line>.
Your worktree is a checkout of the black repo; work only in <worktree>/black-rust/.
Read first (all under <worktree>/): MIGRATION-AGENT.md §1, §4 and §5 (rules + definition of done); docs/migration/ENV.md section "RE workflow"; docs/migration/NOTES.md sub-heading(s) <named sub-headings relevant to area>; the docs/migration/OPEN.md bullets that mention <area>.
Inputs (read-only): pseudo-C in /tmp/blk/re/<set>/pc/ (index.jsonl for callers/callees), packets from tools/re/packet.py, ROM build at $BLACK_ROM_BUILD (absolute path; not in your worktree).
Scope: <functions / address range / files you own>. Do not edit: Cargo.toml of other crates, tools/re/signatures.tsv, tools/re/sets.txt, MIGRATION-AGENT.md, anything under docs/migration/.
Method: docs/migration/ENV.md "RE workflow". Use the decompiled C, not the .s files, unless the C has decompiler warnings. Port per MIGRATION-AGENT.md §4. Verify every function with tools/rom-diff (Unicorn) against the original; record hashes.
Env (workers cannot `export`/`.`; prefix each command instead, one plain command per call): `env CARGO_TARGET_DIR=/tmp/blk/target-<ID> BLK_OUT=/tmp/blk/out-<ID> BLACK_ROM_BUILD=$HOME/workplace/black/black-rom/build/black.us GATE_NO_GAME=1 tools/gate.sh` (same prefix for check-*.sh; add `PATH=/home/ec2-user/.cargo/bin:/usr/local/bin:/usr/bin:/bin` first if cargo isn't found). Create/edit files with the Write/Edit tools, not heredocs; put scripts in files and run `python3 <file>`. Unicorn python = /tmp/blk/venv/bin/python -I.
The job has no time or token limit: finish it in this session (a spec job of ~100 fns is ~300–500k tokens). Never report BLOCKED for size or sandbox friction; only for a real harness defect.
Do not commit, push, rebase or merge. Stage your changes with git add <explicit paths> (never -A), run tools/gate.sh until green, and leave them staged; the coordinator reviews and commits.
You may not edit tools/rom-diff/*, tools/ui-diff/*, existing stubs, check-*.sh, or recorded hashes in tests (except by running the check script, which records only on a match). If the harness is missing or wrong, stop and report "BLOCKED: harness".
Final message, first line "Task: <ID>" (or "BLOCKED: <reason>"), then: branch, summary, functions ported / verified (+ hashes), deviations, new open questions (for docs/migration/OPEN.md), new prototypes for signatures.tsv, tokens/time spent if known.
```

## Model routing (which `model:` to pass)

`Agent({subagent_type: "general-purpose", isolation: "worktree", model: "<tier>", prompt: …})`. The coordinator itself runs on Opus. Why tiers need rules, and the Z14 pilot numbers: `NOTES.md` "Lessons".

| Model                       | Use for                                                                                                                                                                                                                                                                                                                                                   |
| --------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Opus** (coordinator)      | Waves, merges, SEQ architecture items (B9 split, F1, Z2/Z6 shared types, trait design), review of any change touching a harness or hash. Spec/harness jobs of 100+ functions (Sonnet kept quitting early on them in session 4).                                                                                                                         |
| **Sonnet** (default worker) | Stateful flow code (server flow, script VM), writing/extending harnesses and stub semantics (as separate harness jobs, never in the same job as the port), T1 triage, asset readers, Bevy screens, the UI-diff tool code (`tools/ui-diff/*`, written once by a harness job, never edited by screen workers), Z-tooling. Anything where mistakes are expensive or not caught by the diff. |
| **Haiku** (bulk worker)     | Leaf functions from a packet with an existing harness: cluster instances generated from a ported template, simple handlers, the mismatch-fix loop given only the failing case. Default for leaf handler batches with an existing harness (Z14 decision).                                                                                                 |

Rules that make Haiku safe (summarised in [§5](../MIGRATION-AGENT.md)):

1. **Escalation:** Haiku fails ~3 diff iterations (or returns BLOCKED) → re-spawn the item on Sonnet with the failure output. Sonnet fails twice → `[!]` for the coordinator (Opus) to handle or split.
2. **Hands off the tests:** porting workers may not touch `tools/rom-diff/*`, `tools/ui-diff/*`, stubs, `check-*.sh`, or recorded hashes/scores (other than via the check script). The coordinator rejects any porting branch whose staged diff touches them; harness changes come only from harness jobs and get coordinator review.
3. **Gate:** the gate must pass (clippy/fmt catch some quality issues, not faithfulness), and the coordinator re-runs the item's diff check on merged `main` (rule 6).
4. **Spot audit:** every ~20 Haiku merges, the coordinator (or a Sonnet job) reviews a random 2–3 ports against their pseudo-C for faithfulness the fuzzing could miss (rare branches); findings go to `OPEN.md`.

Re-check the Haiku/Sonnet choice every few waves per **verified** function (tokens and wall-clock per verified function, not per call) and record it in `LOG.md`.

## UI screen job (Phase G/U; replaces the Method/Inputs lines above)

```
Inputs (read-only): the original asm/pseudo-C of the screen as the behaviour/layout/timing spec (D3: rewrite in Bevy, never port DS hardware calls); extracted assets under $BLACK_ASSETS; reference frames in <repo>/black-rust/ui-ref/<case>/ when present (gitignored, never copy them anywhere).
Method: edit the screen → `python3 -I tools/ui-diff/capture_port.py <case>` → `/tmp/blk/venv/bin/python -I tools/ui-diff/compare.py --case <case>` → read ui-ref/report/<case>.json (scores per screen per frame; pass ≥ 0.70). You may open a failure image ui-ref/report/<case>_<frame>.png (reference | port | heat-map) at most 3 times for this item; after that, report the best score and stop. Cases for this screen go into ui-ref/manifest.json (committed); animations are multi-frame cases. If ui-ref/ has no goldens for a case, add the recipe to your report for docs/migration/HUMAN.md and finish with "pending sign-off".
```
