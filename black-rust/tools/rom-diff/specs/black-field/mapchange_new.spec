# fndiff spec: event_mapchange.c part 2 (overlay 20 Thumb code 0x02184CEC..0x02185694; F6b): the map-change event constructors (MAPCHANGE
# work M init 02184CEC, one constructor per transition kind: position / rail position / warp record / special location / union room /
# comm), the player's LOCATION (021852F0) and the zone entry from the special location (0218562C).
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-field mapchange_new --per-function
# Function list, closure stubs, `force`, the work M and the arena layout: see the header of mapchange_proc.spec.
# Constructors: ev = 020122C0(gsys, 0, proc, 0x9C); M = 020124E8(ev); 02184CEC(M, gsys); the LOCATION at M + 0x10 from the arguments
# (02014184 grid / 020141A8 rail / 02162694 warp record / 02014204 zone) with the direction through 020142A4 (stored back as u16 into
# the caller's stack argument, read as s16); M + 0x2C the door kind, M + 0x40 the season flag, M + 0x30 the zone-change kind.
area mapchange_new
set field

# 02184CEC(M, gsys): M+0 gsys, +4 gd, +8 fm, +0xC u16 02188C9C(fm), +0x40 = +0x44 = 0, +0x48 = +0x4A = season;
# 021AAAF0(02188CE8(fm), 1)
fn 02184cec ret=none inputs=100
  nowork
  arg buf 0x100
  arg u32 any
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02188c9c force ret=pick+u32:0/1/0x3f/0x108 args=v  # FIELDMAP zone
  stub 02188ce8 force ret=u32 args=v  # FIELDMAP -> encount object
  stub 021aaaf0 force ret=none args=v,v  # encount object (x, 1)

# 02184D38(gsys, a1, zone, pos, dir, flag) -> event (proc 0x02184389): LOCATION grid (zone, 020142A4(dir), pos), M+0x2C = 0,
# M+0x40 = flag. Arena: arg 6.
fn 02184d38 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x10
  arg u32 alt range 0 5 || any
  arg u32 any
  arg buf 0x1000
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg6:0:1 args=v  # GMEVENT work (event)
  stub 02184cec ret=none args=v,v  # M init (M, gsys)
  stub 020142a4 force ret=pick+u32:0/1/2/3/0xFFFF args=v  # direction code
  stub 02014184 force ret=none args=out:0x1C,v,v,v,v,v  # LOCATION grid init (loc, zone, dir, x, y, z)
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02188c9c force ret=pick+u32:0/1/0x3f/0x108 args=v  # FIELDMAP zone
  stub 02188ce8 force ret=u32 args=v  # FIELDMAP -> encount object
  stub 021aaaf0 force ret=none args=v,v  # encount object (x, 1)

# 02184D90(gsys, a1, zone, rail, dir, flag) -> event (proc 0x02184389): LOCATION rail (zone, dir, u16 r+0, u16 r+6, s16 r+4).
# Arena: arg 6.
fn 02184d90 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x10
  arg u32 alt range 0 5 || any
  arg u32 any
  arg buf 0x1000
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg6:0:1 args=v  # GMEVENT work (event)
  stub 02184cec ret=none args=v,v  # M init (M, gsys)
  stub 020142a4 force ret=pick+u32:0/1/2/3/0xFFFF args=v  # direction code
  stub 020141a8 force ret=none args=out:0x1C,v,v,v,v,v  # LOCATION rail init (loc, zone, dir, line, front, side)
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02188c9c force ret=pick+u32:0/1/0x3f/0x108 args=v  # FIELDMAP zone
  stub 02188ce8 force ret=u32 args=v  # FIELDMAP -> encount object
  stub 021aaaf0 force ret=none args=v,v  # encount object (x, 1)

