# fndiff spec: the field-skill (HM / field move) table code of overlay 10 (Thumb code 0x0215E840..0x0215F168, the second part of
# F1 row 10.19 "move_pokemon.c"; F5b). No __FILE__ string of its own (move_pokemon.c's only allocation is in 0215E440, and
# scrcmd_network.c starts at 0x0215F168); the name is by role (UNVERIFIED). Grammar: svflow_specs/README.md (generic mode).
# Run: tools/check-fn.sh black-field field_skill --per-function
#
# Function list: bl/blx targets of the whole field set, odd code pointers (the two skill tables), the Ghidra index and a Thumb walk
# from every entry: 43 entries (Ghidra misses 0215EA30, 0215EA58, 0215EB60, 0215EBD0, 0215EC40, 0215EC80, 0215EC94, 0215EDA0,
# 0215EE38, 0215EED0, 0215EF40 and splits 0215ED04 / 0215ED2A / 0215ED72 / 0215EE0A / 0215EEA2 / 0215EF8A / 0215EFB0 / 0215F01E /
# 0215F044 out of their procs). Render-only functions: none.
# Closure stubs (as core4 / F4a / F5a): a block lists the stubs of its callees and of the callees of its in-area descendants (also the
# table targets of 0215EA14 / 0215EA38), so the area can be ported in any order (checked with every function treated as ported: no
# fault, stray write or unstubbed callee). Every out-of-area stub is `force`.
#
# SKILL CTX (0x14, built by 0215E840(fm, ctx)): +0 u16 zone (02188C9C(fm)), +2 u16 usable flags (bit k = skill k may be used: 0 the
# front OBJ is code 0x6C, 1 / 2 021A3504 / 021A3524 on the player's front tile, 3 a front OBJ that 0216E5FC accepts and nothing blocks
# (0216E5B0) or the rect 0x021711F4 (flag 0x112), 4 + 6 02013C40(zone), 5 bit 0 of 02014470(EVENT WORK obj) or the rect 0x021711EC
# (flag 0x113), 7 02013C50(zone), 8 + 9 the zone allows field moves (02013E04; otherwise every flag is cleared), 10 021CD440(fm);
# all cleared when 02012B70(GAMEDATA) == 1), +4 player form (021A3098: 2 / 3 checked by skill 1), +8 GAMESYS (02188C80), +0xC the
# front OBJ (021A3250, 0 = none), +0x10 FIELDMAP.
# Skill tables (ROM, 11 x {event fn, check fn}): 0x021711FC (normal zones) and 0x02171254 (zones with 02013FFC(zone) != 0: skills 3 and
# 5 use the events 0215EFCC / 0215F060). 0215F088(ctx, k) -> check fn, 0215F0B8(ctx, k) -> event fn (k >= 11 -> 0; a negative k reads
# before the table: not drawn). Check fn(ctx) -> 0 usable, else a reason (1, or 4 for skill 1 in form 2). Event fn(param, ctx) -> a
# GMEVENT: param = {u16 a, u16 b, u32 c} (0215EA30); most events create a 0x14-byte work (0215F0E8: +0 0x19740205, +4 ctx+0xC, +8
# GAMESYS, +0xC param.a|b, +0x10 param.c) whose proc starts the skill script (021587B0 / 02158780 + 02159058(script, (u16) +0xC, 0, 0,
# 0)); others hand off to overlay-21 events (021C6078, 021C8268, 021D1C24). 02165534(n) is game_beacon (X1, force stub).
# All blocks are `nowork`; objects are `arg buf` arenas or stub answers.
area field_skill
set field

