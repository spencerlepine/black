use super::*;
use crate::box_data::{COUNT_OFS, FLAGS_OFS, NAMES_OFS, NAME_SIZE, WALLPAPER_OFS};
use crate::config::Config;
use crate::mail::{Mail, Pms};
use crate::mystatus::MyStatus;
use crate::playtime::PlayTime;
use crate::symbol_save::SymbolRecord;

/// Box names "Box 1".."Box 24" as 0x28-byte strings (any bytes do for the model).
struct Names;

impl NewGameHooks for Names {
    fn box_name(&mut self, i: u32) -> Option<[u8; NAME_SIZE]> {
        let mut n = [0xFFu8; NAME_SIZE];
        for (k, c) in format!("Box {}", i + 1).encode_utf16().enumerate() {
            n[2 * k..2 * k + 2].copy_from_slice(&c.to_le_bytes());
        }
        Some(n)
    }
    fn misc_name(&mut self, entry: u32) -> [u16; 9] {
        [entry as u16, 0xFFFF, 0, 0, 0, 0, 0, 0, 0]
    }
}

/// No message bank.
struct NoNames;

impl NewGameHooks for NoNames {
    fn box_name(&mut self, _i: u32) -> Option<[u8; NAME_SIZE]> {
        None
    }
    fn misc_name(&mut self, _entry: u32) -> [u16; 9] {
        [0xFFFF; 9]
    }
}

const K: GameConsts = GameConsts::BLACK_US;

fn new_game() -> SaveData {
    SaveData::new_game(&mut Names, K)
}

/// A save with every modelled block away from its new-game state.
fn played() -> SaveData {
    let mut s = new_game();
    s.info.config.set_text_speed(2);
    s.info.my_status.name = [0x41, 0x42, 0x43, 0xFFFF, 0, 0, 0, 0];
    s.info.my_status.set_id(0x1234_5678);
    s.info.play_time.add_seconds(123_456);
    s.info.mark_extra(3);
    let mut m = Mail::empty(K);
    m.create(K, 4, &s.info.my_status);
    m.set_pms(
        &Pms {
            kind: 1,
            id: 2,
            words: [3, 4],
        },
        1,
    );
    s.mailbox.set(5, &m);
    s.regulation.cards[1].regulation.set(4, 50);
    s.regulation.cards[0].set_param(2, 0xFFFF_FF80);
    s.regulation.summaries[0].set(0, 2, 495);
    s.battle_rec.start(7);
    s.misc.add_money(3000);
    s.misc.give_badge(2);
    s.bsubway_play.set(3, &[0x34, 0x12]);
    s.bsubway_score.add_bp(321);
    s.symbols.add(
        SymbolRecord {
            species: 25,
            a: 1,
            b: 2,
            c: 3,
            d: 4,
        },
        3,
        &mut Plain,
    );
    s.battle_examination.flags = 0x8005;
    s.bag.0[0] = 4;
    s.bag.0[2] = 10;
    s.party.0[4] = 1;
    s.boxes.trays[23].0[0x100] = 0xAB;
    s
}

#[test]
fn round_trip_new_game() {
    let s = new_game();
    assert_eq!(decode(&encode(&s)).unwrap(), s);
}

#[test]
fn round_trip_played() {
    let s = played();
    let bytes = encode(&s);
    let back = decode(&bytes).unwrap();
    assert_eq!(back, s);
    // and the original images survive too
    assert_eq!(back.info.to_bytes(), s.info.to_bytes());
    assert_eq!(back.mailbox.to_bytes(), s.mailbox.to_bytes());
    assert_eq!(back.symbols.to_bytes(), s.symbols.to_bytes());
}

