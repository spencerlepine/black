# fndiff spec: fieldmap.c, first part (overlay 21 Thumb code 0x02187F00..0x02189BD0, F1 area 21.00; F2a-spec): the FIELDMAP
# proc (create / setup sequence / per-frame update / teardown), its accessors, the zone-change check, the map-setup record.
# The event checks of fieldmap.c (0x02189BD0..0x0218B752) are the `fieldmap_event` area.
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-field fieldmap --per-function
#
# Not specced (render-only, G8: graphics/hardware calls without FIELDMAP logic): 02188D5C, 02188DB0, 02188DF0, 02188E38, 02188E50,
# 02188EE4, 02189084, 021890AC, 021890C8, 02189108, 02189140, 02189154, 021891C0, 021897DC, 02189810, 02189998, 021899C4 (one line
# each in the crate README). Skipped veneers: 02188DA0 (`ldr r0,=021D35F4; ldr r3,=0203FC28; bx r3`), 02189B28 (`ldr r3,=02013B51; bx r3`:
# callers stub 02013B50). Callers of the render-only functions stub them by address (they are never ported in this crate).
#
# FIELDMAP (0x154 bytes, Heap_AllocDebug(heap, 0x154, 1, "fieldmap.c", 0x209) in 02188030; the arena `work` image holds it at +0):
#   +0x00 u16 heap id            +0x04 game system (GAMESYS)       +0x08 GAMEDATA (020120F4(gamesys))
#   +0x0C area data (0201407C(heap, zone record +2, season))           +0x10 map 3D handler (0218E018)    +0x14 (021A03D0)
#   +0x18 (021A0104)  +0x1C (021A02C4)  +0x20 (021A19E8)  +0x24 (021A0CF4)  +0x28 (021901EC; nullable)  +0x2C u32 (0 at setup)
#   +0x30 (021BC388(gamesys, 0x93); nullable)  +0x34 (021BE2A8)  +0x38 MMDL system (02012964; nullable)  +0x3C (021BB1F4)
#   +0x40 scene area (021AB5F4)  +0x44 (021AB7E4)  +0x48 map draw (0218C9E8)  +0x50..+0x8B MAP SETUP record (0x3C bytes, 02189B30)
#   +0x8C player (021A2E50)  +0x90 (021A9ED8: encounter)  +0x94 (021ABD50)  +0x98 (0217F8C0; nullable)  +0x9C (021D33A8)
#   +0xA0 (021C7F70)  +0xA4..+0xB8 render objects (02188EE4)  +0xAC VBlank task  +0xBC FIELDMAP FUNC system (021BB890)
#   +0xC0 u32 state (02188BE8: 1 = running; 02188BD8 sets 2)  +0xC4 u32 proc step, +0xC8 u8 sub-step, +0xC9 u8 half (02188B58)
#   +0xCC u32 frame counter  +0xD0 u16 zone id (setup)  +0xD4 LOCATION (0x1C bytes: player position record, 020141E0/020141F0)
#   +0xD8 u16 current zone id (02188C9C; written by the zone change 021893C0)  +0xF0 -> fx32 position[3] (nullable, 02188D40)
#   +0xF4..+0xFF fx32 position[3]  +0x100 (02189A80 camera distance)  +0x104 (02189ACC: 494 or 310)  +0x108 -> MAP CTRL (below)
#   +0x10C ctrl work (02188CA4/CAC)  +0x110 task system (0x20 tasks), +0x114 its block  +0x118 SLOT {s32 id (-1 free), ptr}
#   +0x120 (021C1A08)  +0x124 (021C1AF4)  +0x128 (021C31D4)  +0x12C u32 draw mode (021899F8; index of the draw tables 0x021D35B4 /
#   0x021D35D8)  +0x130  +0x134 (021C6D10)  +0x138  +0x13C (021CF45C)  +0x140 u32 flag  +0x144  +0x148 (address: 02189A60)
#   +0x14C (021D317C)  +0x150 u32
# MAP CTRL (ROM 0x021D88F8 grid / 0x021D8AD8 rail, picked by 02189BA8): +0 u32 kind (0x021D35A8[kind] == 2: hybrid, 021A6DC0),
#   +4 fn(fieldmap, position, ..) (setup), +8 fn(fieldmap, position) (per frame), +0xC fn(fieldmap) (teardown). In the specs:
#   work+0x400, fns FAKE 0x02EE0101 / 0x02EE0111 / 0x02EE0121.
# MAP SETUP table 0x021D3700: 8 records of 0x48 bytes indexed by 02013B50(zone) (zone kind): +0..+0x3B copied by 02189B30,
#   +0x3C -> MAP CTRL (02189BA8), +0x40 u32 "use the matrix size" flag, +0x44 u32 (02189BBC: heap size, 02187F00).
# PROC table 0x021D36B8: 6 steps x {fn, fn} (half 0 / 1), each fn(gamesys, fieldmap) -> 0 toggle half, 1 next step, 2 finished:
#   step 0 021880C8 (graphics init), 1 021881D8 (setup sequence), 2 021885E8 (start), 3 02188758 / 02188910 (frame update / draw),
#   4 02188990 (teardown), 5 02188AD4 (graphics exit).
# Arena zones: FIELDMAP work+0, 020120F8 result (player record) work+0x200, 02012ED0 result work+0x220, proc work work+0x240,
#   02012AA0 result work+0x260, 0218ED3C result work+0x280, MAP CTRL work+0x400, position (+0xF0) work+0x480, out words work+0x500.
area fieldmap
set field

# 02187F00(proc, seq, gamesys) (proc init): *seq == 0: 02012AC8(020120F8(gamesys)); n = 02189BBC(); heap create 0203064C(1, 0x15,
# n - 0xD000), (1, 0x70, 0xD000), (0x15, 0x50, 0xC000); 02030678(0x021DE700, 0x10000, 0x89); 0203064C(0x89, 0x92, 0x6400),
# (0x89, 0x93, 0x7400), (0x89, 0x96, 0x500); w = 0203159C(proc, 4, 0x15); *w = 02188030(gamesys, 0x15); 02012110(gamesys, *w);
# *seq += 1; returns 0. *seq == 1: if !0201214C(gamesys): x = 02012138(gamesys); 02026454(x) is 1 or 2: 02026400(x); returns 1.
# Other: 0.
fn 02187f00 ret=u32 inputs=200
  arg u32 any
  arg work 0x500
  arg u32 any
  fix work 0x500 u32 alt range 0 2 || any
  stub 0201214c force ret=pick:0/1 args=v
  stub 02012138 force ret=u32 args=v  # main
  stub 02026454 force ret=pick:0/1/2/3 args=v
  stub 02026400 force ret=none args=  # main
  stub 020120f8 force ret=ptr:work:0x200:1 args=v  # gamesys -> player record (fieldmap reads +4..+0xF)
  stub 02012ac8 force ret=u32 args=v  # main
  stub 02189bbc ret=u32 args=v  # in-area
  stub 0203064c force ret=none args=v,v,v  # heap create (parent heap, new heap id, size)
  stub 02030678 force ret=none args=v,v,v  # heap create at a fixed address (base, size, heap id)
  stub 0203159c force ret=ptr:work:0x240:1 args=v,v,v
  stub 02188030 ret=ptr:work:0:1 args=v,v  # in-area
  stub 02012110 force ret=none args=v,v  # main
  stub 02013b50 force ret=pick:0/1/2/3/4/5/6/7 args=v  # zone kind (index of the MAP SETUP table 0x021D3700)
  stub 02030734 force ret=ptr:work:0:1 args=v,v,v,v,v
  stub 020120f4 force ret=u32 args=v  # main
  stub 02013b38 force ret=u32 args=v  # main
  stub 0202e83c force ret=u32 args=v  # main
  stub 0201407c force ret=u32 args=v,v,v  # main
  stub 02189ba8 ret=ptr:work:0x400:1 args=v  # in-area

