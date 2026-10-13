# fndiff spec: fieldmap.c, second part (overlay 21 Thumb code 0x02189BD0..0x0218B752, F1 area 21.00; F2a-spec): the field
# event checks (talk, touch, step, push, bump, sign, warp, hidden item, special tiles), the EVENT REQUEST they share and the
# small party helpers of the field moves. Skipped veneer: 0218AECC (`ldr r3,=021626B9; bx r3`: callers stub 021626B8).
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-field fieldmap_event --per-function
#
# Entry points (set by the FIELDMAP start step 021885E8 with 0201209C(gamesys, fn, fieldmap)): 0218A074 (grid map),
# 0218A240 (rail map, kind 1), 0218A4BC (hybrid, kind 2: picks grid / rail by 02188CFC), 0218A0E4 (comm zones). Each returns a
# GMEVENT (0 = none). 02189BD0 / 0218A278 / 0218A0E4 are the checks; 0218A6E8 / 0218A728 the post-event bookkeeping.
#
# EVENT REQUEST (0x40 bytes on the caller's stack, cleared and filled by 0218A4E0(req, gamesys, fieldmap)):
#   +0x00 u16 heap id   +0x04 gamesys   +0x08 GAMEDATA   +0x0C zone event data (02012844(gd))   +0x10 u16 current zone id
#   +0x14 -> FIELDMAP   +0x18 player (FIELDMAP +0x8C)   +0x1C u32 player move state (021A308C; 0 / 3 walk / ..)
#   +0x20 u32 (021A3080: 1 / 2)   +0x24 u16 facing direction (0 up, 1 down, 2 left, 3 right)   +0x26 u16 pending script id
#   (02014478(02012B38(gd)))   +0x28 u32 key held mask   +0x2C u32 key trigger mask   +0x30 u16 trigger direction
#   (0x40 up 0, 0x80 down 1, 0x20 left 2, 0x10 right 3, else 9)   +0x34 -> player position fx32[3] (0216D888(mmdl))
#   +0x38 u32 (021A30C8)   +0x3C u32 FLAGS: bit 0 = held & 1 (A: talk), bit 1 = held & 0x400, bit 6 = held & 0x800,
#   bit 7 = none of the three, bit 10 = move state 0 or 3 (those four only then), bit 2 = (+0x20 == 1 && state == 3),
#   bit 3 = state == 3 (stepped), bit 4 = state != 3 && +0x20 == 2, bit 5 = trigger direction == facing, bit 9 = 021A3584(player)
# In the specs a function that takes the request gets work+0x500 (fixes below), its +0x14 = work+0 (FIELDMAP) and its +0x34 =
# work+0x600; callers that build it on their stack see a stub 0218A4E0 that fills it with masked random bytes (normal mode).
# Porting order: 0218A4E0 first (a real request consumer with a stubbed builder reads a zero request).
# Arena: FIELDMAP work+0 (+0x108 MAP CTRL work+0x400), REQUEST work+0x500, position work+0x600, BG event record (0216282C) work+0x640,
# GMEVENT work (020124E8) work+0x700.
area fieldmap_event
set field

# 02189BD0(gamesys, fm, &talked, &flag): *talked = *flag = 0; req = 0218A4E0(stack, gamesys, fm); the checks in order, the first
# that returns a GMEVENT wins: 0218A738 (pending script; *talked = 1), 021C868C(gamesys, *02012ED0(gd)) / 021AE0CC(fm) (*talked = 1),
# FLAGS bit 2: 0218A728(gd), 0218A76C (talk; *talked = 1), 0218ACD0 (talk to a BG object), 0218AD28; 0218A9B0 (step counters);
# FLAGS bit 3: 0218A7F0; 0218A824; 0218A878; 0218B208; then the walk-time checks 021A2F20 (with flag 0x964 of 020142E8), the comm /
# trainer / hidden-item / sign / warp checks of the ov11/ov18 callees (0217BFD8, 0217CA84, 0217C070, 0217B810, 0217689C, 0216B3A8,
# 021754B4, 0217FC6C, 0217BDE0, 0217AFC4, 0217FB5C), FLAGS bit 0: 0218B4B0 (+ 0219F01C, *talked = 1), 02162A54 / 0218B56C /
# 021CFAC0 / 021CFC14 / 021CD440 / 02162940 (scripts via 02158768); state 0 or 3: 0218B408; FLAGS bit 5: 0218AED4, 0218AF90;
# FLAGS bit 6, 10, 1, 7: 020221A4-gated comm and menu checks (0215FF34/021A0ED4/0215FF54, 021C06CC, 021A0F90/0215FC40 (*flag = 1),
# 021A0EB0, 0218ACA8), 0217B410. Returns the GMEVENT or 0.
fn 02189bd0 ret=u32 inputs=1500
  arg u32 any
  arg work 0
  arg work 0x780
  arg work 0x784
  fix work 0x8C u32 any
  fix work 0x90 u32 any
  fix work 0x108 u32 addrof work 0x400
  fix work 0x400 u32 alt range 0 2 || range 0 8
  fix work 0x150 u32 alt const 0 || any
  fix work 0xD8 u16 alt range 0 0x1ff || any
  stub 0218a4e0 ret=none args=outm:0x40:0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/3/0/0/0/3/0/0/0/3/0/1/0/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/3/0/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/7/0/0,v,v
  stub 0218a738 ret=pick:0/0/0/0/0/0/1 args=p  # in-area
  stub 02012ed0 force ret=ptr:work:0x220:1 args=v  # gamedata -> record whose first word is read
  stub 021c868c force ret=pick:0/0/0/0/0/1 args=v,v
  stub 021ae0cc force ret=pick:0/0/0/0/0/1 args=v
  stub 0218a728 ret=none args=v  # in-area
  stub 0218a76c ret=pick:0/0/0/0/0/0/1 args=v  # in-area
  stub 0218acd0 ret=pick:0/0/0/0/0/0/1 args=v,v,v  # in-area
  stub 0218ad28 ret=pick:0/0/0/0/0/0/1 args=v,v  # in-area
  stub 0218a9b0 ret=pick:0/0/0/0/0/0/1 args=p,v  # in-area
  stub 0218a7f0 ret=pick:0/0/0/0/0/0/1 args=v  # in-area
  stub 0218a824 ret=pick:0/0/0/0/0/0/1 args=v  # in-area
  stub 0218a878 ret=pick:0/0/0/0/0/0/1 args=v,v,v  # in-area
  stub 0218b208 ret=pick:0/0/0/0/0/0/1 args=p,p,v  # in-area
  stub 0201296c force ret=u32 args=v  # main
  stub 020142e8 force ret=pick:0/1/2 args=v,v  # main
  stub 021a2f20 force ret=pick:0/0/0/0/0/1 args=v,v,v,v
  stub 02012138 force ret=u32 args=v  # main
  stub 02158344 force ret=pick:0/1 args=v
  stub 021a3074 force ret=u32 args=v  # ov21
  stub 0217bfd8 force ret=pick:0/0/0/0/0/1/2 args=v,v
  stub 0217ca84 force ret=u32 args=v,v  # ov18
  stub 02026454 force ret=pick:0/1/2/3/4/5/6 args=v
  stub 02012b70 force ret=pick:0/1/2 args=v
  stub 0217c070 force ret=pick:0/0/1/2 args=v
  stub 0217b810 force ret=u32 args=v  # ov18
  stub 0217689c force ret=pick:0/0/0/1/2 args=v,v
  stub 02012fb0 force ret=u32 args=v  # main
  stub 0216b3a8 force ret=u32 args=v,v,v  # ov10
  stub 021754b4 force ret=pick:0/0/0/1/2 args=v,out:4
  stub 021a2f6c force ret=none args=v  # ov21
  stub 0217fc6c force ret=u32 args=v,v,v,v,v  # ov18
  stub 0217bde0 force ret=pick:0/0/0/0/1 args=v,v,v
  stub 0217afc4 force ret=pick:0/0/0/1/2 args=v,v,out:4
  stub 0217fb5c force ret=u32 args=v,v,p,v,v  # ov18
  stub 0218b408 ret=pick:0/0/0/0/0/0/1 args=p,v,v  # in-area
  stub 0218aed4 ret=pick:0/0/0/0/0/0/1 args=p,p,v  # in-area
  stub 0218af90 ret=pick:0/0/0/0/0/0/1 args=p,v,v  # in-area
  stub 020221a4 force ret=pick:0/1/1 args=
  stub 02188ccc force ret=u32 args=v  # fieldmap (other F2a area)
  stub 0215ff34 force ret=pick:0/1 args=v
  stub 021a0ed4 force ret=pick:0/1 args=v
  stub 0215ff54 force ret=pick:0/0/0/1 args=v,v,v
  stub 0201212c force ret=pick:0/1/2 args=v
  stub 021c06cc force ret=u32 args=v  # ov21
  stub 021a0f90 force ret=pick:0/6/6/7 args=v
  stub 0215fc40 force ret=u32 args=v,v,v  # ov10
  stub 021a0eb0 force ret=pick:0/0/0/0/1 args=v,v
  stub 0218aca8 ret=pick:0/0/0/0/0/0/1 args=v  # in-area
  stub 0217b410 force ret=pick:0/0/0/1 args=v,v,v
  stub 021a3050 force ret=u32 args=v  # ov21
  stub 021a3038 force ret=none args=v,outm:12:0/0/3/0/0/0/0/0/0/0/3/0
  stub 02164208 force ret=none args=v,in:12,v
  stub 0218b4b0 ret=pick:0/0/0/0/0/0/1 args=p,v  # in-area
  stub 0216d5b8 force ret=u32 args=v  # ov10
  stub 02158d90 force ret=pick:0/1/2 args=v
  stub 0219f01c force ret=u32 args=v,p,v,p,v  # ov21
  stub 02162a54 force ret=pick:0xFFFF/0xFFFF/0xFFFF/0xFFFF/1 args=v,v,in:12,v
  stub 02158768 force ret=u32 args=v,v,v,v  # ov10
  stub 0218b56c ret=pick:0xFFFF/0xFFFF/0xFFFF/0x2712/0x2716 args=p  # in-area
  stub 02012ebc force ret=u32 args=v  # main
  stub 020071cc force ret=u32 args=v  # main
  stub 02188c9c force ret=u16 args=v  # fieldmap (other F2a area)
  stub 021cfac0 force ret=pick:0xFF/0xFF/0xFF/0xFF/0/1 args=v,v,v,v,v
  stub 021cfc14 force ret=u32 args=v,p,v  # ov21
  stub 021cd440 force ret=pick:0/0/0/0/1/2 args=v
  stub 02013ffc force ret=pick:0/1 args=v
  stub 02162940 force ret=pick:0xFFFF/0xFFFF/0xFFFF/1 args=v,v,in:12
  stub 02188c84 force ret=u16 args=v  # fieldmap (other F2a area)
  stub 020120f4 force ret=u32 args=v  # main
  stub 02012844 force ret=u32 args=v  # main
  stub 020362dc force ret=pick:0/1/0x400/0x800/0x401/0xC01 args=
  stub 0203630c force ret=pick:0/0x10/0x20/0x40/0x80/0xF0 args=
  stub 02188cb4 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021a2fe4 force ret=none args=v  # ov21
  stub 021a3080 force ret=pick:0/1/2/3 args=v
  stub 021a308c force ret=pick:0/1/2/3 args=v
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v
  stub 0216d888 force ret=u32 args=v  # ov10
  stub 02012b38 force ret=u32 args=v  # main
  stub 02014478 force ret=u32 args=v  # main
  stub 021a30c8 force ret=u32 args=v  # ov21
  stub 021a3584 force ret=pick:0/1 args=v
  stub 02014474 force ret=none args=v,v  # main
  stub 02158b00 force ret=pick:0/0/0/1 args=v,v
  stub 02012f2c force ret=u32 args=v  # main
  stub 02008dac force ret=u32 args=v,v,v  # main
  stub 0218a8bc ret=pick:0/0/0/0/0/0/1 args=v  # in-area
  stub 0218a784 ret=pick:0/0/0/0/0/0/1 args=p,v  # in-area
  stub 0218ae04 ret=pick:0/1/0xFFFF args=v,v
  stub 021625cc force ret=u32 args=v,v  # ov10
  stub 021626ec force ret=pick:0/1 args=v
  stub 0218aeac ret=pick:0/1 args=v  # in-area
  stub 0218adc8 ret=none args=v,v,v  # in-area
  stub 0218ad80 ret=pick:0/0/0/0/0/0/1 args=v,v,v,v  # in-area
  stub 02188c98 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021ab0d4 force ret=u32 args=v,in:12
  stub 021ab0f0 force ret=u32 args=v  # ov21
  stub 021c89a8 force ret=pick:0/1 args=v  # ov21
  stub 021c8a30 force ret=u32 args=v,v,v  # ov21
  stub 021c8bc4 force ret=pick:0/1 args=v  # ov21
  stub 021c8c88 force ret=u32 args=v,v,v  # ov21
  stub 02188c80 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 0218ac84 ret=pick:0/0/0/0/0/0/1 args=v,p,v  # in-area
  stub 0218ab9c ret=none args=v,v  # in-area
  stub 0218ac08 ret=pick:0/0/0/0/0/0/1 args=v,v,v  # in-area
  stub 0218ab50 ret=pick:0/0/0/0/0/0/1 args=p,v,v,v  # in-area
  stub 0218ac4c ret=pick:0/0/0/0/0/0/1 args=v,v,v  # in-area
  stub 021a2fa4 force ret=pick:0/1/2/3/9 args=v,v
  stub 02013d5c force ret=pick:0/1 args=v
  stub 0217b4d8 force ret=u32 args=v,v,v,v,v  # ov18
  stub 02188ce8 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 02013dd0 force ret=pick:0/1 args=v
  stub 0218dc70 force ret=u32 args=v  # ov21
  stub 021aa248 force ret=u32 args=v,v  # ov21
  stub 021a9f40 force ret=u32 args=v,v  # ov21
  stub 021a3098 force ret=pick:0/1/2/2 args=v
  stub 021a3364 force ret=none args=v,v,out:12  # position in direction d (player, d, &out)
  stub 021ab0f8 force ret=pick:0/1/2/4 args=v
  stub 021ab2e8 force ret=pick:0/1 args=v
  stub 0218b6f8 ret=pick:0/1 args=v,v  # in-area
  stub 0218b3dc ret=pick:0/1 args=p,v,v,v,v  # in-area
  stub 0218ae48 ret=none args=v,out:12  # in-area
  stub 021626e8 force ret=pick:0/1/5/6/7 args=v
  stub 021be288 force ret=pick:0/0/0/1 args=v,v
  stub 021f3a88 force ret=u32 args=v,v,v
  stub 021f38a4 force ret=u32 args=v,v
  stub 0218b630 ret=pick:0/1/6 args=v,v  # in-area
  stub 021ab310 force ret=pick:0/1 args=v
  stub 02163e58 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0x3/0x80
  stub 021f404c force ret=u32 args=v
  stub 021f37e0 force ret=u32 args=v
  stub 02010f18 force ret=pick:0/0x30/0x31 args=
  stub 020124ac force ret=u32 args=v,v,v,v  # main
  stub 021a3460 force ret=none args=v,outm:2:0x7/0,outm:2:0x7/0,outm:2:0x7/0
  stub 02164294 force ret=none args=v,v,v,out:12  # tile -> fx32 position (x, y, z, &out)
  stub 021ab194 force ret=pick:0/1 args=v
  stub 02164240 force ret=none args=v,outm:2:0xFF/0xFF,outm:2:0xFF/0xFF,v  # ov10 (dir, &x, &z, n): moves the s16 tile x / z
  stub 02188c7c force ret=u32 args=v  # fieldmap (other F2a area)
  stub 0216d8ac force ret=u32 args=v  # ov10
  stub 0216dddc force ret=u32 args=v,v,v,v,v,v  # ov10
  stub 021a3128 force ret=u32 args=v,v  # ov21
  stub 02013e04 force ret=pick:0/1 args=v
  stub 021a3504 force ret=pick:0/1 args=v,v,v
  stub 021a3524 force ret=pick:0/1 args=v,v,v
  stub 021ab1a0 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab1ac force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab1b8 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab1d0 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab1dc force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab1e8 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab1f4 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab200 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab20c force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab218 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab224 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab230 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab1c4 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 0201210c force ret=u32 args=v  # main
  stub 021ab188 force ret=pick:0/1 args=v
  stub 0216282c force ret=nullable:4:ptr:work:0x640:1 args=v,v,in:12,v
  stub 0216d7c4 force ret=pick:0/0x58/0x5B/0xA3/0xA6/0xA7 args=v
  stub 02162928 force ret=pick:0xFFFF/0xFFFF/0xFFFF/1 args=v,v,in:12,v
  stub 02162564 force ret=pick:0/1/0xFFFF args=v,in:12
  stub 021626b8 force ret=pick:0/1 args=v,v
  stub 02014110 force ret=none args=out:0x1C,v,v,v,v,v,v,v  # LOCATION init (loc, zone, ..)
  stub 02012878 force ret=none args=v,in:0x1C  # store a LOCATION in the game data (gd, &loc)
  stub 02162b68 force ret=u32 args=v,in:12  # ov10
  stub 02185410 force ret=u32 args=v,p,v,v  # ov20
  stub 02185360 force ret=u32 args=v,p,v,v  # ov20
  stub 021aa944 force ret=pick:0/1/2 args=v
  stub 021aa80c force ret=none args=v  # ov21
  stub 02008978 force ret=pick:0/1/0x7E/0x7F/0x80/0xFFFF args=v
  stub 02008980 force ret=u32 args=v,v  # main
  stub 02012934 force ret=u32 args=v  # main
  stub 0201aa18 force ret=pick:0/1/2 args=v
  stub 0201ab28 force ret=u32 args=v,v  # main
  stub 0201b7ec force ret=u32 args=v,v,v,v  # main
  stub 021d220c force ret=u32 args=v,v  # ov21
  stub 02008990 force ret=u32 args=v  # main
  stub 02008988 force ret=none args=v,v  # main
  stub 02188d50 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021c3378 force ret=none args=v  # ov21
  stub 0218aad0 force ret=none args=v,v  # in-area (verified port; callers keep the stub: the callee logs its entry r2 / r3)
  stub 0218ab0c ret=pick:0/1 args=v  # in-area
  stub 0200d564 force ret=pick:0/1/0x100 args=v
  stub 0216d83c force ret=u32 args=v  # ov10
  stub 0216d86c force ret=u32 args=v  # ov10
  stub 021641e8 force ret=u32 args=v  # ov10
  stub 021641f8 force ret=u32 args=v  # ov10
  stub 0216ddc0 force ret=pick:0/0/0/1 args=v,v,v,v
  stub 020122c0 force ret=u32 args=v,v,v,v  # main
  stub 020124e8 force ret=ptr:work:0x700:1 args=v
  stub 02017e1c force ret=pick:0/1/0x39/0x7F args=v,v,v
  stub 02008970 force ret=none args=v,outm:4:0xFF/0xFF/1/0
  stub 02010ff4 force ret=u32 args=v,p,v,v  # main
  stub 0218aa74 ret=none args=v  # in-area
  stub 0200896c force ret=u32 args=v,v  # main
  stub 0218aa2c ret=pick:1/2 args=v  # in-area
  stub 02017e40 force ret=none args=v,v,v,v  # main

