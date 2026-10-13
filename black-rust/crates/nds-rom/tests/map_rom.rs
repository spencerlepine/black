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

/// The matrix layout the game reads (`FUN_02159D04`, overlay 10; F2a): with `flags == 1` the land plane
/// (`width * height` container ids) comes first, then the zone plane. In matrix 0 (the 29x27 overworld) every
/// overworld block's container is the model `mapXX_YY` of that very column XX and row YY, and Nuvema Town (zone
/// 389) is block (24, 23) on container 0 (`map24_23`). The old interleaved reading paired neighbouring words.
#[test]
fn matrix_planes_match_the_game() {
    let Some(d) = load() else {
        nds_rom::testenv::rom_missing("ROM files not found");
        return;
    };
    let m = &d.matrices[0];
    assert_eq!((m.flags, m.width, m.height), (1, 29, 27));
    assert_eq!(m.zone_at(24, 23), Some(389));
    assert_eq!(m.map_at(24, 23), Some(0));
    assert_eq!(m.zone_at(23, 23), Some(317));
    let mut named = 0;
    for y in 0..m.height as usize {
        for x in 0..m.width as usize {
            let Some(c) = m.map_at(x, y) else { continue };
            let con = &d.containers[c as usize];
            if !con.is_wb() {
                continue;
            }
            let b = nds_rom::g3d::model::Bmd0::parse(con.model().unwrap()).unwrap();
            let name = &b.models[0].name;
            let Some((xs, ys)) = name.strip_prefix("map").and_then(|r| r.split_once('_')) else {
                continue;
            };
            if let (Ok(mx), Ok(my)) = (xs.parse::<usize>(), ys.parse::<usize>()) {
                assert_eq!(
                    (mx, my),
                    (x, y),
                    "matrix 0 block ({x}, {y}) has container {c} = {name}"
                );
                named += 1;
            }
        }
    }
    assert!(named > 100, "only {named} blocks carry a mapXX_YY model");
    // Both two-plane matrices: the zone plane holds real zone ids (or the empty marker).
    for i in [0, 175] {
        let m = &d.matrices[i];
        let zones = m.cells_b.as_ref().unwrap();
        assert_eq!(zones.len(), m.cells.len());
        for (k, &z) in zones.iter().enumerate() {
            let no = nds_rom::map::matrix::NO_MAP;
            assert!(
                z == no || (z as usize) < d.zones.len(),
                "matrix {i} block {k} zone {z}"
            );
        }
    }
    assert_eq!(d.matrices.iter().filter(|m| m.flags == 1).count(), 2);
}

/// The 36-byte NPC records are the MMDL HEADERs of `fldmmdl.c` (F4a-spec: `FUN_0216CFC4` / `FUN_0216D040` /
/// `FUN_0216D0F8` in overlay 10): `kind` (+0x18) is 0 (grid) or 1 (rail), and a grid record's height (+0x20) is fx32,
/// mostly whole tiles (`n << 16`; 86 records are half / quarter tiles). `load` checks the byte round trip.
#[test]
fn npc_records_are_mmdl_headers() {
    let Some(d) = load() else {
        nds_rom::testenv::rom_missing("ROM files not found");
        return;
    };
    let (mut grid, mut rail, mut whole) = (0, 0, 0);
    for (i, e) in d.events.iter().enumerate() {
        for n in &e.npcs {
            match n.kind {
                0 => {
                    grid += 1;
                    whole += (n.height & 0xFFFF == 0) as usize;
                }
                1 => rail += 1,
                k => panic!("zone {i} npc {}: kind {k}", n.id),
            }
        }
    }
    assert_eq!((grid, rail, whole), (2217, 40, 2131));
}

