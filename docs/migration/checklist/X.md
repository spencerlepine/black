# Out of scope (D9) — revisit only by explicit decision

Part of the master checklist (moved out of `MIGRATION-AGENT.md` section 6 on 2026-10-13). Status markers and rules: [§5](../../MIGRATION-AGENT.md). Format: `- [status] ID — TAG — description — deps: … — notes`. A done-note longer than ~400 chars is cut here and kept in full in `../log/<ID>.md`.


- [-] X1 — networking / C-Gear / Wi-Fi / IR (52 files, ≈10,189 fns), `overlay_116/121/177/186/208`-net parts; T1 adds DWC/GameSpy in main (`unk_020583CC..unk_02062634`, 261 fns) and overlays 0, 3, 4, 7, 8, 13, 15, 17, 99, 102–105 (IR), 122 (C-Gear), 128, 178, 219 (Geonet)
- [-] X2 — Musicals (`overlay_112` mus\_\*), Dream World, PWT/Battle Subway online, mystery gift; + `overlay_184/185` (musical helpers)
- [-] X3 — SDK/OS, ARM9 autoload (ITCM/DTCM), runtime library — replaced by Rust std/Bevy; port individual helpers only when ported logic calls them (e.g. `_u32_div_f` semantics).; T1: SDK/NNS bands `unk_02062634..unk_0209CE10` (~918 fns)