#[test]
fn round_trip_random_images() {
    // Every block model is lossless over arbitrary images, so a save built from random block
    // bytes survives encode/decode bit for bit.
    let mut x = 0x1234_5678u32;
    let mut rnd = |n: usize| -> Vec<u8> {
        (0..n)
            .map(|_| {
                x = x.wrapping_mul(1_103_515_245).wrapping_add(12345);
                (x >> 16) as u8
            })
            .collect()
    };
    let mut s = new_game();
    s.info = SaveInfo::from_bytes(&rnd(crate::save_info::SIZE));
    s.mailbox = MailBox::from_bytes(&rnd(crate::mail::MAILBOX_SIZE));
    s.regulation = RegulationBlock::from_bytes(&rnd(crate::regulation::BLOCK_SIZE));
    s.battle_rec = BattleRecBlock::from_bytes(&rnd(crate::battle_rec::SIZE));
    s.bsubway_play = BsubwayPlay::from_bytes(&rnd(crate::bsubway::PLAY_SIZE));
    s.bsubway_score = BsubwayScore::from_bytes(&rnd(crate::bsubway::SCORE_SIZE));
    s.symbols = SymbolSave::from_bytes(&rnd(crate::symbol_save::SIZE));
    s.battle_examination = BattleExamination::from_bytes(&rnd(crate::battle_examination::SIZE));
    s.bag = Bag(rnd(BAG_SIZE));
    s.misc = Misc::from_bytes(&rnd(crate::misc::SIZE));
    let back = decode(&encode(&s)).unwrap();
    assert_eq!(back, s);
}

