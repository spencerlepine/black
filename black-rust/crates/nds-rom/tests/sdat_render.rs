//! A9 renderer against the real SDAT. Skipped when the file is absent.

use nds_rom::sdat::render::{detect_loop, RenderOptions, Renderer};
use nds_rom::sdat::{Kind, Sdat};
use nds_rom::sha1_hex;
use std::path::PathBuf;

fn load() -> Option<Vec<u8>> {
    let dir = std::env::var_os("BLACK_ROM_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../black-rom"));
    std::fs::read(dir.join("files/wb_sound_data.sdat")).ok()
}

fn short() -> RenderOptions {
    RenderOptions {
        loops: 1,
        max_seconds: 3.0,
        tail_seconds: 0.25,
    }
}

#[test]
fn renders_every_bgm_and_me() {
    let Some(bytes) = load() else {
        nds_rom::testenv::rom_missing("wb_sound_data.sdat not found");
        return;
    };
    let sdat = Sdat::parse(&bytes).unwrap();
    let mut r = Renderer::new(&sdat);
    let mut n = 0;
    for id in (1000..=1178).chain(1300..=1331) {
        if sdat.entry(Kind::Seq, id).is_none() {
            continue;
        }
        let out = r
            .render(id, &short())
            .unwrap_or_else(|e| panic!("SEQ {id}: {e}"));
        assert_eq!(out.pcm.channels.len(), 2);
        assert!(out.pcm.frames() > 0, "SEQ {id} empty");
        n += 1;
    }
    assert!(n > 150, "rendered {n}");
}

#[test]
fn golden_first_second_loops_and_length() {
    let Some(bytes) = load() else {
        nds_rom::testenv::rom_missing("wb_sound_data.sdat not found");
        return;
    };
    let sdat = Sdat::parse(&bytes).unwrap();
    let mut r = Renderer::new(&sdat);
    let opts = RenderOptions {
        loops: 1,
        max_seconds: 1.0,
        tail_seconds: 0.0,
    };
    for (id, sha) in GOLDEN {
        let pcm = r.render(*id, &opts).unwrap().pcm;
        assert_eq!(pcm.frames(), 32728, "SEQ {id} first second");
        let got = sha1_hex(&pcm.interleaved_bytes());
        eprintln!("GOLDEN {id} {got}");
        assert_eq!(got, *sha, "SEQ {id} sha1");
    }
    for (id, s, e) in LOOPS {
        let l = detect_loop(&sdat, *id, 600.0).unwrap();
        eprintln!("LOOP {id} {l:?}");
        assert_eq!(l, Some((*s, *e)), "SEQ {id} loop");
    }
    // a non-looping ME: full length incl. the 2 s release tail
    let me = r.render(1300, &RenderOptions::default()).unwrap();
    eprintln!("ME 1300 frames {}", me.pcm.frames());
    assert!(!me.loops);
    assert_eq!(me.pcm.frames(), ME_FRAMES);
}

const GOLDEN: &[(usize, &str)] = &[
    (1000, "69660b5d2dd02a889be0309de7e0a1e183c4af44"),
    (1050, "aa05740822a43a2b8ae5424865fe2fbc06e33392"),
];
const LOOPS: &[(usize, usize, usize)] = &[(1000, 319438, 1780096), (1050, 170, 1767482)];
const ME_FRAMES: usize = 91877;
