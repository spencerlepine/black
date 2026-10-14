# B10k-spec — full checklist note (moved from MIGRATION-AGENT.md section 6, Phase B, 2026-10-13)

Verbatim line as it stood on `main` 2732e87:

- [x] B10k-spec — (<HASH_UNKNOWN>; ~23 min, 280k tokens) core6: **52 fns, 2216/2220 insns (4 dead in 70e4), strict clean, closure stubs**; 5 smoke ports, core6 hash cc5ece4b226eff4a; 9 `bx pc` veneers skipped. Traps: core1 7c18; core2 65a8; core3 66ac 6668 6720 676c 6848 6878 68a8 6948 69b8 6a04 6bf8 6e3c 71e0 7c24; core4 6948 6a9c 6b5c 6bb8 6cac 6dd0 728c 7330 7374 7474 75ec 7664 76f4 7780 77b0 7820 785c 78a8 78f4 7998 79e0 7ad0 7b0c 7b3c 7b7c; core5 70e4 (all 021cxxxx). **Caller-stub mismatches (harness job before registering):** extra args 6668 (core3:111, real 2), 6878 (core3:326, real 2), 68a8 (core3:327, real 3), 6948 (core3:189 + core4:2061, real 3); arg3 pointer passed as v: 6b5c (core4:552), 6e3c (core3:71), 728c (core4:1024), 7374 (core4:1025), 7474 (core4:1026), 71e0 (core3:739; also ret=none but returns 0/1)
