# Human queue (the coordinator appends; the human answers inline and the coordinator acts on it)

Moved out of `MIGRATION-AGENT.md` (its old section 10) on 2026-10-13. Format: `- [ ] YYYY-MM-DD — <item> — <what's needed> — default if unanswered: <…>`. Never block on an entry: if it has a default, proceed with the default and note that in `LOG.md`; if not, work around it. Keep this file under 8 KB: resolved `[x]` entries older than a week move to `LOG.md`.

## Open

- [ ] 2026-10-11 — G8 — overworld as 3D (glTF from NSBMD, closest to the original) vs 2D re-render — default if unanswered: **3D**, A5 goes in scope.
- [ ] 2026-10-11 — R4 — where to host the wasm build privately — default: **localhost only**.
- [ ] 2026-10-12 — A10c — look at /tmp/blk/a10c/contact.png (item icons, type/ball icons, UI sprite bundle; some UI cells may be magenta = wrong palette) and /tmp/blk/a10b/contact.png + contact_overworld.png (trainers, overworld) — default: continue.
- [ ] 2026-10-11 — U-phase — visual sign-off per screen: since 2026-10-13 this is `tools/ui-diff` scores against your goldens (below); an item at `[x] pending sign-off` lists its best score here and you decide — default: continue, mark `[x] pending sign-off`, list the screen here.
- [ ] 2026-10-11 — R2 — full playthrough by a human — no default; do R3/R5 and fixes from automated tests meanwhile.
- [ ] 2026-10-13 — F8-spec5 scope — WFBC commands 450..457 (Black City / White Forest; fed by Entralink but present offline) are specced in `scrcmd_misc21b`: port them or treat as X1 and drop? Also who owns overlay 48 (cmd 382 handler)? — default: port 450..457 (they are offline-visible), park 382 under F10.
- [x] 2026-10-13 — U0 goldens — (resolved the same evening: no human capture needed. The human asked for a scalable solution; melonDS 1.1 has no scripting, so `capture_ref.py` now drives the DeSmuME core headless via py-desmume with frame-exact input scripts and savestates; the title cases are captured and scored automatically. Remaining human role for UI: sign-off decisions on `pending sign-off` items only.)

## Resolved (recent)

- [x] 2026-10-11 — HOST (resolved: Ghidra 12.1.4 in ~/tools, brew openjdk@21 symlinked at the mise JDK path; doctor.sh green on macOS) — this session runs on macOS, not the Linux host `ENV.md` documents. — default: run only Ghidra-free items and skip decompile-dependent ones.
- [x] 2026-10-13 — GIT PUSH — pushes to github.com/spencerlepine/black stopped working ~05:50Z (askpass hang; the `gh-push` tmux loop stuck the same way). **Resolved by ~17:30Z: origin/main is level with local main again (the background push job works).**
- [x] 2026-10-12 — A9 — SIGNED OFF by the human 2026-10-12 ("sounds are working great"; OGG default = external tool stands).
- [x] 2026-10-11 — A9 — by-ear check of rendered music against the original (signed off 2026-10-12).
- [x] 2026-10-12 — A10/G1 — A10 contact sheet SIGNED OFF ("looks perfect"); G1 screenshot SIGNED OFF 2026-10-12.
- [x] 2026-10-12 — A5 — all 3 glTF files pass the Khronos glTF Validator and "work great" (human 2026-10-12).
- [x] 2026-10-12 — G2 — (superseded 2026-10-13: the `/tmp/blk/g2/*.png` look is replaced by the ui-diff cases `G2-start-menu` / `G2-overworld-idle`, scored automatically once U0b's overworld savestate exists) — default: continue.
- [x] 2026-10-12 — G5 — (superseded 2026-10-13: `/tmp/blk/g5/*.png` replaced by the ui-diff title cases `U1-title` / `U1-title-anim`, scored automatically) — default: continue.
