# Current state (overwritten by the coordinator at every checkpoint; ≤ 8 KB)

- **Last updated:** 2026-10-14 ~12:35Z — session 6 resumed at 12:25Z after the usage pause (GREEN, 5h window 0%, weekly 26%). Doctor green, main clean at 7829fcb.
- **Overall:** see the table below (`python3 -I tools/progress.py`, full counts in `PROGRESS.json`; the commit-subject percentage is `--percent`). 37% → 36% on 2026-10-14 is the A10 `[~]` reset (0.5 item credit), not lost work.

<!-- progress-table:start -->
| bucket | done | of | ratio | weight | contributes |
| --- | ---: | ---: | ---: | ---: | ---: |
| logic (ported fns / in-scope fns per SCOPE.tsv) | 5585 | 10087.4 | 55.4% | 0.560 | 31.0% |
| phase A (items) | 15 | 16 | 93.8% | 0.075 | 7.0% |
| phase G (items) | 4 | 9 | 44.4% | 0.050 | 2.2% |
| phase U (items) | 0.5 | 23 | 2.2% | 0.190 | 0.4% |
| phases H+R (items) | 0 | 8 | 0.0% | 0.125 | 0.0% |
| **overall** | | | | | **41%** |

Ported by crate: black-battle 2125, black-field 1869, black-script 1012, black-storage 345, black-pokemon 156, black-text 61, black-core 26.
UI cases: U1-title 0.07, U1-title-anim 0.12.

_2026-10-14T13:44:18Z, commit d8f1c81, `python3 -I tools/progress.py`._
<!-- progress-table:end -->

- **Milestones so far:** battle handlers complete; btl_server_flow 644; battle_server.s 207/207; btl_client 228/231; battle AI ai1 161/161; B16 first full headless battle through the ports. Pokémon: poke_tool/create/personal/growth/waza/evolution/icon. Storage: party/bag/box. Field: map matrix/fieldmap, player 304/306, fldmmdl 253/253 + acmd 242/242, trainer_eye 38/38, encounter 151/151, eventdata 184/185; script VM + ~700 scrcmd handlers. Text: strbuf/wordset/layout/speed. Assets A0–A12 mostly; Bevy shell G1/G2/G4/G5. Harness: Z19/Z20/Z22/Z23, check-force/symbols/docsize, progress.py, ui-diff with scripted DeSmuME references.
- **Running:** S5b (Opus, worktree-agent-a142c72c690aef250: remaining save blocks), F5b (Opus, worktree-agent-a33ccd900ce8e7ce5), F13 (Opus, worktree-agent-a6d47baf4cf57655b: daycare), B18b (Opus, worktree-agent-adae199e185c4775e: ptr lint for all crates), B19 (Opus, worktree-agent-aefa3baf9274f6a52: whole-battle Unicorn differential).
- **Next:** merge as workers report; then F2b, F3b, F6b, U0b, F14, F11.
- **Blocked:** none. B16 flake: B17 fixed `ro_snap` keying; 3/3 full black-battle suites green on main after merge (root cause not proven, OPEN.md).
- **Usage guard:** RUN (GREEN 0% session / 26% weekly at 12:25Z; USAGE-CHECK cron b177a4a3). Last window: 5 agents burned ~84% in ~3 h.
- **Disk:** 39 GB free (the human cleared space 2026-10-14); 5 slots usable.
- **RE tooling:** Ghidra headless + rizin + rea (MCP) via `tools/re/` (`ENV.md` "RE workflow"); `tools/doctor.sh` green on macOS. Sets battle, field, bag (ov70), party-summary (ov90/91/131/134) decompiled under /tmp/blk/re/.
