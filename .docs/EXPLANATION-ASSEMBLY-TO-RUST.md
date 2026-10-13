spencerlepine/black · Pokémon Black, checked 13 Oct 2026

# From assembly to Rust, in plain terms

What the old code is, what the new code is, how faithful the translation is, and whether it makes modding easier. Written for someone who knows Python and TypeScript.

## The short answers

**Will Rust be easier to mod?**Yes, by a lot, mostly because the new version runs on a normal computer or in a browser and keeps art and music as normal files. Today the logic code still copies the original's memory layout closely, so some of it is still cryptic. That improves as it gets cleaned up.

**Is there C you could edit?**Barely. The original side is about **1.42 million lines of assembly** and only **372 lines of C** (two files). Someone has to turn assembly into C by hand, and almost none of that has been done.

**Copy or rewrite?**Both. Game rules (battles, Pokémon stats, scripts) are translated one function at a time and tested to behave exactly like the original. Graphics, menus and sound are rewritten from scratch.

**More or less code?**Less. Rust is about **154,600 lines** today at roughly 19% done. Each ported function is usually 2 to 6 times shorter than its assembly.

**For an expert in both?**The language matters less than where the code runs. The C version is stuck on Nintendo DS hardware. The Rust version is a normal program, and Rust's compiler catches whole classes of crashes that C lets through.

## Where the languages sit

Think of a line. On the left, you tell the chip exactly what to do, one tiny step at a time. On the right, you describe what you want and the language handles the details. Moving right makes code shorter and friendlier. Moving left gives more control and speed.

For you specifically: Rust will feel like a stricter TypeScript. It has types, `enum`s, and functions like you're used to. The new part is that Rust makes you be explicit about who owns each piece of data, which is how it prevents crashes without a garbage collector.

## One real function, three ways

Pokémon store their two types (say Water and Ground) packed into one number. The game has a small helper that answers "does this Pokémon have type X?" Here it is in the original assembly, exactly as it sits in `black-rom/asm/battle_view_core.s`, and its Rust port in `black-rust/crates/black-battle/src/poke_type.rs`.

Original assembly18 lines

```
FUN_overlay_d_93__021e8d24:
    push {r3, r4, r5, lr}
    add r5, r0, #0
    add r4, r1, #0
    bl FUN_overlay_d_93__021e8cfc
    cmp r4, r0
    bne _021E8D36
    mov r0, #1
    pop {r3, r4, r5, pc}
_021E8D36:
    add r0, r5, #0
    bl FUN_overlay_d_93__021e8d04
    cmp r4, r0
    bne _021E8D44
    mov r0, #1
    pop {r3, r4, r5, pc}
_021E8D44:
    mov r0, #0
    pop {r3, r4, r5, pc}
```

Rust port3 lines

```
/// C: `FUN_overlay_d_93__021e8d24`
pub fn pair_has_type(pair: u16, ty: u8) -> bool {
    pair_type1(pair) == ty || pair_type2(pair) == ty
}
```

Same idea in Pythonfor comparison

```
def pair_has_type(pair: int, ty: int) -> bool:
    return pair_type1(pair) == ty or pair_type2(pair) == ty
```

The assembly works with numbered storage slots (`r0` to `r5`), compares, and jumps to labels. The function doesn't even have a real name, just its memory address `021e8d24`. The Rust says what it means, and the comment points back to the original address so anyone can check it.

### Not every port is that pretty yet

Here's a slightly bigger one: "find which slot this Pokémon is in, in the battle party." The Rust is far easier to read, but notice `mem.r8(p + PARTY_COUNT)`. It reads raw bytes at an offset (`PARTY_COUNT` is `0x18`, byte 24 of the party), the same way the assembly does. That's on purpose, explained below.

Original assembly · battle_server.s17 lines

```
BattleParty_GetMonIndex:
    ldrb r2, [r0, #0x18]
    mov r3, #0
    cmp r2, #0
    ble _021B9CB4
_021B9CA0:
    lsl r2, r3, #2
    ldr r2, [r0, r2]
    cmp r2, r1
    bne _021B9CAC
    add r0, r3, #0
    bx lr
_021B9CAC:
    ldrb r2, [r0, #0x18]
    add r3, r3, #1
    cmp r3, r2
    blt _021B9CA0
_021B9CB4:
    mov r0, #0
    mvn r0, r0
    bx lr
```

Rust port · server3.rs9 lines

