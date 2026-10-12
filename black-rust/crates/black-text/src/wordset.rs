//! `wordset.c` (F9, `unk_0201EC64`): the word-set buffers that hold the values substituted into
//! message strings, and `StringExpandPlaceholders`. Main ARM9 Thumb code, verified per function
//! with `tools/check-fn.sh black-text wordset` (spec `tools/rom-diff/specs/black-text/wordset.spec`).
//!
//! This file holds the smoke ports that prove the spec; F9 ports the rest of the area (the `Set*`
//! setters funnel through `FUN_0201EDD0`/`FUN_0201ED84`, which need `MsgData`, a hook).
//!
//! In the original a word set is `{u32 count, u32 heap, Entry *entries, StrBuf *scratch}` (0x10
//! bytes) and an entry is `{u8 info[5], pad, pad, pad, StrBuf *str}` (12 bytes). Here the pointers
//! become owned values; the string buffers keep the `strbuf` byte image.

use crate::strbuf::{self, add_char, ch, clear, len, EOS};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0201_ED50,
    0x0201_ED64,
    0x0201_ED84,
    0x0201_EDD0,
    0x0201_EE0C,
    0x0201_EE60,
    0x0201_EFA4,
    0x0201_F02C,
    0x0201_F048,
    0x0201_F184,
    0x0201_F198,
    0x0201_F1B0,
    0x0201_F1C8,
    0x0201_F1E0,
    0x0201_F230,
    0x0201_F250,
    0x0201_F31C,
    0x0201_F35C,
    0x0201_F41C,
    0x0201_F448,
    0x0201_F47C,
    0x0201_F4A0,
];

/// Command-start unit of a message (`FUN_0201D5C4` returns it).
pub const TAG: u16 = 0xF000;
/// First unit of a 9-bit compressed string.
pub const COMPRESSED: u16 = 0xF100;
/// The 9-bit code that ends a compressed string.
pub const CODE_END: u16 = 0x1FF;

/// Characters for the 9-bit codes `0x100..0x132` (table at `0x0209E864`).
pub(crate) const CODE_TABLE: [u16; 50] = {
    let mut t = [0u16; 50];
    let head = [
        0x152, 0x153, 0x15E, 0x15F, 0x2018, 0x201C, 0x201D, 0x201E, 0x2026,
    ];
    let mut i = 0;
    while i < 9 {
        t[i] = head[i];
        i += 1;
    }
    while i < 49 {
        t[i] = 0x2460 + (i as u16 - 9);
        i += 1;
    }
    t[49] = 0xFF65;
    t
};

/// Powers of ten indexed by digit count (table at `0x0209E838`; entry 0 is as stored in the ROM).
const POW10: [u32; 11] = [
    1_000_000_000,
    1,
    10,
    100,
    1_000,
    10_000,
    100_000,
    1_000_000,
    10_000_000,
    100_000_000,
    1_000_000_000,
];
/// Digit characters, ASCII style (table at `0x0209E758`) and full-width (`0x0209E784`).
const DIGITS_ASCII: [u16; 11] = [
    0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3F,
];
const DIGITS_WIDE: [u16; 11] = [
    0xFF10, 0xFF11, 0xFF12, 0xFF13, 0xFF14, 0xFF15, 0xFF16, 0xFF17, 0xFF18, 0xFF19, 0xFF1F,
];

/// One word: five info bytes (`+4` is the flag byte read by [`expand`]) and its string buffer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub info: [u8; 5],
    /// A `strbuf` image.
    pub buf: Vec<u8>,
}

/// A word set. `entries` is the allocation; only the first `count` entries are in use by the
/// bounds-checked functions. `heap` is the allocator id (`+4`, read by the `MsgData` setters);
/// `scratch` is the `StrBuf` the setters fill before copying into an entry (`+0xC`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WordSet {
    pub count: u32,
    pub heap: u32,
    pub entries: Vec<Entry>,
    /// A `strbuf` image.
    pub scratch: Vec<u8>,
}