# 0215E840(fm, ctx): fill the SKILL CTX (see the header). Locals: the OBJ position (0216D88C out 12 -> 0216E5B0 in 12), the
# 021CD440 out word, the player grid x / y / z (021A35FC: three u16 outs; x and z go to 0215F118 with the rects).
fn 0215e840 ret=none inputs=1000
  nowork
  arg u32 any
  arg buf 0x20
  stub 02188c9c force ret=pick+u16:0/1/0x3f/0x108 args=v  # FIELDMAP zone
  stub 02188c80 force ret=u32 args=v  # FIELDMAP -> GAMESYS
  stub 02188cb4 force ret=u32 args=v  # FIELDMAP -> player
  stub 021a3098 force ret=pick:0/1/2/3 args=v  # player form
  stub 021a3250 force ret=nullable:3:u32 args=v  # player: front OBJ (0 = none)
  stub 0216d598 force ret=pick+u32:0x6C/0x6D/0x24/0 args=v  # OBJ code
  stub 02188c7c force ret=u32 args=v  # FIELDMAP -> MMDLSYS
  stub 0216d88c force ret=none args=v,out:12  # OBJ position (m, &pos)
  stub 0216e5fc force ret=pick:0/1 args=v  # OBJ code kind test
  stub 0216e5b0 force ret=pick:0/1 args=v,in:12  # MMDLSYS: something at pos (sys, &pos)
  stub 021a3050 force ret=pick:0/1/2/3 args=v  # player direction
  stub 021a30c8 force ret=u32 args=v  # player attribute
  stub 021a3128 force ret=u32 args=v,v  # player front attribute (player, dir)
  stub 021a3504 force ret=pick+u32:0/1/2 args=v,v,v  # (player, a, b) skill-1 test
  stub 021a3524 force ret=pick+u32:0/1/2 args=v,v,v  # (player, a, b) skill-2 test
  stub 02013c40 force ret=pick+u32:0/1/2 args=v  # zonedata (zone)
  stub 02013c50 force ret=pick+u32:0/1/2 args=v  # zonedata (zone)
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 02012b38 force ret=u32 args=v  # GAMEDATA -> EVENT WORK +0x1B8 object
  stub 02014470 force ret=pick+u32:0/1/2/3 args=v  # (obj) flags
  stub 021cd440 force ret=pick+u32:0/1/2 args=v,out:2  # (fm, &out)
  stub 02013ffc force ret=pick:0/1 args=v  # zonedata (zone)
  stub 021a35fc force ret=none args=v,outv:2:13/14/15/39/40/41/7,outv:2:0/1,outv:2:24/25/26/27/28/3  # player grid (p, &x, &y, &z)
  stub 0201400c force ret=pick:0/1 args=v  # zonedata (zone)
  stub 02014018 force ret=pick:0/1 args=v  # zonedata (zone)
  stub 0215f118 ret=pick:0/1 args=v,v,v,v,v  # in-area: in rect (rect, x, z, flag, fm)
  stub 02013e04 force ret=pick+u32:0/1 args=v  # zonedata (zone): field moves allowed
  stub 02012b70 force ret=pick+u8:0/1/2 args=v  # GAMEDATA u8 +0x1CC
  # descendants (0215F118)
  stub 0201296c force ret=u32 args=v  # GAMEDATA -> EVENT WORK
  stub 020142e8 force ret=pick:0/1 args=v,v  # event flag check (ew, flag)

# 0215EA14(k, ctx) -> f = 0215F088(ctx, k); f ? f(ctx) : 1
fn 0215ea14 ret=u32 inputs=400
  nowork
  arg u32 alt range 0 10 || range 0 12
  arg buf 0x20
  fix arg1 2 u16 any
  fix arg1 4 u32 alt oneof 2 3 || any
  stub 0215f088 ret=pick:0/0x0215EA59/0x0215EAC9/0x0215EB4D/0x0215EBBD/0x0215EC6D/0x0215EC81/0x0215EE25/0x0215ED8D/0x0215EC2D/0x0215EF2D/0x0215EEBD args=v,v  # in-area: check fn
  stub 0215ea58 ret=pick:0/1 args=v  # in-area: check fns
  stub 0215eac8 ret=pick:0/1/4 args=v
  stub 0215eb4c ret=pick:0/1 args=v
  stub 0215ebbc ret=pick:0/1 args=v
  stub 0215ec6c ret=pick:0/1 args=v
  stub 0215ec80 ret=pick:0/1 args=v
  stub 0215ee24 ret=pick:0/1 args=v
  stub 0215ed8c ret=pick:0/1 args=v
  stub 0215ec2c ret=pick:0/1 args=v
  stub 0215ef2c ret=pick:0/1 args=v
  stub 0215eebc ret=pick:0/1 args=v
  # descendants
  stub 02013ffc force ret=pick:0/1 args=v  # zonedata (zone)
  stub 0215f104 ret=pick:0/1 args=v,v  # in-area: flag test

