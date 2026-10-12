# svflow spec: harness self-test. Three trivial ROM functions that exercise the `work` and `bpp`
# argument kinds and the u8/u16/u32 return kinds on both sides; testkit.rs::tests re-implements
# them inline and checks the Python hashes (python svflow.py selftest --all --json).
area selftest

# u16 at work+0x1F80 (B10c api area; not a port, only a harness probe)
fn 021c7cd4 ret=u16
  arg work

# low 16 bits of work word at +0x10
fn 021c8138 ret=u32
  arg work

# BattleMon move count (battle_mon::move_count)
fn 021d5448 ret=u8
  arg bpp