/// The callees of `wordset.c` that are not ported here, as trait methods (the harness routes them
/// to the spec's stub lines). The `MsgData` ones are `msgdata.c`; `FUN_02008500` is not
/// decompiled yet, so its behaviour is not modelled, only its call.
pub trait WordSetHooks {
    /// C: `FUN_02045B38(type, arc, bank, heap)`: the MsgData address, 0 when it cannot be opened.
    fn msg_create(&mut self, typ: u32, arc: u32, bank: u32, heap: u32) -> u32;
    /// C: `FUN_02045CAC(md, msgid, sb)`: the message `msgid` into `sb`.
    fn msg_fetch(&mut self, md: u32, msgid: u32, sb: &mut [u8]);
    /// C: `FUN_02045C04(md)`.
    fn msg_delete(&mut self, md: u32);
    /// C: `FUN_02045F50(md)`: the number of messages in the bank.
    fn msg_count(&mut self, md: u32) -> u32;
    /// C: `FUN_02008500(v, sb)`. UNVERIFIED: what it writes into `sb`.
    fn fill_02008500(&mut self, v: u32, sb: &mut [u8]);
}

/// C: `FUN_0201ED50`: zero the five info bytes of an entry.
pub fn entry_reset(info: &mut [u8; 5]) {
    *info = [0; 5];
}

/// C: `FUN_0201ED64`: reset entry `i`'s info bytes and empty its string (no bounds check).
pub fn clear_entry(ws: &mut WordSet, i: usize) {
    let e = &mut ws.entries[i];
    entry_reset(&mut e.info);
    clear(&mut e.buf);
}

/// C: `FUN_0201ED84`: if `i < count`, set entry `i`'s info bytes to `info` (zeros when `None`) and
/// copy `src` into its string (`strbuf::copy`).
pub fn set_entry(ws: &mut WordSet, i: usize, src: &[u8], info: Option<&[u8; 5]>) {
    if i as u32 >= ws.count {
        return;
    }
    let e = &mut ws.entries[i];
    match info {
        Some(b) => e.info = *b,
        None => entry_reset(&mut e.info),
    }
    strbuf::copy(&mut e.buf, src);
}

/// C: `FUN_0201F31C`: empty the string of every entry below `count` (info bytes untouched).
pub fn clear_all(ws: &mut WordSet) {
    for e in ws.entries.iter_mut().take(ws.count as usize) {
        clear(&mut e.buf);
    }
}

/// C: `FUN_0201F35C`: write `value` as decimal into `sb`. `digits` indexes [`POW10`] (the field
/// width is `digits`), `mode` 2 prints leading zeros, 1 pads them with a space, otherwise skips
/// them; the last digit is always printed. `ascii` picks ASCII-style digits, '-' and ' ' over the
/// full-width forms.
pub fn format_number(sb: &mut [u8], value: i32, digits: u32, mode: u32, ascii: bool) {
    let table = if ascii { &DIGITS_ASCII } else { &DIGITS_WIDE };
    clear(sb);
    let mut v = value as u32;
    if value < 0 {
        v = v.wrapping_neg();
        add_char(sb, if ascii { 0x2D } else { 0x30FC });
    }
    let mut mode = mode;
    let mut d = POW10[digits as usize];
    while d != 0 {
        let q = (v / d) & 0xFFFF;
        v = v.wrapping_sub(d.wrapping_mul(q));
        let q = q.min(10) as usize;
        if mode == 2 || q != 0 || d == 1 {
            mode = 2;
            add_char(sb, table[q]);
        } else if mode == 1 {
            add_char(sb, if ascii { 0x20 } else { 0x3000 });
        }
        d /= 10;
    }
}