# 02187FD0(proc, seq, gamesys, work) (proc main): 02188B58(gamesys, *work) == 1 ? 1 : 0
fn 02187fd0 ret=u32 inputs=200
  arg u32 any
  arg u32 any
  arg u32 any
  arg work 0x240
  fix work 0x240 u32 addrof work 0
  fix work 0x28 u32 alt const 0 || any
  fix work 0x30 u32 alt const 0 || any
  fix work 0x38 u32 alt const 0 || any
  fix work 0x98 u32 alt const 0 || any
  fix work 0xC0 u32 alt oneof 0 1 2 || any
  fix work 0xC4 u32 range 0 5
  fix work 0xC8 u8 alt range 0 7 || oneof 0 1
  fix work 0xC9 u8 range 0 1
  fix work 0xD0 u16 alt oneof 0x3f 0x8c 0x8d 0x8e 0x8f 0 || range 0 0x1ff || any
  fix work 0xD8 u16 alt oneof 0x3f 0x8c 0xFFFF || range 0 0x1ff
  fix work 0xF0 u32 alt const 0 || addrof work 0x480
  fix work 0x108 u32 addrof work 0x400
  fix work 0x118 s32 alt oneof 0xFFFFFFFF 0 1 2 || any
  fix work 0x140 u32 alt const 0 || any
  fix work 0x150 u32 alt const 0 || any
  fix work 0x400 u32 alt range 0 2 || range 0 8
  fix work 0x404 u32 const 0x02EE0101
  fix work 0x408 u32 const 0x02EE0111
  fix work 0x40C u32 const 0x02EE0121
  fix work 0x2C u32 alt const 0 || any
  global 04000204 4
  stub 02188b58 ret=pick:0/1/2/3 args=v,v  # in-area
  stub 021880c8 ret=pick:0/1/2/3 args=p,v  # in-area
  stub 021881d8 ret=pick:0/1/2/3 args=v,v  # in-area
  stub 021885e8 ret=pick:0/1/2/3 args=v,v  # in-area
  stub 02188758 ret=pick:0/1/2/3 args=v,v  # in-area
  stub 02188910 ret=pick:0/1/2/3 args=v,v  # in-area
  stub 02188990 ret=pick:0/1/2/3 args=v,v  # in-area
  stub 02188ad4 ret=pick:0/1/2/3 args=p,v  # in-area
  stub 020433e0 force ret=none args=v  # graphics init (table)
  stub 02188d5c force ret=none args=v,v,v,v  # fieldmap
  stub 0204a48c force ret=none args=v,v,v  # (3 words)
  stub 020434dc force ret=none args=v,v  # display layer on/off (mask, on)
  stub 02043598 force ret=none args=v,v  # display layer on/off, sub screen (mask, on)
  stub 02045088 force ret=none args=v  # (heap) allocates a global 4-byte block
  stub 0201dd68 force ret=none args=  # main
  stub 02046440 force ret=none args=v,v,v,v,v,v,v  # graphics object create (7 words)
  stub 02188ee4 force ret=none args=v  # fieldmap
  stub 02012984 force ret=u32 args=v  # main
  stub 0218c9e8 force ret=u32 args=v,v  # ov21
  stub 020056a0 force ret=u32 args=v,v,v  # main
  stub 02030cb0 force ret=u32 args=v  # task-system size for n tasks (n * 32 + 0x38)
  stub 02030734 force ret=u32 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line): blocks that dereference the result override with ptr:
  stub 02030cbc force ret=u32 args=v,v  # task system init (n, block): returns the block
  stub 021c6d10 force ret=u32 args=v,v  # ov21
  stub 021d317c force ret=u32 args=v  # ov21
  stub 02013cc8 force ret=pick:0/1/2 args=v
  stub 02013ce0 force ret=pick:0/1/2 args=v
  stub 02034ac4 force ret=none args=v  # overlay load (id; -1 = none)
  stub 02035660 force ret=none args=v  # (one argument)
  stub 020120f4 force ret=u32 args=v  # main
  stub 021a9ed8 force ret=u32 args=v  # ov21
  stub 02012138 force ret=u32 args=v  # main
  stub 02026388 force ret=none args=v,v  # main
  stub 021bb890 force ret=u32 args=v,v,v  # fieldmap_func (other F2a area)
  stub 021c1a08 force ret=u32 args=v  # ov21
  stub 021c1af4 force ret=u32 args=v,v  # ov21
  stub 021cf45c force ret=u32 args=v,v  # ov21
  stub 02012ebc force ret=u32 args=v  # main
  stub 020071cc force ret=u32 args=v  # main
  stub 021c31d4 force ret=u32 args=v,v,v  # ov21
  stub 02189684 ret=none args=v,v  # in-area
  stub 020129ac force ret=u32 args=v  # main
  stub 021a0cf4 force ret=u32 args=v,v,v  # ov21
  stub 020120f8 force ret=ptr:work:0x200:1 args=v  # gamesys -> player record (fieldmap reads +4..+0xF)
  stub 02012ae4 force ret=u32 args=v  # main
  stub 02012b28 force ret=pick:0/1/2 args=v
  stub 02013c20 force ret=pick:0/1 args=v
  stub 02012b2c force ret=none args=v,v  # main
  stub 02012944 force ret=u32 args=v  # main
  stub 02008550 force ret=u32 args=v  # main
  stub 021a2e50 force ret=u32 args=v,v,v,v,v  # ov21
  stub 0218d158 force ret=none args=v,v  # ov21
  stub 02189758 ret=none args=v  # in-area
  stub 021be2a8 force ret=u32 args=v,v,v  # ov21
  stub 02188c9c ret=u16 args=v  # in-area
  stub 02189154 force ret=none args=v,v  # fieldmap
  stub 021a0104 force ret=u32 args=v  # ov21
  stub 021a02c4 force ret=u32 args=v  # ov21
  stub 02013eec force ret=u32 args=v  # main
  stub 02013f2c force ret=u32 args=v  # main
  stub 021a0338 force ret=u32 args=v,v,v,v  # ov21
  stub 020140dc force ret=u32 args=v  # main
  stub 0203f4fc force ret=u32 args=  # RTC: seconds of the day (no arguments)
  stub 0202e83c force ret=u32 args=v  # main
  stub 021a03d0 force ret=u32 args=v,v,v,v,v  # ov21
  stub 02012a30 force ret=u32 args=v  # main
  stub 021a19e8 force ret=u32 args=v,v,v,v,v,v,v  # ov21
  stub 02012994 force ret=pick:0/0xF/0x10 args=v
  stub 021a1b74 force ret=none args=v,v,v  # ov21
  stub 021be1dc force ret=none args=v  # ov21
  stub 02189204 ret=none args=v  # in-area
  stub 021abd50 force ret=u32 args=v,v,v  # ov21
  stub 021abf68 force ret=none args=v,v  # ov21
  stub 021d33f8 force ret=u32 args=v  # ov21
  stub 021ac1ec force ret=none args=v,v  # ov21
  stub 021abe30 force ret=none args=v,v,v  # ov21
  stub 0217f8c0 force ret=u32 args=v  # ov18
  stub 02014110 force ret=none args=out:0x1C,v,v,v,v,v,v,v  # LOCATION init (loc, zone, ..)
  stub 0218cee4 force ret=none args=v,v,v  # ov21
  stub 02013c9c force ret=pick:0/1/0xFFFFFFFF args=v
  stub 021bb2f8 force ret=none args=v,v,v,v  # ov21
  stub 02189acc ret=u32 args=v  # in-area
  stub 021895e8 ret=none args=v,v,v  # in-area
  stub 021bb1f4 force ret=u32 args=v,v,v,v  # ov21
  stub 021d33a8 force ret=u32 args=v,v,v,v  # ov21
  stub 0218dc6c force ret=u32 args=v  # ov21
  stub 02012974 force ret=u32 args=v  # main
  stub 02189b30 ret=none args=v,v,v,v  # in-area
  stub 0218b8ac force ret=none args=v,v,v,v  # ov21
  stub 021c6534 force ret=none args=v,v  # ov21
  stub 02189614 ret=none args=v,v  # in-area
  stub 0216447c force ret=none args=  # ov10
  stub 021901ec force ret=u32 args=v,v  # ov21
  stub 021bc388 force ret=u32 args=v,v,v  # ov21
  stub 02012b38 force ret=u32 args=v  # main
  stub 02014470 force ret=u32 args=v  # main
  stub 021c7f70 force ret=u32 args=v,v  # ov21
  stub 02013bbc force ret=u32 args=v  # main
  stub 0218e018 force ret=u32 args=v,v,v,v,v  # ov21
  stub 021ab5f4 force ret=u32 args=v,v,v  # ov21
  stub 021ab7e4 force ret=u32 args=v  # ov21
  stub 02ee0100 force ret=none args=v,v,v  # fieldmap MAP CTRL +4 setup(fieldmap, position, x)
  stub 02014460 force ret=pick:0/1/2 args=v
  stub 02189b04 ret=pick:0/1 args=v  # in-area
  stub 02005c64 force ret=pick:0/1 args=
  stub 02014458 force ret=pick:0/1 args=v
  stub 02158ac8 force ret=none args=v  # ov10
  stub 02014454 force ret=none args=v,v  # main
  stub 0201445c force ret=none args=v,v  # main
  stub 0218ca78 force ret=none args=v  # ov21
  stub 0201209c force ret=none args=v,v,v  # main
  stub 020129dc force ret=none args=v,v  # main
  stub 02188cf0 ret=pick:0/1/2/3 args=v  # in-area
  stub 02158ad4 force ret=none args=v  # ov10
  stub 0216cd64 force ret=none args=v  # ov10
  stub 0219c624 force ret=none args=v  # ov21
  stub 0219c630 force ret=pick:0/1 args=v seq
  stub 021a1b50 force ret=pick:0/0/1 args=v seq
  stub 021a1af4 force ret=none args=v,v  # ov21
  stub 02189004 ret=none args=v  # in-area
  stub 02189084 force ret=none args=v,v,v,v  # fieldmap
  stub 021890ac force ret=u32 args=v  # fieldmap
  stub 0204a600 force ret=none args=  # (no arguments)
  stub 021903e0 force ret=none args=v  # ov21
  stub 021abfdc force ret=none args=v  # ov21
  stub 021bb948 force ret=none args=v  # fieldmap_func (other F2a area)
  stub 02012104 force ret=pick:0/1 args=v
  stub 0201447c force ret=none args=v,v  # main
  stub 02012e40 force ret=pick:0/0/1 args=v
  stub 021888c8 ret=none args=v  # in-area
  stub 02035648 force ret=none args=  # (reads a global, tail call; no arguments)
  stub 021892cc ret=u32 args=v  # in-area
  stub 020120d8 force ret=pick:0/1 args=v
  stub 02168eec force ret=none args=v  # ov10
  stub 021bc484 force ret=none args=v  # ov21
  stub 021c8000 force ret=none args=v  # ov21
  stub 021a2eb8 force ret=none args=v  # ov21
  stub 021737a4 force ret=none args=v  # ov11
  stub 021a0d68 force ret=none args=v  # ov21
  stub 021be258 force ret=none args=v  # ov21
  stub 0217f914 force ret=none args=p,v  # ov18
  stub 02030d28 force ret=none args=v  # task system run (sys)
  stub 021c6d90 force ret=none args=v  # ov21
  stub 02012ed0 force ret=ptr:work:0x220:1 args=v
  stub 021c8660 force ret=u32 args=v  # ov21
  stub 0217ae20 force ret=none args=v,v,v  # ov18
  stub 02ee0110 force ret=none args=v,v  # fieldmap MAP CTRL +8 frame(fieldmap, position)
  stub 020129f0 force ret=none args=v  # main
  stub 0218cc34 force ret=none args=v  # ov21
  stub 021a0e58 force ret=none args=v  # ov21
  stub 021bc4f0 force ret=none args=v  # ov21
  stub 021cf480 force ret=none args=v  # ov21
  stub 021c1c04 force ret=none args=v,v,v,v  # ov21
  stub 021c1ad0 force ret=none args=v,v,v,v  # ov21
  stub 021bb90c force ret=none args=v  # fieldmap_func (other F2a area)
  stub 020263b0 force ret=none args=v,v  # main
  stub 021bc424 force ret=none args=v  # ov21
  stub 021be218 force ret=none args=v  # ov21
  stub 021be2e0 force ret=none args=v  # ov21
  stub 021a9f28 force ret=none args=v  # ov21
  stub 021a0d44 force ret=u32 args=v  # ov21
  stub 020129b8 force ret=none args=v,v  # main
  stub 021abde8 force ret=none args=v  # ov21
  stub 0217f8ec force ret=none args=v  # ov18
  stub 021a1aac force ret=none args=v  # ov21
  stub 021a0448 force ret=none args=v  # ov21
  stub 021a02f4 force ret=none args=v  # ov21
  stub 020307b0 force ret=none args=v  # Heap_Free(block)
  stub 021d33e0 force ret=none args=v  # ov21
  stub 021bb240 force ret=none args=v  # ov21
  stub 021ab81c force ret=none args=v  # ov21
  stub 021ab624 force ret=none args=v  # ov21
  stub 0218e0e4 force ret=none args=v  # ov21
  stub 021895dc ret=none args=v  # in-area
  stub 021892b4 ret=none args=v  # in-area
  stub 021a2e90 force ret=none args=v  # ov21
  stub 0218d05c force ret=none args=v  # ov21
  stub 02190728 force ret=none args=v,p,v,v  # ov21
  stub 021902bc force ret=none args=v  # ov21
  stub 021c7f9c force ret=none args=v  # ov21
  stub 02ee0120 force ret=none args=v  # fieldmap MAP CTRL +0xC teardown(fieldmap)
  stub 02030da4 force ret=none args=  # empty function (bx lr)
  stub 021c6d5c force ret=none args=v  # ov21
  stub 02030eac force ret=none args=v  # task delete (task)
  stub 0218ca4c force ret=none args=v  # ov21
  stub 021890c8 force ret=u32 args=v,v,v,v  # fieldmap
  stub 0204a5ac force ret=none args=  # (no arguments)
  stub 020450c8 force ret=none args=  # frees that global block
  stub 02188e38 force ret=none args=p,v,v,v  # fieldmap
  stub 02034a5c force ret=none args=v  # overlay unload (id; -1 = none)
  stub 02013fa8 force ret=pick:0/2/0xFFFF args=v
  stub 021ab650 force ret=none args=v  # ov21
  stub 021ab868 force ret=u32 args=v  # ov21
  stub 021ab840 force ret=none args=v,v,v,v  # ov21
  stub 021ab880 force ret=u32 args=v  # ov21
  stub 021ab884 force ret=u32 args=v  # ov21
  stub 021ab888 force ret=u32 args=  # ov21
  stub 021ab634 force ret=u32 args=v,v,v,v  # ov21
  stub 02013c80 force ret=pick:0/1 args=v
  stub 02012964 force ret=u32 args=v  # main
  stub 0216ccdc force ret=none args=v,v,v,v,v,v  # ov10
  stub 0219c554 force ret=none args=v,v,v  # ov21
  stub 0219c680 force ret=none args=v  # ov21
  stub 0217c0a8 force ret=u32 args=v  # ov18
  stub 021a31a4 force ret=u32 args=v,v  # ov21
  stub 0219c658 force ret=none args=v,v  # ov21
  stub 0219d448 force ret=none args=v,v  # ov21
  stub 0218ed50 force ret=u32 args=v  # ov21
  stub 0216cdb8 force ret=none args=v,v  # ov10
  stub 0216d2f8 force ret=none args=v  # ov10
  stub 02189ae8 ret=u32 args=v  # in-area
  stub 0216d974 force ret=u32 args=v,v  # ov10
  stub 02013ba8 force ret=pick:0/1/2 args=v
  stub 02013bd0 force ret=pick:0/3/0xFFFF args=v
  stub 0218f14c force ret=u32 args=v,v,v  # ov21
  stub 0218f134 force ret=u32 args=v,p,p,v  # ov21
  stub 02013b50 force ret=pick:0/1/2/3/4/5/6/7 args=v  # zone kind (index of the MAP SETUP table 0x021D3700)
  stub 02159bf4 force ret=u32 args=v  # map_matrix
  stub 02159bf8 force ret=u32 args=v  # map_matrix
  stub 02159bfc force ret=u32 args=v  # map_matrix
  stub 02159c20 force ret=u32 args=v  # map_matrix
  stub 020140bc force ret=u32 args=v  # main
  stub 020140c0 force ret=u32 args=v  # main
  stub 020140cc force ret=u32 args=v  # main
  stub 02013dd0 force ret=pick:0/1 args=v
  stub 0201444c force ret=pick:0/1 args=v
  stub 020127d0 force ret=u32 args=v,v  # main
  stub 0218dc74 force ret=none args=v,v,v  # ov21
  stub 0218dc70 force ret=u32 args=v  # ov21
  stub 02012844 force ret=u32 args=v  # main
  stub 021939c0 force ret=none args=v,v,v,v  # ov21
  stub 02193abc force ret=none args=v,v,v  # ov21
  stub 02193afc force ret=u32 args=v,v  # ov21
  stub 020504d8 force ret=none args=v  # (render object)
  stub 021bb36c force ret=none args=v  # ov21
  stub 021ab660 force ret=none args=v,v  # ov21
  stub 021a030c force ret=none args=v,v  # ov21
  stub 021a0140 force ret=u32 args=v  # ov21
  stub 021a0458 force ret=none args=v,v  # ov21
  stub 0218cb34 force ret=pick:0/1 args=v
  stub 0203d12c force ret=none args=v  # sound / effect request (id)
  stub 02047ce4 force ret=none args=  # clears two words of a global
  stub 0203630c force ret=u32 args=  # key input: trigger mask (no arguments)
  stub 0218e104 force ret=none args=v,v  # ov21
  stub 02189a80 ret=u32 args=v  # in-area
  stub 0218932c ret=pick:0/1 args=v  # in-area
  stub 02189370 ret=pick:0/1 args=v  # in-area
  stub 021893c0 ret=none args=v  # in-area
  stub 02162458 force ret=pick:0/1 args=v
  stub 02162438 force ret=none args=v,v,v  # ov10
  stub 0216d2ac force ret=none args=v  # ov10
  stub 0216cd50 force ret=none args=v  # ov10
  stub 0218ed3c force ret=ptr:work:0x280:1 args=v
  stub 020127a4 force ret=u32 args=v  # main
  stub 02012aa0 force ret=ptr:work:0x260:1 args=v
  stub 020141e0 force ret=none args=v,outm:12:3/0/0/0/0/0/0/0/3/0/0/0
  stub 020141f0 force ret=none args=v,v  # main
  stub 02159c48 force ret=pick:0/1 args=v,v,v  # map_matrix (in bounds)
  stub 02159bd4 force ret=pick:0/1/0x3f/0xFFFF args=v,v,v  # map_matrix (zone id at a position)
  stub 021aa8ec force ret=none args=v  # ov21
  stub 0216df1c force ret=none args=v  # ov10
  stub 021623dc force ret=none args=v  # ov10
  stub 0216242c force ret=none args=v,v  # ov10
  stub 021643ac force ret=none args=v,v  # ov10
  stub 02189518 ret=none args=v,v  # in-area
  stub 0218958c ret=none args=v,v  # in-area
  stub 02189560 ret=none args=v,v  # in-area
  stub 021895d0 ret=none args=v,v  # in-area
  stub 021895bc ret=none args=v,v  # in-area
  stub 021bc550 force ret=none args=v,v  # ov21
  stub 0202735c force ret=none args=v,v  # main
  stub 0215a014 force ret=none args=v,v  # map_matrix
  stub 02158ae0 force ret=none args=v,v  # ov10
  stub 021894e0 ret=none args=v,v,v,v  # in-area
  stub 0215874c force ret=u32 args=v  # ov10
  stub 02012134 force ret=u32 args=v  # main
  stub 0202b544 force ret=u32 args=v,v  # main
  stub 02029cf0 force ret=u32 args=v,v,v  # main
  stub 02188c68 ret=u32 args=v  # in-area
  stub 021a0358 force ret=none args=v  # ov21
  stub 02188c6c ret=u32 args=v  # in-area
  stub 0202c83c force ret=none args=v,v  # main
  stub 021a1c34 force ret=u32 args=v,v  # ov21
  stub 02012ac4 force ret=u32 args=v,v  # main
  stub 02162744 force ret=pick:0/1/5 args=v
  stub 0201296c force ret=u32 args=v  # main
  stub 02162740 force ret=u32 args=v  # ov10
  stub 0216ceac force ret=u32 args=v,v,v,v,v  # ov10

# 02187FE8(proc, seq, gamesys, work) (proc end): 021880A4(*work); 02012110(gamesys, 0); 020315D4(proc); heap delete 0x96, 0x92, 0x93,
# 0x89, 0x50, 0x70, 0x15 (020306F0); returns 1
fn 02187fe8 ret=u32 inputs=100
  arg u32 any
  arg u32 any
  arg u32 any
  arg work 0x240
  fix work 0x240 u32 addrof work 0
  stub 021880a4 ret=none args=v  # in-area
  stub 02012110 force ret=none args=v,v  # main
  stub 020315d4 force ret=none args=v  # proc work free (proc)
  stub 020306f0 force ret=none args=v  # heap delete (heap id)
  stub 020307b0 force ret=none args=v  # Heap_Free(block)
  stub 020371ac force ret=none args=v,v  # writes two halfwords of a RAM block (a, b)

# 02188030(gamesys, heap): fm = Heap_AllocDebug(heap, 0x154, 1, "fieldmap.c", 0x209); +0 = heap (u16); +0xC0 = 0; +4 = gamesys;
# +8 = 020120F4(gamesys); +0xD0 = 02012AC8(020120F8(gamesys)) (u16); +0xC = 0201407C(heap, 02013B38(zone), 0202E83C(gamesys));
# +0x108 = 02189BA8(zone); returns fm
fn 02188030 ret=ofs:work inputs=100
  arg u32 any
  arg u32 any
  stub 02030734 force ret=ptr:work:0:1 args=v,v,v,v,v
  stub 020120f4 force ret=u32 args=v  # main
  stub 020120f8 force ret=ptr:work:0x200:1 args=v  # gamesys -> player record (fieldmap reads +4..+0xF)
  stub 02012ac8 force ret=u32 args=v  # main
  stub 02013b38 force ret=u32 args=v  # main
  stub 0202e83c force ret=u32 args=v  # main
  stub 0201407c force ret=u32 args=v,v,v  # main
  stub 02189ba8 ret=ptr:work:0x400:1 args=v  # in-area
  stub 02013b50 force ret=pick:0/1/2/3/4/5/6/7 args=v  # zone kind (index of the MAP SETUP table 0x021D3700)

# 021880A4(fm): 020140B4(fm+0xC); Heap_Free(fm); 02012110(gamesys (+4), 0); 020371AC(0, 0)
fn 021880a4 ret=none inputs=50
  arg work 0
  stub 020307b0 force ret=none args=v  # Heap_Free(block)
  stub 02012110 force ret=none args=v,v  # main
  stub 020371ac force ret=none args=v,v  # writes two halfwords of a RAM block (a, b)

# 021880C8(gamesys, fm) (proc step 0, graphics init): 020433E0(0x021D3688); 02188D5C(fm); 0204A48C(0x021D3604, 0x021D3688, 0x89);
# 020434DC(0x10, 1); 02043598(0x10, 1); 02045088(heap); 0201DD60(); 02046440(0, 3, 0, 1, 0x1000, heap, 0x02188E51); 02188EE4(fm);
# +0x48 = 0218C9E8(heap, 02012984(gamedata)); +0xAC = 020056A0(0x02189109, fm, 0) (VBlank task); +0x114 = Heap_AllocDebug(heap,
# 02030CB0(0x20), 0, "fieldmap.c", 0x26B); +0x110 = 02030CBC(0x20, +0x114); +0x134 = 021C6D10(10, heap); +0x14C = 021D317C(heap);
# overlay 02034AC4(02013CC8(zone) == 1 || 02013CE0(zone) == 1 ? 0x13 : 0x12); 02035660(0x1E); returns 1
fn 021880c8 ret=u32 inputs=100
  arg u32 any
  arg work 0
  fix work 0x28 u32 alt const 0 || any
  fix work 0x30 u32 alt const 0 || any
  fix work 0x38 u32 alt const 0 || any
  fix work 0x98 u32 alt const 0 || any
  fix work 0xC0 u32 alt oneof 0 1 2 || any
  fix work 0xC4 u32 range 0 5
  fix work 0xC8 u8 alt range 0 7 || oneof 0 1
  fix work 0xC9 u8 range 0 1
  fix work 0xD0 u16 alt oneof 0x3f 0x8c 0x8d 0x8e 0x8f 0 || range 0 0x1ff || any
  fix work 0xD8 u16 alt oneof 0x3f 0x8c 0xFFFF || range 0 0x1ff
  fix work 0xF0 u32 alt const 0 || addrof work 0x480
  fix work 0x108 u32 addrof work 0x400
  fix work 0x118 s32 alt oneof 0xFFFFFFFF 0 1 2 || any
  fix work 0x140 u32 alt const 0 || any
  fix work 0x150 u32 alt const 0 || any
  fix work 0x400 u32 alt range 0 2 || range 0 8
  fix work 0x404 u32 const 0x02EE0101
  fix work 0x408 u32 const 0x02EE0111
  fix work 0x40C u32 const 0x02EE0121
  fix work 0x2C u32 alt const 0 || any
  stub 020433e0 force ret=none args=v  # graphics init (table)
  stub 02188d5c force ret=none args=v,v,v,v  # fieldmap
  stub 0204a48c force ret=none args=v,v,v  # (3 words)
  stub 020434dc force ret=none args=v,v  # display layer on/off (mask, on)
  stub 02043598 force ret=none args=v,v  # display layer on/off, sub screen (mask, on)
  stub 02045088 force ret=none args=v  # (heap) allocates a global 4-byte block
  stub 0201dd68 force ret=none args=  # main
  stub 02046440 force ret=none args=v,v,v,v,v,v,v  # graphics object create (7 words)
  stub 02188ee4 force ret=none args=v  # fieldmap
  stub 02012984 force ret=u32 args=v  # main
  stub 0218c9e8 force ret=u32 args=v,v  # ov21
  stub 020056a0 force ret=u32 args=v,v,v  # main
  stub 02030cb0 force ret=u32 args=v  # task-system size for n tasks (n * 32 + 0x38)
  stub 02030734 force ret=u32 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line): blocks that dereference the result override with ptr:
  stub 02030cbc force ret=u32 args=v,v  # task system init (n, block): returns the block
  stub 021c6d10 force ret=u32 args=v,v  # ov21
  stub 021d317c force ret=u32 args=v  # ov21
  stub 02013cc8 force ret=pick:0/1/2 args=v
  stub 02013ce0 force ret=pick:0/1/2 args=v
  stub 02034ac4 force ret=none args=v  # overlay load (id; -1 = none)
  stub 02035660 force ret=none args=v  # (one argument)

