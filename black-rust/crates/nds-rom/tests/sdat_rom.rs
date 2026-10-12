//! SDAT against the real ROM (`wb_sound_data.sdat`). Skipped when the file is absent.

use nds_rom::sdat::{sbnk::Sbnk, ssar::Ssar, sseq::Sseq, strm::Strm, swar::Swar, Kind, Sdat};
use nds_rom::sha1_hex;
use std::path::PathBuf;

fn load() -> Option<Vec<u8>> {
    let dir = std::env::var_os("BLACK_ROM_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../black-rom"));
    std::fs::read(dir.join("files/wb_sound_data.sdat")).ok()
}

#[test]
fn parses_every_entry() {
    let Some(bytes) = load() else {
        eprintln!("skipping: wb_sound_data.sdat not found");
        return;
    };
    let sdat = Sdat::parse(&bytes).unwrap();
    for k in Kind::ALL {
        eprintln!("{}: {}", k.name(), sdat.count(k));
    }
    let (mut events, mut waves, mut insts, mut ssar_seqs, mut strm_samples) = (0, 0, 0, 0, 0);
    for (id, e) in sdat.entries(Kind::Seq).iter().enumerate() {
        if e.is_some() {
            events += Sseq::parse(sdat.entry_file(Kind::Seq, id).unwrap())
                .unwrap_or_else(|err| panic!("SEQ {id}: {err}"))
                .event_count();
        }
    }
    for (id, e) in sdat.entries(Kind::SeqArc).iter().enumerate() {
        if e.is_some() {
            let s = Ssar::parse(sdat.entry_file(Kind::SeqArc, id).unwrap())
                .unwrap_or_else(|err| panic!("SEQARC {id}: {err}"));
            ssar_seqs += s.sequences.len();
        }
    }
    for (id, e) in sdat.entries(Kind::Bank).iter().enumerate() {
        if e.is_some() {
            insts += Sbnk::parse(sdat.entry_file(Kind::Bank, id).unwrap())
                .unwrap_or_else(|err| panic!("BANK {id}: {err}"))
                .instruments
                .len();
        }
    }
    for (id, e) in sdat.entries(Kind::WaveArc).iter().enumerate() {
        if e.is_some() {
            let a = Swar::parse(sdat.entry_file(Kind::WaveArc, id).unwrap())
                .unwrap_or_else(|err| panic!("WAVEARC {id}: {err}"));
            for (w, swav) in a.waves.iter().enumerate() {
                swav.decode()
                    .unwrap_or_else(|err| panic!("WAVEARC {id} wave {w}: {err}"));
                waves += 1;
            }
        }
    }
    for (id, e) in sdat.entries(Kind::Strm).iter().enumerate() {
        if e.is_some() {
            let s = Strm::parse(sdat.entry_file(Kind::Strm, id).unwrap())
                .unwrap_or_else(|err| panic!("STRM {id}: {err}"));
            strm_samples += s
                .decode()
                .unwrap_or_else(|err| panic!("STRM {id}: {err}"))
                .frames();
        }
    }
    eprintln!(
        "events {events} ssar_seqs {ssar_seqs} instruments {insts} swav {waves} strm_samples {strm_samples}"
    );
}

#[test]
fn golden() {
    let Some(bytes) = load() else {
        eprintln!("skipping: wb_sound_data.sdat not found");
        return;
    };
    let sdat = Sdat::parse(&bytes).unwrap();
    // (swar id, swav index, frames, loop, sha1 of interleaved little-endian PCM)
    type Golden = (usize, usize, usize, Option<(usize, usize)>, &'static str);
    let waves: [Golden; 2] = [
        (1, 0, 8184, None, "2572b3d8b6126c154b08b3df7ed18094e0c5aab3"),
        (
            1000,
            0,
            19664,
            Some((11488, 19664)),
            "6a0138611ec98f9d0e2fd12a2721e271181254ae",
        ),
    ];
    for (a, w, frames, looping, sha) in waves {
        let sw = Swar::parse(sdat.entry_file(Kind::WaveArc, a).unwrap()).unwrap();
        let p = sw.waves[w].decode().unwrap();
        assert_eq!((p.frames(), p.looping), (frames, looping), "{a}:{w}");
        assert_eq!(sha1_hex(&p.interleaved_bytes()), sha, "{a}:{w}");
    }
    for (id, n) in [(1000usize, 3201usize), (1100, 3136)] {
        let q = Sseq::parse(sdat.entry_file(Kind::Seq, id).unwrap()).unwrap();
        assert_eq!(q.event_count(), n, "SEQ {id}");
    }
}