# 02184DE8(gsys, a1, zone, rail, dir, flag) -> event (proc 0x02184505): as 02184D90. Arena: arg 6.
fn 02184de8 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x10
  arg u32 alt range 0 5 || any
  arg u32 any
  arg buf 0x1000
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg6:0:1 args=v  # GMEVENT work (event)
  stub 02184cec ret=none args=v,v  # M init (M, gsys)
  stub 020142a4 force ret=pick+u32:0/1/2/3/0xFFFF args=v  # direction code
  stub 020141a8 force ret=none args=out:0x1C,v,v,v,v,v  # LOCATION rail init (loc, zone, dir, line, front, side)
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02188c9c force ret=pick+u32:0/1/0x3f/0x108 args=v  # FIELDMAP zone
  stub 02188ce8 force ret=u32 args=v  # FIELDMAP -> encount object
  stub 021aaaf0 force ret=none args=v,v  # encount object (x, 1)

# 02184E40(gsys, a1, p, zone, pos) -> event (proc 0x02184731): LOCATION grid (zone, 1, pos), M+0x34 = the 12 bytes at p,
# M+0x2C = 0. Arena: arg 5.
fn 02184e40 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x10
  arg u32 any
  arg buf 0x10
  arg buf 0x1000
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg5:0:1 args=v  # GMEVENT work (event)
  stub 02184cec ret=none args=v,v  # M init (M, gsys)
  stub 02014184 force ret=none args=out:0x1C,v,v,v,v,v  # LOCATION grid init (loc, zone, dir, x, y, z)
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02188c9c force ret=pick+u32:0/1/0x3f/0x108 args=v  # FIELDMAP zone
  stub 02188ce8 force ret=u32 args=v  # FIELDMAP -> encount object
  stub 021aaaf0 force ret=none args=v,v  # encount object (x, 1)

# 02184E94(a0, gsys) -> event (proc 0x021847CD): M+0x10 = copy of 020128B8(gd), 02185F18(gd, M+0x10), M+0x10 = 1, M+0x2C = 0,
# M+0x40 = 1. Arena: arg 2.
fn 02184e94 ret=u32 inputs=100
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x1000
  fix arg2 0x2C0 u32 alt range 0 4 || any
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg2:0:1 args=v  # GMEVENT work (event)
  stub 02184cec ret=none args=v,v  # M init (M, gsys)
  stub 020128b8 force ret=ptr:arg2:0x2C0:1 args=v  # GAMEDATA -> LOCATION
  stub 02185f18 force ret=none args=v,out:0x1C  # special location copy (gd, &loc)
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02188c9c force ret=pick+u32:0/1/0x3f/0x108 args=v  # FIELDMAP zone
  stub 02188ce8 force ret=u32 args=v  # FIELDMAP -> encount object
  stub 021aaaf0 force ret=none args=v,v  # encount object (x, 1)

# 02184EEC(gsys) -> event (proc 0x02184895): as 02184E94. Arena: arg 1.
fn 02184eec ret=u32 inputs=100
  nowork
  arg u32 any
  arg buf 0x1000
  fix arg1 0x2C0 u32 alt range 0 4 || any
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg1:0:1 args=v  # GMEVENT work (event)
  stub 02184cec ret=none args=v,v  # M init (M, gsys)
  stub 020128b8 force ret=ptr:arg1:0x2C0:1 args=v  # GAMEDATA -> LOCATION
  stub 02185f18 force ret=none args=v,out:0x1C  # special location copy (gd, &loc)
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02188c9c force ret=pick+u32:0/1/0x3f/0x108 args=v  # FIELDMAP zone
  stub 02188ce8 force ret=u32 args=v  # FIELDMAP -> encount object
  stub 021aaaf0 force ret=none args=v,v  # encount object (x, 1)