# 021881D8(gamesys, fm) (proc step 1, setup sequence; +0xC8 = sub-step 0..6, then +0xC8 += 1, +0x12C = 0; returns 1 while running,
# 0 after sub-step 6): 0: 0216447C(gd, zone); +0x28 = 021901EC(heap, +0xA4); +0x2C = 0; +0x30 = 021BC388(gamesys, 0x93);
# +0xA0 = 021C7F70(02014470(02012B38(gd)), heap); +0x10 = 0218E018(02013BBC(zone), 0, +0xA4, fm+0xF4, heap); +0x40 = 021AB5F4(heap,
# +0x10, fm); +0x44 = 021AB7E4(heap). 1: +0x3C = 021BB1F4(heap, +0x10, +0x40, +0x44); +0x9C = 021D33A8(heap, gamesys, fm, zone);
# 02189B30(zone, +0xC, fm+0x50, 02012974(gd)); 0218B8AC(0218DC6C(+0x48), zone, +0xC, 021D33F8(+0x9C)); 021C6534(+0x130, ..);
# 02189614(gd, fm, zone). 2: 02014110(fm+0xD4, zone, 0, 0, 0, +0xF4, +0xF8, +0xFC); 0218CEE4(+0x48, fm+0x50, 021D33F8(+0x9C));
# 02013C9C(zone) != -1: 021BB2F8(+0x3C, .., heap); +0x104 = 02189ACC(zone); 021895E8(fm, zone, heap). 3: 02189204(fm);
# +0x94 = 021ABD50(fm, 0x18, heap); 021ABF68(.., 0x5E); 021AC1EC(+0x94, ..); 021ABE30(+0x94, 0x021D8F10, *0x021D8F00);
# zone kind 02013CC8 / 02013CE0 == 1: +0x98 = 0217F8C0(heap). 4: p = 020120F8(gamesys); 02012B28(p) == 1 && 02013C20(zone) == 0:
# 02012B2C(p, 0); +0x8C = 021A2E50(p, fm, p+4, 02008550(02012944(gd)), heap); +0xF4..+0xFF = p+4..; MAP CTRL fn +4 (fm, fm+0xF4,
# 02012AE4(p)); 0218D158(+0x48, fm+0xF4); 02189758(fm+0x118); +0x34 = 021BE2A8(0x3C, 0x23, heap); 02189154(+0xC, 02188C9C(fm));
# +0x18 = 021A0104(heap); +0x1C = 021A02C4(heap); 021A0338(+0x1C, 02013EEC(zone), 02013F2C(zone), 1); +0x14 = 021A03D0(..);
# +0x20 = 021A19E8(..); 021A1B74(+0x20, 02012994(gd), 0x92); 021BE1DC(fm). 5: +0x24 = 021A0CF4(heap, fm, 020129AC(gd)).
# 6: +0x90 = 021A9ED8(fm); 02026388(02012138(gamesys), fm); +0xBC = 021BB890(fm, heap, 0x20); +0x120 = 021C1A08(0x50);
# +0x124 = 021C1AF4(0x50); +0x13C = 021CF45C(heap, fm); +0x128 = 021C31D4(heap, fm, 0200BDB0(02012EBC(gd))); 02189684(fm, zone).
fn 021881d8 ret=u32 inputs=400
  arg u32 any
  arg work 0
  fix work 0x28 u32 alt const 0 || any
  fix work 0x30 u32 alt const 0 || any
  fix work 0x38 u32 alt const 0 || any
  fix work 0x98 u32 alt const 0 || any
  fix work 0xC0 u32 alt oneof 0 1 2 || any
  fix work 0xC4 u32 range 0 5
  fix work 0xC8 u8 alt range 0 7 || oneof 0 1
  fix work 0xC9 u8 range 0 1
  fix work 0xD0 u16 alt oneof 0x3f 0x8c 0x8d 0x8e 0x8f 0 || range 0 0x1ff || any
  fix work 0xD8 u16 alt oneof 0x3f 0x8c 0xFFFF || range 0 0x1ff
  fix work 0xF0 u32 alt const 0 || addrof work 0x480
  fix work 0x108 u32 addrof work 0x400
  fix work 0x118 s32 alt oneof 0xFFFFFFFF 0 1 2 || any
  fix work 0x140 u32 alt const 0 || any
  fix work 0x150 u32 alt const 0 || any
  fix work 0x400 u32 alt range 0 2 || range 0 8
  fix work 0x404 u32 const 0x02EE0101
  fix work 0x408 u32 const 0x02EE0111
  fix work 0x40C u32 const 0x02EE0121
  fix work 0x2C u32 alt const 0 || any
  fix work 0xC8 u8 alt range 0 7 || range 0 6
  stub 020120f4 force ret=u32 args=v  # main
  stub 021a9ed8 force ret=u32 args=v  # ov21
  stub 02012138 force ret=u32 args=v  # main
  stub 02026388 force ret=none args=v,v  # main
  stub 021bb890 force ret=u32 args=v,v,v  # fieldmap_func (other F2a area)
  stub 021c1a08 force ret=u32 args=v  # ov21
  stub 021c1af4 force ret=u32 args=v,v  # ov21
  stub 021cf45c force ret=u32 args=v,v  # ov21
  stub 02012ebc force ret=u32 args=v  # main
  stub 020071cc force ret=u32 args=v  # main
  stub 021c31d4 force ret=u32 args=v,v,v  # ov21
  stub 02189684 ret=none args=v,v  # in-area
  stub 020129ac force ret=u32 args=v  # main
  stub 021a0cf4 force ret=u32 args=v,v,v  # ov21
  stub 020120f8 force ret=ptr:work:0x200:1 args=v  # gamesys -> player record (fieldmap reads +4..+0xF)
  stub 02012ae4 force ret=u32 args=v  # main
  stub 02012b28 force ret=pick:0/1/2 args=v
  stub 02013c20 force ret=pick:0/1 args=v
  stub 02012b2c force ret=none args=v,v  # main
  stub 02012944 force ret=u32 args=v  # main
  stub 02008550 force ret=u32 args=v  # main
  stub 021a2e50 force ret=u32 args=v,v,v,v,v  # ov21
  stub 0218d158 force ret=none args=v,v  # ov21
  stub 02189758 ret=none args=v  # in-area
  stub 021be2a8 force ret=u32 args=v,v,v  # ov21
  stub 02188c9c ret=u16 args=v  # in-area
  stub 02189154 force ret=none args=v,v  # fieldmap
  stub 021a0104 force ret=u32 args=v  # ov21
  stub 021a02c4 force ret=u32 args=v  # ov21
  stub 02013eec force ret=u32 args=v  # main
  stub 02013f2c force ret=u32 args=v  # main
  stub 021a0338 force ret=u32 args=v,v,v,v  # ov21
  stub 020140dc force ret=u32 args=v  # main
  stub 0203f4fc force ret=u32 args=  # RTC: seconds of the day (no arguments)
  stub 0202e83c force ret=u32 args=v  # main
  stub 021a03d0 force ret=u32 args=v,v,v,v,v  # ov21
  stub 02012a30 force ret=u32 args=v  # main
  stub 021a19e8 force ret=u32 args=v,v,v,v,v,v,v  # ov21
  stub 02012994 force ret=pick:0/0xF/0x10 args=v
  stub 021a1b74 force ret=none args=v,v,v  # ov21
  stub 021be1dc force ret=none args=v  # ov21
  stub 02189204 ret=none args=v  # in-area
  stub 021abd50 force ret=u32 args=v,v,v  # ov21
  stub 021abf68 force ret=none args=v,v  # ov21
  stub 021d33f8 force ret=u32 args=v  # ov21
  stub 021ac1ec force ret=none args=v,v  # ov21
  stub 021abe30 force ret=none args=v,v,v  # ov21
  stub 02013cc8 force ret=pick:0/1/2 args=v
  stub 02013ce0 force ret=pick:0/1/2 args=v
  stub 0217f8c0 force ret=u32 args=v  # ov18
  stub 02014110 force ret=none args=out:0x1C,v,v,v,v,v,v,v  # LOCATION init (loc, zone, ..)
  stub 0218cee4 force ret=none args=v,v,v  # ov21
  stub 02013c9c force ret=pick:0/1/0xFFFFFFFF args=v
  stub 021bb2f8 force ret=none args=v,v,v,v  # ov21
  stub 02189acc ret=u32 args=v  # in-area
  stub 021895e8 ret=none args=v,v,v  # in-area
  stub 021bb1f4 force ret=u32 args=v,v,v,v  # ov21
  stub 021d33a8 force ret=u32 args=v,v,v,v  # ov21
  stub 0218dc6c force ret=u32 args=v  # ov21
  stub 02012974 force ret=u32 args=v  # main
  stub 02189b30 ret=none args=v,v,v,v  # in-area
  stub 0218b8ac force ret=none args=v,v,v,v  # ov21
  stub 021c6534 force ret=none args=v,v  # ov21
  stub 02189614 ret=none args=v,v  # in-area
  stub 0216447c force ret=none args=  # ov10
  stub 021901ec force ret=u32 args=v,v  # ov21
  stub 021bc388 force ret=u32 args=v,v,v  # ov21
  stub 02012b38 force ret=u32 args=v  # main
  stub 02014470 force ret=u32 args=v  # main
  stub 021c7f70 force ret=u32 args=v,v  # ov21
  stub 02013bbc force ret=u32 args=v  # main
  stub 0218e018 force ret=u32 args=v,v,v,v,v  # ov21
  stub 021ab5f4 force ret=u32 args=v,v,v  # ov21
  stub 021ab7e4 force ret=u32 args=v  # ov21
  stub 02ee0100 force ret=none args=v,v,v  # fieldmap MAP CTRL +4 setup(fieldmap, position, x)
  stub 02013fa8 force ret=pick:0/2/0xFFFF args=v
  stub 021ab650 force ret=none args=v  # ov21
  stub 021ab868 force ret=u32 args=v  # ov21
  stub 021ab840 force ret=none args=v,v,v,v  # ov21
  stub 021ab880 force ret=u32 args=v  # ov21
  stub 021ab884 force ret=u32 args=v  # ov21
  stub 021ab888 force ret=u32 args=  # ov21
  stub 021ab634 force ret=u32 args=v,v,v,v  # ov21
  stub 02013c80 force ret=pick:0/1 args=v
  stub 02012964 force ret=u32 args=v  # main
  stub 0216ccdc force ret=none args=v,v,v,v,v,v  # ov10
  stub 0219c554 force ret=none args=v,v,v  # ov21
  stub 0219c680 force ret=none args=v  # ov21
  stub 0217c0a8 force ret=u32 args=v  # ov18
  stub 021a31a4 force ret=u32 args=v,v  # ov21
  stub 0219c658 force ret=none args=v,v  # ov21
  stub 0219d448 force ret=none args=v,v  # ov21
  stub 0218ed50 force ret=u32 args=v  # ov21
  stub 0216cdb8 force ret=none args=v,v  # ov10
  stub 0216d2f8 force ret=none args=v  # ov10
  stub 02189ae8 ret=u32 args=v  # in-area
  stub 0216d974 force ret=u32 args=v,v  # ov10
  stub 02013ba8 force ret=pick:0/1/2 args=v
  stub 02013bd0 force ret=pick:0/3/0xFFFF args=v
  stub 0218f14c force ret=u32 args=v,v,v  # ov21
  stub 0218f134 force ret=u32 args=v,p,p,v  # ov21
  stub 02013b50 force ret=pick:0/1/2/3/4/5/6/7 args=v  # zone kind (index of the MAP SETUP table 0x021D3700)
  stub 02159bf4 force ret=u32 args=v  # map_matrix
  stub 02159bf8 force ret=u32 args=v  # map_matrix
  stub 02159bfc force ret=u32 args=v  # map_matrix
  stub 02159c20 force ret=u32 args=v  # map_matrix
  stub 020140bc force ret=u32 args=v  # main
  stub 020140c0 force ret=u32 args=v  # main
  stub 020140cc force ret=u32 args=v  # main
  stub 02013dd0 force ret=pick:0/1 args=v
  stub 0201444c force ret=pick:0/1 args=v
  stub 020127d0 force ret=u32 args=v,v  # main
  stub 0218dc74 force ret=none args=v,v,v  # ov21
  stub 0218dc70 force ret=u32 args=v  # ov21
  stub 02012844 force ret=u32 args=v  # main
  stub 021939c0 force ret=none args=v,v,v,v  # ov21
  stub 02193abc force ret=none args=v,v,v  # ov21
  stub 02193afc force ret=u32 args=v,v  # ov21

# 021885E8(gamesys, fm) (proc step 2): sub-step 0: 0218CA78(+0x48); +0x38 != 0: do { 0216CD64(+0x38); 0219C624(+0x38) } while
# 0219C630(+0x38) == 1; while 021A1B50(+0x20): 021A1AF4(+0x20, 0x92); 02189004(fm); 02189084(fm); 021890AC(fm); 0204A600();
# +0x28: 021903E0(+0x28); 021ABE04(+0x94); 021BB948(+0xBC); sub-step += 1. Sub-step 0 or 1 (falls through): 02014460(02012B38(gd)) == 1
# && 02189B04(zone) == 1 && 02005C64(): returns 0; 02014458(..) ? (02158AC8(gamesys, heap), 02014454(.., 0)) : 02158AD4(gamesys, heap);
# 0201445C(.., 0); 0218CA78(+0x48); event check fn = zone kind 02013CC8 / 02013CE0 == 1 ? 0x0218A0E5 : ctrl kind 02188CF0(fm) == 1 ?
# 0x0218A241 : == 2 ? 0x0218A4BD : 0x0218A075; 0201209C(gamesys, fn, fm); +0xC0 = 1; 020129DC(020120F4(gamesys), 1); returns 1.
# Other sub-steps: 1.
fn 021885e8 ret=u32 inputs=400
  arg u32 any
  arg work 0
  fix work 0x28 u32 alt const 0 || any
  fix work 0x30 u32 alt const 0 || any
  fix work 0x38 u32 alt const 0 || any
  fix work 0x98 u32 alt const 0 || any
  fix work 0xC0 u32 alt oneof 0 1 2 || any
  fix work 0xC4 u32 range 0 5
  fix work 0xC8 u8 alt range 0 7 || oneof 0 1
  fix work 0xC9 u8 range 0 1
  fix work 0xD0 u16 alt oneof 0x3f 0x8c 0x8d 0x8e 0x8f 0 || range 0 0x1ff || any
  fix work 0xD8 u16 alt oneof 0x3f 0x8c 0xFFFF || range 0 0x1ff
  fix work 0xF0 u32 alt const 0 || addrof work 0x480
  fix work 0x108 u32 addrof work 0x400
  fix work 0x118 s32 alt oneof 0xFFFFFFFF 0 1 2 || any
  fix work 0x140 u32 alt const 0 || any
  fix work 0x150 u32 alt const 0 || any
  fix work 0x400 u32 alt range 0 2 || range 0 8
  fix work 0x404 u32 const 0x02EE0101
  fix work 0x408 u32 const 0x02EE0111
  fix work 0x40C u32 const 0x02EE0121
  fix work 0x2C u32 alt const 0 || any
  global 04000204 4
  fix work 0xC8 u8 alt range 0 2 || any
  stub 02012b38 force ret=u32 args=v  # main
  stub 02014460 force ret=pick:0/1/2 args=v
  stub 02189b04 ret=pick:0/1 args=v  # in-area
  stub 02005c64 force ret=pick:0/1 args=
  stub 02014458 force ret=pick:0/1 args=v
  stub 02158ac8 force ret=none args=v  # ov10
  stub 02014454 force ret=none args=v,v  # main
  stub 0201445c force ret=none args=v,v  # main
  stub 0218ca78 force ret=none args=v  # ov21
  stub 02013cc8 force ret=pick:0/1/2 args=v
  stub 02013ce0 force ret=pick:0/1/2 args=v
  stub 0201209c force ret=none args=v,v,v  # main
  stub 020120f4 force ret=u32 args=v  # main
  stub 020129dc force ret=none args=v,v  # main
  stub 02188cf0 ret=pick:0/1/2/3 args=v  # in-area
  stub 02158ad4 force ret=none args=v  # ov10
  stub 0216cd64 force ret=none args=v  # ov10
  stub 0219c624 force ret=none args=v  # ov21
  stub 0219c630 force ret=pick:0/1 args=v seq
  stub 021a1b50 force ret=pick:0/0/1 args=v seq
  stub 021a1af4 force ret=none args=v,v  # ov21
  stub 02189004 ret=none args=v  # in-area
  stub 02189084 force ret=none args=v,v,v,v  # fieldmap
  stub 021890ac force ret=u32 args=v  # fieldmap
  stub 0204a600 force ret=none args=  # (no arguments)
  stub 021903e0 force ret=none args=v  # ov21
  stub 021abfdc force ret=none args=v  # ov21
  stub 021bb948 force ret=none args=v  # fieldmap_func (other F2a area)
  stub 020504d8 force ret=none args=v  # (render object)
  stub 021bb36c force ret=none args=v  # ov21
  stub 021ab660 force ret=none args=v,v  # ov21
  stub 021a030c force ret=none args=v,v  # ov21
  stub 021a0140 force ret=u32 args=v  # ov21
  stub 0203f4fc force ret=u32 args=  # RTC: seconds of the day (no arguments)
  stub 021a0458 force ret=none args=v,v  # ov21
  stub 0218cb34 force ret=pick:0/1 args=v
  stub 0203d12c force ret=none args=v  # sound / effect request (id)