# 0218A074(gamesys, fm) (grid entry): f = 0201210C(gamesys); e = 02188CE8(f); 021AA8F8(e); ev = 02189BD0(gamesys, fm, &talked, &flag);
# ev: talked: 021AA8EC(e); 021BC5BC(02188D20(f)); 021A3550(02188CB4(f), flag); 021A356C(02188CB4(f)); 02189A70(f): 021F3B10(..);
# 0218A6E8(gamesys, f). Returns ev
fn 0218a074 ret=u32 inputs=300
  arg u32 any
  arg work 0
  fix work 0x8C u32 any
  fix work 0x90 u32 any
  fix work 0x108 u32 addrof work 0x400
  fix work 0x400 u32 alt range 0 2 || range 0 8
  fix work 0x150 u32 alt const 0 || any
  fix work 0xD8 u16 alt range 0 0x1ff || any
  stub 0201210c force ret=u32 args=v  # main
  stub 02188ce8 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021aaf54 force ret=none args=v  # ov21
  stub 02189bd0 ret=pick:0/0/1/2 args=v,v,outm:4:1/0/0/0,outm:4:1/0/0/0
  stub 021aa8ec force ret=none args=v  # ov21
  stub 02188d20 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021bc5bc force ret=none args=v  # ov21
  stub 02188cb4 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021a3550 force ret=none args=v,v  # ov21
  stub 021a356c force ret=none args=v  # ov21
  stub 02189a70 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021f3b10 force ret=none args=v
  stub 0218a6e8 ret=none args=v,v  # in-area
  stub 0218a4e0 ret=none args=outm:0x40:0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/3/0/0/0/3/0/0/0/3/0/1/0/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/3/0/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/7/0/0,v,v
  stub 0218a738 ret=pick:0/0/0/0/0/0/1 args=p  # in-area
  stub 02012ed0 force ret=ptr:work:0x220:1 args=v  # gamedata -> record whose first word is read
  stub 021c868c force ret=pick:0/0/0/0/0/1 args=v,v
  stub 021ae0cc force ret=pick:0/0/0/0/0/1 args=v
  stub 0218a728 ret=none args=v  # in-area
  stub 0218a76c ret=pick:0/0/0/0/0/0/1 args=v  # in-area
  stub 0218acd0 ret=pick:0/0/0/0/0/0/1 args=v,v,v  # in-area
  stub 0218ad28 ret=pick:0/0/0/0/0/0/1 args=v,v  # in-area
  stub 0218a9b0 ret=pick:0/0/0/0/0/0/1 args=p,v  # in-area
  stub 0218a7f0 ret=pick:0/0/0/0/0/0/1 args=v  # in-area
  stub 0218a824 ret=pick:0/0/0/0/0/0/1 args=v  # in-area
  stub 0218a878 ret=pick:0/0/0/0/0/0/1 args=v,v,v  # in-area
  stub 0218b208 ret=pick:0/0/0/0/0/0/1 args=p,p,v  # in-area
  stub 0201296c force ret=u32 args=v  # main
  stub 020142e8 force ret=pick:0/1/2 args=v,v  # main
  stub 021a2f20 force ret=pick:0/0/0/0/0/1 args=v,v,v,v
  stub 02012138 force ret=u32 args=v  # main
  stub 02158344 force ret=pick:0/1 args=v
  stub 021a3074 force ret=u32 args=v  # ov21
  stub 0217bfd8 force ret=pick:0/0/0/0/0/1/2 args=v,v
  stub 0217ca84 force ret=u32 args=v,v  # ov18
  stub 02026454 force ret=pick:0/1/2/3/4/5/6 args=v
  stub 02012b70 force ret=pick:0/1/2 args=v
  stub 0217c070 force ret=pick:0/0/1/2 args=v
  stub 0217b810 force ret=u32 args=v  # ov18
  stub 0217689c force ret=pick:0/0/0/1/2 args=v,v
  stub 02012fb0 force ret=u32 args=v  # main
  stub 0216b3a8 force ret=u32 args=v,v,v  # ov10
  stub 021754b4 force ret=pick:0/0/0/1/2 args=v,out:4
  stub 021a2f6c force ret=none args=v  # ov21
  stub 0217fc6c force ret=u32 args=v,v,v,v,v  # ov18
  stub 0217bde0 force ret=pick:0/0/0/0/1 args=v,v,v
  stub 0217afc4 force ret=pick:0/0/0/1/2 args=v,v,out:4
  stub 0217fb5c force ret=u32 args=v,v,p,v,v  # ov18
  stub 0218b408 ret=pick:0/0/0/0/0/0/1 args=p,v,v  # in-area
  stub 0218aed4 ret=pick:0/0/0/0/0/0/1 args=p,p,v  # in-area
  stub 0218af90 ret=pick:0/0/0/0/0/0/1 args=p,v,v  # in-area
  stub 020221a4 force ret=pick:0/1/1 args=
  stub 02188ccc force ret=u32 args=v  # fieldmap (other F2a area)
  stub 0215ff34 force ret=pick:0/1 args=v
  stub 021a0ed4 force ret=pick:0/1 args=v
  stub 0215ff54 force ret=pick:0/0/0/1 args=v,v,v
  stub 0201212c force ret=pick:0/1/2 args=v
  stub 021c06cc force ret=u32 args=v  # ov21
  stub 021a0f90 force ret=pick:0/6/6/7 args=v
  stub 0215fc40 force ret=u32 args=v,v,v  # ov10
  stub 021a0eb0 force ret=pick:0/0/0/0/1 args=v,v
  stub 0218aca8 ret=pick:0/0/0/0/0/0/1 args=v  # in-area
  stub 0217b410 force ret=pick:0/0/0/1 args=v,v,v
  stub 021a3050 force ret=u32 args=v  # ov21
  stub 021a3038 force ret=none args=v,outm:12:0/0/3/0/0/0/0/0/0/0/3/0
  stub 02164208 force ret=none args=v,in:12,v
  stub 0218b4b0 ret=pick:0/0/0/0/0/0/1 args=p,v  # in-area
  stub 0216d5b8 force ret=u32 args=v  # ov10
  stub 02158d90 force ret=pick:0/1/2 args=v
  stub 0219f01c force ret=u32 args=v,p,v,p,v  # ov21
  stub 02162a54 force ret=pick:0xFFFF/0xFFFF/0xFFFF/0xFFFF/1 args=v,v,in:12,v
  stub 02158768 force ret=u32 args=v,v,v,v  # ov10
  stub 0218b56c ret=pick:0xFFFF/0xFFFF/0xFFFF/0x2712/0x2716 args=p  # in-area
  stub 02012ebc force ret=u32 args=v  # main
  stub 020071cc force ret=u32 args=v  # main
  stub 02188c9c force ret=u16 args=v  # fieldmap (other F2a area)
  stub 021cfac0 force ret=pick:0xFF/0xFF/0xFF/0xFF/0/1 args=v,v,v,v,v
  stub 021cfc14 force ret=u32 args=v,p,v  # ov21
  stub 021cd440 force ret=pick:0/0/0/0/1/2 args=v
  stub 02013ffc force ret=pick:0/1 args=v
  stub 02162940 force ret=pick:0xFFFF/0xFFFF/0xFFFF/1 args=v,v,in:12
  stub 021a1358 force ret=pick:0/1/2 args=v
  stub 020120f4 force ret=u32 args=v  # main
  stub 02012a30 force ret=u32 args=v  # main
  stub 02028a70 force ret=u32 args=v  # main
  stub 021f2ab8 force ret=pick:0/1 args=v
  stub 021f2ad4 force ret=none args=v
  stub 02188c84 force ret=u16 args=v  # fieldmap (other F2a area)
  stub 02012844 force ret=u32 args=v  # main
  stub 020362dc force ret=pick:0/1/0x400/0x800/0x401/0xC01 args=
  stub 0203630c force ret=pick:0/0x10/0x20/0x40/0x80/0xF0 args=
  stub 021a2fe4 force ret=none args=v  # ov21
  stub 021a3080 force ret=pick:0/1/2/3 args=v
  stub 021a308c force ret=pick:0/1/2/3 args=v
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v
  stub 0216d888 force ret=u32 args=v  # ov10
  stub 02012b38 force ret=u32 args=v  # main
  stub 02014478 force ret=u32 args=v  # main
  stub 021a30c8 force ret=u32 args=v  # ov21
  stub 021a3584 force ret=pick:0/1 args=v
  stub 02014474 force ret=none args=v,v  # main
  stub 02158b00 force ret=pick:0/0/0/1 args=v,v
  stub 02012f2c force ret=u32 args=v  # main
  stub 02008dac force ret=u32 args=v,v,v  # main
  stub 0218a8bc ret=pick:0/0/0/0/0/0/1 args=v  # in-area
  stub 0218a784 ret=pick:0/0/0/0/0/0/1 args=p,v  # in-area
  stub 0218ae04 ret=pick:0/1/0xFFFF args=v,v
  stub 021625cc force ret=u32 args=v,v  # ov10
  stub 021626ec force ret=pick:0/1 args=v
  stub 0218aeac ret=pick:0/1 args=v  # in-area
  stub 0218adc8 ret=none args=v,v,v  # in-area
  stub 0218ad80 ret=pick:0/0/0/0/0/0/1 args=v,v,v,v  # in-area
  stub 02188c98 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021ab0d4 force ret=u32 args=v,in:12
  stub 021ab0f0 force ret=u32 args=v  # ov21
  stub 021c89a8 force ret=pick:0/1 args=v  # ov21
  stub 021c8a30 force ret=u32 args=v,v,v  # ov21
  stub 021c8bc4 force ret=pick:0/1 args=v  # ov21
  stub 021c8c88 force ret=u32 args=v,v,v  # ov21
  stub 02188c80 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 0218ac84 ret=pick:0/0/0/0/0/0/1 args=v,p,v  # in-area
  stub 0218ab9c ret=none args=v,v  # in-area
  stub 0218ac08 ret=pick:0/0/0/0/0/0/1 args=v,v,v  # in-area
  stub 0218ab50 ret=pick:0/0/0/0/0/0/1 args=p,v,v,v  # in-area
  stub 0218ac4c ret=pick:0/0/0/0/0/0/1 args=v,v,v  # in-area
  stub 021a2fa4 force ret=pick:0/1/2/3/9 args=v,v
  stub 02013d5c force ret=pick:0/1 args=v
  stub 0217b4d8 force ret=u32 args=v,v,v,v,v  # ov18
  stub 02013dd0 force ret=pick:0/1 args=v
  stub 0218dc70 force ret=u32 args=v  # ov21
  stub 021aa248 force ret=u32 args=v,v  # ov21
  stub 021a9f40 force ret=u32 args=v,v  # ov21
  stub 021a3098 force ret=pick:0/1/2/2 args=v
  stub 021a3364 force ret=none args=v,v,out:12  # position in direction d (player, d, &out)
  stub 021ab0f8 force ret=pick:0/1/2/4 args=v
  stub 021ab2e8 force ret=pick:0/1 args=v
  stub 0218b6f8 ret=pick:0/1 args=v,v  # in-area
  stub 0218b3dc ret=pick:0/1 args=p,v,v,v,v  # in-area
  stub 0218ae48 ret=none args=v,out:12  # in-area
  stub 021626e8 force ret=pick:0/1/5/6/7 args=v
  stub 021be288 force ret=pick:0/0/0/1 args=v,v
  stub 021f3a88 force ret=u32 args=v,v,v
  stub 021f38a4 force ret=u32 args=v,v
  stub 0218b630 ret=pick:0/1/6 args=v,v  # in-area
  stub 021ab310 force ret=pick:0/1 args=v
  stub 02163e58 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0x3/0x80
  stub 021f404c force ret=u32 args=v
  stub 021f37e0 force ret=u32 args=v
  stub 02010f18 force ret=pick:0/0x30/0x31 args=
  stub 020124ac force ret=u32 args=v,v,v,v  # main
  stub 021a3460 force ret=none args=v,outm:2:0x7/0,outm:2:0x7/0,outm:2:0x7/0
  stub 02164294 force ret=none args=v,v,v,out:12  # tile -> fx32 position (x, y, z, &out)
  stub 021ab194 force ret=pick:0/1 args=v
  stub 02164240 force ret=none args=v,outm:2:0xFF/0xFF,outm:2:0xFF/0xFF,v  # ov10 (dir, &x, &z, n): moves the s16 tile x / z
  stub 02188c7c force ret=u32 args=v  # fieldmap (other F2a area)
  stub 0216d8ac force ret=u32 args=v  # ov10
  stub 0216dddc force ret=u32 args=v,v,v,v,v,v  # ov10
  stub 021a3128 force ret=u32 args=v,v  # ov21
  stub 02013e04 force ret=pick:0/1 args=v
  stub 021a3504 force ret=pick:0/1 args=v,v,v
  stub 021a3524 force ret=pick:0/1 args=v,v,v
  stub 021ab1a0 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab1ac force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab1b8 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab1d0 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab1dc force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab1e8 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab1f4 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab200 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab20c force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab218 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab224 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab230 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab1c4 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab188 force ret=pick:0/1 args=v
  stub 0216282c force ret=nullable:4:ptr:work:0x640:1 args=v,v,in:12,v
  stub 0216d7c4 force ret=pick:0/0x58/0x5B/0xA3/0xA6/0xA7 args=v
  stub 02162928 force ret=pick:0xFFFF/0xFFFF/0xFFFF/1 args=v,v,in:12,v
  stub 02162564 force ret=pick:0/1/0xFFFF args=v,in:12
  stub 021626b8 force ret=pick:0/1 args=v,v
  stub 02014110 force ret=none args=out:0x1C,v,v,v,v,v,v,v  # LOCATION init (loc, zone, ..)
  stub 02012878 force ret=none args=v,in:0x1C  # store a LOCATION in the game data (gd, &loc)
  stub 02162b68 force ret=u32 args=v,in:12  # ov10
  stub 02185410 force ret=u32 args=v,p,v,v  # ov20
  stub 02185360 force ret=u32 args=v,p,v,v  # ov20
  stub 021aa944 force ret=pick:0/1/2 args=v
  stub 021aa80c force ret=none args=v  # ov21
  stub 02008978 force ret=pick:0/1/0x7E/0x7F/0x80/0xFFFF args=v
  stub 02008980 force ret=u32 args=v,v  # main
  stub 02012934 force ret=u32 args=v  # main
  stub 0201aa18 force ret=pick:0/1/2 args=v
  stub 0201ab28 force ret=u32 args=v,v  # main
  stub 0201b7ec force ret=u32 args=v,v,v,v  # main
  stub 021d220c force ret=u32 args=v,v  # ov21
  stub 02008990 force ret=u32 args=v  # main
  stub 02008988 force ret=none args=v,v  # main
  stub 02188d50 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021c3378 force ret=none args=v  # ov21
  stub 0218aad0 force ret=none args=v,v  # in-area (verified port; callers keep the stub: the callee logs its entry r2 / r3)
  stub 0218ab0c ret=pick:0/1 args=v  # in-area
  stub 0200d564 force ret=pick:0/1/0x100 args=v
  stub 0216d83c force ret=u32 args=v  # ov10
  stub 0216d86c force ret=u32 args=v  # ov10
  stub 021641e8 force ret=u32 args=v  # ov10
  stub 021641f8 force ret=u32 args=v  # ov10
  stub 0216ddc0 force ret=pick:0/0/0/1 args=v,v,v,v
  stub 020122c0 force ret=u32 args=v,v,v,v  # main
  stub 020124e8 force ret=ptr:work:0x700:1 args=v
  stub 02017e1c force ret=pick:0/1/0x39/0x7F args=v,v,v
  stub 02008970 force ret=none args=v,outm:4:0xFF/0xFF/1/0
  stub 02010ff4 force ret=u32 args=v,p,v,v  # main
  stub 0218aa74 ret=none args=v  # in-area
  stub 0200896c force ret=u32 args=v,v  # main
  stub 0218aa2c ret=pick:1/2 args=v  # in-area
  stub 02017e40 force ret=none args=v,v,v,v  # main

