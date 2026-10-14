# Phase P — Pokémon data and formulas (`black-pokemon`) — priority 2

Part of the master checklist (moved out of `MIGRATION-AGENT.md` section 6 on 2026-10-13). Status markers and rules: [§5](../../MIGRATION-AGENT.md). Format: `- [status] ID — TAG — description — deps: … — notes`. A done-note longer than ~400 chars is cut here and kept in full in `../log/<ID>.md`. Finished items are moved out by `tools/archive-checklist.py` (stub in `DONE.md`, full line in `archive/`).

(Exact source files come from T2; expected `poke_tool.c`, `poke_personal.c`, `shinka_check.c`, `waza_tool.c`.)

- [~] P7b (worktree-agent-a9de61f900a5d5df2, 2026-10-14) — PAR — egg hatching (step counter, egg-cycle decrement, hatch conversion): locate the overlay (not in sets.txt; candidates: field step/event code, overlay 215 egg_demo for the scene), add the set, port the logic — deps: P7