# 0215EA30(p, a, b, c): p+0 = (u16) a, p+2 = (u16) b, p+4 = c
fn 0215ea30 ret=none inputs=60
  nowork
  arg buf 0x10
  arg u32 any
  arg u32 any
  arg u32 any

# 0215EA38(k, param, ctx) -> f = 0215F0B8(ctx, k); f ? f(param, ctx) : 0. Arena: arg 3 (the GMEVENT works of the events).
fn 0215ea38 ret=u32 inputs=600
  nowork
  arg u32 alt range 0 10 || range 0 12
  arg buf 0x10
  arg buf 0x20
  arg buf 0x40
  stub 0215f0b8 ret=pick:0/0x0215EA6D/0x0215EAF1/0x0215EB61/0x0215EBD1/0x0215EC55/0x0215EC95/0x0215EE39/0x0215EDA1/0x0215EC41/0x0215EF41/0x0215EED1/0x0215EFCD/0x0215F061 args=v,v  # in-area: event fn
  stub 0215ea6c ret=u32 args=v,v  # in-area: event fns
  stub 0215eaf0 ret=u32 args=v,v
  stub 0215eb60 ret=u32 args=v,v
  stub 0215ebd0 ret=u32 args=v,v
  stub 0215ec54 ret=u32 args=v,v
  stub 0215ec94 ret=u32 args=v,v
  stub 0215ee38 ret=u32 args=v,v
  stub 0215eda0 ret=u32 args=v,v
  stub 0215ec40 ret=u32 args=v,v
  stub 0215ef40 ret=u32 args=v,v
  stub 0215eed0 ret=u32 args=v,v
  stub 0215efcc ret=u32 args=v,v
  stub 0215f060 ret=u32 args=v,v
  # descendants (0215F0B8, the event fns, 0215F0E8)
  stub 02013ffc force ret=pick:0/1 args=v  # zonedata (zone)
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg3:0:1 args=v  # GMEVENT work (event)
  stub 0215f0e8 ret=none args=v,v,v  # in-area: work init
  stub 02165534 force ret=none args=v  # game_beacon (X1): skill used
  stub 021c8268 force ret=u32 args=v,v,v  # ov21 event (gsys, fm, param.c)
  stub 021c6078 force ret=u32 args=v,v,v  # ov21 event (gsys, fm, (u8) param.a)
  stub 021d1c24 force ret=u32 args=v,v,v  # ov21 event (gsys, fm, (u8) param.a)

# Check fns (ctx) -> 0 usable: 0215F104(ctx, k) set -> 0, else 1 (skills 0, 2, 3, 8, 4, 5, 7, 6, 10, 9 at 0215EA58, 0215EB4C,
# 0215EBBC, 0215EC2C, 0215EC6C, 0215EC80, 0215ED8C, 0215EE24, 0215EEBC, 0215EF2C); 0215EAC8: form +4 == 2 -> 4, == 3 -> 1, else bit 1.
fn 0215ea58 ret=u32 inputs=100
  nowork
  arg buf 0x20
  stub 0215f104 ret=pick:0/1 args=v,v  # in-area: flag test

fn 0215eac8 ret=u32 inputs=200
  nowork
  arg buf 0x20
  fix arg0 4 u32 alt oneof 2 3 || range 0 4 || any
  stub 0215f104 ret=pick:0/1 args=v,v  # in-area: flag test

