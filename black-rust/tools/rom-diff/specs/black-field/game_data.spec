# fndiff spec: game_data.c / game_event.c / the game-system record (F0-spec), main ARM9 Thumb code
# 0x02011D40..0x02013090 (T1 units unk_02011D40, unk_02012224, unk_02012704, unk_02012CE0).  Grammar:
# svflow_specs/README.md (generic mode).  Run: tools/check-fn.sh black-field game_data --per-function
#
# What is here (the T1 names are misleading; read from the code):
#  * 0x02011D40..0x02012224: the GAME SYSTEM record (0x34 bytes, FUN_02011F24 returns its size, FUN_02011F28 fills
#    it): +0 u16 heap id, +4 u32, +8 u32 (FUN_02031324(heap)), +0xC u32 (result of FUN_02031330 in the main loop),
#    +0x10/+0x14 u32 (callback pair, FUN_0201209C), +0x18 u32 (current event, 02012104/02012108), +0x1C -> GAMEDATA
#    (FUN_020124F4 allocates it), +0x20 u32 (02012110), +0x24 -> FUN_020261E4 object, +0x28 u32, +0x2C -> object
#    (FUN_0202B4BC(gamedata, arg)), +0x30 u8, +0x31 u8 (FUN_020120A4: "event active" cache), +0x32 u8, +0x33 bit 0.
#    The main loop (FUN_02011FCC), the boot/shutdown glue (02011D40..02011E8E, 02011EAC..02011F18, 02011F90,
#    02011FB8, 02012088) and the ICA-style helpers only sequence unported subsystems (UI/hardware): not specced.
#  * 0x02012264..0x02012704: game_event.c, a task queue: EVENT record (0x14 bytes, FUN_020122C0): +0 u32 func,
#    +4 u32 data0, +8 u32 state, +0xC -> data block (size r3 at creation), +0x10 -> owner (the game system).
#    Only the accessors and the free are specced; the dispatchers call through function pointers (02012304,
#    02012380, 020123D8, 0201240C, 02012444, 02012474, 020124AC) and are not specced.
#  * 0x02012704..0x02013090: game_data.c. GAMEDATA is a 0x6B0-byte record (FUN_020124F4 allocates and fills it from
#    ~40 subsystem constructors; FUN_02012704 frees them; neither is specced: pure fan-out).  Everything else here is
#    an accessor and is specced; the layout, from those accessors:
#      +0x000 -> SaveData (first word; the save-block getter FUN_020071CC(save, id) is tail called by many getters)
#      +0x004 + i*0x44 (i < 5): five 0x44-byte PLAYER records (FUN_02012A60 inits one; see below)
#      +0x114: 0x20-byte block; +0x134 = +0x114 + 0x20 (FUN_02012944)
#      +0x158 u32; +0x15C ptr (halfword via FUN_02008850); +0x160/+0x164/+0x168/+0x16C ptr to 0x1C-byte structs
#      (get / copy-in pairs); +0x170: inline 0x1C-byte struct; +0x18C/+0x190/+0x194/+0x198/+0x19C/+0x1A0 u32
#      +0x1A4 u32, +0x1A8 u32, +0x1AC -> EVENT WORK (flags and vars: event_work.spec), +0x1B0/+0x1B4/+0x1B8 u32,
#      +0x1BC u32 (get/set)
#      +0x1C4..+0x1CF: byte fields (see the accessors: +0x1C4, +0x1C5, +0x1C6, +0x1C7, +0x1C8 (counter), +0x1C9
#      (setting it nonzero clears +0x1C8), +0x1CA, +0x1CB, +0x1CC, +0x1CD, +0x1CE ("busy" flag of the camera/
#      screen glue), +0x1CF); +0x1D0 + i: byte array; +0x1D4 + i*0x10: 0x10-byte records; +0x214 + i*0x1E8: 0x1E8-byte
#      records (copy/clear at 0x02012800/0x0201281C); +0x5E4 u32; +0x5E8, +0x5EC, +0x680, +0x694, +0x6A4 addresses;
#      +0x684, +0x688 (get/set), +0x68C, +0x690, +0x6A8 (get/set), +0x6AC u32.
#    PLAYER record (0x44 bytes): +0 u16, +4..+0xF three words, +0x10..+0x17 four halfwords, +0x18 u16 (a facing/
#    type code: FUN_02012AE4 classifies it into 0..3), +0x1B u8, +0x1C u16, +0x40 u32.
#  * 0x02012FF0..0x02013090: the global 0x020A7298 holds a pointer to a record whose first word is a handle; the
#    four functions turn a value of that handle into an index of the signed byte table 0x0209DFB0 (5 entries).
#
# Stub arguments follow the registers of the disassembly.  Callees outside this file (and outside the specced
# functions) are stubs; a spec'd callee in this file runs as real code.
area game_data
set field

# ---------------------------------------------------------------- game system record (0x34 bytes)

# FUN_02011F24(): size of the game system record, 0x34
fn 02011f24 ret=u32 inputs=100
  nowork

