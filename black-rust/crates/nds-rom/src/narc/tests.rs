use super::*;
use crate::fnt::Fnt;
use crate::lz;
use std::path::{Path, PathBuf};

const ROM_FILES: &str =
    "$HOME/workplace/black/black-rom/files";

#[test]
fn build_and_parse_nameless() {
    let files: Vec<Vec<u8>> = vec![vec![1, 2, 3], vec![], vec![9; 8], vec![7]];
    let narc = Narc::build(&files);
    let bytes = narc.to_bytes();
    assert_eq!(&bytes[..4], b"NARC");
    assert_eq!(bytes.len() % 4, 0);
    let back = Narc::parse(&bytes).unwrap();
    assert_eq!(back.len(), 4);
    for (i, f) in files.iter().enumerate() {
        assert_eq!(back.file(i), f.as_slice());
    }
    assert!(back.names().is_none());
    assert_eq!(back.to_bytes(), bytes);
    assert!(back.get(4).is_none());
}

#[test]
fn empty_archive() {
    let bytes = Narc::build::<&[u8]>(&[]).to_bytes();
    let n = Narc::parse(&bytes).unwrap();
    assert!(n.is_empty());
    assert_eq!(n.to_bytes(), bytes);
}

#[test]
fn named_archive() {
    let fnt = Fnt::from_paths(0, &["a.bin", "dir/b.bin"]).unwrap();
    let mut narc = Narc::build(&[&b"AA"[..], &b"BBB"[..]]);
    narc.fntb = fnt.build().unwrap();
    let back = Narc::parse(&narc.to_bytes()).unwrap();
    assert_eq!(
        back.names().unwrap(),
        vec!["a.bin".to_string(), "dir/b.bin".to_string()]
    );
    assert_eq!(back.file(1), b"BBB");
}

#[test]
fn rejects_garbage() {
    assert!(Narc::parse(b"").is_err());
    assert!(Narc::parse(b"NARC").is_err());
    let good = Narc::build(&[&b"xyz"[..]]).to_bytes();
    for cut in 0..good.len() {
        assert!(Narc::parse(&good[..cut]).is_err(), "cut {cut}");
    }
    let mut bad = good.clone();
    bad[0] = b'X';
    assert!(Narc::parse(&bad).is_err());
}

#[test]
fn mutated_bytes_never_panic() {
    let good = Narc::build(&[&b"hello"[..], &b"world!"[..]]).to_bytes();
    let mut s = 0x9e37_79b9_7f4a_7c15u64;
    for _ in 0..20000 {
        let mut b = good.clone();
        for _ in 0..3 {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            let i = (s as usize) % b.len();
            b[i] = (s >> 32) as u8;
        }
        if let Ok(n) = Narc::parse(&b) {
            for f in n.files() {
                let _ = f.len();
            }
            let _ = n.names();
            let _ = n.to_bytes();
        }
    }
}

fn collect_narcs(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for a in std::fs::read_dir(root.join("a/0"))
        .into_iter()
        .flatten()
        .flatten()
    {
        for f in std::fs::read_dir(a.path()).into_iter().flatten().flatten() {
            out.push(f.path());
        }
    }
    out.sort();
    out
}

/// Every `a/0/*/*` file of the local ROM filesystem parses, rebuilds byte-identically (when
/// nameless) and its LZ members decompress. Skipped when the ROM files are missing.
#[test]
fn real_rom_narcs() {
    let root = Path::new(ROM_FILES);
    if !root.join("a/0").is_dir() {
        eprintln!("skipping: {ROM_FILES} not found");
        return;
    }
    let (mut narcs, mut members, mut lz10, mut lz11, mut other, mut named) = (0, 0, 0, 0, 0, 0);
    let (mut lookalike, mut tail_only) = (0, 0);
    for path in collect_narcs(root) {
        let bytes = std::fs::read(&path).unwrap();
        if !bytes.starts_with(b"NARC") {
            other += 1;
            continue;
        }
        let narc = Narc::parse(&bytes).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        narcs += 1;
        members += narc.len();
        if narc.names().is_some() {
            named += 1;
        } else {
            assert_eq!(narc.to_bytes(), bytes, "round trip {}", path.display());
        }
        for f in narc.files() {
            if let Some(kind) = lz::kind(f) {
                if let Some(out) = lz::decompress_any(f) {
                    assert!(!out.is_empty() || f.len() <= 8);
                    match kind {
                        lz::LzKind::Lz10 => lz10 += 1,
                        lz::LzKind::Lz11 => lz11 += 1,
                    }
                } else {
                    lookalike += 1;
                    if lz::decompress(f).is_ok() {
                        tail_only += 1;
                    }
                }
            }
        }
    }
    eprintln!(
        "NARC summary: {narcs} NARCs ({named} named, {other} not NARC), {members} members, \
         {lz10} LZ10, {lz11} LZ11 ({lookalike} LZ-looking but undecodable, {tail_only} of them decode with a long tail)"
    );
    assert!(narcs > 0 && members > 0);
}