# 02184F40(gsys) -> event (proc 0x0218495D): 02014204(M+0x10, 02159F94(02012914(gd))), M+0x10 = 1, M+0x2C = 0. Arena: arg 1.
fn 02184f40 ret=u32 inputs=100
  nowork
  arg u32 any
  arg buf 0x1000
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 02012914 force ret=u32 args=v  # GAMEDATA getter
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg1:0:1 args=v  # GMEVENT work (event)
  stub 02184cec ret=none args=v,v  # M init (M, gsys)
  stub 02159f94 force ret=u32 args=v  # (x)
  stub 02014204 force ret=none args=out:0x1C,v  # LOCATION init by zone (loc, zone)
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02188c9c force ret=pick+u32:0/1/0x3f/0x108 args=v  # FIELDMAP zone
  stub 02188ce8 force ret=u32 args=v  # FIELDMAP -> encount object
  stub 021aaaf0 force ret=none args=v,v  # encount object (x, 1)

# 02184F88(gsys) -> event (proc 0x02184A15): M+0x10 = copy of 02012894(gd), M+0x10 = 1, u16 M+0x18 = 2, M+0x2C = 0. Arena: arg 1.
fn 02184f88 ret=u32 inputs=100
  nowork
  arg u32 any
  arg buf 0x1000
  fix arg1 0x200 u32 alt range 0 4 || any
  fix arg1 0x204 u16 alt oneof 136 158 || range 0 20
  fix arg1 0x208 u16 range 0 4
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg1:0:1 args=v  # GMEVENT work (event)
  stub 02184cec ret=none args=v,v  # M init (M, gsys)
  stub 02012894 force ret=ptr:arg1:0x200:1 args=v  # GAMEDATA -> LOCATION
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02188c9c force ret=pick+u32:0/1/0x3f/0x108 args=v  # FIELDMAP zone
  stub 02188ce8 force ret=u32 args=v  # FIELDMAP -> encount object
  stub 021aaaf0 force ret=none args=v,v  # encount object (x, 1)

# 02184FD4(gsys, zone) -> event (proc 0x02184AB5): 02014204(M+0x10, zone), M+0x2C = 0, u16 M+0x18 = 1; the PLAYER record's
# LOCATION (zone 02012AC8, dir 02012AE4, position 02012AA0) -> 0201289C(gd, &loc). Arena: arg 2.
fn 02184fd4 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x1000
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg2:0:1 args=v  # GMEVENT work (event)
  stub 02184cec ret=none args=v,v  # M init (M, gsys)
  stub 02014204 force ret=none args=out:0x1C,v  # LOCATION init by zone (loc, zone)
  stub 020127a4 force ret=u32 args=v  # GAMEDATA -> PLAYER record
  stub 02012aa0 force ret=ptr:arg2:0x340:1 args=v  # PLAYER record -> position (+4)
  stub 02012ac8 force ret=u32 args=v  # PLAYER record: zone
  stub 02012ae4 force ret=u32 args=v  # PLAYER record: direction
  stub 02014184 force ret=none args=out:0x1C,v,v,v,v,v  # LOCATION grid init (loc, zone, dir, x, y, z)
  stub 0201289c force ret=none args=v,in:0x1C  # GAMEDATA: LOCATION = *loc
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02188c9c force ret=pick+u32:0/1/0x3f/0x108 args=v  # FIELDMAP zone
  stub 02188ce8 force ret=u32 args=v  # FIELDMAP -> encount object
  stub 021aaaf0 force ret=none args=v,v  # encount object (x, 1)

# 02185050(gsys, a1, zone, pos, dir) -> event (proc 0x02184B55): LOCATION grid, M+0x2C = 5, M+0x40 = 0. Arena: arg 5.
fn 02185050 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x10
  arg u32 alt range 0 5 || any
  arg buf 0x1000
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg5:0:1 args=v  # GMEVENT work (event)
  stub 02184cec ret=none args=v,v  # M init (M, gsys)
  stub 020142a4 force ret=pick+u32:0/1/2/3/0xFFFF args=v  # direction code
  stub 02014184 force ret=none args=out:0x1C,v,v,v,v,v  # LOCATION grid init (loc, zone, dir, x, y, z)
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02188c9c force ret=pick+u32:0/1/0x3f/0x108 args=v  # FIELDMAP zone
  stub 02188ce8 force ret=u32 args=v  # FIELDMAP -> encount object
  stub 021aaaf0 force ret=none args=v,v  # encount object (x, 1)

