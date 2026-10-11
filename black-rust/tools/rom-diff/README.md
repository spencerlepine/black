# rom-diff

Runs original Pokemon Black machine code (ARM9 + overlay 93) under the
[Unicorn](https://www.unicorn-engine.org/) emulator and prints FNV-1a hashes that the
`*_matches_rom_fuzz` tests in `crates/black-battle` must reproduce.

Inputs are read at runtime from the decomp build (`black-rom/build/black.us/main.sbin` and
`OVY_93.sbin`, which match the retail ROM). Override the directory with `BLACK_ROM_BUILD`. Nothing
from the ROM is copied into this repo.

```sh
python3 -m venv /tmp/blk/venv && /tmp/blk/venv/bin/pip install unicorn
/tmp/blk/venv/bin/python -I tools/rom-diff/gen.py            # all suites
/tmp/blk/venv/bin/python -I tools/rom-diff/gen.py event_var  # one suite
```

- `emu.py` maps main RAM, loads ARM9 at `0x02004000` and overlay 93 at `0x021B6100`, and calls a
  Thumb function with r0-r3. `Emu.stub(addr, fn)` replaces an undecompiled callee with a Python
  function. The return honours interworking: Unicorn picks Thumb from bit 0 of the value
  written to PC.
- `gen.py` has one function per Rust fuzz test. Each one copies its Rust test's generator
  (xorshift32 seeding, call order, hashed values) exactly, and its stubs behave like the Rust
  test mocks.

To verify a newly ported function, add the call to both sides with `"TODO"` as the expected hash
in Rust. Run `gen.py` and check that its hash equals the one the Rust test prints, then paste it in.
Never copy the Rust output on its own.