fn 0215eb4c ret=u32 inputs=100
  nowork
  arg buf 0x20
  stub 0215f104 ret=pick:0/1 args=v,v  # in-area: flag test

fn 0215ebbc ret=u32 inputs=100
  nowork
  arg buf 0x20
  stub 0215f104 ret=pick:0/1 args=v,v  # in-area: flag test

fn 0215ec2c ret=u32 inputs=100
  nowork
  arg buf 0x20
  stub 0215f104 ret=pick:0/1 args=v,v  # in-area: flag test

fn 0215ec6c ret=u32 inputs=100
  nowork
  arg buf 0x20
  stub 0215f104 ret=pick:0/1 args=v,v  # in-area: flag test

fn 0215ec80 ret=u32 inputs=100
  nowork
  arg buf 0x20
  stub 0215f104 ret=pick:0/1 args=v,v  # in-area: flag test

fn 0215ed8c ret=u32 inputs=100
  nowork
  arg buf 0x20
  stub 0215f104 ret=pick:0/1 args=v,v  # in-area: flag test

fn 0215ee24 ret=u32 inputs=100
  nowork
  arg buf 0x20
  stub 0215f104 ret=pick:0/1 args=v,v  # in-area: flag test

fn 0215eebc ret=u32 inputs=100
  nowork
  arg buf 0x20
  stub 0215f104 ret=pick:0/1 args=v,v  # in-area: flag test

fn 0215ef2c ret=u32 inputs=100
  nowork
  arg buf 0x20
  stub 0215f104 ret=pick:0/1 args=v,v  # in-area: flag test

# Script events (param, ctx) -> ev = 020122C0(ctx+8, 0, proc, 0x14); 0215F0E8(020124E8(ev), param, ctx); ev. Procs: 0215EA6C ->
# 0x0215EA95, 0215EAF0 -> 0x0215EB19, 0215EB60 -> 0x0215EB89, 0215EBD0 -> 0x0215EBF9, 0215EC94 -> 0x0215ECBD, 0215EED0 -> 0x0215EEF9,
# 0215EFCC -> 0x0215EF61, 0215F060 -> 0x0215EFF5; 0215EDA0 / 0215EE38 first call 02165534(0x5B / 100) (procs 0x0215EDD1 / 0x0215EE69).
# Arena: arg 2 (the work).
fn 0215ea6c ret=u32 inputs=100
  nowork
  arg buf 0x10
  arg buf 0x20
  arg buf 0x40
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg2:0:1 args=v  # GMEVENT work (event)
  stub 0215f0e8 ret=none args=v,v,v  # in-area: work init

fn 0215eaf0 ret=u32 inputs=100
  nowork
  arg buf 0x10
  arg buf 0x20
  arg buf 0x40
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg2:0:1 args=v  # GMEVENT work (event)
  stub 0215f0e8 ret=none args=v,v,v  # in-area: work init

fn 0215eb60 ret=u32 inputs=100
  nowork
  arg buf 0x10
  arg buf 0x20
  arg buf 0x40
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg2:0:1 args=v  # GMEVENT work (event)
  stub 0215f0e8 ret=none args=v,v,v  # in-area: work init

fn 0215ebd0 ret=u32 inputs=100
  nowork
  arg buf 0x10
  arg buf 0x20
  arg buf 0x40
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg2:0:1 args=v  # GMEVENT work (event)
  stub 0215f0e8 ret=none args=v,v,v  # in-area: work init

fn 0215ec94 ret=u32 inputs=100
  nowork
  arg buf 0x10
  arg buf 0x20
  arg buf 0x40
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg2:0:1 args=v  # GMEVENT work (event)
  stub 0215f0e8 ret=none args=v,v,v  # in-area: work init

fn 0215eda0 ret=u32 inputs=100
  nowork
  arg buf 0x10
  arg buf 0x20
  arg buf 0x40
  stub 02165534 force ret=none args=v  # game_beacon (X1): skill used
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg2:0:1 args=v  # GMEVENT work (event)
  stub 0215f0e8 ret=none args=v,v,v  # in-area: work init

