//! Integration tests against the real, locally built Pokemon Black artifacts.
//!
//! They are skipped (not failed) when the build output is absent, unless `REQUIRE_ROM` is set.
//! The source tree defaults to `../../../black-rom` relative to this crate; set `BLACK_ROM_DIR` to
//! override. The build (untracked, so worktrees only have it there) is `$BLACK_ROM_BUILD`, else
//! `build/black.us` in the source tree.

use nds_rom::build::{assemble, default_inputs, parse_file_list};
use nds_rom::fnt::Fnt;
use nds_rom::overlay::parse_table;
use nds_rom::{blz, sha1_hex, Rom};
use std::path::PathBuf;

fn project() -> PathBuf {
    std::env::var_os("BLACK_ROM_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../black-rom"))
}

fn build_dir() -> PathBuf {
    std::env::var_os("BLACK_ROM_BUILD")
        .map(PathBuf::from)
        .unwrap_or_else(|| project().join("build/black.us"))
}

fn rom_path() -> PathBuf {
    build_dir().join("pokeblack.us.nds")
}

macro_rules! require {
    ($path:expr) => {
        if !$path.exists() {
            nds_rom::testenv::rom_missing(&format!("{} not found", $path.display()));
            return;
        }
    };
}

fn expected_sha1() -> String {
    let text = std::fs::read_to_string(project().join("black.us/rom.sha1")).unwrap();
    text.split_whitespace().next().unwrap().to_string()
}

#[test]
fn parses_and_hashes() {
    let path = rom_path();
    require!(path);
    let rom = Rom::open(&path).unwrap();
    assert!(rom.header.header_crc_valid());
    assert!(rom.header.logo_crc_valid());
    assert_eq!(rom.header.game_code_str(), "IRBO");
    assert_eq!(rom.header.arm9_overlay_count(), 237);
    assert_eq!(rom.fat.len(), 484);
    assert_eq!(sha1_hex(&rom.data), expected_sha1());
}

#[test]
fn fnt_roundtrip_and_rebuild() {
    let fnt_path = project().join("black.us/fnt.bin");
    let files_path = project().join("black.us/files.txt");
    require!(fnt_path);
    let bytes = std::fs::read(&fnt_path).unwrap();
    let fnt = Fnt::parse(&bytes).unwrap();
    assert_eq!(fnt.build().unwrap(), bytes);
    let order = parse_file_list(&std::fs::read_to_string(files_path).unwrap());
    let rebuilt = Fnt::from_paths(237, &order).unwrap();
    assert_eq!(rebuilt.build().unwrap(), bytes);
}

#[test]
fn blz_matches_python_pipeline() {
    let build = build_dir();
    require!(build.join("packed/main_table.sbin"));
    let table = parse_table(&std::fs::read(build.join("main_table.sbin")).unwrap()).unwrap();
    let raw_ids: Vec<usize> =
        std::fs::read_to_string(project().join("black.us/overlay_compression.txt"))
            .unwrap()
            .lines()
            .filter_map(|l| l.split('#').next()?.trim().parse().ok())
            .collect();
    let mut checked = 0;
    for ov in table.iter().filter(|o| !raw_ids.contains(&(o.id as usize))) {
        let raw = std::fs::read(build.join(format!("OVY_{}.sbin", ov.id))).unwrap();
        if raw.len() > 4096 {
            continue; // keep debug-mode test time reasonable; the CLI covers the rest
        }
        let packed = std::fs::read(build.join(format!("packed/OVY_{}.sbin", ov.id))).unwrap();
        let ours = blz::compress(&raw, 0).unwrap_or_else(|| raw.clone());
        assert_eq!(ours, packed, "overlay {}", ov.id);
        if packed != raw {
            assert_eq!(blz::decompress(&packed).unwrap(), raw);
        }
        checked += 1;
    }
    assert!(checked > 20, "only checked {checked} overlays");
}

#[test]
fn assembled_rom_is_byte_identical() {
    let build = build_dir();
    require!(rom_path());
    require!(build.join("packed/arm9.bin"));
    let mut inputs = default_inputs(&project(), &build);
    if std::env::var_os("BLACK_ROM_BUILD").is_some() {
        // sub/build is untracked like build/: take arm7 from the tree the build belongs to
        // (<black-rom>/build/black.us -> <black-rom>/sub/build).
        inputs.arm7 = build.join("../../sub/build/arm7.sbin");
    }
    require!(inputs.arm7);
    let (rom, report) = assemble(&inputs).unwrap();
    assert_eq!(report.files, 484);
    assert_eq!(sha1_hex(&rom), expected_sha1());
}
