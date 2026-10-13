//! Support for the workspace's real-ROM tests (they read `black-rom/files` and the untracked ROM
//! build at `$BLACK_ROM_BUILD`). Not part of the ROM API.

/// Reports that ROM data a test needs is missing; the caller then returns. Normally the test is
/// skipped. With `REQUIRE_ROM` set (and not `0`; `tools/gate.sh` sets `REQUIRE_ROM=1`) it fails
/// instead, so a run that cannot find the ROM files or build does not pass by skipping.
pub fn rom_missing(what: &str) {
    if std::env::var_os("REQUIRE_ROM").is_some_and(|v| !v.is_empty() && v != "0") {
        panic!("REQUIRE_ROM is set but {what} (set BLACK_ROM_BUILD / BLACK_ROM_DIR)");
    }
    eprintln!("skipping: {what}");
}