/// Copy the scratch into entry `i` (`set_entry` with the scratch taken out of `ws` for the call).
fn set_from_scratch(ws: &mut WordSet, i: usize, info: Option<&[u8; 5]>) {
    let s = std::mem::take(&mut ws.scratch);
    set_entry(ws, i, &s, info);
    ws.scratch = s;
}

/// C: `FUN_0201EDD0(ws, i, bank, msgid)`: `md = MsgData_Create(0, 2, bank, heap)`; if `md` is
/// non-null, fetch `msgid` into the scratch, set entry `i` from it (info zeroed), delete `md`.
pub fn set_msg(ws: &mut WordSet, i: usize, bank: u32, msgid: u32, h: &mut dyn WordSetHooks) {
    let md = h.msg_create(0, 2, bank & 0xFFFF, ws.heap & 0xFFFF);
    if md != 0 {
        h.msg_fetch(md, msgid, &mut ws.scratch);
        set_from_scratch(ws, i, None);
        h.msg_delete(md);
    }
}

/// C: `FUN_0201EE0C(ws, i, src, n, plain)`: set entry `i` from `src` with info byte `+3` =
/// `(n & 0x7F) | (plain == 0) << 7` and the other info bytes zero.
pub fn set_entry_count(ws: &mut WordSet, i: usize, src: &[u8], n: u32, plain: u32) {
    let mut info = [0u8; 5];
    info[3] = (n as u8 & 0x7F) | if plain == 0 { 0x80 } else { 0 };
    set_entry(ws, i, src, Some(&info));
}

/// C: `sub_0201EE60(ws, i, msgid)`: fetch `msgid` from the MsgData `md` (the caller passes
/// `*0x020A7370`) into the scratch and set entry `i` from it (info zeroed).
pub fn set_msg_global(ws: &mut WordSet, i: usize, md: u32, msgid: u32, h: &mut dyn WordSetHooks) {
    h.msg_fetch(md, msgid, &mut ws.scratch);
    set_from_scratch(ws, i, None);
}

/// C: `FUN_0201EFA4(ws, i, value, digits, mode, ascii)`: format `value` into the scratch
/// (`format_number`) and set entry `i` from it.
pub fn set_number(ws: &mut WordSet, i: usize, value: i32, digits: u32, mode: u32, ascii: u32) {
    format_number(&mut ws.scratch, value, digits, mode, ascii != 0);
    set_from_scratch(ws, i, None);
}

/// C: `FUN_0201F02C(ws, i, v)`: `FUN_02008500(v, scratch)`, then set entry `i` from the scratch.
pub fn set_via_02008500(ws: &mut WordSet, i: usize, v: u32, h: &mut dyn WordSetHooks) {
    h.fill_02008500(v, &mut ws.scratch);
    set_from_scratch(ws, i, None);
}

/// C: `FUN_0201F048(ws, i, id)`: `md = MsgData_Create(0, 2, 0xE7, heap)`; if `md` is non-null:
/// fetch message `id` and set entry `i` from it when `id < count`, else clear entry `i`; delete `md`.
pub fn set_msg_bank_e7(ws: &mut WordSet, i: usize, id: u32, h: &mut dyn WordSetHooks) {
    let md = h.msg_create(0, 2, 0xE7, ws.heap & 0xFFFF);
    if md != 0 {
        if id < h.msg_count(md) {
            h.msg_fetch(md, id, &mut ws.scratch);
            set_from_scratch(ws, i, None);
        } else {
            clear_entry(ws, i);
        }
        h.msg_delete(md);
    }
}

/// C: `FUN_0201F184(ws, i, v)`: `set_msg(ws, i, 0xA0, v < 0x30 ? v : 0)` (signed compare).
pub fn set_msg_bank_a0(ws: &mut WordSet, i: usize, v: u32, h: &mut dyn WordSetHooks) {
    let msgid = if (v as i32) < 0x30 { v } else { 0 };
    set_msg(ws, i, 0xA0, msgid, h);
}

