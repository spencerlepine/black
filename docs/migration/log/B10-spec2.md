# B10-spec2 — full checklist note (moved from MIGRATION-AGENT.md section 6, Phase B, 2026-10-13)

Verbatim line as it stood on `main` 2732e87:

- [x] B10-spec2 — SEQ (harness job) — fix specs per B10e/B10d `OPEN.md` requests: bound slot index (aa94/aacc/ab10), `021cc668` handling when ported (bb208/bacac/baef8/bab9c), same-callee dual arg shapes (021eeb64), new specs 021bb0e0/021bb368, 021cb730 slot range, 021cc768 BattleMon model (021cc800); then move the unverified fns into PORTED via the check script. — deps: B10d, B10e — (<HASH_UNKNOWN>; server_side 53/53 93d9452f55ccb1df (+021bb0e0/021bb368 state fns), events 53/53 dbe1679a5938763a (5 hashes changed: S now at a real address; cc668/cc768 stronger stubs). Grammar: `stub … force`, `outn`. Slot index bounded to 0..3 (evidence: 4-slot table, callers, bb85c/bb890 reject ≥4). use_item keeps slot 0..6 with forced handler stubs (reachable sentinel 6 reads uninit stack). No port bugs)
