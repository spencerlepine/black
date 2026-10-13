//! Map data readers against the real ROM. Skipped when the ROM files are absent.

use nds_rom::map::{MapContainer, MapMatrix, ZoneEvents, ZoneHeader};
use nds_rom::narc::Narc;
use std::path::PathBuf;

fn narc(id: u32) -> Option<Narc> {
    let dir = std::env::var_os("BLACK_ROM_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../black-rom"));
    let p = dir.join(format!("files/a/{}/{}/{}", id / 100, id / 10 % 10, id % 10));
    Narc::parse(&std::fs::read(p).ok()?).ok()
}

struct Data {
    zones: Vec<ZoneHeader>,
    matrices: Vec<MapMatrix>,
    containers: Vec<MapContainer>,
    events: Vec<ZoneEvents>,
}

fn load() -> Option<Data> {
    let (z, m, c, e) = (narc(12)?, narc(9)?, narc(8)?, narc(125)?);
    assert_eq!(z.len(), 1);
    let zones = ZoneHeader::parse_table(z.file(0)).unwrap();
    let mut matrices = Vec::new();
    for (i, f) in m.files().enumerate() {
        let x = MapMatrix::parse(f).unwrap_or_else(|e| panic!("matrix {i}: {e}"));
        assert_eq!(x.to_bytes(), f, "matrix {i} round trip");
        matrices.push(x);
    }
    let mut containers = Vec::new();
    for (i, f) in c.files().enumerate() {
        let x = MapContainer::parse(f).unwrap_or_else(|e| panic!("container {i}: {e}"));
        assert_eq!(x.to_bytes(), f, "container {i} round trip");
        containers.push(x);
    }
    let mut events = Vec::new();
    for (i, f) in e.files().enumerate() {
        let x = ZoneEvents::parse(f).unwrap_or_else(|e| panic!("events {i}: {e}"));
        assert_eq!(x.to_bytes(), f, "events {i} round trip");
        events.push(x);
    }
    Some(Data {
        zones,
        matrices,
        containers,
        events,
    })
}

#[test]
fn parses_every_map_file() {
    let Some(d) = load() else {
        nds_rom::testenv::rom_missing("ROM files not found");
        return;
    };
    assert_eq!(d.zones.len(), 427);
    assert_eq!(d.matrices.len(), 255);
    assert_eq!(d.containers.len(), 649);
    assert_eq!(d.events.len(), 428);
    for (i, z) in d.zones.iter().enumerate() {
        assert_eq!(z.zone_id as usize, i);
        assert_eq!(z.to_bytes().len(), 48);
        assert!((z.matrix_id as usize) < d.matrices.len());
        assert!(z.name_id < 117);
    }
    // Every matrix cell points at an existing container.
    for (i, m) in d.matrices.iter().enumerate() {
        for &c in m.cells.iter().chain(m.cells_b.iter().flatten()) {
            assert!(
                c == nds_rom::map::matrix::NO_MAP || (c as usize) < d.containers.len(),
                "matrix {i} cell {c}"
            );
        }
    }
    let (mut wb, mut perm, mut bld) = (0, 0, 0);
    for (i, c) in d.containers.iter().enumerate() {
        if c.is_wb() {
            wb += 1;
            let g = c.permissions().unwrap_or_else(|e| panic!("perm {i}: {e}"));
            assert!(g.width == 32 && g.height == 32 || g.width == 64 && g.height == 64);
            assert_eq!(g.to_bytes(), c.sections[1]);
            perm += g.cells.len();
            bld += c
                .buildings()
                .unwrap_or_else(|e| panic!("bld {i}: {e}"))
                .len();
            assert_eq!(&c.model().unwrap()[..4], b"BMD0");
        }
    }
    let n = |f: fn(&ZoneEvents) -> usize| d.events.iter().map(f).sum::<usize>();
    eprintln!(
        "zones {} matrices {} containers {} (WB {wb}, perm cells {perm}, buildings {bld}) events {}: bg {} npc {} warp {} trigger {}",
        d.zones.len(),
        d.matrices.len(),
        d.containers.len(),
        d.events.len(),
        n(|e| e.bg.len()),
        n(|e| e.npcs.len()),
        n(|e| e.warps.len()),
        n(|e| e.triggers.len()),
    );
    assert_eq!(wb, 572);
    // Warps point at real zones (0xFFFF = 6 special/dynamic warps).
    for (i, e) in d.events.iter().enumerate() {
        for w in &e.warps {
            assert!(
                w.dest_zone == 0xFFFF || (w.dest_zone as usize) < d.zones.len(),
                "zone {i} warp"
            );
        }
    }
}

/// Nuvema Town (the player's hometown): zone 389, place name 4 in text bank 89.
#[test]
fn golden_nuvema_town() {
    let Some(d) = load() else {
        nds_rom::testenv::rom_missing("ROM files not found");
        return;
    };
    let z = &d.zones[389];
    assert_eq!((z.zone_id, z.name_id, z.matrix_id), (389, 4, 0));
    let m = &d.matrices[z.matrix_id as usize];
    assert_eq!((m.width, m.height, m.flags), (29, 27, 1));
    let e = &d.events[389];
    assert_eq!(
        (e.bg.len(), e.npcs.len(), e.warps.len(), e.triggers.len()),
        (5, 6, 4, 2)
    );
    let dests: Vec<u16> = e.warps.iter().map(|w| w.dest_zone).collect();
    assert_eq!(dests, [390, 392, 394, 396]);
    // The four warp targets are house interiors that share the town's place name.
    for dz in dests {
        assert_eq!(d.zones[dz as usize].name_id, 4);
    }
    // Zone 6 and 389 both lie on the 29x27 overworld matrix.
    assert_eq!(d.zones[6].matrix_id, 0);
}