# 0218A0E4(gamesys, fm) (comm zones): x = 02012138(gamesys); c = 02026474(x); req = 0218A4E0(..); c == 0: 0200FB24() ?
# (020221A4() ? 021C06CC(gamesys, fm) : 0) : 021850FC(gamesys, fm); c != 0: 0217391C(c) == 1 (0217392C(c) == 0: 02026400(x); 0);
# FLAGS bit 2: 0218A784(req, 9); 0217B80C; FLAGS bit 0: 0217B83C; FLAGS bit 0 && 0217394C(gamesys) == 1: 0218B4B0 -> 0216D5B8 /
# 02158D90 / 0216D588 / 0217AF38 -> 0219F01C; 0201212C(gamesys) == 1 && 020221A4(): 021C06CC; 0217394C == 0: 0; FLAGS bit 1 &&
# 020221A4(): 0215FCE0(gamesys, fm, heap); 0
fn 0218a0e4 ret=u32 inputs=1500
  arg u32 any
  arg work 0
  fix work 0x8C u32 any
  fix work 0x90 u32 any
  fix work 0x108 u32 addrof work 0x400
  fix work 0x400 u32 alt range 0 2 || range 0 8
  fix work 0x150 u32 alt const 0 || any
  fix work 0xD8 u16 alt range 0 0x1ff || any
  stub 02012138 force ret=u32 args=v  # main
  stub 02026474 force ret=pick:0/0/1 args=v
  stub 0218a4e0 ret=none args=outm:0x40:0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/3/0/0/0/3/0/0/0/3/0/1/0/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/3/0/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/7/0/0,v,v
  stub 0200fb24 force ret=pick:0/1 args=
  stub 020221a4 force ret=pick:0/1/1 args=
  stub 021c06cc force ret=u32 args=v  # ov21
  stub 021850fc force ret=u32 args=v  # ov20
  stub 0217391c force ret=pick:0/1/2 args=v
  stub 02173928 force ret=pick:0/1 args=v
  stub 02026400 force ret=none args=  # main
  stub 0218a784 ret=pick:0/0/0/0/0/0/1 args=p,v  # in-area
  stub 0217b80c force ret=pick:0/0/0/1 args=v,v
  stub 0217b83c force ret=pick:0/0/0/1 args=v,v
  stub 0217394c force ret=pick:0/1/1/2 args=v
  stub 0218b4b0 ret=pick:0/1 args=p,v
  stub 0216d5b8 force ret=u32 args=v  # ov10
  stub 02158d90 force ret=pick:0/1/2 args=v
  stub 021a3074 force ret=u32 args=v  # ov21
  stub 0216d588 force ret=u32 args=v  # ov10
  stub 0217af38 force ret=pick:0/1 args=v
  stub 021a2f6c force ret=none args=v  # ov21
  stub 0219f01c force ret=u32 args=v,p,v,p,v  # ov21
  stub 0201212c force ret=pick:0/1/2 args=v
  stub 0215fce0 force ret=u32 args=v,v,v  # ov10
  stub 02188c84 force ret=u16 args=v  # fieldmap (other F2a area)
  stub 020120f4 force ret=u32 args=v  # main
  stub 02012844 force ret=u32 args=v  # main
  stub 02188c9c force ret=u16 args=v  # fieldmap (other F2a area)
  stub 020362dc force ret=pick:0/1/0x400/0x800/0x401/0xC01 args=
  stub 0203630c force ret=pick:0/0x10/0x20/0x40/0x80/0xF0 args=
  stub 02188cb4 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021a2fe4 force ret=none args=v  # ov21
  stub 021a3080 force ret=pick:0/1/2/3 args=v
  stub 021a308c force ret=pick:0/1/2/3 args=v
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v
  stub 0216d888 force ret=u32 args=v  # ov10
  stub 02012b38 force ret=u32 args=v  # main
  stub 02014478 force ret=u32 args=v  # main
  stub 021a30c8 force ret=u32 args=v  # ov21
  stub 021a3584 force ret=pick:0/1 args=v
  stub 0201210c force ret=u32 args=v  # main
  stub 0201296c force ret=u32 args=v  # main
  stub 021a3038 force ret=none args=v,out:12
  stub 02188c98 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021ab0d4 force ret=u32 args=v,in:12
  stub 021ab0f0 force ret=u32 args=v  # ov21
  stub 021ab188 force ret=pick:0/0/1 args=v
  stub 02162928 force ret=pick:0xFFFF/0xFFFF/0xFFFF/1 args=v,v,in:12,v
  stub 02158768 force ret=u32 args=v,v,v,v  # ov10
  stub 021a3460 force ret=none args=v,outm:2:0x7/0,outm:2:0x7/0,outm:2:0x7/0
  stub 02164294 force ret=none args=v,v,v,out:12  # tile -> fx32 position (x, y, z, &out)
  stub 021ab194 force ret=pick:0/1 args=v
  stub 02164240 force ret=none args=v,outm:2:0xFF/0xFF,outm:2:0xFF/0xFF,v  # ov10 (dir, &x, &z, n): moves the s16 tile x / z
  stub 02188c7c force ret=u32 args=v  # fieldmap (other F2a area)
  stub 0216d8ac force ret=u32 args=v  # ov10
  stub 0216dddc force ret=u32 args=v,v,v,v,v,v  # ov10

# 0218A240(gamesys, fm) (rail entry): ev = 0218A278(gamesys, fm); ev: f = 0201210C(gamesys); 021BC5BC(02188D20(f)); 02189A70(f):
# 021F3B10(..); 0218A6E8(gamesys, f). Returns ev
fn 0218a240 ret=u32 inputs=200
  arg u32 any
  arg work 0
  fix work 0x8C u32 any
  fix work 0x90 u32 any
  fix work 0x108 u32 addrof work 0x400
  fix work 0x400 u32 alt range 0 2 || range 0 8
  fix work 0x150 u32 alt const 0 || any
  fix work 0xD8 u16 alt range 0 0x1ff || any
  stub 0218a278 ret=pick:0/0/1/2 args=v,v,p,v  # in-area
  stub 0201210c force ret=u32 args=v  # main
  stub 02188d20 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021bc5bc force ret=none args=v  # ov21
  stub 02189a70 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021f3b10 force ret=none args=v
  stub 0218a6e8 ret=none args=v,v  # in-area
  stub 0218a4e0 ret=none args=outm:0x40:0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/3/0/0/0/3/0/0/0/3/0/1/0/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/3/0/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/7/0/0,v,v
  stub 021a3074 force ret=u32 args=v  # ov21
  stub 0219e634 force ret=pick:0/1 args=v,out:8
  stub 0218a738 ret=pick:0/0/0/0/0/0/1 args=p  # in-area
  stub 0218a728 ret=none args=v  # in-area
  stub 0201296c force ret=u32 args=v  # main
  stub 0219e2f4 force ret=none args=v,outm:8:0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF
  stub 02162958 force ret=pick:0xFFFF/0xFFFF/0xFFFF/1 args=v,v,in:8
  stub 02158768 force ret=u32 args=v,v,v,v  # ov10
  stub 0218b0cc ret=pick:0/0/0/0/0/0/1 args=p,p,v  # in-area
  stub 0218a9b0 ret=pick:0/0/0/0/0/0/1 args=p,v  # in-area
  stub 0218b208 ret=pick:0/0/0/0/0/0/1 args=p,p,v  # in-area
  stub 0218b53c ret=pick:0/0/0/0/0/0/1 args=p,v  # in-area
  stub 0216d5b8 force ret=u32 args=v  # ov10
  stub 02158d90 force ret=pick:0/1/2 args=v
  stub 021a2f6c force ret=none args=v  # ov21
  stub 0219f01c force ret=u32 args=v,p,v,p,v  # ov21
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v
  stub 02162a5c force ret=pick:0xFFFF/0xFFFF/0xFFFF/1 args=v,v,in:8,v
  stub 02188cb4 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 0218b310 ret=pick:0/1 args=v,p  # in-area
  stub 0218b13c ret=pick:0/0/0/0/0/0/1 args=p,p,v,p  # in-area
  stub 0218b24c ret=pick:0/0/0/0/0/0/1 args=p,v,v  # in-area
  stub 020221a4 force ret=pick:0/1/1 args=
  stub 02188ccc force ret=u32 args=v  # fieldmap (other F2a area)
  stub 0215ff34 force ret=pick:0/1 args=v
  stub 021a0ed4 force ret=pick:0/1 args=v
  stub 0215ff54 force ret=pick:0/0/0/1 args=v,v,v
  stub 0201212c force ret=pick:0/1/2 args=v
  stub 021c06cc force ret=u32 args=v  # ov21
  stub 021a0f90 force ret=pick:0/6/6/7 args=v
  stub 0215fc40 force ret=u32 args=v,v,v  # ov10
  stub 021a0eb0 force ret=pick:0/0/0/0/1 args=v,v
  stub 0218aca8 ret=pick:0/0/0/0/0/0/1 args=v  # in-area
  stub 021a1358 force ret=pick:0/1/2 args=v
  stub 020120f4 force ret=u32 args=v  # main
  stub 02012a30 force ret=u32 args=v  # main
  stub 02028a70 force ret=u32 args=v  # main
  stub 021f2ab8 force ret=pick:0/1 args=v
  stub 021f2ad4 force ret=none args=v
  stub 02188c84 force ret=u16 args=v  # fieldmap (other F2a area)
  stub 02012844 force ret=u32 args=v  # main
  stub 02188c9c force ret=u16 args=v  # fieldmap (other F2a area)
  stub 020362dc force ret=pick:0/1/0x400/0x800/0x401/0xC01 args=
  stub 0203630c force ret=pick:0/0x10/0x20/0x40/0x80/0xF0 args=
  stub 021a2fe4 force ret=none args=v  # ov21
  stub 021a3080 force ret=pick:0/1/2/3 args=v
  stub 021a308c force ret=pick:0/1/2/3 args=v
  stub 0216d888 force ret=u32 args=v  # ov10
  stub 02012b38 force ret=u32 args=v  # main
  stub 02014478 force ret=u32 args=v  # main
  stub 021a30c8 force ret=u32 args=v  # ov21
  stub 021a3584 force ret=pick:0/1 args=v
  stub 02014474 force ret=none args=v,v  # main
  stub 02158b00 force ret=pick:0/0/0/1 args=v,v
  stub 02012f2c force ret=u32 args=v  # main
  stub 02008dac force ret=u32 args=v,v,v  # main
  stub 02162598 force ret=pick:0/1/0xFFFF args=v,in:8
  stub 021625cc force ret=u32 args=v,v  # ov10
  stub 021626ec force ret=pick:0/1 args=v
  stub 0218aeac ret=pick:0/1 args=v  # in-area
  stub 0218b2a8 ret=none args=v,v,p  # in-area
  stub 0218b2e8 ret=pick:0/0/0/0/0/0/1 args=v,v,v,p  # in-area
  stub 02188c80 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 0218ac84 ret=pick:0/0/0/0/0/0/1 args=v,p,v  # in-area
  stub 0218ab9c ret=none args=v,v  # in-area
  stub 0218ac08 ret=pick:0/0/0/0/0/0/1 args=v,v,v  # in-area
  stub 0218ab50 ret=pick:0/0/0/0/0/0/1 args=p,v,v,v  # in-area
  stub 0218ac4c ret=pick:0/0/0/0/0/0/1 args=v,v,v  # in-area
  stub 02188ce8 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 02013dd0 force ret=pick:0/1 args=v
  stub 02188c98 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 0218dc70 force ret=u32 args=v  # ov21
  stub 021aa248 force ret=u32 args=v,v  # ov21
  stub 021a9f40 force ret=u32 args=v,v  # ov21
  stub 02188c7c force ret=u32 args=v  # fieldmap (other F2a area)
  stub 0219e570 force ret=u32 args=v,in:8,v  # ov21 (sys, &rail position, flag)
  stub 0219e424 force ret=u32 args=v,v,v  # ov21
  stub 021626b8 force ret=pick:0/1 args=v,v
  stub 021626e8 force ret=pick:0/1/5/6/7 args=v
  stub 021a2fa4 force ret=pick:0/1/2/9 args=v,v
  stub 021ab0f0 force ret=u32 args=v  # ov21
  stub 021bb188 force ret=pick:0/1 args=v
  stub 021c46f8 force ret=u32 args=v,v,v  # ov21
  stub 02010f18 force ret=pick:0/0x30/0x31 args=
  stub 020124ac force ret=u32 args=v,v,v,v  # main
  stub 02014134 force ret=none args=out:0x1C,v,v,v,v,v,v,v  # LOCATION init, rail form (loc, id, b, c, d, e, f, g)
  stub 02012878 force ret=none args=v,in:0x1C  # store a LOCATION in the game data (gd, &loc)
  stub 02162bcc force ret=u32 args=v,in:8  # ov10
  stub 02185360 force ret=u32 args=v,p,v,v  # ov20
  stub 02012ebc force ret=u32 args=v  # main
  stub 021aa944 force ret=pick:0/1/2 args=v
  stub 021aa80c force ret=none args=v  # ov21
  stub 020071cc force ret=u32 args=v  # main
  stub 02008978 force ret=pick:0/1/0x7E/0x7F/0x80/0xFFFF args=v
  stub 02008980 force ret=u32 args=v,v  # main
  stub 02012934 force ret=u32 args=v  # main
  stub 0201aa18 force ret=pick:0/1/2 args=v
  stub 0201ab28 force ret=u32 args=v,v  # main
  stub 0201b7ec force ret=u32 args=v,v,v,v  # main
  stub 02013ffc force ret=pick:0/1 args=v
  stub 021d220c force ret=u32 args=v,v  # ov21
  stub 02008990 force ret=u32 args=v  # main
  stub 02008988 force ret=none args=v,v  # main
  stub 02188d50 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021c3378 force ret=none args=v  # ov21
  stub 0218aad0 force ret=none args=v,v  # in-area (verified port; callers keep the stub: the callee logs its entry r2 / r3)
  stub 0218ab0c ret=pick:0/1 args=v  # in-area
  stub 0200d564 force ret=pick:0/1/0x100 args=v
  stub 02008970 force ret=none args=v,outm:4:0xFF/0xFF/1/0
  stub 02010ff4 force ret=u32 args=v,p,v,v  # main
  stub 0218aa74 ret=none args=v  # in-area
  stub 0200896c force ret=u32 args=v,v  # main
  stub 02017e1c force ret=pick:0/1 args=v,v,v
  stub 0218aa2c ret=pick:1/2 args=v  # in-area
  stub 02017e40 force ret=none args=v,v,v,v  # main

# 0218A278(gamesys, fm) (rail checks): req = 0218A4E0(..); 0219E634(021A3074(player), &rp); 0218A738; FLAGS bit 2: 0218A728,
# 02162958(evdata, ev, rail position of 0219E2F4) -> 02158768, 0218B0CC, 0218A9B0; 0218B208; FLAGS bit 0: 0218B53C (+ 0216D5B8 /
# 02158D90 -> 0219F01C), 02162A5C(evdata, ev, rail pos, 0216D5F8(mmdl)) -> 02158768; FLAGS bit 5: 0218B310 / 0218B13C; 0218B24C;
# FLAGS bit 6/10/1/7 comm and menu checks as in 02189BD0 (021A0EB0 with bit 7 != 0); returns the GMEVENT or 0
fn 0218a278 ret=u32 inputs=500
  arg u32 any
  arg work 0
  fix work 0x8C u32 any
  fix work 0x90 u32 any
  fix work 0x108 u32 addrof work 0x400
  fix work 0x400 u32 alt range 0 2 || range 0 8
  fix work 0x150 u32 alt const 0 || any
  fix work 0xD8 u16 alt range 0 0x1ff || any
  stub 0218a4e0 ret=none args=outm:0x40:0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/3/0/0/0/3/0/0/0/3/0/1/0/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/3/0/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/7/0/0,v,v
  stub 021a3074 force ret=u32 args=v  # ov21
  stub 0219e634 force ret=pick:0/1 args=v,out:8
  stub 0218a738 ret=pick:0/0/0/0/0/0/1 args=p  # in-area
  stub 0218a728 ret=none args=v  # in-area
  stub 0201296c force ret=u32 args=v  # main
  stub 0219e2f4 force ret=none args=v,outm:8:0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF
  stub 02162958 force ret=pick:0xFFFF/0xFFFF/0xFFFF/1 args=v,v,in:8
  stub 02158768 force ret=u32 args=v,v,v,v  # ov10
  stub 0218b0cc ret=pick:0/0/0/0/0/0/1 args=p,p,v  # in-area
  stub 0218a9b0 ret=pick:0/0/0/0/0/0/1 args=p,v  # in-area
  stub 0218b208 ret=pick:0/0/0/0/0/0/1 args=p,p,v  # in-area
  stub 0218b53c ret=pick:0/0/0/0/0/0/1 args=p,v  # in-area
  stub 0216d5b8 force ret=u32 args=v  # ov10
  stub 02158d90 force ret=pick:0/1/2 args=v
  stub 021a2f6c force ret=none args=v  # ov21
  stub 0219f01c force ret=u32 args=v,p,v,p,v  # ov21
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v
  stub 02162a5c force ret=pick:0xFFFF/0xFFFF/0xFFFF/1 args=v,v,in:8,v
  stub 02188cb4 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 0218b310 ret=pick:0/1 args=v,p  # in-area
  stub 0218b13c ret=pick:0/0/0/0/0/0/1 args=p,p,v,p  # in-area
  stub 0218b24c ret=pick:0/0/0/0/0/0/1 args=p,v,v  # in-area
  stub 020221a4 force ret=pick:0/1/1 args=
  stub 02188ccc force ret=u32 args=v  # fieldmap (other F2a area)
  stub 0215ff34 force ret=pick:0/1 args=v
  stub 021a0ed4 force ret=pick:0/1 args=v
  stub 0215ff54 force ret=pick:0/0/0/1 args=v,v,v
  stub 0201212c force ret=pick:0/1/2 args=v
  stub 021c06cc force ret=u32 args=v  # ov21
  stub 021a0f90 force ret=pick:0/6/6/7 args=v
  stub 0215fc40 force ret=u32 args=v,v,v  # ov10
  stub 021a0eb0 force ret=pick:0/0/0/0/1 args=v,v
  stub 0218aca8 ret=pick:0/0/0/0/0/0/1 args=v  # in-area
  stub 02188c84 force ret=u16 args=v  # fieldmap (other F2a area)
  stub 020120f4 force ret=u32 args=v  # main
  stub 02012844 force ret=u32 args=v  # main
  stub 02188c9c force ret=u16 args=v  # fieldmap (other F2a area)
  stub 020362dc force ret=pick:0/1/0x400/0x800/0x401/0xC01 args=
  stub 0203630c force ret=pick:0/0x10/0x20/0x40/0x80/0xF0 args=
  stub 021a2fe4 force ret=none args=v  # ov21
  stub 021a3080 force ret=pick:0/1/2/3 args=v
  stub 021a308c force ret=pick:0/1/2/3 args=v
  stub 0216d888 force ret=u32 args=v  # ov10
  stub 02012b38 force ret=u32 args=v  # main
  stub 02014478 force ret=u32 args=v  # main
  stub 021a30c8 force ret=u32 args=v  # ov21
  stub 021a3584 force ret=pick:0/1 args=v
  stub 02014474 force ret=none args=v,v  # main
  stub 02158b00 force ret=pick:0/0/0/1 args=v,v
  stub 02012f2c force ret=u32 args=v  # main
  stub 02008dac force ret=u32 args=v,v,v  # main
  stub 02162598 force ret=pick:0/1/0xFFFF args=v,in:8
  stub 021625cc force ret=u32 args=v,v  # ov10
  stub 021626ec force ret=pick:0/1 args=v
  stub 0218aeac ret=pick:0/1 args=v  # in-area
  stub 0218b2a8 ret=none args=v,v,p  # in-area
  stub 0218b2e8 ret=pick:0/0/0/0/0/0/1 args=v,v,v,p  # in-area
  stub 02188c80 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 0218ac84 ret=pick:0/0/0/0/0/0/1 args=v,p,v  # in-area
  stub 0218ab9c ret=none args=v,v  # in-area
  stub 0218ac08 ret=pick:0/0/0/0/0/0/1 args=v,v,v  # in-area
  stub 0218ab50 ret=pick:0/0/0/0/0/0/1 args=p,v,v,v  # in-area
  stub 0218ac4c ret=pick:0/0/0/0/0/0/1 args=v,v,v  # in-area
  stub 02188ce8 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 02013dd0 force ret=pick:0/1 args=v
  stub 02188c98 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 0218dc70 force ret=u32 args=v  # ov21
  stub 021aa248 force ret=u32 args=v,v  # ov21
  stub 021a9f40 force ret=u32 args=v,v  # ov21
  stub 02188c7c force ret=u32 args=v  # fieldmap (other F2a area)
  stub 0219e570 force ret=u32 args=v,in:8,v  # ov21 (sys, &rail position, flag)
  stub 0219e424 force ret=u32 args=v,v,v  # ov21
  stub 021626b8 force ret=pick:0/1 args=v,v
  stub 021626e8 force ret=pick:0/1/5/6/7 args=v
  stub 021a2fa4 force ret=pick:0/1/2/9 args=v,v
  stub 021ab0f0 force ret=u32 args=v  # ov21
  stub 021bb188 force ret=pick:0/1 args=v
  stub 021c46f8 force ret=u32 args=v,v,v  # ov21
  stub 02010f18 force ret=pick:0/0x30/0x31 args=
  stub 020124ac force ret=u32 args=v,v,v,v  # main
  stub 02014134 force ret=none args=out:0x1C,v,v,v,v,v,v,v  # LOCATION init, rail form (loc, id, b, c, d, e, f, g)
  stub 02012878 force ret=none args=v,in:0x1C  # store a LOCATION in the game data (gd, &loc)
  stub 02162bcc force ret=u32 args=v,in:8  # ov10
  stub 02185360 force ret=u32 args=v,p,v,v  # ov20
  stub 02012ebc force ret=u32 args=v  # main
  stub 021aa944 force ret=pick:0/1/2 args=v
  stub 021aa80c force ret=none args=v  # ov21
  stub 020071cc force ret=u32 args=v  # main
  stub 02008978 force ret=pick:0/1/0x7E/0x7F/0x80/0xFFFF args=v
  stub 02008980 force ret=u32 args=v,v  # main
  stub 02012934 force ret=u32 args=v  # main
  stub 0201aa18 force ret=pick:0/1/2 args=v
  stub 0201ab28 force ret=u32 args=v,v  # main
  stub 0201b7ec force ret=u32 args=v,v,v,v  # main
  stub 02013ffc force ret=pick:0/1 args=v
  stub 021d220c force ret=u32 args=v,v  # ov21
  stub 02008990 force ret=u32 args=v  # main
  stub 02008988 force ret=none args=v,v  # main
  stub 02188d50 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021c3378 force ret=none args=v  # ov21
  stub 0218aad0 force ret=none args=v,v  # in-area (verified port; callers keep the stub: the callee logs its entry r2 / r3)
  stub 0218ab0c ret=pick:0/1 args=v  # in-area
  stub 0200d564 force ret=pick:0/1/0x100 args=v
  stub 02008970 force ret=none args=v,outm:4:0xFF/0xFF/1/0
  stub 02010ff4 force ret=u32 args=v,p,v,v  # main
  stub 0218aa74 ret=none args=v  # in-area
  stub 0200896c force ret=u32 args=v,v  # main
  stub 02017e1c force ret=pick:0/1 args=v,v,v
  stub 0218aa2c ret=pick:1/2 args=v  # in-area
  stub 02017e40 force ret=none args=v,v,v,v  # main