/// C: `FUN_0201F198(ws, i, v)`: `v > 8` clears entry `i`, else `set_msg(ws, i, 0x2B, v)`.
pub fn set_msg_bank_2b(ws: &mut WordSet, i: usize, v: u32, h: &mut dyn WordSetHooks) {
    if v > 8 {
        clear_entry(ws, i);
    } else {
        set_msg(ws, i, 0x2B, v, h);
    }
}

/// C: `FUN_0201F1B0(ws, i, v)`: `v > 0x91` clears entry `i`, else `set_msg(ws, i, 0, v)`.
pub fn set_msg_bank_0(ws: &mut WordSet, i: usize, v: u32, h: &mut dyn WordSetHooks) {
    if v > 0x91 {
        clear_entry(ws, i);
    } else {
        set_msg(ws, i, 0, v, h);
    }
}

/// C: `FUN_0201F1C8(ws, i, v)`: `v > 8` clears entry `i`, else `set_msg(ws, i, 0x38, v)`.
pub fn set_msg_bank_38(ws: &mut WordSet, i: usize, v: u32, h: &mut dyn WordSetHooks) {
    if v > 8 {
        clear_entry(ws, i);
    } else {
        set_msg(ws, i, 0x38, v, h);
    }
}

/// C: `FUN_0201F230(ws, i, v)`: `set_msg(ws, i, 278, clamp(v, 1, 12) - 1)` (unsigned clamp).
pub fn set_msg_bank_278(ws: &mut WordSet, i: usize, v: u32, h: &mut dyn WordSetHooks) {
    set_msg(ws, i, 278, v.clamp(1, 12) - 1, h);
}

/// C: `FUN_0201F1E0(ws, i, v)` (unlabeled): `md = MsgData_Create(0, 3, 461, heap)`; clear entry
/// `i`; if `v <= 6` and `md` is non-null, fetch message `v + 0x2B` and set entry `i` from it, then
/// delete `md` (the delete is skipped for `v > 6`, as in the original).
pub fn set_msg_bank_461(ws: &mut WordSet, i: usize, v: u32, h: &mut dyn WordSetHooks) {
    let md = h.msg_create(0, 3, 461, ws.heap & 0xFFFF);
    clear_entry(ws, i);
    if v <= 6 && md != 0 {
        h.msg_fetch(md, v + 0x2B, &mut ws.scratch);
        set_from_scratch(ws, i, None);
        h.msg_delete(md);
    }
}

/// C: `FUN_0201F41C`: copy `n` units (a raw `u16` string, no header) from `src` to `dst`, stopping
/// after an `EOS`, then store `EOS` in the last of the `n` units. UNVERIFIED: `n == 0` (the
/// original writes one unit before `dst`; the spec uses `n >= 1`).
pub fn copy_units(src: &[u8], dst: &mut [u8], n: u32) {
    let n = n as usize;
    if n == 0 {
        return;
    }
    for i in 0..n {
        let c = strbuf::rd16(src, 2 * i);
        strbuf::wr16(dst, 2 * i, c);
        if c == EOS {
            break;
        }
    }
    strbuf::wr16(dst, 2 * (n - 1), EOS);
}

/// C: `FUN_0201F448`: are two raw `u16` strings (no header) equal up to the terminator.
pub fn units_eq(a: &[u8], b: &[u8]) -> bool {
    let mut i = 0;
    if strbuf::rd16(a, 0) != EOS {
        loop {
            if strbuf::rd16(a, 2 * i) != strbuf::rd16(b, 2 * i) {
                return false;
            }
            i += 1;
            if strbuf::rd16(a, 2 * i) == EOS {
                break;
            }
        }
    }
    strbuf::rd16(b, 2 * i) == EOS
}

/// C: `FUN_0201F47C`: is the string 9-bit compressed (non-empty and starting with `0xF100`).
pub fn is_compressed(sb: &[u8]) -> bool {
    len(sb) != 0 && ch(sb, 0) == COMPRESSED
}