# 02188758(gamesys, fm) (proc step 3, frame update): !02012104(gamesys): 0201447C(02012B38(gd), 0); 02012E40(gd): return 0;
# +0x140: 021888C8(fm); 0204A600(); return 0. 02035648(); 021892CC(fm); !020120D8(gamesys): MAP CTRL fn +8 (fm, fm+0xF4),
# 02168EEC(gd); +0x30: 021BC484(+0x30); 021C7FAC(+0xA0); 021A2EB8(+0x8C); zone kind 02013CC8 / 02013CE0 == 1:
# 021737A4(02012138(gamesys), fm) else 0217AE20(fm, gamesys, +0x8C); 021A0D68(+0x24); +0x38: EXMEMCNT &= 0xFF7F, 0216CD64(+0x38),
# EXMEMCNT |= 0x80; 021BE258(fm); 021ABE04(+0x94); +0x98: 0217F914(+0x98); 021BB948(+0xBC); +0x28: 021903E0(+0x28);
# 02030D28(+0x110); 021C6D90(+0x134); +0xF0: copy its 12 bytes to +0xF4; 0218D158(+0x48, fm+0xF4); 02189004(fm);
# 021C8660(*02012ED0(gd)); 0204A600(); 021888C8(fm); returns 0
fn 02188758 ret=u32 inputs=400
  arg u32 any
  arg work 0
  fix work 0x28 u32 alt const 0 || any
  fix work 0x30 u32 alt const 0 || any
  fix work 0x38 u32 alt const 0 || any
  fix work 0x98 u32 alt const 0 || any
  fix work 0xC0 u32 alt oneof 0 1 2 || any
  fix work 0xC4 u32 range 0 5
  fix work 0xC8 u8 alt range 0 7 || oneof 0 1
  fix work 0xC9 u8 range 0 1
  fix work 0xD0 u16 alt oneof 0x3f 0x8c 0x8d 0x8e 0x8f 0 || range 0 0x1ff || any
  fix work 0xD8 u16 alt oneof 0x3f 0x8c 0xFFFF || range 0 0x1ff
  fix work 0xF0 u32 alt const 0 || addrof work 0x480
  fix work 0x108 u32 addrof work 0x400
  fix work 0x118 s32 alt oneof 0xFFFFFFFF 0 1 2 || any
  fix work 0x140 u32 alt const 0 || any
  fix work 0x150 u32 alt const 0 || any
  fix work 0x400 u32 alt range 0 2 || range 0 8
  fix work 0x404 u32 const 0x02EE0101
  fix work 0x408 u32 const 0x02EE0111
  fix work 0x40C u32 const 0x02EE0121
  fix work 0x2C u32 alt const 0 || any
  global 04000204 4
  stub 02012104 force ret=pick:0/1 args=v
  stub 02012b38 force ret=u32 args=v  # main
  stub 0201447c force ret=none args=v,v  # main
  stub 02012e40 force ret=pick:0/0/1 args=v
  stub 021888c8 ret=none args=v  # in-area
  stub 0204a600 force ret=none args=  # (no arguments)
  stub 02035648 force ret=none args=  # (reads a global, tail call; no arguments)
  stub 021892cc ret=u32 args=v  # in-area
  stub 020120d8 force ret=pick:0/1 args=v
  stub 02168eec force ret=none args=v  # ov10
  stub 021bc484 force ret=none args=v  # ov21
  stub 021c8000 force ret=none args=v  # ov21
  stub 021a2eb8 force ret=none args=v  # ov21
  stub 02013cc8 force ret=pick:0/1/2 args=v
  stub 02013ce0 force ret=pick:0/1/2 args=v
  stub 02012138 force ret=u32 args=v  # main
  stub 021737a4 force ret=none args=v  # ov11
  stub 021a0d68 force ret=none args=v  # ov21
  stub 0216cd64 force ret=none args=v  # ov10
  stub 021be258 force ret=none args=v  # ov21
  stub 021abfdc force ret=none args=v  # ov21
  stub 0217f914 force ret=none args=p,v  # ov18
  stub 021bb948 force ret=none args=v  # fieldmap_func (other F2a area)
  stub 021903e0 force ret=none args=v  # ov21
  stub 02030d28 force ret=none args=v  # task system run (sys)
  stub 021c6d90 force ret=none args=v  # ov21
  stub 0218d158 force ret=none args=v,v  # ov21
  stub 02189004 ret=none args=v  # in-area
  stub 02012ed0 force ret=ptr:work:0x220:1 args=v
  stub 021c8660 force ret=u32 args=v  # ov21
  stub 0217ae20 force ret=none args=v,v,v  # ov18
  stub 02ee0110 force ret=none args=v,v  # fieldmap MAP CTRL +8 frame(fieldmap, position)
  stub 02047ce4 force ret=none args=  # clears two words of a global
  stub 0203630c force ret=u32 args=  # key input: trigger mask (no arguments)
  stub 0218e104 force ret=none args=v,v  # ov21
  stub 02189a80 ret=u32 args=v  # in-area
  stub 02189084 force ret=none args=v,v,v,v  # fieldmap
  stub 0218932c ret=pick:0/1 args=v  # in-area
  stub 02189370 ret=pick:0/1 args=v  # in-area
  stub 021893c0 ret=none args=v  # in-area
  stub 02012844 force ret=u32 args=v  # main
  stub 02162458 force ret=pick:0/1 args=v
  stub 02012984 force ret=u32 args=v  # main
  stub 02162438 force ret=none args=v,v,v  # ov10
  stub 020504d8 force ret=none args=v  # (render object)
  stub 021bb36c force ret=none args=v  # ov21
  stub 021ab660 force ret=none args=v,v  # ov21
  stub 021a030c force ret=none args=v,v  # ov21
  stub 021a1af4 force ret=none args=v,v  # ov21
  stub 021a0140 force ret=u32 args=v  # ov21
  stub 0203f4fc force ret=u32 args=  # RTC: seconds of the day (no arguments)
  stub 021a0458 force ret=none args=v,v  # ov21
  stub 0218cb34 force ret=pick:0/1 args=v
  stub 0203d12c force ret=none args=v  # sound / effect request (id)
  stub 0218ed3c force ret=ptr:work:0x280:1 args=v
  stub 020120f4 force ret=u32 args=v  # main
  stub 020127a4 force ret=u32 args=v  # main
  stub 02012aa0 force ret=ptr:work:0x260:1 args=v
  stub 020141e0 force ret=none args=v,outm:12:3/0/0/0/0/0/0/0/3/0/0/0
  stub 020141f0 force ret=none args=v,v  # main
  stub 02012974 force ret=u32 args=v  # main
  stub 02159c48 force ret=pick:0/1 args=v,v,v  # map_matrix (in bounds)
  stub 02159bd4 force ret=pick:0/1/0x3f/0xFFFF args=v,v,v  # map_matrix (zone id at a position)
  stub 021aa8ec force ret=none args=v  # ov21
  stub 0216df1c force ret=none args=v  # ov10
  stub 021623dc force ret=none args=v  # ov10
  stub 0216242c force ret=none args=v,v  # ov10
  stub 021643ac force ret=none args=v,v  # ov10
  stub 02189518 ret=none args=v,v  # in-area
  stub 0218958c ret=none args=v,v  # in-area
  stub 02189560 ret=none args=v,v  # in-area
  stub 021895d0 ret=none args=v,v  # in-area
  stub 021895bc ret=none args=v,v  # in-area
  stub 02189684 ret=none args=v,v  # in-area
  stub 021bc550 force ret=none args=v,v  # ov21
  stub 0202735c force ret=none args=v,v  # main
  stub 0215a014 force ret=none args=v,v  # map_matrix
  stub 02158ae0 force ret=none args=v,v  # ov10
  stub 021894e0 ret=none args=v,v,v,v  # in-area
  stub 0215874c force ret=u32 args=v  # ov10
  stub 021895e8 ret=none args=v,v,v  # in-area
  stub 02012134 force ret=u32 args=v  # main
  stub 02012a30 force ret=u32 args=v  # main
  stub 02012b28 force ret=pick:0/1/2 args=v
  stub 0202b544 force ret=u32 args=v,v  # main
  stub 02029cf0 force ret=u32 args=v,v,v  # main
  stub 02188c68 ret=u32 args=v  # in-area
  stub 021a0358 force ret=none args=v  # ov21
  stub 02013eec force ret=u32 args=v  # main
  stub 02013f2c force ret=u32 args=v  # main
  stub 021a0338 force ret=u32 args=v,v,v,v  # ov21
  stub 02188c6c ret=u32 args=v  # in-area
  stub 0202c83c force ret=none args=v,v  # main
  stub 02012994 force ret=pick:0/0xF/0x10 args=v
  stub 021a1c34 force ret=u32 args=v,v  # ov21
  stub 02012ac4 force ret=u32 args=v,v  # main
  stub 02013fa8 force ret=pick:0/2/0xFFFF args=v
  stub 021ab650 force ret=none args=v  # ov21
  stub 021ab868 force ret=u32 args=v  # ov21
  stub 021ab840 force ret=none args=v,v,v,v  # ov21
  stub 021ab880 force ret=u32 args=v  # ov21
  stub 021ab884 force ret=u32 args=v  # ov21
  stub 021ab888 force ret=u32 args=  # ov21
  stub 021ab634 force ret=u32 args=v,v,v,v  # ov21
  stub 02013c80 force ret=pick:0/1 args=v
  stub 02162744 force ret=pick:0/1/5 args=v
  stub 0201296c force ret=u32 args=v  # main
  stub 02162740 force ret=u32 args=v  # ov10
  stub 0216ceac force ret=u32 args=v,v,v,v,v  # ov10
  stub 02013bd0 force ret=pick:0/3/0xFFFF args=v
  stub 0218f14c force ret=u32 args=v,v,v  # ov21
  stub 0218f134 force ret=u32 args=v,p,p,v  # ov21

# 021888C8(fm) (draw step): 02047CE4(); 0218E104(+0x10, (u16) 0203630C()); +0x100 = 02189A80(fm); EXMEMCNT &= 0xFF7F;
# 02189084(fm); EXMEMCNT |= 0x80
fn 021888c8 ret=none inputs=60
  arg work 0
  fix work 0x28 u32 alt const 0 || any
  fix work 0x30 u32 alt const 0 || any
  fix work 0x38 u32 alt const 0 || any
  fix work 0x98 u32 alt const 0 || any
  fix work 0xC0 u32 alt oneof 0 1 2 || any
  fix work 0xC4 u32 range 0 5
  fix work 0xC8 u8 alt range 0 7 || oneof 0 1
  fix work 0xC9 u8 range 0 1
  fix work 0xD0 u16 alt oneof 0x3f 0x8c 0x8d 0x8e 0x8f 0 || range 0 0x1ff || any
  fix work 0xD8 u16 alt oneof 0x3f 0x8c 0xFFFF || range 0 0x1ff
  fix work 0xF0 u32 alt const 0 || addrof work 0x480
  fix work 0x108 u32 addrof work 0x400
  fix work 0x118 s32 alt oneof 0xFFFFFFFF 0 1 2 || any
  fix work 0x140 u32 alt const 0 || any
  fix work 0x150 u32 alt const 0 || any
  fix work 0x400 u32 alt range 0 2 || range 0 8
  fix work 0x404 u32 const 0x02EE0101
  fix work 0x408 u32 const 0x02EE0111
  fix work 0x40C u32 const 0x02EE0121
  fix work 0x2C u32 alt const 0 || any
  global 04000204 4
  stub 02047ce4 force ret=none args=  # clears two words of a global
  stub 0203630c force ret=u32 args=  # key input: trigger mask (no arguments)
  stub 0218e104 force ret=none args=v,v  # ov21
  stub 02189a80 ret=u32 args=v  # in-area
  stub 02189084 force ret=none args=v,v,v,v  # fieldmap
  stub 0218ed3c force ret=ptr:work:0x280:1 args=v

# 02188910(gamesys, fm) (proc step 3 half 1, draw): 02012E40(gd) != 0: 020129F0(020120F4(gamesys)), return 0; EXMEMCNT &= 0xFF7F;
# 0218CC34(+0x48); EXMEMCNT |= 0x80; 021A0E58(+0x24); +0x28: 021903E0(+0x28); +0x30: 021BC4F0(+0x30); EXMEMCNT &= 0xFF7F;
# 021890AC(fm); EXMEMCNT |= 0x80; 020129F0(020120F4(gamesys)); returns 0
fn 02188910 ret=u32 inputs=200
  arg u32 any
  arg work 0
  fix work 0x28 u32 alt const 0 || any
  fix work 0x30 u32 alt const 0 || any
  fix work 0x38 u32 alt const 0 || any
  fix work 0x98 u32 alt const 0 || any
  fix work 0xC0 u32 alt oneof 0 1 2 || any
  fix work 0xC4 u32 range 0 5
  fix work 0xC8 u8 alt range 0 7 || oneof 0 1
  fix work 0xC9 u8 range 0 1
  fix work 0xD0 u16 alt oneof 0x3f 0x8c 0x8d 0x8e 0x8f 0 || range 0 0x1ff || any
  fix work 0xD8 u16 alt oneof 0x3f 0x8c 0xFFFF || range 0 0x1ff
  fix work 0xF0 u32 alt const 0 || addrof work 0x480
  fix work 0x108 u32 addrof work 0x400
  fix work 0x118 s32 alt oneof 0xFFFFFFFF 0 1 2 || any
  fix work 0x140 u32 alt const 0 || any
  fix work 0x150 u32 alt const 0 || any
  fix work 0x400 u32 alt range 0 2 || range 0 8
  fix work 0x404 u32 const 0x02EE0101
  fix work 0x408 u32 const 0x02EE0111
  fix work 0x40C u32 const 0x02EE0121
  fix work 0x2C u32 alt const 0 || any
  global 04000204 4
  stub 02012e40 force ret=pick:0/1 args=v
  stub 020120f4 force ret=u32 args=v  # main
  stub 020129f0 force ret=none args=v  # main
  stub 0218cc34 force ret=none args=v  # ov21
  stub 021a0e58 force ret=none args=v  # ov21
  stub 021903e0 force ret=none args=v  # ov21
  stub 021bc4f0 force ret=none args=v  # ov21
  stub 021890ac force ret=u32 args=v  # fieldmap

# 02188990(gamesys, fm) (proc step 4, teardown): 021CF480(+0x13C); 021C1C04(+0x124); 021C1AD0(+0x120); 0201209C(gamesys, 0, 0);
# 021A2EB8(+0x8C); 021BB90C(+0xBC); 020263B0(02012138(gamesys), fm); +0x30: 021BC424(+0x30); 021BE218(fm); 021BE2E0(+0x34);
# 021A9F28(+0x90); 020129B8(020120F4(gamesys), 021A0D44(+0x24)); 021ABDE8(+0x94); +0x98: 0217F8EC(+0x98), +0x98 = 0;
# 021A1AAC(+0x20); +0x20 = 0; 021A0448(+0x14); 021A02F4(+0x1C); 021A0138(+0x18); 021D33E0(+0x9C); 021BB240(+0x3C); 021AB81C(+0x44);
# 021AB624(+0x40); 0218E0E4(+0x10); MAP CTRL fn +0xC (fm); 021895DC(fm); 021892B4(fm); 021A2E90(+0x8C); 0218D05C(+0x48);
# +0x2C: 02190728(+0x2C); +0x28: 021902BC(+0x28); 020129DC(020120F4(gamesys), 0); 02035660(0x3C); 021C31FC(+0x128);
# 021C7F9C(+0xA0); returns 1 (Ghidra stops after 021A0138)
fn 02188990 ret=u32 inputs=200
  arg u32 any
  arg work 0
  fix work 0x28 u32 alt const 0 || any
  fix work 0x30 u32 alt const 0 || any
  fix work 0x38 u32 alt const 0 || any
  fix work 0x98 u32 alt const 0 || any
  fix work 0xC0 u32 alt oneof 0 1 2 || any
  fix work 0xC4 u32 range 0 5
  fix work 0xC8 u8 alt range 0 7 || oneof 0 1
  fix work 0xC9 u8 range 0 1
  fix work 0xD0 u16 alt oneof 0x3f 0x8c 0x8d 0x8e 0x8f 0 || range 0 0x1ff || any
  fix work 0xD8 u16 alt oneof 0x3f 0x8c 0xFFFF || range 0 0x1ff
  fix work 0xF0 u32 alt const 0 || addrof work 0x480
  fix work 0x108 u32 addrof work 0x400
  fix work 0x118 s32 alt oneof 0xFFFFFFFF 0 1 2 || any
  fix work 0x140 u32 alt const 0 || any
  fix work 0x150 u32 alt const 0 || any
  fix work 0x400 u32 alt range 0 2 || range 0 8
  fix work 0x404 u32 const 0x02EE0101
  fix work 0x408 u32 const 0x02EE0111
  fix work 0x40C u32 const 0x02EE0121
  fix work 0x2C u32 alt const 0 || any
  stub 021cf480 force ret=none args=v  # ov21
  stub 021c1c04 force ret=none args=v,v,v,v  # ov21
  stub 021c1ad0 force ret=none args=v,v,v,v  # ov21
  stub 0201209c force ret=none args=v,v,v  # main
  stub 021a2eb8 force ret=none args=v  # ov21
  stub 021bb90c force ret=none args=v  # fieldmap_func (other F2a area)
  stub 02012138 force ret=u32 args=v  # main
  stub 020263b0 force ret=none args=v,v  # main
  stub 021bc424 force ret=none args=v  # ov21
  stub 021be218 force ret=none args=v  # ov21
  stub 021be2e0 force ret=none args=v  # ov21
  stub 021a9f28 force ret=none args=v  # ov21
  stub 020120f4 force ret=u32 args=v  # main
  stub 021a0d44 force ret=u32 args=v  # ov21
  stub 020129b8 force ret=none args=v,v  # main
  stub 021abde8 force ret=none args=v  # ov21
  stub 0217f8ec force ret=none args=v  # ov18
  stub 021a1aac force ret=none args=v  # ov21
  stub 021a0448 force ret=none args=v  # ov21
  stub 021a02f4 force ret=none args=v  # ov21
  stub 020307b0 force ret=none args=v  # Heap_Free(block)
  stub 021d33e0 force ret=none args=v  # ov21
  stub 021bb240 force ret=none args=v  # ov21
  stub 021ab81c force ret=none args=v  # ov21
  stub 021ab624 force ret=none args=v  # ov21
  stub 0218e0e4 force ret=none args=v  # ov21
  stub 021895dc ret=none args=v  # in-area
  stub 021892b4 ret=none args=v  # in-area
  stub 021a2e90 force ret=none args=v  # ov21
  stub 0218d05c force ret=none args=v  # ov21
  stub 02190728 force ret=none args=v,p,v,v  # ov21
  stub 021902bc force ret=none args=v  # ov21
  stub 020129dc force ret=none args=v,v  # main
  stub 02035660 force ret=none args=v  # (one argument)
  stub 021c7f9c force ret=none args=v  # ov21
  stub 02ee0120 force ret=none args=v  # fieldmap MAP CTRL +0xC teardown(fieldmap)
  stub 0216d2ac force ret=none args=v  # ov10
  stub 0216cd50 force ret=none args=v  # ov10