# 0218A4BC(gamesys, fm) (hybrid entry): 02188CFC(fm) == 0 ? 0218A074(gamesys, fm) : 0218A240(gamesys, fm)
fn 0218a4bc ret=u32 inputs=200
  arg u32 any
  arg work 0
  fix work 0x8C u32 any
  fix work 0x90 u32 any
  fix work 0x108 u32 addrof work 0x400
  fix work 0x400 u32 alt range 0 2 || range 0 8
  fix work 0x150 u32 alt const 0 || any
  fix work 0xD8 u16 alt range 0 0x1ff || any
  stub 02188cfc force ret=pick:0/1/2/3 args=v  # fieldmap (other F2a area)
  stub 0218a074 ret=u32 args=v,v  # in-area
  stub 0218a240 ret=u32 args=v,v,p,v  # in-area
  stub 0201210c force ret=u32 args=v  # main
  stub 02188ce8 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021aaf54 force ret=none args=v  # ov21
  stub 02189bd0 ret=pick:0/0/1/2 args=v,v,outm:4:1/0/0/0,outm:4:1/0/0/0
  stub 021aa8ec force ret=none args=v  # ov21
  stub 02188d20 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021bc5bc force ret=none args=v  # ov21
  stub 02188cb4 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021a3550 force ret=none args=v,v  # ov21
  stub 021a356c force ret=none args=v  # ov21
  stub 02189a70 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021f3b10 force ret=none args=v
  stub 0218a6e8 ret=none args=v,v  # in-area
  stub 0218a278 ret=pick:0/0/1/2 args=v,v,p,v  # in-area
  stub 0218a4e0 ret=none args=outm:0x40:0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/3/0/0/0/3/0/0/0/3/0/1/0/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/3/0/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/7/0/0,v,v
  stub 0218a738 ret=pick:0/0/0/0/0/0/1 args=p  # in-area
  stub 02012ed0 force ret=ptr:work:0x220:1 args=v  # gamedata -> record whose first word is read
  stub 021c868c force ret=pick:0/0/0/0/0/1 args=v,v
  stub 021ae0cc force ret=pick:0/0/0/0/0/1 args=v
  stub 0218a728 ret=none args=v  # in-area
  stub 0218a76c ret=pick:0/0/0/0/0/0/1 args=v  # in-area
  stub 0218acd0 ret=pick:0/0/0/0/0/0/1 args=v,v,v  # in-area
  stub 0218ad28 ret=pick:0/0/0/0/0/0/1 args=v,v  # in-area
  stub 0218a9b0 ret=pick:0/0/0/0/0/0/1 args=p,v  # in-area
  stub 0218a7f0 ret=pick:0/0/0/0/0/0/1 args=v  # in-area
  stub 0218a824 ret=pick:0/0/0/0/0/0/1 args=v  # in-area
  stub 0218a878 ret=pick:0/0/0/0/0/0/1 args=v,v,v  # in-area
  stub 0218b208 ret=pick:0/0/0/0/0/0/1 args=p,p,v  # in-area
  stub 0201296c force ret=u32 args=v  # main
  stub 020142e8 force ret=pick:0/1/2 args=v,v  # main
  stub 021a2f20 force ret=pick:0/0/0/0/0/1 args=v,v,v,v
  stub 02012138 force ret=u32 args=v  # main
  stub 02158344 force ret=pick:0/1 args=v
  stub 021a3074 force ret=u32 args=v  # ov21
  stub 0217bfd8 force ret=pick:0/0/0/0/0/1/2 args=v,v
  stub 0217ca84 force ret=u32 args=v,v  # ov18
  stub 02026454 force ret=pick:0/1/2/3/4/5/6 args=v
  stub 02012b70 force ret=pick:0/1/2 args=v
  stub 0217c070 force ret=pick:0/0/1/2 args=v
  stub 0217b810 force ret=u32 args=v  # ov18
  stub 0217689c force ret=pick:0/0/0/1/2 args=v,v
  stub 02012fb0 force ret=u32 args=v  # main
  stub 0216b3a8 force ret=u32 args=v,v,v  # ov10
  stub 021754b4 force ret=pick:0/0/0/1/2 args=v,out:4
  stub 021a2f6c force ret=none args=v  # ov21
  stub 0217fc6c force ret=u32 args=v,v,v,v,v  # ov18
  stub 0217bde0 force ret=pick:0/0/0/0/1 args=v,v,v
  stub 0217afc4 force ret=pick:0/0/0/1/2 args=v,v,out:4
  stub 0217fb5c force ret=u32 args=v,v,p,v,v  # ov18
  stub 0218b408 ret=pick:0/0/0/0/0/0/1 args=p,v,v  # in-area
  stub 0218aed4 ret=pick:0/0/0/0/0/0/1 args=p,p,v  # in-area
  stub 0218af90 ret=pick:0/0/0/0/0/0/1 args=p,v,v  # in-area
  stub 020221a4 force ret=pick:0/1/1 args=
  stub 02188ccc force ret=u32 args=v  # fieldmap (other F2a area)
  stub 0215ff34 force ret=pick:0/1 args=v
  stub 021a0ed4 force ret=pick:0/1 args=v
  stub 0215ff54 force ret=pick:0/0/0/1 args=v,v,v
  stub 0201212c force ret=pick:0/1/2 args=v
  stub 021c06cc force ret=u32 args=v  # ov21
  stub 021a0f90 force ret=pick:0/6/6/7 args=v
  stub 0215fc40 force ret=u32 args=v,v,v  # ov10
  stub 021a0eb0 force ret=pick:0/0/0/0/1 args=v,v
  stub 0218aca8 ret=pick:0/0/0/0/0/0/1 args=v  # in-area
  stub 0217b410 force ret=pick:0/0/0/1 args=v,v,v
  stub 021a3050 force ret=u32 args=v  # ov21
  stub 021a3038 force ret=none args=v,outm:12:0/0/3/0/0/0/0/0/0/0/3/0
  stub 02164208 force ret=none args=v,in:12,v
  stub 0218b4b0 ret=pick:0/0/0/0/0/0/1 args=p,v  # in-area
  stub 0216d5b8 force ret=u32 args=v  # ov10
  stub 02158d90 force ret=pick:0/1/2 args=v
  stub 0219f01c force ret=u32 args=v,p,v,p,v  # ov21
  stub 02162a54 force ret=pick:0xFFFF/0xFFFF/0xFFFF/0xFFFF/1 args=v,v,in:12,v
  stub 02158768 force ret=u32 args=v,v,v,v  # ov10
  stub 0218b56c ret=pick:0xFFFF/0xFFFF/0xFFFF/0x2712/0x2716 args=p  # in-area
  stub 02012ebc force ret=u32 args=v  # main
  stub 020071cc force ret=u32 args=v  # main
  stub 02188c9c force ret=u16 args=v  # fieldmap (other F2a area)
  stub 021cfac0 force ret=pick:0xFF/0xFF/0xFF/0xFF/0/1 args=v,v,v,v,v
  stub 021cfc14 force ret=u32 args=v,p,v  # ov21
  stub 021cd440 force ret=pick:0/0/0/0/1/2 args=v
  stub 02013ffc force ret=pick:0/1 args=v
  stub 02162940 force ret=pick:0xFFFF/0xFFFF/0xFFFF/1 args=v,v,in:12
  stub 021a1358 force ret=pick:0/1/2 args=v
  stub 020120f4 force ret=u32 args=v  # main
  stub 02012a30 force ret=u32 args=v  # main
  stub 02028a70 force ret=u32 args=v  # main
  stub 021f2ab8 force ret=pick:0/1 args=v
  stub 021f2ad4 force ret=none args=v
  stub 0219e634 force ret=pick:0/1 args=v,out:8
  stub 0219e2f4 force ret=none args=v,outm:8:0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF
  stub 02162958 force ret=pick:0xFFFF/0xFFFF/0xFFFF/1 args=v,v,in:8
  stub 0218b0cc ret=pick:0/0/0/0/0/0/1 args=p,p,v  # in-area
  stub 0218b53c ret=pick:0/0/0/0/0/0/1 args=p,v  # in-area
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v
  stub 02162a5c force ret=pick:0xFFFF/0xFFFF/0xFFFF/1 args=v,v,in:8,v
  stub 0218b310 ret=pick:0/1 args=v,p  # in-area
  stub 0218b13c ret=pick:0/0/0/0/0/0/1 args=p,p,v,p  # in-area
  stub 0218b24c ret=pick:0/0/0/0/0/0/1 args=p,v,v  # in-area
  stub 02188c84 force ret=u16 args=v  # fieldmap (other F2a area)
  stub 02012844 force ret=u32 args=v  # main
  stub 020362dc force ret=pick:0/1/0x400/0x800/0x401/0xC01 args=
  stub 0203630c force ret=pick:0/0x10/0x20/0x40/0x80/0xF0 args=
  stub 021a2fe4 force ret=none args=v  # ov21
  stub 021a3080 force ret=pick:0/1/2/3 args=v
  stub 021a308c force ret=pick:0/1/2/3 args=v
  stub 0216d888 force ret=u32 args=v  # ov10
  stub 02012b38 force ret=u32 args=v  # main
  stub 02014478 force ret=u32 args=v  # main
  stub 021a30c8 force ret=u32 args=v  # ov21
  stub 021a3584 force ret=pick:0/1 args=v
  stub 02014474 force ret=none args=v,v  # main
  stub 02158b00 force ret=pick:0/0/0/1 args=v,v
  stub 02012f2c force ret=u32 args=v  # main
  stub 02008dac force ret=u32 args=v,v,v  # main
  stub 0218a8bc ret=pick:0/0/0/0/0/0/1 args=v  # in-area
  stub 0218a784 ret=pick:0/0/0/0/0/0/1 args=p,v  # in-area
  stub 0218ae04 ret=pick:0/1/0xFFFF args=v,v
  stub 021625cc force ret=u32 args=v,v  # ov10
  stub 021626ec force ret=pick:0/1 args=v
  stub 0218aeac ret=pick:0/1 args=v  # in-area
  stub 0218adc8 ret=none args=v,v,v  # in-area
  stub 0218ad80 ret=pick:0/0/0/0/0/0/1 args=v,v,v,v  # in-area
  stub 02188c98 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021ab0d4 force ret=u32 args=v,in:12
  stub 021ab0f0 force ret=u32 args=v  # ov21
  stub 021c89a8 force ret=pick:0/1 args=v  # ov21
  stub 021c8a30 force ret=u32 args=v,v,v  # ov21
  stub 021c8bc4 force ret=pick:0/1 args=v  # ov21
  stub 021c8c88 force ret=u32 args=v,v,v  # ov21
  stub 02188c80 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 0218ac84 ret=pick:0/0/0/0/0/0/1 args=v,p,v  # in-area
  stub 0218ab9c ret=none args=v,v  # in-area
  stub 0218ac08 ret=pick:0/0/0/0/0/0/1 args=v,v,v  # in-area
  stub 0218ab50 ret=pick:0/0/0/0/0/0/1 args=p,v,v,v  # in-area
  stub 0218ac4c ret=pick:0/0/0/0/0/0/1 args=v,v,v  # in-area
  stub 021a2fa4 force ret=pick:0/1/2/3/9 args=v,v
  stub 02013d5c force ret=pick:0/1 args=v
  stub 0217b4d8 force ret=u32 args=v,v,v,v,v  # ov18
  stub 02013dd0 force ret=pick:0/1 args=v
  stub 0218dc70 force ret=u32 args=v  # ov21
  stub 021aa248 force ret=u32 args=v,v  # ov21
  stub 021a9f40 force ret=u32 args=v,v  # ov21
  stub 021a3098 force ret=pick:0/1/2/2 args=v
  stub 021a3364 force ret=none args=v,v,out:12  # position in direction d (player, d, &out)
  stub 021ab0f8 force ret=pick:0/1/2/4 args=v
  stub 021ab2e8 force ret=pick:0/1 args=v
  stub 0218b6f8 ret=pick:0/1 args=v,v  # in-area
  stub 0218b3dc ret=pick:0/1 args=p,v,v,v,v  # in-area
  stub 0218ae48 ret=none args=v,out:12  # in-area
  stub 021626e8 force ret=pick:0/1/5/6/7 args=v
  stub 021be288 force ret=pick:0/0/0/1 args=v,v
  stub 021f3a88 force ret=u32 args=v,v,v
  stub 021f38a4 force ret=u32 args=v,v
  stub 0218b630 ret=pick:0/1/6 args=v,v  # in-area
  stub 021ab310 force ret=pick:0/1 args=v
  stub 02163e58 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0x3/0x80
  stub 021f404c force ret=u32 args=v
  stub 021f37e0 force ret=u32 args=v
  stub 02010f18 force ret=pick:0/0x30/0x31 args=
  stub 020124ac force ret=u32 args=v,v,v,v  # main
  stub 021a3460 force ret=none args=v,outm:2:0x7/0,outm:2:0x7/0,outm:2:0x7/0
  stub 02164294 force ret=none args=v,v,v,out:12  # tile -> fx32 position (x, y, z, &out)
  stub 021ab194 force ret=pick:0/1 args=v
  stub 02164240 force ret=none args=v,outm:2:0xFF/0xFF,outm:2:0xFF/0xFF,v  # ov10 (dir, &x, &z, n): moves the s16 tile x / z
  stub 02188c7c force ret=u32 args=v  # fieldmap (other F2a area)
  stub 0216d8ac force ret=u32 args=v  # ov10
  stub 0216dddc force ret=u32 args=v,v,v,v,v,v  # ov10
  stub 021a3128 force ret=u32 args=v,v  # ov21
  stub 02013e04 force ret=pick:0/1 args=v
  stub 021a3504 force ret=pick:0/1 args=v,v,v
  stub 021a3524 force ret=pick:0/1 args=v,v,v
  stub 021ab1a0 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab1ac force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab1b8 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab1d0 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab1dc force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab1e8 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab1f4 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab200 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab20c force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab218 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab224 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab230 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab1c4 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 02162598 force ret=pick:0/1/0xFFFF args=v,in:8
  stub 0218b2a8 ret=none args=v,v,p  # in-area
  stub 0218b2e8 ret=pick:0/0/0/0/0/0/1 args=v,v,v,p  # in-area
  stub 0219e570 force ret=u32 args=v,in:8,v  # ov21 (sys, &rail position, flag)
  stub 0219e424 force ret=u32 args=v,v,v  # ov21
  stub 021626b8 force ret=pick:0/1 args=v,v
  stub 021bb188 force ret=pick:0/1 args=v
  stub 021c46f8 force ret=u32 args=v,v,v  # ov21
  stub 021ab188 force ret=pick:0/1 args=v
  stub 0216282c force ret=nullable:4:ptr:work:0x640:1 args=v,v,in:12,v
  stub 0216d7c4 force ret=pick:0/0x58/0x5B/0xA3/0xA6/0xA7 args=v
  stub 02162928 force ret=pick:0xFFFF/0xFFFF/0xFFFF/1 args=v,v,in:12,v
  stub 02162564 force ret=pick:0/1/0xFFFF args=v,in:12
  stub 02014110 force ret=none args=out:0x1C,v,v,v,v,v,v,v  # LOCATION init (loc, zone, ..)
  stub 02012878 force ret=none args=v,in:0x1C  # store a LOCATION in the game data (gd, &loc)
  stub 02162b68 force ret=u32 args=v,in:12  # ov10
  stub 02185410 force ret=u32 args=v,p,v,v  # ov20
  stub 02185360 force ret=u32 args=v,p,v,v  # ov20
  stub 021aa944 force ret=pick:0/1/2 args=v
  stub 021aa80c force ret=none args=v  # ov21
  stub 02008978 force ret=pick:0/1/0x7E/0x7F/0x80/0xFFFF args=v
  stub 02008980 force ret=u32 args=v,v  # main
  stub 02012934 force ret=u32 args=v  # main
  stub 0201aa18 force ret=pick:0/1/2 args=v
  stub 0201ab28 force ret=u32 args=v,v  # main
  stub 0201b7ec force ret=u32 args=v,v,v,v  # main
  stub 021d220c force ret=u32 args=v,v  # ov21
  stub 02008990 force ret=u32 args=v  # main
  stub 02008988 force ret=none args=v,v  # main
  stub 02188d50 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021c3378 force ret=none args=v  # ov21
  stub 0218aad0 force ret=none args=v,v  # in-area (verified port; callers keep the stub: the callee logs its entry r2 / r3)
  stub 0218ab0c ret=pick:0/1 args=v  # in-area
  stub 0200d564 force ret=pick:0/1/0x100 args=v
  stub 0216d83c force ret=u32 args=v  # ov10
  stub 0216d86c force ret=u32 args=v  # ov10
  stub 021641e8 force ret=u32 args=v  # ov10
  stub 021641f8 force ret=u32 args=v  # ov10
  stub 0216ddc0 force ret=pick:0/0/0/1 args=v,v,v,v
  stub 020122c0 force ret=u32 args=v,v,v,v  # main
  stub 020124e8 force ret=ptr:work:0x700:1 args=v
  stub 02017e1c force ret=pick:0/1/0x39/0x7F args=v,v,v
  stub 02014134 force ret=none args=out:0x1C,v,v,v,v,v,v,v  # LOCATION init, rail form (loc, id, b, c, d, e, f, g)
  stub 02162bcc force ret=u32 args=v,in:8  # ov10
  stub 02008970 force ret=none args=v,outm:4:0xFF/0xFF/1/0
  stub 02010ff4 force ret=u32 args=v,p,v,v  # main
  stub 0218aa74 ret=none args=v  # in-area
  stub 0200896c force ret=u32 args=v,v  # main
  stub 0218aa2c ret=pick:1/2 args=v  # in-area
  stub 02017e40 force ret=none args=v,v,v,v  # main