/// C: `FUN_0201F4A0`: append the word string `src` to `dst`. A compressed string is unpacked:
/// 9-bit codes from the units after the header, LSB first; `0x1FF` ends, below `0x100` is the
/// character itself, `0x100..0x132` maps through [`CODE_TABLE`], anything higher becomes `'?'`.
pub fn append_word(dst: &mut [u8], src: &[u8]) {
    if !is_compressed(src) {
        strbuf::append(dst, src);
        return;
    }
    let mut p = 1usize;
    let mut bit = 0u32;
    loop {
        let mut code = (ch(src, p) as u32 >> bit) & 0x1FF;
        bit += 9;
        if bit >= 16 {
            p += 1;
            bit -= 16;
            if bit != 0 {
                code |= ((ch(src, p) as u32) << (9 - bit)) & 0x1FF;
            }
        }
        if code == CODE_END as u32 {
            return;
        }
        let c = if code >= 0x132 {
            0x3F
        } else if code >= 0x100 {
            CODE_TABLE[code as usize - 0x100]
        } else {
            code as u16
        };
        add_char(dst, c);
    }
}

// ---- message tag helpers (`0x0201D5C4..0x0201D688`, another file; private, exercised through `expand`)

/// A tag is `{0xF000, group << 8 | cmd, n, args[n]}`: `FUN_0201D5CC`, groups 1 and 2 are
/// word-set variables.
fn tag_is_variable(s: &[u8], p: usize) -> bool {
    let group = if ch(s, p) == TAG {
        ch(s, p + 1) >> 8
    } else {
        0xFF
    };
    group == 1 || group == 2
}

/// `FUN_0201D670(tag, 0)`: the first argument.
fn tag_arg0(s: &[u8], p: usize) -> u16 {
    if ch(s, p) == TAG {
        ch(s, p + 3)
    } else {
        0
    }
}

/// `FUN_0201D688`: the unit after the tag (`p` itself when `p` is not a tag).
fn tag_end(s: &[u8], p: usize) -> usize {
    if ch(s, p) == TAG {
        p + 3 + ch(s, p + 2) as usize
    } else {
        p
    }
}

/// `FUN_0201D60C(sb, group, cmd, n, args)`: append a tag.
fn add_tag(sb: &mut [u8], group: u16, cmd: u16, args: &[u16]) {
    add_char(sb, TAG);
    add_char(sb, group << 8 | cmd);
    add_char(sb, args.len() as u16);
    for &a in args {
        add_char(sb, a);
    }
}

/// C: `FUN_0201F250` (`StringExpandPlaceholders`): empty `dst`, then copy `src` into it, replacing
/// every variable tag (group 1 or 2) by the string of word-set entry `arg0` (no bounds check; an
/// entry whose flag byte `info[4]` is set first gets two layout tags `{BD06, idx}` and `{BD04, 0x54}`
/// appended to its own string). Other tags are copied through unchanged.
pub fn expand(ws: &mut WordSet, dst: &mut [u8], src: &[u8]) {
    clear(dst);
    let mut p = 0usize;
    while ch(src, p) != EOS {
        let c = ch(src, p);
        if c != TAG {
            add_char(dst, c);
            p += 1;
        } else if tag_is_variable(src, p) {
            let idx = tag_arg0(src, p) as usize;
            let e = &mut ws.entries[idx];
            if e.info[4] != 0 {
                add_tag(&mut e.buf, 0xBD, 6, &[idx as u16]);
                add_tag(&mut e.buf, 0xBD, 4, &[0x54]);
            }
            append_word(dst, &e.buf);
            p = tag_end(src, p);
        } else {
            let end = tag_end(src, p);
            for q in p..end {
                add_char(dst, ch(src, q));
            }
            p = end;
        }
    }
}

#[cfg(test)]
mod tests;
