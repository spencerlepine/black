# Current state (overwritten by the coordinator at every checkpoint; ≤ 8 KB)

- **Last updated:** 2026-10-14 ~12:35Z — session 6 resumed at 12:25Z after the usage pause (GREEN, 5h window 0%, weekly 26%). Doctor green, main clean at 7829fcb.
- **Overall:** see the table below (`python3 -I tools/progress.py`, full counts in `PROGRESS.json`; the commit-subject percentage is `--percent`). 37% → 36% on 2026-10-14 is the A10 `[~]` reset (0.5 item credit), not lost work.

<!-- progress-table:start -->
| bucket | done | of | ratio | weight | contributes |
| --- | ---: | ---: | ---: | ---: | ---: |
| logic (ported fns / in-scope fns per SCOPE.tsv) | 5584 | 10087.4 | 55.4% | 0.560 | 31.0% |
| phase A (items) | 13.5 | 16 | 84.4% | 0.075 | 6.3% |
| phase G (items) | 4 | 9 | 44.4% | 0.050 | 2.2% |
| phase U (items) | 0.5 | 23 | 2.2% | 0.190 | 0.4% |
| phases H+R (items) | 0 | 8 | 0.0% | 0.125 | 0.0% |
| **overall** | | | | | **40%** |

Ported by crate: black-battle 2125, black-field 1869, black-script 1011, black-storage 345, black-pokemon 156, black-text 61, black-core 26.
UI cases: U1-title 0.07, U1-title-anim 0.12.

_2026-10-14T13:01:52Z, commit 34bdde9, `python3 -I tools/progress.py`._
<!-- progress-table:end -->

- **Milestones so far:** battle handlers complete; btl_server_flow 644; battle_server.s 207/207; btl_client 228/231; battle AI ai1 161/161; B16 first full headless battle through the ports. Pokémon: poke_tool/create/personal/growth/waza/evolution/icon. Storage: party/bag/box. Field: map matrix/fieldmap, player 304/306, fldmmdl 253/253 + acmd 242/242, trainer_eye 38/38, encounter 151/151, eventdata 184/185; script VM + ~700 scrcmd handlers. Text: strbuf/wordset/layout/speed. Assets A0–A12 mostly; Bevy shell G1/G2/G4/G5. Harness: Z19/Z20/Z22/Z23, check-force/symbols/docsize, progress.py, ui-diff with scripted DeSmuME references.
- **Running (since ~12:30Z):** validator (Opus) on 4ea6a37..7829fcb; B18 (Opus, worktree-agent-a6c560338857d4623: bare-`P` harness rule + call sites, AI-mask flow gaps); F8g (Sonnet, worktree-agent-a1329b24663ddeded: scrcmd coverage audit); S5b (Opus, worktree-agent-a142c72c690aef250: remaining save blocks); A10d (Sonnet, worktree-agent-a7ad2312ab2ee3bf5: battle UI/menu/effect sprite converters).
- **Next:** merge as workers report; then F5b, F2b, F3b, F6b, F13 (daycare), A10d, U0b, F14, F11.
- **Blocked:** none. B16 flake: B17 fixed `ro_snap` keying; 3/3 full black-battle suites green on main after merge (root cause not proven, OPEN.md).
- **Usage guard:** RUN (GREEN 0% session / 26% weekly at 12:25Z; USAGE-CHECK cron b177a4a3). Last window: 5 agents burned ~84% in ~3 h.
- **Disk:** 39 GB free (the human cleared space 2026-10-14); 5 slots usable.
- **RE tooling:** Ghidra headless + rizin + rea (MCP) via `tools/re/` (`ENV.md` "RE workflow"); `tools/doctor.sh` green on macOS. Sets battle, field, bag (ov70), party-summary (ov90/91/131/134) decompiled under /tmp/blk/re/.
