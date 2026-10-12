# fndiff spec: text speed (F9c): FUN_02013080 (speed table at 0x0209DFB0: 3 1 -2 -4 6), FUN_02013020 /
# FUN_02013050 (option -> table index, option read by the stubbed FUN_0200873c/FUN_02008374) and
# FUN_0201CDF4 (speed value -> {delay, chars per tick}).  Main ARM9 Thumb code.
# Grammar: svflow_specs/README.md (generic mode).  Run: tools/check-fn.sh black-text speed --per-function
area speed
set field

fn 02013080 ret=u32
  nowork
  arg u32 alt range 0 6 || oneof 0xffffffff 0x80000000 100

fn 02013020 ret=u32
  nowork
  stub 0200873c ret=none args=v
  stub 02008374 ret=mod:6 args=v

fn 02013050 ret=u32
  nowork
  stub 0200873c ret=none args=v
  stub 02008374 ret=mod:6 args=v

fn 0201cdf4 ret=none
  nowork
  arg u32 alt range 0 9 || oneof 0xffffffff 0xfffffffe 0xfffffffc 0xfffffffb 0x80000000 0x7fffffff 0x100 0xfffffeff 0x1ff
  arg buf 4