# 021850A8(gsys) -> event (proc 0x02184C79): LOCATION grid (0x1A6, 020142A4(0), 0xB8000, 0, 0xF8000), M+0x2C = 0. Arena: arg
# 1.
fn 021850a8 ret=u32 inputs=100
  nowork
  arg u32 any
  arg buf 0x1000
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg1:0:1 args=v  # GMEVENT work (event)
  stub 02184cec ret=none args=v,v  # M init (M, gsys)
  stub 020142a4 force ret=pick+u32:0/1/2/3/0xFFFF args=v  # direction code
  stub 02014184 force ret=none args=out:0x1C,v,v,v,v,v  # LOCATION grid init (loc, zone, dir, x, y, z)
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02188c9c force ret=pick+u32:0/1/0x3f/0x108 args=v  # FIELDMAP zone
  stub 02188ce8 force ret=u32 args=v  # FIELDMAP -> encount object
  stub 021aaaf0 force ret=none args=v,v  # encount object (x, 1)

# 021850FC(gsys) -> event (proc 0x02184BF9): M+0x10 = copy of 02012894(gd). Arena: arg 1.
fn 021850fc ret=u32 inputs=100
  nowork
  arg u32 any
  arg buf 0x1000
  fix arg1 0x200 u32 alt range 0 4 || any
  fix arg1 0x204 u16 alt oneof 136 158 || range 0 20
  fix arg1 0x208 u16 range 0 4
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 02012894 force ret=ptr:arg1:0x200:1 args=v  # GAMEDATA -> LOCATION
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg1:0:1 args=v  # GMEVENT work (event)
  stub 02184cec ret=none args=v,v  # M init (M, gsys)
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02188c9c force ret=pick+u32:0/1/0x3f/0x108 args=v  # FIELDMAP zone
  stub 02188ce8 force ret=u32 args=v  # FIELDMAP -> encount object
  stub 021aaaf0 force ret=none args=v,v  # encount object (x, 1)

# 02185140(gsys, a1, pos, a3) -> 02185BFC(gsys, fm, &loc, a3, k): 021852F0(&cur, fm), 020128E4(gd, &cur), 0200BFA4(..., 0);
# I = 02158344(02012138(gsys)): with I and 02158590(gd): k = 02176230(I, I + 0x1AC), d = k - 02012B58(gd) (+ byte I+0x902 when < 0),
# x += 0x10000 + d << 22; loc = grid (a1, 020142A4(0), x, y, z). Arena: arg 4.
fn 02185140 ret=u32 inputs=600
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x10
  arg u32 any
  arg buf 0x1000
  fix arg4 0x186 u16 any
  fix arg4 0xA02 u8 range 1 8
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 021852f0 ret=none args=out:0x1C,v  # LOCATION of the player (&loc, fm)
  stub 020128e4 force ret=none args=v,in:0x1C  # GAMEDATA: LOCATION = *loc
  stub 02012ebc force ret=u32 args=v  # GAMEDATA -> save data
  stub 0200be48 force ret=u32 args=v  # save block 0x35 (thunk to 020071CC)
  stub 02012fcc force ret=u32 args=v  # GAMEDATA getter
  stub 0200bfa4 force ret=none args=v,v,v  # (x, y, k)
  stub 02012138 force ret=u32 args=v  # GAMESYS -> intrude / comm system I0
  stub 02158344 force ret=nullable:3:ptr:arg4:0x100:1 args=v  # comm record I (X1) of I0
  stub 02158590 force ret=pick:0/1 args=v  # (gd)
  stub 02176230 force ret=pick+u32:0/1/2/3/4/5/6/7 args=v,v  # comm (I, I + 0x1AC)
  stub 02012b58 force ret=pick+u32:0/1/2/3/4/5/6/7 args=v  # GAMEDATA getter (time slot)
  stub 020142a4 force ret=pick+u32:0/1/2/3/0xFFFF args=v  # direction code
  stub 02014184 force ret=none args=out:0x1C,v,v,v,v,v  # LOCATION grid init (loc, zone, dir, x, y, z)
  stub 02185bfc force ret=u32 args=v,v,in:0x1C,v,v  # comm map change event (gsys, fm, &loc, a, b)
  stub 02188cb4 force ret=u32 args=v  # FIELDMAP -> FIELD_PLAYER
  stub 02188cfc force ret=pick:0/1 args=v  # FIELDMAP map kind (0 grid, 1 rail)
  stub 021a3074 force ret=u32 args=v  # FIELD_PLAYER -> MMDL
  stub 0216d888 force ret=ptr:arg4:0x360:1 args=v  # fldmmdl: MMDL position
  stub 02188c9c force ret=pick+u32:0/1/0x3f/0x108 args=v  # FIELDMAP zone
  stub 021a36bc force ret=none args=v,out:8  # FIELD_PLAYER: rail location (pl, &r)
  stub 020141a8 force ret=none args=out:0x1C,v,v,v,v,v  # LOCATION rail init (loc, zone, dir, line, front, side)