# FUN_02011F28(sys, heap, a2): fill the record: +0 = heap (u16), +4 = a2, +8 = FUN_02031324(heap), +0xC/+0x10/+0x14/+0x18 = 0,
# +0x1C = FUN_020124F4(heap) (GAMEDATA), +0x20 = 0, +0x24 = FUN_020261E4(heap), +0x28 = 0, +0x2C = FUN_0202B4BC(gamedata, heap arg),
# +0x32 = 0, +0x33 bit 0 = 0; then FUN_02012248(sys), FUN_020122A4(sys) (stubbed at their callee 02035138 / 02035064), FUN_020139B0(heap arg)
fn 02011f28 ret=none inputs=100
  nowork
  arg buf 0x40
  arg u32 alt range 0 0x30 || any
  arg u32 any
  stub 02031324 ret=u32 args=v
  stub 020124f4 ret=u32 args=v
  stub 020261e4 ret=u32 args=v
  stub 0202b4bc ret=u32 args=v,v
  stub 02035138 ret=none args=v,v
  stub 02035064 ret=none args=v,v
  stub 020139b0 ret=none args=v

# FUN_0201209C(sys, a, b): +0x10 = a, +0x14 = b
fn 0201209c ret=none inputs=100
  nowork
  arg buf 0x40
  arg u32 any
  arg u32 any

# FUN_020120A4(sys): +0x31 = (FUN_020123C4(sys) != 0), i.e. whether an event is queued (+0x18 != 0)
fn 020120a4 ret=none inputs=100
  nowork
  arg buf 0x40
  fix arg0 0x18 u32 alt const 0 || any

# FUN_020120B4(sys, a, b, c): tail call FUN_02031348(*(sys + 8), a, b, c)
fn 020120b4 ret=u32 inputs=100
  nowork
  arg buf 0x40
  arg u32 any
  arg u32 any
  arg u32 any
  stub 02031348 ret=u32 args=v,v,v,v

# FUN_020120C0(sys): FUN_020120B4(sys, 0x15, 0x021D359C, sys)
fn 020120c0 ret=u32 inputs=100
  nowork
  arg buf 0x40
  stub 02031348 ret=u32 args=v,v,v,v

# FUN_020120D8(sys): 1 if an event is queued (+0x18 != 0) else the cached byte at +0x31
fn 020120d8 ret=u8 inputs=100
  nowork
  arg buf 0x40
  fix arg0 0x18 u32 alt const 0 || any

# FUN_020120F0(sys): u32 at +0xC
fn 020120f0 ret=u32 inputs=100
  nowork
  arg buf 0x40

# FUN_020120F4(sys): u32 at +0x1C
fn 020120f4 ret=u32 inputs=100
  nowork
  arg buf 0x40

# FUN_02012104(sys): u32 at +0x18
fn 02012104 ret=u32 inputs=100
  nowork
  arg buf 0x40

# FUN_0201210C(sys): u32 at +0x20
fn 0201210c ret=u32 inputs=100
  nowork
  arg buf 0x40

# FUN_02012134(sys): u32 at +0x2C
fn 02012134 ret=u32 inputs=100
  nowork
  arg buf 0x40

# FUN_02012138(sys): u32 at +0x24
fn 02012138 ret=u32 inputs=100
  nowork
  arg buf 0x40

# FUN_0201212C(sys): u8 at +0x30
fn 0201212c ret=u8 inputs=100
  nowork
  arg buf 0x40

# FUN_02012144(sys): u8 at +0x32
fn 02012144 ret=u8 inputs=100
  nowork
  arg buf 0x40

# FUN_02012108(sys, v): u32 at +0x18 = v
fn 02012108 ret=none inputs=100
  nowork
  arg buf 0x40
  arg u32 any

# FUN_02012110(sys, v): u32 at +0x20 = v
fn 02012110 ret=none inputs=100
  nowork
  arg buf 0x40
  arg u32 any

# FUN_02012124(sys, v): u8 at +0x30 = v
fn 02012124 ret=none inputs=100
  nowork
  arg buf 0x40
  arg u8 any

# FUN_0201213C(sys, v): u8 at +0x32 = v
fn 0201213c ret=none inputs=100
  nowork
  arg buf 0x40
  arg u8 any

# FUN_02012114(sys): +0x20 != 0
fn 02012114 ret=bool inputs=100
  nowork
  arg buf 0x40
  fix arg0 0x20 u32 alt const 0 || any

# FUN_020120F8(sys): tail call FUN_020127A4(*(sys + 0x1C)) = GAMEDATA + 0x114 (the 0x20-byte block; reached through the pointer)
fn 020120f8 ret=ofs:arg1 inputs=100
  nowork
  arg buf 0x40
  arg buf 0x6C0
  fix arg0 0x1C u32 addrof arg1

# FUN_0201214C(sys): tail call FUN_02012BB0(*(sys + 0x1C)): 1 if FUN_02008448(gamedata[0x198]) == 1
fn 0201214c ret=bool inputs=100
  nowork
  arg buf 0x40
  arg buf 0x6C0
  fix arg0 0x1C u32 addrof arg1
  stub 02008448 ret=pick:0/1 args=v

# FUN_02012158(sys, flag): tail call FUN_02012BC8(*(sys + 0x1C), flag)
fn 02012158 ret=none inputs=100
  nowork
  arg buf 0x40
  arg u32 alt const 0 || any
  arg buf 0x6C0
  fix arg0 0x1C u32 addrof arg2
  stub 02008450 ret=none args=v,v

