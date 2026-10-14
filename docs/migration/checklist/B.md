# Phase B — Battle logic (`black-battle`) — priority 1

Part of the master checklist (moved out of `MIGRATION-AGENT.md` section 6 on 2026-10-13). Status markers and rules: [§5](../../MIGRATION-AGENT.md). Format: `- [status] ID — TAG — description — deps: … — notes`. A done-note longer than ~400 chars is cut here and kept in full in `../log/<ID>.md`. Finished items are moved out by `tools/archive-checklist.py` (stub in `DONE.md`, full line in `archive/`).

- [~] B17 (worktree-agent-a8aa8754c8d18669b, 2026-10-14) — PAR — Rewire the ai1 ports to real pointer values (~150 sites) so the B16 integration runs the Rust AI instead of a stand-in. — deps: B14, B16 — see `../log/B16.md`
- [ ] B18 — PAR — Harness: reject bare `P` for buf/bpp spec arguments in svflow specs + fix the ~121 bare-`P` call sites (re-record affected areas). — deps: B16 — harness job (Opus); hands-off rule 2 does not apply, it is the harness item
- [ ] B19 — SEQ — Unicorn differential of a whole battle (OV93–96 + ARM9 boot shim) against the B16 integration. — deps: B16, B17
