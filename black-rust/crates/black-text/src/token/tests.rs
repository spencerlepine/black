use super::*;
use nds_rom::msg::{units_from_tokens, Bank, Token as RomToken};
use nds_rom::narc::Narc;
use std::path::PathBuf;

fn u(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

#[test]
fn parse_and_encode_small_strings() {
    let mut s = u("Hi ");
    s.extend([TAG, 0x0100, 1, 0]);
    s.push(NEWLINE);
    s.extend(u("!"));
    s.extend([TAG, 0xBE00, 0]);
    s.push(END);
    let t = parse(&s);
    assert_eq!(
        t,
        [
            Token::Char(b'H' as u16),
            Token::Char(b'i' as u16),
            Token::Char(b' ' as u16),
            Token::Tag {
                id: 0x0100,
                args: vec![0]
            },
            Token::NewLine,
            Token::Char(b'!' as u16),
            Token::Tag {
                id: 0xBE00,
                args: vec![]
            },
            Token::End,
        ]
    );
    assert_eq!(t[3].group(), Some(1));
    assert_eq!(t[3].cmd(), Some(0));
    assert_eq!(t[0].group(), None);
    assert_eq!(encode(&t), s);
}

#[test]
fn parse_stops_at_end_and_keeps_truncated_tags() {
    let s = [0x41, END, 0x42];
    assert_eq!(parse(&s), [Token::Char(0x41), Token::End]);
    let s = [0x41, TAG, 0x0100, 3, 9];
    let t = parse(&s);
    assert_eq!(t[1], Token::Truncated(vec![TAG, 0x0100, 3, 9]));
    assert_eq!(encode(&t), s);
    assert_eq!(parse(&[TAG, 1]), [Token::Truncated(vec![TAG, 1])]);
    assert!(parse(&[]).is_empty());
}

#[test]
fn compressed_strings_are_unpacked() {
    // 'A' (0x41), 0x100 -> 0x152, end
    let codes = [0x41u32, 0x100, 0x1FF];
    let (mut acc, mut bits, mut v) = (0u64, 0u32, vec![COMPRESSED]);
    for c in codes {
        acc |= (c as u64) << bits;
        bits += 9;
        while bits >= 16 {
            v.push(acc as u16);
            acc >>= 16;
            bits -= 16;
        }
    }
    v.push(acc as u16);
    assert_eq!(parse(&v), [Token::Char(0x41), Token::Char(0x152)]);
}

fn rom_dir() -> PathBuf {
    std::env::var_os("BLACK_ROM_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../black-rom"))
}

/// Raw units of every entry of every message bank (`a/0/0/2`, `a/0/0/3`), with the `compressed`
/// flag. `None` when the ROM files are absent.
pub(crate) fn rom_entries() -> Option<Vec<(Vec<u16>, bool)>> {
    let mut out = Vec::new();
    for arc in ["a/0/0/2", "a/0/0/3"] {
        let bytes = std::fs::read(rom_dir().join("files").join(arc)).ok()?;
        let narc = Narc::parse(&bytes).unwrap();
        for i in 0..narc.len() {
            let bank = Bank::parse(narc.file(i)).unwrap();
            for e in bank.entries() {
                out.push((units_from_tokens(&e.tokens).unwrap(), e.compressed));
            }
        }
    }
    Some(out)
}

/// Every message of the ROM (not compressed ones as stored; compressed ones as their raw codes)
/// goes through `parse` and `encode` unchanged and the token counts agree with `nds_rom::msg`.
#[test]
fn rom_round_trip() {
    let Some(entries) = rom_entries() else {
        eprintln!("skipping: message banks not found");
        return;
    };
    let (mut n, mut tags, mut breaks) = (0, 0, 0);
    for (units, _) in &entries {
        let mut s = units.clone();
        s.push(END);
        let t = parse(&s);
        assert_eq!(encode(&t), s);
        assert_eq!(t.last(), Some(&Token::End));
        assert!(!t.iter().any(|x| matches!(x, Token::Truncated(_))));
        n += 1;
        tags += t.iter().filter(|x| matches!(x, Token::Tag { .. })).count();
        breaks += t.iter().filter(|x| matches!(x, Token::NewLine)).count();
    }
    // the same counts from nds_rom's own tokens
    let (mut tags2, mut breaks2) = (0, 0);
    for arc in ["a/0/0/2", "a/0/0/3"] {
        let bytes = std::fs::read(rom_dir().join("files").join(arc)).unwrap();
        let narc = Narc::parse(&bytes).unwrap();
        for i in 0..narc.len() {
            for e in Bank::parse(narc.file(i)).unwrap().entries() {
                for t in &e.tokens {
                    match t {
                        RomToken::Cmd { .. } => tags2 += 1,
                        RomToken::NewLine => breaks2 += 1,
                        _ => {}
                    }
                }
            }
        }
    }
    assert_eq!((tags, breaks), (tags2, breaks2));
    eprintln!("round-tripped {n} messages, {tags} tags, {breaks} line breaks");
    assert!(n > 50_000);
}