# 02188AD4(gamesys, fm) (proc step 5, graphics exit): 02030DA4(+0x110); Heap_Free(+0x114); 021D31A4(+0x14C); +0x14C = 0;
# 021C6D5C(+0x134); 02030EAC(+0xAC); 0218CA4C(+0x48); 021890C8(fm); 0204A5AC(); 020450C8(); 02188E38(fm);
# overlay unload 02034A5C(zone kind 02013CC8 / 02013CE0 == 1 ? 0x13 : 0x12); returns 2
fn 02188ad4 ret=u32 inputs=100
  arg u32 any
  arg work 0
  fix work 0x28 u32 alt const 0 || any
  fix work 0x30 u32 alt const 0 || any
  fix work 0x38 u32 alt const 0 || any
  fix work 0x98 u32 alt const 0 || any
  fix work 0xC0 u32 alt oneof 0 1 2 || any
  fix work 0xC4 u32 range 0 5
  fix work 0xC8 u8 alt range 0 7 || oneof 0 1
  fix work 0xC9 u8 range 0 1
  fix work 0xD0 u16 alt oneof 0x3f 0x8c 0x8d 0x8e 0x8f 0 || range 0 0x1ff || any
  fix work 0xD8 u16 alt oneof 0x3f 0x8c 0xFFFF || range 0 0x1ff
  fix work 0xF0 u32 alt const 0 || addrof work 0x480
  fix work 0x108 u32 addrof work 0x400
  fix work 0x118 s32 alt oneof 0xFFFFFFFF 0 1 2 || any
  fix work 0x140 u32 alt const 0 || any
  fix work 0x150 u32 alt const 0 || any
  fix work 0x400 u32 alt range 0 2 || range 0 8
  fix work 0x404 u32 const 0x02EE0101
  fix work 0x408 u32 const 0x02EE0111
  fix work 0x40C u32 const 0x02EE0121
  fix work 0x2C u32 alt const 0 || any
  stub 02030da4 force ret=none args=  # empty function (bx lr)
  stub 020307b0 force ret=none args=v  # Heap_Free(block)
  stub 021c6d5c force ret=none args=v  # ov21
  stub 02030eac force ret=none args=v  # task delete (task)
  stub 0218ca4c force ret=none args=v  # ov21
  stub 021890c8 force ret=u32 args=v,v,v,v  # fieldmap
  stub 0204a5ac force ret=none args=  # (no arguments)
  stub 020450c8 force ret=none args=  # frees that global block
  stub 02188e38 force ret=none args=p,v,v,v  # fieldmap
  stub 02013cc8 force ret=pick:0/1/2 args=v
  stub 02013ce0 force ret=pick:0/1/2 args=v
  stub 02034a5c force ret=none args=v  # overlay unload (id; -1 = none)

# 02188B58(gamesys, fm) (proc driver): +0xCC += 1; f = PROC table 0x021D36B8 [+0xC4] [+0xC9]; r = f ? f(gamesys, fm) : 0;
# r 0: +0xC9 = !+0xC9; 1: +0xC4 += 1, +0xC8 = +0xC9 = 0; 2: returns 1; returns 0
fn 02188b58 ret=u32 inputs=300
  arg u32 any
  arg work 0
  fix work 0x28 u32 alt const 0 || any
  fix work 0x30 u32 alt const 0 || any
  fix work 0x38 u32 alt const 0 || any
  fix work 0x98 u32 alt const 0 || any
  fix work 0xC0 u32 alt oneof 0 1 2 || any
  fix work 0xC4 u32 range 0 5
  fix work 0xC8 u8 alt range 0 7 || oneof 0 1
  fix work 0xC9 u8 range 0 1
  fix work 0xD0 u16 alt oneof 0x3f 0x8c 0x8d 0x8e 0x8f 0 || range 0 0x1ff || any
  fix work 0xD8 u16 alt oneof 0x3f 0x8c 0xFFFF || range 0 0x1ff
  fix work 0xF0 u32 alt const 0 || addrof work 0x480
  fix work 0x108 u32 addrof work 0x400
  fix work 0x118 s32 alt oneof 0xFFFFFFFF 0 1 2 || any
  fix work 0x140 u32 alt const 0 || any
  fix work 0x150 u32 alt const 0 || any
  fix work 0x400 u32 alt range 0 2 || range 0 8
  fix work 0x404 u32 const 0x02EE0101
  fix work 0x408 u32 const 0x02EE0111
  fix work 0x40C u32 const 0x02EE0121
  fix work 0x2C u32 alt const 0 || any
  global 04000204 4
  stub 021880c8 ret=pick:0/1/2/3 args=p,v  # in-area
  stub 021881d8 ret=pick:0/1/2/3 args=v,v  # in-area
  stub 021885e8 ret=pick:0/1/2/3 args=v,v  # in-area
  stub 02188758 ret=pick:0/1/2/3 args=v,v  # in-area
  stub 02188910 ret=pick:0/1/2/3 args=v,v  # in-area
  stub 02188990 ret=pick:0/1/2/3 args=v,v  # in-area
  stub 02188ad4 ret=pick:0/1/2/3 args=p,v  # in-area
  stub 020433e0 force ret=none args=v  # graphics init (table)
  stub 02188d5c force ret=none args=v,v,v,v  # fieldmap
  stub 0204a48c force ret=none args=v,v,v  # (3 words)
  stub 020434dc force ret=none args=v,v  # display layer on/off (mask, on)
  stub 02043598 force ret=none args=v,v  # display layer on/off, sub screen (mask, on)
  stub 02045088 force ret=none args=v  # (heap) allocates a global 4-byte block
  stub 0201dd68 force ret=none args=  # main
  stub 02046440 force ret=none args=v,v,v,v,v,v,v  # graphics object create (7 words)
  stub 02188ee4 force ret=none args=v  # fieldmap
  stub 02012984 force ret=u32 args=v  # main
  stub 0218c9e8 force ret=u32 args=v,v  # ov21
  stub 020056a0 force ret=u32 args=v,v,v  # main
  stub 02030cb0 force ret=u32 args=v  # task-system size for n tasks (n * 32 + 0x38)
  stub 02030734 force ret=u32 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line): blocks that dereference the result override with ptr:
  stub 02030cbc force ret=u32 args=v,v  # task system init (n, block): returns the block
  stub 021c6d10 force ret=u32 args=v,v  # ov21
  stub 021d317c force ret=u32 args=v  # ov21
  stub 02013cc8 force ret=pick:0/1/2 args=v
  stub 02013ce0 force ret=pick:0/1/2 args=v
  stub 02034ac4 force ret=none args=v  # overlay load (id; -1 = none)
  stub 02035660 force ret=none args=v  # (one argument)
  stub 020120f4 force ret=u32 args=v  # main
  stub 021a9ed8 force ret=u32 args=v  # ov21
  stub 02012138 force ret=u32 args=v  # main
  stub 02026388 force ret=none args=v,v  # main
  stub 021bb890 force ret=u32 args=v,v,v  # fieldmap_func (other F2a area)
  stub 021c1a08 force ret=u32 args=v  # ov21
  stub 021c1af4 force ret=u32 args=v,v  # ov21
  stub 021cf45c force ret=u32 args=v,v  # ov21
  stub 02012ebc force ret=u32 args=v  # main
  stub 020071cc force ret=u32 args=v  # main
  stub 021c31d4 force ret=u32 args=v,v,v  # ov21
  stub 02189684 ret=none args=v,v  # in-area
  stub 020129ac force ret=u32 args=v  # main
  stub 021a0cf4 force ret=u32 args=v,v,v  # ov21
  stub 020120f8 force ret=ptr:work:0x200:1 args=v  # gamesys -> player record (fieldmap reads +4..+0xF)
  stub 02012ae4 force ret=u32 args=v  # main
  stub 02012b28 force ret=pick:0/1/2 args=v
  stub 02013c20 force ret=pick:0/1 args=v
  stub 02012b2c force ret=none args=v,v  # main
  stub 02012944 force ret=u32 args=v  # main
  stub 02008550 force ret=u32 args=v  # main
  stub 021a2e50 force ret=u32 args=v,v,v,v,v  # ov21
  stub 0218d158 force ret=none args=v,v  # ov21
  stub 02189758 ret=none args=v  # in-area
  stub 021be2a8 force ret=u32 args=v,v,v  # ov21
  stub 02188c9c ret=u16 args=v  # in-area
  stub 02189154 force ret=none args=v,v  # fieldmap
  stub 021a0104 force ret=u32 args=v  # ov21
  stub 021a02c4 force ret=u32 args=v  # ov21
  stub 02013eec force ret=u32 args=v  # main
  stub 02013f2c force ret=u32 args=v  # main
  stub 021a0338 force ret=u32 args=v,v,v,v  # ov21
  stub 020140dc force ret=u32 args=v  # main
  stub 0203f4fc force ret=u32 args=  # RTC: seconds of the day (no arguments)
  stub 0202e83c force ret=u32 args=v  # main
  stub 021a03d0 force ret=u32 args=v,v,v,v,v  # ov21
  stub 02012a30 force ret=u32 args=v  # main
  stub 021a19e8 force ret=u32 args=v,v,v,v,v,v,v  # ov21
  stub 02012994 force ret=pick:0/0xF/0x10 args=v
  stub 021a1b74 force ret=none args=v,v,v  # ov21
  stub 021be1dc force ret=none args=v  # ov21
  stub 02189204 ret=none args=v  # in-area
  stub 021abd50 force ret=u32 args=v,v,v  # ov21
  stub 021abf68 force ret=none args=v,v  # ov21
  stub 021d33f8 force ret=u32 args=v  # ov21
  stub 021ac1ec force ret=none args=v,v  # ov21
  stub 021abe30 force ret=none args=v,v,v  # ov21
  stub 0217f8c0 force ret=u32 args=v  # ov18
  stub 02014110 force ret=none args=out:0x1C,v,v,v,v,v,v,v  # LOCATION init (loc, zone, ..)
  stub 0218cee4 force ret=none args=v,v,v  # ov21
  stub 02013c9c force ret=pick:0/1/0xFFFFFFFF args=v
  stub 021bb2f8 force ret=none args=v,v,v,v  # ov21
  stub 02189acc ret=u32 args=v  # in-area
  stub 021895e8 ret=none args=v,v,v  # in-area
  stub 021bb1f4 force ret=u32 args=v,v,v,v  # ov21
  stub 021d33a8 force ret=u32 args=v,v,v,v  # ov21
  stub 0218dc6c force ret=u32 args=v  # ov21
  stub 02012974 force ret=u32 args=v  # main
  stub 02189b30 ret=none args=v,v,v,v  # in-area
  stub 0218b8ac force ret=none args=v,v,v,v  # ov21
  stub 021c6534 force ret=none args=v,v  # ov21
  stub 02189614 ret=none args=v,v  # in-area
  stub 0216447c force ret=none args=  # ov10
  stub 021901ec force ret=u32 args=v,v  # ov21
  stub 021bc388 force ret=u32 args=v,v,v  # ov21
  stub 02012b38 force ret=u32 args=v  # main
  stub 02014470 force ret=u32 args=v  # main
  stub 021c7f70 force ret=u32 args=v,v  # ov21
  stub 02013bbc force ret=u32 args=v  # main
  stub 0218e018 force ret=u32 args=v,v,v,v,v  # ov21
  stub 021ab5f4 force ret=u32 args=v,v,v  # ov21
  stub 021ab7e4 force ret=u32 args=v  # ov21
  stub 02ee0100 force ret=none args=v,v,v  # fieldmap MAP CTRL +4 setup(fieldmap, position, x)
  stub 02014460 force ret=pick:0/1/2 args=v
  stub 02189b04 ret=pick:0/1 args=v  # in-area
  stub 02005c64 force ret=pick:0/1 args=
  stub 02014458 force ret=pick:0/1 args=v
  stub 02158ac8 force ret=none args=v  # ov10
  stub 02014454 force ret=none args=v,v  # main
  stub 0201445c force ret=none args=v,v  # main
  stub 0218ca78 force ret=none args=v  # ov21
  stub 0201209c force ret=none args=v,v,v  # main
  stub 020129dc force ret=none args=v,v  # main
  stub 02188cf0 ret=pick:0/1/2/3 args=v  # in-area
  stub 02158ad4 force ret=none args=v  # ov10
  stub 0216cd64 force ret=none args=v  # ov10
  stub 0219c624 force ret=none args=v  # ov21
  stub 0219c630 force ret=pick:0/1 args=v seq
  stub 021a1b50 force ret=pick:0/0/1 args=v seq
  stub 021a1af4 force ret=none args=v,v  # ov21
  stub 02189004 ret=none args=v  # in-area
  stub 02189084 force ret=none args=v,v,v,v  # fieldmap
  stub 021890ac force ret=u32 args=v  # fieldmap
  stub 0204a600 force ret=none args=  # (no arguments)
  stub 021903e0 force ret=none args=v  # ov21
  stub 021abfdc force ret=none args=v  # ov21
  stub 021bb948 force ret=none args=v  # fieldmap_func (other F2a area)
  stub 02012104 force ret=pick:0/1 args=v
  stub 0201447c force ret=none args=v,v  # main
  stub 02012e40 force ret=pick:0/0/1 args=v
  stub 021888c8 ret=none args=v  # in-area
  stub 02035648 force ret=none args=  # (reads a global, tail call; no arguments)
  stub 021892cc ret=u32 args=v  # in-area
  stub 020120d8 force ret=pick:0/1 args=v
  stub 02168eec force ret=none args=v  # ov10
  stub 021bc484 force ret=none args=v  # ov21
  stub 021c8000 force ret=none args=v  # ov21
  stub 021a2eb8 force ret=none args=v  # ov21
  stub 021737a4 force ret=none args=v  # ov11
  stub 021a0d68 force ret=none args=v  # ov21
  stub 021be258 force ret=none args=v  # ov21
  stub 0217f914 force ret=none args=p,v  # ov18
  stub 02030d28 force ret=none args=v  # task system run (sys)
  stub 021c6d90 force ret=none args=v  # ov21
  stub 02012ed0 force ret=ptr:work:0x220:1 args=v
  stub 021c8660 force ret=u32 args=v  # ov21
  stub 0217ae20 force ret=none args=v,v,v  # ov18
  stub 02ee0110 force ret=none args=v,v  # fieldmap MAP CTRL +8 frame(fieldmap, position)
  stub 020129f0 force ret=none args=v  # main
  stub 0218cc34 force ret=none args=v  # ov21
  stub 021a0e58 force ret=none args=v  # ov21
  stub 021bc4f0 force ret=none args=v  # ov21
  stub 021cf480 force ret=none args=v  # ov21
  stub 021c1c04 force ret=none args=v,v,v,v  # ov21
  stub 021c1ad0 force ret=none args=v,v,v,v  # ov21
  stub 021bb90c force ret=none args=v  # fieldmap_func (other F2a area)
  stub 020263b0 force ret=none args=v,v  # main
  stub 021bc424 force ret=none args=v  # ov21
  stub 021be218 force ret=none args=v  # ov21
  stub 021be2e0 force ret=none args=v  # ov21
  stub 021a9f28 force ret=none args=v  # ov21
  stub 021a0d44 force ret=u32 args=v  # ov21
  stub 020129b8 force ret=none args=v,v  # main
  stub 021abde8 force ret=none args=v  # ov21
  stub 0217f8ec force ret=none args=v  # ov18
  stub 021a1aac force ret=none args=v  # ov21
  stub 021a0448 force ret=none args=v  # ov21
  stub 021a02f4 force ret=none args=v  # ov21
  stub 020307b0 force ret=none args=v  # Heap_Free(block)
  stub 021d33e0 force ret=none args=v  # ov21
  stub 021bb240 force ret=none args=v  # ov21
  stub 021ab81c force ret=none args=v  # ov21
  stub 021ab624 force ret=none args=v  # ov21
  stub 0218e0e4 force ret=none args=v  # ov21
  stub 021895dc ret=none args=v  # in-area
  stub 021892b4 ret=none args=v  # in-area
  stub 021a2e90 force ret=none args=v  # ov21
  stub 0218d05c force ret=none args=v  # ov21
  stub 02190728 force ret=none args=v,p,v,v  # ov21
  stub 021902bc force ret=none args=v  # ov21
  stub 021c7f9c force ret=none args=v  # ov21
  stub 02ee0120 force ret=none args=v  # fieldmap MAP CTRL +0xC teardown(fieldmap)
  stub 02030da4 force ret=none args=  # empty function (bx lr)
  stub 021c6d5c force ret=none args=v  # ov21
  stub 02030eac force ret=none args=v  # task delete (task)
  stub 0218ca4c force ret=none args=v  # ov21
  stub 021890c8 force ret=u32 args=v,v,v,v  # fieldmap
  stub 0204a5ac force ret=none args=  # (no arguments)
  stub 020450c8 force ret=none args=  # frees that global block
  stub 02188e38 force ret=none args=p,v,v,v  # fieldmap
  stub 02034a5c force ret=none args=v  # overlay unload (id; -1 = none)
  stub 02013fa8 force ret=pick:0/2/0xFFFF args=v
  stub 021ab650 force ret=none args=v  # ov21
  stub 021ab868 force ret=u32 args=v  # ov21
  stub 021ab840 force ret=none args=v,v,v,v  # ov21
  stub 021ab880 force ret=u32 args=v  # ov21
  stub 021ab884 force ret=u32 args=v  # ov21
  stub 021ab888 force ret=u32 args=  # ov21
  stub 021ab634 force ret=u32 args=v,v,v,v  # ov21
  stub 02013c80 force ret=pick:0/1 args=v
  stub 02012964 force ret=u32 args=v  # main
  stub 0216ccdc force ret=none args=v,v,v,v,v,v  # ov10
  stub 0219c554 force ret=none args=v,v,v  # ov21
  stub 0219c680 force ret=none args=v  # ov21
  stub 0217c0a8 force ret=u32 args=v  # ov18
  stub 021a31a4 force ret=u32 args=v,v  # ov21
  stub 0219c658 force ret=none args=v,v  # ov21
  stub 0219d448 force ret=none args=v,v  # ov21
  stub 0218ed50 force ret=u32 args=v  # ov21
  stub 0216cdb8 force ret=none args=v,v  # ov10
  stub 0216d2f8 force ret=none args=v  # ov10
  stub 02189ae8 ret=u32 args=v  # in-area
  stub 0216d974 force ret=u32 args=v,v  # ov10
  stub 02013ba8 force ret=pick:0/1/2 args=v
  stub 02013bd0 force ret=pick:0/3/0xFFFF args=v
  stub 0218f14c force ret=u32 args=v,v,v  # ov21
  stub 0218f134 force ret=u32 args=v,p,p,v  # ov21
  stub 02013b50 force ret=pick:0/1/2/3/4/5/6/7 args=v  # zone kind (index of the MAP SETUP table 0x021D3700)
  stub 02159bf4 force ret=u32 args=v  # map_matrix
  stub 02159bf8 force ret=u32 args=v  # map_matrix
  stub 02159bfc force ret=u32 args=v  # map_matrix
  stub 02159c20 force ret=u32 args=v  # map_matrix
  stub 020140bc force ret=u32 args=v  # main
  stub 020140c0 force ret=u32 args=v  # main
  stub 020140cc force ret=u32 args=v  # main
  stub 02013dd0 force ret=pick:0/1 args=v
  stub 0201444c force ret=pick:0/1 args=v
  stub 020127d0 force ret=u32 args=v,v  # main
  stub 0218dc74 force ret=none args=v,v,v  # ov21
  stub 0218dc70 force ret=u32 args=v  # ov21
  stub 02012844 force ret=u32 args=v  # main
  stub 021939c0 force ret=none args=v,v,v,v  # ov21
  stub 02193abc force ret=none args=v,v,v  # ov21
  stub 02193afc force ret=u32 args=v,v  # ov21
  stub 020504d8 force ret=none args=v  # (render object)
  stub 021bb36c force ret=none args=v  # ov21
  stub 021ab660 force ret=none args=v,v  # ov21
  stub 021a030c force ret=none args=v,v  # ov21
  stub 021a0140 force ret=u32 args=v  # ov21
  stub 021a0458 force ret=none args=v,v  # ov21
  stub 0218cb34 force ret=pick:0/1 args=v
  stub 0203d12c force ret=none args=v  # sound / effect request (id)
  stub 02047ce4 force ret=none args=  # clears two words of a global
  stub 0203630c force ret=u32 args=  # key input: trigger mask (no arguments)
  stub 0218e104 force ret=none args=v,v  # ov21
  stub 02189a80 ret=u32 args=v  # in-area
  stub 0218932c ret=pick:0/1 args=v  # in-area
  stub 02189370 ret=pick:0/1 args=v  # in-area
  stub 021893c0 ret=none args=v  # in-area
  stub 02162458 force ret=pick:0/1 args=v
  stub 02162438 force ret=none args=v,v,v  # ov10
  stub 0216d2ac force ret=none args=v  # ov10
  stub 0216cd50 force ret=none args=v  # ov10
  stub 0218ed3c force ret=ptr:work:0x280:1 args=v
  stub 020127a4 force ret=u32 args=v  # main
  stub 02012aa0 force ret=ptr:work:0x260:1 args=v
  stub 020141e0 force ret=none args=v,outm:12:3/0/0/0/0/0/0/0/3/0/0/0
  stub 020141f0 force ret=none args=v,v  # main
  stub 02159c48 force ret=pick:0/1 args=v,v,v  # map_matrix (in bounds)
  stub 02159bd4 force ret=pick:0/1/0x3f/0xFFFF args=v,v,v  # map_matrix (zone id at a position)
  stub 021aa8ec force ret=none args=v  # ov21
  stub 0216df1c force ret=none args=v  # ov10
  stub 021623dc force ret=none args=v  # ov10
  stub 0216242c force ret=none args=v,v  # ov10
  stub 021643ac force ret=none args=v,v  # ov10
  stub 02189518 ret=none args=v,v  # in-area
  stub 0218958c ret=none args=v,v  # in-area
  stub 02189560 ret=none args=v,v  # in-area
  stub 021895d0 ret=none args=v,v  # in-area
  stub 021895bc ret=none args=v,v  # in-area
  stub 021bc550 force ret=none args=v,v  # ov21
  stub 0202735c force ret=none args=v,v  # main
  stub 0215a014 force ret=none args=v,v  # map_matrix
  stub 02158ae0 force ret=none args=v,v  # ov10
  stub 021894e0 ret=none args=v,v,v,v  # in-area
  stub 0215874c force ret=u32 args=v  # ov10
  stub 02012134 force ret=u32 args=v  # main
  stub 0202b544 force ret=u32 args=v,v  # main
  stub 02029cf0 force ret=u32 args=v,v,v  # main
  stub 02188c68 ret=u32 args=v  # in-area
  stub 021a0358 force ret=none args=v  # ov21
  stub 02188c6c ret=u32 args=v  # in-area
  stub 0202c83c force ret=none args=v,v  # main
  stub 021a1c34 force ret=u32 args=v,v  # ov21
  stub 02012ac4 force ret=u32 args=v,v  # main
  stub 02162744 force ret=pick:0/1/5 args=v
  stub 0201296c force ret=u32 args=v  # main
  stub 02162740 force ret=u32 args=v  # ov10
  stub 0216ceac force ret=u32 args=v,v,v,v,v  # ov10

