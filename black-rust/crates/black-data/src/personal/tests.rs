use super::*;
use crate::{from_ron, to_ron};
use black_core::ids::{stat, types};

fn sample() -> Personal {
    Personal {
        base_stats: [45, 49, 49, 45, 65, 65],
        types: [types::GRASS, types::POISON],
        catch_rate: 45,
        stage: 1,
        ev_yield: [0, 0, 0, 0, 1, 0],
        unk_0a_hi: 0,
        held_items: [0, 0, 0],
        gender_ratio: 31,
        egg_cycles: 20,
        base_friendship: 70,
        growth_rate: 3,
        egg_groups: [1, 7],
        abilities: [65, 0, 34],
        flee_rate: 0,
        form_stats_index: 0,
        form_sprite_index: 0,
        form_count: 1,
        color: 3,
        unk_21_bit6: false,
        unk_21_bit7: false,
        base_exp: 64,
        height: 7,
        weight: 69,
        tm_hm: [0x8439_8720, 0x0011_9a08, 0xa260_0420, 4],
        tutors: Some(1),
    }
}

#[test]
fn ron_round_trip_synthetic() {
    let mut short = sample();
    short.tutors = None;
    let table = PersonalTable(vec![sample(), short]);
    let text = to_ron(&table).unwrap();
    assert_eq!(from_ron::<PersonalTable>(&text).unwrap(), table);
}

#[test]
fn helpers() {
    let p = sample();
    assert!(!p.is_genderless());
    assert!(p.can_learn_tm_hm(5));
    assert!(!p.can_learn_tm_hm(0));
    assert_eq!(p.base_stats[stat::SP_ATTACK as usize], 65);
}

#[cfg(feature = "rom")]
mod rom {
    use super::*;
    use nds_rom::msg::Bank;
    use nds_rom::narc::Narc;
    use std::path::PathBuf;

    fn files_dir() -> PathBuf {
        std::env::var_os("BLACK_ROM_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../black-rom"))
            .join("files/a/0")
    }

    fn load(path: &str) -> Option<Vec<u8>> {
        let p = files_dir().join(path);
        if !p.exists() {
            nds_rom::testenv::rom_missing(&format!("{} not found", p.display()));
            return None;
        }
        Some(std::fs::read(p).unwrap())
    }

    fn table() -> Option<(Vec<u8>, PersonalTable)> {
        let bytes = load("1/6")?;
        let t = PersonalTable::from_narc(&bytes).unwrap();
        Some((bytes, t))
    }

    #[test]
    fn byte_round_trip_all_members() {
        let Some((bytes, table)) = table() else {
            return;
        };
        let narc = Narc::parse(&bytes).unwrap();
        assert_eq!(narc.len(), 669);
        assert_eq!(table.0.len(), 668);
        for (i, p) in table.0.iter().enumerate() {
            assert_eq!(p.to_bytes().unwrap(), narc.file(i), "member {i}");
        }
        let trailer = PersonalTable::trailer_from_narc(&bytes).unwrap();
        assert_eq!(trailer.len(), 650);
        let rebuilt = table.to_narc(&trailer).unwrap();
        let again = Narc::parse(&rebuilt).unwrap();
        assert_eq!(again.len(), narc.len());
        for i in 0..narc.len() {
            assert_eq!(again.file(i), narc.file(i), "rebuilt member {i}");
        }
    }

    #[test]
    fn odd_sizes() {
        let Some((bytes, table)) = table() else {
            return;
        };
        let narc = Narc::parse(&bytes).unwrap();
        let odd: Vec<_> = (0..narc.len())
            .filter(|&i| narc.file(i).len() != 60)
            .map(|i| (i, narc.file(i).len()))
            .collect();
        assert_eq!(odd, [(0, 56), (668, 1300)]);
        assert_eq!(table.0[0].tutors, None);
    }

    #[test]
    fn ron_round_trip_real() {
        let Some((_, table)) = table() else {
            return;
        };
        let text = to_ron(&table).unwrap();
        assert_eq!(from_ron::<PersonalTable>(&text).unwrap(), table);
    }

    #[test]
    fn spot_checks() {
        let Some((_, t)) = table() else {
            return;
        };
        let g = |i: usize| t.get(i).unwrap();
        let bulb = g(1);
        assert_eq!(bulb.base_stats, [45, 49, 49, 45, 65, 65]);
        assert_eq!(bulb.types, [types::GRASS, types::POISON]);
        assert_eq!(bulb.catch_rate, 45);
        assert_eq!(bulb.ev_yield, [0, 0, 0, 0, 1, 0]);
        assert_eq!(bulb.abilities, [65, 0, 34]);
        assert_eq!(bulb.gender_ratio, 31);
        assert_eq!((bulb.base_exp, bulb.height, bulb.weight), (64, 7, 69));

        let snivy = g(495);
        assert_eq!(snivy.base_stats, [45, 45, 55, 63, 45, 55]);
        assert_eq!(snivy.types, [types::GRASS, types::GRASS]);
        assert_eq!(snivy.ev_yield, [0, 0, 0, 1, 0, 0]);
        assert_eq!(snivy.abilities, [65, 65, 126]);

        let victini = g(494);
        assert_eq!(victini.base_stats, [100; 6]);
        assert_eq!(victini.types, [types::PSYCHIC, types::FIRE]);
        assert_eq!(victini.ev_yield, [3, 0, 0, 0, 0, 0]);
        assert_eq!(victini.abilities, [162, 0, 0]);
        assert_eq!(victini.catch_rate, 3);
        assert!(victini.is_genderless());

        let gengar = g(94);
        assert_eq!(gengar.types, [types::GHOST, types::POISON]);
        assert_eq!(gengar.base_stats, [60, 65, 60, 110, 130, 75]);

        let magnemite = g(81);
        assert!(magnemite.is_genderless());
        assert_eq!(magnemite.types, [types::ELECTRIC, types::STEEL]);

        let pikachu = g(25);
        assert_eq!(pikachu.base_stats, [35, 55, 30, 90, 50, 40]);
        assert_eq!(pikachu.abilities, [9, 0, 31]);
    }

    #[test]
    fn species_names_line_up() {
        let (Some((_, t)), Some(arc)) = (table(), load("0/2")) else {
            return;
        };
        let narc = Narc::parse(&arc).unwrap();
        let bank = Bank::parse(narc.file(70)).unwrap();
        let name = |i: usize| bank.entries()[i].render();
        assert_eq!(name(1), "Bulbasaur");
        assert_eq!(name(495), "Snivy");
        assert_eq!(name(494), "Victini");
        assert_eq!(t.0[494].base_stats, [100; 6]);
    }
}