# FUN_02012164(sys): 1 when no event/blocker is active: +0x32 == 0, FUN_02013D34 == 0, FUN_0200FB24 == 0, FUN_0201214C == 1,
# FUN_02158318(sys) == 1, not (FUN_02035390 == 1 and FUN_020353B0 == 1), and FUN_020090E0 (after FUN_0200921C(gamedata)) != 0
fn 02012164 ret=bool inputs=300
  nowork
  arg buf 0x40
  arg buf 0x6C0
  fix arg0 0x1C u32 addrof arg1
  fix arg0 0x32 u8 alt const 0 || const 0 || any
  stub 02013d34 ret=pick:0/0/1 args=v
  stub 0200fb24 ret=pick:0/0/1 args=
  stub 02158318 ret=pick:1/1/0 args=v
  stub 02035390 ret=pick:0/1 args=
  stub 020353b0 ret=pick:0/1 args=
  stub 0200921c ret=none args=v
  stub 020090e0 ret=pick:0/1 args=
  stub 02008448 ret=pick:1/1/0 args=v

# FUN_020121CC(sys): u = sys[0x24]; if FUN_0203CD98() == 0 and FUN_02026454(u) == 0 and FUN_02012164(sys) == 1:
# FUN_020263D8(u, 1, sys), return 1; else 0
fn 020121cc ret=bool inputs=300
  nowork
  arg buf 0x40
  arg buf 0x6C0
  fix arg0 0x1C u32 addrof arg1
  stub 0203cd98 ret=pick:0/0/1 args=
  stub 02026454 ret=pick:0/0/1 args=v
  stub 020263d8 ret=none args=v,v,v
  fix arg0 0x32 u8 alt const 0 || const 0 || any
  stub 02013d34 ret=pick:0/0/1 args=v
  stub 0200fb24 ret=pick:0/0/1 args=
  stub 02158318 ret=pick:1/1/0 args=v
  stub 02035390 ret=pick:0/1 args=
  stub 020353b0 ret=pick:0/1 args=
  stub 0200921c ret=none args=v
  stub 020090e0 ret=pick:0/1 args=
  stub 02008448 ret=pick:1/1/0 args=v

# FUN_0201205C(sys): if FUN_020120D8(sys) == 0 and FUN_02012114(sys) == 1 (+0x20 != 0): FUN_02188BE8(FUN_0201210C(sys)) == 1 then: FUN_020121CC(sys)
fn 0201205c ret=none inputs=300
  nowork
  arg buf 0x40
  arg buf 0x6C0
  fix arg0 0x1C u32 addrof arg1
  fix arg0 0x18 u32 alt const 0 || any
  fix arg0 0x20 u32 alt const 0 || any
  stub 02188be8 ret=pick:0/1 args=v
  stub 0203cd98 ret=pick:0/0/1 args=
  stub 02026454 ret=pick:0/0/1 args=v
  stub 020263d8 ret=none args=v,v,v
  fix arg0 0x32 u8 alt const 0 || const 0 || any
  stub 02013d34 ret=pick:0/0/1 args=v
  stub 0200fb24 ret=pick:0/0/1 args=
  stub 02158318 ret=pick:1/1/0 args=v
  stub 02035390 ret=pick:0/1 args=
  stub 020353b0 ret=pick:0/1 args=
  stub 0200921c ret=none args=v
  stub 020090e0 ret=pick:0/1 args=
  stub 02008448 ret=pick:1/1/0 args=v

# FUN_02012204(sys): +0x33 bit 0
fn 02012204 ret=u8 inputs=100
  nowork
  arg buf 0x40

# FUN_02012210(sys): +0x33 bit 0 = 0
fn 02012210 ret=none inputs=100
  nowork
  arg buf 0x40

# FUN_02012220(sys): if FUN_0201214C(sys): FUN_02012158(sys, 0) and +0x33 = (+0x33 & ~1) | 1
fn 02012220 ret=none inputs=200
  nowork
  arg buf 0x40
  arg buf 0x6C0
  fix arg0 0x1C u32 addrof arg1
  stub 02008448 ret=pick:0/1 args=v
  stub 02008450 ret=none args=v,v

# FUN_02012248(sys): tail call FUN_02035138(0x02012221, sys)
fn 02012248 ret=none inputs=100
  nowork
  arg buf 0x40
  stub 02035138 ret=none args=v,v

# FUN_02012258(): tail call FUN_02035138(0, 0)
fn 02012258 ret=none inputs=100
  nowork
  stub 02035138 ret=none args=v,v

# FUN_02012264(sys): 1 if FUN_02026454(sys[0x24]) != 0 or FUN_0203DA4C() != 0; 0 if FUN_020353B0() != 0; else FUN_0201214C(sys) != 0
fn 02012264 ret=bool inputs=300
  nowork
  arg buf 0x40
  arg buf 0x6C0
  fix arg0 0x1C u32 addrof arg1
  stub 02026454 ret=pick:0/0/1 args=v
  stub 0203da4c ret=pick:0/0/1 args=
  stub 020353b0 ret=pick:0/0/1 args=
  stub 02008448 ret=pick:0/1 args=v

# FUN_020122A4(sys): tail call FUN_02035064(0x02012265, sys)
fn 020122a4 ret=none inputs=100
  nowork
  arg buf 0x40
  stub 02035064 ret=none args=v,v

# FUN_020122B4(): tail call FUN_02035064(0, 0)
fn 020122b4 ret=none inputs=100
  nowork
  stub 02035064 ret=none args=v,v