# 0218A4E0(req, gamesys, fm): MI_CpuFill8(req, 0, 0x40); fills the EVENT REQUEST (header) from the FIELDMAP getters 02188C84,
# 02188C9C, 02188CB4, 020120F4, 02012844, the key masks 020362DC / 0203630C, the player queries 021A2FE4, 021A3080, 021A308C,
# 021A3074 -> 0216D5F8 / 0216D888, 021A30C8, 021A3584, and 02014478(02012B38(gd)); computes +0x30 and the FLAGS word
fn 0218a4e0 ret=none inputs=400
  arg work 0x500
  arg u32 any
  arg work 0
  fix work 0x8C u32 any
  fix work 0x90 u32 any
  fix work 0x108 u32 addrof work 0x400
  fix work 0x400 u32 alt range 0 2 || range 0 8
  fix work 0x150 u32 alt const 0 || any
  fix work 0xD8 u16 alt range 0 0x1ff || any
  stub 02188c84 force ret=u16 args=v  # fieldmap (other F2a area)
  stub 020120f4 force ret=u32 args=v  # main
  stub 02012844 force ret=u32 args=v  # main
  stub 02188c9c force ret=u16 args=v  # fieldmap (other F2a area)
  stub 020362dc force ret=pick:0/1/0x400/0x800/0x401/0xC01 args=
  stub 0203630c force ret=pick:0/0x10/0x20/0x40/0x80/0xF0 args=
  stub 02188cb4 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021a2fe4 force ret=none args=v  # ov21
  stub 021a3080 force ret=pick:0/1/2/3 args=v
  stub 021a308c force ret=pick:0/1/2/3 args=v
  stub 021a3074 force ret=u32 args=v  # ov21
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v
  stub 0216d888 force ret=u32 args=v  # ov10
  stub 02012b38 force ret=u32 args=v  # main
  stub 02014478 force ret=u32 args=v  # main
  stub 021a30c8 force ret=u32 args=v  # ov21
  stub 021a3584 force ret=pick:0/1 args=v

# 0218A6E8(gamesys, f): x = 021A1358(02188CCC(f)); x == 0: 02028A70(02012A30(020120F4(gamesys))); else 021F2AB8(..) == 0:
# 02028A70(..), 021F2AD4(x)
fn 0218a6e8 ret=none inputs=100
  arg u32 any
  arg work 0
  stub 02188ccc force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021a1358 force ret=pick:0/1/2 args=v
  stub 020120f4 force ret=u32 args=v  # main
  stub 02012a30 force ret=u32 args=v  # main
  stub 02028a70 force ret=u32 args=v  # main
  stub 021f2ab8 force ret=pick:0/1 args=v
  stub 021f2ad4 force ret=none args=v

# 0218A728(gd): 02008DF0(02012F2C(gd), 0) (thunk 02008DF0 -> 02008DAC(x, 0, 1))
fn 0218a728 ret=none inputs=20
  arg u32 any
  stub 02012f2c force ret=u32 args=v  # main
  stub 02008dac force ret=u32 args=v,v,v  # main

# 0218A738(req): +0x26 != 0: 02014474(02012B38(gd), 0); return 02158768(gamesys, +0x26, 0, heap); else 02158B00(gamesys, heap) (0 stays 0)
fn 0218a738 ret=u32 inputs=100
  arg work 0x500
  fix work 0x514 u32 addrof work 0
  fix work 0x51C u32 alt oneof 0 1 2 3 || any
  fix work 0x520 u32 alt oneof 0 1 2 || any
  fix work 0x524 u16 alt range 0 3 || oneof 9 4
  fix work 0x526 u16 alt const 0 || any
  fix work 0x52C u32 alt oneof 0 0x10 0x20 0x40 0x80 || any
  fix work 0x530 u16 alt range 0 3 || const 9
  fix work 0x534 u32 addrof work 0x600
  fix work 0x600 u32 alt range 0 0x40000 || any
  fix work 0x608 u32 alt range 0 0x40000 || any
  stub 02012b38 force ret=u32 args=v  # main
  stub 02014474 force ret=none args=v,v  # main
  stub 02158768 force ret=u32 args=v,v,v,v  # ov10
  stub 02158b00 force ret=pick:0/0/0/1 args=v,v

# 0218A76C(req): 0218A8BC(req) || 0218A784(req, 9)
fn 0218a76c ret=u32 inputs=200
  arg work 0x500
  fix work 0x8C u32 any
  fix work 0x90 u32 any
  fix work 0x108 u32 addrof work 0x400
  fix work 0x400 u32 alt range 0 2 || range 0 8
  fix work 0x150 u32 alt const 0 || any
  fix work 0xD8 u16 alt range 0 0x1ff || any
  fix work 0x514 u32 addrof work 0
  fix work 0x51C u32 alt oneof 0 1 2 3 || any
  fix work 0x520 u32 alt oneof 0 1 2 || any
  fix work 0x524 u16 alt range 0 3 || oneof 9 4
  fix work 0x526 u16 alt const 0 || any
  fix work 0x52C u32 alt oneof 0 0x10 0x20 0x40 0x80 || any
  fix work 0x530 u16 alt range 0 3 || const 9
  fix work 0x534 u32 addrof work 0x600
  fix work 0x600 u32 alt range 0 0x40000 || any
  fix work 0x608 u32 alt range 0 0x40000 || any
  fix work 0x648 u16 alt const 0 || any
  fix work 0x64A u16 range 0 3
  fix work 0x64C u16 range 0 3
  fix work 0x64E u16 range 0 2
  fix work 0x650 u16 range 0 2
  stub 0218a8bc ret=pick:0/0/0/0/0/0/1 args=v  # in-area
  stub 0218a784 ret=pick:0/0/0/0/0/0/1 args=p,v  # in-area
  stub 0201210c force ret=u32 args=v  # main
  stub 02188cb4 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021a3038 force ret=none args=v,outm:12:0/0/3/0/0/0/0/0/0/0/3/0
  stub 02188c98 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021ab0d4 force ret=u32 args=v,in:12
  stub 021ab0f0 force ret=u32 args=v  # ov21
  stub 021ab188 force ret=pick:0/1 args=v
  stub 0201296c force ret=u32 args=v  # main
  stub 0216282c force ret=nullable:4:ptr:work:0x640:1 args=v,v,in:12,v
  stub 02158768 force ret=u32 args=v,v,v,v  # ov10
  stub 021a3098 force ret=pick:0/1/2 args=v
  stub 021a3074 force ret=u32 args=v  # ov21
  stub 0216d7c4 force ret=pick:0/0x58/0x5B/0xA3/0xA6/0xA7 args=v
  stub 02162928 force ret=pick:0xFFFF/0xFFFF/0xFFFF/1 args=v,v,in:12,v

# 0218A784(req, dir): f = 0201210C(gamesys); ev = 0201296C(gd); 021A3038(player, &pos); a = 021AB0F0(021AB0D4(02188C98(f), &pos));
# 021AB188(a) == 1: 0; s = 02162928(evdata, ev, &pos, dir) (talk target script); s != 0xFFFF ? 02158768(gamesys, s, 0, heap) : 0
fn 0218a784 ret=u32 inputs=200
  arg work 0x500
  arg u32 alt oneof 9 0 1 2 3 || any
  fix work 0x8C u32 any
  fix work 0x90 u32 any
  fix work 0x108 u32 addrof work 0x400
  fix work 0x400 u32 alt range 0 2 || range 0 8
  fix work 0x150 u32 alt const 0 || any
  fix work 0xD8 u16 alt range 0 0x1ff || any
  fix work 0x514 u32 addrof work 0
  fix work 0x51C u32 alt oneof 0 1 2 3 || any
  fix work 0x520 u32 alt oneof 0 1 2 || any
  fix work 0x524 u16 alt range 0 3 || oneof 9 4
  fix work 0x526 u16 alt const 0 || any
  fix work 0x52C u32 alt oneof 0 0x10 0x20 0x40 0x80 || any
  fix work 0x530 u16 alt range 0 3 || const 9
  fix work 0x534 u32 addrof work 0x600
  fix work 0x600 u32 alt range 0 0x40000 || any
  fix work 0x608 u32 alt range 0 0x40000 || any
  stub 0201210c force ret=u32 args=v  # main
  stub 0201296c force ret=u32 args=v  # main
  stub 021a3038 force ret=none args=v,out:12
  stub 02188c98 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021ab0d4 force ret=u32 args=v,in:12
  stub 021ab0f0 force ret=u32 args=v  # ov21
  stub 021ab188 force ret=pick:0/0/1 args=v
  stub 02162928 force ret=pick:0xFFFF/0xFFFF/0xFFFF/1 args=v,v,in:12,v
  stub 02158768 force ret=u32 args=v,v,v,v  # ov10

# 0218A7F0(req): 0218A784(req, facing) || (d = 021A2FA4(player, trigger), d == 9 || d == facing ? 0 : 0218A784(req, d))
fn 0218a7f0 ret=u32 inputs=200
  arg work 0x500
  fix work 0x8C u32 any
  fix work 0x90 u32 any
  fix work 0x108 u32 addrof work 0x400
  fix work 0x400 u32 alt range 0 2 || range 0 8
  fix work 0x150 u32 alt const 0 || any
  fix work 0xD8 u16 alt range 0 0x1ff || any
  fix work 0x514 u32 addrof work 0
  fix work 0x51C u32 alt oneof 0 1 2 3 || any
  fix work 0x520 u32 alt oneof 0 1 2 || any
  fix work 0x524 u16 alt range 0 3 || oneof 9 4
  fix work 0x526 u16 alt const 0 || any
  fix work 0x52C u32 alt oneof 0 0x10 0x20 0x40 0x80 || any
  fix work 0x530 u16 alt range 0 3 || const 9
  fix work 0x534 u32 addrof work 0x600
  fix work 0x600 u32 alt range 0 0x40000 || any
  fix work 0x608 u32 alt range 0 0x40000 || any
  stub 0218a784 ret=pick:0/0/0/0/0/0/1 args=p,v  # in-area
  stub 021a2fa4 force ret=pick:0/1/2/3/9 args=v,v
  stub 0201210c force ret=u32 args=v  # main
  stub 0201296c force ret=u32 args=v  # main
  stub 021a3038 force ret=none args=v,out:12
  stub 02188c98 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021ab0d4 force ret=u32 args=v,in:12
  stub 021ab0f0 force ret=u32 args=v  # ov21
  stub 021ab188 force ret=pick:0/0/1 args=v
  stub 02162928 force ret=pick:0xFFFF/0xFFFF/0xFFFF/1 args=v,v,in:12,v
  stub 02158768 force ret=u32 args=v,v,v,v  # ov10

# 0218A824(req): d = direction of the trigger mask (0x40 0, 0x80 1, 0x20 2, 0x10 3, else 9); d == facing or d == 9: 0;
# state 1 or 2: 0; else 0218A784(req, d)
fn 0218a824 ret=u32 inputs=200
  arg work 0x500
  fix work 0x8C u32 any
  fix work 0x90 u32 any
  fix work 0x108 u32 addrof work 0x400
  fix work 0x400 u32 alt range 0 2 || range 0 8
  fix work 0x150 u32 alt const 0 || any
  fix work 0xD8 u16 alt range 0 0x1ff || any
  fix work 0x514 u32 addrof work 0
  fix work 0x51C u32 alt oneof 0 1 2 3 || any
  fix work 0x520 u32 alt oneof 0 1 2 || any
  fix work 0x524 u16 alt range 0 3 || oneof 9 4
  fix work 0x526 u16 alt const 0 || any
  fix work 0x52C u32 alt oneof 0 0x10 0x20 0x40 0x80 || any
  fix work 0x530 u16 alt range 0 3 || const 9
  fix work 0x534 u32 addrof work 0x600
  fix work 0x600 u32 alt range 0 0x40000 || any
  fix work 0x608 u32 alt range 0 0x40000 || any
  stub 0218a784 ret=pick:0/0/0/0/0/0/1 args=p,v  # in-area
  stub 0201210c force ret=u32 args=v  # main
  stub 0201296c force ret=u32 args=v  # main
  stub 021a3038 force ret=none args=v,out:12
  stub 02188c98 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021ab0d4 force ret=u32 args=v,in:12
  stub 021ab0f0 force ret=u32 args=v  # ov21
  stub 021ab188 force ret=pick:0/0/1 args=v
  stub 02162928 force ret=pick:0xFFFF/0xFFFF/0xFFFF/1 args=v,v,in:12,v
  stub 02158768 force ret=u32 args=v,v,v,v  # ov10

# 0218A878(req, gamesys, fm): !02013D5C(zone): 0; d = 021A2FA4(player, trigger); d == 9: 0; else 0217B4D8(gamesys, fm, player,
# +0x34, facing, d)
fn 0218a878 ret=u32 inputs=200
  arg work 0x500
  arg u32 any
  arg work 0
  fix work 0x514 u32 addrof work 0
  fix work 0x51C u32 alt oneof 0 1 2 3 || any
  fix work 0x520 u32 alt oneof 0 1 2 || any
  fix work 0x524 u16 alt range 0 3 || oneof 9 4
  fix work 0x526 u16 alt const 0 || any
  fix work 0x52C u32 alt oneof 0 0x10 0x20 0x40 0x80 || any
  fix work 0x530 u16 alt range 0 3 || const 9
  fix work 0x534 u32 addrof work 0x600
  fix work 0x600 u32 alt range 0 0x40000 || any
  fix work 0x608 u32 alt range 0 0x40000 || any
  stub 02013d5c force ret=pick:0/1 args=v
  stub 021a2fa4 force ret=pick:0/1/2/3/9 args=v,v
  stub 0217b4d8 force ret=u32 args=v,v,v,v,v  # ov18