# 02185214(gsys) -> event: loc = copy of 020128DC(gd), zone 02013DD0 == 1: 02014204(&loc, zone); ev = 02185C4C(gsys, fm, &loc);
# 0200BFA4(..., 1); comm cleanup (0202648C, 02026454 / 02026484); 021A3238(PLAYER) == 0: 0217BCDC(0, gsys, 0, 0, 0); 02012B78(gd, 0);
# no comm record: 02012B64 / 02012B8C (gd, 0), 02012B4C(gd, 1). Arena: arg 1.
fn 02185214 ret=u32 inputs=400
  nowork
  arg u32 any
  arg buf 0x1000
  fix arg1 0x304 u16 range 0 20
  fix arg1 0x186 u16 any
  fix arg1 0xA02 u8 range 1 8
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 020128dc force ret=ptr:arg1:0x300:1 args=v  # GAMEDATA -> LOCATION
  stub 02013dd0 force ret=pick+u32:0/1 args=v  # zonedata predicate (zone)
  stub 02014204 force ret=none args=out:0x1C,v  # LOCATION init by zone (loc, zone)
  stub 02185c4c force ret=u32 args=v,v,in:0x1C  # comm map change event (gsys, fm, &loc)
  stub 02012ebc force ret=u32 args=v  # GAMEDATA -> save data
  stub 0200be48 force ret=u32 args=v  # save block 0x35 (thunk to 020071CC)
  stub 02012fcc force ret=u32 args=v  # GAMEDATA getter
  stub 0200bfa4 force ret=none args=v,v,v  # (x, y, k)
  stub 02012138 force ret=u32 args=v  # GAMESYS -> intrude / comm system I0
  stub 0202648c force ret=none args=v  # comm (I0)
  stub 02026454 force ret=pick+u32:0/1/2/3 args=v  # comm state (I0)
  stub 02026484 force ret=none args=v  # comm (I0)
  stub 02188cb4 force ret=u32 args=v  # FIELDMAP -> FIELD_PLAYER
  stub 021a3238 force ret=pick:0/1 args=v  # FIELD_PLAYER predicate
  stub 0217bcdc force ret=none args=v,v,v,v,v  # (0, gsys, 0, 0, 0)
  stub 02012b78 force ret=none args=v,v  # GAMEDATA setter
  stub 02158344 force ret=nullable:3:ptr:arg1:0x100:1 args=v  # comm record I (X1) of I0
  stub 02012b64 force ret=none args=v,v  # GAMEDATA setter
  stub 02012b8c force ret=none args=v,v  # GAMEDATA setter
  stub 02012b4c force ret=none args=v,v  # GAMEDATA setter

