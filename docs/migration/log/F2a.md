# F2a — full checklist note (moved from MIGRATION-AGENT.md section 6, Phase F, 2026-10-13)

Verbatim line as it stood on `main` 2732e87:

- [x] F2a — (fc918f6 in /tmp/blk/wt-F2a, merge pending; Opus, 650k tokens) **map_matrix 33/33, fieldmap 95/101, fieldmap_event 45/52, fieldmap_func 13/13, land_data_patch 6/6 = 192/205** verified; `hooks::FieldEnv::call` seam (draw side only as addresses, D3); fx_cos computed (checked vs all 4096 ROM entries). 13 unregistered: 7 whose stubs log unread entry regs (may verify now that Z19 zeroes them: 021880C8 021885E8 021888C8 02188990 02188AD4 021895E8 0218AAD0), 6 whose stubs log a stack address as `v` (0218AED4 0218AF90 0218B0CC 0218B13C 0218B4B0 0218B53C → `in:N`). **nds_rom MapMatrix fixed** (flags==1 = land plane then zone plane; `zone_at`); **Nuvema = block (24,23) on container map24_23** (A5/A6's "container 389" was the zone id; container 389 is m_dun1703). 5/6 mutations (021BBC58 loop bound 30 → input gap)
