# Current state (overwritten by the coordinator at every checkpoint; ≤ 8 KB)

- **Last updated:** 2026-10-15 ~08:30Z — session 6 resumed by the one-shot cron (session 4%, weekly 74%, extra usage 73% unchanged). ui-ref states/references regenerated in the main checkout (identical phashes).
- **Overall:** see the table below (`python3 -I tools/progress.py`, full counts in `PROGRESS.json`; the commit-subject percentage is `--percent`). 37% → 36% on 2026-10-14 is the A10 `[~]` reset (0.5 item credit), not lost work.

<!-- progress-table:start -->
| bucket | done | of | ratio | weight | contributes |
| --- | ---: | ---: | ---: | ---: | ---: |
| logic (ported fns / in-scope fns per SCOPE.tsv) | 6646 | 10087.4 | 65.9% | 0.560 | 36.9% |
| phase A (items) | 15 | 16 | 93.8% | 0.075 | 7.0% |
| phase G (items) | 5 | 9 | 55.6% | 0.050 | 2.8% |
| phase U (items) | 1 | 23 | 4.3% | 0.190 | 0.8% |
| phases H+R (items) | 0 | 8 | 0.0% | 0.125 | 0.0% |
| **overall** | | | | | **48%** |

Ported by crate: black-field 2671, black-battle 2125, black-script 1012, black-storage 562, black-pokemon 186, black-text 61, black-core 38.
UI cases: G2-overworld-idle 0.30, G2-start-menu 0.29, U1-title 0.07, U1-title-anim 0.18.

_2026-10-15T09:19:31Z, commit 1338d50, `python3 -I tools/progress.py`._
<!-- progress-table:end -->

- **Milestones so far:** battle handlers complete; btl_server_flow 644; battle_server.s 207/207; btl_client 228/231; battle AI ai1 161/161; B16 first full headless battle through the ports. Pokémon: poke_tool/create/personal/growth/waza/evolution/icon. Storage: party/bag/box. Field: map matrix/fieldmap, player 304/306, fldmmdl 253/253 + acmd 242/242, trainer_eye 38/38, encounter 151/151, eventdata 184/185; script VM + ~700 scrcmd handlers. Text: strbuf/wordset/layout/speed. Assets A0–A12 mostly; Bevy shell G1/G2/G4/G5. Harness: Z19/Z20/Z22/Z23, check-force/symbols/docsize, progress.py, ui-diff with scripted DeSmuME references.
- **Running:** Z25 (Sonnet, worktree-agent-afc572e82869cf4b1: divider/sqrt in emu.py + lint), validator (Sonnet, 22b42dd..6168493). All on Sonnet: weekly is 1 point from its 75% drain.
- **Next on resume:** G-phase (G3 asset loading, G6 audio, G7 sprite/animation, G8 3D map renderer, G9 save/load), U1/U2 screens now that references exist, black-core FX/VEC port (F11+F3b need it), tail-call register audit, validator on 22b42dd..tip. Weekly is at 73%: the next window will hit the weekly 75% drain early.
- **Blocked:** none. B16 flake: B17 fixed `ro_snap` keying; 3/3 full black-battle suites green on main after merge (root cause not proven, OPEN.md).
- **Usage guard:** DRAINING (weekly ORANGE 75%) since ~09:05Z — no new spawns; Z25, G3, validator finish and merge; then pause until the weekly reset: resume_at 2026-10-20 13:05Z (resume_cron "5 6 20 10 *", local time). Session 10%, extra usage 73% (unchanged).
- **Disk:** 39 GB free (the human cleared space 2026-10-14); worker cap 3 (human, 2026-10-14: CPU).
- **RE tooling:** Ghidra headless + rizin + rea (MCP) via `tools/re/` (`ENV.md` "RE workflow"); `tools/doctor.sh` green on macOS. Sets battle, field, bag (ov70), party-summary (ov90/91/131/134) decompiled under /tmp/blk/re/.
