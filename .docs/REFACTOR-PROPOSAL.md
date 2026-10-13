# Refactor proposal: split the plan file, compute progress, add UI comparison testing

Standalone brief for the agent that refactors `github.com/spencerlepine/black` in place. The
migration loop is paused; `main` is at `5d252cf` ("F4b-port1 started (19% complete)"). After this
refactor the coordinator must be able to resume from the `/loop` prompt and find everything by
following the new reading order, with no human explanation. All paths below are relative to the repo
root unless noted. Every "today" number was measured on `5d252cf`.

Three changes, in dependency order: **1** (split) first, **2** (progress) second because it writes
into the new layout, **3** (UI diff) last because it adds checklist items and a gate step to the new
files. Do not port any game code during this refactor, and do not edit `black-rom/` (read-only, see
`.claude/settings.json` `deny`).

---

## 1. Split `MIGRATION-AGENT.md` (257 KB) into a short entry file plus topic files

### Why

`MIGRATION-AGENT.md` is the plan, the memory, the checklist, the research notes, the open-question
list, the session log and the human queue in one 257,094-byte file. The `/loop` prompt in its §0
tells the coordinator to "Read … fully" every iteration, and `VALIDATION-AGENT.md:9` already has to
warn the validator *not* to. Measured section sizes today (bytes):

| section | bytes | nature |
| --- | --- | --- |
| §0 Launch | 2,518 | fixed |
| §1 Goal and locked decisions (D1–D9) | 6,509 | fixed |
| §2 Current state | 8,911 | overwritten each session |
| §3 Paths, tools, environment | 17,653 | mostly fixed |
| §3b RE workflow | 9,261 | fixed |
| §4 Architecture | 3,588 | fixed |
| §5 Session protocol | 23,396 | fixed, but grows with "trap found on <date>" notes |
| §5b Validation | 4,015 | fixed |
| §6 Master checklist | **111,780** | 235 item lines; the longest single line is 2,853 chars of merge notes |
| §7 Reference notes | 21,561 | append-only |
| §8 Open questions | 20,990 | 59 bullets, append-only |
| §9 Session log | 22,497 | 85 entries, append-only |
| §10 Human queue | 3,895 | small |

The file has already lost data to its own size: the 2026-10-13 07:34Z commit "§8: re-add 9 session
notes that were silently dropped (wrong anchor)" and the 13:05Z commit "§2 Running line rewritten
(was stale since wave 4: unchecked replace)". Both are edit-anchor failures on a file too large to
re-read (`git log --oneline main | grep -E 'dropped|stale'`).

### Target layout

Keep the file name `MIGRATION-AGENT.md` (it is referenced from 15 places, listed below) but make it
the **entry point only**, ≤ 25 KB, containing today's §0, §1, §4, §5 (trimmed), §5b, and a new
"Reading order" table. Move everything else under `docs/migration/`:

| new file | from | size cap | who reads it, when |
| --- | --- | --- | --- |
| `MIGRATION-AGENT.md` | §0, §1, §4, §5, §5b | 25 KB | coordinator every iteration; workers read §1, §4, §5 "Rules" + "Definition of done" |
| `docs/migration/STATE.md` | §2 | 8 KB | coordinator every iteration (read + overwrite); the validator for the "~N% done" line |
| `docs/migration/ENV.md` | §3 + §3b | 30 KB | coordinator at session start (`doctor.sh` covers most of it); workers read only the §3b part |
| `docs/migration/checklist/Z.md`, `T.md`, `A.md`, `B.md`, `P.md`, `S.md`, `F.md`, `G.md`, `U.md`, `H.md`, `R.md`, `X.md` (out of scope) | §6, one file per `### Phase` heading (`MIGRATION-AGENT.md:387` onward) | 40 KB each; **an item's done-note is capped at 400 chars**, the rest moves to `docs/migration/log/<ITEM-ID>.md` | coordinator reads only the phases it is dispatching from (step 3 of the iteration) |
| `docs/migration/NOTES.md` | §7 | 40 KB | workers, by named sub-heading only (as `VALIDATION-AGENT.md:55` already does) |
| `docs/migration/OPEN.md` | §8 | 40 KB | coordinator every iteration; workers for their area |
| `docs/migration/LOG.md` | §9 | none (append-only) | nobody reads it whole: the coordinator reads the **last 10 entries** on resume (`tail -n`) |
| `docs/migration/HUMAN.md` | §10 | 8 KB | coordinator each iteration; the human |

