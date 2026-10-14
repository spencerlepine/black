# F4c-spec — full checklist note (moved from MIGRATION-AGENT.md section 6, Phase F, 2026-10-13)

Verbatim line as it stood on `main` 2732e87:

- [x] F4c-spec — (f4c-spec committed, merge pending; Opus, ~1.5 h, 290k tokens) black-field area **trainer_eye** = event_trainer_eye.c 0x021AE0CC–0x021AE948 (38 fns; F1's ~55 included scrcmd_talk), 941/941 insns, 6 smoke ports (7eaf18973a1bd90a), 4/4 mutations. **Sight**: trainer kind from event type (1,4..8,11..13 = facing dir; 2 = all four dirs), range = header param 0, trainer id from script id (0215A460), defeated check 0215A4D8; line test table 0x021D9338 (same column/row + height, within range); every tile between free and not the effect-encounter tile; double-battle pairing by battle type 0215A4C8 (0 = two separate trainers, 1 = partner with same trainer id, 2/3 = triple/rotation?); script 0xE20; approach = 18-state TASK (table 0x021D9348): face, ! balloon, 30-frame wait, walk distance−1, player turns
