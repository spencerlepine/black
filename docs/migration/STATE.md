# Current state (overwritten by the coordinator at every checkpoint; ≤ 8 KB)

- **Last updated:** 2026-10-13 — **Layout refactored on 2026-10-13: read `MIGRATION-AGENT.md` "Reading order". Progress is computed by `tools/progress.py`. UI items are gated by `tools/ui-diff/` (goldens requested in `HUMAN.md`).** Before that: 2026-10-13 ~19:15Z, session 4 paused for the day at the human's request (all workers merged, no worktrees, `main` = origin/main, gate + all checks green; validation findings V-0006..V-0019 fixed by hand, gate runs with `REQUIRE_ROM=1`).
- **Overall:** see the table below (`python3 -I tools/progress.py`, full counts in `PROGRESS.json`; the commit-subject percentage is `--percent`). Re-baselined 2026-10-13 from the estimated 19% to the computed figure (logic = ported addresses / in-scope labelled functions per `SCOPE.tsv`; phases A/G/U/H/R = ticked checklist items); the jump is a measurement change, not progress.

<!-- progress-table:start -->
| bucket | done | of | ratio | weight | contributes |
| --- | ---: | ---: | ---: | ---: | ---: |
| logic (ported fns / in-scope fns per SCOPE.tsv) | 4971 | 10087.4 | 49.3% | 0.560 | 27.6% |
| phase A (items) | 13.5 | 16 | 84.4% | 0.075 | 6.3% |
| phase G (items) | 4 | 9 | 44.4% | 0.050 | 2.2% |
| phase U (items) | 0.5 | 23 | 2.2% | 0.190 | 0.4% |
| phases H+R (items) | 0 | 8 | 0.0% | 0.125 | 0.0% |
| **overall** | | | | | **37%** |

Ported by crate: black-battle 2125, black-field 1712, black-script 868, black-pokemon 97, black-storage 85, black-text 61, black-core 26.
UI cases: U1-title 0.07, U1-title-anim 0.12.

_2026-10-14T01:06:53Z, commit 18ae32d, `python3 -I tools/progress.py`._
<!-- progress-table:end -->

- **Milestones so far:** battle handlers complete (abilities 242, items 229, moves 337, ov95 effects 30 + side-effect adder/remover); btl_server_flow 644; battle_server.s 207/207; btl_client 228/231; btl_adapter 45; battle_record 9; battle AI ai1 161/161; B16 first full headless battle through the ports. Pokémon: poke_tool/create/personal/growth/waza 31/evolution/icon verified. Storage: party/bag/box. Field: F0, F1 split, script VM (vm 16 + script_sys 90), scrcmd 15 areas ported (~700 cmd handlers), fieldmap 192/205, fldmmdl 253/253 + acmd 242/242, player 304/306, trainer_eye 38/38, encounter 151/151, eventdata 184/185. Text: strbuf/wordset/layout/speed. Assets A0–A12 mostly; Bevy shell G1/G2/G4/G5. Harness: deterministic CPU state (Z19), PORTED per set (Z20), check-force.py, check-symbols.py, check-docsize.py, progress.py, ui-diff (U0, port side).
- **Running:** nothing — PAUSED by the human (2026-10-13 ~19:15Z); the layout refactor landed after that, outside the loop. Everything is on `main`, verified (last full 90-area verification after the Z22/F4c/F5a/F4b-spec1 batch; later merges F4b-spec2/3, F8e, F8e-spec2, F4b-port1, Z23, F5a-spec2, B16, F8-spec5 each re-checked on their areas + gate + check-force/symbols). Hashes quoted in the checklist are stale for areas re-recorded by Z22/Z23; the authoritative ones are the RECORDED tables in the tests.
- **Next on resume:** full verification of `main`; validator run on the 706dc9d-after batch (last R-20261013-1528 tested a9c4242); then F8f / F4b-port2 / F5b / B17–B19 / harness follow-ups (nullable: selector double-mix, stray-write modelling, `bl`-as-branch) / F4b-fix MOVE_PROC_TABLES to 0x55 entries. U0's goldens are requested in `HUMAN.md`; until they exist the UI cases stay `pending sign-off`.
- **Blocked:** none.
- **Usage guard:** no cap (per-token billing, Bedrock; session 4) — the loop is stopped and the USAGE-CHECK cron deleted; resume by re-running the §0 `/loop` prompt of `MIGRATION-AGENT.md`. (Values when a cap applies: `USAGE-CAP.md`.)
- **RE tooling:** Ghidra headless + rizin + rea (MCP) all work via `tools/re/` (`ENV.md` "RE workflow"); `tools/doctor.sh` bootstraps a session. Battle and field sets fully decompiled.
- **Queued next (when a slot frees):** F8f (port the 4 F8-spec5 areas), F4b-port2, F5b, B17 (ai1 pointer rewire), B18 (`P`-arg audit), B19 (whole-battle Unicorn differential), harness job for the B10b/B10c mutation survivors, B10-wire follow-ups (BppEnv::move_max_pp → waza::max_pp). Disk: keep ≤ 4 workers when under ~20 GB free.