#[test]
fn new_game_matches_block_inits() {
    let s = SaveData::new_game(&mut NoNames, K);
    // block 0: FUN_02007464 without a message bank: wallpapers i % 16, current 0, 8 boxes
    let meta = &s.boxes.meta;
    assert_eq!(meta.len(), box_data::META_SIZE);
    for i in 0..BOXES {
        assert_eq!(meta[WALLPAPER_OFS + i] as usize, i % 16);
    }
    assert_eq!(meta[FLAGS_OFS], 0);
    assert_eq!(meta[COUNT_OFS], 8);
    assert!(meta[..WALLPAPER_OFS].iter().all(|&b| b == 0));
    // with names
    let named = new_game();
    assert_eq!(
        &named.boxes.meta[NAMES_OFS..NAMES_OFS + 4],
        &[b'B', 0, b'o', 0]
    );
    // blocks 1..24: 30 cleared BoxPokemon each
    let mut tray = vec![0u8; box_data::TRAY_SIZE];
    for k in 0..box_data::SLOTS {
        let o = k * box_data::MON_SIZE;
        black_pokemon::create::clear_box_pokemon(&mut tray[o..o + box_data::MON_SIZE]);
    }
    assert!(s.boxes.trays.iter().all(|t| t.0 == tray));
    assert_eq!(s.boxes.trays.len(), 24);
    // block 25: zero
    assert_eq!(s.bag.0, vec![0; BAG_SIZE]);
    // block 26: capacity 6, count 0
    assert_eq!(&s.party.0[..8], &[6, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(s.party.0.len(), party::PARTY_SIZE);
    // block 27: FUN_02008704 = config / mystatus / playtime inits on a cleared block
    let mut want = [0u8; crate::save_info::SIZE];
    want[..4].copy_from_slice(&Config::new_game().to_bytes());
    want[4..0x24].copy_from_slice(&MyStatus::new_game().to_bytes());
    want[0x24..0x2C].copy_from_slice(&PlayTime::new_game().to_bytes());
    assert_eq!(s.info.to_bytes(), want);
    assert_eq!(&want[..4], &[1, 2, 0, 0]);
    assert_eq!((want[4 + 0x1A], want[4 + 0x1B]), (2, 0x15));
    // block 40: 20 cleared mails
    let mail = Mail::empty(K).to_bytes();
    assert!(s.mailbox.to_bytes().chunks(0x38).all(|m| m == mail));
    // block 47: zero
    assert_eq!(
        s.regulation.to_bytes(),
        vec![0; crate::regulation::BLOCK_SIZE]
    );
    // block 52: names from the bank, +0x47..+0x49 = 0xFF, rest zero
    let m = s.misc.to_bytes();
    assert!(m[0x20..0x44].iter().all(|&b| b == 0xFF));
    assert_eq!(&m[0x44..0x4A], &[0, 0, 0, 0xFF, 0xFF, 0xFF]);
    assert!(m[..0x20].iter().chain(&m[0x4A..]).all(|&b| b == 0));
    assert_eq!(new_game().misc.name_20[0], 0x151);
    // block 48: id -1, rest zero
    let br = s.battle_rec.to_bytes();
    assert_eq!(&br[..4], &[0xFF; 4]);
    assert!(br[4..].iter().all(|&b| b == 0));
    // blocks 57 / 58: zero; +3 of the score = 1
    assert_eq!(s.bsubway_play.to_bytes(), [0; crate::bsubway::PLAY_SIZE]);
    let sc = s.bsubway_score.to_bytes();
    assert_eq!(sc[3], 1);
    assert_eq!(sc.iter().filter(|&&b| b != 0).count(), 1);
    // block 61: zero (plaintext; the original seals it)
    assert_eq!(s.symbols.to_bytes(), vec![0; crate::symbol_save::SIZE]);
    // extra area 5: zero
    assert_eq!(
        s.battle_examination.to_bytes(),
        vec![0; crate::battle_examination::SIZE]
    );
}

#[test]
fn envelope_rejects_damage_and_versions() {
    let bytes = encode(&new_game());
    assert_eq!(&bytes[..8], b"BLKSAVE\0");
    assert_eq!(&bytes[8..12], &FORMAT_VERSION.to_le_bytes());
    let mut bad = bytes.clone();
    *bad.last_mut().unwrap() ^= 1;
    assert_eq!(decode(&bad), Err(SaveError::Corrupt));
    assert_eq!(decode(&bytes[..bytes.len() - 1]), Err(SaveError::Corrupt));
    assert_eq!(decode(b"nope"), Err(SaveError::NotASave));
    let mut newer = bytes.clone();
    newer[8..12].copy_from_slice(&(FORMAT_VERSION + 1).to_le_bytes());
    assert_eq!(decode(&newer), Err(SaveError::TooNew(FORMAT_VERSION + 1)));
    let mut older = bytes;
    older[8..12].copy_from_slice(&0u32.to_le_bytes());
    assert_eq!(decode(&older), Err(SaveError::UnknownVersion(0)));
}

#[test]
fn crc32_check_value() {
    assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
}

#[test]
fn memory_storage_round_trip() {
    let mut st = MemoryStorage::default();
    assert_eq!(read_save(&st, "main").unwrap(), None);
    let s = played();
    write_save(&mut st, "main", &s).unwrap();
    assert_eq!(read_save(&st, "main").unwrap(), Some(s));
    st.remove("main").unwrap();
    assert_eq!(read_save(&st, "main").unwrap(), None);
    assert!(st.save("../x", b"").is_err());
    assert!(!valid_slot(""));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn file_storage_round_trip() {
    let dir = std::env::temp_dir().join(format!("black-storage-save-{}", std::process::id()));
    let mut st = FileStorage::new(&dir);
    assert_eq!(read_save(&st, "slot_1").unwrap(), None);
    let s = played();
    write_save(&mut st, "slot_1", &s).unwrap();
    assert_eq!(read_save(&st, "slot_1").unwrap(), Some(s.clone()));
    // a second save replaces the first
    let mut t = s;
    t.info.play_time.add_seconds(1);
    write_save(&mut st, "slot_1", &t).unwrap();
    assert_eq!(read_save(&st, "slot_1").unwrap(), Some(t));
    st.remove("slot_1").unwrap();
    st.remove("slot_1").unwrap();
    assert_eq!(read_save(&st, "slot_1").unwrap(), None);
    let _ = std::fs::remove_dir_all(&dir);
}
