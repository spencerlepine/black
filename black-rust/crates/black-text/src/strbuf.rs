//! `strbuf.c` (F9): the string buffer `{u16 capacity, u16 length, u32 magic, u16 chars[]}` and the
//! message-string decryption of `msgdata.c`. Main ARM9 code (ARM mode), verified per function
//! with `tools/check-fn.sh black-text strbuf` (spec `tools/rom-diff/specs/black-text/strbuf.spec`).
//!
//! Deviation: the original reads the end-of-string marker from the global at `0x020A7E7C` (always
//! `0xFFFF` unless `set_eos` ran); the ports use [`EOS`] and `set_eos` only models the store.

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0204_5730,
    0x0204_5738,
    0x0204_5740,
    0x0204_5748,
    0x0204_5750,
    0x0204_5758,
    0x0204_5760,
    0x0204_5768,
    0x0204_5770,
    0x0204_5778,
    0x0204_5780,
    0x0204_5788,
    0x0204_5790,
    0x0204_5798,
    0x0204_57A0,
    0x0204_57B0,
    0x0204_5808,
    0x0204_581C,
    0x0204_5838,
    0x0204_5874,
    0x0204_589C,
    0x0204_58F8,
    0x0204_5900,
    0x0204_5924,
    0x0204_5A00,
    0x0204_5A38,
    0x0204_5A90,
    0x0204_5A98,
    0x0204_5AA8,
    0x0204_5AFC,
    0x0204_5CAC,
    0x0204_5E04,
    0x0204_5E3C,
];

use crate::hooks::{MsgHooks, StrHeap};

/// The end-of-string marker (the u16 at `0x020A7E7C` in the ROM image).
pub const EOS: u16 = 0xFFFF;
/// Offset of the first character.
pub const CHARS_OFS: usize = 8;

pub(crate) fn rd16(b: &[u8], ofs: usize) -> u16 {
    u16::from_le_bytes([b[ofs], b[ofs + 1]])
}

pub(crate) fn wr16(b: &mut [u8], ofs: usize, v: u16) {
    b[ofs..ofs + 2].copy_from_slice(&v.to_le_bytes());
}

pub(crate) fn ch(sb: &[u8], i: usize) -> u16 {
    rd16(sb, CHARS_OFS + 2 * i)
}

/// C: `FUN_020458F8`: the length in characters.
pub fn len(sb: &[u8]) -> u16 {
    rd16(sb, 2)
}

/// C: `FUN_0204581C`: empty the buffer (`length = 0`, `chars[0] = EOS`).
pub fn clear(sb: &mut [u8]) {
    wr16(sb, 2, 0);
    wr16(sb, CHARS_OFS, EOS);
}

/// C: `FUN_02045900`: cut the string to `i` characters if `i < length`.
pub fn truncate(sb: &mut [u8], i: u32) {
    if i < len(sb) as u32 {
        wr16(sb, CHARS_OFS + 2 * i as usize, EOS);
        wr16(sb, 2, i as u16);
    }
}

/// C: `FUN_02045AFC`: append one character if there is room (`length + 1 < capacity`, signed).
pub fn add_char(sb: &mut [u8], c: u16) {
    let n = len(sb) as i32;
    if n + 1 >= rd16(sb, 0) as i32 {
        return;
    }
    wr16(sb, 2, (n + 1) as u16);
    wr16(sb, CHARS_OFS + 2 * n as usize, c);
    wr16(sb, CHARS_OFS + 2 * (n as usize + 1), EOS);
}

/// C: `FUN_02045838`: copy `src` into `dst` (with its terminator) if `dst`'s capacity exceeds
/// `src`'s length.
pub fn copy(dst: &mut [u8], src: &[u8]) {
    let n = len(src) as usize;
    if rd16(dst, 0) as usize <= n {
        return;
    }
    let bytes = (n + 1) * 2;
    dst[CHARS_OFS..CHARS_OFS + bytes].copy_from_slice(&src[CHARS_OFS..CHARS_OFS + bytes]);
    wr16(dst, 2, n as u16);
}

/// C: `FUN_0204589C`: are the two strings equal up to the terminator.
pub fn eq(a: &[u8], b: &[u8]) -> bool {
    let mut i = 0;
    if ch(a, 0) != ch(b, 0) {
        return false;
    }
    loop {
        if ch(a, i) == EOS {
            return true;
        }
        i += 1;
        if ch(a, i) != ch(b, i) {
            return false;
        }
    }
}

