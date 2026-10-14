# B6h — full checklist note (moved from MIGRATION-AGENT.md section 6, Phase B, 2026-10-13)

Verbatim line as it stood on `main` 2732e87:

- [x] B6h — SEQ (harness job) — move-handler harness (`handlers.py moves`, `handlers::moves`, new stubs) + 2 smoke ports. — deps: Z9, B5h — (<HASH_UNKNOWN> + <HASH_UNKNOWN>; 337 distinct handlers (`handlers.py moves --list`), stub tags 0x12F–0x154 (move mode only; meanings guessed), moves hash d0fa65d01826f672 with 021e0838/021e25dc; abilities/items hashes re-verified on main. 21 move-code helpers must be ported, not stubbed: 021e03d8 03f8 0944 112c 19f8 1a2c 1aa4 1ae4 26c8 3c5c 3e84 3f08 4958 4a08 556c 5784 60a8 8384 8624 8860 8d48; pure helpers 021b8c20 8c38 8d08 99f0 9a50, 021d7d6c 7d9c 7e90 5844, 02018790/02018820 (hidden-power IV parity; port from asm), 0202136c, 02082d44. `check-handlers.sh` now runs cargo fmt after the sync)
