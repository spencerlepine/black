//! Server command queue (0x021CC8D8-0x021CD6F0): packed-word helpers, byte/word queue push/pop, command write/read by format, record allocator (B10a).

use super::env::{Arg, FlowEnv};

/// Functions of this area that are ported and checked by `tests::svflow_match_rom`.
pub const PORTED: &[u32] = &[
    0x021CC8D8, 0x021CC918, 0x021CC95C, 0x021CC960, 0x021CC96C, 0x021CC974, 0x021CC99C, 0x021CCAA8,
    0x021CCAD4, 0x021CCAE0, 0x021CCB18, 0x021CCB78, 0x021CD4B0, 0x021CC9A4, 0x021CC9AC, 0x021CCA74,
    0x021CCB34, 0x021CCB98, 0x021CCBE8, 0x021CCC0C, 0x021CD0F0, 0x021CD4FC, 0x021CD568, 0x021CD5E0,
    0x021CD64C, 0x021CD6AC, 0x021CD6EC,
];

/// Byte capacity of a command buffer: the write count at +0 must stay below this
/// (literal at 0x021CCACC).
pub const QUEUE_CAP: u32 = 0xBB8;

/// Write count at which `push_u16` calls the fatal routine: two bytes must still fit
/// (literal at 0x021CCB10).
pub const QUEUE_CAP_U16: u32 = 0xBB7;

/// Fatal-error routine the buffer writers call on overflow; never returns.
const FATAL: u32 = 0x0203_4714;
/// Its first and third argument at the overflow call sites (literal at 0x021CCAD0).
const FATAL_ARG: u32 = 0x021F_3A84;

/// C: `FUN_overlay_d_93__021cc95c`: stores a 32-bit word (`*p = v`).
pub fn set_word(p: &mut [u8], v: u32) {
    p[..4].copy_from_slice(&v.to_le_bytes());
}

/// C: `FUN_overlay_d_93__021ccaa8`: appends one byte to a command buffer (count at +0, bytes
/// from +8); calls the fatal-error routine first when the buffer already holds `QUEUE_CAP`
/// bytes, after which it must not write.
pub fn push_u8<E: FlowEnv>(env: &mut E, q: &mut [u8], v: u8) {
    let count = u32::from_le_bytes(q[..4].try_into().unwrap());
    if count >= QUEUE_CAP {
        env.call(
            FATAL,
            &mut [Arg::V(FATAL_ARG), Arg::V(0), Arg::V(FATAL_ARG)],
        );
        return;
    }
    q[..4].copy_from_slice(&(count + 1).to_le_bytes());
    q[8 + count as usize] = v;
}

fn rd32(p: &[u8]) -> u32 {
    u32::from_le_bytes(p[..4].try_into().unwrap())
}

fn wr32(p: &mut [u8], v: u32) {
    p[..4].copy_from_slice(&v.to_le_bytes());
}

/// Bits 0-9 of the packed word handled by `021cc8d8`..`021cc9a4`.
const WORD_LOW10: u32 = 0x3FF;
/// Bit 30 (set by `set_mode_flag30`) and bit 29 / bits 28-29 (the mode).
const WORD_FLAG30: u32 = 1 << 30;

/// Shared tail of `advance_fields` / `advance_fields_set_low10`: copies bits 10-18 of `u` up to
/// bits 19-27 and clears the flag bits 28, 29 and 30.
fn advance_fields_word(u: u32) -> u32 {
    let field = ((u & 0x7FFFF) >> 10) << 19;
    ((u & 0xF007_FFFF) | field) & 0xDFFF_FFFF & 0xEFFF_FFFF & 0xBFFF_FFFF
}

/// C: `FUN_overlay_d_93__021cc8d8`: updates a packed 32-bit word in place and returns the old
/// value: bits 0-9 are cleared, bits 10-18 are copied to bits 19-27, bits 28-30 are cleared.
pub fn advance_fields(p: &mut [u8]) -> u32 {
    let old = rd32(p);
    wr32(p, advance_fields_word(old & 0xFFFF_FC00));
    old
}

/// C: `FUN_overlay_d_93__021cc918`: as `advance_fields`, but bits 0-9 become `v & 0x3FF`.
pub fn advance_fields_set_low10(p: &mut [u8], v: u32) -> u32 {
    let old = rd32(p);
    let u = (old & 0xFFFF_FC00) | (v & WORD_LOW10);
    wr32(p, advance_fields_word(u));
    old
}

