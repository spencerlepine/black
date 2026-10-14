# B10-spec4 — full checklist note (moved from MIGRATION-AGENT.md section 6, Phase B, 2026-10-13)

Verbatim line as it stood on `main` 2732e87:

- [x] B10-spec4 — (~28 min, 237k tokens; coordinator-reviewed) **core4 102/102 (c364d57982ab0816), core5 66/66 (bb45a0472a24066e)**. Part A: 12 core5 caller stubs fixed from disassembly (5620 r3 `outn` null; 5718 `p`, 572c `p,v`; 5798 6 words; 6350/6394/62d8/6458/64b4/6564 arg0 `v`; 63d0 `ret=u16 args=p,v`; 5dac kept `v` (only null caller in core4)) + `force`; Rust call sites adjusted (`waddr` params in core1/core2; 5418 calls 5620 directly). Part B: new stub grammar **`lt:K`** (answer `mix % argK`, 0 if argK = 0: rand_range invariant) on 78ac in 39f4/3810/376c, and **`v16`** (log/mix low halfword only; 7c70 reads only `(u16)p1`) on 7c70 in 0f24/11a4/11f0 blocks; svflow.py + black-fndiff mirrored with unit tests. Hashes changed (explained per fn): core1 51fde53dba108531, core2 6c1852465bd84012, core3 be6cb444e0d8de76