# 0218A8BC(req): f = 0201210C(gamesys); p = 02188CB4(f); 021A3038(p, &pos); 021AB188(021AB0F0(021AB0D4(02188C98(f), &pos))) != 1: 0;
# bg = 0216282C(evdata, 0201296C(gd), &pos, 9); !bg or bg+8 != 0: 0; tile (pos x, z) == (bg+0xA + bg+0xE / 2, bg+0xC + bg+0x10 / 2):
# 02158768(gamesys, bg+0, 0, heap); else 021A3098(p) != 0 or 0216D7C4(021A3074(p)) in 0x58..0x5B, 0xA3..0xA6: the same; else 0
fn 0218a8bc ret=u32 inputs=300
  arg work 0x500
  fix work 0x8C u32 any
  fix work 0x90 u32 any
  fix work 0x108 u32 addrof work 0x400
  fix work 0x400 u32 alt range 0 2 || range 0 8
  fix work 0x150 u32 alt const 0 || any
  fix work 0xD8 u16 alt range 0 0x1ff || any
  fix work 0x514 u32 addrof work 0
  fix work 0x51C u32 alt oneof 0 1 2 3 || any
  fix work 0x520 u32 alt oneof 0 1 2 || any
  fix work 0x524 u16 alt range 0 3 || oneof 9 4
  fix work 0x526 u16 alt const 0 || any
  fix work 0x52C u32 alt oneof 0 0x10 0x20 0x40 0x80 || any
  fix work 0x530 u16 alt range 0 3 || const 9
  fix work 0x534 u32 addrof work 0x600
  fix work 0x600 u32 alt range 0 0x40000 || any
  fix work 0x608 u32 alt range 0 0x40000 || any
  fix work 0x648 u16 alt const 0 || any
  fix work 0x64A u16 range 0 3
  fix work 0x64C u16 range 0 3
  fix work 0x64E u16 range 0 2
  fix work 0x650 u16 range 0 2
  stub 0201210c force ret=u32 args=v  # main
  stub 02188cb4 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021a3038 force ret=none args=v,outm:12:0/0/3/0/0/0/0/0/0/0/3/0
  stub 02188c98 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021ab0d4 force ret=u32 args=v,in:12
  stub 021ab0f0 force ret=u32 args=v  # ov21
  stub 021ab188 force ret=pick:0/1 args=v
  stub 0201296c force ret=u32 args=v  # main
  stub 0216282c force ret=nullable:4:ptr:work:0x640:1 args=v,v,in:12,v
  stub 02158768 force ret=u32 args=v,v,v,v  # ov10
  stub 021a3098 force ret=pick:0/1/2 args=v
  stub 021a3074 force ret=u32 args=v  # ov21
  stub 0216d7c4 force ret=pick:0/0x58/0x5B/0xA3/0xA6/0xA7 args=v

# 0218A9B0(req, fm): gs = 02188C80(fm); gd = 020120F4(gs); (FLAGS bit 2 or bit 9) && (e = 0218AC84(fm, gs, gd)): e;
# !bit 2: 0; 0218AB9C(gd, fm); 0218AC08(gs, fm, zone) || 0218AB50(req, fm, gs, gd) || 0218AC4C(fm, gs, gd) || 0
fn 0218a9b0 ret=u32 inputs=300
  arg work 0x500
  arg work 0
  fix work 0x8C u32 any
  fix work 0x90 u32 any
  fix work 0x108 u32 addrof work 0x400
  fix work 0x400 u32 alt range 0 2 || range 0 8
  fix work 0x150 u32 alt const 0 || any
  fix work 0xD8 u16 alt range 0 0x1ff || any
  fix work 0x514 u32 addrof work 0
  fix work 0x51C u32 alt oneof 0 1 2 3 || any
  fix work 0x520 u32 alt oneof 0 1 2 || any
  fix work 0x524 u16 alt range 0 3 || oneof 9 4
  fix work 0x526 u16 alt const 0 || any
  fix work 0x52C u32 alt oneof 0 0x10 0x20 0x40 0x80 || any
  fix work 0x530 u16 alt range 0 3 || const 9
  fix work 0x534 u32 addrof work 0x600
  fix work 0x600 u32 alt range 0 0x40000 || any
  fix work 0x608 u32 alt range 0 0x40000 || any
  stub 02188c84 force ret=u16 args=v  # fieldmap (other F2a area)
  stub 02188c80 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 020120f4 force ret=u32 args=v  # main
  stub 0218ac84 ret=pick:0/0/0/0/0/0/1 args=v,p,v  # in-area
  stub 0218ab9c ret=none args=v,v  # in-area
  stub 0218ac08 ret=pick:0/0/0/0/0/0/1 args=v,v,v  # in-area
  stub 0218ab50 ret=pick:0/0/0/0/0/0/1 args=p,v,v,v  # in-area
  stub 0218ac4c ret=pick:0/0/0/0/0/0/1 args=v,v,v  # in-area
  stub 02012ebc force ret=u32 args=v  # main
  stub 02188ce8 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021aa944 force ret=pick:0/1/2 args=v
  stub 021aa80c force ret=none args=v  # ov21
  stub 020071cc force ret=u32 args=v  # main
  stub 02008978 force ret=pick:0/1/0x7E/0x7F/0x80/0xFFFF args=v
  stub 02008980 force ret=u32 args=v,v  # main
  stub 02012934 force ret=u32 args=v  # main
  stub 02188c9c force ret=u16 args=v  # fieldmap (other F2a area)
  stub 0201aa18 force ret=pick:0/1/2 args=v
  stub 0201ab28 force ret=u32 args=v,v  # main
  stub 0201b7ec force ret=u32 args=v,v,v,v  # main
  stub 02013ffc force ret=pick:0/1 args=v
  stub 021d220c force ret=u32 args=v,v  # ov21
  stub 02008990 force ret=u32 args=v  # main
  stub 02008988 force ret=none args=v,v  # main
  stub 02188d50 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021c3378 force ret=none args=v  # ov21
  stub 0218aad0 force ret=none args=v,v  # in-area (verified port; callers keep the stub: the callee logs its entry r2 / r3)
  stub 0218ab0c ret=pick:0/1 args=v  # in-area
  stub 02158768 force ret=u32 args=v,v,v,v  # ov10
  stub 0200d564 force ret=pick:0/1/0x100 args=v
  stub 02008970 force ret=none args=v,outm:4:0xFF/0xFF/1/0
  stub 02010ff4 force ret=u32 args=v,p,v,v  # main
  stub 0218aa74 ret=none args=v  # in-area
  stub 0200896c force ret=u32 args=v,v  # main
  stub 02017e1c force ret=pick:0/1 args=v,v,v
  stub 0218aa2c ret=pick:1/2 args=v  # in-area
  stub 02017e40 force ret=none args=v,v,v,v  # main

# 0218AA2C(party): for i < 0201AA18(party): m = 0201AB28(party, i); p = 02017E1C(m, 0xA, 0) (held item); 02017E1C(m, 0x4C, 0) == 0
# && (p == 0x31 || p == 0x28): 2; else 1
fn 0218aa2c ret=u32 inputs=200
  arg u32 any
  stub 0201aa18 force ret=pick:0/1/2/6 args=v
  stub 0201ab28 force ret=u32 args=v,v  # main
  stub 02017e1c force ret=pick:0/1/9/0x28/0x31/0x4C args=v,v,v

# 0218AA74(party): n = 0201AA18(party); k = 0218AA2C(party); for i < n: m = 0201AB28(party, i); 02017E1C(m, 0x4C, 0) == 1 (egg):
# c = 02017E1C(m, 9, 0) (hatch counter); 02017E40(m, 9, c >= k ? c - k : 0)
fn 0218aa74 ret=none inputs=200
  arg u32 any
  stub 0201aa18 force ret=pick:0/1/2/6 args=v
  stub 0218aa2c ret=pick:1/2 args=v  # in-area
  stub 0201ab28 force ret=u32 args=v,v  # main
  stub 02017e1c force ret=pick:0/1/2/9/0x28/0x31 args=v,v,v
  stub 02017e40 force ret=none args=v,v,v,v  # main

# 0218AAD0(gd, party): s = 02008854(02012EBC(gd)) (thunk -> 020071CC(x, 0x1C)); 02008970(s, &c); c += 02010FF4(0x100);
# c > 0x10000: 0218AA74(party), c = 0; 0200896C(s) (the egg step counter; the stored c is a stack word: Ghidra hides it)
fn 0218aad0 ret=none inputs=200
  arg u32 any
  arg u32 any
  stub 02012ebc force ret=u32 args=v  # main
  stub 020071cc force ret=u32 args=v  # main
  stub 02008970 force ret=none args=v,outm:4:0xFF/0xFF/1/0
  stub 02010ff4 force ret=u32 args=v,p,v,v  # main
  stub 0218aa74 ret=none args=v  # in-area
  stub 0200896c force ret=u32 args=v,v  # main
  stub 0201aa18 force ret=pick:0/1/2 args=v
  stub 0218aa2c ret=pick:1/2 args=v  # in-area
  stub 0201ab28 force ret=u32 args=v,v  # main
  stub 02017e1c force ret=pick:0/1/2/9 args=v,v,v
  stub 02017e40 force ret=none args=v,v,v,v  # main

# 0218AB0C(party): any i < 0201AA18(party) with 02017E1C(m, 0x4C, 0) == 1 && 02017E1C(m, 9, 0) == 0 (an egg ready to hatch): 1; else 0
fn 0218ab0c ret=u32 inputs=200
  arg u32 any
  stub 0201aa18 force ret=pick:0/1/2/6 args=v
  stub 0201ab28 force ret=u32 args=v,v  # main
  stub 02017e1c force ret=pick:0/1 args=v,v,v

# 0218AB50(req, fm, gs, gd): h = 02188C84(fm); party = 02012934(gd); 021C3378(02188D50(fm)); 0218AAD0(gd, party);
# 0218AB0C(party) ? 02158768(gs, 0x8CA, 0, h) : 0
fn 0218ab50 ret=u32 inputs=200
  arg work 0x500
  arg work 0
  arg u32 any
  arg u32 any
  stub 02188c84 force ret=u16 args=v  # fieldmap (other F2a area)
  stub 02012934 force ret=u32 args=v  # main
  stub 02188d50 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021c3378 force ret=none args=v  # ov21
  stub 0218aad0 force ret=none args=v,v  # in-area (verified port; callers keep the stub: the callee logs its entry r2 / r3)
  stub 0218ab0c ret=pick:0/1 args=v  # in-area
  stub 02158768 force ret=u32 args=v,v,v,v  # ov10
  stub 02012ebc force ret=u32 args=v  # main
  stub 020071cc force ret=u32 args=v  # main
  stub 02008970 force ret=none args=v,outm:4:0xFF/0xFF/1/0
  stub 02010ff4 force ret=u32 args=v,p,v,v  # main
  stub 0218aa74 ret=none args=v  # in-area
  stub 0200896c force ret=u32 args=v,v  # main
  stub 0201aa18 force ret=pick:0/1/2 args=v
  stub 0201ab28 force ret=u32 args=v,v  # main
  stub 02017e1c force ret=pick:0/1 args=v,v,v
  stub 0218aa2c ret=pick:1/2 args=v  # in-area
  stub 02017e40 force ret=none args=v,v,v,v  # main

# 0218AB9C(gd, fm): s = 02008854(02012EBC(gd)); n = 02008978(s) + 1 (u16); n >= 0x80: n = 0, wrap; 02008980(s, n); wrap: party =
# 02012934(gd); for i < 0201AA18(party): 0201B7EC(0201AB28(party, i), 02188C9C(fm), 02188C84(fm)) (friendship walk step)
fn 0218ab9c ret=none inputs=200
  arg u32 any
  arg work 0
  stub 02012ebc force ret=u32 args=v  # main
  stub 020071cc force ret=u32 args=v  # main
  stub 02008978 force ret=pick:0/1/0x7E/0x7F/0x80/0xFFFF args=v
  stub 02008980 force ret=u32 args=v,v  # main
  stub 02012934 force ret=u32 args=v  # main
  stub 02188c9c force ret=u16 args=v  # fieldmap (other F2a area)
  stub 02188c84 force ret=u16 args=v  # fieldmap (other F2a area)
  stub 0201aa18 force ret=pick:0/1/2 args=v
  stub 0201ab28 force ret=u32 args=v,v  # main
  stub 0201b7ec force ret=u32 args=v,v,v,v  # main

# 0218AC08(gs, fm, zone): s = 02008854(02012EBC(020120F4(gs))); 02013FFC(zone) == 0: 0; e = 021D220C(gs, fm);
# 02008988(s, 02008990(s) + 1); returns e
fn 0218ac08 ret=u32 inputs=200
  arg u32 any
  arg work 0
  arg u32 any
  stub 020120f4 force ret=u32 args=v  # main
  stub 02012ebc force ret=u32 args=v  # main
  stub 020071cc force ret=u32 args=v  # main
  stub 02013ffc force ret=pick:0/1 args=v
  stub 021d220c force ret=u32 args=v,v  # ov21
  stub 02008990 force ret=u32 args=v  # main
  stub 02008988 force ret=none args=v,v  # main

# 0218AC4C(fm, gs, gd): 0200D418(02012EBC(gd)) (thunk -> 020071CC(x, 0x44)); (u8) 0200D564(..) != 0 ? 02158768(gs, 0x2792, 0, heap) : 0
fn 0218ac4c ret=u32 inputs=100
  arg work 0
  arg u32 any
  arg u32 any
  stub 02012ebc force ret=u32 args=v  # main
  stub 020071cc force ret=u32 args=v  # main
  stub 0200d564 force ret=pick:0/1/0x100 args=v
  stub 02188c84 force ret=u16 args=v  # fieldmap (other F2a area)
  stub 02158768 force ret=u32 args=v,v,v,v  # ov10

# 0218AC84(fm, gs, gd): 02012EBC(gd); e = 02188CE8(fm); r = 021AA944(e); r == 0: 021AA80C(e); returns r
fn 0218ac84 ret=u32 inputs=100
  arg work 0
  arg u32 any
  arg u32 any
  stub 02012ebc force ret=u32 args=v  # main
  stub 02188ce8 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021aa944 force ret=pick:0/1/2 args=v
  stub 021aa80c force ret=none args=v  # ov21

# 0218ACA8(gs): 02010F18() == 0x30: 0; else 020124AC(gs, 0x53, 0x021F532D, 0) (GMEVENT create)
fn 0218aca8 ret=u32 inputs=60
  arg u32 any
  stub 02010f18 force ret=pick:0/0x30/0x31 args=
  stub 020124ac force ret=u32 args=v,v,v,v  # main

# 0218ACD0(req, fm, pos): id = 0218AE04(req, pos); id == 0xFFFF: 0; o = 021625CC(evdata, id); 021626EC(o) == 1 or !0218AEAC(o): 0;
# 0218ADC8(req, id, pos); returns 0218AD80(req, fm, id, pos)
fn 0218acd0 ret=u32 inputs=200
  arg work 0x500
  arg work 0
  arg work 0x600
  fix work 0x514 u32 addrof work 0
  fix work 0x51C u32 alt oneof 0 1 2 3 || any
  fix work 0x520 u32 alt oneof 0 1 2 || any
  fix work 0x524 u16 alt range 0 3 || oneof 9 4
  fix work 0x526 u16 alt const 0 || any
  fix work 0x52C u32 alt oneof 0 0x10 0x20 0x40 0x80 || any
  fix work 0x530 u16 alt range 0 3 || const 9
  fix work 0x534 u32 addrof work 0x600
  fix work 0x600 u32 alt range 0 0x40000 || any
  fix work 0x608 u32 alt range 0 0x40000 || any
  stub 0218ae04 ret=pick:0/1/0xFFFF args=v,v
  stub 021625cc force ret=u32 args=v,v  # ov10
  stub 021626ec force ret=pick:0/1 args=v
  stub 0218aeac ret=pick:0/1 args=v  # in-area
  stub 0218adc8 ret=none args=v,v,v  # in-area
  stub 0218ad80 ret=pick:0/0/0/0/0/0/1 args=v,v,v,v  # in-area
  stub 02162564 force ret=pick:0/1/0xFFFF args=v,in:12
  stub 021626b8 force ret=pick:0/1 args=v,v
  stub 021626e8 force ret=pick:0/1/5/6/7 args=v
  stub 02188c9c force ret=u16 args=v  # fieldmap (other F2a area)
  stub 02014110 force ret=none args=out:0x1C,v,v,v,v,v,v,v  # LOCATION init (loc, zone, ..)
  stub 02012878 force ret=none args=v,in:0x1C  # store a LOCATION in the game data (gd, &loc)
  stub 02162b68 force ret=u32 args=v,in:12  # ov10
  stub 02185410 force ret=u32 args=v,p,v,v  # ov20
  stub 02185360 force ret=u32 args=v,p,v,v  # ov20

# 0218AD28(req, fm): 021A3038(player, &pos); a = 021AB0F0(021AB0D4(02188C98(fm), &pos)); 021C8A74(a) (thunk 021C89A8):
# 021C8A30(0, gamesys, fm); 021C8CF0(a) (thunk 021C8BC4): 021C8C88(0, gamesys, fm); else 0
fn 0218ad28 ret=u32 inputs=200
  arg work 0x500
  arg work 0
  fix work 0x514 u32 addrof work 0
  fix work 0x51C u32 alt oneof 0 1 2 3 || any
  fix work 0x520 u32 alt oneof 0 1 2 || any
  fix work 0x524 u16 alt range 0 3 || oneof 9 4
  fix work 0x526 u16 alt const 0 || any
  fix work 0x52C u32 alt oneof 0 0x10 0x20 0x40 0x80 || any
  fix work 0x530 u16 alt range 0 3 || const 9
  fix work 0x534 u32 addrof work 0x600
  fix work 0x600 u32 alt range 0 0x40000 || any
  fix work 0x608 u32 alt range 0 0x40000 || any
  stub 021a3038 force ret=none args=v,out:12
  stub 02188c98 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021ab0d4 force ret=u32 args=v,in:12
  stub 021ab0f0 force ret=u32 args=v  # ov21
  stub 021c89a8 force ret=pick:0/1 args=v  # ov21
  stub 021c8a30 force ret=u32 args=v,v,v  # ov21
  stub 021c8bc4 force ret=pick:0/1 args=v  # ov21
  stub 021c8c88 force ret=u32 args=v,v,v  # ov21
  stub 021c8a74 force ret=pick:0/1 args=v
  stub 021c8cf0 force ret=pick:0/1 args=v