# ---------------------------------------------------------------- game_event.c (EVENT records, 0x14 bytes)

# FUN_020124E4(ev): u32 at +0x10 (owner)
fn 020124e4 ret=u32 inputs=100
  nowork
  arg buf 0x20

# FUN_020124E8(ev): u32 at +0xC (data block)
fn 020124e8 ret=u32 inputs=100
  nowork
  arg buf 0x20

# FUN_020124EC(ev): address ev + 8
fn 020124ec ret=ofs:arg0 inputs=100
  nowork
  arg buf 0x20

# FUN_020124F0(ev): u32 at +0
fn 020124f0 ret=u32 inputs=100
  nowork
  arg buf 0x20

# FUN_02012314(ev): free +0xC if non-null, then free ev
fn 02012314 ret=none inputs=150
  nowork
  arg buf 0x20
  fix arg0 0xC u32 alt const 0 || any
  stub 020307b0 ret=none args=v seq

# FUN_02012374(ev, p): *p = ev; owner = ev[0x10]; owner[0x18] = p (FUN_02012108(owner, p))
fn 02012374 ret=none inputs=100
  nowork
  arg buf 0x20
  arg buf 0x10
  arg buf 0x40
  fix arg0 0x10 u32 addrof arg2

# FUN_020123C4(sys): 1 if +0x18 != 0
fn 020123c4 ret=bool inputs=100
  nowork
  arg buf 0x40
  fix arg0 0x18 u32 alt const 0 || any

# ---------------------------------------------------------------- GAMEDATA accessors (0x6B0 bytes)

# FUN_02012798(gd, i): address gd + 0x4 + i * 0x44 (PLAYER record i; no bounds check)
fn 02012798 ret=ofs:arg0 inputs=100
  nowork
  arg buf 0x6C0
  arg u32 alt range 0 5 || any

# FUN_020127A4(gamedata): address gamedata + 0x114 (returned as an offset into the buffer)
fn 020127a4 ret=ofs:arg0 inputs=100
  nowork
  arg buf 0x6C0

# FUN_020127AC(gd, i): address gd + 0x1D4 + i * 0x10 (0x10-byte record i; no bounds check)
fn 020127ac ret=ofs:arg0 inputs=100
  nowork
  arg buf 0x6C0
  arg u32 alt range 0 8 || any

# FUN_020127B8(gd): FUN_020127AC(gd, 3)
fn 020127b8 ret=ofs:arg0 inputs=100
  nowork
  arg buf 0x6C0

# FUN_020127C4(gd): FUN_020127D0(gd, 1)
fn 020127c4 ret=ofs:arg0 inputs=100
  nowork
  arg buf 0x6C0

# FUN_020127D0(gd, i): address gd + 0x214 + i * 0x1E8 (0x1E8-byte record i; no bounds check)
fn 020127d0 ret=ofs:arg0 inputs=100
  nowork
  arg buf 0x6C0
  arg u32 alt range 0 2 || any

# FUN_020127E0(gd): FUN_0200A890(FUN_0200A864(*gd))
fn 020127e0 ret=u32 inputs=100
  nowork
  arg buf 0x6C0
  stub 0200a864 ret=u32 args=v
  stub 0200a890 ret=u32 args=v

# FUN_020127F0(gd): FUN_0200A898(FUN_0200A864(*gd))
fn 020127f0 ret=u32 inputs=100
  nowork
  arg buf 0x6C0
  stub 0200a864 ret=u32 args=v
  stub 0200a898 ret=u32 args=v

# FUN_02012800(gd, src): copy 0x1E8 bytes from src to gd + 0x214, then FUN_02161D64(gd + 0x214)
fn 02012800 ret=none inputs=100
  nowork
  arg buf 0x6C0
  arg buf 0x1F0
  stub 02161d64 ret=none args=v

# FUN_0201281C(gd): zero gd + 0x214 .. +0x3FC, then FUN_02161D64(gd + 0x3FC)
fn 0201281c ret=none inputs=100
  nowork
  arg buf 0x6C0
  stub 02161d64 ret=none args=v

# FUN_0201283C(gamedata): u32 at +0x1A0
fn 0201283c ret=u32 inputs=100
  nowork
  arg buf 0x6C0

# FUN_02012844(gamedata): u32 at +0x158
fn 02012844 ret=u32 inputs=100
  nowork
  arg buf 0x6C0

# FUN_0201284C(gamedata): u32 at +0x160
fn 0201284c ret=u32 inputs=100
  nowork
  arg buf 0x6C0

# FUN_02012854(gd, src): copy 0x1C bytes from src to the struct *(gd + 0x160) points to
fn 02012854 ret=none inputs=100
  nowork
  arg buf 0x6C0
  arg buf 0x20
  region dst 0x20
  fix arg0 0x160 u32 addrof region:dst

# FUN_02012870(gamedata): u32 at +0x164
fn 02012870 ret=u32 inputs=100
  nowork
  arg buf 0x6C0

# FUN_02012878(gd, src): copy 0x1C bytes from src to the struct *(gd + 0x164) points to
fn 02012878 ret=none inputs=100
  nowork
  arg buf 0x6C0
  arg buf 0x20
  region dst 0x20
  fix arg0 0x164 u32 addrof region:dst

