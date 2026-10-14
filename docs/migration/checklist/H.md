# Phase H — Audio extras

Part of the master checklist (moved out of `MIGRATION-AGENT.md` section 6 on 2026-10-13). Status markers and rules: [§5](../../MIGRATION-AGENT.md). Format: `- [status] ID — TAG — description — deps: … — notes`. A done-note longer than ~400 chars is cut here and kept in full in `../log/<ID>.md`. Finished items are moved out by `tools/archive-checklist.py` (stub in `DONE.md`, full line in `archive/`).


- [ ] H1 — PAR — Pokémon cries (SWAR/SSAR → OGG, pitch per form). — deps: A8
- [ ] H2 — PAR — Field sound animations (`overlay_056` field_fk_sound_anime) as G6 events. — deps: G6
- [ ] H3 — SEQ — (Optional, after release) Real-time SSEQ sequencer in Rust for dynamic music. — deps: A8, G6