# 0218AD80(req, fm, id, pos): o = 021625CC(evdata, (u16) id); w = pos ? 021626A8(o, pos) : 0; 021626E8(o) == 6 ?
# 02185410(gamesys, fm, o, w) : 02185360(gamesys, fm, o, w) (warp / exit events, ov20)
fn 0218ad80 ret=u32 inputs=200
  arg work 0x500
  arg work 0
  arg u32 any
  arg u32 alt const 0 || addrof work 0x600
  fix work 0x514 u32 addrof work 0
  fix work 0x51C u32 alt oneof 0 1 2 3 || any
  fix work 0x520 u32 alt oneof 0 1 2 || any
  fix work 0x524 u16 alt range 0 3 || oneof 9 4
  fix work 0x526 u16 alt const 0 || any
  fix work 0x52C u32 alt oneof 0 0x10 0x20 0x40 0x80 || any
  fix work 0x530 u16 alt range 0 3 || const 9
  fix work 0x534 u32 addrof work 0x600
  fix work 0x600 u32 alt range 0 0x40000 || any
  fix work 0x608 u32 alt range 0 0x40000 || any
  stub 021625cc force ret=u32 args=v,v  # ov10
  stub 02162b68 force ret=u32 args=v,in:12  # ov10
  stub 021626e8 force ret=pick:0/5/6 args=v
  stub 02185410 force ret=u32 args=v,p,v,v  # ov20
  stub 02185360 force ret=u32 args=v,p,v,v  # ov20

# 0218ADC8(req, id, pos): 02014110(&loc, 02188C9C(fm), (s16) id, 0, 0, pos[0], pos[1], pos[2]); 02012878(gd, &loc) (the warp
# return location)
fn 0218adc8 ret=none inputs=100
  arg work 0x500
  arg u32 any
  arg work 0x600
  fix work 0x514 u32 addrof work 0
  fix work 0x51C u32 alt oneof 0 1 2 3 || any
  fix work 0x520 u32 alt oneof 0 1 2 || any
  fix work 0x524 u16 alt range 0 3 || oneof 9 4
  fix work 0x526 u16 alt const 0 || any
  fix work 0x52C u32 alt oneof 0 0x10 0x20 0x40 0x80 || any
  fix work 0x530 u16 alt range 0 3 || const 9
  fix work 0x534 u32 addrof work 0x600
  fix work 0x600 u32 alt range 0 0x40000 || any
  fix work 0x608 u32 alt range 0 0x40000 || any
  stub 02188c9c force ret=u16 args=v  # fieldmap (other F2a area)
  stub 02014110 force ret=none args=out:0x1C,v,v,v,v,v,v,v  # LOCATION init (loc, zone, ..)
  stub 02012878 force ret=none args=v,in:0x1C  # store a LOCATION in the game data (gd, &loc)

# 0218AE04(req, pos): id = 02162564(evdata, pos); id == 0xFFFF: 0xFFFF; o = 021625CC(evdata, id); 0218AEAC(o) or 021626B8(o, facing)
# (thunk 0218AECC): id; else 0xFFFF
fn 0218ae04 ret=u32 inputs=200
  arg work 0x500
  arg work 0x600
  fix work 0x514 u32 addrof work 0
  fix work 0x51C u32 alt oneof 0 1 2 3 || any
  fix work 0x520 u32 alt oneof 0 1 2 || any
  fix work 0x524 u16 alt range 0 3 || oneof 9 4
  fix work 0x526 u16 alt const 0 || any
  fix work 0x52C u32 alt oneof 0 0x10 0x20 0x40 0x80 || any
  fix work 0x530 u16 alt range 0 3 || const 9
  fix work 0x534 u32 addrof work 0x600
  fix work 0x600 u32 alt range 0 0x40000 || any
  fix work 0x608 u32 alt range 0 0x40000 || any
  stub 02162564 force ret=pick:0/1/0xFFFF args=v,in:12
  stub 021625cc force ret=u32 args=v,v  # ov10
  stub 0218aeac ret=pick:0/1 args=v  # in-area
  stub 021626b8 force ret=pick:0/1 args=v,v
  stub 021626e8 force ret=pick:0/1/5/6/7 args=v

# 0218AE48(req, out): out = *(+0x34) (3 words); facing 0: z -= 0x10000, 1: z += 0x10000, 2: x -= 0x10000, 3: x += 0x10000
# (the tile in front)
fn 0218ae48 ret=none inputs=100
  arg work 0x500
  arg work 0x780
  fix work 0x514 u32 addrof work 0
  fix work 0x51C u32 alt oneof 0 1 2 3 || any
  fix work 0x520 u32 alt oneof 0 1 2 || any
  fix work 0x524 u16 alt range 0 3 || oneof 9 4
  fix work 0x526 u16 alt const 0 || any
  fix work 0x52C u32 alt oneof 0 0x10 0x20 0x40 0x80 || any
  fix work 0x530 u16 alt range 0 3 || const 9
  fix work 0x534 u32 addrof work 0x600
  fix work 0x600 u32 alt range 0 0x40000 || any
  fix work 0x608 u32 alt range 0 0x40000 || any
  fix work 0x524 u16 alt range 0 3 || range 0 5

# 0218AEAC(o): k = 021626E8(o); k < 7 and bit k of 0x61 (kinds 0, 5, 6): 1; else 0
fn 0218aeac ret=u32 inputs=100
  arg u32 any
  stub 021626e8 force ret=pick:0/1/2/3/4/5/6/7/8 args=v

# 0218AED4(req, gs, fm): 0218AE48(req, &front); a = 021AB0F8(021AB0D4(02188C98(fm), &front)); !(a & 1): 0; id = 0218AE04(req, +0x34);
# id != 0xFFFF && 021626EC(o = 021625CC(evdata, id)) == 0 && 021626E8(o) == 1: 0218ADC8(req, id, +0x34), 0218AD80(req, fm, id, +0x34);
# id = 0218AE04(req, &front); id != 0xFFFF && 021626EC(021625CC(..)) == 0: 0218ADC8(req, id, +0x34), 0218AD80(req, fm, id, &front); 0
fn 0218aed4 ret=u32 inputs=300
  arg work 0x500
  arg u32 any
  arg work 0
  fix work 0x8C u32 any
  fix work 0x90 u32 any
  fix work 0x108 u32 addrof work 0x400
  fix work 0x400 u32 alt range 0 2 || range 0 8
  fix work 0x150 u32 alt const 0 || any
  fix work 0xD8 u16 alt range 0 0x1ff || any
  fix work 0x514 u32 addrof work 0
  fix work 0x51C u32 alt oneof 0 1 2 3 || any
  fix work 0x520 u32 alt oneof 0 1 2 || any
  fix work 0x524 u16 alt range 0 3 || oneof 9 4
  fix work 0x526 u16 alt const 0 || any
  fix work 0x52C u32 alt oneof 0 0x10 0x20 0x40 0x80 || any
  fix work 0x530 u16 alt range 0 3 || const 9
  fix work 0x534 u32 addrof work 0x600
  fix work 0x600 u32 alt range 0 0x40000 || any
  fix work 0x608 u32 alt range 0 0x40000 || any
  stub 0218ae48 ret=none args=v,out:12  # in-area
  stub 02188c98 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021ab0d4 force ret=u32 args=v,in:12
  stub 021ab0f8 force ret=pick:0/1/2/3 args=v
  stub 0218ae04 ret=pick:0/1/0xFFFF args=v,p  # in-area
  stub 021625cc force ret=u32 args=v,v  # ov10
  stub 021626ec force ret=pick:0/1 args=v
  stub 021626e8 force ret=pick:0/1/5/6/7 args=v
  stub 0218adc8 ret=none args=v,v,v  # in-area
  stub 0218ad80 ret=pick:0/0/0/0/0/0/1 args=v,v,v,v  # in-area
  stub 02162564 force ret=pick:0/1/0xFFFF args=v,in:12
  stub 0218aeac ret=pick:0/1 args=v  # in-area
  stub 021626b8 force ret=pick:0/1 args=v,v
  stub 02188c9c force ret=u16 args=v  # fieldmap (other F2a area)
  stub 02014110 force ret=none args=out:0x1C,v,v,v,v,v,v,v  # LOCATION init (loc, zone, ..)
  stub 02012878 force ret=none args=v,in:0x1C  # store a LOCATION in the game data (gd, &loc)
  stub 02162b68 force ret=u32 args=v,in:12  # ov10
  stub 02185410 force ret=u32 args=v,p,v,v  # ov20
  stub 02185360 force ret=u32 args=v,p,v,v  # ov20

# 0218AF90(req, gs, fm): 0218AE48(req, &front); 021BE288(fm, 6): 021F3A88(gs, x >> 16, z >> 16) (rounded toward zero);
# 021BE288(fm, 0x19 / 0x1A / 0x1B): 021F38A4(fm, facing); a = 021AB0D4(02188C98(fm), &front); !(021AB0F8(a) & 1): 0;
# 021A3098(player) == 2 && 0218B630(gd, 0x7F) != 6 && 021AB310(021AB0F0(a)) == 1: p = front; 02164208(facing, &p, 0x10000);
# 02163E58(021A3074(player), &p, &h) == 1 && front.y > h: 02158768(gs, 0x271F, 0, heap) (waterfall);
# 0218AE04(req, &front) == 0xFFFF: 021BE288(fm, 2) ? 021F404C(gs) : 021BE288(fm, 5) ? 021F37E0(gs) : 0; 0
fn 0218af90 ret=u32 inputs=400
  arg work 0x500
  arg u32 any
  arg work 0
  fix work 0x8C u32 any
  fix work 0x90 u32 any
  fix work 0x108 u32 addrof work 0x400
  fix work 0x400 u32 alt range 0 2 || range 0 8
  fix work 0x150 u32 alt const 0 || any
  fix work 0xD8 u16 alt range 0 0x1ff || any
  fix work 0x514 u32 addrof work 0
  fix work 0x51C u32 alt oneof 0 1 2 3 || any
  fix work 0x520 u32 alt oneof 0 1 2 || any
  fix work 0x524 u16 alt range 0 3 || oneof 9 4
  fix work 0x526 u16 alt const 0 || any
  fix work 0x52C u32 alt oneof 0 0x10 0x20 0x40 0x80 || any
  fix work 0x530 u16 alt range 0 3 || const 9
  fix work 0x534 u32 addrof work 0x600
  fix work 0x600 u32 alt range 0 0x40000 || any
  fix work 0x608 u32 alt range 0 0x40000 || any
  stub 0218ae48 ret=none args=v,out:12  # in-area
  stub 021be288 force ret=pick:0/0/0/1 args=v,v
  stub 021f3a88 force ret=u32 args=v,v,v
  stub 021f38a4 force ret=u32 args=v,v
  stub 02188c98 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021ab0d4 force ret=u32 args=v,in:12
  stub 021ab0f8 force ret=pick:0/1/2/3 args=v
  stub 021a3098 force ret=pick:0/1/2 args=v
  stub 0218b630 ret=pick:0/1/6 args=v,v  # in-area
  stub 021ab0f0 force ret=u32 args=v  # ov21
  stub 021ab310 force ret=pick:0/1 args=v
  stub 021a3074 force ret=u32 args=v  # ov21
  stub 02164208 force ret=none args=v,in:12,v
  stub 02163e58 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0x3/0x80
  stub 02158768 force ret=u32 args=v,v,v,v  # ov10
  stub 0218ae04 ret=pick:0/1/0xFFFF args=v,v
  stub 021f404c force ret=u32 args=v
  stub 021f37e0 force ret=u32 args=v
  stub 02012934 force ret=u32 args=v  # main
  stub 0201aa18 force ret=pick:0/1/2 args=v
  stub 0201ab28 force ret=u32 args=v,v  # main
  stub 02017e1c force ret=pick:0/1/0x39/0x7F args=v,v,v
  stub 02162564 force ret=pick:0/1/0xFFFF args=v,in:12
  stub 021625cc force ret=u32 args=v,v  # ov10
  stub 0218aeac ret=pick:0/1 args=v  # in-area
  stub 021626b8 force ret=pick:0/1 args=v,v
  stub 021626e8 force ret=pick:0/1/5/6/7 args=v

# 0218B0CC(req, gs, fm): 0219E2F4(021A3074(02188CB4(fm)), &rp) (rail position, 8 bytes); id = 02162598(evdata, &rp); id == 0xFFFF: 0;
# o = 021625CC(evdata, id); 021626EC(o) == 1 or !0218AEAC(o): 0; 0218B2A8(req, id, &rp); returns 0218B2E8(req, fm, id, &rp)
fn 0218b0cc ret=u32 inputs=200
  arg work 0x500
  arg u32 any
  arg work 0
  fix work 0x514 u32 addrof work 0
  fix work 0x51C u32 alt oneof 0 1 2 3 || any
  fix work 0x520 u32 alt oneof 0 1 2 || any
  fix work 0x524 u16 alt range 0 3 || oneof 9 4
  fix work 0x526 u16 alt const 0 || any
  fix work 0x52C u32 alt oneof 0 0x10 0x20 0x40 0x80 || any
  fix work 0x530 u16 alt range 0 3 || const 9
  fix work 0x534 u32 addrof work 0x600
  fix work 0x600 u32 alt range 0 0x40000 || any
  fix work 0x608 u32 alt range 0 0x40000 || any
  stub 02188cb4 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021a3074 force ret=u32 args=v  # ov21
  stub 0219e2f4 force ret=none args=v,outm:8:0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF
  stub 02162598 force ret=pick:0/1/0xFFFF args=v,in:8
  stub 021625cc force ret=u32 args=v,v  # ov10
  stub 021626ec force ret=pick:0/1 args=v
  stub 0218aeac ret=pick:0/1 args=v  # in-area
  stub 0218b2a8 ret=none args=v,v,p  # in-area
  stub 0218b2e8 ret=pick:0/0/0/0/0/0/1 args=v,v,v,p  # in-area
  stub 021626e8 force ret=pick:0/1/5/6/7 args=v
  stub 02188c9c force ret=u16 args=v  # fieldmap (other F2a area)
  stub 02014134 force ret=none args=out:0x1C,v,v,v,v,v,v,v  # LOCATION init, rail form (loc, id, b, c, d, e, f, g)
  stub 02012878 force ret=none args=v,in:0x1C  # store a LOCATION in the game data (gd, &loc)
  stub 02162bcc force ret=u32 args=v,in:8  # ov10
  stub 02185360 force ret=u32 args=v,p,v,v  # ov20

# 0218B13C(req, gs, fm, rp2): rail position rp of the player (0219E2F4); id = 02162598(evdata, &rp) != 0xFFFF && 021626EC(o) == 0
# && 021626B8(o, facing) && 021626E8(o) == 1: 0218B2A8 / 0218B2E8 with rp; else id = 02162598(evdata, rp2): != 0xFFFF &&
# 021626EC(o) == 0 && 021626B8(o, facing): 0218B2A8(req, id, &rp), 0218B2E8(req, fm, id, &rp); else 0
fn 0218b13c ret=u32 inputs=300
  arg work 0x500
  arg u32 any
  arg work 0
  arg work 0x600
  fix work 0x514 u32 addrof work 0
  fix work 0x51C u32 alt oneof 0 1 2 3 || any
  fix work 0x520 u32 alt oneof 0 1 2 || any
  fix work 0x524 u16 alt range 0 3 || oneof 9 4
  fix work 0x526 u16 alt const 0 || any
  fix work 0x52C u32 alt oneof 0 0x10 0x20 0x40 0x80 || any
  fix work 0x530 u16 alt range 0 3 || const 9
  fix work 0x534 u32 addrof work 0x600
  fix work 0x600 u32 alt range 0 0x40000 || any
  fix work 0x608 u32 alt range 0 0x40000 || any
  stub 02188cb4 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021a3074 force ret=u32 args=v  # ov21
  stub 0219e2f4 force ret=none args=v,outm:8:0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF
  stub 02162598 force ret=pick:0/1/0xFFFF args=v,in:8
  stub 021625cc force ret=u32 args=v,v  # ov10
  stub 021626ec force ret=pick:0/1 args=v
  stub 021626b8 force ret=pick:0/1 args=v,v
  stub 021626e8 force ret=pick:0/1/5/6/7 args=v
  stub 0218b2a8 ret=none args=v,v,p  # in-area
  stub 0218b2e8 ret=pick:0/0/0/0/0/0/1 args=v,v,v,p  # in-area
  stub 02188c9c force ret=u16 args=v  # fieldmap (other F2a area)
  stub 02014134 force ret=none args=out:0x1C,v,v,v,v,v,v,v  # LOCATION init, rail form (loc, id, b, c, d, e, f, g)
  stub 02012878 force ret=none args=v,in:0x1C  # store a LOCATION in the game data (gd, &loc)
  stub 02162bcc force ret=u32 args=v,in:8  # ov10
  stub 02185360 force ret=u32 args=v,p,v,v  # ov20

# 0218B208(req, gs, fm): e = 02188CE8(fm); !FLAGS bit 3: 0; 02013DD0(zone) ? 021AA248(e, 0218DC70(02188C98(fm))) : 021A9F40(e, 0)
# (the wild encounter checks)
fn 0218b208 ret=u32 inputs=200
  arg work 0x500
  arg u32 any
  arg work 0
  fix work 0x514 u32 addrof work 0
  fix work 0x51C u32 alt oneof 0 1 2 3 || any
  fix work 0x520 u32 alt oneof 0 1 2 || any
  fix work 0x524 u16 alt range 0 3 || oneof 9 4
  fix work 0x526 u16 alt const 0 || any
  fix work 0x52C u32 alt oneof 0 0x10 0x20 0x40 0x80 || any
  fix work 0x530 u16 alt range 0 3 || const 9
  fix work 0x534 u32 addrof work 0x600
  fix work 0x600 u32 alt range 0 0x40000 || any
  fix work 0x608 u32 alt range 0 0x40000 || any
  stub 02188ce8 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 02013dd0 force ret=pick:0/1 args=v
  stub 02188c98 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 0218dc70 force ret=u32 args=v  # ov21
  stub 021aa248 force ret=u32 args=v,v  # ov21
  stub 021a9f40 force ret=u32 args=v,v  # ov21

