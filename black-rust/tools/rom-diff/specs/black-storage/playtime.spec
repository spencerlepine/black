# fndiff spec (S5): playtime.c of the main ARM9 (0x0200857C..0x02008700), the 8-byte play-time
# block. Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-storage playtime --per-function
#
# PLAYTIME = 8 bytes: +0 u16 hours, +2 u8 minutes, +3 u8 seconds, +4 u32 stamp (bits 0-6 year,
# 7-10 month, 11-15 day, 16-20 hour, 21-26 minute). Every function takes the block as `arg buf 8`.
# FUN_02088864 / FUN_0208890C (RTC date / time into a stack struct) are stubbed (`outm` so every
# field bit varies); _u32_div_f and MI_CpuCopy8 run as real code.
area playtime
set field

# FUN_020085A0(pt): time 0, stamp bits 0-26 cleared
fn 020085a0 ret=none
  nowork
  arg buf 8

# FUN_020085D4(pt, secs): add seconds with carry, stop at / clamp to 999:59:59
fn 020085d4 ret=none inputs=3000
  nowork
  arg buf 8
  arg u32 alt range 0 120 || range 0 4000 || range 0 400000 || any
  fix arg0 0 u16 alt near 999 2 || const 999 || range 0 1000 || any
  fix arg0 2 u8 alt const 59 || near 59 2 || range 0 59 || any
  fix arg0 3 u8 alt const 59 || near 59 2 || range 0 59 || any

# FUN_0200863C(dst, src): MI_CpuCopy8(src, dst, 8)
fn 0200863c ret=none
  nowork
  arg buf 8
  arg buf 8

fn 0200864c ret=u32
  nowork
  arg buf 8

fn 02008650 ret=u32
  nowork
  arg buf 8

fn 02008654 ret=u32
  nowork
  arg buf 8

# FUN_02008658(pt): stamp from RTC date (year, month, day, weekday) and time (hour, minute, second)
fn 02008658 ret=none
  nowork
  arg buf 8
  stub 02088864 ret=none args=outm:16:0xFF/0xFF/0xFF/0xFF
  stub 0208890c ret=none args=outm:12:0xFF/0xFF/0xFF/0xFF

fn 020086c8 ret=u32
  nowork
  arg buf 8

fn 020086d0 ret=u32
  nowork
  arg buf 8

fn 020086d8 ret=u32
  nowork
  arg buf 8

fn 020086e0 ret=u32
  nowork
  arg buf 8

fn 020086e8 ret=u32
  nowork
  arg buf 8

# FUN_020086F0(pt, &out): *out = stamp word
fn 020086f0 ret=none
  nowork
  arg buf 8
  arg buf 4

# FUN_020086F8(pt, &src): stamp word = *src
fn 020086f8 ret=none
  nowork
  arg buf 8
  arg buf 4