/// C: `FUN_02045AA8`: append `src` (with its terminator) if the result fits the capacity.
pub fn append(dst: &mut [u8], src: &[u8]) {
    let (dl, sl) = (len(dst) as usize, len(src) as usize);
    if (dl + sl + 1) as i32 > rd16(dst, 0) as i32 {
        return;
    }
    let bytes = (sl + 1) * 2;
    let at = CHARS_OFS + 2 * dl;
    dst[at..at + bytes].copy_from_slice(&src[CHARS_OFS..CHARS_OFS + bytes]);
    wr16(dst, 2, (dl + sl) as u16);
}

/// C: `FUN_02045E3C`: decrypt `n` characters in place with the message key
/// (`key = (id + 3) * 0x2983`, 16 bits, rotated left by 3 per character); stops at an `EOS`
/// result and returns its index, else `n`.
pub fn decrypt(chars: &mut [u8], n: u32, id: u32) -> u32 {
    let mut key = (id.wrapping_add(3).wrapping_mul(0x2983) & 0xFFFF) as u16;
    for i in 0..n as usize {
        let c = rd16(chars, 2 * i) ^ key;
        wr16(chars, 2 * i, c);
        if c == EOS {
            return i as u32;
        }
        key = key.rotate_left(3);
    }
    n
}

fn rd32(b: &[u8], ofs: usize) -> u32 {
    u32::from_le_bytes(b[ofs..ofs + 4].try_into().unwrap())
}

// The text-parameter record accessors (`FUN_02045730..98`). UNVERIFIED: the struct name.

/// C: `FUN_02045730`: byte at +2.
pub fn rec_u8_2(r: &[u8]) -> u8 {
    r[2]
}
/// C: `FUN_02045738`: byte at +5.
pub fn rec_u8_5(r: &[u8]) -> u8 {
    r[5]
}
/// C: `FUN_02045740`: byte at +6.
pub fn rec_u8_6(r: &[u8]) -> u8 {
    r[6]
}
/// C: `FUN_02045748`: byte at +7.
pub fn rec_u8_7(r: &[u8]) -> u8 {
    r[7]
}
/// C: `FUN_02045750`: byte at +8.
pub fn rec_u8_8(r: &[u8]) -> u8 {
    r[8]
}
/// C: `FUN_02045758`: byte at +3.
pub fn rec_u8_3(r: &[u8]) -> u8 {
    r[3]
}
/// C: `FUN_02045760`: byte at +4.
pub fn rec_u8_4(r: &[u8]) -> u8 {
    r[4]
}
/// C: `FUN_02045768`: u16 at +0xA.
pub fn rec_u16_a(r: &[u8]) -> u16 {
    rd16(r, 0xA)
}
/// C: `FUN_02045770`: u32 at +0xC.
pub fn rec_u32_c(r: &[u8]) -> u32 {
    rd32(r, 0xC)
}
/// C: `FUN_02045778`: byte at +9.
pub fn rec_u8_9(r: &[u8]) -> u8 {
    r[9]
}
/// C: `FUN_02045780`: set the byte at +3.
pub fn set_rec_u8_3(r: &mut [u8], v: u8) {
    r[3] = v;
}
/// C: `FUN_02045788`: set the byte at +4.
pub fn set_rec_u8_4(r: &mut [u8], v: u8) {
    r[4] = v;
}
/// C: `FUN_02045790`: set the byte at +8.
pub fn set_rec_u8_8(r: &mut [u8], v: u8) {
    r[8] = v;
}
/// C: `FUN_02045798`: set the byte at +9.
pub fn set_rec_u8_9(r: &mut [u8], v: u8) {
    r[9] = v;
}

/// C: `FUN_020457A0`: store the end-of-string marker into its 4-byte global image
/// (`strh`: only the low half changes).
pub fn set_eos(global: &mut [u8], c: u16) {
    wr16(global, 0, c);
}