/// C: `FUN_overlay_d_93__021cc960`: bits 0-9 of the packed word.
pub fn word_low10(p: &[u8]) -> u32 {
    rd32(p) & WORD_LOW10
}

/// C: `FUN_overlay_d_93__021cc96c`: bit 30 of the packed word.
pub fn word_flag30(p: &[u8]) -> u32 {
    (rd32(p) & 0x7FFF_FFFF) >> 30
}

/// C: `FUN_overlay_d_93__021cc974`: `flag == 0` clears bit 28, otherwise sets bits 28 and 29;
/// then always sets bit 30.
pub fn set_mode_flag30(p: &mut [u8], flag: u32) {
    let w = rd32(p);
    let w = if flag == 0 {
        w & 0xEFFF_FFFF
    } else {
        w | 0x3000_0000
    };
    wr32(p, w | WORD_FLAG30);
}

/// C: `FUN_overlay_d_93__021cc99c`: bits 28-29 of the packed word.
pub fn word_bits28_29(p: &[u8]) -> u32 {
    (rd32(p) & 0x1FFF_FFFF) >> 28
}

/// C: `FUN_overlay_d_93__021ccad4`: reads the byte at the read cursor (+4) of a command buffer
/// (bytes from +8) and advances the cursor.
pub fn pop_u8(q: &mut [u8]) -> u8 {
    let cur = rd32(&q[4..]);
    wr32(&mut q[4..], cur.wrapping_add(1));
    q[8 + cur as usize]
}

/// C: `FUN_overlay_d_93__021ccae0`: appends a 16-bit value, high byte first; calls the
/// fatal-error routine first when the write count is at least `QUEUE_CAP_U16`, after which it
/// must not write.
pub fn push_u16<E: FlowEnv>(env: &mut E, q: &mut [u8], v: u16) {
    let count = rd32(q);
    if count >= QUEUE_CAP_U16 {
        env.call(
            FATAL,
            &mut [Arg::V(FATAL_ARG), Arg::V(0), Arg::V(FATAL_ARG)],
        );
        return;
    }
    q[8 + count as usize] = (v >> 8) as u8;
    q[9 + count as usize] = v as u8;
    wr32(q, count + 2);
}

/// C: `FUN_overlay_d_93__021ccb18`: reads a 16-bit value (high byte first) at the read cursor
/// and advances it by 2.
pub fn pop_u16(q: &mut [u8]) -> u16 {
    let cur = rd32(&q[4..]);
    let i = 8 + cur as usize;
    let v = (q[i] as u16) << 8 | q[i + 1] as u16;
    wr32(&mut q[4..], cur.wrapping_add(2));
    v
}

/// C: `FUN_overlay_d_93__021ccb78`: reads a 24-bit value (high byte first) at the read cursor
/// and advances it by 3.
pub fn pop_u24(q: &mut [u8]) -> u32 {
    let cur = rd32(&q[4..]);
    let i = 8 + cur as usize;
    let v = (q[i] as u32) << 16 | (q[i + 1] as u32) << 8 | q[i + 2] as u32;
    wr32(&mut q[4..], cur.wrapping_add(3));
    v
}

/// Per-command byte at 0x021EFF14, indexed by command ID: the argument-encoding format that
/// `021ccc0c` (write) and `021cd0f0` (read) switch on; its low nibble is the number of argument
/// words `put_cmd` copies. The table's end is not known (commands up to 0x5F are used by
/// `021cd4fc`); 0x80 bytes are kept so every index below 0x80 reads what the original reads.
pub const CMD_ARG_TABLE: [u8; 0x80] = [
    0x00, 0x12, 0x12, 0x01, 0x03, 0x03, 0x42, 0x03, 0x03, 0x03, 0x03, 0x08, 0x01, 0x01, 0x22, 0x53,
    0x01, 0x12, 0x04, 0x33, 0x12, 0x53, 0x12, 0x12, 0x26, 0x02, 0x02, 0x02, 0x02, 0x12, 0x12, 0x15,
    0x01, 0x22, 0x02, 0x01, 0x01, 0x33, 0x02, 0x12, 0x01, 0x32, 0x01, 0x01, 0x02, 0x26, 0x01, 0x01,
    0x24, 0x42, 0x43, 0x43, 0x01, 0x02, 0x01, 0x23, 0x23, 0x01, 0x01, 0x02, 0x12, 0x14, 0x07, 0x02,
    0x01, 0x01, 0x01, 0x42, 0x23, 0x22, 0x16, 0x12, 0x32, 0x12, 0x54, 0x01, 0x11, 0x12, 0x43, 0x02,
    0x14, 0x01, 0x01, 0x02, 0x13, 0x11, 0x01, 0x01, 0x01, 0x12, 0x00, 0x00, 0x10, 0x10, 0x00, 0x00,
    0x06, 0x04, 0x02, 0x01, 0x12, 0x00, 0x11, 0x00, 0x13, 0x00, 0x14, 0x00, 0x3D, 0x00, 0x3E, 0x00,
    0x3F, 0x00, 0x40, 0x00, 0x0D, 0x00, 0x0E, 0x00, 0x0F, 0x00, 0x10, 0x00, 0x11, 0x00, 0x12, 0x00,
];

