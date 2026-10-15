# Current state (overwritten by the coordinator at every checkpoint; ≤ 8 KB)

- **Last updated:** 2026-10-15 ~05:45Z — session 6 PAUSED (drain finished at session 73%; weekly 73%), fifth pause. Merged since the 03:25Z resume: F14b, F2b (F2 done), F3b (F3 done), U0b; validator R-20261015-0326 clean. Main green at 57b6d15+46e41b2.
- **Overall:** see the table below (`python3 -I tools/progress.py`, full counts in `PROGRESS.json`; the commit-subject percentage is `--percent`). 37% → 36% on 2026-10-14 is the A10 `[~]` reset (0.5 item credit), not lost work.

<!-- progress-table:start -->
| bucket | done | of | ratio | weight | contributes |
| --- | ---: | ---: | ---: | ---: | ---: |
| logic (ported fns / in-scope fns per SCOPE.tsv) | 6634 | 10087.4 | 65.8% | 0.560 | 36.8% |
| phase A (items) | 15 | 16 | 93.8% | 0.075 | 7.0% |
| phase G (items) | 4 | 9 | 44.4% | 0.050 | 2.2% |
| phase U (items) | 1 | 23 | 4.3% | 0.190 | 0.8% |
| phases H+R (items) | 0 | 8 | 0.0% | 0.125 | 0.0% |
| **overall** | | | | | **47%** |

Ported by crate: black-field 2671, black-battle 2125, black-script 1012, black-storage 562, black-pokemon 186, black-text 61, black-core 26.
UI cases: U1-title 0.07, U1-title-anim 0.12.

_2026-10-15T05:28:20Z, commit 46e41b2, `python3 -I tools/progress.py`._
<!-- progress-table:end -->

- **Milestones so far:** battle handlers complete; btl_server_flow 644; battle_server.s 207/207; btl_client 228/231; battle AI ai1 161/161; B16 first full headless battle through the ports. Pokémon: poke_tool/create/personal/growth/waza/evolution/icon. Storage: party/bag/box. Field: map matrix/fieldmap, player 304/306, fldmmdl 253/253 + acmd 242/242, trainer_eye 38/38, encounter 151/151, eventdata 184/185; script VM + ~700 scrcmd handlers. Text: strbuf/wordset/layout/speed. Assets A0–A12 mostly; Bevy shell G1/G2/G4/G5. Harness: Z19/Z20/Z22/Z23, check-force/symbols/docsize, progress.py, ui-diff with scripted DeSmuME references.
- **Running:** nothing (the coordinator regenerates the ui-ref savestates/references in the main checkout in the background; gitignored).
- **Next on resume:** G-phase (G3 asset loading, G6 audio, G7 sprite/animation, G8 3D map renderer, G9 save/load), U1/U2 screens now that references exist, black-core FX/VEC port (F11+F3b need it), tail-call register audit, validator on 22b42dd..tip. Weekly is at 73%: the next window will hit the weekly 75% drain early.
- **Blocked:** none. B16 flake: B17 fixed `ro_snap` keying; 3/3 full black-battle suites green on main after merge (root cause not proven, OPEN.md).
- **Usage guard:** PAUSED (session drain done at 73%) until 2026-10-15 08:25Z — resume cron 4c5ab36b ( session-only: if the `claude` process exits, re-paste the loop prompt from MIGRATION-AGENT.md section 0). Weekly 73%, extra usage 73% (unchanged).
- **Disk:** 39 GB free (the human cleared space 2026-10-14); worker cap 3 (human, 2026-10-14: CPU).
- **RE tooling:** Ghidra headless + rizin + rea (MCP) via `tools/re/` (`ENV.md` "RE workflow"); `tools/doctor.sh` green on macOS. Sets battle, field, bag (ov70), party-summary (ov90/91/131/134) decompiled under /tmp/blk/re/.
