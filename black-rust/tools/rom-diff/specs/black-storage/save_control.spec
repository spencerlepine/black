# fndiff spec (S5): the save-control status bytes of save_control.c (main ARM9 0x020071E4..0x0200720C).
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-storage save_control --per-function
# The control object is `arg buf 8` (only +0 / +1 are read); the bytes are planted as 0 / 1 / other.
area save_control
set field

fn 020071e4 ret=u32
  nowork
  arg buf 8

fn 020071ec ret=u32
  nowork
  arg buf 8

# FUN_020071F0(ctrl): ctrl[0] == 1 && ctrl[1] == 1
fn 020071f0 ret=bool inputs=400
  nowork
  arg buf 8
  fix arg0 0 u8 alt const 1 || const 1 || const 0 || any
  fix arg0 1 u8 alt const 1 || const 1 || const 0 || any