# 02188BD8(fm): +0xC0 = 2; +0xC4 = 4 (jump to the teardown step)
fn 02188bd8 ret=none inputs=20
  arg work 0

# 02188BE8(fm): +0xC0 == 1
fn 02188be8 ret=bool inputs=60
  arg work 0
  fix work 0xC0 u32 alt oneof 0 1 2 || any

# 02188BF8(fm): r = 021A3098(+0x8C); r == 1: 021A2F48(+0x8C, 1), done; r == 0: 020061E4(0x55F), 021A2F48(+0x8C, 2),
# 02008DF0(02012F2C(gd), 3) (thunk 02008DF0 -> 02008DAC(x, 3, 1)), done; done: 021A2F54(+0x8C); returns done
fn 02188bf8 ret=u32 inputs=200
  arg work 0
  stub 021a3098 force ret=pick:0/1/2 args=v
  stub 021a2f48 force ret=none args=v  # ov21
  stub 021a2f54 force ret=none args=v  # ov21
  stub 020061e4 force ret=none args=  # main
  stub 02012f2c force ret=u32 args=v  # main
  stub 02008dac force ret=u32 args=v,v,v  # main

# 02188C54(fm): +0x28
fn 02188c54 ret=u32 inputs=10
  arg work 0

# 02188C58(fm): +0x10
fn 02188c58 ret=u32 inputs=10
  arg work 0

# 02188C5C(fm): +0x3C
fn 02188c5c ret=u32 inputs=10
  arg work 0

# 02188C60(fm): +0x14
fn 02188c60 ret=u32 inputs=10
  arg work 0

# 02188C64(fm): +0x18
fn 02188c64 ret=u32 inputs=10
  arg work 0

# 02188C68(fm): +0x1C
fn 02188c68 ret=u32 inputs=10
  arg work 0

# 02188C6C(fm): +0x20
fn 02188c6c ret=u32 inputs=10
  arg work 0

# 02188C70(fm): tail call 0202C794(gamesys (+4)) (`ldr r0,[r0,4]; ldr r3,=0202C795; bx r3`)
fn 02188c70 ret=u32 inputs=20
  arg work 0
  stub 0202c794 force ret=u32 args=v  # (gamesys)

# 02188C7C(fm): +0x38
fn 02188c7c ret=u32 inputs=10
  arg work 0

# 02188C80(fm): +0x4
fn 02188c80 ret=u32 inputs=10
  arg work 0

# 02188C84(fm): u16 +0 (heap id)
fn 02188c84 ret=u16 inputs=10
  arg work 0

# 02188C88(fm): +0xB4
fn 02188c88 ret=u32 inputs=10
  arg work 0

# 02188C90(fm): +0xB8
fn 02188c90 ret=u32 inputs=10
  arg work 0

# 02188C98(fm): +0x48
fn 02188c98 ret=u32 inputs=10
  arg work 0

# 02188C9C(fm): u16 +0xD8 (current zone id)
fn 02188c9c ret=u16 inputs=10
  arg work 0

# 02188CA4(fm): +0x10C
fn 02188ca4 ret=u32 inputs=10
  arg work 0

# 02188CAC(fm, v): +0x10C = v
fn 02188cac ret=none inputs=10
  arg work 0
  arg u32 any

# 02188CB4(fm): +0x8C (player)
fn 02188cb4 ret=u32 inputs=10
  arg work 0

# 02188CBC(fm): +0x8C != 0
fn 02188cbc ret=bool inputs=40
  arg work 0
  fix work 0x8C u32 alt const 0 || any

# 02188CCC(fm): +0x24
fn 02188ccc ret=u32 inputs=10
  arg work 0

# 02188CD0(fm): +0x94
fn 02188cd0 ret=u32 inputs=10
  arg work 0

# 02188CD8(fm): +0x130
fn 02188cd8 ret=u32 inputs=10
  arg work 0

# 02188CE0(fm): +0x98
fn 02188ce0 ret=u32 inputs=10
  arg work 0

# 02188CE8(fm): +0x90
fn 02188ce8 ret=u32 inputs=10
  arg work 0

# 02188CF0(fm): MAP CTRL kind (u32 *(+0x108))
fn 02188cf0 ret=u32 inputs=20
  arg work 0
  fix work 0x28 u32 alt const 0 || any
  fix work 0x30 u32 alt const 0 || any
  fix work 0x38 u32 alt const 0 || any
  fix work 0x98 u32 alt const 0 || any
  fix work 0xC0 u32 alt oneof 0 1 2 || any
  fix work 0xC4 u32 range 0 5
  fix work 0xC8 u8 alt range 0 7 || oneof 0 1
  fix work 0xC9 u8 range 0 1
  fix work 0xD0 u16 alt oneof 0x3f 0x8c 0x8d 0x8e 0x8f 0 || range 0 0x1ff || any
  fix work 0xD8 u16 alt oneof 0x3f 0x8c 0xFFFF || range 0 0x1ff
  fix work 0xF0 u32 alt const 0 || addrof work 0x480
  fix work 0x108 u32 addrof work 0x400
  fix work 0x118 s32 alt oneof 0xFFFFFFFF 0 1 2 || any
  fix work 0x140 u32 alt const 0 || any
  fix work 0x150 u32 alt const 0 || any
  fix work 0x400 u32 alt range 0 2 || range 0 8
  fix work 0x404 u32 const 0x02EE0101
  fix work 0x408 u32 const 0x02EE0111
  fix work 0x40C u32 const 0x02EE0121
  fix work 0x2C u32 alt const 0 || any

# 02188CFC(fm): k = ROM byte 0x021D35A8 [MAP CTRL kind]; k == 2 ? 021A6DC0(+0x10C) : k
fn 02188cfc ret=u32 inputs=60
  arg work 0
  fix work 0x28 u32 alt const 0 || any
  fix work 0x30 u32 alt const 0 || any
  fix work 0x38 u32 alt const 0 || any
  fix work 0x98 u32 alt const 0 || any
  fix work 0xC0 u32 alt oneof 0 1 2 || any
  fix work 0xC4 u32 range 0 5
  fix work 0xC8 u8 alt range 0 7 || oneof 0 1
  fix work 0xC9 u8 range 0 1
  fix work 0xD0 u16 alt oneof 0x3f 0x8c 0x8d 0x8e 0x8f 0 || range 0 0x1ff || any
  fix work 0xD8 u16 alt oneof 0x3f 0x8c 0xFFFF || range 0 0x1ff
  fix work 0xF0 u32 alt const 0 || addrof work 0x480
  fix work 0x108 u32 addrof work 0x400
  fix work 0x118 s32 alt oneof 0xFFFFFFFF 0 1 2 || any
  fix work 0x140 u32 alt const 0 || any
  fix work 0x150 u32 alt const 0 || any
  fix work 0x400 u32 alt range 0 2 || range 0 8
  fix work 0x404 u32 const 0x02EE0101
  fix work 0x408 u32 const 0x02EE0111
  fix work 0x40C u32 const 0x02EE0121
  fix work 0x2C u32 alt const 0 || any
  stub 021a6dc0 force ret=u32 args=v  # ov21

# 02188D20(fm): +0x30
fn 02188d20 ret=u32 inputs=10
  arg work 0

# 02188D24(fm): +0xBC
fn 02188d24 ret=u32 inputs=10
  arg work 0

# 02188D2C(fm): +0x34
fn 02188d2c ret=u32 inputs=10
  arg work 0

# 02188D30(fm): +0x110
fn 02188d30 ret=u32 inputs=10
  arg work 0

# 02188D38(fm): +0x134
fn 02188d38 ret=u32 inputs=10
  arg work 0

# 02188D40(fm, p): +0xF0 = p
fn 02188d40 ret=none inputs=10
  arg work 0
  arg u32 any

# 02188D48(fm): +0x2C
fn 02188d48 ret=u32 inputs=10
  arg work 0

# 02188D4C(fm, v): +0x2C = v
fn 02188d4c ret=none inputs=10
  arg work 0
  arg u32 any

# 02188D50(fm): +0x128
fn 02188d50 ret=u32 inputs=10
  arg work 0

# 02188D58(fm): +0xC
fn 02188d58 ret=u32 inputs=10
  arg work 0

# 02188E30(fm): +0x14C
fn 02188e30 ret=u32 inputs=10
  arg work 0

# 02189004(fm) (per-frame map work): 020504D8(+0xB0), (+0xB4), (+0xB8); 021BB36C(+0x3C); 021AB660(+0x40, fm+0xF4);
# 021A030C(+0x1C, heap); 021A1AF4(+0x20, 0x92); 021A0140(+0x18); 021A0458(+0x14, 0203F4FC()); EXMEMCNT &= 0xFF7F;
# r = 0218CB34(+0x48); EXMEMCNT |= 0x80; r != 0: 0203D12C(2)
fn 02189004 ret=none inputs=100
  arg work 0
  global 04000204 4
  stub 020504d8 force ret=none args=v  # (render object)
  stub 021bb36c force ret=none args=v  # ov21
  stub 021ab660 force ret=none args=v,v  # ov21
  stub 021a030c force ret=none args=v,v  # ov21
  stub 021a1af4 force ret=none args=v,v  # ov21
  stub 021a0140 force ret=u32 args=v  # ov21
  stub 0203f4fc force ret=u32 args=  # RTC: seconds of the day (no arguments)
  stub 021a0458 force ret=none args=v,v  # ov21
  stub 0218cb34 force ret=pick:0/1 args=v
  stub 0203d12c force ret=none args=v  # sound / effect request (id)

