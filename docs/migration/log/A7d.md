# A7d — full checklist note (moved from MIGRATION-AGENT.md section 6, Phase A, 2026-10-13)

Verbatim line as it stood on `main` 2732e87:

- [x] A7d — `moves` (021) (coordinator; 36-B layout from getter `FUN_0201BDB4(record, param)`; stat fields at 0x15–0x1D from data (Ghidra mis-decompiles that switch range); 560 records byte round-trip) - [x] A7e — `items` (024) (coordinator; layout from getter `FUN_02021014(record, param 0..0x11)`; 627 records byte round-trip; several fields named by getter param only; usage block at +0x10 raw until S4) - [x] A7f — `trainers` (092, 093) (coordinator; 616 trainers: 20-B record per getter `FUN_0202A230` (loads ARC 0x5C = 092), party entries 8 B (+2 item if format&2, +8 moves if format&1); trainer 0 is a dummy with a 6-B party kept raw; Patrat check on trainer 1; several fields UNVERIFIED meanings)
