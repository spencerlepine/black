# Current state (overwritten by the coordinator at every checkpoint; ≤ 8 KB)

- **Last updated:** 2026-10-14 ~22:30Z — session 6 resumed by the human after the third usage pause (GREEN: 5h 0%, weekly 51%). Doctor green, main clean at 40002b2.
- **Overall:** see the table below (`python3 -I tools/progress.py`, full counts in `PROGRESS.json`; the commit-subject percentage is `--percent`). 37% → 36% on 2026-10-14 is the A10 `[~]` reset (0.5 item credit), not lost work.

<!-- progress-table:start -->
| bucket | done | of | ratio | weight | contributes |
| --- | ---: | ---: | ---: | ---: | ---: |
| logic (ported fns / in-scope fns per SCOPE.tsv) | 6224 | 10087.4 | 61.7% | 0.560 | 34.6% |
| phase A (items) | 15 | 16 | 93.8% | 0.075 | 7.0% |
| phase G (items) | 4 | 9 | 44.4% | 0.050 | 2.2% |
| phase U (items) | 0.5 | 23 | 2.2% | 0.190 | 0.4% |
| phases H+R (items) | 0 | 8 | 0.0% | 0.125 | 0.0% |
| **overall** | | | | | **44%** |

Ported by crate: black-field 2261, black-battle 2125, black-script 1012, black-storage 562, black-pokemon 186, black-text 61, black-core 26.
UI cases: U1-title 0.07, U1-title-anim 0.12.

_2026-10-14T23:13:58Z, commit 9e7ca5d, `python3 -I tools/progress.py`._
<!-- progress-table:end -->

- **Milestones so far:** battle handlers complete; btl_server_flow 644; battle_server.s 207/207; btl_client 228/231; battle AI ai1 161/161; B16 first full headless battle through the ports. Pokémon: poke_tool/create/personal/growth/waza/evolution/icon. Storage: party/bag/box. Field: map matrix/fieldmap, player 304/306, fldmmdl 253/253 + acmd 242/242, trainer_eye 38/38, encounter 151/151, eventdata 184/185; script VM + ~700 scrcmd handlers. Text: strbuf/wordset/layout/speed. Assets A0–A12 mostly; Bevy shell G1/G2/G4/G5. Harness: Z19/Z20/Z22/Z23, check-force/symbols/docsize, progress.py, ui-diff with scripted DeSmuME references.
- **Running (cap 3):** F2b (Opus, worktree-agent-ade14361ab2bbfdd2: buildmodel + scenearea), F3b (resumed, worktree-agent-a858739933e9b77b9), F12 (resumed, worktree-agent-ab3c43b94fcb82115).
- **Next:** F14b, U0b, G-phase.
- **Blocked:** none. B16 flake: B17 fixed `ro_snap` keying; 3/3 full black-battle suites green on main after merge (root cause not proven, OPEN.md).
- **Usage guard:** RUN (GREEN 0% session / 51% weekly at 22:25Z; USAGE-CHECK cron ae0b0b51). Weekly budget: ~49% left for 5.5 days.
- **Disk:** 39 GB free (the human cleared space 2026-10-14); worker cap 3 (human, 2026-10-14: CPU).
- **RE tooling:** Ghidra headless + rizin + rea (MCP) via `tools/re/` (`ENV.md` "RE workflow"); `tools/doctor.sh` green on macOS. Sets battle, field, bag (ov70), party-summary (ov90/91/131/134) decompiled under /tmp/blk/re/.