# FUN_02012894(gamedata): u32 at +0x168
fn 02012894 ret=u32 inputs=100
  nowork
  arg buf 0x6C0

# FUN_0201289C(gd, src): copy 0x1C bytes from src to the struct *(gd + 0x168) points to
fn 0201289c ret=none inputs=100
  nowork
  arg buf 0x6C0
  arg buf 0x20
  region dst 0x20
  fix arg0 0x168 u32 addrof region:dst

# FUN_020128B8(gamedata): u32 at +0x16C
fn 020128b8 ret=u32 inputs=100
  nowork
  arg buf 0x6C0

# FUN_020128C0(gd, src): copy 0x1C bytes from src to the struct *(gd + 0x16C) points to
fn 020128c0 ret=none inputs=100
  nowork
  arg buf 0x6C0
  arg buf 0x20
  region dst 0x20
  fix arg0 0x16C u32 addrof region:dst

# FUN_020128DC(gamedata): address gamedata + 0x170 (returned as an offset into the buffer)
fn 020128dc ret=ofs:arg0 inputs=100
  nowork
  arg buf 0x6C0

# FUN_020128E4(gd, src): copy 0x1C bytes from src to gd + 0x170
fn 020128e4 ret=none inputs=100
  nowork
  arg buf 0x6C0
  arg buf 0x20

# FUN_02012900(gd, v): *(u16 *)FUN_02008850(*(gd + 0x15C)) = v
fn 02012900 ret=none inputs=100
  nowork
  arg buf 0x6C0
  arg u16 any
  arg buf 0x10
  stub 02008850 ret=ptr:arg2:0:1 args=v

# FUN_02012914(gd): *(u16 *)FUN_02008850(*(gd + 0x15C))
fn 02012914 ret=u16 inputs=100
  nowork
  arg buf 0x6C0
  arg buf 0x10
  stub 02008850 ret=ptr:arg1:0:1 args=v

# FUN_02012924(gamedata): u32 at +0x18C
fn 02012924 ret=u32 inputs=100
  nowork
  arg buf 0x6C0

# FUN_0201292C(gamedata): u32 at +0x190
fn 0201292c ret=u32 inputs=100
  nowork
  arg buf 0x6C0

# FUN_02012934(gamedata): u32 at +0x194
fn 02012934 ret=u32 inputs=100
  nowork
  arg buf 0x6C0

# FUN_0201293C(gamedata): u32 at +0x19C
fn 0201293c ret=u32 inputs=100
  nowork
  arg buf 0x6C0

# FUN_02012944(gd): FUN_020127A4(gd) + 0x20 = gd + 0x134
fn 02012944 ret=ofs:arg0 inputs=100
  nowork
  arg buf 0x6C0

# FUN_02012950(gd, i): address gd + 0x24 + i * 0x44 (PLAYER record i + 0x20; no bounds check)
fn 02012950 ret=ofs:arg0 inputs=100
  nowork
  arg buf 0x6C0
  arg u32 alt range 0 5 || any

# FUN_0201295C(gamedata): u32 at +0x1A4
fn 0201295c ret=u32 inputs=100
  nowork
  arg buf 0x6C0

# FUN_02012964(gamedata): u32 at +0x1A8
fn 02012964 ret=u32 inputs=100
  nowork
  arg buf 0x6C0

# FUN_0201296C(gamedata): u32 at +0x1AC
fn 0201296c ret=u32 inputs=100
  nowork
  arg buf 0x6C0

# FUN_02012974(gamedata): u32 at +0x1B0
fn 02012974 ret=u32 inputs=100
  nowork
  arg buf 0x6C0

# FUN_0201297C(gamedata): u32 at +0x690
fn 0201297c ret=u32 inputs=100
  nowork
  arg buf 0x6C0

# FUN_02012984(gamedata): u8 at +0x1C4
fn 02012984 ret=u8 inputs=100
  nowork
  arg buf 0x6C0

# FUN_0201298C(gamedata, v): u8 at +0x1C4 = v
fn 0201298c ret=none inputs=100
  nowork
  arg buf 0x6C0
  arg u8 any

# FUN_02012994(gamedata): u8 at +0x1C5
fn 02012994 ret=u8 inputs=100
  nowork
  arg buf 0x6C0

# FUN_020129A0(gamedata, v): u8 at +0x1C5 = v
fn 020129a0 ret=none inputs=100
  nowork
  arg buf 0x6C0
  arg u8 any

# FUN_020129AC(gamedata): u8 at +0x1C6
fn 020129ac ret=u8 inputs=100
  nowork
  arg buf 0x6C0

# FUN_020129B8(gamedata, v): u8 at +0x1C6 = v
fn 020129b8 ret=none inputs=100
  nowork
  arg buf 0x6C0
  arg u8 any

# FUN_020129C4(gamedata): u8 at +0x1C7
fn 020129c4 ret=u8 inputs=100
  nowork
  arg buf 0x6C0

# FUN_020129D0(gamedata, v): u8 at +0x1C7 = v
fn 020129d0 ret=none inputs=100
  nowork
  arg buf 0x6C0
  arg u8 any

# FUN_020129DC(gd, v): +0x1C9 = v (u8); if v != 0: +0x1C8 = 0
fn 020129dc ret=none inputs=100
  nowork
  arg buf 0x6C0
  arg u8 alt const 0 || any

