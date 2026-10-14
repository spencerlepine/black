# F5a — full checklist note (moved from MIGRATION-AGENT.md section 6, Phase F, 2026-10-13)

Verbatim line as it stood on `main` 2732e87:

- [x] F5a (59d3838, merged 2026-10-13; Opus, ~2.5 h, 400k tokens) — 147/151 verified: encount_st 10 (0e78a61f591671cc), enc_pokeset 25 (fc4dbf578e169a8c), encount 15/19 (75a53e7247a857c0), effect_encount 29 (bae064b190095912), map_attr 68 (36aa2102c19d726f); 6/11 mutations (3 survived: 021AABF4 tile 27 picks, 021A9828 Cute Charm, 021AA42C 02010FB0 answers too wide). 021A9F40/A0A0/A180/A248 UNVERIFIED: encount.spec stubs log the stack EFP pointer `v` — fix F5a-spec2 (`stub 021a96b8 args=v,in:0x2C,in:0x14`, `021a9780 args=v,in:0x2C,v`, `0201347c args=v,v,v,in:14,v,v`; ports pass CArg::In). 021AA0A0 is `ret=none` but returns the 0216E7C8 event. Harness: check-fn.sh filter `encount::tests::fndiff_match_rom` also matches effect_encount (needs `--exact`, coordinator fix)
