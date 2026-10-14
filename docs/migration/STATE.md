# Current state (overwritten by the coordinator at every checkpoint; ≤ 8 KB)

- **Last updated:** 2026-10-14 ~15:10Z — session 6 PAUSED at the usage cap (RED 94%). Merged since the 12:25Z resume: F4b-spec4 + F8f-spec2, P7b, F8g, F8h (F8 done), A10d (A10 done), V-0032, B18, S5b, F13, B19, F5b (F5 done); A7 reconciled. All verified on main (gate, areas, 4-battle Unicorn diff).
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

_2026-10-14T15:08:36Z, commit 03a7bcb, `python3 -I tools/progress.py`._
<!-- progress-table:end -->

- **Milestones so far:** battle handlers complete; btl_server_flow 644; battle_server.s 207/207; btl_client 228/231; battle AI ai1 161/161; B16 first full headless battle through the ports. Pokémon: poke_tool/create/personal/growth/waza/evolution/icon. Storage: party/bag/box. Field: map matrix/fieldmap, player 304/306, fldmmdl 253/253 + acmd 242/242, trainer_eye 38/38, encounter 151/151, eventdata 184/185; script VM + ~700 scrcmd handlers. Text: strbuf/wordset/layout/speed. Assets A0–A12 mostly; Bevy shell G1/G2/G4/G5. Harness: Z19/Z20/Z22/Z23, check-force/symbols/docsize, progress.py, ui-diff with scripted DeSmuME references.
- **Running:** nothing. B18b paused with staged work in its worktree (see checklist/B.md; possible leftover mutant).
- **Next on resume:** B18b (resume, check for a leftover mutant first); validator on 7829fcb..tip; B19b (events.spec pass-through args); then F2b, F3b, F6b, F11, F14, U0b, G-phase (cap 3 workers incl. validator).
- **Blocked:** none. B16 flake: B17 fixed `ro_snap` keying; 3/3 full black-battle suites green on main after merge (root cause not proven, OPEN.md).
- **Usage guard:** PAUSED (RED 94%) until 2026-10-14 17:25Z — resume cron 9ed16f89 ( session-only: if the `claude` process exits, re-paste the loop prompt from MIGRATION-AGENT.md section 0).
- **Disk:** 39 GB free (the human cleared space 2026-10-14); worker cap 3 (human, 2026-10-14: CPU).
- **RE tooling:** Ghidra headless + rizin + rea (MCP) via `tools/re/` (`ENV.md` "RE workflow"); `tools/doctor.sh` green on macOS. Sets battle, field, bag (ov70), party-summary (ov90/91/131/134) decompiled under /tmp/blk/re/.