Today's long per-item merge notes in §6 (e.g. B5, B6, Z17 at `MIGRATION-AGENT.md:445–450`, each
over 1,500 chars) are the main bulk. Move each note body verbatim into
`docs/migration/log/<ID>.md` and leave on the checklist line: status, ID, PAR/SEQ, the one-line
description, deps, commit sha, and the verified count + area hash (e.g. "items 229/229, hash
bace48c3bb734cd4"). Hashes stay on the checklist line because `check-*.sh` and the validator look
them up.

### Mechanics the agent must preserve

- **Status markers and rules 1–3 of §5** (`[ ]`/`[~]`/`[x]`/`[!]`/`[-]`, "only the coordinator edits
  the checklist", "mark `[~]` before starting") now apply to `docs/migration/checklist/*.md`.
- **`tools/sync-docs.py`** (`black-rust/tools/sync-docs.py`) ticks `black-rust/checklists/*.md`
  (per-function ticks) and `SYMBOLS.md`; it does not touch `MIGRATION-AGENT.md`, so it needs no
  change. `tools/mkchecklist.py` likewise.
- **Commit rule** (§5 "Commits"): the coordinator commits with explicit `git add` paths. Replace
  `git add MIGRATION-AGENT.md` with `git add MIGRATION-AGENT.md docs/migration` wherever it appears
  (§0 "Context overrides", §5 step 4 "Checkpoint", §5 "Pause" step 1 and 4).
- **The `/loop` prompt in §0** becomes: `Read $HOME/workplace/black/MIGRATION-AGENT.md fully (it is
  short), then the files its "Reading order" table lists for an iteration, and act as the coordinator
  in §5 …` (rest unchanged).
- **Worker prompt template** (§5): the line `Read these sections of <worktree>/MIGRATION-AGENT.md
  first: §1, §3, §3b, §4, §5 (rules + definition of done), §7 notes relevant to <area>, §8` becomes
  a list of files: `MIGRATION-AGENT.md` §1/§4/§5, `docs/migration/ENV.md` ("RE workflow" section),
  `docs/migration/NOTES.md` (named sub-heading), `docs/migration/OPEN.md` (their area).
- **Section-number references** inside the moved text ("§5 rule 6", "§8", "§3b") must be rewritten
  as file links (`[§5 rule 6](../../MIGRATION-AGENT.md#…)`, `OPEN.md`, `ENV.md`). Grep for `§` after
  the move; there are hundreds.
- **External references to update** (every one of these cites a section number or the whole file):
  `VALIDATION-AGENT.md:3,9,55,108,418`; `README.md:11,72,183,188`;
  `black-rust/tools/check-force.py:8,32`; `black-rust/tools/re/gensigs.py:14`;
  `black-rust/tools/re/packet.py:2`; `black-rust/tools/re/queue.py:2`;
  `black-rust/tools/rom-diff/genstub.py:2`; `black-rust/tools/rom-diff/genspec.py:2`;
  `black-rust/tools/doctor.sh:28,30`; `black-rust/crates/black-game/Cargo.toml:14`;
  `validation/VALIDATION.md:3`; `validation/RUNS.md:11–12`.
- **Size guard.** Add `black-rust/tools/check-docsize.py`: exit 1 if `MIGRATION-AGENT.md` > 25 KB,
  `docs/migration/STATE.md` or `HUMAN.md` > 8 KB, or any other `docs/migration/**/*.md` except
  `LOG.md` and `log/*.md` > 40 KB. Call it from `black-rust/tools/gate.sh` next to
  `check-symbols.py` (`gate.sh:32`). This is what stops the file from regrowing.
- **Trim §5 while moving it**: the dated "Trap (found 2026-10-13): …" and "found 2026-10-12" notes
  embedded in rules 4, 6 and 10 are lessons, not rules. Keep the rule in one sentence; move the
  story to `docs/migration/NOTES.md` under "Lessons (coordinator)".

### Acceptance

`wc -c MIGRATION-AGENT.md` ≤ 25,000; `grep -c '§' MIGRATION-AGENT.md docs/migration/*.md` shows no
dangling section references (every `§n` either stays inside the file that defines it or is a link);
`tools/gate.sh` green with the new `check-docsize.py` step; `git grep -n 'MIGRATION-AGENT.md §'`
returns only references to sections that still exist (§0, §1, §4, §5, §5b).

---

## 2. Compute the completion percentage instead of estimating it

### Why

Every coordinator commit subject ends with `(XX% complete)` (§5 "Commits", human request
2026-10-11). The number is the coordinator's own guess, written into §2 "Overall: ~19%
(effort-weighted)", and §5 says "keep it monotonic unless re-baselined", so it can only drift
upward. Nothing checks it.

### Data that already exists

- **Numerator.** Every ported function's original address is in a `pub const PORTED: &[u32]` array
  in its Rust module (e.g. `black-rust/crates/black-storage/src/party.rs:8`,
  `black-rust/crates/black-battle/src/handlers/abilities.rs:3339`). `tools/rom-diff/svflow.py:295–316`
  (`ported()` / `ported_flow()`) and `tools/check-force.py:28–45` already parse these. Addresses are
  keyed per memory set (Z20: a battle-set address and a field-set overlay address can coincide;
  `tools/re/sets.txt` lists the two sets today: `battle main 93 94 95 96`, `field main 10 11 18 20 21`).
  Measured today (unique addresses, before set-keying): black-battle 2,125, black-field 1,476,
  black-script 842, black-pokemon 97, black-storage 85, black-text 61, black-core 26 → **4,697**.
- **Denominator.** In-scope original functions = labelled logic files from the §7 "Code size" table
  (battle core 1,888 + field 5,139 + PC box/bag 893 = 7,920; the 4,413 UI functions and the 10,252
  networking ones are out: D3 rewrite and D9) **plus** the in-scope part of the unidentified code
  from `black-rust/tools/re/T1-TRIAGE.tsv` (`scope` column: `in` = 2,180 asm fns today, plus the
  `in:` share of the `mixed(in:N/out:M)` rows ≈ 1,550). `tools/mkchecklist.py:24` shows how to count
  functions in an asm file (`(thumb|arm)_func_start`). §2's own effort table says "~8–9k fns", which
  agrees.
- **Effort weights.** §2 "Effort estimate" table, agent-hour midpoints: logic 675, assets 90, Bevy
  shell 60, screens 225, integration 150 → total 1,200. Weights: logic 0.56, A 0.075, G 0.05, U
  0.19, R 0.125.

### What to build

`black-rust/tools/progress.py` (Python 3, stdlib only, `python3 -I`), run from `black-rust/`:

1. Numerator: union of `(set, addr)` over every `pub const PORTED` (reuse the regex from
   `check-force.py`; map a crate to its set the way `svflow.py` does).
2. Denominator: `*_func_start` labels in the in-scope asm files. Put the file→scope mapping in a
   checked-in table `docs/migration/SCOPE.tsv` (`file`, `scope ∈ {logic, ui, sdk, out}`, `source`),
   generated once from `T1-TRIAGE.tsv` + the §7 table and then hand-maintained. `T1-overrides.tsv`
   is the precedent for hand labels.
3. Phase progress for A, G, U, H, R = ticked items / items in `docs/migration/checklist/<P>.md`
   (`[x]` = 1, `[~]` = 0.5, `[!]`/`[-]` excluded). For U, an item counts as `[x]` only if its UI-diff
   cases pass (part 3).
4. Overall = 0.56·(ported/in-scope) + 0.075·A + 0.05·G + 0.19·U + 0.125·(H+R).
5. Output: `docs/migration/PROGRESS.json` (all the counts, per crate and per phase) and, with
   `--percent`, the single integer for the commit subject. Print a per-crate table otherwise.

Rules to change in §5: `(XX% complete)` **is** `python3 -I tools/progress.py --percent` at commit
time; delete "keep it monotonic"; the coordinator's checkpoint step runs `progress.py` and commits
`PROGRESS.json`. Replace §2's "Overall: ~19% (effort-weighted)" sentence in `STATE.md` with the
script's table. Add to the gate: `progress.py` must run without error (it is cheap; do not make the
gate compare it with the commit message).

Expected first result: the logic bucket alone is roughly 4,697 / ~8,600 ≈ 55% ported, so the
overall figure will land around 30–35%, above the current 19%. That is fine: it is the first
measured number. Record the re-baseline in `LOG.md`.

---

## 3. UI comparison testing (Phase G/U guardrail)

### Why

Phase U (22 screens, `MIGRATION-AGENT.md` "### Phase U") has no automatic check. The plan's only
verification for UI is `MIGRATION-AGENT.md:736`: "side-by-side screenshot/timing comparison against
the original running in an emulator (manual; human sign-off)", and §10 already holds three
unanswered "look at /tmp/blk/g2/*.png" items. Blind Bevy rebuilds will drift. Goal from the human:
≥ 0.70 similarity per screen, strive higher, **minimise LLM tokens**: all image work is local
compute; an agent reads numbers, and looks at a picture only when a number is red.

### What exists to build on

- `black-game` can already take headless screenshots: `BLACK_SCREENSHOT=<png>` saves the composite
  window at frame `BLACK_SCREENSHOT_FRAME` (default 30) and exits 30 frames later
  (`black-rust/crates/black-game/src/main.rs:132–156`). `BLACK_SCRIPT="70:A,180:Start"` injects
  button presses by frame (`src/screens.rs:21–36`), `BLACK_FADE_FRAMES` fixes fade length
  (`screens.rs:52`). The screen flow ticks in `FixedUpdate` at 60 Hz (`screens.rs:74`), the layout
  is two 256×192 screens stacked, integer-scaled, letterboxed (G1, `main.rs` `integer_scale`).
- The human has melonDS and the ROM. Goldens are therefore captured by the human, once per case,
  from the emulator; the agents never run the emulator.
- D5 applies: frames rendered from the ROM are Nintendo IP. Goldens live in **`black-rust/ui-ref/`,
  gitignored like `assets/`** (add it to `black-rust/.gitignore` and to the `gate.sh:33` tracked-files
  check). Only hashes, scores and recipes are committed.

### Design: `black-rust/tools/ui-diff/`

**Cases.** `black-rust/ui-ref/manifest.json` (committed) lists cases:
`{ "id": "U6-party-open", "screen": "U6", "port": {"script": "70:Start,120:Down,150:A",
"frames": [160, 175, 190]}, "ref": {"savestate": "party.mln", "inputs": "…", "frames": [160,175,190]},
"regions": {"top": "whole", "bottom": "whole"}, "phash": {"160": "…"} }`. One case = one scripted
path and 1–N frame indices; multi-frame cases cover animations (frames at fixed offsets after the
triggering input). The `ref.inputs` text is the human's recipe, not executed by code.

**Reference capture (human, once per case, local).** `tools/ui-diff/capture_ref.sh`: the human
loads the named savestate in melonDS, performs the recipe, and the script grabs the emulator window
N times at the listed frames using whatever the host has (`xdotool`+`import`/`grim` on Linux,
`screencapture -l <windowid>` on macOS; if the installed melonDS build exposes scripting or a
frame-step hotkey, prefer that for frame-exact grabs). It crops to the two 256×192 screens (the
emulator's window scale is detected from the window size, as `integer_scale` does), writes
`ui-ref/<case>/ref_<frame>.png`, and stores each frame's perceptual hash in `manifest.json` so a
later drift of the golden set itself is detectable. Manual timing jitter is acceptable: the metric
below is tolerant, and the case can list a small frame window (`"frames": [[158,162], …]`) where the
best-matching frame counts.

**Port capture (agent, headless).** `tools/ui-diff/capture_port.py <case>` runs
`cargo run -p black-game` with `BLACK_SCRIPT`, `BLACK_FADE_FRAMES` and a new
`BLACK_SCREENSHOT_FRAMES=160,175,190` (extend `screenshot_and_exit` in `main.rs` to accept a list and
write `<path>_<frame>.png`; count frames in `FixedUpdate`, not `Update`, so the index is
deterministic), then crops the composite into top/bottom 256×192 at scale 1. Needs a window:
use `xvfb-run` on the Linux host (add it to `tools/doctor.sh` checks).

**Compare (local compute, no LLM).** `tools/ui-diff/compare.py [--case ID | --all]`, in the
existing `/tmp/blk/venv` (add `pillow`, `numpy`, `scikit-image` to `doctor.sh`'s venv setup):

1. Quantise both images to RGB555 (the DS colour depth) so Bevy's 8-bit output is not penalised.
2. Per screen, per frame: **structure** = SSIM on grayscale (scikit-image, 7×7 window);
   **colour** = 1 − mean ΔE76 in Lab over 8×8 blocks, scaled so ΔE 20 → 0; **layout** = IoU of Canny
   edge maps after a 2-px dilation (tolerates font anti-aliasing and 1-px offsets).
   `score = 0.5·structure + 0.3·colour + 0.2·layout`. Pass ≥ **0.70**. Regions in the manifest can
   override weights (e.g. a text box uses layout 0.5 / structure 0.3 / colour 0.2).
3. For multi-frame cases also report **timing**: the first frame index at which the frame hash
   changes after the trigger must match the reference within ±3 frames.
4. Write `ui-diff/report/<case>.json` (scores per screen per frame, timing, pass/fail) and, **only
   on failure**, `ui-diff/report/<case>_<frame>.png` = reference | port | heat-map, 3×256 wide. A
   passing case produces no image.

**Token policy (write into §5 and the Phase U checklist header).**

- A UI worker's loop is: edit → `capture_port.py` → `compare.py --case` → read the JSON. It may
  open a failure PNG **at most 3 times per item** (one image ≈ 1–2k tokens); after that it marks the
  item `[x] pending sign-off` with the best score, and the human decides in `HUMAN.md`.
- The validator (`VALIDATION-AGENT.md` §5 layers V1–V5) gets a **V6: UI** layer that only re-runs
  `compare.py --all` and files a finding when any case dropped by > 0.05 against the score recorded in
  the last `PROGRESS.json`. It never opens images. This is the drift guard.
- The gate (`gate.sh`) runs `compare.py --all --regress` when `ui-ref/` exists: fail if any
  previously passing case now fails or drops > 0.05; skip silently when the goldens are absent (a
  worktree without them, like `REQUIRE_ROM=0`).
- Model routing (§5 "Model routing"): screen rebuilds stay on Sonnet; `compare.py` and
  `capture_port.py` are tool code, written once by a Sonnet harness job and not edited by UI
  porting workers (same hands-off rule as `tools/rom-diff/*`).

**Checklist changes (in `docs/migration/checklist/U.md` and `G.md`).**

- New `U0 — SEQ — UI diff kit`: `tools/ui-diff/` + the `BLACK_SCREENSHOT_FRAMES` change +
  `doctor.sh`/`gate.sh` hooks + `ui-ref/manifest.json` with the first three cases (title screen G5
  message box, start menu G2, overworld placeholder), scored against goldens the human captures.
  deps: G1, G2, G5. Put the golden-capture request in `HUMAN.md` with the exact recipes.
- Every U item gains a deliverable line: "cases in `manifest.json` for this screen, all ≥ 0.70;
  animations as multi-frame cases". Definition of done (§5) gets the same sentence for Phase U.
- G2 and G5 (`[x] (pending visual check)` today, §10 lines for `/tmp/blk/g2`, `/tmp/blk/g5`) become
  the first cases, which retires those two §10 entries.

### Acceptance

`compare.py` on two identical PNGs scores 1.0; on a PNG against itself shifted by 1 px scores
> 0.9; on an all-black vs the title screen scores < 0.3 (put these three in
`tools/ui-diff/test_compare.py`). With goldens absent the gate passes and prints "ui-diff: no
goldens, skipped". With the first three cases captured, `PROGRESS.json` shows their scores.

---

## Hand-back to the coordinator

When the refactor is committed and pushed (one PR or a short series on `main`, gate green, commit
subjects ending in the `progress.py --percent` number), append one entry to
`docs/migration/LOG.md` and one line to `docs/migration/STATE.md` "Last updated" saying: "Layout
refactored on <date>: read `MIGRATION-AGENT.md` 'Reading order'. Progress is computed by
`tools/progress.py`. UI items are gated by `tools/ui-diff/` (goldens requested in `HUMAN.md`)." The
human then re-pastes the `/loop` prompt from `MIGRATION-AGENT.md` §0.
