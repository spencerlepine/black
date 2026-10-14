# fndiff spec (S5b): the system-info save block 37 of the main ARM9 (0x02008AEC..0x02008C20): 0x5C bytes
# (+0 u64 RTC offset, +8 MAC, +0xE/+0xF birthday, +0x10 the 0x38-byte game clock).
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-storage sysinfo --per-function
# The OS / RTC queries are stubbed (OS_GetOwnerRtcOffset 02087D08, OS_GetMacAddress 02087C6C, OS_GetOwnerInfo
# 02087C88, GF RTC date/time 0203F49C, RTC_ConvertDateToDay 02088F58, RTC_ConvertDateTimeToSecond 02089004).
# MIi_CpuClear32 runs as real code.
area sysinfo
set field

fn 02008aec ret=u32 inputs=10
  nowork

# sub_02008AF0(block): zero, start the game clock at +0x10
fn 02008af0 ret=none inputs=50
  nowork
  arg buf 0x5c
  stub 0203f49c ret=none args=out:16,out:12
  stub 02088f58 ret=u32 args=in:16
  stub 02089004 ret=u64 args=in:16,in:12

# 0x02008B20(block): RTC offset, MAC, birthday of this DS
fn 02008b20 ret=none inputs=50
  nowork
  arg buf 0x5c
  stub 02087d08 ret=u64 args=
  stub 02087c6c ret=none args=out:6
  stub 02087c88 ret=none args=out:84

# FUN_02008B4C(block): MAC equal (random images: only the "different" answer is drawn; byte compare loop)
fn 02008b4c ret=bool inputs=200
  nowork
  arg buf 0x5c
  stub 02087c6c ret=none args=out:6

# FUN_02008B78(block): RTC offset equal
fn 02008b78 ret=bool inputs=200
  nowork
  arg buf 0x5c
  stub 02087d08 ret=u64 args=

fn 02008b94 ret=u8 inputs=20
  nowork
  arg buf 0x5c

fn 02008b98 ret=u8 inputs=20
  nowork
  arg buf 0x5c

# FUN_02008b9c(gt): start the clock
fn 02008b9c ret=none inputs=50
  nowork
  arg buf 0x38
  stub 0203f49c ret=none args=out:16,out:12
  stub 02088f58 ret=u32 args=in:16
  stub 02089004 ret=u64 args=in:16,in:12

# FUN_02008BD0(gt): penalty != 0
fn 02008bd0 ret=bool inputs=100
  nowork
  arg buf 0x38
  fix arg0 0x34 u32 alt const 0 || any

# FUN_02008be0(gt, n): clamp to 1440, count down
fn 02008be0 ret=none inputs=400
  nowork
  arg buf 0x38
  arg u32 alt range 0 2000 || any
  fix arg0 0x34 u32 alt range 0 2000 || near 0x5a0 3 || any

# FUN_02008C00(gt): clock changed
fn 02008c00 ret=none inputs=50
  nowork
  arg buf 0x38
  stub 0203f49c ret=none args=out:16,out:12
  stub 02088f58 ret=u32 args=in:16