fn 0215ee38 ret=u32 inputs=100
  nowork
  arg buf 0x10
  arg buf 0x20
  arg buf 0x40
  stub 02165534 force ret=none args=v  # game_beacon (X1): skill used
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg2:0:1 args=v  # GMEVENT work (event)
  stub 0215f0e8 ret=none args=v,v,v  # in-area: work init

fn 0215eed0 ret=u32 inputs=100
  nowork
  arg buf 0x10
  arg buf 0x20
  arg buf 0x40
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg2:0:1 args=v  # GMEVENT work (event)
  stub 0215f0e8 ret=none args=v,v,v  # in-area: work init

fn 0215efcc ret=u32 inputs=100
  nowork
  arg buf 0x10
  arg buf 0x20
  arg buf 0x40
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg2:0:1 args=v  # GMEVENT work (event)
  stub 0215f0e8 ret=none args=v,v,v  # in-area: work init

fn 0215f060 ret=u32 inputs=100
  nowork
  arg buf 0x10
  arg buf 0x20
  arg buf 0x40
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg2:0:1 args=v  # GMEVENT work (event)
  stub 0215f0e8 ret=none args=v,v,v  # in-area: work init

# Hand-off events (param, ctx): 0215EC40 -> 021C6078(ctx+8, ctx+0x10, (u8) param.a) (tail call); 0215EC54: 02165534(19),
# 021C8268(ctx+8, ctx+0x10, param.c); 0215EF40: 02165534(0x1C0), 021D1C24(ctx+8, ctx+0x10, (u8) param.a)
fn 0215ec40 ret=u32 inputs=100
  nowork
  arg buf 0x10
  arg buf 0x20
  stub 021c6078 force ret=u32 args=v,v,v  # ov21 event (gsys, fm, (u8) param.a)

fn 0215ec54 ret=u32 inputs=100
  nowork
  arg buf 0x10
  arg buf 0x20
  stub 02165534 force ret=none args=v  # game_beacon (X1): skill used
  stub 021c8268 force ret=u32 args=v,v,v  # ov21 event (gsys, fm, param.c)

fn 0215ef40 ret=u32 inputs=100
  nowork
  arg buf 0x10
  arg buf 0x20
  stub 02165534 force ret=none args=v  # game_beacon (X1): skill used
  stub 021d1c24 force ret=u32 args=v,v,v  # ov21 event (gsys, fm, (u8) param.a)

# Script procs (ev, seq, w) -> 0: 0201296C(020120F4(w+8)) (unused); 021587B0(ev, script, 0, 0) -> s; 02159058(s, (u16) w+0xC, 0, 0, 0).
# Scripts: 0215EA94 0x2715, 0215EB18 0x2713, 0215EB88 0x2717, 0215EBF8 0x2711, 0215EEF8 0x271C.
fn 0215ea94 ret=u32 inputs=100
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0201296c force ret=u32 args=v  # GAMEDATA -> EVENT WORK
  stub 021587b0 force ret=u32 args=v,v,v,v  # script event (ev, script, 0, 0)
  stub 02159058 force ret=none args=v,v,v,v,v  # script param (s, id, 0, 0, 0)

fn 0215eb18 ret=u32 inputs=100
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0201296c force ret=u32 args=v  # GAMEDATA -> EVENT WORK
  stub 021587b0 force ret=u32 args=v,v,v,v  # script event (ev, script, 0, 0)
  stub 02159058 force ret=none args=v,v,v,v,v  # script param (s, id, 0, 0, 0)

fn 0215eb88 ret=u32 inputs=100
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0201296c force ret=u32 args=v  # GAMEDATA -> EVENT WORK
  stub 021587b0 force ret=u32 args=v,v,v,v  # script event (ev, script, 0, 0)
  stub 02159058 force ret=none args=v,v,v,v,v  # script param (s, id, 0, 0, 0)

