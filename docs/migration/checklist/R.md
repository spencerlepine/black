# Phase R — Integration and release (SEQ, last)

Part of the master checklist (moved out of `MIGRATION-AGENT.md` section 6 on 2026-10-13). Status markers and rules: [§5](../../MIGRATION-AGENT.md). Format: `- [status] ID — TAG — description — deps: … — notes`. A done-note longer than ~400 chars is cut here and kept in full in `../log/<ID>.md`. Finished items are moved out by `tools/archive-checklist.py` (stub in `DONE.md`, full line in `archive/`).


- [ ] R1 — SEQ — Vertical slice: new game → first route → first wild battle → catch → save/load, native and browser. — deps: U1, U2, U3, U4, U6, U8, U10, F12, B16
- [ ] R2 — SEQ — Full story playthrough fixes (scripts, events, trainers) through the Elite Four/credits. — deps: R1, F8, F10, U-phase
- [ ] R3 — SEQ — Performance pass (wasm frame time, asset sizes, lazy loading). — deps: R1
- [ ] R4 — SEQ — Serve the wasm build locally only (`trunk serve` / a static file server on localhost; **human decision 2026-10-13: never hosted anywhere, no paid hosting**; D5). — deps: R1
- [ ] R5 — SEQ — Retire `black-battle-wasm` + `web/` playground once `black-game` covers it. — deps: R1