/// C: `FUN_020457B0`: allocate a buffer for `cap` characters in `heap` (`strbuf.c` line 100) and
/// initialise it (magic `0xB6F8D2EC`, empty). Returns its address.
pub fn create(cap: u32, heap: u32, h: &mut dyn StrHeap) -> u32 {
    let addr = h.alloc(heap, (cap << 1).wrapping_add(10), 0, 0x020A_7E80, 100);
    let sb = h.bytes(addr);
    sb[4..8].copy_from_slice(&0xB6F8_D2ECu32.to_le_bytes());
    wr16(sb, 0, cap as u16);
    wr16(sb, 2, 0);
    wr16(sb, CHARS_OFS, EOS);
    addr
}

/// C: `FUN_02045808`: clear the magic and free the buffer at `addr` (`sb` is its bytes).
pub fn free(sb: &mut [u8], addr: u32, h: &mut dyn StrHeap) {
    sb[4..8].copy_from_slice(&0u32.to_le_bytes());
    h.free(addr);
}

/// C: `FUN_02045874`: a new buffer holding a copy of `src`; returns its address.
pub fn dup(src: &[u8], heap: u32, h: &mut dyn StrHeap) -> u32 {
    let addr = create(len(src) as u32 + 1, heap, h);
    copy(h.bytes(addr), src);
    addr
}

/// C: `FUN_02045924`: copy characters from `src` up to its terminator, stopping when
/// `length >= capacity - 1` (signed); the result is terminated.
pub fn copy_chars(sb: &mut [u8], src: &[u8]) {
    wr16(sb, 2, 0);
    let mut i = 0;
    while rd16(src, 2 * i) != EOS {
        let n = rd16(sb, 2) as i32;
        if n >= rd16(sb, 0) as i32 - 1 {
            break;
        }
        wr16(sb, 2, (n + 1) as u16);
        wr16(sb, CHARS_OFS + 2 * n as usize, rd16(src, 2 * i));
        i += 1;
    }
    let n = len(sb) as usize;
    wr16(sb, CHARS_OFS + 2 * n, EOS);
}

/// C: `FUN_02045A00`: if `n != 0 && n <= capacity`, copy `n` characters from `src` and set
/// `length = n - 1` (the source carries its own terminator).
pub fn set_from(sb: &mut [u8], src: &[u8], n: u32) {
    if n == 0 || n > rd16(sb, 0) as u32 {
        return;
    }
    let bytes = 2 * n as usize;
    sb[CHARS_OFS..CHARS_OFS + bytes].copy_from_slice(&src[..bytes]);
    wr16(sb, 2, (n - 1) as u16);
}

/// C: `FUN_02045A38`: copy characters into `dst` up to `n` or the terminator (inclusive); if `n`
/// were copied, the last one becomes the terminator. `n == 0` is a no-op here (the original would
/// write before `dst`; UNVERIFIED, spec starts at 1).
pub fn copy_out(sb: &[u8], dst: &mut [u8], n: u32) {
    let n = n as usize;
    if n == 0 {
        return;
    }
    let mut i = 0;
    while i < n {
        let c = ch(sb, i);
        wr16(dst, 2 * i, c);
        if c == EOS {
            break;
        }
        i += 1;
    }
    if i == n {
        wr16(dst, 2 * (n - 1), EOS);
    }
}

/// C: `FUN_02045E04`: decrypt the buffer's characters in place with message id `id`, then cut at
/// the terminator found.
pub fn decrypt_buf(sb: &mut [u8], id: u32) {
    let n = len(sb) as u32 + 1;
    let at = decrypt(&mut sb[CHARS_OFS..], n, id);
    truncate(sb, at);
}

/// C: `FUN_02045CAC`, preloaded-table case: fetch message `id` into `sb`. `count` is the header's
/// entry count, `table` the preloaded data (entries `{u32 offset, u16 length, u16 ?}` from +4,
/// strings at `table + offset`); `msgdata` is the MsgData's address (for the reload hook).
/// The non-preloaded (file read) path is not ported.
pub fn msg_get(
    msgdata: u32,
    count: u16,
    table: &[u8],
    id: u32,
    sb: &mut [u8],
    h: &mut dyn MsgHooks,
) {
    if id >= count as u32 {
        clear(sb);
        return;
    }
    h.reload_language(msgdata);
    let e = 4 + 8 * id as usize;
    let (ofs, n) = (rd32(table, e) as usize, rd16(table, e + 4) as u32);
    set_from(sb, &table[ofs..], n);
    decrypt_buf(sb, id);
}

#[cfg(test)]
mod tests;
