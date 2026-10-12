//! ROM -> `assets/data/*.ron` extractor (native only, `rom` feature).

use crate::egg_moves::EggMoveTable;
use crate::encounters::EncounterData;
use crate::evolution::EvolutionTable;
use crate::growth::GrowthTables;
use crate::items::ItemTable;
use crate::learnset::LearnsetTable;
use crate::loader::file;
use crate::moves::MoveTable;
use crate::personal::PersonalTable;
use crate::tmhm::TmHmTable;
use crate::trainers::TrainerTable;
use crate::tutors::TutorTable;
use crate::types::TypeChart;
use std::path::{Path, PathBuf};

/// Where the converters find their inputs.
#[derive(Debug, Clone)]
pub struct RomSource {
    /// The extracted ROM file system (`.../black-rom/files`), holding `a/0/1/6` etc.
    pub files: PathBuf,
    /// Overlay 93 image (`OVY_93.sbin`), source of the type chart.
    pub overlay93: PathBuf,
    /// Main ARM9 image (`main.sbin`), source of the TM/HM table.
    pub main_sbin: PathBuf,
}

/// RAM address the type chart lives at inside overlay 93.
const OVERLAY93_BASE: u32 = 0x021B_6100;

impl RomSource {
    /// From the `black-rom` root: `files/` and `build/black.us/OVY_93.sbin`.
    pub fn from_root(root: &Path) -> Self {
        RomSource {
            files: root.join("files"),
            overlay93: root.join("build/black.us/OVY_93.sbin"),
            main_sbin: root.join("build/black.us/main.sbin"),
        }
    }

    fn arc(&self, path: &str) -> Result<Vec<u8>, String> {
        let p = self.files.join(path);
        std::fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))
    }
}

/// The twelve tables, converted from the ROM.
pub struct Tables {
    pub personal: PersonalTable,
    pub learnset: LearnsetTable,
    pub evolution: EvolutionTable,
    pub moves: MoveTable,
    pub items: ItemTable,
    pub trainers: TrainerTable,
    pub encounters: EncounterData,
    pub growth: GrowthTables,
    pub types: TypeChart,
    pub tmhm: TmHmTable,
    pub egg_moves: EggMoveTable,
    pub tutors: TutorTable,
}

/// Reads and converts every table.
pub fn read_tables(src: &RomSource) -> Result<Tables, String> {
    let overlay =
        std::fs::read(&src.overlay93).map_err(|e| format!("{}: {e}", src.overlay93.display()))?;
    let main =
        std::fs::read(&src.main_sbin).map_err(|e| format!("{}: {e}", src.main_sbin.display()))?;
    let personal = PersonalTable::from_narc(&src.arc("a/0/1/6")?).map_err(|e| e.to_string())?;
    Ok(Tables {
        tutors: TutorTable::from_personal(&personal).ok_or("tutor bits outside 0..=6")?,
        tmhm: TmHmTable::from_main(&main).ok_or("main.sbin too short for the TM/HM table")?,
        egg_moves: EggMoveTable::from_narc(&src.arc("a/1/2/3")?)?,
        personal,
        learnset: LearnsetTable::from_narc(&src.arc("a/0/1/8")?)?,
        evolution: EvolutionTable::from_narcs(&src.arc("a/0/1/9")?, &src.arc("a/0/2/0")?)?,
        moves: MoveTable::from_narc(&src.arc("a/0/2/1")?)?,
        items: ItemTable::from_narc(&src.arc("a/0/2/4")?)?,
        trainers: TrainerTable::from_narcs(&src.arc("a/0/9/2")?, &src.arc("a/0/9/3")?)?,
        encounters: EncounterData::from_narc(&src.arc("a/1/2/6")?)?,
        growth: GrowthTables::from_narc(&src.arc("a/0/1/7")?)?,
        types: TypeChart::from_overlay93(&overlay, OVERLAY93_BASE)
            .ok_or("type chart not found in overlay 93")?,
    })
}

