# black-text

Message engine: string buffers, control codes, variable expansion, word wrap.
Logic crate (D6): no Bevy, and no `nds-rom` dependency; assets reach it as plain slices/traits.

## Ported (area `strbuf`, `tools/check-fn.sh black-text strbuf --per-function`)
Undecompiled callees go in `hooks` as traits (§4 porting pattern). Ports are verified per function
with `tools/check-fn.sh black-text <area> --per-function` against the original machine code.

`strbuf.c` / `msgdata.c` get-string path, main ARM9 0x02045730..0x02045E94 (ARM mode). All 33
spec'd functions match the ROM. Buffers are raw byte images `{u16 capacity, u16 length,
u32 magic 0xB6F8D2EC, u16 chars[]}` (same as the original, so save/ROM data can alias them).

| C | Rust (`strbuf`) |
|---|------|
| `FUN_02045730..98` | `rec_*` getters/setters of a small byte-field record (struct name UNVERIFIED) |
| `FUN_020457A0` | `set_eos` (models the global store; ports use the `EOS` constant) |
| `FUN_020457B0`, `02045808`, `02045874` | `create`, `free`, `dup` over the `StrHeap` hook |
| `FUN_0204581C`, `5838`, `589C`, `58F8`, `5900`, `5924`, `5A00`, `5A38`, `5A90`, `5A98`, `5AA8`, `5AFC` | `clear`, `copy`, `eq`, `len`, `truncate`, `copy_chars`, `set_from`, `copy_out`, chars offset, `EOS`, `append`, `add_char` |
| `FUN_02045CAC`, `5E04`, `5E3C` | `msg_get` (preloaded-table path), `decrypt_buf`, `decrypt` |

Deliberate deviations: the end-of-string marker is the constant `0xFFFF` (the original reads a
global that `FUN_020457A0` can change; nothing in the game does). `copy_out` with `n == 0` is a
no-op (the original writes before the destination). `msg_get` ports only the preloaded-MsgData
path; the file-read path (`FUN_0204960C`/`FUN_020495F4`) and `MsgData_Create` (`FUN_02045B38`) are
not ported. The language-reload callee `FUN_02046180` is behind `MsgHooks`.

## Plan for the rest of F9 (not started)

1. **Control-code tokens.** A message is a `u16` string after `msg_get`. Data-side parsing already
   exists in `nds_rom::msg` (A3: `F000 id n params..` commands, `FFFE` newline, `FFFF` end, `BExx`
   wait/clear, `FF00(n)` colour, `BDxx` layout, `F100` compressed). `black-text` must not depend
   on it (D6): define its own `Token` enum + `parse(&[u16]) -> Vec<Token>` (small, ~100 lines,
   unit-tested against `nds_rom::msg` output in `black-cli` or a dev-dependency-free fixture
   dump), and keep text as `Vec<u16>` at runtime.
2. **Word insertion (`wordset.c`, `unk_0201EC64` = `WordSet_Create(8, 0x20, heap)`).** Port the
   WordSet struct (8 slots x 0x20 chars + per-slot gender/plural flags), the `Set*` setters
   (species, nickname, move, item, trainer name, number with digit count/zero pad, place), then
   `StringExpand`: walk the template, replace each `F000 0100..` variable command with the slot's
   text. Needs the name banks through a `NameSource` trait (bank id + index -> `Vec<u16>`), with
   `black-cli`/assets implementing it. Leaf-first from the `index.jsonl` callees of the
   `FUN_02024368` family; verify the string-only functions with `check-fn.sh` specs (new
   `wordset.spec`, same style as `strbuf.spec`), stub the bank loaders.
3. **Word wrap / layout.** Use `nds_rom::gffont::GfFont::char_width` (A4b: font 0 = 12x15 for
   dialogue, 2 = 11x13, widths per glyph) behind a `trait GlyphWidth { fn width(&self, c: u16)
   -> u8 }` in `black-text` (so the crate stays free of `nds-rom`); the Bevy side (G5) implements it
   with `GfFont`. Algorithm: accumulate widths + letter spacing until the box width, break at the
   last space (`0x01DE`? UNVERIFIED code), force `FFFE` newlines, and paginate by the box line
   count with `BE00`/`BE01` wait/clear. The original does this incrementally inside `printsys.c`
   (render side, G5), so port only the deterministic measurement/pagination as pure functions and
   check against traced `printsys.c` behaviour (UNVERIFIED, no check-fn spec: it is render-bound).
4. **Text speed.** A 3-value setting (slow/mid/fast) -> frames per character; read from the
   `printsys` tick code (G5), exposed as `TextSpeed::frames_per_char`. UNVERIFIED values.

Order: 1 (tokens) -> 2 (wordset, specs first via the harness job) -> 3 -> 4.
| `strbuf.c` (`FUN_02045730`..`FUN_02045E3C`; see `strbuf/tests.rs` for the recorded area hash) | `strbuf` |
| `wordset.c` `FUN_0201ED50` | `wordset::entry_reset` |
| `FUN_0201ED64` | `wordset::clear_entry` |
| `FUN_0201ED84` | `wordset::set_entry` |
| `FUN_0201F31C` | `wordset::clear_all` |
| `FUN_0201F35C` | `wordset::format_number` |
| `FUN_0201F41C` | `wordset::copy_units` |
| `FUN_0201F448` | `wordset::units_eq` |
| `FUN_0201F47C` | `wordset::is_compressed` |
| `FUN_0201F4A0` | `wordset::append_word` |
| `FUN_0201F250` (`StringExpandPlaceholders`) | `wordset::expand` |

Deliberate deviations: the string buffer keeps its byte image (`{u16 capacity, u16 length, u32 magic,
u16 chars[]}`) so the ports run on the same bytes as the original; the end-of-string marker is the
constant `0xFFFF` instead of the global at `0x020A7E7C`; `WordSet`/`Entry` own their string buffers
instead of holding pointers. `wordset::copy_units` returns early for `n == 0` (the original writes
one unit before the destination; UNVERIFIED, not covered by the spec).

## F9 status and plan

Done: `strbuf` (F9-strbuf) and the `wordset` spec (22 functions, 10 smoke ports, hash
`8ed7e84a3aa08476`). Remaining for F9:

1. `MsgData` (`msgdata.c`, `FUN_02045B38` create / `FUN_02045CAC` get-string / `FUN_02045C04` delete /
   `FUN_02045F50` count): load a bank from `black-data`'s text JSON instead of an ARC; the rest of
   the pipeline only needs `get(id, &mut StrBuf)`. Put it behind a `MsgHooks` trait.
2. The `wordset` setters (`FUN_0201EDD0` and its ~20 constant-bank wrappers such as `0201F184`,
   `0201F198`, `0201F230`; `FUN_0201EE0C`, `FUN_0201EE60`, `FUN_0201EFA4`, `FUN_0201F02C`,
   `FUN_0201F048`, `FUN_0201F1E0`): all specced, they only need the `MsgData` hook. Name sources
   (trainer/player name, mon nickname, item/move/species names) are other systems: take them as
   `&[u16]` strings or as `hooks` traits.
3. Create / delete (`FUN_0201EC64` = `create(8, 0x20, heap)`, `FUN_0201EC74`, `FUN_0201ED04`) are
   allocator calls and become `WordSet::new(count, buf_len)` / `Drop`; not specced (the allocator
   stub returns one fixed pointer, so the several allocations would alias).
4. Word wrap and text speed: render side (`printsys.c`, `gf_font.c`, G5) behind a `GlyphWidth`
   trait; no ROM harness for the font data.

## Control-code format (from `wordset.c`, cross-checked with `nds_rom::msg`)

`black-text` does not depend on `nds-rom`; this is the shared format.

Strings are `u16` units terminated by `0xFFFF` (EOS). The engine treats these specially:

| unit | meaning |
|---|---|
| `0xF000` | start of a command (tag, below). `FUN_0201D5C4` returns this value. |
| `0xFFFE` | line break. The string-buffer and expansion code copy it verbatim; the text renderer interprets it. `nds_rom::msg::Token::NewLine`. |
| `0xFFFF` | end of string. |
| `0xF100` as unit 0 | the string is 9-bit compressed (below). `nds_rom::msg` calls this `compressed`. |

A tag is `{0xF000, word, n, args[n]}`: `word` is `group << 8 | cmd`, `n` the argument count, then `n`
`u16` arguments, so a tag is `3 + n` units. `nds_rom::msg::Token::Cmd { id, params }` is the same
thing with `id = word` and `params = args` (`n` is implicit). Helpers in the ROM (another file,
`0x0201D5C4..0x0201D688`): group = `word >> 8` (`FUN_0201D5F4`), cmd = `word & 0xFF`
(`FUN_0201D658`), argument i = unit `3 + i` (`FUN_0201D670`), end = `p + 3 + n` (`FUN_0201D688`),
build = `FUN_0201D60C(sb, group, cmd, n, args)`. All return neutral values (0xFF / 0 / `p`) when the
unit is not `0xF000`.

* **Word-set variable tags: groups 1 and 2** (`FUN_0201D5CC`). Argument 0 is the entry index of the
  word set (`WordSet.entries[idx]`, no bounds check in the original; scripts must stay below
  `count`). These match the A3 table: `0100` trainer/player name, `0101` species, `0102` nickname,
  `0103` type, `0105` place, `0107` move, `0109` item, `010B` box, `010D` stat, `010E` trainer
  class, `0110` title, `02xx` numbers. `StringExpandPlaceholders` (`FUN_0201F250`) replaces the
  whole tag by that entry's string.
* **Layout tags, group `0xBD`:** before substituting an entry whose flag byte (`info[4]`) is set, the
  expander appends `{F000, BD06, 1, idx}` and then `{F000, BD04, 1, 0x54}` to the entry's own
  string. These are the article/plural/gender markers the renderer reads (`nds_rom` A3 notes list
  `BDxx` as layout).
* **Every other tag is copied through unchanged** (`3 + n` units): `BE00` wait for button, `BE01`
  page clear, `BE02(n)` wait n frames, `FF00(n)` colour, and so on belong to the renderer (G5).
* Word-set entries also come from `FUN_0201F35C` (`format_number`): decimal with `digits` as the
  width (index into the power-of-ten table), leading zeros printed / space-padded / skipped by
  `mode` 2 / 1 / 0, `-` (`0x2D`, or full-width `0x30FC`), digits `0x30..0x39` or `0xFF10..0xFF19`.
  The `02xx` number tags select this.
* **9-bit compression** (`FUN_0201F47C` / `FUN_0201F4A0`): a word string whose first unit is
  `0xF100` (and length is nonzero) is a LSB-first bit stream of 9-bit codes in the units after the
  header; `0x1FF` ends it. At insertion time (`append_word`) codes below `0x100` are the character
  itself, `0x100..0x131` map through a 50-entry table (`0x152 0x153 0x15E 0x15F 0x2018 0x201C
  0x201D 0x201E 0x2026`, then `0x2460..=0x2487`, then `0xFF65`), anything higher becomes `'?'`.
  `nds_rom::msg` decompresses a bank entry to the raw codes and re-compresses them (it rejects
  `>= 0x1FF`), so it never applies this table: do the mapping when the string reaches the
  engine, as the original does.
* Message decryption (`strbuf::decrypt`, `FUN_02045E3C`): key `(id + 3) * 0x2983` rotated left by 3
  per unit, equal to `nds_rom::msg`'s `0x7C89 + 0x2983 * i`.

## F9c: tokens, compressed strings, layout, text speed

* `token` (`Token`, `parse`, `encode`): `Char(u16)`, `NewLine`, `Tag{id,args}`, `End`, `Truncated`.
  Tested against all 56,700 messages of the ROM banks `a/0/0/2` and `a/0/0/3` (dev-dependency on
  `nds-rom`; the tests skip when `black-rom/files` is absent, `BLACK_ROM_DIR` overrides):
  `encode(parse(s)) == s`, tag and line-break counts equal `nds_rom::msg`'s.
* `compress::decode`: 9-bit decode plus the 50-entry insertion table. Same algorithm as the
  ROM-verified `wordset::append_word` (`FUN_0201F4A0`); the tests check both agree on random streams
  and on the 589 compressed messages of the ROM, and the table against an independent list.
* `layout` (area `layout`): `line_width` = `FUN_0201D26C`, `max_line_width` = `FUN_0201D304`,
  verified with `tools/check-fn.sh black-text layout` (glyph advance `FUN_0201DC6C` stubbed behind
  `GlyphWidth`). The original never wraps automatically: lines break only at `0xFFFE`. `wrap`,
  `paginate`, `line_count` are UNVERIFIED conveniences, not ports.
* `speed` (area `speed`): table `0x0209DFB0` = `3 1 -2 -4 6`; `pace(v)` = `FUN_0201CDF4` (v >= 0:
  delay v, 1 char/tick; v < 0: -v chars/tick); option mappings `FUN_02013020`/`FUN_02013050`.
  Verified. `Pacer` and the slow/mid/fast naming of the options are UNVERIFIED.