```
pub fn party_index_of(mem: &Mem, p: u32, mon: u32) -> u32 {
    let mut i = 0u32;
    while (i as i32) < mem.r8(p + PARTY_COUNT) as i32 {
        if mem.r32(p + (i << 2)) == mon {
            return i;
        }
        i += 1;
    }
    u32::MAX  // "not found", same as the original's -1
}
```

What a later cleanup could look likeillustration, not in the repo

```
fn index_of(party: &Party, mon: MonId) -> Option<usize> {
    party.mons.iter().position(|m| *m == mon)
}
```

## Copying or rewriting?

The project's plan (`MIGRATION-AGENT.md`, decisions D2 and D3) splits the game into two kinds of code and treats them differently.

So for game rules the answer is: it is a careful translation, not a creative rewrite. The goal is that a battle in the Rust version plays out exactly like the real cartridge. That's why some ports still read memory by byte offset. Keeping the original layout makes it possible to prove the Rust gives identical results, by running the original machine code in an emulator (Unicorn) and the Rust side by side on hundreds of random inputs. Once everything matches, the code can be safely tidied into friendlier shapes, and the tests will catch any mistake.

It is not "making it read worse," though. Even the faithful ports turn jumps and register juggling into loops, named functions and comments pointing to the original address. Readability goes up a lot. It just hasn't reached its final, cleanest form.

## How much code, before and after

Counted directly in the repo today. Bars are drawn to the same scale.

Original, all assembly406 files, \~28,700 functions

1,418,960 lines

…of which instructionsthe rest is data and labels

\~976,600

Original, decompiled C2 files

372 lines

Rust so far340 files, \~19% done; 33,700+ lines are tests

154,587 lines

Rust when finishedmy estimate, not measured

roughly 400k to 700k?

The finished size is a guess. Dividing 154,600 by 19% gives about 800,000, but that's too simple: the 19% measures effort, not lines, and a lot of the remaining work is graphics and menus, which get rebuilt in a modern engine instead of copied. A chunk of the Rust is also tests that the original never had. Either way, expect clearly less code than the assembly, and much more readable code, but not a tiny program. Pokémon is a big game.

## What changes for modding

Here's what adding a new move, a new item or a new screen looks like in each version.

|  | Original (assembly + a little C) | Rust version |
| --- | --- | --- |
| Where it runs | Nintendo DS ROM only. You test in an emulator. | A normal app on your computer, or in a web browser. |
| Building it | Needs Nintendo's old proprietary compiler and SDK, run through Wine in Docker. | `cargo build`, the standard free Rust tool. |
| Reading it | Mostly assembly; many functions only have an address for a name. | Named functions, types and comments pointing back to the original. |
| Art and music | Packed in DS-specific archive formats. | Extracted to PNG, OGG and JSON. Swap a file to change it. |
| Limits | DS memory (a few MB) and its graphics chips. Big changes hit hardware walls. | A whole PC. Add as much as you want. |
| Catching mistakes | A typo can silently corrupt memory and crash later. | The compiler refuses many bugs before the game runs. Tests check behaviour. |
| Ease of a quick hack | Easy to poke one value if you know where it lives. | The strict compiler can feel slow at first. You have to satisfy it. |

### If you were an expert in both C and Rust

There would still be a real difference, but the bigger part of it isn't the language.

**The platform is the main thing.** A C expert working on the original is still writing a DS program: it must fit the DS's memory, use its graphics hardware, and be built with Nintendo's toolchain. The Rust version is an ordinary program, so an expert can use any library, any screen size, online features, or a browser build.

**The language still helps.** In C, nothing stops you from reading past the end of a list or using data after it's been freed, and bugs like that show up as random crashes much later. Rust's compiler blocks those. Rust's `enum` and `match` also force you to handle every case: add a new status condition and the compiler lists every place that needs updating. In a codebase this big, that's a big deal for adding features without breaking old ones.

**The tests are a bonus.** Every ported function has a test proving it matches the original. If you change a rule on purpose, you see exactly what else changed.

**The catch:** experts often find C quicker for small, low-level hacks, and Rust's strictness takes time to learn. For a big game you want to extend for years, Rust's safety usually wins.

**Bottom line.** For you, the Rust version is the one to mod. It's readable, runs anywhere, and uses normal file formats. Right now the battle and Pokémon logic are the furthest along; the overworld and screens are still in progress, so the game isn't fully playable in Rust yet.

Sources: line counts from `wc -l` over `black-rom/{asm,src,include}` and `black-rust/**/*.rs` at commit `ea8cae4`; plan and progress from `MIGRATION-AGENT.md` §1–2; code samples copied from the files named above. The finished-size range and the cleanup example are my illustrations, not from the repo.