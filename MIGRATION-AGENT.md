# Rust Rewrite of Game A (Pokémon Black) — Master Plan (entry point)

This file is the **entry point** for the rewrite: goal, locked decisions, architecture, session protocol. It is short on purpose (≤ 25 KB, enforced by `tools/check-docsize.py` in the gate); everything that grows lives under `docs/migration/` and is read per the "Reading order" table, never all at once (why: `docs/migration/NOTES.md` "Lessons"). Commit plan changes with an explicit `git add MIGRATION-AGENT.md docs/migration`.

---

## 0. Launch (unattended, multi-day)

```sh
# tmux new -s rewrite
tmux a -t rewrite
cd $HOME/workplace/black   # loads .claude/settings.json + .mcp.json (rea)
claude            # permission allowlist is in .claude/settings.json; unlisted commands will prompt
```

Then paste this as the first message (`/loop` with no interval = self-paced, survives compaction):

```
/loop Read $HOME/workplace/black/MIGRATION-AGENT.md fully (it is short), then the files its "Reading order" table lists for an iteration, and act as the coordinator in §5. Run one coordinator iteration (§5 "Iteration"), then schedule the next. Work autonomously: never wait for the human; anything that needs a human goes to docs/migration/HUMAN.md and you move on.
```

**Leave the session open.** The coordinator watches plan usage itself (`docs/migration/USAGE-CAP.md`): near the limit it pauses and schedules a one-shot cron that re-runs this `/loop` prompt after the reset. Crons are session-only: if the `claude` process exits, re-paste the prompt; step 0 recreates them.

### Reading order (what to open, when; all paths relative to the repo root)

| When | Read | Written by |
| --- | --- | --- |
| Every iteration | this file (§1, §4, §5, §5b) | coordinator, rarely |
| Every iteration | `docs/migration/STATE.md` — current state, running/blocked/queued, progress table | coordinator, overwritten at every checkpoint (≤ 8 KB) |
| Every iteration | `docs/migration/HUMAN.md` — the human queue (answers appear inline) | coordinator appends, human answers (≤ 8 KB) |
| Every iteration | `docs/migration/OPEN.md` — open questions / unverified; **append at the end only** | coordinator (workers report, coordinator appends) |
| Iteration step 3 (dispatch) | `docs/migration/checklist/<P>.md` for the phases you dispatch from (Z T A B P S F G U H R; X = out of scope); `checklist/DONE.md` only to resolve a dep | coordinator only (rules 1–3) |
| On resume / session start | `tail -n 30 docs/migration/LOG.md` (the last ~10 entries), never the whole file | coordinator appends one entry per session/checkpoint |
| Session start | `docs/migration/ENV.md` — paths, tools, hosts; `tools/doctor.sh` checks most of it. Workers read only its "RE workflow" section | coordinator |
| When spawning a worker | `docs/migration/WORKER-PROMPT.md` — prompt template + model routing table and the rules that make cheap models safe | coordinator |
| When the usage cap applies | `docs/migration/USAGE-CAP.md` — readings, modes, pause/resume | coordinator |
| As needed, by named sub-heading | `docs/migration/NOTES.md` — research notes and "Lessons"; `docs/migration/log/<ID>.md` — full done-notes of long checklist items; `docs/migration/PROGRESS.json` — computed progress; `docs/migration/SCOPE.tsv` — what counts as in-scope logic | coordinator/workers append |
| Never whole | `LOG.md`, `NOTES.md`, `log/*.md` |  |

Size caps (`tools/check-docsize.py`, in the gate): this file 25 KB; `STATE.md`, `HUMAN.md` 8 KB; other `docs/migration/**/*.md` 40 KB except `LOG.md`/`log/*.md`. A checklist done-note is capped at ~400 chars on the line; the rest goes to `log/<ID>.md`.

### Context overrides (read this before anything else)