fn 0215ebf8 ret=u32 inputs=100
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0201296c force ret=u32 args=v  # GAMEDATA -> EVENT WORK
  stub 021587b0 force ret=u32 args=v,v,v,v  # script event (ev, script, 0, 0)
  stub 02159058 force ret=none args=v,v,v,v,v  # script param (s, id, 0, 0, 0)

fn 0215eef8 ret=u32 inputs=100
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0201296c force ret=u32 args=v  # GAMEDATA -> EVENT WORK
  stub 021587b0 force ret=u32 args=v,v,v,v  # script event (ev, script, 0, 0)
  stub 02159058 force ret=none args=v,v,v,v,v  # script param (s, id, 0, 0, 0)

# 0215ECBC(ev, seq, w) (skill 5, Flash-like): gd = 020120F4(w+8), 0201296C(gd), E = 02012B38(gd), x = 021C7FBC(02189A10(fm)),
# y = 02188C84(fm) (fm = 0201210C(w+8), fetched twice); 0: s = 02158780(ev, 0x2718, 0, 0, y), 02159058(s, (u16) w+0xC, 0, 0, 0),
# seq 1; 1: 02014468(E) != 0 -> 1; else 02014464(E, 1), 02165534(0x94), 021C7E50(x, 1), E2 = 02012B38(gd): 0201446C(E2,
# 02014470(E2) & ~1 | 2), seq 2; 2: 021C7E54(x) == 3 -> 1; -> 0
fn 0215ecbc ret=u32 inputs=300
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 3 || any
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0201296c force ret=u32 args=v  # GAMEDATA -> EVENT WORK
  stub 02012b38 force ret=u32 args=v  # GAMEDATA -> EVENT WORK +0x1B8 object
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 02189a10 force ret=u32 args=v  # FIELDMAP getter
  stub 021c7fbc force ret=u32 args=v  # -> light / fog object
  stub 02188c84 force ret=u16 args=v  # FIELDMAP getter
  stub 02158780 force ret=u32 args=v,v,v,v,v  # script event (ev, script, 0, 0, y)
  stub 02159058 force ret=none args=v,v,v,v,v  # script param (s, id, 0, 0, 0)
  stub 02014468 force ret=pick:0/1 args=v  # (obj) flash state
  stub 02014464 force ret=none args=v,v  # (obj, 1)
  stub 02165534 force ret=none args=v  # game_beacon (X1): skill used
  stub 021c7e50 force ret=none args=v,v  # (x, 1)
  stub 02014470 force ret=pick+u32:0/1/2/3 args=v  # (obj) flags
  stub 0201446c force ret=none args=v,v  # (obj, flags)
  stub 021c7e54 force ret=pick+u32:0/1/2/3 args=v  # (x) state

# 0215EDD0 / 0215EE68 (ev, seq, w): 0: s = 02158780(ev, 0x271A / 0x2719, 0, 0, 21), 02159058(s, (u16) w+0xC, 0, 0, 0), seq 1;
# 1: change to 02184EEC / 02184F40(w+8); -> 0
fn 0215edd0 ret=u32 inputs=100
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 2 || any
  stub 02158780 force ret=u32 args=v,v,v,v,v  # script event (ev, script, 0, 0, 21)
  stub 02159058 force ret=none args=v,v,v,v,v  # script param (s, id, 0, 0, 0)
  stub 02184eec force ret=u32 args=v  # (gsys) -> event (ov20)
  stub 0201235c force ret=none args=v,v  # GMEVENT change (event, next)

fn 0215ee68 ret=u32 inputs=100
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 2 || any
  stub 02158780 force ret=u32 args=v,v,v,v,v  # script event (ev, script, 0, 0, 21)
  stub 02159058 force ret=none args=v,v,v,v,v  # script param (s, id, 0, 0, 0)
  stub 02184f40 force ret=u32 args=v  # (gsys) -> event (ov20)
  stub 0201235c force ret=none args=v,v  # GMEVENT change (event, next)