# 02189204(fm): gd = 020120F4(gamesys); +0x38 = m = 02012964(gd); 0216CCDC(m, heap, gd, fm, +0x48, +0x3C); 0219C554(+0x38, +0xB0, 0x20);
# 0219C680(+0x38); z = +0xD8; 02013CC8(z) == 0 && 02013CE0(z) == 0 ? 0217C0A8(02012138(gamesys)) : 02008550(02012944(gd));
# 0219C658(m, 021A31A4(.., 0)); 0219D448(+0x38, +0x130); 0216CDB8(+0x38, 0218ED50(+0x10)); 0216D2F8(+0x38);
# 0216D974(m, 02189AE8(+0xD0))
fn 02189204 ret=none inputs=200
  arg work 0
  fix work 0x28 u32 alt const 0 || any
  fix work 0x30 u32 alt const 0 || any
  fix work 0x38 u32 alt const 0 || any
  fix work 0x98 u32 alt const 0 || any
  fix work 0xC0 u32 alt oneof 0 1 2 || any
  fix work 0xC4 u32 range 0 5
  fix work 0xC8 u8 alt range 0 7 || oneof 0 1
  fix work 0xC9 u8 range 0 1
  fix work 0xD0 u16 alt oneof 0x3f 0x8c 0x8d 0x8e 0x8f 0 || range 0 0x1ff || any
  fix work 0xD8 u16 alt oneof 0x3f 0x8c 0xFFFF || range 0 0x1ff
  fix work 0xF0 u32 alt const 0 || addrof work 0x480
  fix work 0x108 u32 addrof work 0x400
  fix work 0x118 s32 alt oneof 0xFFFFFFFF 0 1 2 || any
  fix work 0x140 u32 alt const 0 || any
  fix work 0x150 u32 alt const 0 || any
  fix work 0x400 u32 alt range 0 2 || range 0 8
  fix work 0x404 u32 const 0x02EE0101
  fix work 0x408 u32 const 0x02EE0111
  fix work 0x40C u32 const 0x02EE0121
  fix work 0x2C u32 alt const 0 || any
  stub 020120f4 force ret=u32 args=v  # main
  stub 02012964 force ret=u32 args=v  # main
  stub 0216ccdc force ret=none args=v,v,v,v,v,v  # ov10
  stub 0219c554 force ret=none args=v,v,v  # ov21
  stub 0219c680 force ret=none args=v  # ov21
  stub 02013cc8 force ret=pick:0/1 args=v
  stub 02013ce0 force ret=pick:0/1 args=v
  stub 02012138 force ret=u32 args=v  # main
  stub 0217c0a8 force ret=u32 args=v  # ov18
  stub 021a31a4 force ret=u32 args=v,v  # ov21
  stub 0219c658 force ret=none args=v,v  # ov21
  stub 0219d448 force ret=none args=v,v  # ov21
  stub 0218ed50 force ret=u32 args=v  # ov21
  stub 0216cdb8 force ret=none args=v,v  # ov10
  stub 0216d2f8 force ret=none args=v  # ov10
  stub 02189ae8 ret=u32 args=v  # in-area
  stub 0216d974 force ret=u32 args=v,v  # ov10
  stub 02012944 force ret=u32 args=v  # main
  stub 02008550 force ret=u32 args=v  # main

# 021892B4(fm): 0216D2AC(+0x38); 0216CD50(+0x38); +0x38 = 0
fn 021892b4 ret=none inputs=20
  arg work 0
  stub 0216d2ac force ret=none args=v  # ov10
  stub 0216cd50 force ret=none args=v  # ov10

# 021892CC(fm) (zone change check): 0218932C(fm) == 1 && 02189370(fm) == 1: 021893C0(fm), return 1; else
# p = 02012844(gd); 02162458(p) == 0: EXMEMCNT &= 0xFF7F, 02162438(p, +0xD8, 02012984(gd)), EXMEMCNT |= 0x80; returns 0
fn 021892cc ret=u32 inputs=300
  arg work 0
  fix work 0x28 u32 alt const 0 || any
  fix work 0x30 u32 alt const 0 || any
  fix work 0x38 u32 alt const 0 || any
  fix work 0x98 u32 alt const 0 || any
  fix work 0xC0 u32 alt oneof 0 1 2 || any
  fix work 0xC4 u32 range 0 5
  fix work 0xC8 u8 alt range 0 7 || oneof 0 1
  fix work 0xC9 u8 range 0 1
  fix work 0xD0 u16 alt oneof 0x3f 0x8c 0x8d 0x8e 0x8f 0 || range 0 0x1ff || any
  fix work 0xD8 u16 alt oneof 0x3f 0x8c 0xFFFF || range 0 0x1ff
  fix work 0xF0 u32 alt const 0 || addrof work 0x480
  fix work 0x108 u32 addrof work 0x400
  fix work 0x118 s32 alt oneof 0xFFFFFFFF 0 1 2 || any
  fix work 0x140 u32 alt const 0 || any
  fix work 0x150 u32 alt const 0 || any
  fix work 0x400 u32 alt range 0 2 || range 0 8
  fix work 0x404 u32 const 0x02EE0101
  fix work 0x408 u32 const 0x02EE0111
  fix work 0x40C u32 const 0x02EE0121
  fix work 0x2C u32 alt const 0 || any
  global 04000204 4
  fix work 0x260 s32 alt range 0 3 || any
  fix work 0x268 s32 alt range 0 3 || any
  stub 0218932c ret=pick:0/1 args=v  # in-area
  stub 02189370 ret=pick:0/1 args=v  # in-area
  stub 021893c0 ret=none args=v  # in-area
  stub 02012844 force ret=u32 args=v  # main
  stub 02162458 force ret=pick:0/1 args=v
  stub 02012984 force ret=u32 args=v  # main
  stub 02162438 force ret=none args=v,v,v  # ov10
  stub 020120f4 force ret=u32 args=v  # main
  stub 020127a4 force ret=u32 args=v  # main
  stub 02012aa0 force ret=ptr:work:0x260:1 args=v
  stub 020141e0 force ret=none args=v,outm:12:3/0/0/0/0/0/0/0/3/0/0/0
  stub 020141f0 force ret=none args=v,v  # main
  stub 02012974 force ret=u32 args=v  # main
  stub 02159c48 force ret=pick:0/1 args=v,v,v  # map_matrix (in bounds)
  stub 02159bd4 force ret=pick:0/1/0x3f/0xFFFF args=v,v,v  # map_matrix (zone id at a position)
  stub 0203d12c force ret=none args=v  # sound / effect request (id)
  stub 021aa8ec force ret=none args=v  # ov21
  stub 0216df1c force ret=none args=v  # ov10
  stub 021623dc force ret=none args=v  # ov10
  stub 0216242c force ret=none args=v,v  # ov10
  stub 021643ac force ret=none args=v,v  # ov10
  stub 02189518 ret=none args=v,v  # in-area
  stub 0218958c ret=none args=v,v  # in-area
  stub 02189560 ret=none args=v,v  # in-area
  stub 021895d0 ret=none args=v,v  # in-area
  stub 021895bc ret=none args=v,v  # in-area
  stub 02189684 ret=none args=v,v  # in-area
  stub 020120d8 force ret=pick:0/1 args=v
  stub 021bc550 force ret=none args=v,v  # ov21
  stub 0202735c force ret=none args=v,v  # main
  stub 0215a014 force ret=none args=v,v  # map_matrix
  stub 02158ae0 force ret=none args=v,v  # ov10
  stub 021894e0 ret=none args=v,v,v,v  # in-area
  stub 02012138 force ret=u32 args=v  # main
  stub 0215874c force ret=u32 args=v  # ov10
  stub 021895e8 ret=none args=v,v,v  # in-area
  stub 02012134 force ret=u32 args=v  # main
  stub 02012a30 force ret=u32 args=v  # main
  stub 02012b28 force ret=pick:0/1/2 args=v
  stub 0202b544 force ret=u32 args=v,v  # main
  stub 02029cf0 force ret=u32 args=v,v,v  # main
  stub 02188c68 ret=u32 args=v  # in-area
  stub 021a0358 force ret=none args=v  # ov21
  stub 02013eec force ret=u32 args=v  # main
  stub 02013f2c force ret=u32 args=v  # main
  stub 021a0338 force ret=u32 args=v,v,v,v  # ov21
  stub 02188c6c ret=u32 args=v  # in-area
  stub 0202c83c force ret=none args=v,v  # main
  stub 02012994 force ret=pick:0/0xF/0x10 args=v
  stub 021a1c34 force ret=u32 args=v,v  # ov21
  stub 02012ac4 force ret=u32 args=v,v  # main
  stub 02013fa8 force ret=pick:0/2/0xFFFF args=v
  stub 021ab650 force ret=none args=v  # ov21
  stub 021ab868 force ret=u32 args=v  # ov21
  stub 021ab840 force ret=none args=v,v,v,v  # ov21
  stub 021ab880 force ret=u32 args=v  # ov21
  stub 021ab884 force ret=u32 args=v  # ov21
  stub 021ab888 force ret=u32 args=  # ov21
  stub 021ab634 force ret=u32 args=v,v,v,v  # ov21
  stub 02013c80 force ret=pick:0/1 args=v
  stub 02162744 force ret=pick:0/1/5 args=v
  stub 0201296c force ret=u32 args=v  # main
  stub 02162740 force ret=u32 args=v  # ov10
  stub 0216ceac force ret=u32 args=v,v,v,v,v  # ov10
  stub 02013bd0 force ret=pick:0/3/0xFFFF args=v
  stub 0218f14c force ret=u32 args=v,v,v  # ov21
  stub 0218f134 force ret=u32 args=v,p,p,v  # ov21

# 0218932C(fm): p = 02012AA0(020127A4(020120F4(gamesys))) (player position record); 020141E0(fm+0xD4, &pos);
# p[0] == pos.x && p[2] == pos.z: 0; else 020141F0(fm+0xD4, p), 1
fn 0218932c ret=u32 inputs=200
  arg work 0
  fix work 0x260 s32 alt range 0 3 || any
  fix work 0x268 s32 alt range 0 3 || any
  stub 020120f4 force ret=u32 args=v  # main
  stub 020127a4 force ret=u32 args=v  # main
  stub 02012aa0 force ret=ptr:work:0x260:1 args=v
  stub 020141e0 force ret=none args=v,outm:12:3/0/0/0/0/0/0/0/3/0/0/0
  stub 020141f0 force ret=none args=v,v  # main

# 02189370(fm): m = 02012974(gd) (map matrix); 020141E0(fm+0xD4, &pos); 02159C48(m, x, z) == 1 && (z2 = 02159BD4(m, x, z)) != 0xFFFF
# && z2 != (s16) +0xD8: 1; else 0
fn 02189370 ret=u32 inputs=200
  arg work 0
  fix work 0x28 u32 alt const 0 || any
  fix work 0x30 u32 alt const 0 || any
  fix work 0x38 u32 alt const 0 || any
  fix work 0x98 u32 alt const 0 || any
  fix work 0xC0 u32 alt oneof 0 1 2 || any
  fix work 0xC4 u32 range 0 5
  fix work 0xC8 u8 alt range 0 7 || oneof 0 1
  fix work 0xC9 u8 range 0 1
  fix work 0xD0 u16 alt oneof 0x3f 0x8c 0x8d 0x8e 0x8f 0 || range 0 0x1ff || any
  fix work 0xD8 u16 alt oneof 0x3f 0x8c 0xFFFF || range 0 0x1ff
  fix work 0xF0 u32 alt const 0 || addrof work 0x480
  fix work 0x108 u32 addrof work 0x400
  fix work 0x118 s32 alt oneof 0xFFFFFFFF 0 1 2 || any
  fix work 0x140 u32 alt const 0 || any
  fix work 0x150 u32 alt const 0 || any
  fix work 0x400 u32 alt range 0 2 || range 0 8
  fix work 0x404 u32 const 0x02EE0101
  fix work 0x408 u32 const 0x02EE0111
  fix work 0x40C u32 const 0x02EE0121
  fix work 0x2C u32 alt const 0 || any
  stub 02012974 force ret=u32 args=v  # main
  stub 020141e0 force ret=none args=v,out:12
  stub 02159c48 force ret=pick:0/1 args=v,v,v  # map_matrix (in bounds)
  stub 02159bd4 force ret=pick:0/1/0x3f/0xFFFF args=v,v,v  # map_matrix (zone id at a position)

# 021893C0(fm) (zone change): gd = 020120F4(gamesys); p = 02012844(gd); m = +0x38; mtx = 02012974(gd); 0203D12C(2);
# 021AA8EC(+0x90); 020141E0(fm+0xD4, &pos); z = 02159BD4(mtx, x, z); EXMEMCNT &= 0xFF7F; 0216DF1C(m); 021623DC(p);
# 0216242C(p, z, 02012984(gd)); EXMEMCNT |= 0x80; 021643AC(gd, fm); 02189518(fm, z); 0218958C(fm, z); 02189560(fm, z);
# 021895D0(fm, z); 021895BC(gd, z); 02189684(fm, z); +0x30 && !020120D8(gamesys): 021BC550(+0x30, z); +0xD8 = z;
# 0202735C(z, gd); 0215A014(gd, z); 02158AE0(gamesys, 4); EXMEMCNT &= 0xFF7F; 021894E0(gd, m, p, z); EXMEMCNT |= 0x80;
# 0215874C(02012138(gamesys))
fn 021893c0 ret=none inputs=300
  arg work 0
  fix work 0x28 u32 alt const 0 || any
  fix work 0x30 u32 alt const 0 || any
  fix work 0x38 u32 alt const 0 || any
  fix work 0x98 u32 alt const 0 || any
  fix work 0xC0 u32 alt oneof 0 1 2 || any
  fix work 0xC4 u32 range 0 5
  fix work 0xC8 u8 alt range 0 7 || oneof 0 1
  fix work 0xC9 u8 range 0 1
  fix work 0xD0 u16 alt oneof 0x3f 0x8c 0x8d 0x8e 0x8f 0 || range 0 0x1ff || any
  fix work 0xD8 u16 alt oneof 0x3f 0x8c 0xFFFF || range 0 0x1ff
  fix work 0xF0 u32 alt const 0 || addrof work 0x480
  fix work 0x108 u32 addrof work 0x400
  fix work 0x118 s32 alt oneof 0xFFFFFFFF 0 1 2 || any
  fix work 0x140 u32 alt const 0 || any
  fix work 0x150 u32 alt const 0 || any
  fix work 0x400 u32 alt range 0 2 || range 0 8
  fix work 0x404 u32 const 0x02EE0101
  fix work 0x408 u32 const 0x02EE0111
  fix work 0x40C u32 const 0x02EE0121
  fix work 0x2C u32 alt const 0 || any
  global 04000204 4
  stub 020120f4 force ret=u32 args=v  # main
  stub 02012844 force ret=u32 args=v  # main
  stub 02012974 force ret=u32 args=v  # main
  stub 0203d12c force ret=none args=v  # sound / effect request (id)
  stub 021aa8ec force ret=none args=v  # ov21
  stub 020141e0 force ret=none args=v,out:12
  stub 02159bd4 force ret=pick:0/1/0x3f/0x8c args=v,v,v  # map_matrix (zone id at a position)
  stub 0216df1c force ret=none args=v  # ov10
  stub 021623dc force ret=none args=v  # ov10
  stub 02012984 force ret=u32 args=v  # main
  stub 0216242c force ret=none args=v,v  # ov10
  stub 021643ac force ret=none args=v,v  # ov10
  stub 02189518 ret=none args=v,v  # in-area
  stub 0218958c ret=none args=v,v  # in-area
  stub 02189560 ret=none args=v,v  # in-area
  stub 021895d0 ret=none args=v,v  # in-area
  stub 021895bc ret=none args=v,v  # in-area
  stub 02189684 ret=none args=v,v  # in-area
  stub 020120d8 force ret=pick:0/1 args=v
  stub 021bc550 force ret=none args=v,v  # ov21
  stub 0202735c force ret=none args=v,v  # main
  stub 0215a014 force ret=none args=v,v  # map_matrix
  stub 02158ae0 force ret=none args=v,v  # ov10
  stub 021894e0 ret=none args=v,v,v,v  # in-area
  stub 02012138 force ret=u32 args=v  # main
  stub 0215874c force ret=u32 args=v  # ov10
  stub 021895e8 ret=none args=v,v,v  # in-area
  stub 02012134 force ret=u32 args=v  # main
  stub 02012a30 force ret=u32 args=v  # main
  stub 020127a4 force ret=u32 args=v  # main
  stub 02012b28 force ret=pick:0/1/2 args=v
  stub 0202b544 force ret=u32 args=v,v  # main
  stub 02029cf0 force ret=u32 args=v,v,v  # main
  stub 02188c68 ret=u32 args=v  # in-area
  stub 021a0358 force ret=none args=v  # ov21
  stub 02013eec force ret=u32 args=v  # main
  stub 02013f2c force ret=u32 args=v  # main
  stub 021a0338 force ret=u32 args=v,v,v,v  # ov21
  stub 02188c6c ret=u32 args=v  # in-area
  stub 0202c83c force ret=none args=v,v  # main
  stub 02012994 force ret=pick:0/0xF/0x10 args=v
  stub 021a1c34 force ret=u32 args=v,v  # ov21
  stub 02012ac4 force ret=u32 args=v,v  # main
  stub 02013fa8 force ret=pick:0/2/0xFFFF args=v
  stub 021ab650 force ret=none args=v  # ov21
  stub 021ab868 force ret=u32 args=v  # ov21
  stub 021ab840 force ret=none args=v,v,v,v  # ov21
  stub 021ab880 force ret=u32 args=v  # ov21
  stub 021ab884 force ret=u32 args=v  # ov21
  stub 021ab888 force ret=u32 args=  # ov21
  stub 021ab634 force ret=u32 args=v,v,v,v  # ov21
  stub 02013c80 force ret=pick:0/1 args=v
  stub 02162744 force ret=pick:0/1/5 args=v
  stub 0201296c force ret=u32 args=v  # main
  stub 02162740 force ret=u32 args=v  # ov10
  stub 0216ceac force ret=u32 args=v,v,v,v,v  # ov10
  stub 02013bd0 force ret=pick:0/3/0xFFFF args=v
  stub 0218f14c force ret=u32 args=v,v,v  # ov21
  stub 0218f134 force ret=u32 args=v,p,p,v  # ov21
  stub 02013dd0 force ret=pick:0/1 args=v