# 021852F0(&loc, fm): grid map: 02014184(loc, zone, 1, MMDL position); rail: 021A36BC(pl, &r), 020141A8(loc, zone, 2, u16 r+0,
# u16 r+6, s16 r+4). Arena: arg 2.
fn 021852f0 ret=none inputs=200
  nowork
  arg buf 0x20
  arg u32 any
  arg buf 0x1000
  stub 02188cb4 force ret=u32 args=v  # FIELDMAP -> FIELD_PLAYER
  stub 02188cfc force ret=pick:0/1 args=v  # FIELDMAP map kind (0 grid, 1 rail)
  stub 021a3074 force ret=u32 args=v  # FIELD_PLAYER -> MMDL
  stub 0216d888 force ret=ptr:arg2:0x360:1 args=v  # fldmmdl: MMDL position
  stub 02188c9c force ret=pick+u32:0/1/0x3f/0x108 args=v  # FIELDMAP zone
  stub 02014184 force ret=none args=out:0x1C,v,v,v,v,v  # LOCATION grid init (loc, zone, dir, x, y, z)
  stub 021a36bc force ret=none args=v,out:8  # FIELD_PLAYER: rail location (pl, &r)
  stub 020141a8 force ret=none args=out:0x1C,v,v,v,v,v  # LOCATION rail init (loc, zone, dir, line, front, side)

# 02185360(gsys, a1, warp, a3) -> event (proc 0x02184389): M+0x40 = 1; 021625E8(warp): M+0x10 = copy of 02012894(gd), else
# 02162694(warp, M+0x10, a3); M+0x2C = 021626E8(warp); special location kept (02013E7C) -> 020128C0; 02185EB0(gd, M+0x10);
# 0201447C(02012B38(gd), 2). Arena: arg 4.
fn 02185360 ret=u32 inputs=300
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x1000
  fix arg4 0x284 u16 range 0 20
  fix arg4 0x200 u32 alt range 0 4 || any
  fix arg4 0x204 u16 alt oneof 136 158 || range 0 20
  fix arg4 0x208 u16 range 0 4
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg4:0:1 args=v  # GMEVENT work (event)
  stub 02184cec ret=none args=v,v  # M init (M, gsys)
  stub 021625e8 force ret=pick:0/1 args=v  # eventdata: warp to the special location
  stub 02012894 force ret=ptr:arg4:0x200:1 args=v  # GAMEDATA -> LOCATION
  stub 02162694 force ret=none args=v,out:0x1C,v  # eventdata: warp destination (warp, &loc, x)
  stub 021626e8 force ret=pick+u32:0/1/2/3/5/6/7 args=v  # eventdata: warp door kind
  stub 02012870 force ret=ptr:arg4:0x280:1 args=v  # GAMEDATA -> LOCATION (+0x164)
  stub 02013e7c force ret=pick+u32:0/1 args=v  # zonedata predicate (zone)
  stub 020128c0 force ret=none args=v,in:0x1C  # GAMEDATA: LOCATION (+0x16C) = *loc
  stub 02185eb0 force ret=none args=v,in:0x1C  # special location update (gd, loc)
  stub 02012b38 force ret=u32 args=v  # GAMEDATA -> EVENT WORK +0x1B8 object
  stub 0201447c force ret=none args=v,v  # (obj, flag)
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02188c9c force ret=pick+u32:0/1/0x3f/0x108 args=v  # FIELDMAP zone
  stub 02188ce8 force ret=u32 args=v  # FIELDMAP -> encount object
  stub 021aaaf0 force ret=none args=v,v  # encount object (x, 1)