/// The BG / warp / trigger records as `eventdata_system.c` reads them (overlay 10, F6a; layouts in `black-field`'s
/// README), checked over all 428 ARC 125 members (`load` checks the byte round trip): background kind 2 = hidden
/// item with script `8000 + i` (i < 0xB1, the hidden-item table) and facing rules 0..6; warp exit codes 0..4, rail
/// warps (`pos_kind` 1) and grid warps whose door is at a tile centre; trigger `unused` is 0 and grid heights are in
/// `fx32 >> 12` units.
#[test]
fn bg_warp_trigger_records_match_eventdata_system() {
    let Some(d) = load() else {
        nds_rom::testenv::rom_missing("ROM files not found");
        return;
    };
    use std::collections::BTreeMap;
    let mut bg_kind: BTreeMap<_, usize> = BTreeMap::new();
    let mut facing: BTreeMap<_, usize> = BTreeMap::new();
    let (mut bg_rail, mut hidden, mut bg_y16) = (0, Vec::new(), 0);
    let mut exit: BTreeMap<_, usize> = BTreeMap::new();
    let mut door: BTreeMap<_, usize> = BTreeMap::new();
    let mut size: BTreeMap<_, usize> = BTreeMap::new();
    let (mut w_rail, mut w_centre, mut w_grid, mut w_0x100, mut w_off) = (0, 0, 0, 0, 0);
    let mut t_kind: BTreeMap<_, usize> = BTreeMap::new();
    let (mut t_rail, mut t_var, mut t_unused, mut t_y16, mut t_grid) = (0, 0, 0, 0, 0);
    for e in &d.events {
        for b in &e.bg {
            *bg_kind.entry(b.kind).or_insert(0) += 1;
            *facing.entry(b.facing).or_insert(0) += 1;
            if b.pos_kind != 0 {
                bg_rail += 1;
            } else if b.y % 16 == 0 {
                bg_y16 += 1;
            }
            if b.kind == 2 {
                hidden.push(b.script);
            }
        }
        for w in &e.warps {
            *exit.entry(w.exit_dir).or_insert(0) += 1;
            *door.entry(w.door).or_insert(0) += 1;
            w_0x100 += (w.dest_warp == 0x100) as usize;
            w_off += (w.dest_zone == 0xFFFF || w.dest_warp == 0xFFFF) as usize;
            if w.pos_kind == 1 {
                w_rail += 1;
            } else {
                w_grid += 1;
                *size.entry((w.width, w.depth)).or_insert(0) += 1;
                w_centre += (w.x & 15 == 8 && w.z & 15 == 8) as usize;
            }
        }
        for t in &e.triggers {
            *t_kind.entry(t.kind).or_insert(0) += 1;
            t_var += (t.var != 0) as usize;
            t_unused += (t.unused != 0) as usize;
            if t.pos_kind != 0 {
                t_rail += 1;
            } else {
                t_grid += 1;
                t_y16 += (t.y % 16 == 0) as usize;
            }
        }
    }
    fn c<K: Ord>(m: &BTreeMap<K, usize>, k: K) -> usize {
        m.get(&k).copied().unwrap_or(0)
    }
    // background events: 184 hidden items, scripts 8000..=8176 (all 0xB1 items of the table, 7 placed twice)
    assert_eq!(
        (c(&bg_kind, 0), c(&bg_kind, 1), c(&bg_kind, 2)),
        (351, 127, 184)
    );
    assert!(facing.keys().all(|&f| f <= 6), "facing rules {facing:?}");
    assert_eq!(
        (c(&facing, 4), c(&facing, 0), c(&facing, 6)),
        (325, 235, 71)
    );
    assert_eq!((bg_rail, bg_y16), (25, 611));
    hidden.sort();
    hidden.dedup();
    assert_eq!(hidden, (8000..=8176).collect::<Vec<u16>>());
    // warps: exit codes 0..4, 77 rail warps, every grid door at a tile centre, 6 disabled
    assert!(exit.keys().all(|&x| x <= 4), "exit codes {exit:?}");
    assert_eq!(c(&door, 0), 1);
    assert_eq!(
        (w_rail, w_grid, w_centre, w_0x100, w_off),
        (77, 809, 809, 0, 6)
    );
    assert_eq!(
        (c(&size, (1, 1)), c(&size, (3, 1)), c(&size, (1, 3))),
        (476, 195, 70)
    );
    // triggers: kinds 0..4 and 6, `unused` always 0
    assert!(
        t_kind.keys().all(|&k| k <= 4 || k == 6),
        "trigger kinds {t_kind:?}"
    );
    assert_eq!((c(&t_kind, 0), c(&t_kind, 6)), (232, 110));
    assert_eq!(
        (t_rail, t_grid, t_var, t_unused, t_y16),
        (10, 363, 350, 0, 348)
    );
}