# FUN_020129F0(gd): +0x1C8 = 0
fn 020129f0 ret=none inputs=100
  nowork
  arg buf 0x6C0

# FUN_020129FC(gd): if +0x1C9 != 0: return the old +0x1C8 and increment it (u8); else 0
fn 020129fc ret=u8 inputs=100
  nowork
  arg buf 0x6C0
  fix arg0 0x1C9 u8 alt const 0 || any

# FUN_02012A24(gamedata): u8 at +0x1C9
fn 02012a24 ret=u8 inputs=100
  nowork
  arg buf 0x6C0

# FUN_02012A30(gamedata): u32 at +0x1B4
fn 02012a30 ret=u32 inputs=100
  nowork
  arg buf 0x6C0

# FUN_02012A38(gamedata): u32 at +0x5E4
fn 02012a38 ret=u32 inputs=100
  nowork
  arg buf 0x6C0

# FUN_02012A44(gamedata): u32 at +0x1BC
fn 02012a44 ret=u32 inputs=100
  nowork
  arg buf 0x6C0

# FUN_02012A4C(gamedata, v): u32 at +0x1BC = v
fn 02012a4c ret=none inputs=100
  nowork
  arg buf 0x6C0
  arg u32 any

# FUN_02012A54(gamedata): u32 at +0x68C
fn 02012a54 ret=u32 inputs=100
  nowork
  arg buf 0x6C0

# ---------------------------------------------------------------- PLAYER record (0x44 bytes)

# FUN_02012A60(rec): zero +0, +4/+8/+0xC, +0x10 (u16), +0x12/+0x13/+0x1B/+0x3A/+0x3D (u8), +0x14, +0x16, +0x18, +0x30; +0x1C = 0xFFFF
fn 02012a60 ret=none inputs=100
  nowork
  arg buf 0x48

# FUN_02012A90(rec, src): copy 12 bytes from src to rec + 4
fn 02012a90 ret=none inputs=100
  nowork
  arg buf 0x48
  arg buf 0x10

# FUN_02012AA0(rec): address rec + 4
fn 02012aa0 ret=ofs:arg0 inputs=100
  nowork
  arg buf 0x48

# FUN_02012AA4(rec, src): rec[+0x10, +0x12, +0x14, +0x16] = the four halfwords of src
fn 02012aa4 ret=none inputs=100
  nowork
  arg buf 0x48
  arg buf 0x10

# FUN_02012AB8(rec): address rec + 0x10
fn 02012ab8 ret=ofs:arg0 inputs=100
  nowork
  arg buf 0x48

# FUN_02012ABC(rec, v): u8 at +0x1B = v
fn 02012abc ret=none inputs=100
  nowork
  arg buf 0x48
  arg u8 any

# FUN_02012AC0(rec): u8 at +0x1B
fn 02012ac0 ret=u8 inputs=100
  nowork
  arg buf 0x48

# FUN_02012AC4(rec, v): u16 at +0x0 = v
fn 02012ac4 ret=none inputs=100
  nowork
  arg buf 0x48
  arg u16 any

# FUN_02012AC8(rec): u16 at +0x0
fn 02012ac8 ret=u16 inputs=100
  nowork
  arg buf 0x48

# FUN_02012ACC(rec, v): u16 at +0x18 = v
fn 02012acc ret=none inputs=100
  nowork
  arg buf 0x48
  arg u16 any

# FUN_02012AD0(rec): u16 at +0x18
fn 02012ad0 ret=u16 inputs=100
  nowork
  arg buf 0x48

# FUN_02012AD4(rec, i): rec[+0x18] = u16 table 0x0209DFA8 [i]
fn 02012ad4 ret=none inputs=100
  nowork
  arg buf 0x48
  arg u32 range 0 16

# FUN_02012AE4(rec): classify the u16 at +0x18: (0x2000, 0x6000) -> 2, [0x6000, 0xA000] -> 1, (0xA000, 0xE000) -> 3, else 0
fn 02012ae4 ret=u32 inputs=200
  nowork
  arg buf 0x48
  fix arg0 0x18 u16 alt near 0x2000 3 || near 0x6000 3 || near 0xA000 3 || near 0xE000 3 || any

# FUN_02012B28(rec): u32 at +0x40
fn 02012b28 ret=u32 inputs=100
  nowork
  arg buf 0x48

# FUN_02012B2C(rec, v): u32 at +0x40 = v
fn 02012b2c ret=none inputs=100
  nowork
  arg buf 0x48
  arg u32 any

# FUN_02012B30(rec, v): u16 at +0x1C = v
fn 02012b30 ret=none inputs=100
  nowork
  arg buf 0x48
  arg u16 any

# FUN_02012B34(rec): u16 at +0x1C
fn 02012b34 ret=u16 inputs=100
  nowork
  arg buf 0x48

# ---------------------------------------------------------------- GAMEDATA, continued

# FUN_02012B38(gamedata): u32 at +0x1B8
fn 02012b38 ret=u32 inputs=100
  nowork
  arg buf 0x6C0

# FUN_02012B40(gamedata): u8 at +0x1CA
fn 02012b40 ret=u8 inputs=100
  nowork
  arg buf 0x6C0

# FUN_02012B4C(gamedata, v): u8 at +0x1CA = v
fn 02012b4c ret=none inputs=100
  nowork
  arg buf 0x6C0
  arg u8 any