/// Number of `(id, size)` pairs in `RECORD_SIZE_TABLE`.
const RECORD_COUNT: usize = 0x3B;
/// Offsets (record bytes from +4) at or past this are refused by `alloc_record`.
const RECORD_AREA_LIMIT: u32 = 0x1F5;
/// Terminator of the argument lists of `put_cmd_list` / `read_cmd_list`.
const LIST_END: u32 = 0xFFFF_0000;

/// `(id, size in bytes)` pairs at 0x021EFE9C, used by `alloc_record`.
pub const RECORD_SIZE_TABLE: [u8; RECORD_COUNT * 2] = [
    0x00, 0x08, 0x01, 0x04, 0x05, 0x2C, 0x06, 0x2C, 0x08, 0x28, 0x09, 0x2C, 0x0A, 0x2C, 0x0B, 0x3C,
    0x0C, 0x38, 0x0E, 0x3C, 0x0F, 0x0C, 0x12, 0x08, 0x10, 0x0C, 0x11, 0x34, 0x07, 0x30, 0x13, 0x2C,
    0x14, 0x08, 0x04, 0x28, 0x02, 0x04, 0x03, 0x04, 0x15, 0x0C, 0x16, 0x0C, 0x17, 0x0C, 0x18, 0x0C,
    0x19, 0x34, 0x1A, 0x08, 0x1B, 0x34, 0x1D, 0x2C, 0x1C, 0x08, 0x1E, 0x20, 0x1F, 0x30, 0x20, 0x30,
    0x21, 0x0C, 0x22, 0x08, 0x23, 0x2C, 0x24, 0x74, 0x25, 0x0C, 0x26, 0x08, 0x27, 0x08, 0x28, 0x04,
    0x29, 0x50, 0x2A, 0x08, 0x2B, 0x08, 0x2C, 0x2C, 0x2D, 0x2C, 0x2E, 0x2C, 0x2F, 0x2C, 0x30, 0x08,
    0x31, 0x2C, 0x32, 0x2C, 0x33, 0x2C, 0x34, 0x2C, 0x35, 0x04, 0x36, 0x30, 0x37, 0x30, 0x38, 0x04,
    0x39, 0x2C, 0x3A, 0x08, 0x3B, 0x04,
];

fn set_out(out: &mut [u8], i: usize, v: u32) {
    out[4 * i..4 * i + 4].copy_from_slice(&v.to_le_bytes());
}

fn get_arg(args: &[u8], i: usize) -> u32 {
    rd32(&args[4 * i..])
}

/// C: `FUN_overlay_d_93__021cc9a4`: bit 29 of the packed word.
pub fn word_flag29(p: &[u8]) -> u32 {
    (rd32(p) & 0x3FFF_FFFF) >> 29
}

/// C: `sub_021cc9ac`: allocates a record of kind `id` in a buffer whose first word holds the
/// used size (bits 10-18) and whose records start at +4. The size comes from
/// `RECORD_SIZE_TABLE` (rounded up to a multiple of 4); the record is zeroed and gets a header
/// word (id in bits 0-7, `v & 0x1F` in bits 8-12, size in bits 13-22, bit 26). Returns the
/// record's offset from `p`, or 0 when the id is unknown, has size 0 or the record does not fit.
pub fn alloc_record(p: &mut [u8], id: u32, v: u32) -> u32 {
    let mut size = 0u32;
    for i in 0..RECORD_COUNT {
        if id == RECORD_SIZE_TABLE[i * 2] as u32 {
            size = RECORD_SIZE_TABLE[i * 2 + 1] as u32;
            break;
        }
    }
    if size == 0 {
        return 0;
    }
    while size & 3 != 0 {
        size += 1;
    }
    let off = (rd32(p) & 0x7FFFF) >> 10;
    if off + size >= RECORD_AREA_LIMIT {
        return 0;
    }
    let at = 4 + off as usize;
    p[at..at + size as usize].fill(0);
    let body = ((id & 0xFF) | (rd32(&p[at..]) & 0xFFFF_FF00)) & 0xFF80_1FFF | (size & 0x3FF) << 13;
    let hdr = ((v & 0x1F) << 8 | 0xFFFF_E0FF & body) & 0xFF7F_FFFF | 0x0400_0000;
    wr32(&mut p[at..], hdr);
    let w = rd32(p);
    wr32(
        p,
        (((((w & 0x7FFFF) >> 10) + size) & 0x1FF) << 10) | (0xFFF8_03FF & w),
    );
    at as u32
}

