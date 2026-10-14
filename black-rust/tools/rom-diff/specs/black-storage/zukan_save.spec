# fndiff spec (S5b): the Pokedex save block 55 of the main ARM9 (0x0200C720..0x0200D414): 0x4D4 bytes (magic,
# flags, caught / seen / shown species bits, form bits, language bits, Spinda personality).
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-storage zukan_save --per-function
# The rodata tables 0x0209DE20 / 0x0209DE2C / 0x0209DE4C are read as real data; MI_CpuFill8 runs as real code.
# Stubbed: the Pokemon parameter / gender / shiny getters and lock (02017E1C / 02017F00 / 02017EAC / 02017D30 /
# 02017D70), the personal data (0201ADE4 / 0201AE1C / 0201AE14), the regional Pokedex table 0201AF80 (a pointer
# into a test-only arena argument) and Heap_Free. Species are drawn around the form species, the table ends and
# the 1..649 bounds; functions that index a bit set by species without checking it get 1..649 only.
area zukan_save
set field

# FUN_0200C720(species): 0 or > 649
fn 0200c720 ret=bool inputs=200
  nowork
  arg u32 alt range 0 3 || near 649 3 || any

# 0x0200C738(zukan, species, lang): set the language bit
fn 0200c738 ret=none inputs=600
  nowork
  arg buf 0x4d4
  arg u32 alt range 2 10 || near 493 3 || range 2 649
  arg u32 alt range 0 10 || any

# FUN_0200c780 / FUN_0200c7a0(species): not in the excluded lists
fn 0200c780 ret=bool inputs=300
  nowork
  arg u32 alt oneof 151 251 385 386 489 490 491 492 493 494 645 646 647 648 649 || range 0 700

fn 0200c7a0 ret=bool inputs=300
  nowork
  arg u32 alt oneof 494 645 646 647 648 649 || range 0 700

# FUN_0200C7C0 / FUN_0200c7f4(species): form bit start / form table index (-1 if none)
fn 0200c7c0 ret=u32 inputs=300
  nowork
  arg u32 alt oneof 201 386 492 487 479 422 423 412 413 351 421 585 586 648 555 550 || range 0 700 || any

fn 0200c7f4 ret=u32 inputs=300
  nowork
  arg u32 alt oneof 201 386 492 487 479 422 423 412 413 351 421 585 586 648 555 550 || range 0 700 || any

# 0x0200C818(zukan, species, gender, shiny): set a seen bit
fn 0200c818 ret=none inputs=400
  nowork
  arg buf 0x4d4
  arg u32 range 1 649
  arg u32 alt range 0 3 || any
  arg u32 alt range 0 2 || any

# FUN_0200c914(zukan, species, _, shiny, form): set form-seen bits
fn 0200c914 ret=none inputs=800
  nowork
  arg buf 0x4d4
  arg u32 alt oneof 201 386 492 487 479 422 423 412 413 351 421 585 586 648 555 550 || range 1 649
  arg u32 any
  arg u32 alt range 0 2 || any
  arg u32 alt range 0 30 || any

fn 0200c9ec ret=u32 inputs=10
  nowork

# sub_0200c9f4(zukan): zero, magic
fn 0200c9f4 ret=none inputs=30
  nowork
  arg buf 0x4d4

# flag accessors on +4
fn 0200ca28 ret=none inputs=50
  nowork
  arg buf 0x4d4

fn 0200ca38 ret=u32 inputs=50
  nowork
  arg buf 0x4d4

fn 0200ca40 ret=none inputs=50
  nowork
  arg buf 0x4d4

fn 0200ca54 ret=u32 inputs=50
  nowork
  arg buf 0x4d4

fn 0200ca5c ret=none inputs=50
  nowork
  arg buf 0x4d4
  arg u32 any

fn 0200ca70 ret=u32 inputs=50
  nowork
  arg buf 0x4d4

fn 0200ca78 ret=none inputs=50
  nowork
  arg buf 0x4d4
  arg u32 any

fn 0200ca8c ret=u16 inputs=50
  nowork
  arg buf 0x4d4

