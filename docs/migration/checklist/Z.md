# Phase Z — Foundations (do first, all SEQ)

Part of the master checklist (moved out of `MIGRATION-AGENT.md` section 6 on 2026-10-13). Status markers and rules: [§5](../../MIGRATION-AGENT.md). Format: `- [status] ID — TAG — description — deps: … — notes`. A done-note longer than ~400 chars is cut here and kept in full in `../log/<ID>.md`. Finished items are moved out by `tools/archive-checklist.py` (stub in `DONE.md`, full line in `archive/`).

- [ ] Z11 — PAR — Generators: (a) getter/registry/pool tables → generated Rust tables from bytes (no LLM); (b) cluster templates: from `clusters.txt`, port one member, generate the others from their constants, verify each with the harness. Use on B5/B6 first. — deps: Z8, Z9
