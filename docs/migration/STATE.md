# Current state (overwritten by the coordinator at every checkpoint; ≤ 8 KB)

- **Last updated:** 2026-10-14 ~07:20Z — session 6 (macOS, Claude plan account: usage cap applies) started by the coordinator loop. Main gate green (full, target-coord) on 4ea6a37. Checklists reconciled: stale "merge pending"/duplicate `[ ]` spec lines in B.md/F.md fixed with shas (B11, B14, B16, F2a, F3a, F4a, F4c, F5a, F6a, F9 done), A10 stale `[~]` reset to `[ ]` (remaining A10d), Z11 dropped (superseded), B16 follow-ups split into B17/B18/B19.
- **Overall:** see the table below (`python3 -I tools/progress.py`, full counts in `PROGRESS.json`; the commit-subject percentage is `--percent`). 37% → 36% on 2026-10-14 is the A10 `[~]` reset (0.5 item credit), not lost work.

<!-- progress-table:start -->
| bucket | done | of | ratio | weight | contributes |
| --- | ---: | ---: | ---: | ---: | ---: |
| logic (ported fns / in-scope fns per SCOPE.tsv) | 5565 | 10087.4 | 55.2% | 0.560 | 30.9% |
| phase A (items) | 13 | 16 | 81.2% | 0.075 | 6.1% |
| phase G (items) | 4 | 9 | 44.4% | 0.050 | 2.2% |
| phase U (items) | 0.5 | 23 | 2.2% | 0.190 | 0.4% |
| phases H+R (items) | 0 | 8 | 0.0% | 0.125 | 0.0% |
| **overall** | | | | | **40%** |

Ported by crate: black-battle 2125, black-field 1869, black-script 1011, black-storage 328, black-pokemon 148, black-text 61, black-core 26.
UI cases: U1-title 0.07, U1-title-anim 0.12.

_2026-10-14T10:10:37Z, commit 38c1ac0, `python3 -I tools/progress.py`._
<!-- progress-table:end -->

- **Milestones so far:** battle handlers complete; btl_server_flow 644; battle_server.s 207/207; btl_client 228/231; battle AI ai1 161/161; B16 first full headless battle through the ports. Pokémon: poke_tool/create/personal/growth/waza/evolution/icon. Storage: party/bag/box. Field: map matrix/fieldmap, player 304/306, fldmmdl 253/253 + acmd 242/242, trainer_eye 38/38, encounter 151/151, eventdata 184/185; script VM + ~700 scrcmd handlers. Text: strbuf/wordset/layout/speed. Assets A0–A12 mostly; Bevy shell G1/G2/G4/G5. Harness: Z19/Z20/Z22/Z23, check-force/symbols/docsize, progress.py, ui-diff with scripted DeSmuME references.
- **Running:** B17 (Opus, worktree-agent-a8aa8754c8d18669b: ai1 pointer rewire + B16 flake), S4b (Sonnet, worktree-agent-a0c23aeb9862338b4: item-use checks/effects). Worktrees under `.claude/worktrees/`.
- **Next:** merge as workers report; F4b-spec4 (mvproc3c widening), F8f-spec2 (misc21b spec), P7b (hatching), B18 (after B17), S5, F5b, F2b, F3b, F6b, A10d, U0b. Usage will reach DRAIN (ORANGE) around 10:05Z at the current burn.
- **Blocked:** none. Known flake: `b16_single_battle` under the full parallel black-battle suite (OPEN.md; with B17).
- **Usage guard:** DRAINING (ORANGE 77%) since 09:52Z — no new spawns; B17, S4b may finish and merge; then pause until resume_at 12:25Z (resume_cron "25 5 14 10 *").
- **Disk:** 39 GB free (the human cleared space 2026-10-14); 5 slots usable.
- **RE tooling:** Ghidra headless + rizin + rea (MCP) via `tools/re/` (`ENV.md` "RE workflow"); `tools/doctor.sh` green on macOS. Sets battle, field, bag (ov70), party-summary (ov90/91/131/134) decompiled under /tmp/blk/re/.
