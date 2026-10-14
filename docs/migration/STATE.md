# Current state (overwritten by the coordinator at every checkpoint; ≤ 8 KB)

- **Last updated:** 2026-10-14 ~17:30Z — session 6 resumed at 17:25Z after the second usage pause (GREEN: 5h 2%, weekly 38%). Doctor green, main clean at e5f4db7.
- **Overall:** see the table below (`python3 -I tools/progress.py`, full counts in `PROGRESS.json`; the commit-subject percentage is `--percent`). 37% → 36% on 2026-10-14 is the A10 `[~]` reset (0.5 item credit), not lost work.

<!-- progress-table:start -->
| bucket | done | of | ratio | weight | contributes |
| --- | ---: | ---: | ---: | ---: | ---: |
| logic (ported fns / in-scope fns per SCOPE.tsv) | 5915 | 10087.4 | 58.6% | 0.560 | 32.8% |
| phase A (items) | 15 | 16 | 93.8% | 0.075 | 7.0% |
| phase G (items) | 4 | 9 | 44.4% | 0.050 | 2.2% |
| phase U (items) | 0.5 | 23 | 2.2% | 0.190 | 0.4% |
| phases H+R (items) | 0 | 8 | 0.0% | 0.125 | 0.0% |
| **overall** | | | | | **43%** |

Ported by crate: black-battle 2125, black-field 1952, black-script 1012, black-storage 562, black-pokemon 186, black-text 61, black-core 26.
UI cases: U1-title 0.07, U1-title-anim 0.12.

_2026-10-14T18:06:49Z, commit 4eef755, `python3 -I tools/progress.py`._
<!-- progress-table:end -->

- **Milestones so far:** battle handlers complete; btl_server_flow 644; battle_server.s 207/207; btl_client 228/231; battle AI ai1 161/161; B16 first full headless battle through the ports. Pokémon: poke_tool/create/personal/growth/waza/evolution/icon. Storage: party/bag/box. Field: map matrix/fieldmap, player 304/306, fldmmdl 253/253 + acmd 242/242, trainer_eye 38/38, encounter 151/151, eventdata 184/185; script VM + ~700 scrcmd handlers. Text: strbuf/wordset/layout/speed. Assets A0–A12 mostly; Bevy shell G1/G2/G4/G5. Harness: Z19/Z20/Z22/Z23, check-force/symbols/docsize, progress.py, ui-diff with scripted DeSmuME references.
- **Running (cap 3):** B18b (Opus, worktree-agent-adae199e185c4775e: rebased, fixing 14 lint sites from S5b specs), F6b (Opus, worktree-agent-ae735f1311f730ad5: event_mapchange.c), F14 (Opus, worktree-agent-ac0ada1b9ab597a2b: field_task + proclink).
- **Next:** B19b after B18b (both touch battle specs/testkit); F2b, F3b, F11, F14, U0b, G-phase.
- **Blocked:** none. B16 flake: B17 fixed `ro_snap` keying; 3/3 full black-battle suites green on main after merge (root cause not proven, OPEN.md).
- **Usage guard:** RUN (GREEN 2% session / 38% weekly at 17:25Z; USAGE-CHECK cron ba633253).
- **Disk:** 39 GB free (the human cleared space 2026-10-14); worker cap 3 (human, 2026-10-14: CPU).
- **RE tooling:** Ghidra headless + rizin + rea (MCP) via `tools/re/` (`ENV.md` "RE workflow"); `tools/doctor.sh` green on macOS. Sets battle, field, bag (ov70), party-summary (ov90/91/131/134) decompiled under /tmp/blk/re/.