# FUN_02012B58(gamedata): u8 at +0x1CB
fn 02012b58 ret=u8 inputs=100
  nowork
  arg buf 0x6C0

# FUN_02012B64(gamedata, v): u8 at +0x1CB = v
fn 02012b64 ret=none inputs=100
  nowork
  arg buf 0x6C0
  arg u8 any

# FUN_02012B70(gamedata): u8 at +0x1CC
fn 02012b70 ret=u8 inputs=100
  nowork
  arg buf 0x6C0

# FUN_02012B78(gamedata, v): u8 at +0x1CC = v
fn 02012b78 ret=none inputs=100
  nowork
  arg buf 0x6C0
  arg u8 any

# FUN_02012B80(gamedata): u8 at +0x1CD
fn 02012b80 ret=u8 inputs=100
  nowork
  arg buf 0x6C0

# FUN_02012B8C(gamedata, v): u8 at +0x1CD = v
fn 02012b8c ret=none inputs=100
  nowork
  arg buf 0x6C0
  arg u8 any

# FUN_02012B98(gd, i): byte at gd + 0x1D0 + i (no bounds check)
fn 02012b98 ret=u8 inputs=100
  nowork
  arg buf 0x6C0
  arg u32 alt range 0 8 || range 0 0x40

# FUN_02012BA4(gd, i, v): byte at gd + 0x1D0 + i = v
fn 02012ba4 ret=none inputs=100
  nowork
  arg buf 0x6C0
  arg u32 alt range 0 8 || range 0 0x40
  arg u8 any

# FUN_02012BB0(gd): FUN_02008448(*(gd + 0x198)) == 1
fn 02012bb0 ret=bool inputs=100
  nowork
  arg buf 0x6C0
  stub 02008448 ret=pick:0/1 args=v

# FUN_02012BC8(gd, flag): tail call FUN_02008450(*(gd + 0x198), flag != 0)
fn 02012bc8 ret=none inputs=100
  nowork
  arg buf 0x6C0
  arg u32 alt const 0 || any
  stub 02008450 ret=none args=v,v

# FUN_02012BE0(gd, a, b): FUN_0200C418(FUN_0200C40C(*gd), a, b)
fn 02012be0 ret=none inputs=100
  nowork
  arg buf 0x6C0
  arg u32 any
  arg u32 any
  stub 0200c40c ret=u32 args=v
  stub 0200c418 ret=none args=v,v,v

# FUN_02012BF8(gd, a): FUN_0200C44C(FUN_0200C404(*gd), a), returned (V-0022 audit: tail bl + pop; 0200C44C is
# "1 if one of the 25 bytes at x equals a, else 0", ~27 overlay callers test the result)
fn 02012bf8 ret=u32 inputs=100
  nowork
  arg buf 0x6C0
  arg u32 any
  stub 0200c404 ret=u32 args=v
  stub 0200c44c ret=bool args=v,v

# FUN_02012C0C(gamedata): address gamedata + 0x5E8 (returned as an offset into the buffer)
fn 02012c0c ret=ofs:arg0 inputs=100
  nowork
  arg buf 0x6C0

# FUN_02012E40(gamedata): u8 at +0x1CE
fn 02012e40 ret=u8 inputs=100
  nowork
  arg buf 0x6C0

# FUN_02012EBC(gamedata): u32 at +0x0
fn 02012ebc ret=u32 inputs=100
  nowork
  arg buf 0x6C0

# FUN_02012EC0(gamedata): u32 at +0x0
fn 02012ec0 ret=u32 inputs=100
  nowork
  arg buf 0x6C0

# FUN_02012EC4(gamedata): address gamedata + 0x5EC (returned as an offset into the buffer)
fn 02012ec4 ret=ofs:arg0 inputs=100
  nowork
  arg buf 0x6C0

# FUN_02012ED0(gamedata): address gamedata + 0x680 (returned as an offset into the buffer)
fn 02012ed0 ret=ofs:arg0 inputs=100
  nowork
  arg buf 0x6C0

# FUN_02012ED8(gamedata): u32 at +0x688
fn 02012ed8 ret=u32 inputs=100
  nowork
  arg buf 0x6C0

# FUN_02012EE4(gamedata, v): u32 at +0x688 = v
fn 02012ee4 ret=none inputs=100
  nowork
  arg buf 0x6C0
  arg u32 any

# FUN_02012EF0(gd): tail call FUN_020071CC(*gd, 0x3A): save block 0x3A
fn 02012ef0 ret=u32 inputs=100
  nowork
  arg buf 0x6C0
  stub 020071cc ret=u32 args=v,v

# FUN_02012EFC(gd): tail call FUN_020071CC(*gd, 0x3B): save block 0x3B
fn 02012efc ret=u32 inputs=100
  nowork
  arg buf 0x6C0
  stub 020071cc ret=u32 args=v,v

# FUN_02012F08(gd): tail call FUN_0200C0F0(*gd): 
fn 02012f08 ret=u32 inputs=100
  nowork
  arg buf 0x6C0
  stub 0200c0f0 ret=u32 args=v

# FUN_02012F14(gd): tail call FUN_02009EC8(*gd): 
fn 02012f14 ret=u32 inputs=100
  nowork
  arg buf 0x6C0
  stub 02009ec8 ret=u32 args=v