/// C: `FUN_overlay_d_93__021cca74`: releases the record at offset `rec` from `p` if it is the
/// last one allocated (its end equals the used size), by shrinking the used size.
pub fn free_last_record(p: &mut [u8], rec: u32) {
    let used = (rd32(p) & 0x7FFFF) >> 10;
    let size = (rd32(&p[rec as usize..]) & 0x7F_FFFF) >> 13;
    if size <= used && rec.wrapping_sub(4).wrapping_add(size) == used {
        let w = rd32(p);
        wr32(p, ((used - size) & 0x1FF) << 10 | (w & 0xFFF8_03FF));
    }
}

/// Shared prologue of `push_u24` / `push_u32`: the fatal-error call when the write count is at
/// least `cap`. Returns true when the caller must not write.
fn queue_full<E: FlowEnv>(env: &mut E, q: &[u8], cap: u32) -> bool {
    if rd32(q) >= cap {
        env.call(
            FATAL,
            &mut [Arg::V(FATAL_ARG), Arg::V(0), Arg::V(FATAL_ARG)],
        );
        return true;
    }
    false
}

/// C: `FUN_overlay_d_93__021ccb34`: appends a 24-bit value, high byte first; fatal-error call
/// first when the write count is at least 0xBB6.
pub fn push_u24<E: FlowEnv>(env: &mut E, q: &mut [u8], v: u32) {
    if queue_full(env, q, 0xBB6) {
        return;
    }
    let count = rd32(q);
    q[8 + count as usize] = (v >> 16) as u8;
    q[9 + count as usize] = (v >> 8) as u8;
    q[10 + count as usize] = v as u8;
    wr32(q, count + 3);
}

/// C: `FUN_overlay_d_93__021ccb98`: appends a 32-bit value, high byte first; fatal-error call
/// first when the write count is at least 0xBB5.
pub fn push_u32<E: FlowEnv>(env: &mut E, q: &mut [u8], v: u32) {
    if queue_full(env, q, 0xBB5) {
        return;
    }
    let count = rd32(q);
    q[8 + count as usize] = (v >> 24) as u8;
    q[9 + count as usize] = (v >> 16) as u8;
    q[10 + count as usize] = (v >> 8) as u8;
    q[11 + count as usize] = v as u8;
    wr32(q, count + 4);
}

/// C: `FUN_overlay_d_93__021ccbe8`: reads a 32-bit value (high byte first) at the read cursor
/// and advances it by 4.
pub fn pop_u32(q: &mut [u8]) -> u32 {
    let cur = rd32(&q[4..]);
    let i = 8 + cur as usize;
    let v = u32::from_be_bytes([q[i], q[i + 1], q[i + 2], q[i + 3]]);
    wr32(&mut q[4..], cur.wrapping_add(4));
    v
}

/// C: `FUN_overlay_d_93__021cd6ec`: empty function (`bx lr`).
pub fn nop_021cd6ec() {}