# 02185410(gsys, a1, warp, a3) -> event (proc 0x02184411): M+0x40 = 1, 02162694(warp, M+0x10, a3), M+0x2C = 021626E8(warp).
# Arena: arg 4.
fn 02185410 ret=u32 inputs=100
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x1000
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg4:0:1 args=v  # GMEVENT work (event)
  stub 02184cec ret=none args=v,v  # M init (M, gsys)
  stub 02162694 force ret=none args=v,out:0x1C,v  # eventdata: warp destination (warp, &loc, x)
  stub 021626e8 force ret=pick+u32:0/1/2/3/5/6/7 args=v  # eventdata: warp door kind
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02188c9c force ret=pick+u32:0/1/0x3f/0x108 args=v  # FIELDMAP zone
  stub 02188ce8 force ret=u32 args=v  # FIELDMAP -> encount object
  stub 021aaaf0 force ret=none args=v,v  # encount object (x, 1)

# 02185458(gsys, a1, kind, zone, pos, dir) -> event (proc 0x02184505): LOCATION grid, M+0x2C = 0, M+0x30 = kind. Arena: arg
# 6.
fn 02185458 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x10
  arg u32 alt range 0 5 || any
  arg buf 0x1000
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg6:0:1 args=v  # GMEVENT work (event)
  stub 02184cec ret=none args=v,v  # M init (M, gsys)
  stub 020142a4 force ret=pick+u32:0/1/2/3/0xFFFF args=v  # direction code
  stub 02014184 force ret=none args=out:0x1C,v,v,v,v,v  # LOCATION grid init (loc, zone, dir, x, y, z)
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02188c9c force ret=pick+u32:0/1/0x3f/0x108 args=v  # FIELDMAP zone
  stub 02188ce8 force ret=u32 args=v  # FIELDMAP -> encount object
  stub 021aaaf0 force ret=none args=v,v  # encount object (x, 1)

# 021854B0(a0, a1, zone, pos, dir) -> 02185458(a0, a1, 0, zone, pos, u16 dir). Arena: arg 5.
fn 021854b0 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x10
  arg u32 alt range 0 5 || any
  arg buf 0x1000
  stub 02185458 ret=u32 args=v,v,v,v,v,v  # map change event (gsys, a1, kind, zone, pos, dir)
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg5:0:1 args=v  # GMEVENT work (event)
  stub 02184cec ret=none args=v,v  # M init (M, gsys)
  stub 020142a4 force ret=pick+u32:0/1/2/3/0xFFFF args=v  # direction code
  stub 02014184 force ret=none args=out:0x1C,v,v,v,v,v  # LOCATION grid init (loc, zone, dir, x, y, z)
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02188c9c force ret=pick+u32:0/1/0x3f/0x108 args=v  # FIELDMAP zone
  stub 02188ce8 force ret=u32 args=v  # FIELDMAP -> encount object
  stub 021aaaf0 force ret=none args=v,v  # encount object (x, 1)

# 021854CC(gsys, a1, a2, loc, dir) -> event (proc 0x02184505): 02012B2C(PLAYER, 0); M+0x10 = copy of loc, u16 M+0x18 = dir,
# M+0x2C = 0, M+0x30 = 1; 02185EB0(gd, M+0x10). Arena: arg 5.
fn 021854cc ret=u32 inputs=100
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x20
  arg u32 any
  arg buf 0x1000
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 020127a4 force ret=u32 args=v  # GAMEDATA -> PLAYER record
  stub 02012b2c force ret=none args=v,v  # PLAYER record +0x40 = form
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg5:0:1 args=v  # GMEVENT work (event)
  stub 02184cec ret=none args=v,v  # M init (M, gsys)
  stub 02185eb0 force ret=none args=v,in:0x1C  # special location update (gd, loc)
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02188c9c force ret=pick+u32:0/1/0x3f/0x108 args=v  # FIELDMAP zone
  stub 02188ce8 force ret=u32 args=v  # FIELDMAP -> encount object
  stub 021aaaf0 force ret=none args=v,v  # encount object (x, 1)

