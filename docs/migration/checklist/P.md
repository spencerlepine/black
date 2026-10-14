# Phase P — Pokémon data and formulas (`black-pokemon`) — priority 2

Part of the master checklist (moved out of `MIGRATION-AGENT.md` section 6 on 2026-10-13). Status markers and rules: [§5](../../MIGRATION-AGENT.md). Format: `- [status] ID — TAG — description — deps: … — notes`. A done-note longer than ~400 chars is cut here and kept in full in `../log/<ID>.md`. Finished items are moved out by `tools/archive-checklist.py` (stub in `DONE.md`, full line in `archive/`).

(Exact source files come from T2; expected `poke_tool.c`, `poke_personal.c`, `shinka_check.c`, `waza_tool.c`.)

- [~] P7 (worktree-agent-a680479667f83fdbe, 2026-10-14) — PAR — Eggs, breeding, egg moves, daycare. — deps: P1, P4 — egg moves = ARC 123 (650 records, read in overlay 21), baby forms ARC 020 (`FUN_021C4504`); daycare code is field-side