# 0215EF60 / 0215EFF4 (ev, seq, w): 0201296C(020120F4(w+8)); y = 02188C84(0201210C(w+8)); 0: s = 02158780(ev, 0x271E / 0x271D, 0,
# 0, y), 02159058(s, (u16) w+0xC, 0, 0, 0), seq 1; 1: 021587B0(ev, 0x28E3 / 0x28E2, 0, 0); -> 0
fn 0215ef60 ret=u32 inputs=100
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 2 || any
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0201296c force ret=u32 args=v  # GAMEDATA -> EVENT WORK
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 02188c84 force ret=u16 args=v  # FIELDMAP getter
  stub 02158780 force ret=u32 args=v,v,v,v,v  # script event (ev, script, 0, 0, y)
  stub 02159058 force ret=none args=v,v,v,v,v  # script param (s, id, 0, 0, 0)
  stub 021587b0 force ret=u32 args=v,v,v,v  # script event (ev, script, 0, 0)

fn 0215eff4 ret=u32 inputs=100
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 2 || any
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0201296c force ret=u32 args=v  # GAMEDATA -> EVENT WORK
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 02188c84 force ret=u16 args=v  # FIELDMAP getter
  stub 02158780 force ret=u32 args=v,v,v,v,v  # script event (ev, script, 0, 0, y)
  stub 02159058 force ret=none args=v,v,v,v,v  # script param (s, id, 0, 0, 0)
  stub 021587b0 force ret=u32 args=v,v,v,v  # script event (ev, script, 0, 0)

# 0215F088(ctx, k) -> k >= 11 (signed) -> 0; table (02013FFC(ctx zone) ? 0x02171254 : 0x021711FC)[k].check
fn 0215f088 ret=u32 inputs=200
  nowork
  arg buf 0x20
  arg u32 range 0 12
  stub 02013ffc force ret=pick:0/1 args=v  # zonedata (zone)

# 0215F0B8(ctx, k) -> the same with .event
fn 0215f0b8 ret=u32 inputs=200
  nowork
  arg buf 0x20
  arg u32 range 0 12
  stub 02013ffc force ret=pick:0/1 args=v  # zonedata (zone)

# 0215F0E8(w, param, ctx): w+0 = 0x19740205, +4 = ctx+0xC, +0xC = param[0], +0x10 = param[1], +8 = ctx+8
fn 0215f0e8 ret=none inputs=60
  nowork
  arg buf 0x40
  arg buf 0x10
  arg buf 0x20

# 0215F104(ctx, k) -> bit k of the u16 flags at ctx+2 (k >= 32: 0)
fn 0215f104 ret=u32 inputs=200
  nowork
  arg buf 0x20
  arg u32 alt range 0 16 || range 30 40 || any

# 0215F118(rect, x, z, flag, fm) -> event flag `flag` set (02188C80(fm) -> GAMEDATA -> EVENT WORK) -> 0; else |rect.x - x| <
# rect.w and |rect.z - z| < rect.h (s32 differences of u16 fields: rect = {u16 x, z, w, h})
fn 0215f118 ret=u32 inputs=600
  nowork
  arg buf 0x10
  arg u16 alt range 0 0x40 || any
  arg u16 alt range 0 0x40 || any
  arg u32 any
  arg u32 any
  fix arg0 0 u16 alt rel 1 oneof 0xFFFFFFFE 0xFFFFFFFF 0 1 2 || any
  fix arg0 2 u16 alt rel 2 oneof 0xFFFFFFFD 0xFFFFFFFE 0xFFFFFFFF 0 1 2 3 || any
  fix arg0 4 u16 alt range 0 3 || any
  fix arg0 6 u16 alt range 0 3 || any
  stub 02188c80 force ret=u32 args=v  # FIELDMAP -> GAMESYS
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0201296c force ret=u32 args=v  # GAMEDATA -> EVENT WORK
  stub 020142e8 force ret=pick:0/1 args=v,v  # event flag check (ew, flag)
