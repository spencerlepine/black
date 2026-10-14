# Phase B — Battle logic (`black-battle`) — priority 1

Part of the master checklist (moved out of `MIGRATION-AGENT.md` section 6 on 2026-10-13). Status markers and rules: [§5](../../MIGRATION-AGENT.md). Format: `- [status] ID — TAG — description — deps: … — notes`. A done-note longer than ~400 chars is cut here and kept in full in `../log/<ID>.md`. Finished items are moved out by `tools/archive-checklist.py` (stub in `DONE.md`, full line in `archive/`).

- [~] B18 (worktree-agent-a6c560338857d4623, 2026-10-14) — PAR — (also: AI masks that stop in flow areas, from B17: bits 0/1/4 → core6 021c728c gets a caller-local record buffer via 021c7e14 → core4 021c1e74; bit 11 → core2 021bd708 → calc 021d8244 touches the flow work unlent; bit 12 → vm.c reads unmapped 0x02309B70; bits 14/15 → script members missing from archive 0xAB) Harness: reject bare `P` for buf/bpp spec arguments in svflow specs + fix the ~121 bare-`P` call sites (re-record affected areas). — deps: B16 — harness job (Opus); hands-off rule 2 does not apply, it is the harness item
- [ ] B19 — SEQ — Unicorn differential of a whole battle (OV93–96 + ARM9 boot shim) against the B16 integration. — deps: B16, B17
