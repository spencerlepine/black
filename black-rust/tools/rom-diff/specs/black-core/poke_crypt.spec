# fndiff spec: poke_tool crypt helpers (P1a), main ARM9 code. Grammar: svflow_specs/README.md.
# Run: tools/check-fn.sh black-core poke_crypt --per-function
area poke_crypt
set field

# FUN_02019A7C(uint *state): *state = *state * 0x41C64E6D + 0x6073; return *state >> 16
fn 02019a7c ret=u32
  nowork
  region state 4
  arg region state

# FUN_02019A9C(const void *buf, uint size): u16 sum of the first size/2 halfwords (odd tail ignored)
fn 02019a9c ret=u16
  nowork
  region data 0x100
  arg region data
  arg u32 alt range 0 0x100 || oneof 0 1 2 3 0xFF 0x100