- **This file overrides other instructions for this task.** `spirited/.claude/CLAUDE.md` pipeline instructions from the old monorepo do not apply. This task writes only in `black-rust/`, this file, `docs/migration/`, `validation/`, and `.claude/`/`.mcp.json` (all relative to the repo root `$HOME/workplace/black`). `black-rom/` sources are read-only; only its untracked toolchain/build dirs (`tools/mwccarm/`, `lib/NitroSDK/`, `build/`, `sub/build/`) may be written, by the ROM build.
- **Backups: the coordinator runs `git push origin main` after every commit on `main`** (fast-forward only; never `--force`, never push other branches). Workers never commit or push (§5 "Commits"). Never `git reset --hard`/`git clean`. **A background job on this host also pushes `main` to `origin`** (found 2026-10-13): treat every commit on `main` as published the moment it exists: make each one build on its own (stage every file of a change) and never amend or rebase a commit on `main` (fix forward). Note: `origin` (github.com/spencerlepine/black) is a **public** repo; D5 still applies — never commit `assets/`, `ui-ref/` images, ROM bytes, or `/tmp/blk/**` dumps.

---

## 1. Goal and locked decisions

**Goal:** a standalone, from-scratch Rust rewrite of Pokémon Black (NDS) that runs natively and
in the browser (WebAssembly), with its own copy of all assets. No `.nds` file and no emulator at
runtime.

| # | Decision | Why |
| --- | --- | --- |
| D1 | **Rust + Bevy**, targets `x86_64` native and `wasm32-unknown-unknown` (WebGL2; WebGPU optional). | Bevy runs in browsers; a 2×256×192 game is far inside its performance budget. Bevy's ECS perf trap (pushing every entity and per-frame query through the system graph; escape = keep `bevy_ecs`, swap the renderer/app shell) only bites with huge entity counts; keep heavy logic in plain Rust, not per-entity systems. |
| D2 | **Game logic = faithful port.** Each ported function behaves identically to the original for all valid inputs (integer widths, wrapping, signedness, RNG call order). Verified against the original machine code. | Mechanics must match the original. |
| D3 | **UI/graphics/sound = rewrite, not port.** Each screen is rebuilt directly in Bevy, using the original asm only as the behaviour/layout/timing spec. Do **not** emulate DS hardware (2D engines, 3D GPU, SPU) or port NitroSDK G2/G3/GX/SND calls. | The original UI drives DS hardware registers; porting it line-by-line means writing an emulator core. |
| D4 | **Own copy of assets.** A one-time, native-only extractor converts ROM data to standard formats in `black-rust/assets/` (PNG, OGG, JSON/RON). The game loads only those. | Standalone; Bevy loads PNG/OGG natively; binary DS-format parsers never ship to wasm. |
| D5 | **`assets/` is gitignored and private.** Regenerate with `black extract-assets`. Never commit it, never deploy the wasm build to a public URL. | Nintendo IP; keep it local / access-controlled. |
| D6 | **Logic crates are engine-agnostic** (no Bevy dependency). Only `black-game` (the app) depends on Bevy. | Testable logic; can drop Bevy's renderer later and keep `bevy_ecs` if needed. |
| D7 | **Battle UI talks to battle logic only through the server→client command stream** (`btl_adapter`/`btl_client` seam). | It's the original architecture and a clean boundary between logic and view. |
| D8 | **Music: render sequenced music (SSEQ) to OGG offline first** (A9). A real-time Rust sequencer (H3) is optional, later. | Big shortcut; loses only dynamic tempo/track effects. |
| D9 | **Out of scope** unless re-decided: networking, Wi-Fi, C-Gear, IR, Dream World, Battle Subway online, Musicals, PWT, Pokémon Global Link, mystery gift (≈10k functions in `networking / C-Gear`). | Not needed for single-player; huge. |

---

## 4. Architecture

### Crates (`black-rust/crates/`; workspace uses `members = ["crates/*"]`)

