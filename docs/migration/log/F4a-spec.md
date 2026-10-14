# F4a-spec — full checklist note (moved from MIGRATION-AGENT.md section 6, Phase F, 2026-10-13)

Verbatim line as it stood on `main` 2732e87:

- [x] F4a-spec — (f4a-spec in /tmp/blk/wt-F4as, merge pending; Opus, ~2.5 h, 400k tokens) black-field areas **fldmmdl** (0216CC58–0216E62C: 253 fns incl. 18 Ghidra missed, 2754/2754; cf1b5fcbd5cf11de) + **event_battle_head** (0216E62C–0216E954 = head of event_battle.c, for F5b: 7 fns; c22d9c951e9751cb); strict/closure/order clean; 7 smoke ports. Layouts (crate README): MMDLSYS 0x48, MMDL 0x100 (grid s16 +0x30..+0x40, VecFx32 +0x44, proc tables +0x8C/+0x90), HEADER 0x24 = A6 NPC record — **nds_rom events::Npc mislabels: `y` is grid z, `unk_20` is fx32 height, `unk_04` packs move code/event/dir/params/limits, u32 kind at +0x18 (0 grid / 1 rail)**. Grammar `outp:work:OFS:N`. In-area force stubs 0216E400/0216E40C/0216D8AC → callers' ports reach them via hooks. Trap: in-area getters compared by callers need `pick:` answers
