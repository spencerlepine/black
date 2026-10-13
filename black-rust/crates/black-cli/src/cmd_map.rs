//! `black map <zone-id>`: summary dump of one zone (header, matrix, events, containers).

use clap::Args;
use nds_rom::map::{
    MapContainer, MapMatrix, ZoneEvents, ZoneHeader, ARC_MAP_CONTAINER, ARC_MAP_MATRIX,
    ARC_ZONE_EVENTS, ARC_ZONE_HEADERS,
};
use nds_rom::narc::Narc;
use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Args)]
pub struct MapArgs {
    /// Zone id (0..=426)
    pub zone: usize,
    /// ROM `files` directory (default: $BLACK_ROM_DIR/files)
    #[arg(long)]
    pub files: Option<PathBuf>,
}

fn open(files: &Path, id: u32) -> Result<Narc> {
    let p = files.join(format!("a/{}/{}/{}", id / 100, id / 10 % 10, id % 10));
    Ok(Narc::parse(
        &std::fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))?,
    )?)
}

pub fn run(a: MapArgs) -> Result<()> {
    let files = match a.files {
        Some(f) => f,
        None => PathBuf::from(
            std::env::var_os("BLACK_ROM_DIR").ok_or("pass --files or set BLACK_ROM_DIR")?,
        )
        .join("files"),
    };
    let zones = ZoneHeader::parse_table(open(&files, ARC_ZONE_HEADERS)?.file(0))?;
    let z = zones
        .get(a.zone)
        .ok_or_else(|| format!("zone {} out of range (0..{})", a.zone, zones.len()))?;
    println!("{z:#?}");
    let matrices = open(&files, ARC_MAP_MATRIX)?;
    let m = MapMatrix::parse(
        matrices
            .get(z.matrix_id as usize)
            .ok_or("matrix id out of range")?,
    )?;
    println!(
        "matrix {}: {}x{} flags {} ({} maps)",
        z.matrix_id,
        m.width,
        m.height,
        m.flags,
        m.cells
            .iter()
            .filter(|&&c| c != nds_rom::map::matrix::NO_MAP)
            .count()
    );
    let events = open(&files, ARC_ZONE_EVENTS)?;
    let e = ZoneEvents::parse(events.get(a.zone).ok_or("no event file")?)?;
    println!(
        "events: {} bg, {} npc, {} warp, {} trigger, {} tail bytes",
        e.bg.len(),
        e.npcs.len(),
        e.warps.len(),
        e.triggers.len(),
        e.tail.len()
    );
    for w in &e.warps {
        println!("  warp -> zone {:#06x}", w.dest_zone);
    }
    for n in &e.npcs {
        println!(
            "  npc {} obj {:#x} at ({}, {}) height {:#x}{}",
            n.id,
            n.obj_code,
            n.x,
            n.z,
            n.height,
            if n.kind == 1 { " (rail)" } else { "" }
        );
    }
    if m.width as usize * m.height as usize == 1 {
        let containers = open(&files, ARC_MAP_CONTAINER)?;
        let id = m.cells[0] as usize;
        let c = MapContainer::parse(containers.get(id).ok_or("map id out of range")?)?;
        println!(
            "map container {id}: magic {:?}, {} sections",
            String::from_utf8_lossy(&c.magic),
            c.sections.len()
        );
        if c.is_wb() {
            println!("  buildings: {}", c.buildings()?.len());
        }
    }
    Ok(())
}