# FUN_02012F20(gamedata): u32 at +0x684
fn 02012f20 ret=u32 inputs=100
  nowork
  arg buf 0x6C0

# FUN_02012F2C(gd): tail call FUN_020071CC(*gd, 0x26): save block 0x26
fn 02012f2c ret=u32 inputs=100
  nowork
  arg buf 0x6C0
  stub 020071cc ret=u32 args=v,v

# FUN_02012F38(gamedata): address gamedata + 0x694 (returned as an offset into the buffer)
fn 02012f38 ret=ofs:arg0 inputs=100
  nowork
  arg buf 0x6C0

# FUN_02012F44(gd): tail call FUN_020071CC(*gd, 0x2A): save block 0x2A
fn 02012f44 ret=u32 inputs=100
  nowork
  arg buf 0x6C0
  stub 020071cc ret=u32 args=v,v

# FUN_02012F50(gamedata): address gamedata + 0x6A4 (returned as an offset into the buffer)
fn 02012f50 ret=ofs:arg0 inputs=100
  nowork
  arg buf 0x6C0

# FUN_02012F5C(gd): tail call FUN_020089C0(*gd): 
fn 02012f5c ret=u32 inputs=100
  nowork
  arg buf 0x6C0
  stub 020089c0 ret=u32 args=v

# FUN_02012F68(gd): tail call FUN_020071CC(*gd, 0x42): save block 0x42
fn 02012f68 ret=u32 inputs=100
  nowork
  arg buf 0x6C0
  stub 020071cc ret=u32 args=v,v

# FUN_02012F74(gamedata, v): u32 at +0x6A8 = v
fn 02012f74 ret=none inputs=100
  nowork
  arg buf 0x6C0
  arg u32 any

# FUN_02012F80(gamedata): u32 at +0x6A8
fn 02012f80 ret=u32 inputs=100
  nowork
  arg buf 0x6C0

# FUN_02012F8C(gd): tail call FUN_02009448(*gd): 
fn 02012f8c ret=u32 inputs=100
  nowork
  arg buf 0x6C0
  stub 02009448 ret=u32 args=v

# FUN_02012F98(gd): tail call FUN_02007B44(*gd): 
fn 02012f98 ret=u32 inputs=100
  nowork
  arg buf 0x6C0
  stub 02007b44 ret=u32 args=v

# FUN_02012FA4(gamedata, v): u8 at +0x1CF = v
fn 02012fa4 ret=none inputs=100
  nowork
  arg buf 0x6C0
  arg u8 any

# FUN_02012FB0(gamedata): u8 at +0x1CF
fn 02012fb0 ret=u8 inputs=100
  nowork
  arg buf 0x6C0

# FUN_02012FBC(gd): tail call FUN_020085D4(*(gd + 0x6AC))
fn 02012fbc ret=u32 inputs=100
  nowork
  arg buf 0x6C0
  stub 020085d4 ret=u32 args=v

# FUN_02012FCC(gamedata): u32 at +0x6AC
fn 02012fcc ret=u32 inputs=100
  nowork
  arg buf 0x6C0

# FUN_02012FD8(gd): tail call FUN_020071CC(*gd, 0x43): save block 0x43
fn 02012fd8 ret=u32 inputs=100
  nowork
  arg buf 0x6C0
  stub 020071cc ret=u32 args=v,v

# FUN_02012FE4(save): tail call FUN_020071CC(save, 0x2D)
fn 02012fe4 ret=u32 inputs=100
  nowork
  arg u32 any
  stub 020071cc ret=u32 args=v,v

# ---------------------------------------------------------------- global handle 0x020A7298

# FUN_02012FF0(v): the global 0x020A7298 = v
fn 02012ff0 ret=none inputs=100
  nowork
  arg u32 any
  global 020a7298 4

# FUN_02013080(i): signed byte of the table 0x0209DFB0 at min(i, 4)
fn 02013080 ret=u8 inputs=100
  nowork
  arg u32 alt range 0 6 || any

# FUN_02012FFC(): v = FUN_02008374(FUN_0200873C(**0x020A7298)); table[min(FUN_02008374(), 4)]
fn 02012ffc ret=u8 inputs=150
  nowork
  global 020a7298 4
  region gs 0x10
  fix global0 0 u32 addrof region:gs
  stub 0200873c ret=u32 args=v
  stub 02008374 ret=pick:0/1/2/3/4/5/9 args=v

# FUN_02013020(): v = FUN_02008374(FUN_0200873C(**0x020A7298)); FUN_02013080(map {0: 1, 1: 2, 2: 3, else v})
fn 02013020 ret=u8 inputs=150
  nowork
  global 020a7298 4
  region gs 0x10
  fix global0 0 u32 addrof region:gs
  stub 0200873c ret=u32 args=v
  stub 02008374 ret=pick:0/1/2/3/4/5/9 args=v

# FUN_02013050(): v = FUN_02008374(FUN_0200873C(**0x020A7298)); FUN_02013080(map {0: 4, 1: 0, 2: 1, else v})
fn 02013050 ret=u8 inputs=150
  nowork
  global 020a7298 4
  region gs 0x10
  fix global0 0 u32 addrof region:gs
  stub 0200873c ret=u32 args=v
  stub 02008374 ret=pick:0/1/2/3/4/5/9 args=v
