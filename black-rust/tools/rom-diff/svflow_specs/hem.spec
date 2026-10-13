# svflow spec: hem (0x021C8810-0x021C8918; B10bc-spec). Harness-job owned; porting workers may not edit.
# Grammar: svflow_specs/README.md (B10-specs .. B12-spec additions). No new grammar.
#
# What this area is: the HANDEX manager of btl_server_flow.c. Two record kinds live here:
#  * message records ("str", 0x24 bytes, also embedded at +8.. of many HANDEX records): u16 string id at +0; u16 at +2 =
#    bits 0-7 kind (1 / 2 = the two message families of 021c99a0), bits 8-14 argument count (max 8), bit 15 "has the
#    extra u16 at +0x20"; up to 8 u32 arguments at +4..+0x20 (the 8th slot doubles as the +0x20 extra word).
#  * HANDEX records: allocated by the cmd_queue record allocator 021cc9ac in the manager at work + 0x1D74 (word 0 = packed
#    header: bits 0-9 a value handed to some procs, 10-18 bytes used, 28-30 flags; records from +4, 0x1F5 bytes); record
#    word 0 = id (bits 0-7, the 021c8918 switch value 0x00-0x3B), pokeID (bits 8-12), size (bits 13-22), flags 23-26.
#   021c8810   str_clear(rec): MI_CpuFill8(rec, 0, 0x24), then clears the kind byte again
#   021c8828   str_is_set(rec): (rec[+2] & 0xFF) != 0
#   021c8838   str_init(rec, kind, id): kind byte, id, argument count := 0
#   021c8860   str_push(rec, v): append an argument (no-op when 8 are there)
#   021c888c   str_set_extra(rec, v): rec[+0x20] = v and bit 15 (unless 8 arguments are there)
#   021c88a4   handex_push(work, id, pokeID): thunk-like `adds r0, 0x1D74; bx 021cc9ac` = alloc_record(work + 0x1D74, id, v)
#   021c88b4   handex_push_exec(work, id, pokeID): 88a4 then 88c8 (the record is executed at once)
#   021c88c8   handex_pop(work, rec): 021c8918(work, rec) (execute; handex area), then free_last_record(work + 0x1D74, rec)
#   021c88e4   handex_state(work): 0 when manager bit 30 is clear, else 2 if bit 29 is set, 1 if not
#   021c8910   `ldr r3,[pc]; bx r3` thunk to 021c88e4 (specced as a function: the thunk runs, 88e4 is real or stubbed)
#
# Register facts (disassembly; the pseudo-C and the B9 notes were wrong here): 8828 is a predicate on a str record, not
# an "init"; 888c sets the extra word, it does not push a HANDEX; 88a4 takes (work, id, v) and returns the record
# pointer (0 when the record does not fit); 88b4 takes (work, id, v) and passes r1/r2 through to 88a4; 88e4 takes only
# the work pointer and calls the cmd_queue bit readers cc96c / cc9a4 (not cca9c).
#
# SvFlowWork fields seen here: +0x1D74 HANDEX manager (packed word, records from +0x1D78).
#
# Layout (core4 convention, closure stubs): every block lists the stubs of its callees and of its in-area descendants'
# callees, so any porting order needs no spec edit (checked: with every hem fn treated as ported, all blocks run without a
# fault, a stray write or an unstubbed callee). Callees in cmd_queue (cc96c cc9a4 cc9ac cca74, all PORTED) run as real
# code on both sides; MI_CpuFill8 is in svflow.py ALLOW. The executor 021c8918 (handex area) is a `force` stub: hem's
# ports call it through env.call, and its own spec in handex.spec verifies it.
#
# Input notes: str argument counts are drawn 0..9 (alt any) so the "8 arguments" bound is hit; 88a4 ids are mostly valid
# table ids (0x0D and > 0x3B allocate nothing); 88b4 always allocates (a failed allocation would make 88c8 / cca74 read
# address 0, which the real game never does); 88c8's record is one of the first three slots with small sizes so the
# "last record" shrink path of cca74 is taken on a share of the inputs.

area hem

# str_clear(rec)
fn 021c8810 ret=none
  arg buf 0x24

# str_is_set(rec): kind byte != 0
fn 021c8828 ret=bool
  arg buf 4

# str_init(rec, kind, id)
fn 021c8838 ret=none
  arg buf 0x24
  arg u32 any
  arg u32 any

# str_push(rec, v)
fn 021c8860 ret=none
  arg buf 0x24
  arg u32 any
  fix arg0 2 u16 alt range 0 9 || any bits=8:7

# str_set_extra(rec, v)
fn 021c888c ret=none
  arg buf 0x24
  arg u32 any
  fix arg0 2 u16 alt range 0 9 || any bits=8:7

# handex_push(work, id, v) -> record pointer (0 = no room / unknown id)
fn 021c88a4 ret=ofs:work
  arg work
  arg u32 alt range 0 0xC || range 0xE 0x3B || any
  arg u32 any
  fix work 0x1D74 u32 alt range 0 0x100 || any bits=10:9

# handex_push_exec(work, id, v): 88a4 then 88c8
fn 021c88b4 ret=none
  arg work
  arg u32 alt range 0 0xC || range 0xE 0x3B
  arg u32 any
  fix work 0x1D74 u32 range 0 0x100 bits=10:9
  stub 021c88a4 ret=ptr:work:0x1D78:1 args=p,v,v
  stub 021c88c8 ret=none args=p,v
  stub 021c8918 force ret=none args=p,v

# handex_pop(work, rec): execute (021c8918, force stub) then free_last_record(work + 0x1D74, rec)
fn 021c88c8 ret=none
  arg work
  arg u32 oneof 0x02E01D78 0x02E01D7C 0x02E01D80
  fix work 0x1D74 u32 range 0 12 bits=10:9
  fix work 0x1D78 u32 range 0 4 bits=13:10
  fix work 0x1D7C u32 range 0 4 bits=13:10
  fix work 0x1D80 u32 range 0 4 bits=13:10
  stub 021c8918 force ret=none args=p,v

# handex_state(work): 0 / 1 / 2 from manager bits 30 / 29
fn 021c88e4 ret=u32
  arg work

# thunk to 021c88e4 (the thunk lands on the 88e4 stub, or on the real 88e4 once it is ported)
fn 021c8910 ret=u32
  arg work
  stub 021c88e4 ret=pick:0/1/2 args=p