/// C: `FUN_overlay_d_93__021ccc0c`: writes command `cmd` into the buffer: the ID as a 16-bit
/// value, then its arguments (the words of `args`, at least as many as the format uses)
/// packed as `fmt` (the command's `CMD_ARG_TABLE` byte) says. Formats the switch does not list
/// write the ID only.
pub fn write_cmd<E: FlowEnv>(env: &mut E, q: &mut [u8], cmd: u32, fmt: u32, args: &[u8]) {
    let a = |i: usize| get_arg(args, i);
    push_u16(env, q, cmd as u16);
    match fmt {
        0x54 => {
            push_u8(env, q, a(0) as u8);
            push_u8(env, q, a(1) as u8);
            push_u16(env, q, a(2) as u16);
            push_u16(env, q, a(3) as u16);
        }
        0x53 => {
            push_u8(env, q, a(0) as u8);
            push_u8(env, q, a(1) as u8);
            push_u32(env, q, a(2));
        }
        0x44 => {
            let v = (a(1) & 0x1F) << 6 | (a(0) & 0x1F) << 11 | a(2) & 0x3F;
            push_u16(env, q, v as u16);
            push_u16(env, q, a(3) as u16);
        }
        0x43 => {
            push_u8(env, q, a(0) as u8);
            push_u8(env, q, a(1) as u8);
            push_u16(env, q, a(2) as u16);
        }
        0x42 => push_u8(env, q, (a(1) & 7 | (a(0) & 0x1F) << 3) as u8),
        1 => push_u8(env, q, a(0) as u8),
        2 => {
            push_u8(env, q, a(0) as u8);
            push_u8(env, q, a(1) as u8);
        }
        3 => {
            push_u8(env, q, (a(1) & 7 | (a(0) & 0x1F) << 3) as u8);
            push_u8(env, q, a(2) as u8);
        }
        4 => {
            push_u8(env, q, (a(1) & 7 | (a(0) & 0x1F) << 3) as u8);
            push_u8(env, q, a(2) as u8);
            push_u16(env, q, a(3) as u16);
        }
        5 => {
            let v = (a(1) & 0x1F) << 5 | (a(0) & 0x1F) << 10 | a(2) & 0x1F;
            push_u16(env, q, v as u16);
            push_u16(env, q, a(3) as u16);
            push_u16(env, q, a(4) as u16);
        }
        6 => {
            let v = (a(1) & 0x1F) << 5 | (a(0) & 0x1F) << 10 | a(2) & 0x1F;
            push_u16(env, q, v as u16);
            let w = (a(4) & 0x1F) << 5 | (a(3) & 0x1F) << 10 | a(5) & 0x1F;
            push_u16(env, q, w as u16);
        }
        7 | 8 => {
            for i in 0..fmt as usize {
                push_u8(env, q, a(i) as u8);
            }
        }
        0x11 => push_u16(env, q, a(0) as u16),
        0x12 => {
            push_u8(env, q, a(0) as u8);
            push_u16(env, q, a(1) as u16);
        }
        0x13 => {
            push_u8(env, q, (a(1) & 7 | (a(0) & 0x1F) << 3) as u8);
            push_u16(env, q, a(2) as u16);
        }
        0x14 => {
            push_u8(env, q, (a(1) & 7 | (a(0) & 0x1F) << 3) as u8);
            push_u8(env, q, (a(3) & 7 | (a(2) & 0x1F) << 3) as u8);
        }
        0x15 => {
            push_u16(env, q, ((a(0) & 0x1F) << 3 | a(1) & 7) as u16);
            push_u16(env, q, (a(3) & 1 | (a(2) & 0x7F) << 1) as u16);
            push_u16(env, q, a(4) as u16);
        }
        0x16 => {
            let v =
                (a(2) & 1) << 2 | (a(1) & 3) << 3 | (a(0) & 7) << 5 | (a(3) & 1) << 1 | a(4) & 1;
            push_u8(env, q, v as u8);
            push_u16(env, q, a(5) as u16);
        }
        0x22 => {
            push_u8(env, q, a(0) as u8);
            push_u32(env, q, a(1));
        }
        0x23 => {
            let v = a(2) & 0x1F | (a(0) & 0x1F) << 10 | (a(1) & 0x1F) << 5;
            push_u16(env, q, v as u16);
        }
        0x24 => {
            push_u24(
                env,
                q,
                (a(1) & 0x1F) << 14 | (a(0) & 0x1F) << 19 | a(2) & 0x3FFF,
            );
            push_u8(env, q, a(3) as u8);
        }
        0x25 => {
            let v = (a(0) & 7) << 5 | (a(1) & 7) << 2 | (a(2) & 1) << 1 | a(3) & 1;
            push_u8(env, q, v as u8);
            push_u16(env, q, a(4) as u16);
        }
        0x26 => {
            let v = (a(0) & 0x1F) << 15 | (a(1) & 0x1F) << 10 | (a(2) & 0x1F) << 5 | a(3) & 0x1F;
            push_u24(env, q, v);
            push_u16(env, q, a(4) as u16);
            push_u16(env, q, a(5) as u16);
        }
        0x32 => push_u8(env, q, (a(1) & 0xF | (a(0) & 0xF) << 4) as u8),
        0x33 => push_u24(
            env,
            q,
            a(2) & 0x3FFF | (a(0) & 0x1F) << 19 | (a(1) & 0x1F) << 14,
        ),
        0x34 => {
            push_u24(
                env,
                q,
                (a(1) & 0x1F) << 14 | (a(0) & 0x1F) << 19 | a(2) & 0x3FFF,
            );
            push_u16(env, q, a(3) as u16);
        }
        _ => {}
    }
}