# 021894E0(gd, m, p, z): e = 02162744(p); e != 0: 0216CEAC(m, 02162740(p), z, e, 0201296C(gd))
fn 021894e0 ret=none inputs=100
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  stub 02162744 force ret=pick:0/1/5 args=v
  stub 0201296c force ret=u32 args=v  # main
  stub 02162740 force ret=u32 args=v  # ov10
  stub 0216ceac force ret=u32 args=v,v,v,v,v  # ov10

# 02189518(fm, z): gd = 020120F4(gamesys); x = 02012134(gamesys); w = 02012A30(gd); k = 02012B28(020127A4(gd));
# 0202B544(x, (u16) z); k != 2: 02029CF0(w, gd, (u16) z)
fn 02189518 ret=none inputs=100
  arg work 0
  arg u32 any
  stub 020120f4 force ret=u32 args=v  # main
  stub 02012134 force ret=u32 args=v  # main
  stub 02012a30 force ret=u32 args=v  # main
  stub 020127a4 force ret=u32 args=v  # main
  stub 02012b28 force ret=pick:0/1/2 args=v
  stub 0202b544 force ret=u32 args=v,v  # main
  stub 02029cf0 force ret=u32 args=v,v,v  # main

# 02189560(fm, z): s = 02188C6C(fm); 0202C83C(gamesys, z); (u16) 02012994(gd) != 0xF: 021A1BDC(s)
fn 02189560 ret=none inputs=100
  arg work 0
  arg u32 any
  stub 02188c6c ret=u32 args=v  # in-area
  stub 0202c83c force ret=none args=v,v  # main
  stub 02012994 force ret=pick:0/0xF/0x10 args=v
  stub 021a1c34 force ret=u32 args=v,v  # ov21

# 0218958C(fm, z): 021A0358(02188C68(fm)); 021A0338(+0x1C, 02013EEC(z), 02013F2C(z), 1)
fn 0218958c ret=none inputs=60
  arg work 0
  arg u32 any
  stub 02188c68 ret=u32 args=v  # in-area
  stub 021a0358 force ret=none args=v  # ov21
  stub 02013eec force ret=u32 args=v  # main
  stub 02013f2c force ret=u32 args=v  # main
  stub 021a0338 force ret=u32 args=v,v,v,v  # ov21

# 021895BC(gd, z): 02012AC4(020127A4(gd), (u16) z)
fn 021895bc ret=none inputs=40
  arg u32 any
  arg u32 any
  stub 020127a4 force ret=u32 args=v  # main
  stub 02012ac4 force ret=u32 args=v,v  # main

# 021895D0(fm, z): tail call 021895E8(fm, z, heap (u16 +0)) (`ldrh r2,[r0]; ldr r3,=021895E9; bx r3`)
fn 021895d0 ret=none inputs=60
  arg work 0
  arg u32 any
  stub 021895e8 ret=none args=v,v,v  # in-area
  stub 02013bd0 force ret=pick:0/3/0xFFFF args=v
  stub 0218f14c force ret=u32 args=v,v,v  # ov21
  stub 0218f134 force ret=u32 args=v,p,p,v  # ov21

# 021895DC(fm): +0x10C = 0
fn 021895dc ret=none inputs=10
  arg work 0

# 021895E8(fm, z, heap): w = 02013BD0((u16) z); w != 0xFFFF ? 0218F14C(+0x10, w, heap) : 0218F134(+0x10)
fn 021895e8 ret=none inputs=100
  arg work 0
  arg u32 any
  arg u32 any
  stub 02013bd0 force ret=pick:0/3/0xFFFF args=v
  stub 0218f14c force ret=u32 args=v,v,v  # ov21
  stub 0218f134 force ret=u32 args=v,p,p,v  # ov21

# 02189614(gd, fm): s = 02012B38(gd); 02013DD0(zone (+0xD0)) != 0: k = 0201444C(s); x = k ? 020127D0(+8, 0) : 020127C4(+8)
# (thunk 020127C4 -> 020127D0(x, 1)); 0218DC74(+0x48, x, k); e = 0218DC70(+0x48); 021939C0(e, 02012844(gd), k, heap);
# 02193ABC(e, +0x10, heap); 02193AFC(e, 02012964(+8))
fn 02189614 ret=none inputs=100
  arg u32 any
  arg work 0
  stub 02012b38 force ret=u32 args=v  # main
  stub 02013dd0 force ret=pick:0/1 args=v
  stub 0201444c force ret=pick:0/1 args=v
  stub 020127d0 force ret=u32 args=v,v  # main
  stub 0218dc74 force ret=none args=v,v,v  # ov21
  stub 0218dc70 force ret=u32 args=v  # ov21
  stub 02012844 force ret=u32 args=v  # main
  stub 021939c0 force ret=none args=v,v,v,v  # ov21
  stub 02193abc force ret=none args=v,v,v  # ov21
  stub 02012964 force ret=u32 args=v  # main
  stub 02193afc force ret=u32 args=v,v  # ov21

# 02189684(fm, z): w = 02013FA8((u16) z); w != 0xFFFF: 021AB650(+0x40); 021AB868(+0x44); 021AB840(+0x44, 1, w, 0x96);
# 021AB634(+0x40, 021AB880(+0x44), 021AB884(+0x44), 021AB888(+0x44)); else 02013C80(z) == 0: 021AB650(+0x40), 021AB868(+0x44)
fn 02189684 ret=none inputs=100
  arg work 0
  arg u32 any
  stub 02013fa8 force ret=pick:0/2/0xFFFF args=v
  stub 021ab650 force ret=none args=v  # ov21
  stub 021ab868 force ret=u32 args=v  # ov21
  stub 021ab840 force ret=none args=v,v,v,v  # ov21
  stub 021ab880 force ret=u32 args=v  # ov21
  stub 021ab884 force ret=u32 args=v  # ov21
  stub 021ab888 force ret=u32 args=  # ov21
  stub 021ab634 force ret=u32 args=v,v,v,v  # ov21
  stub 02013c80 force ret=pick:0/1 args=v

# 021896F4(x): k = 02026454(x); k > 5: 2; k == 0: 0200FB24() ? 0202647C(x) : 0; else k (jump table 0..5)
fn 021896f4 ret=u32 inputs=200
  arg u32 any
  stub 02026454 force ret=pick:0/1/2/3/4/5/6 args=v
  stub 0200fb24 force ret=pick:0/1 args=
  stub 0202647c force ret=u32 args=v  # main

# 02189734(x): 02026454(x) == 0: 1; else !02026460(x): 02026400(x); 0
fn 02189734 ret=u32 inputs=100
  arg u32 any
  stub 02026454 force ret=pick:0/1/2 args=v
  stub 02026460 force ret=pick:0/1 args=v
  stub 02026400 force ret=none args=  # main

# 02189758(slot): slot.id = -1; slot.ptr = 0
fn 02189758 ret=none inputs=10
  arg work 0x118

# 02189764(fm, id, heap, size): slot = fm+0x118; id == -1 or slot.id != -1: 0; slot.id = id; slot.ptr = Heap_AllocDebug(heap, size, 1,
# "fieldmap.c", 0xD7C); returns slot.ptr
fn 02189764 ret=u32 inputs=100
  arg work 0
  arg s32 alt oneof 0xFFFFFFFF 0 5 || any
  arg u32 any
  arg u32 any
  fix work 0x28 u32 alt const 0 || any
  fix work 0x30 u32 alt const 0 || any
  fix work 0x38 u32 alt const 0 || any
  fix work 0x98 u32 alt const 0 || any
  fix work 0xC0 u32 alt oneof 0 1 2 || any
  fix work 0xC4 u32 range 0 5
  fix work 0xC8 u8 alt range 0 7 || oneof 0 1
  fix work 0xC9 u8 range 0 1
  fix work 0xD0 u16 alt oneof 0x3f 0x8c 0x8d 0x8e 0x8f 0 || range 0 0x1ff || any
  fix work 0xD8 u16 alt oneof 0x3f 0x8c 0xFFFF || range 0 0x1ff
  fix work 0xF0 u32 alt const 0 || addrof work 0x480
  fix work 0x108 u32 addrof work 0x400
  fix work 0x118 s32 alt oneof 0xFFFFFFFF 0 1 2 || any
  fix work 0x140 u32 alt const 0 || any
  fix work 0x150 u32 alt const 0 || any
  fix work 0x400 u32 alt range 0 2 || range 0 8
  fix work 0x404 u32 const 0x02EE0101
  fix work 0x408 u32 const 0x02EE0111
  fix work 0x40C u32 const 0x02EE0121
  fix work 0x2C u32 alt const 0 || any
  stub 02030734 force ret=u32 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line): blocks that dereference the result override with ptr:

# 021897A0(fm, id): id != -1: Heap_Free(slot.ptr), slot.id = -1 (the id is not compared with slot.id)
fn 021897a0 ret=none inputs=60
  arg work 0
  arg s32 alt oneof 0xFFFFFFFF 0 5 || any
  stub 020307b0 force ret=none args=v  # Heap_Free(block)

# 021897BC(fm, id): id == -1 or id != slot.id: 0; else slot.ptr
fn 021897bc ret=u32 inputs=100
  arg work 0
  arg s32 alt oneof 0xFFFFFFFF 0 1 2 || any
  fix work 0x28 u32 alt const 0 || any
  fix work 0x30 u32 alt const 0 || any
  fix work 0x38 u32 alt const 0 || any
  fix work 0x98 u32 alt const 0 || any
  fix work 0xC0 u32 alt oneof 0 1 2 || any
  fix work 0xC4 u32 range 0 5
  fix work 0xC8 u8 alt range 0 7 || oneof 0 1
  fix work 0xC9 u8 range 0 1
  fix work 0xD0 u16 alt oneof 0x3f 0x8c 0x8d 0x8e 0x8f 0 || range 0 0x1ff || any
  fix work 0xD8 u16 alt oneof 0x3f 0x8c 0xFFFF || range 0 0x1ff
  fix work 0xF0 u32 alt const 0 || addrof work 0x480
  fix work 0x108 u32 addrof work 0x400
  fix work 0x118 s32 alt oneof 0xFFFFFFFF 0 1 2 || any
  fix work 0x140 u32 alt const 0 || any
  fix work 0x150 u32 alt const 0 || any
  fix work 0x400 u32 alt range 0 2 || range 0 8
  fix work 0x404 u32 const 0x02EE0101
  fix work 0x408 u32 const 0x02EE0111
  fix work 0x40C u32 const 0x02EE0121
  fix work 0x2C u32 alt const 0 || any

# 021899F0(fm): +0x12C (draw mode)
fn 021899f0 ret=u32 inputs=10
  arg work 0

# 021899F8(fm, v): +0x12C = v
fn 021899f8 ret=none inputs=10
  arg work 0
  arg u32 any

# 02189A00(fm): +0x124
fn 02189a00 ret=u32 inputs=10
  arg work 0

# 02189A08(fm): +0x13C
fn 02189a08 ret=u32 inputs=10
  arg work 0

# 02189A10(fm): +0xA0
fn 02189a10 ret=u32 inputs=10
  arg work 0

# 02189A18(fm): +0x40
fn 02189a18 ret=u32 inputs=10
  arg work 0

# 02189A1C(fm, v): +0x138 = v
fn 02189a1c ret=none inputs=10
  arg work 0
  arg u32 any

# 02189A24(fm): +0x138
fn 02189a24 ret=u32 inputs=10
  arg work 0

# 02189A2C(fm): tail call 020120D8(gamesys (+4)) (`ldr r0,[r0,4]; ldr r3,=020120D9; bx r3`)
fn 02189a2c ret=u32 inputs=20
  arg work 0
  stub 020120d8 force ret=pick:0/1 args=v

# 02189A38(fm): 020113D8(02012984(gd (+8)))
fn 02189a38 ret=u32 inputs=20
  arg work 0
  stub 02012984 force ret=u32 args=v  # main
  stub 020113d8 force ret=u32 args=v,v,v,v  # main

# 02189A48(fm): +0x144
fn 02189a48 ret=u32 inputs=10
  arg work 0

# 02189A50(fm, v): +0x144 = v
fn 02189a50 ret=none inputs=10
  arg work 0
  arg u32 any

# 02189A58(fm, v): +0x140 = v
fn 02189a58 ret=none inputs=10
  arg work 0
  arg u32 any

# 02189A60(fm): fm + 0x148 (address)
fn 02189a60 ret=ofs:work inputs=10
  arg work 0

# 02189A68(fm, v): +0x150 = v
fn 02189a68 ret=none inputs=10
  arg work 0
  arg u32 any

# 02189A70(fm): +0x150
fn 02189a70 ret=u32 inputs=10
  arg work 0

# 02189A78(fm): +0x100
fn 02189a78 ret=u32 inputs=10
  arg work 0

# 02189A80(fm): a = u16 *0218ED3C(+0x10) (camera angle); c = |s16 ROM sin/cos table 0x020A1B38 [2 (a >> 4) + 1]| (cos);
# d = ((c << 14) + 0x800) >> 12 (64-bit) - 0x4000; returns max(d, -0x2000)
fn 02189a80 ret=u32 inputs=200
  arg work 0
  fix work 0x280 u16 any
  stub 0218ed3c force ret=ptr:work:0x280:1 args=v

# 02189ACC(z): 02013BA8(z) == 1 ? 494 : 310
fn 02189acc ret=u32 inputs=40
  nowork
  arg u32 any
  stub 02013ba8 force ret=pick:0/1/2 args=v

# 02189AE8(z): z == 0x3F: 0; 02013CE0(z) == 1: 0; else 1
fn 02189ae8 ret=u32 inputs=60
  nowork
  arg u32 alt oneof 0x3f 0 || any
  stub 02013ce0 force ret=pick:0/1/2 args=v

# 02189B04(z): z is 0x3F, 0x8C, 0x8D or 0x8E
fn 02189b04 ret=bool inputs=60
  nowork
  arg u32 alt oneof 0x3f 0x8c 0x8d 0x8e 0x8f 0x3e || any

# 02189B30(z, area, out, mtx): k = 02013B50(z) (thunk 02189B28); out[0..0x3B] = MAP SETUP 0x021D3700 [k] (0x3C bytes);
# record +0x40 != 0: out+0x14 = 02159BF4(mtx) (u16), +0x16 = 02159BF8(mtx) (u16), +0x18 = 02159BFC(mtx), +0x1C = 02159C20(mtx);
# out+0x20 = 1; +0x24 = 0xE; +0x28 = 020140BC(area); +0x2C = 020140C0(area); +0x30 = 020140CC(area)
fn 02189b30 ret=none inputs=100
  arg u32 any
  arg u32 any
  arg work 0x50
  arg u32 any
  stub 02013b50 force ret=pick:0/1/2/3/4/5/6/7 args=v  # zone kind (index of the MAP SETUP table 0x021D3700)
  stub 02159bf4 force ret=u32 args=v  # map_matrix
  stub 02159bf8 force ret=u32 args=v  # map_matrix
  stub 02159bfc force ret=u32 args=v  # map_matrix
  stub 02159c20 force ret=u32 args=v  # map_matrix
  stub 020140bc force ret=u32 args=v  # main
  stub 020140c0 force ret=u32 args=v  # main
  stub 020140cc force ret=u32 args=v  # main

# 02189BA8(z): MAP SETUP 0x021D3700 [02013B50(z)] +0x3C (MAP CTRL pointer)
fn 02189ba8 ret=u32 inputs=40
  nowork
  arg u32 any
  stub 02013b50 force ret=pick:0/1/2/3/4/5/6/7 args=v  # zone kind (index of the MAP SETUP table 0x021D3700)

# 02189BBC(z): MAP SETUP 0x021D3700 [02013B50(z)] +0x44
fn 02189bbc ret=u32 inputs=40
  nowork
  arg u32 any
  stub 02013b50 force ret=pick:0/1/2/3/4/5/6/7 args=v  # zone kind (index of the MAP SETUP table 0x021D3700)
