use super::*;
use crate::narc::Narc;
use std::path::PathBuf;

fn hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

/// One section, one entry, `strings` already encrypted (vectors computed independently).
fn bank_bytes(strings: &[u8], chars: u16, flags: u16) -> Vec<u8> {
    let pad = strings.len() % 4;
    let size = 4 + 8 + (strings.len() + pad) as u32;
    let mut b = Vec::new();
    b.extend_from_slice(&[1, 0, 1, 0]);
    b.extend_from_slice(&size.to_le_bytes());
    b.extend_from_slice(&0u32.to_le_bytes());
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&size.to_le_bytes());
    b.extend_from_slice(&12u32.to_le_bytes());
    b.extend_from_slice(&chars.to_le_bytes());
    b.extend_from_slice(&flags.to_le_bytes());
    b.extend_from_slice(strings);
    b.resize(b.len() + pad, 0);
    b
}

#[test]
fn handmade_plain_vector() {
    // "Hi", line break, command 0x0100 with params 1 and 2
    let bytes = bank_bytes(&hex("c17c22e4a1ddf9e2c89646be24f22d918376"), 9, 0x4a);
    let bank = Bank::parse(&bytes).unwrap();
    let e = &bank.entries()[0];
    assert_eq!(e.flags, 0x4a);
    assert!(!e.compressed);
    assert_eq!(
        e.tokens,
        vec![
            Token::Text("Hi".into()),
            Token::NewLine,
            Token::Cmd {
                id: 0x100,
                params: vec![1, 2]
            }
        ]
    );
    assert_eq!(e.render(), "Hi\n[VAR 0100(0001,0002)]");
    assert_eq!(bank.to_bytes().unwrap(), bytes);
}

#[test]
fn handmade_compressed_vector() {
    let bytes = bank_bytes(&hex("898d0834cbb3aad405e8bb41"), 6, 0);
    let bank = Bank::parse(&bytes).unwrap();
    let e = &bank.entries()[0];
    assert!(e.compressed);
    assert_eq!(e.tokens, vec![Token::Text("Cheren".into())]);
    assert_eq!(bank.to_bytes().unwrap(), bytes);
}

#[test]
fn compression_padding_cases() {
    // every length, so the end code lands on every bit offset
    for n in 0..40u16 {
        let units: Vec<u16> = (0..n).map(|i| 0x20 + i).collect();
        let packed = compress(&units).unwrap();
        assert_eq!(packed.last(), Some(&END));
        assert_eq!(decompress(&packed).unwrap(), units);
    }
    assert!(compress(&[0x1FF]).is_err());
}

#[test]
fn render_roundtrip_and_escapes() {
    let tokens = vec![
        Token::Text("a[b]\\c\u{1}\u{e000}".into()),
        Token::NewLine,
        Token::Cmd {
            id: 0xBE01,
            params: vec![],
        },
        Token::Cmd {
            id: 0x207,
            params: vec![1, 0xFFFF],
        },
        Token::Raw(0xD800),
        Token::Text("end".into()),
    ];
    let s = render(&tokens);
    assert_eq!(
        s,
        "a\\[b]\\\\c\\x01\u{e000}\n[VAR BE01][VAR 0207(0001,FFFF)][RAW D800]end"
    );
    assert_eq!(parse_rendered(&s).unwrap(), tokens);
    let units = units_from_tokens(&tokens).unwrap();
    assert_eq!(tokens_from_units(&units).unwrap(), tokens);
}

#[test]
fn rejects_bad_input() {
    assert!(tokens_from_units(&[0xF000, 1]).is_err());
    assert!(tokens_from_units(&[0xF000, 1, 2, 3]).is_err());
    assert!(tokens_from_units(&[0x41, 0xFFFF]).is_err());
    assert!(units_from_tokens(&[Token::Text("\u{f000}".into())]).is_err());
    assert!(Bank::parse(&[1, 0, 1]).is_err());
    assert!(parse_rendered("[NOPE]").is_err());
}

#[test]
fn json_escaping() {
    let bank = Bank {
        reserved: 0,
        tails: vec![vec![]],
        sections: vec![vec![Entry {
            flags: 3,
            compressed: false,
            tokens: vec![Token::Text("say \"hi\"\u{2}".into()), Token::NewLine],
        }]],
    };
    assert_eq!(
        bank.to_json(),
        "[\n  {\"i\":0,\"flags\":3,\"compressed\":false,\"text\":\"say \\\"hi\\\"\\\\x02\\n\"}\n]\n"
    );
}

fn rom_files() -> PathBuf {
    std::env::var_os("BLACK_ROM_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../black-rom"))
        .join("files/a/0/0")
}

fn load(n: u32) -> Option<Narc> {
    let path = rom_files().join(n.to_string());
    if !path.exists() {
        eprintln!("skipping: {} not found", path.display());
        return None;
    }
    Some(Narc::parse(&std::fs::read(path).unwrap()).unwrap())
}

#[test]
fn real_rom_roundtrip_all_banks() {
    let (Some(sys), Some(story)) = (load(2), load(3)) else {
        return;
    };
    assert_eq!(sys.len(), 288);
    assert_eq!(story.len(), 472);
    let mut entries = 0;
    for narc in [&sys, &story] {
        for (i, f) in narc.files().enumerate() {
            let bank = Bank::parse(f).unwrap_or_else(|e| panic!("bank {i}: {e}"));
            let back = bank.to_bytes().unwrap();
            assert_eq!(back, f, "bank {i} round trip");
            for e in bank.entries() {
                let s = e.render();
                assert_eq!(parse_rendered(&s).unwrap(), e.tokens, "bank {i} render");
            }
            entries += bank.entries().len();
        }
    }
    assert_eq!(entries, 56700);
}

#[test]
fn real_rom_known_strings() {
    let Some(sys) = load(2) else { return };
    let text = |bank: usize, i: usize| Bank::parse(sys.file(bank)).unwrap().entries()[i].render();
    assert_eq!(text(0, 1), "Black");
    // name tables, indexed by species / move / item / ability / type / trainer class id
    assert_eq!(text(70, 1), "Bulbasaur");
    assert_eq!(text(70, 495), "Snivy");
    assert_eq!(text(203, 33), "Tackle");
    assert_eq!(text(54, 1), "Master Ball");
    assert_eq!(text(182, 65), "Overgrow");
    assert_eq!(text(199, 0), "Normal");
    assert_eq!(text(191, 2), "Youngster");
    // command, line break and page break
    assert_eq!(
        text(3, 56),
        "Juniper's words echoed...\n[VAR 0100(0000)]! There's a time and[VAR BE00]\nplace for everything! But not now."
    );
}
