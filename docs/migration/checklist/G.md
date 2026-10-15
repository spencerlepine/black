# Phase G — Game shell (`black-game`, Bevy) — mostly SEQ (foundation for all screens)

Part of the master checklist (moved out of `MIGRATION-AGENT.md` section 6 on 2026-10-13). Status markers and rules: [§5](../../MIGRATION-AGENT.md). Format: `- [status] ID — TAG — description — deps: … — notes`. A done-note longer than ~400 chars is cut here and kept in full in `../log/<ID>.md`. Finished items are moved out by `tools/archive-checklist.py` (stub in `DONE.md`, full line in `archive/`).

- [ ] G6 — PAR — Audio service: BGM with loop points + crossfade, SE, browser autoplay unlock. — deps: G1, A9
- [ ] G7 — PAR — Sprite/animation system for NCER/NANR-derived atlases. — deps: G1, A10
- [ ] G8 — PAR — 3D map renderer for the overworld: glTF from A5 (**decided 3D by the human 2026-10-13**, matching the original's NSBMD maps; no 2D re-render), containers/permissions from A6, camera + lighting values from the asm (g3d_mapper, camera, fog per F1). — deps: G1, A5, A6
- [ ] G9 — SEQ — Save/load integration (S5) + settings (text speed, battle style, sound). — deps: S5, G2