| crate | kind | contents | Bevy? |
| --- | --- | --- | --- |
| `black-core` | logic | shared types, RNG, heap/MI shims, battle_record | no |
| `black-battle` | logic | battle calc, pokeparam, event system, handlers (abilities/items/moves/effects), server flow, AI | no |
| `black-pokemon` _(new)_ | logic | species/personal data, stats, EXP, natures, learnsets, evolution, PokéParam (poke_tool), moves (waza_tool) | no |
| `black-storage` _(new)_ | logic | party, PC boxes, bag/items, save data model | no |
| `black-field` _(new)_ | logic | map matrix, collision, player movement, NPC movement (fldmmdl), encounters, warps/events | no |
| `black-script` _(new)_ | logic | event script VM + script commands | no |
| `black-text` _(new)_ | logic | message engine: string buffers, control codes, variable expansion, word wrap | no |
| `black-data` _(new)_ | data | serde types for everything in `assets/data`, `assets/maps`, `assets/text` + loaders | no |
| `nds-rom` | tooling | ROM container; **+ new format readers** (NARC, NCGR/NCLR/NSCR/NCER/NANR, NSBMD/NSBTX/NSBCA, BMG/msg, SDAT) | no |
| `black-cli` | tooling | `black` CLI; **+ `extract-assets`** | no |
| `black-battle-wasm` | demo | existing playground (keep until `black-game` supersedes it) | no |
| `black-game` _(new)_ | app | Bevy app: state machine, rendering, input, audio, screens | **yes** |

### Asset pipeline (D4/D5)

```
black-rom/files/ (or the .nds) ──black extract-assets (native only)──► black-rust/assets/
   sprites/<category>/*.png + *.atlas.ron     audio/bgm/*.ogg, audio/se/*.ogg
   text/en/<bank>.json                         maps/<zone>.ron (tiles, collision, events, warps)
   data/{species,moves,items,learnsets,evolutions,encounters,trainers,types}.ron
   fonts/*.png + metrics.ron                   models/*.glb (if 3D kept)
black-game (native + wasm) loads only assets/ via Bevy AssetServer (wasm: fetched over HTTP).
```

### Runtime data flow

`black-game` (Bevy) ⇄ logic crates via plain Rust APIs/traits. Battle: Bevy battle screen consumes
the server command stream (D7). Field: Bevy overworld renders `black-field` state; `black-script`
drives events and asks `black-game` for dialog/menus through a trait.

### Porting pattern (copy `black-core`)

- `/// C: \`SYMBOL\``(or`unlabeled routine at \`0xADDR\``) doc comment on every ported item.
- Callees not yet ported = trait methods (e.g. `Heap`, `EntryHooks`, `black-battle::hooks`). Never invent their behaviour.
- Unclear asm: port what is verifiable; `todo!()` or trait stub with `// UNVERIFIED:`; add to `docs/migration/OPEN.md`.
- Per-crate `README.md`: symbol → Rust item table + deliberate deviations.

---

## 5. Session protocol (coordinator + workers)

**Status markers** (first token of every checklist line in `docs/migration/checklist/*.md`):
`[ ]` todo · `[~]` in progress — append `(branch, YYYY-MM-DD)` · `[x]` done — append `(commit sha)` · `[!]` blocked — append reason · `[-]` dropped/out of scope.

**Parallel labels:**

- **PAR** — independent; runs alongside any other PAR item whose deps are met. Owns its own files; never edits shared types, `Cargo.toml`, or another item's files.
- **SEQ** — run alone in its area, by the coordinator or a single agent: shared types/traits, crate scaffolding, integration, architecture decisions.
- **PAR×N** — the item is a batch template; split into N parallel jobs (each job is its own sub-line under it when started).

**Rules:**