# 0218B24C(req, gs, fm): p = 02188CB4(fm); d = 021A2FA4(p, trigger); d == 1 && state 0 or 3 && 021BB188(021AB0F0(+0x38)) &&
# (0219E418(021A3074(p), d) & 3): 021C46F8(gs, fm, d); else 0
fn 0218b24c ret=u32 inputs=300
  arg work 0x500
  arg u32 any
  arg work 0
  fix work 0x514 u32 addrof work 0
  fix work 0x51C u32 alt oneof 0 1 2 3 || any
  fix work 0x520 u32 alt oneof 0 1 2 || any
  fix work 0x524 u16 alt range 0 3 || oneof 9 4
  fix work 0x526 u16 alt const 0 || any
  fix work 0x52C u32 alt oneof 0 0x10 0x20 0x40 0x80 || any
  fix work 0x530 u16 alt range 0 3 || const 9
  fix work 0x534 u32 addrof work 0x600
  fix work 0x600 u32 alt range 0 0x40000 || any
  fix work 0x608 u32 alt range 0 0x40000 || any
  stub 02188cb4 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021a3074 force ret=u32 args=v  # ov21
  stub 021a2fa4 force ret=pick:0/1/2/9 args=v,v
  stub 021ab0f0 force ret=u32 args=v  # ov21
  stub 021bb188 force ret=pick:0/1 args=v
  stub 0219e424 force ret=u32 args=v,v,v  # ov21
  stub 021c46f8 force ret=u32 args=v,v,v  # ov21
  stub 0219e418 force ret=pick:0/1/2/4 args=v,v

# 0218B2A8(req, id, rp): 02014134(&loc, 02188C9C(fm), (s16) id, 0, 0, rp u16 +0, rp u16 +6, rp s16 +4); 02012878(gd, &loc)
fn 0218b2a8 ret=none inputs=100
  arg work 0x500
  arg u32 any
  arg work 0x600
  fix work 0x514 u32 addrof work 0
  fix work 0x51C u32 alt oneof 0 1 2 3 || any
  fix work 0x520 u32 alt oneof 0 1 2 || any
  fix work 0x524 u16 alt range 0 3 || oneof 9 4
  fix work 0x526 u16 alt const 0 || any
  fix work 0x52C u32 alt oneof 0 0x10 0x20 0x40 0x80 || any
  fix work 0x530 u16 alt range 0 3 || const 9
  fix work 0x534 u32 addrof work 0x600
  fix work 0x600 u32 alt range 0 0x40000 || any
  fix work 0x608 u32 alt range 0 0x40000 || any
  stub 02188c9c force ret=u16 args=v  # fieldmap (other F2a area)
  stub 02014134 force ret=none args=out:0x1C,v,v,v,v,v,v,v  # LOCATION init, rail form (loc, id, b, c, d, e, f, g)
  stub 02012878 force ret=none args=v,in:0x1C  # store a LOCATION in the game data (gd, &loc)

# 0218B2E8(req, fm, id, rp): o = 021625CC(evdata, (u16) id); 02185360(gamesys, fm, o, 021626B0(o, rp))
fn 0218b2e8 ret=u32 inputs=100
  arg work 0x500
  arg work 0
  arg u32 any
  arg work 0x600
  fix work 0x514 u32 addrof work 0
  fix work 0x51C u32 alt oneof 0 1 2 3 || any
  fix work 0x520 u32 alt oneof 0 1 2 || any
  fix work 0x524 u16 alt range 0 3 || oneof 9 4
  fix work 0x526 u16 alt const 0 || any
  fix work 0x52C u32 alt oneof 0 0x10 0x20 0x40 0x80 || any
  fix work 0x530 u16 alt range 0 3 || const 9
  fix work 0x534 u32 addrof work 0x600
  fix work 0x600 u32 alt range 0 0x40000 || any
  fix work 0x608 u32 alt range 0 0x40000 || any
  stub 021625cc force ret=u32 args=v,v  # ov10
  stub 02162bcc force ret=u32 args=v,in:8  # ov10
  stub 02185360 force ret=u32 args=v,p,v,v  # ov20

# 0218B310(player, req): (0219E418(021A3074(player), facing) & 3) == 0
fn 0218b310 ret=u32 inputs=100
  arg u32 any
  arg work 0x500
  fix work 0x514 u32 addrof work 0
  fix work 0x51C u32 alt oneof 0 1 2 3 || any
  fix work 0x520 u32 alt oneof 0 1 2 || any
  fix work 0x524 u16 alt range 0 3 || oneof 9 4
  fix work 0x526 u16 alt const 0 || any
  fix work 0x52C u32 alt oneof 0 0x10 0x20 0x40 0x80 || any
  fix work 0x530 u16 alt range 0 3 || const 9
  fix work 0x534 u32 addrof work 0x600
  fix work 0x600 u32 alt range 0 0x40000 || any
  fix work 0x608 u32 alt range 0 0x40000 || any
  stub 021a3074 force ret=u32 args=v  # ov21
  stub 0219e424 force ret=u32 args=v,v,v  # ov21
  stub 0219e418 force ret=pick:0/1/2/4 args=v,v

# 0218B32C(ev, seq, w) (GMEVENT callback of 0218B3DC; w: +4 u16 dir, +6 u16 flag, +8 gamesys, +0xC fieldmap): p = 02188CB4(w+0xC);
# 021A3098(p); m = 021A3074(p); *seq 0: 0216D9A0(0216D67C()); 0216D5E0(m, dir); 1: 021ACBB8(021A2F98(p), 0); 021A2F48(p, 1);
# 021A2F60(p); a = 02197E7C(dir, flag == 1 ? 0x38 : 0x34); 0216D410(m, a); 2: 0216D454(m) == 1: 021A2FD0(p); 3: 0216D9E4(0216D67C());
# 021A35A0(p, 1); return 1. Steps 0..2 advance *seq (step 2 only when done) and return 0
fn 0218b32c ret=u32 inputs=200
  arg u32 any
  arg work 0x780
  arg work 0x700
  fix work 0x780 u32 alt range 0 3 || range 0 5
  fix work 0x706 u16 alt range 0 1 || any
  fix work 0x70C u32 addrof work 0
  stub 02188cb4 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021a3098 force ret=pick:0/1/2 args=v
  stub 021a3074 force ret=u32 args=v  # ov21
  stub 0216d67c force ret=u32 args=v  # ov10
  stub 0216d9e4 force ret=none args=v  # ov10
  stub 021a35a0 force ret=none args=v,v  # ov21
  stub 0216d454 force ret=pick:0/1 args=v
  stub 021a2fd0 force ret=none args=v  # ov21
  stub 021a2f98 force ret=u32 args=v  # ov21
  stub 021acbb8 force ret=none args=v,v  # ov21
  stub 021a2f48 force ret=none args=v  # ov21
  stub 021a2f60 force ret=none args=v  # ov21
  stub 02197e7c force ret=u32 args=v,v
  stub 0216d410 force ret=none args=v,v  # ov10
  stub 0216d9a0 force ret=none args=v  # ov10
  stub 0216d5e0 force ret=none args=v,v  # ov10

# 0218B3DC(req, gs, fm, dir, flag (stack)): ev = 020122C0(gs, 0, 0x0218B32D, 0x10); w = 020124E8(ev); w+8 = gs; w+0xC = fm;
# w+4 = dir (u16); w+6 = flag (u16); returns ev
fn 0218b3dc ret=u32 inputs=100
  arg work 0x500
  arg u32 any
  arg work 0
  arg u32 any
  arg u32 any
  stub 020122c0 force ret=u32 args=v,v,v,v  # main
  stub 020124e8 force ret=ptr:work:0x700:1 args=v

# 0218B408(req, gs, fm) (ledge / jump): p = 02188CB4(fm); 021A3098(p) != 2: 0; d = 021A2FA4(player, trigger); d == 9: 0;
# 021A3364(p, d, &t); a = 021AB0D4(02188C98(fm), &t); bits = 021AB0F8(a); j = 021AB2E8(021AB0F0(a)) == 1: 02164208(d, &t, 0x10000),
# bits = 021AB0F8(021AB0D4(.., &t)); 0218B6F8(req, fm) == 1 && !(bits & 3): 0218B3DC(req, gs, fm, d, j); else 0
fn 0218b408 ret=u32 inputs=300
  arg work 0x500
  arg u32 any
  arg work 0
  fix work 0x514 u32 addrof work 0
  fix work 0x51C u32 alt oneof 0 1 2 3 || any
  fix work 0x520 u32 alt oneof 0 1 2 || any
  fix work 0x524 u16 alt range 0 3 || oneof 9 4
  fix work 0x526 u16 alt const 0 || any
  fix work 0x52C u32 alt oneof 0 0x10 0x20 0x40 0x80 || any
  fix work 0x530 u16 alt range 0 3 || const 9
  fix work 0x534 u32 addrof work 0x600
  fix work 0x600 u32 alt range 0 0x40000 || any
  fix work 0x608 u32 alt range 0 0x40000 || any
  stub 02188cb4 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021a3098 force ret=pick:0/1/2/2 args=v
  stub 02188c98 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021a2fa4 force ret=pick:0/1/2/3/9 args=v,v
  stub 021a3364 force ret=none args=v,v,out:12  # position in direction d (player, d, &out)
  stub 021ab0d4 force ret=u32 args=v,in:12
  stub 021ab0f8 force ret=pick:0/1/2/4 args=v
  stub 021ab0f0 force ret=u32 args=v  # ov21
  stub 021ab2e8 force ret=pick:0/1 args=v
  stub 02164208 force ret=none args=v,in:12,v
  stub 0218b6f8 ret=pick:0/1 args=v,v  # in-area
  stub 0218b3dc ret=pick:0/1 args=p,v,v,v,v  # in-area
  stub 021a3074 force ret=u32 args=v  # ov21
  stub 0216d83c force ret=u32 args=v  # ov10
  stub 0216d86c force ret=u32 args=v  # ov10
  stub 021641e8 force ret=u32 args=v  # ov10
  stub 021641f8 force ret=u32 args=v  # ov10
  stub 02188c7c force ret=u32 args=v  # fieldmap (other F2a area)
  stub 0216ddc0 force ret=pick:0/0/0/1 args=v,v,v,v
  stub 020122c0 force ret=u32 args=v,v,v,v  # main
  stub 020124e8 force ret=ptr:work:0x700:1 args=v

# 0218B4B0(req, fm): m = 021A3074(player); 021A3460(player, &x, &y, &z) (s16 tile x, y, z); 02164294(x, y, z, &pos);
# 021AB194(021AB0F0(021AB0D4(02188C98(fm), &pos))) == 1: 02164240(0216D5F8(m), &x, &z, 1); returns 0216DDDC(02188C7C(fm), x, z,
# 0216D8AC(m), 0x10000, 0) (the MMDL in front, for talk)
fn 0218b4b0 ret=u32 inputs=200
  arg work 0x500
  arg work 0
  fix work 0x514 u32 addrof work 0
  fix work 0x51C u32 alt oneof 0 1 2 3 || any
  fix work 0x520 u32 alt oneof 0 1 2 || any
  fix work 0x524 u16 alt range 0 3 || oneof 9 4
  fix work 0x526 u16 alt const 0 || any
  fix work 0x52C u32 alt oneof 0 0x10 0x20 0x40 0x80 || any
  fix work 0x530 u16 alt range 0 3 || const 9
  fix work 0x534 u32 addrof work 0x600
  fix work 0x600 u32 alt range 0 0x40000 || any
  fix work 0x608 u32 alt range 0 0x40000 || any
  stub 021a3074 force ret=u32 args=v  # ov21
  stub 021a3460 force ret=none args=v,outm:2:0x7/0,outm:2:0x7/0,outm:2:0x7/0
  stub 02164294 force ret=none args=v,v,v,out:12  # tile -> fx32 position (x, y, z, &out)
  stub 02188c98 force ret=u32 args=v  # fieldmap (other F2a area)
  stub 021ab0d4 force ret=u32 args=v,in:12
  stub 021ab0f0 force ret=u32 args=v  # ov21
  stub 021ab194 force ret=pick:0/1 args=v
  stub 0216d5f8 force ret=u32 args=v  # ov10
  stub 02164240 force ret=none args=v,outm:2:0xFF/0xFF,outm:2:0xFF/0xFF,v  # ov10 (dir, &x, &z, n): moves the s16 tile x / z
  stub 02188c7c force ret=u32 args=v  # fieldmap (other F2a area)
  stub 0216d8ac force ret=u32 args=v  # ov10
  stub 0216dddc force ret=u32 args=v,v,v,v,v,v  # ov10

# 0218B53C(req, fm): 0219E634(021A3074(player), &rp) ? 0219E570(02188C7C(fm), &rp, 0) : 0
fn 0218b53c ret=u32 inputs=100
  arg work 0x500
  arg work 0
  fix work 0x514 u32 addrof work 0
  fix work 0x51C u32 alt oneof 0 1 2 3 || any
  fix work 0x520 u32 alt oneof 0 1 2 || any
  fix work 0x524 u16 alt range 0 3 || oneof 9 4
  fix work 0x526 u16 alt const 0 || any
  fix work 0x52C u32 alt oneof 0 0x10 0x20 0x40 0x80 || any
  fix work 0x530 u16 alt range 0 3 || const 9
  fix work 0x534 u32 addrof work 0x600
  fix work 0x600 u32 alt range 0 0x40000 || any
  fix work 0x608 u32 alt range 0 0x40000 || any
  stub 021a3074 force ret=u32 args=v  # ov21
  stub 0219e634 force ret=pick:0/1 args=v,out:8
  stub 02188c7c force ret=u32 args=v  # fieldmap (other F2a area)
  stub 0219e570 force ret=u32 args=v,in:8,v  # ov21 (sys, &rail position, flag)

# 0218B56C(req): 02012B70(gd) == 1: 0xFFFF; a = 021A30C8(player); b = 021A3128(player, facing); t = 021AB0F0(..);
# for the 13 ROM entries {fn, u16 dir, u16 script} at 0x021D3CE8: (dir == facing or dir == 9) && fn(t): script;
# 02013E04(zone) == 0 or 02012B70(gd) == 1: 0xFFFF; 021A3504(player, a, b) && 0218B630(gd, 0x39) != 6: 0x2712;
# 021A3524(player, a, b): 0x2716; else 0xFFFF
fn 0218b56c ret=u32 inputs=300
  arg work 0x500
  fix work 0x514 u32 addrof work 0
  fix work 0x51C u32 alt oneof 0 1 2 3 || any
  fix work 0x520 u32 alt oneof 0 1 2 || any
  fix work 0x524 u16 alt range 0 3 || oneof 9 4
  fix work 0x526 u16 alt const 0 || any
  fix work 0x52C u32 alt oneof 0 0x10 0x20 0x40 0x80 || any
  fix work 0x530 u16 alt range 0 3 || const 9
  fix work 0x534 u32 addrof work 0x600
  fix work 0x600 u32 alt range 0 0x40000 || any
  fix work 0x608 u32 alt range 0 0x40000 || any
  stub 02012b70 force ret=pick:0/1/2 args=v
  stub 021a30c8 force ret=u32 args=v  # ov21
  stub 021a3128 force ret=u32 args=v,v  # ov21
  stub 021ab0f0 force ret=u32 args=v  # ov21
  stub 02013e04 force ret=pick:0/1 args=v
  stub 021a3504 force ret=pick:0/1 args=v,v,v
  stub 0218b630 ret=pick:0/1/6 args=v,v  # in-area
  stub 021a3524 force ret=pick:0/1 args=v,v,v
  stub 021ab1a0 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab1ac force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab1b8 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab1d0 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab1dc force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab1e8 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab1f4 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab200 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab20c force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab218 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab224 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab230 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 021ab1c4 force ret=pick:0/0/0/1 args=v  # attribute predicate (0218B56C table)
  stub 02012934 force ret=u32 args=v  # main
  stub 0201aa18 force ret=pick:0/1/2 args=v
  stub 0201ab28 force ret=u32 args=v,v  # main
  stub 02017e1c force ret=pick:0/1/0x39 args=v,v,v

# 0218B630(gd, move): party = 02012934(gd); first i < 0201AA18(party) of a non-egg mon (02017E1C(m, 0x4C, 0) == 0) with move
# among params 0x36..0x39: i; else 6
fn 0218b630 ret=u32 inputs=300
  arg u32 any
  arg u32 alt oneof 0x39 0x7F || any
  stub 02012934 force ret=u32 args=v  # main
  stub 0201aa18 force ret=pick:0/1/2/6 args=v
  stub 0201ab28 force ret=u32 args=v,v  # main
  stub 02017e1c force ret=pick:0/1/0x39/0x7F args=v,v,v

# 0218B6AC(gs): party = 02012934(020120F4(gs)); count of i < 0201AA18(party) with 02017E1C(m, 0x4C, 0) == 0 && 02017E1C(m, 0xA0, 0) != 0
fn 0218b6ac ret=u32 inputs=200
  arg u32 any
  stub 020120f4 force ret=u32 args=v  # main
  stub 02012934 force ret=u32 args=v  # main
  stub 0201aa18 force ret=pick:0/1/2/6 args=v
  stub 0201ab28 force ret=u32 args=v,v  # main
  stub 02017e1c force ret=pick:0/1 args=v,v,v

# 0218B6F8(req, fm): m = 021A3074(player); d = trigger dir (+0x30); x = 0216D83C(m); z = 0216D86C(m); twice: x += 021641E8(d),
# z += 021641F8(d) (s16); 0216DDC0(02188C7C(fm), x, z, 1) != 0: 0; then 1 (both tiles free)
fn 0218b6f8 ret=u32 inputs=200
  arg work 0x500
  arg work 0
  fix work 0x514 u32 addrof work 0
  fix work 0x51C u32 alt oneof 0 1 2 3 || any
  fix work 0x520 u32 alt oneof 0 1 2 || any
  fix work 0x524 u16 alt range 0 3 || oneof 9 4
  fix work 0x526 u16 alt const 0 || any
  fix work 0x52C u32 alt oneof 0 0x10 0x20 0x40 0x80 || any
  fix work 0x530 u16 alt range 0 3 || const 9
  fix work 0x534 u32 addrof work 0x600
  fix work 0x600 u32 alt range 0 0x40000 || any
  fix work 0x608 u32 alt range 0 0x40000 || any
  stub 021a3074 force ret=u32 args=v  # ov21
  stub 0216d83c force ret=u32 args=v  # ov10
  stub 0216d86c force ret=u32 args=v  # ov10
  stub 021641e8 force ret=u32 args=v  # ov10
  stub 021641f8 force ret=u32 args=v  # ov10
  stub 02188c7c force ret=u32 args=v  # fieldmap (other F2a area)
  stub 0216ddc0 force ret=pick:0/0/0/1 args=v,v,v,v
