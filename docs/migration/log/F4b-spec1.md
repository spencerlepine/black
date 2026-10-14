# F4b-spec1 — full checklist note (moved from MIGRATION-AGENT.md section 6, Phase F, 2026-10-13)

Verbatim line as it stood on `main` 2732e87:

- [x] F4b-spec1 (44df1e8, merged 2026-10-13; Opus, ~1.5 h, 280k tokens) — black-field **acmd1** (14 fns, 0x02197D3C–0x02197F5C; 80622af7be2ef983) + **mmdl_draw** (4 fns 0x02197C4C–0x02197D3C, orphaned after ircbattlemenu ends at 0x02197C4A; 472ce4ccbd163df4), 100% coverage, 6 smoke ports. ACMD layout in README (u16 code +0x26, seq +0x28, tables 0x021D5F58 grid / 0x021D624C rail, 189 codes, step returns 0 done/1 continue). **Real bounds: F4b-spec2 = 0x02197F5C–0x0219A098 (225 fns, grid step targets), F4b-spec3 = 0x0219A098–0x0219C4AC (187 fns, movement-type procs); 0x0219C4AC/C52C = render glue (U2)**. Row 21.11 (g3dobj) also holds the rail acmd steps 0x0219DC18–0x0219E2A0 (needs splitting). Specs must bound code/seq.