/// Serialises every table to its RON text, paired with its file name.
pub fn to_texts(t: &Tables) -> Result<Vec<(&'static str, String)>, String> {
    fn s<T: serde::Serialize>(v: &T) -> Result<String, String> {
        crate::to_ron(v).map_err(|e| e.to_string())
    }
    Ok(vec![
        (file::PERSONAL, s(&t.personal)?),
        (file::LEARNSET, s(&t.learnset)?),
        (file::EVOLUTION, s(&t.evolution)?),
        (file::MOVES, s(&t.moves)?),
        (file::ITEMS, s(&t.items)?),
        (file::TRAINERS, s(&t.trainers)?),
        (file::ENCOUNTERS, s(&t.encounters)?),
        (file::GROWTH, s(&t.growth)?),
        (file::TYPES, s(&t.types)?),
        (file::TMHM, s(&t.tmhm)?),
        (file::EGG_MOVES, s(&t.egg_moves)?),
        (file::TUTORS, s(&t.tutors)?),
    ])
}

/// Writes `<out_dir>/data/<table>.ron` for every table; returns the paths written.
pub fn extract_data(src: &RomSource, out_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let texts = to_texts(&read_tables(src)?)?;
    let dir = out_dir.join("data");
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    texts
        .into_iter()
        .map(|(name, text)| {
            let p = dir.join(name);
            std::fs::write(&p, text).map_err(|e| format!("{}: {e}", p.display()))?;
            Ok(p)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::*;

    fn root() -> PathBuf {
        std::env::var_os("BLACK_ROM_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../black-rom"))
    }

    /// ROM -> struct -> text -> struct == original, for every table.
    #[test]
    fn all_tables_round_trip() {
        let mut src = RomSource::from_root(&root());
        if let Some(b) = std::env::var_os("BLACK_ROM_BUILD") {
            src.overlay93 = PathBuf::from(&b).join("OVY_93.sbin");
            src.main_sbin = PathBuf::from(&b).join("main.sbin");
        }
        if !src.files.join("a/0/1/6").exists() || !src.overlay93.exists() || !src.main_sbin.exists()
        {
            eprintln!("skipping: ROM files missing");
            return;
        }
        let t = read_tables(&src).unwrap();
        let texts = to_texts(&t).unwrap();
        let get = |n: &str| texts.iter().find(|(k, _)| *k == n).unwrap().1.as_bytes();
        assert_eq!(load_personal(get(file::PERSONAL)).unwrap(), t.personal);
        assert_eq!(load_learnset(get(file::LEARNSET)).unwrap(), t.learnset);
        assert_eq!(load_evolution(get(file::EVOLUTION)).unwrap(), t.evolution);
        assert_eq!(load_moves(get(file::MOVES)).unwrap(), t.moves);
        assert_eq!(load_items(get(file::ITEMS)).unwrap(), t.items);
        assert_eq!(load_trainers(get(file::TRAINERS)).unwrap(), t.trainers);
        assert_eq!(
            load_encounters(get(file::ENCOUNTERS)).unwrap(),
            t.encounters
        );
        assert_eq!(load_growth(get(file::GROWTH)).unwrap(), t.growth);
        assert_eq!(load_types(get(file::TYPES)).unwrap(), t.types);
        assert_eq!(load_tmhm(get(file::TMHM)).unwrap(), t.tmhm);
        assert_eq!(load_egg_moves(get(file::EGG_MOVES)).unwrap(), t.egg_moves);
        assert_eq!(load_tutors(get(file::TUTORS)).unwrap(), t.tutors);

        let out = std::env::temp_dir().join(format!("black-data-{}", std::process::id()));
        let written = extract_data(&src, &out).unwrap();
        assert_eq!(written.len(), 12);
        let bytes = std::fs::read(out.join("data").join(file::MOVES)).unwrap();
        assert_eq!(load_moves(&bytes).unwrap(), t.moves);
        std::fs::remove_dir_all(&out).ok();
    }
}
