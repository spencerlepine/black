# Phase B — Battle logic (`black-battle`) — priority 1

Part of the master checklist (moved out of `MIGRATION-AGENT.md` section 6 on 2026-10-13). Status markers and rules: [§5](../../MIGRATION-AGENT.md). Format: `- [status] ID — TAG — description — deps: … — notes`. A done-note longer than ~400 chars is cut here and kept in full in `../log/<ID>.md`. Finished items are moved out by `tools/archive-checklist.py` (stub in `DONE.md`, full line in `archive/`).

- [~] B18b (worktree-agent-adae199e185c4775e, 2026-10-14) — PAR — harness: extend bare_p_lint/`ptr` to the fn-area specs of the other crates (black-text wordset 9 stub args, black-script scrcmd_c03center 1, and any others it flags) and fix their ports; decide + implement ptr checking for stub args the callee declares `work`/`region` (many battle ports still pass bare P for the flow-work pointer) — deps: B18
- [~] B19 (worktree-agent-aefa3baf9274f6a52, 2026-10-14) — SEQ — Unicorn differential of a whole battle (OV93–96 + ARM9 boot shim) against the B16 integration. — deps: B16, B17
