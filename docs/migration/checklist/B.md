# Phase B — Battle logic (`black-battle`) — priority 1

Part of the master checklist (moved out of `MIGRATION-AGENT.md` section 6 on 2026-10-13). Status markers and rules: [§5](../../MIGRATION-AGENT.md). Format: `- [status] ID — TAG — description — deps: … — notes`. A done-note longer than ~400 chars is cut here and kept in full in `../log/<ID>.md`. Finished items are moved out by `tools/archive-checklist.py` (stub in `DONE.md`, full line in `archive/`).

- [~] B19b (worker worktree, 2026-10-14) — PAR — (= V-0033, low; validator confirmed the widened spec MATCHes 41778e03be68b664 at 1000 inputs) harness: events.spec declares the pass-through args (`args=v,v,v` for 021cb174 and 021b8c50, `v,v` for 021cb1a4 and 021b8cb0), re-record events, remove the testkit UNDECLARED_PASS_THROUGH list — deps: B19
