# Human queue (the coordinator appends; the human answers inline and the coordinator acts on it)

Moved out of `MIGRATION-AGENT.md` (its old section 10) on 2026-10-13. Format: `- [ ] YYYY-MM-DD — <item> — <what's needed> — default if unanswered: <…>`. Never block on an entry: if it has a default, proceed with the default and note that in `LOG.md`; if not, work around it. Keep this file under 8 KB: resolved `[x]` entries older than a week move to `LOG.md`.

## Open

- [ ] 2026-10-11 — G8 — overworld as 3D (glTF from NSBMD, closest to the original) vs 2D re-render — default if unanswered: **3D**, A5 goes in scope.
- [ ] 2026-10-11 — R4 — where to host the wasm build privately — default: **localhost only**.
- [ ] 2026-10-12 — A10c — look at /tmp/blk/a10c/contact.png (item icons, type/ball icons, UI sprite bundle; some UI cells may be magenta = wrong palette) and /tmp/blk/a10b/contact.png + contact_overworld.png (trainers, overworld) — default: continue.
- [ ] 2026-10-11 — U-phase — visual sign-off per screen: since 2026-10-13 this is `tools/ui-diff` scores against your goldens (below); an item at `[x] pending sign-off` lists its best score here and you decide — default: continue, mark `[x] pending sign-off`, list the screen here.
- [ ] 2026-10-11 — R2 — full playthrough by a human — no default; do R3/R5 and fixes from automated tests meanwhile.
- [ ] 2026-10-13 — F8-spec5 scope — WFBC commands 450..457 (Black City / White Forest; fed by Entralink but present offline) are specced in `scrcmd_misc21b`: port them or treat as X1 and drop? Also who owns overlay 48 (cmd 382 handler)? — default: port 450..457 (they are offline-visible), park 382 under F10.
- [ ] 2026-10-13 — **U0 goldens** — capture the first three UI reference cases from melonDS into `black-rust/ui-ref/` (gitignored, D5) with `tools/ui-diff/capture_ref.sh <case>`; the script lists the recipe, grabs the emulator window at the listed frames and writes `ui-ref/<case>/ref_<frame>.png` + the perceptual hashes into `ui-ref/manifest.json`. Recipes (the port side is scripted identically; see `black-rust/tools/ui-diff/README.md`):
  - `G5-title-msgbox`: savestate `boot.mln` at the steady title screen, no input, frames 30 / 60 / 90 after the load (title + message box with the real font; the port shows its G5 message box on the Title placeholder at its frames 90 / 120 / 150).
  - `G2-overworld-idle`: `overworld.mln` (any outdoor map, player idle), no input, frames 60 / 100 (idle overworld, both screens; the baseline the placeholders are measured against until U2 lands; port frames 190 / 230 after `A` at 90).
  - `G2-start-menu`: `overworld.mln`, press **Start** at frame 70, frames 98–102 / 113–117 / 128–132 (windows: menu open + first cursor state; the port presses Start at 200 and uses 228–232 / 243–247 / 258–262).
  - Timing jitter is fine (the windows absorb ±2 frames; `MELONDS_FRAMESTEP=1` makes the script prompt per frame if your build has a frame-step hotkey). — default: none; U0 completes with the port side only and its cases stay `pending sign-off` until the goldens exist.

## Resolved (recent)

- [x] 2026-10-11 — HOST (resolved: Ghidra 12.1.4 in ~/tools, brew openjdk@21 symlinked at the mise JDK path; doctor.sh green on macOS) — this session runs on macOS, not the Linux host `ENV.md` documents. — default: run only Ghidra-free items and skip decompile-dependent ones.
- [x] 2026-10-13 — GIT PUSH — pushes to github.com/spencerlepine/black stopped working ~05:50Z (askpass hang; the `gh-push` tmux loop stuck the same way). **Resolved by ~17:30Z: origin/main is level with local main again (the background push job works).**
- [x] 2026-10-12 — A9 — SIGNED OFF by the human 2026-10-12 ("sounds are working great"; OGG default = external tool stands).
- [x] 2026-10-11 — A9 — by-ear check of rendered music against the original (signed off 2026-10-12).
- [x] 2026-10-12 — A10/G1 — A10 contact sheet SIGNED OFF ("looks perfect"); G1 screenshot SIGNED OFF 2026-10-12.
- [x] 2026-10-12 — A5 — all 3 glTF files pass the Khronos glTF Validator and "work great" (human 2026-10-12).
- [x] 2026-10-12 — G2 — (superseded 2026-10-13: the `/tmp/blk/g2/*.png` look is replaced by the U0 cases `G2-start-menu` / `G2-overworld-idle` scored by `tools/ui-diff`; see the U0 goldens entry above) — default: continue.
- [x] 2026-10-12 — G5 — (superseded 2026-10-13: `/tmp/blk/g5/*.png` replaced by the U0 case `G5-title-msgbox`) — default: continue.