/// C: `FUN_overlay_d_93__021cd4b0`: puts command `cmd`: copies as many of `args` (the call's
/// third argument onward; the original also reads past the register arguments on the caller's
/// stack) as the low nibble of its table byte says into the scratch words at 0x021F3B44
/// (`scratch`), then writes the command to `q` with `write_cmd`. `args` must hold at least that
/// many words.
pub fn put_cmd<E: FlowEnv>(env: &mut E, q: &mut [u8], cmd: u32, args: &[u32], scratch: &mut [u8]) {
    let flags = CMD_ARG_TABLE[cmd as usize];
    let n = (flags & 0xF) as usize;
    for (i, a) in args[..n].iter().enumerate() {
        scratch[4 * i..4 * i + 4].copy_from_slice(&a.to_le_bytes());
    }
    write_cmd(env, q, cmd, flags as u32, scratch);
}

/// C: `FUN_overlay_d_93__021cd4fc`: puts command `cmd` with all-zero arguments (a placeholder
/// that `put_cmd_at` can overwrite later): after the command is written, the bytes since the
/// write count at entry are re-wrapped as a skip record (command 0x5F plus a length byte).
/// Returns the offset where the command starts.
pub fn put_cmd_placeholder<E: FlowEnv>(
    env: &mut E,
    q: &mut [u8],
    cmd: u32,
    scratch: &mut [u8],
) -> u32 {
    let flags = CMD_ARG_TABLE[cmd as usize];
    let n = (flags & 0xF) as usize;
    for i in 0..n {
        set_out(scratch, i, 0);
    }
    let start = rd32(q) & 0xFFFF;
    write_cmd(env, q, cmd, flags as u32, scratch);
    let len = rd32(q).wrapping_sub(start) & 0xFF;
    wr32(q, start);
    push_u16(env, q, 0x5F);
    push_u8(env, q, len.wrapping_sub(3) as u8);
    wr32(q, start + len);
    start
}

/// C: `FUN_overlay_d_93__021cd568`: rewrites the command at write position `pos` in place:
/// skips the u16 ID and u8 length found there, then (unless `cmd` is 0x5F) writes `cmd` with
/// `args` (the call's fourth argument onward) at `pos` and restores the write count.
pub fn put_cmd_at<E: FlowEnv>(
    env: &mut E,
    q: &mut [u8],
    pos: u32,
    cmd: u32,
    args: &[u32],
    scratch: &mut [u8],
) {
    let flags = CMD_ARG_TABLE[cmd as usize];
    let n = (flags & 0xF) as usize;
    for (i, a) in args[..n].iter().enumerate() {
        set_out(scratch, i, *a);
    }
    let cursor = rd32(&q[4..]);
    wr32(&mut q[4..], pos);
    pop_u16(q);
    pop_u8(q);
    wr32(&mut q[4..], cursor & 0xFFFF);
    if cmd != 0x5F {
        let count = rd32(q);
        wr32(q, pos);
        write_cmd(env, q, cmd, flags as u32, scratch);
        wr32(q, count & 0xFFFF);
    }
}

/// C: `FUN_overlay_d_93__021cd5e0`: reads the next command ID from the read cursor, skipping
/// 0x5F skip records, and decodes its arguments into `out`: formats 0 and 0x10 through
/// `021cd6ac`, all others through `021cd0f0`. Returns the ID, or 0x5E when the buffer is
/// exhausted.
pub fn read_cmd(q: &mut [u8], out: &mut [u8]) -> u32 {
    let mut id = pop_u16(q) as u32;
    loop {
        if id != 0x5F {
            let c = CMD_ARG_TABLE[id as usize] as i8;
            if c == 0 || c == 0x10 {
                read_cmd_list(q, id & 0xFF, out);
            } else {
                read_cmd_args(q, c as i32 as u32, out);
            }
            return id;
        }
        let skip = pop_u8(q) as u32;
        let cur = rd32(&q[4..]);
        wr32(&mut q[4..], cur.wrapping_add(skip));
        if rd32(q) <= cur.wrapping_add(skip) {
            return 0x5E;
        }
        id = pop_u16(q) as u32;
    }
}