# FUN_0200CA98(zukan, species, gender, shiny, form): set the shown kind / form
fn 0200ca98 ret=none inputs=800
  nowork
  arg buf 0x4d4
  arg u32 alt oneof 201 386 492 487 479 422 423 412 413 351 421 585 586 648 555 550 || range 1 649
  arg u32 alt range 0 3 || any
  arg u32 alt range 0 2 || any
  arg u32 alt range 0 30 || any

# FUN_0200CC20(species): form count
fn 0200cc20 ret=u32 inputs=200
  nowork
  arg u32 alt oneof 201 386 492 487 479 422 423 412 413 351 421 585 586 648 555 550 || range 0 700 || any

# FUN_0200CC40(zukan, species, &gender, &shiny, &form, heap): the shown kind (out words start random)
fn 0200cc40 ret=none inputs=800
  nowork
  arg buf 0x4d4
  arg u32 alt oneof 201 386 492 487 479 422 423 412 413 351 421 585 586 648 555 550 || range 1 649
  arg buf 0x4
  arg buf 0x4
  arg buf 0x4
  arg u32 any
  fix arg0 0x1ac u8 alt const 0 || const 0 || any rep=336,1
  fix arg0 0x30e u8 alt const 0 || const 0 || any rep=18,1
  stub 0201ade4 ret=u32 args=v,v,v
  stub 0201ae1c ret=pick+u32:255/127/0 args=v,v
  stub 0201ae14 ret=none args=v

# FUN_0200CDE0(zukan, mon): register a caught Pokemon
fn 0200cde0 ret=none inputs=1200
  nowork
  arg buf 0x4d4
  arg u32 any
  fix arg0 0x5c u8 alt const 0 || const 0 || any rep=336,1
  fix arg0 0x4d0 u32 alt const 0 || any
  stub 02017d30 ret=u32 args=v
  stub 02017d70 ret=none args=v,v
  stub 02017e1c ret=nth:pick:0/0/0/1;pick:0/2/5/201/327/351/386/479/550/649/650;u32;pick+u32:0/1/2/3/30;pick+u32:0/1/2/5/6/7/8/9 args=v,v,v
  stub 02017f00 ret=pick:0/1/2 args=v
  stub 02017eac ret=pick+u32:0/1 args=v

# FUN_0200CED4(zukan, species): caught bit
fn 0200ced4 ret=bool inputs=300
  nowork
  arg buf 0x4d4
  arg u32 alt range 0 3 || near 649 3 || range 1 649

# FUN_0200CF08(zukan, heap, arena): caught count of the Pokedex in use
fn 0200cf08 ret=u16 inputs=60
  nowork
  arg buf 0x4d4
  arg u32 any
  arg buf 0x520
  fix arg2 0 u16 alt const 999 || range 0 300 rep=650,2
  stub 0201af80 ret=ptr:arg2:0:1 args=v,v
  stub 020307b0 ret=none args=v

fn 0200cf28 ret=u16 inputs=60
  nowork
  arg buf 0x4d4

fn 0200cf54 ret=u16 inputs=60
  nowork
  arg buf 0x4d4
  arg u32 any
  arg buf 0x520
  fix arg2 0 u16 alt const 999 || range 0 300 rep=650,2
  stub 0201af80 ret=ptr:arg2:0:1 args=v,v
  stub 020307b0 ret=none args=v

# FUN_0200CFA0(zukan, mon): register a seen Pokemon
fn 0200cfa0 ret=none inputs=1200
  nowork
  arg buf 0x4d4
  arg u32 any
  fix arg0 0x5c u8 alt const 0 || const 0 || any rep=336,1
  fix arg0 0x4d0 u32 alt const 0 || any
  stub 02017d30 ret=u32 args=v
  stub 02017d70 ret=none args=v,v
  stub 02017e1c ret=nth:pick:0/0/0/1;pick:0/2/5/201/327/351/386/479/550/649/650;u32;pick+u32:0/1/2/3/30;pick+u32:0/1/2/5/6/7/8/9 args=v,v,v
  stub 02017f00 ret=pick:0/1/2 args=v
  stub 02017eac ret=pick+u32:0/1 args=v