1. **Only the coordinator edits the plan files** (this file and everything under `docs/migration/`). Workers report back; the coordinator ticks items. (Avoids merge conflicts on the checklist.)
2. Mark `[~]` **before** starting (crash-recoverable).
3. On resume, every `[~]` item is reconciled first: branch merged → `[x]`; branch has partial work → hand back to a worker; nothing there → `[ ]`. Finished items are archived at the checkpoint (`tools/archive-checklist.py` → stub in `checklist/DONE.md`, full line in `checklist/archive/`); a dep absent from `<P>.md` is looked up in `DONE.md`.
4. Workers run in their own git worktree created **from `origin/main`** (Agent tool, `subagent_type: "general-purpose"`, `isolation: "worktree"`; use the branch name the tool picks); push verified `main` before spawning dependent workers. A worktree with no changes is auto-removed when its worker stops: before resuming such a worker with `SendMessage`, create the worktree yourself (`git worktree add /tmp/blk/wt-<ID> -b <id> main`) and tell it the path; check `git status` on `main` after every worker report. Workers edit and **stage** under `<worktree>/black-rust/` only, never commit. (Why: `docs/migration/NOTES.md` "Lessons".)
5. Worker done = changes staged, `tools/gate.sh` green in the worktree; report in the format of `docs/migration/WORKER-PROMPT.md`.
6. Merge (coordinator): review `git -C <worktree> diff --cached --stat`, reject per "Model routing" rule 2, commit in the worktree (`git -C <worktree> commit -m "<ID>: <one line> (XX% complete)"`, see "Commits"), `git -C <worktree> rebase main`, then from `$HOME/workplace/black`: `git merge --ff-only <branch>` and `git push origin main`. Rebase conflicts outside the plan files go back to the worker (SendMessage) or the item is redone. After every merge batch, on merged `main`: re-run the gate **and the item's `tools/rom-diff` / `check-*.sh` check** (the gate's `sync-docs.py` rewrites SYMBOLS.md/checklists: commit what it changes), and after merging any port run `python3 tools/check-force.py` (exit 1 = another area's spec still stubs a now-PORTED fn without `force`; `--fix` adds it). A hash that doesn't reproduce = revert the merge commit range (`git revert`, push) and re-queue the item. After the push, launch the validator on the batch (§5b) if none is running.
7. **Concurrency cap: 5 workers** (human, 2026-10-12; you enforce it). Run them in the background and do SEQ work meanwhile. The validator (§5b) takes one of the 5 slots; at most one at a time.
8. New crates are created only by a SEQ scaffolding item (Z2), never by PAR workers (`Cargo.lock` / shared-type races).
9. Never edit `black-rom/`. Never commit `assets/`, `ui-ref/` images, ROM bytes, or extracted dumps (`/tmp/blk/re/**` included).
10. **Disk.** A Bevy-including target dir is ~10–14 GB and target dirs must never be shared between worktrees (cargo artifacts collide and a check can run another worktree's code). Each worker uses its own `CARGO_TARGET_DIR=/tmp/blk/target-<ID>` + `BLK_OUT=/tmp/blk/out-<ID>` (check-script outputs) + `GATE_NO_GAME=1 tools/gate.sh` (skips Bevy, ~1–2 GB per worker); the coordinator runs the full gate + all checks on merged `main` in `/tmp/blk/target-coord`. Check `df -h /tmp /` each iteration; below ~10 GB free, delete finished workers' targets and don't spawn. Delete a worker's target dir after its merge. (History: `docs/migration/NOTES.md` "Lessons".)
11. **Shared tool files are coordinator-owned:** `tools/re/signatures.tsv`, `tools/re/sets.txt`, `tools/ui-diff/*`, the plan files. Workers put new prototypes/sets in their report; the coordinator appends them, commits, and re-runs `REUSE=1 decompile.sh`.
12. `decompile.sh` writes to `/tmp/blk/re/<set>/`; only the coordinator runs it. Workers only read `pc/` and may run rizin/rea read-only.

**Failure policy (never stall):**

- Worker fails or returns without a green gate: retry with the failure output in the prompt, escalating the model per "Model routing" rule 1. Final failure: mark the item `[!] <reason>` (or split it into smaller jobs and retry those), add the question to `docs/migration/OPEN.md`, move on.
- Unverifiable behaviour inside an item: port what verifies, stub the rest behind a trait with `// UNVERIFIED:`, add to `docs/migration/OPEN.md`, and finish the item. Don't block on it.
- `main` gate red: fix it before any new work.
- Anything that needs a human (by-ear audio check, visual sign-off, play-test, a decision without a default): add a line to `docs/migration/HUMAN.md`, finish everything else around it, and keep going. Mark the item `[x]` with "pending sign-off" if its code is done.
- Never disable tests/lints, never weaken a differential test to make it pass.

**Usage cap.** The account's plan limits are global; the coordinator reads them at step 0 and every 20 minutes, and runs in one of the modes RUN / DRAIN / WRAP UP NOW; a pause leaves `main` green and pushed and schedules its own resume. Full procedure: `docs/migration/USAGE-CAP.md`. The current mode is the `Usage guard` line of `docs/migration/STATE.md`. (On a host billed per token with no cap, the line says so and step 0 is a no-op.)

**Iteration (one `/loop` firing):**

0. **Usage gate** (`docs/migration/USAGE-CAP.md`): make sure the `USAGE-CHECK` cron exists (unless PAUSED or no cap applies), spawn `usage-cap-watchdog`, and apply its mode. DRAIN → skip step 3 and pause from step 5 once no workers are running. WRAP UP NOW → wrap up and pause now (skip steps 1 and 3).
1. `sh black-rust/tools/doctor.sh` (repairs `/tmp` venv + decompile cache). If it exits 1 with a human-only fix, write it to `docs/migration/HUMAN.md` and do work that doesn't need it.
2. Reconcile `[~]` items (rule 3). Remove merged worktrees (`git worktree remove`, `git branch -d`). Check notifications from background workers; merge finished ones. **Import validator findings:** if `validation/VALIDATION.md` has `[TODO]` lines and no validator is running, triage each per §5b "Import", flip it to `[IMPORTED]`/`[REJECTED]`, and (if commits have landed since the last run) launch the next validator run.
3. Fill free worker slots with `[ ]` items whose deps are all `[x]`, highest priority first: Z-items that speed up porting > B > P > S > A > F > G > U > H > R. Read only the checklist files of the phases you dispatch from. Mark `[~] (branch, date)` before spawning. Do one SEQ item yourself while they run.
4. **Checkpoint** after each merge and at least once per iteration: tick checklist/SYMBOLS (`tools/sync-docs.py`), run `python3 -I tools/archive-checklist.py`, then `python3 -I tools/progress.py` from `black-rust/` (rewrites `docs/migration/PROGRESS.json`), append `docs/migration/LOG.md`, overwrite `docs/migration/STATE.md`, commit the plan files and any coordinator-owned tool files (`git add MIGRATION-AGENT.md docs/migration` plus explicit paths, one-line message; include `validation/*.md` and `validation/COMMITS.csv` when they changed), `git push origin main`.
5. Schedule the next wakeup: ~1200 s while workers run (you are notified when they finish), 60–270 s if about to merge. In DRAIN with no workers left, or in WRAP UP NOW, **pause** instead (`USAGE-CAP.md`).
6. **Stop condition:** every item is `[x]`, `[-]`, or `[!]`/waiting on `docs/migration/HUMAN.md`. Then write a final summary in `docs/migration/STATE.md` and stop the loop.

**Worker prompt:** `docs/migration/WORKER-PROMPT.md` (fill the `<>`; keep it self-contained, workers read only the files it names).

**Commits.** Workers stage + validate (gate green); the coordinator is the only one who commits, merges, and pushes. It commits at every checkpoint and pushes `origin main` each time, so the remote always has a backup. **Every coordinator commit subject is one line ending with ` (XX% complete)`**, where `XX` is the integer printed by `python3 -I tools/progress.py --percent` (run from `black-rust/` at commit time; human request 2026-10-11, computed instead of estimated since 2026-10-13). The number is measured, not monotonic: a re-baseline of `docs/migration/SCOPE.tsv` or a reverted merge may lower it; say so in `LOG.md` when it does. Examples: `B6: 40 move handlers verified (31% complete)`, `Checkpoint: wave 4 merged, STATE/LOG updated (31% complete)`. Applies to every commit the coordinator makes (merges, checkpoints, reverts); workers don't commit, so it doesn't apply to them.

**Model routing** (table and rationale: `docs/migration/WORKER-PROMPT.md`). The Agent tool takes a per-call `model` (`haiku` | `sonnet` | `opus`). Opus: the coordinator, merges, SEQ architecture, harness/hash review, spec jobs Sonnet quits early on. Sonnet: default worker (flow code, harness jobs, screens, tooling). Haiku: leaf functions from a packet with an existing harness. The four rules that make cheap models safe: (1) escalate Haiku → Sonnet after ~3 failed diff iterations, Sonnet → `[!]` after two; (2) **hands off the tests** — porting workers never touch `tools/rom-diff/*`, `tools/ui-diff/*`, stubs, `check-*.sh` or recorded hashes/scores, and the coordinator rejects branches that do; (3) the gate must pass and the coordinator re-runs the item's check on merged `main`; (4) spot-audit 2–3 ports per ~20 Haiku merges against their pseudo-C.

**UI screens (Phase G/U) and tokens.** Screens are rewrites (D3) checked by `tools/ui-diff/` (`black-rust/tools/ui-diff/README.md`): both sides are scripted and headless — `capture_ref.py` runs the original in the DeSmuME core (frame-exact input script, savestates), `capture_port.py` runs the port, `compare.py` scores them locally (no LLM, no human). UI worker loop: edit → `capture_port.py <case>` → `compare.py --case <case>` → read the JSON. It may open a failure PNG (`ui-ref/report/<case>_<frame>.png`, reference | port | heat-map) **at most 3 times per item**; then it marks the item `[x] pending sign-off` with the best score and the human decides in `HUMAN.md`. Pass = score ≥ 0.70 per screen and frame. The validator's V6 layer and the gate's `compare.py --all --regress` only compare numbers against the last `PROGRESS.json`, never images. Without captured frames the UI checks skip silently.

**Definition of done (every item):** gate green; ported functions have `/// C:` comments; verified ones have a differential test (Rust test + `tools/rom-diff` mirror with matching hash) or hand-traced values; checklist/SYMBOLS synced; deviations documented. **Phase U items additionally:** cases in `ui-ref/manifest.json` for the screen, all ≥ 0.70 against the scripted reference captures (or `[x] pending sign-off` with the best score); animations as multi-frame cases.

---

## 5b. Independent validation (VALIDATION-AGENT; added 2026-10-12)

A second, read-only agent re-tests what lands on `main`: re-runs the original code (Unicorn) and the Rust port on every commit batch with widened inputs, audits harness/spec edits, reads pseudo-C against the Rust for classic port errors, re-scores the UI cases (V6, numbers only), and files findings — the per-function harness has blind spots and the porter runs its own check. Its prompt is self-contained: `$HOME/workplace/black/VALIDATION-AGENT.md`. Baseline in `validation/COMMITS.csv`; earlier commits are not reviewed.

**Separation of duties.** The validator never edits source, never commits, never touches the plan files. It writes only `validation/COMMITS.csv`, `validation/VALIDATION.md` (append), `validation/RUNS.md` (append) and `/tmp/blk/{wt,target,out}-validate`. The coordinator is the only agent that fixes code and the only one that commits the `validation/` files (checkpoint, step 4), so there is nothing to merge.

**Launch (coordinator).** After a merge batch is pushed (rule 6) and whenever `[TODO]`/unreviewed commits exist, if no validator is running and the usage mode is RUN:

```
Agent({subagent_type: "general-purpose", model: "sonnet", run_in_background: true, description: "validator run",
       prompt: "Read $HOME/workplace/black/VALIDATION-AGENT.md fully and run exactly one validator run (its section 4). Batch hint: commits <oldest>..<tip> on origin/main (<one line on what landed>). Report in the format of its section 6."})
```

No `isolation: "worktree"` (it makes its own detached worktree at the exact tip). It counts as one worker slot (rule 7); one at a time. Cost: ~11 min and ~150k Sonnet tokens for 2 code commits / 15 areas; Opus for big batches. Don't launch it on docs-only pushes (it would mark them `SKIPPED`). In DRAIN/WRAP UP modes don't launch; a running one is left to finish (a killed run leaves `TODO` rows and re-runs next time).

**Import (coordinator, iteration step 2).** For each `- [TODO] V-NNNN — severity — area — fn — claim — commits — repro — fix notes — run` line in `validation/VALIDATION.md`: `high` → a fix job `[ ] V-NNNN — PAR — <area>: <claim> — deps: none — notes: repro + fix notes` under the owning phase in `docs/migration/checklist/<P>.md` (highest priority there; the worker reproduces on current `main` first, runs the repro, the area check and dependent areas before reporting; a weakened test is restored, never the other way round). `medium` → a harness job or an `OPEN.md` note, plus a checklist line if it needs code. `low` → fold into the next harness job, or fix it yourself in a checkpoint if it is a one-liner. Not a bug (known deviation in `OPEN.md`, intended, or misread) → `[REJECTED] — <reason>`. Then replace that line's `[TODO]` with `[IMPORTED]` / `[REJECTED] — reason`, touching nothing else (the validator only appends; import only while no validator is running); commit the ledger at the next checkpoint.

**Validator feedback.** Its final message has a "Harness notes for the coordinator" line; recurring items go into `VALIDATION-AGENT.md` section 8 "Known pitfalls" (coordinator-owned edit, committed with the ledger).
