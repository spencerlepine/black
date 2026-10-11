use std::path::PathBuf;
use std::process::Command;

fn black() -> Command {
    Command::new(env!("CARGO_BIN_EXE_black"))
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("black-cli-test-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn blz_roundtrip_via_cli() {
    let dir = scratch("blz");
    let input: Vec<u8> = (0..20000u32).map(|i| ((i / 7) % 13) as u8).collect();
    std::fs::write(dir.join("in.bin"), &input).unwrap();
    let ok = black()
        .args(["blz", "compress"])
        .arg(dir.join("in.bin"))
        .arg(dir.join("in.blz"))
        .status()
        .unwrap();
    assert!(ok.success());
    let ok = black()
        .args(["blz", "decompress"])
        .arg(dir.join("in.blz"))
        .arg(dir.join("out.bin"))
        .status()
        .unwrap();
    assert!(ok.success());
    assert_eq!(std::fs::read(dir.join("out.bin")).unwrap(), input);
    assert!(std::fs::metadata(dir.join("in.blz")).unwrap().len() < input.len() as u64);
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn info_on_real_rom_when_present() {
    let rom = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../black-rom/build/black.us/pokeblack.us.nds");
    if !rom.exists() {
        eprintln!("skipping: {} not found", rom.display());
        return;
    }
    let out = black().arg("info").arg(&rom).output().unwrap();
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("IRBO"), "{text}");
    assert!(
        text.contains("26ad0b9967aa279c4a266ee69f52b9b2332399a5"),
        "{text}"
    );
}

#[test]
fn missing_file_is_an_error() {
    let out = black()
        .args(["info", "/nonexistent/rom.nds"])
        .output()
        .unwrap();
    assert!(!out.status.success());
}
