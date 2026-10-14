# Phase A — Asset extraction (`nds-rom` readers + `black extract-assets`)

Part of the master checklist (moved out of `MIGRATION-AGENT.md` section 6 on 2026-10-13). Status markers and rules: [§5](../../MIGRATION-AGENT.md). Format: `- [status] ID — TAG — description — deps: … — notes`. A done-note longer than ~400 chars is cut here and kept in full in `../log/<ID>.md`. Finished items are moved out by `tools/archive-checklist.py` (stub in `DONE.md`, full line in `archive/`).

Readers are PAR (one module each in `nds-rom`). Converters are PAR per asset category.

- [ ] A9b — PAR — Zone → BGM selection logic (field ISS/`field_sound_system.c`, `unk_0200674C`, run in `unk_0201FC1C` 0x02027xxx–0x0202Bxxx; ~300 fns), fed by zone headers. — deps: F2, A9