# FUN_0200D064(zukan, species): any seen bit
fn 0200d064 ret=bool inputs=300
  nowork
  arg buf 0x4d4
  arg u32 alt range 0 3 || near 649 3 || range 1 649
  fix arg0 0x5c u8 alt const 0 || const 0 || any rep=336,1

fn 0200d0ac ret=u16 inputs=60
  nowork
  arg buf 0x4d4
  arg u32 any
  arg buf 0x520
  fix arg2 0 u16 alt const 999 || range 0 300 rep=650,2
  stub 0201af80 ret=ptr:arg2:0:1 args=v,v
  stub 020307b0 ret=none args=v

fn 0200d0cc ret=u16 inputs=60
  nowork
  arg buf 0x4d4

fn 0200d0f8 ret=u16 inputs=60
  nowork
  arg buf 0x4d4
  arg u32 any
  arg buf 0x520
  fix arg2 0 u16 alt const 999 || range 0 300 rep=650,2
  stub 0201af80 ret=ptr:arg2:0:1 args=v,v
  stub 020307b0 ret=none args=v

# 0x0200D144(zukan, species, gender, shiny, form): seen with this kind (shiny 0/1)
fn 0200d144 ret=u32 inputs=800
  nowork
  arg buf 0x4d4
  arg u32 alt oneof 201 386 492 487 479 422 423 412 413 351 421 585 586 648 555 550 || range 0 700
  arg u32 alt range 0 3 || any
  arg u32 range 0 1
  arg u32 range 0 1

# 0x0200D288(zukan, k): Spinda personality for k == 0
fn 0200d288 ret=u32 inputs=50
  nowork
  arg buf 0x4d4
  arg u32 alt const 0 || any

# FUN_0200D298 / FUN_0200D2D8(zukan): national completion (bits mostly set so both answers occur)
fn 0200d298 ret=bool inputs=60
  nowork
  arg buf 0x4d4
  fix arg0 0x8 u8 alt const 0xff || const 0xff || const 0xff || any rep=84,1

fn 0200d2d8 ret=u16 inputs=60
  nowork
  arg buf 0x4d4

# FUN_0200D2B0 / D2C4(zukan, heap, arena): regional completion
fn 0200d2b0 ret=bool inputs=60
  nowork
  arg buf 0x4d4
  arg u32 any
  arg buf 0x520
  fix arg2 0 u16 alt const 999 || range 0 300 || range 0 300 rep=650,2
  stub 0201af80 ret=ptr:arg2:0:1 args=v,v
  stub 020307b0 ret=none args=v

fn 0200d2c4 ret=bool inputs=60
  nowork
  arg buf 0x4d4
  arg u32 any
  arg buf 0x520
  fix arg2 0 u16 alt const 999 || range 0 300 || range 0 300 rep=650,2
  stub 0201af80 ret=ptr:arg2:0:1 args=v,v
  stub 020307b0 ret=none args=v

fn 0200d310 ret=u16 inputs=60
  nowork
  arg buf 0x4d4
  arg u32 any
  arg buf 0x520
  fix arg2 0 u16 alt const 999 || range 0 300 rep=650,2
  stub 0201af80 ret=ptr:arg2:0:1 args=v,v
  stub 020307b0 ret=none args=v

fn 0200d368 ret=u16 inputs=60
  nowork
  arg buf 0x4d4
  arg u32 any
  arg buf 0x520
  fix arg2 0 u16 alt const 999 || range 0 300 rep=650,2
  stub 0201af80 ret=ptr:arg2:0:1 args=v,v
  stub 020307b0 ret=none args=v

# 0x0200D3C0(zukan, species, lang): language bit
fn 0200d3c0 ret=bool inputs=600
  nowork
  arg buf 0x4d4
  arg u32 alt range 2 10 || near 493 3 || range 2 700
  arg u32 alt range 0 10 || any