/// C: `FUN_overlay_d_93__021cd0f0`: decodes the arguments of a command of format `fmt` (its
/// `CMD_ARG_TABLE` byte) from the read cursor of `q` into the words of `out`, the reverse of
/// `write_cmd`. Formats the switch does not list read nothing.
pub fn read_cmd_args(q: &mut [u8], fmt: u32, out: &mut [u8]) {
    match fmt {
        0x54 => {
            set_out(out, 0, pop_u8(q) as u32);
            set_out(out, 1, pop_u8(q) as u32);
            set_out(out, 2, pop_u16(q) as u32);
            set_out(out, 3, pop_u16(q) as u32);
        }
        0x53 => {
            set_out(out, 0, pop_u8(q) as u32);
            set_out(out, 1, pop_u8(q) as u32);
            set_out(out, 2, pop_u32(q));
        }
        0x44 => {
            let v = pop_u16(q) as u32;
            set_out(out, 0, v >> 11 & 0x1F);
            set_out(out, 1, v >> 6 & 0x1F);
            set_out(out, 2, v & 0x3F);
            set_out(out, 3, pop_u16(q) as u32);
        }
        0x43 => {
            set_out(out, 0, pop_u8(q) as u32);
            set_out(out, 1, pop_u8(q) as u32);
            set_out(out, 2, pop_u16(q) as u32);
        }
        0x42 => {
            let v = pop_u8(q) as u32;
            set_out(out, 0, v >> 3 & 0x1F);
            set_out(out, 1, v & 7);
        }
        1 => set_out(out, 0, pop_u8(q) as u32),
        2 => {
            set_out(out, 0, pop_u8(q) as u32);
            set_out(out, 1, pop_u8(q) as u32);
        }
        3 => {
            let v = pop_u8(q) as u32;
            set_out(out, 0, v >> 3 & 0x1F);
            set_out(out, 1, v & 7);
            set_out(out, 2, pop_u8(q) as u32);
        }
        4 => {
            let v = pop_u8(q) as u32;
            set_out(out, 0, v >> 3 & 0x1F);
            set_out(out, 1, v & 7);
            set_out(out, 2, pop_u8(q) as u32);
            set_out(out, 3, pop_u16(q) as u32);
        }
        5 => {
            let v = pop_u16(q) as u32;
            set_out(out, 0, v >> 10 & 0x1F);
            set_out(out, 1, v >> 5 & 0x1F);
            set_out(out, 2, v & 0x1F);
            set_out(out, 3, pop_u16(q) as u32);
            set_out(out, 4, pop_u16(q) as u32);
        }
        6 => {
            let v = pop_u16(q) as u32;
            let w = pop_u16(q) as u32;
            set_out(out, 0, v >> 10 & 0x1F);
            set_out(out, 1, v >> 5 & 0x1F);
            set_out(out, 2, v & 0x1F);
            set_out(out, 3, w >> 10 & 0x1F);
            set_out(out, 4, w >> 5 & 0x1F);
            set_out(out, 5, w & 0x1F);
        }
        7 | 8 => {
            for i in 0..fmt as usize {
                set_out(out, i, pop_u8(q) as u32);
            }
        }
        0x11 => set_out(out, 0, pop_u16(q) as u32),
        0x12 => {
            set_out(out, 0, pop_u8(q) as u32);
            set_out(out, 1, pop_u16(q) as u32);
        }
        0x13 => {
            let v = pop_u8(q) as u32;
            set_out(out, 0, v >> 3 & 0x1F);
            set_out(out, 1, v & 7);
            set_out(out, 2, pop_u16(q) as u32);
        }
        0x14 => {
            let v = pop_u8(q) as u32;
            set_out(out, 1, v & 7);
            set_out(out, 0, v >> 3 & 0x1F);
            let w = pop_u8(q) as u32;
            set_out(out, 2, w >> 3 & 0x1F);
            set_out(out, 3, w & 7);
        }
        0x15 => {
            let v = pop_u16(q) as u32;
            let w = pop_u16(q) as u32;
            set_out(out, 0, (v & 0xFF) >> 3);
            set_out(out, 1, v & 7);
            set_out(out, 2, (w & 0xFF) >> 1);
            set_out(out, 3, w & 1);
            set_out(out, 4, pop_u16(q) as u32);
        }
        0x16 => {
            let v = pop_u8(q) as u32;
            set_out(out, 0, v >> 5 & 7);
            set_out(out, 1, v >> 3 & 3);
            set_out(out, 2, v >> 2 & 1);
            set_out(out, 4, v & 1);
            set_out(out, 3, v >> 1 & 1);
            set_out(out, 5, pop_u16(q) as u32);
        }
        0x22 => {
            set_out(out, 0, pop_u8(q) as u32);
            set_out(out, 1, pop_u32(q));
        }
        0x23 => {
            let v = pop_u16(q) as u32;
            set_out(out, 0, v >> 10 & 0x1F);
            set_out(out, 1, v >> 5 & 0x1F);
            set_out(out, 2, v & 0x1F);
        }
        0x24 => {
            let v = pop_u24(q);
            set_out(out, 0, v >> 19 & 0x1F);
            set_out(out, 1, v >> 14 & 0x1F);
            set_out(out, 2, v & 0x3FFF);
            set_out(out, 3, pop_u8(q) as u32);
        }
        0x25 => {
            let v = pop_u8(q) as u32;
            set_out(out, 0, v >> 5 & 7);
            set_out(out, 1, v >> 2 & 7);
            set_out(out, 3, v & 1);
            set_out(out, 2, v >> 1 & 1);
            set_out(out, 4, pop_u16(q) as u32);
        }
        0x26 => {
            let v = pop_u24(q);
            set_out(out, 0, v >> 15 & 0x1F);
            set_out(out, 1, v >> 10 & 0x1F);
            set_out(out, 3, v & 0x1F);
            set_out(out, 2, v >> 5 & 0x1F);
            set_out(out, 4, pop_u16(q) as u32);
            set_out(out, 5, pop_u16(q) as u32);
        }
        0x32 => {
            let v = pop_u8(q) as u32;
            set_out(out, 0, v >> 4 & 0xF);
            set_out(out, 1, v & 0xF);
        }
        0x33 => {
            let v = pop_u24(q);
            set_out(out, 0, v >> 19 & 0x1F);
            set_out(out, 1, v >> 14 & 0x1F);
            set_out(out, 2, v & 0x3FFF);
        }
        0x34 => {
            let v = pop_u24(q);
            set_out(out, 0, v >> 19 & 0x1F);
            set_out(out, 1, v >> 14 & 0x1F);
            set_out(out, 2, v & 0x3FFF);
            set_out(out, 3, pop_u16(q) as u32);
        }
        _ => {}
    }
}

