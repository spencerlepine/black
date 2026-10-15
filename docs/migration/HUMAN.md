# Human queue (the coordinator appends; the human answers inline and the coordinator acts on it)

Moved out of `MIGRATION-AGENT.md` (its old section 10) on 2026-10-13. Format: `- [ ] YYYY-MM-DD — <item> — <what's needed> — default if unanswered: <…>`. Never block on an entry: if it has a default, proceed with the default and note that in `LOG.md`; if not, work around it. Keep this file under 8 KB: resolved `[x]` entries older than a week move to `LOG.md`.

## Open

- [x] 2026-10-11 — G8 — **DECIDED by the human 2026-10-13: 3D** (glTF from NSBMD, match the original's pattern); A5 in scope. 2D re-render dropped.
- [x] 2026-10-11 — R4 — **DECIDED by the human 2026-10-13: localhost only, always** (no paid hosting). R4 = local serving instructions only.
- [x] 2026-10-12 — A10c — closed 2026-10-13 (human): no manual look; sprite correctness is covered by `tools/ui-diff` scores once screens use the sprites (U-phase). Magenta cells (wrong palette) will show up as red scores there.
- [ ] 2026-10-11 — U-phase — the only standing human role for UI: an item at `[x] pending sign-off` (best `tools/ui-diff` score < 0.70 after the worker's 3 image looks) is listed here with its score and you accept or reject it — default: continue, keep it `pending sign-off`.
- [ ] 2026-10-11 — R2 — full playthrough by a human — no default; do R3/R5 and fixes from automated tests meanwhile.
- [x] 2026-10-13 — F8-spec5 scope — closed 2026-10-13 with the default (human): port WFBC commands 450..457 (offline-visible), park cmd 382 (overlay 48) under F10.
- [x] 2026-10-13 — U0 goldens — (resolved the same evening: no human capture needed. The human asked for a scalable solution; melonDS 1.1 has no scripting, so `capture_ref.py` now drives the DeSmuME core headless via py-desmume with frame-exact input scripts and savestates; the title cases are captured and scored automatically. Remaining human role for UI: sign-off decisions on `pending sign-off` items only.)

- [x] 2026-10-14 — WEEKLY BUDGET — **DECIDED by the human 2026-10-14: keep full pace**; the USAGE-CAP guard (weekly ORANGE/RED → drain/pause) stops the work near the limit. each 5-hour window with 3 Opus workers uses ~10–12% of the weekly limit (54% at 22:55Z, resets 2026-10-20 13:00Z); at full pace the weekly limit hits 75% (DRAIN) in ~1 day and work then stops until the reset. Options: keep full pace, or cap each 5-hour window at ~50% usage so the weekly budget lasts to the reset — default if unanswered: keep full pace (USAGE-CAP rules unchanged).

- [ ] 2026-10-15 — EXTRA USAGE SPEND — paid "extra usage" rose 49% → 73% of $100 (~$24) between the 2026-10-14 20:15Z and 2026-10-15 03:25Z readings, most likely while 3 Opus workers overshot the 5-hour window (last reading RED 94%) between the 20-minute checks. The usage guard only watches the plan windows. Coordinator safeguard from now on: any increase in the extra-usage % → DRAIN immediately (no new spawns, pause once workers finish). Options: (a) keep that safeguard, (b) also lower ORANGE/RED to 65%/80% to leave more headroom, (c) turn extra usage off in your account settings so overshoot just stops work — default if unanswered: (a) + (b).

## Resolved (recent)

- [x] 2026-10-11 — HOST (resolved: Ghidra 12.1.4 in ~/tools, brew openjdk@21 symlinked at the mise JDK path; doctor.sh green on macOS) — this session runs on macOS, not the Linux host `ENV.md` documents. — default: run only Ghidra-free items and skip decompile-dependent ones.
- [x] 2026-10-13 — GIT PUSH — pushes to github.com/spencerlepine/black stopped working ~05:50Z (askpass hang; the `gh-push` tmux loop stuck the same way). **Resolved by ~17:30Z: origin/main is level with local main again (the background push job works).**
- [x] 2026-10-12 — A9 — SIGNED OFF by the human 2026-10-12 ("sounds are working great"; OGG default = external tool stands).
- [x] 2026-10-11 — A9 — by-ear check of rendered music against the original (signed off 2026-10-12).
- [x] 2026-10-12 — A10/G1 — A10 contact sheet SIGNED OFF ("looks perfect"); G1 screenshot SIGNED OFF 2026-10-12.
- [x] 2026-10-12 — A5 — all 3 glTF files pass the Khronos glTF Validator and "work great" (human 2026-10-12).
- [x] 2026-10-12 — G2 — (superseded 2026-10-13: the `/tmp/blk/g2/*.png` look is replaced by the ui-diff cases `G2-start-menu` / `G2-overworld-idle`, scored automatically once U0b's overworld savestate exists) — default: continue.
- [x] 2026-10-12 — G5 — (superseded 2026-10-13: `/tmp/blk/g5/*.png` replaced by the ui-diff title cases `U1-title` / `U1-title-anim`, scored automatically) — default: continue.
