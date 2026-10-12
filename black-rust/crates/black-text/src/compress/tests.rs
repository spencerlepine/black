use super::*;
use crate::strbuf::{self, ch, len, wr16};
use crate::wordset::append_word;

/// Packs codes (each `< 0x1FF`) LSB first, adds the end code and the `0xF100` header.
fn pack(codes: &[u16]) -> Vec<u16> {
    let mut v = vec![COMPRESSED];
    let (mut acc, mut bits) = (0u32, 0u32);
    for &c in codes.iter().chain([&CODE_END]) {
        acc |= (c as u32) << bits;
        bits += 9;
        while bits >= 16 {
            v.push(acc as u16);
            acc >>= 16;
            bits -= 16;
        }
    }
    v.push(acc as u16);
    v.push(0xFFFF);
    v
}

fn strbuf_of(cap: u16, text: &[u16]) -> Vec<u8> {
    let mut b = vec![0u8; strbuf::CHARS_OFS + 2 * cap as usize + 2];
    wr16(&mut b, 0, cap);
    strbuf::clear(&mut b);
    for &c in text {
        strbuf::add_char(&mut b, c);
    }
    b
}

/// What the engine does with the string (the ROM-verified `wordset::append_word`).
fn via_append_word(packed: &[u16]) -> Vec<u16> {
    let src = strbuf_of(packed.len() as u16 + 1, packed);
    let mut dst = strbuf_of(0x600, &[]);
    append_word(&mut dst, &src);
    (0..len(&dst) as usize).map(|i| ch(&dst, i)).collect()
}

/// The table as the wordset job described it, written out independently.
fn expected_map(code: u16) -> u16 {
    const HEAD: [u16; 9] = [
        0x152, 0x153, 0x15E, 0x15F, 0x2018, 0x201C, 0x201D, 0x201E, 0x2026,
    ];
    match code {
        0..=0xFF => code,
        0x100..=0x108 => HEAD[(code - 0x100) as usize],
        0x109..=0x130 => 0x2460 + (code - 0x109),
        0x131 => 0xFF65,
        _ => 0x3F,
    }
}

#[test]
fn table_and_decode() {
    let codes: Vec<u16> = (0..0x1FF).collect();
    let packed = pack(&codes);
    let out = decode(&packed);
    assert_eq!(out.len(), 0x1FF);
    for (c, o) in codes.iter().zip(&out) {
        assert_eq!(*o, expected_map(*c), "code {c:#x}");
    }
    assert_eq!(out[0x100], 0x152);
    assert_eq!(out[0x108], 0x2026);
    assert_eq!(out[0x109], 0x2460);
    assert_eq!(out[0x130], 0x2487);
    assert_eq!(out[0x131], 0xFF65);
    assert_eq!(out[0x132], 0x3F);
    assert_eq!(decode(&packed[1..]), out);
}

#[test]
fn decode_agrees_with_append_word() {
    let mut seed = 0x1234_5678u32;
    for n in 0..200 {
        let codes: Vec<u16> = (0..n)
            .map(|_| {
                seed = seed.wrapping_mul(0x41C6_4E6D).wrapping_add(0x6073);
                ((seed >> 16) % 0x1FF) as u16
            })
            .collect();
        let packed = pack(&codes);
        assert_eq!(decode(&packed), via_append_word(&packed), "n = {n}");
    }
}

#[test]
fn unterminated_stream_yields_what_was_decoded() {
    assert_eq!(decode(&[COMPRESSED, 0x41]), [0x41]);
    assert!(decode(&[]).is_empty());
}

/// All 589 compressed messages of the ROM: pack the raw codes, then `decode` must equal both the
/// independent table mapping and the engine path (`append_word`).
#[test]
fn rom_compressed_entries() {
    let Some(entries) = crate::token::tests::rom_entries() else {
        eprintln!("skipping: message banks not found");
        return;
    };
    let mut n = 0;
    for (units, compressed) in entries {
        if !compressed {
            continue;
        }
        let packed = pack(&units);
        let want: Vec<u16> = units.iter().map(|&c| expected_map(c)).collect();
        assert_eq!(decode(&packed), want);
        assert_eq!(via_append_word(&packed), want);
        n += 1;
    }
    eprintln!("decoded {n} compressed messages");
    assert!(n > 500);
}