/// C: `FUN_overlay_d_93__021cd64c`: puts a variable-length command: `cmd & 0xFF` and `w` as
/// 16-bit values, then (when the low byte of `cmd` is 0x5C) one more 16-bit value, then
/// 32-bit values until one equals 0xFFFF0000 (written too). `rest` = the call's fourth
/// argument onward, which must contain the terminator.
pub fn put_cmd_list<E: FlowEnv>(env: &mut E, q: &mut [u8], cmd: u32, w: u32, rest: &[u32]) {
    push_u16(env, q, (cmd & 0xFF) as u16);
    push_u16(env, q, w as u16);
    let mut i = 0;
    if cmd as u8 == 0x5C {
        push_u16(env, q, rest[0] as u16);
        i = 1;
    }
    loop {
        let v = rest[i];
        push_u32(env, q, v);
        i += 1;
        if v == LIST_END {
            break;
        }
    }
}

/// C: `FUN_overlay_d_93__021cd6ac`: decodes the arguments of a variable-length command (format
/// 0 or 0x10) into `out` (16 words): a 16-bit value, a second one when `cmd` is 0x5C, then
/// 32-bit values up to and including the terminator 0xFFFF0000 (at most 16 words in all).
pub fn read_cmd_list(q: &mut [u8], cmd: u32, out: &mut [u8]) {
    let mut n = 1;
    set_out(out, 0, pop_u16(q) as u32);
    if cmd == 0x5C {
        set_out(out, 1, pop_u16(q) as u32);
        n = 2;
    }
    while n < 0x10 {
        let v = pop_u32(q);
        set_out(out, n, v);
        if v == LIST_END {
            return;
        }
        n += 1;
    }
}

#[cfg(test)]
pub(crate) mod tests;