# 02185534(gsys, a1, zone, pos, dir) -> event (proc 0x02184585): LOCATION grid, M+0x2C = 0. Arena: arg 5.
fn 02185534 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x10
  arg u32 alt range 0 5 || any
  arg buf 0x1000
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg5:0:1 args=v  # GMEVENT work (event)
  stub 02184cec ret=none args=v,v  # M init (M, gsys)
  stub 020142a4 force ret=pick+u32:0/1/2/3/0xFFFF args=v  # direction code
  stub 02014184 force ret=none args=out:0x1C,v,v,v,v,v  # LOCATION grid init (loc, zone, dir, x, y, z)
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02188c9c force ret=pick+u32:0/1/0x3f/0x108 args=v  # FIELDMAP zone
  stub 02188ce8 force ret=u32 args=v  # FIELDMAP -> encount object
  stub 021aaaf0 force ret=none args=v,v  # encount object (x, 1)

# 02185588(gsys, zone, pos) -> event (proc 0x0218466D): 02013D5C(zone) == 1: x += d << 22 with d = (u16 I+0x86 & 7) -
# 02012B58(gd) (+ byte I+0x902 when < 0; 0 without I); LOCATION grid (zone, 020142A4(0), x, y, z), M+0x2C = M+0x40 = 0. Arena: arg 3.
fn 02185588 ret=u32 inputs=400
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x10
  arg buf 0x1000
  fix arg3 0x186 u16 any
  fix arg3 0xA02 u8 range 1 8
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg3:0:1 args=v  # GMEVENT work (event)
  stub 02013d5c force ret=pick+u32:0/1 args=v  # zonedata predicate (zone)
  stub 02012138 force ret=u32 args=v  # GAMESYS -> intrude / comm system I0
  stub 02158344 force ret=nullable:3:ptr:arg3:0x100:1 args=v  # comm record I (X1) of I0
  stub 02012b58 force ret=pick+u32:0/1/2/3/4/5/6/7 args=v  # GAMEDATA getter (time slot)
  stub 02184cec ret=none args=v,v  # M init (M, gsys)
  stub 020142a4 force ret=pick+u32:0/1/2/3/0xFFFF args=v  # direction code
  stub 02014184 force ret=none args=out:0x1C,v,v,v,v,v  # LOCATION grid init (loc, zone, dir, x, y, z)
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02188c9c force ret=pick+u32:0/1/0x3f/0x108 args=v  # FIELDMAP zone
  stub 02188ce8 force ret=u32 args=v  # FIELDMAP -> encount object
  stub 021aaaf0 force ret=none args=v,v  # encount object (x, 1)

# 0218562C(gsys): 02159FA8(02012914(gd), &a); 02014204(&b, 02159F94(02012914(gd))); 020128C0(gd, &b); 02185B70(gsys);
# 02185A68(gsys, u16 a+4); 021857C0(gsys, &a); 0202735C(u16 a+4, gd); 02168EEC(gd); 0216446C(gd, u16 a+4). Arena: arg 1.
fn 0218562c ret=none inputs=100
  nowork
  arg u32 any
  arg buf 0x1000
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 02012914 force ret=u32 args=v  # GAMEDATA getter
  stub 02159fa8 force ret=none args=v,out:0x1C  # (x, &loc)
  stub 02159f94 force ret=u32 args=v  # (x)
  stub 02014204 force ret=none args=out:0x1C,v  # LOCATION init by zone (loc, zone)
  stub 020128c0 force ret=none args=v,in:0x1C  # GAMEDATA: LOCATION (+0x16C) = *loc
  stub 02185b70 force ret=none args=v  # zone leave (gsys)
  stub 02185a68 force ret=none args=v,v  # zone data load (gsys, zone)
  stub 021857c0 force ret=none args=v,in:0x1C  # zone entry (gsys, loc)
  stub 0202735c force ret=none args=v,v  # (zone, gd)
  stub 02168eec force ret=none args=v  # (gd)
  stub 0216446c force ret=none args=v,v  # eventdata_tail zone hook (gd, zone)
