# fndiff spec (S5): mail.c of the main ARM9 (0x02008EAC..0x020090B0): mail records and the mailbox
# save block 40. Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-storage mail --per-function
#
# MAIL = 0x38 bytes: +0 u32 ID, +4 gender, +5 language, +6 version, +7 design (0..11, 0xFF empty),
# +8 name u16[8], +0x1E u16, +0x20 three 8-byte sentences. MAILBOX = 20 mails (0x460 bytes).
# Real code on the ROM side: MIi_CpuFill16 / MI_CpuCopy8, FUN_02045A98 (terminator 0xFFFF from
# 0x020A7E7C), FUN_0202428C (sentence clear) and FUN_02024570 (sentence copy), and the mystatus
# getters FUN_020084F4 / FUN_02008550 / FUN_02008530. FUN_02012944 (game data -> player status) is
# stubbed to the address of a `buf 0x20` status argument the function never reads as a register.
# Mailbox slot indices are kept >= 0 (a negative index addresses memory before the block).
area mail
set field

# FUN_02008EAC(mail): clear
fn 02008eac ret=none
  nowork
  arg buf 0x38

# FUN_02008EF8(mail): design <= 11
fn 02008ef8 ret=bool inputs=300
  nowork
  arg buf 0x38
  fix arg0 7 u8 alt range 0 12 || near 11 1 || const 0xff || any

# FUN_02008F3C(src, dst): MI_CpuCopy8(src, dst, 0x38)
fn 02008f3c ret=none
  nowork
  arg buf 0x38
  arg buf 0x38

# FUN_02008F48(mail, design, _, gamedata): clear, design, name / gender / ID of the player
fn 02008f48 ret=none
  nowork
  arg buf 0x38
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x20
  stub 02012944 ret=ptr:arg4:0:1 args=v

fn 02008f80 ret=u32
  nowork
  arg buf 0x38

# FUN_02008F84(mail): mail + 8
fn 02008f84 ret=ofs:arg0
  nowork
  arg buf 0x38

# FUN_02008F88(mail, src): MI_CpuCopy8(src, mail + 8, 0x10)
fn 02008f88 ret=none
  nowork
  arg buf 0x38
  arg buf 0x10

fn 02008f9c ret=u32
  nowork
  arg buf 0x38

fn 02008fa0 ret=none
  nowork
  arg buf 0x38
  arg u32 any

fn 02008fa4 ret=u32
  nowork
  arg buf 0x38

# FUN_02008FA8(mail, v): design = v if v < 12
fn 02008fa8 ret=none inputs=300
  nowork
  arg buf 0x38
  arg u32 alt range 0 13 || near 12 1 || any

fn 02008fb0 ret=u32
  nowork
  arg buf 0x38

fn 02008fb4 ret=none
  nowork
  arg buf 0x38
  arg u32 any

fn 02008fb8 ret=u32
  nowork
  arg buf 0x38

fn 02008fbc ret=none
  nowork
  arg buf 0x38
  arg u32 any

fn 02008fc0 ret=u32
  nowork
  arg buf 0x38

fn 02008fc4 ret=none
  nowork
  arg buf 0x38
  arg u32 any

# FUN_02008FC8(mail, i): mail + 0x20 + 8i for i < 3, else mail + 0x20
fn 02008fc8 ret=ofs:arg0 inputs=300
  nowork
  arg buf 0x38
  arg u32 alt range 0 4 || any

# FUN_02008FD8(mail, src, i): sentence i = *src (FUN_02024570) if i < 3
fn 02008fd8 ret=none inputs=300
  nowork
  arg buf 0x38
  arg buf 8
  arg u32 alt range 0 4 || any

fn 02008ffc ret=u32 inputs=10
  nowork

# FUN_02009004(mailbox): clear all 20
fn 02009004 ret=none inputs=50
  nowork
  arg buf 0x460

# FUN_02009020(mailbox): FUN_02009074(mailbox, 20)
fn 02009020 ret=u32 inputs=400
  nowork
  arg buf 0x460
  fix arg0 7 u8 alt range 0 11 || range 0 11 || range 0 11 || const 0xff || any rep=20,0x38

# FUN_02009074(mails, n): first i < n with an invalid design, else -1
fn 02009074 ret=u32 inputs=400
  nowork
  arg buf 0x460
  arg u32 alt range 0 20 || oneof 0xffffffff 0xfffffffe 0x80000000
  fix arg0 7 u8 alt range 0 11 || range 0 11 || range 0 11 || const 0xff || any rep=20,0x38

# FUN_0200902C(mailbox, location, idx): clear slot idx (idx < 20)
fn 0200902c ret=none
  nowork
  arg buf 0x460
  arg u32 any
  arg u32 range 0 24

# FUN_0200903C(mailbox, location, idx, src): slot idx = *src (idx < 20)
fn 0200903c ret=none
  nowork
  arg buf 0x460
  arg u32 any
  arg u32 range 0 24
  arg buf 0x38

# unlabeled 0x020090A0(mailbox, location, idx): mailbox + idx * 0x38 if idx < 20, else 0
fn 020090a0 ret=ofs:arg0
  nowork
  arg buf 0x460
  arg u32 any
  arg u32 range 0 24
