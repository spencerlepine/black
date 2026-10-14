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
    // S5b blocks
    s.situation.set_egg_steps(77);
    s.trcard.set_b602();
    s.perap.set_exists();
    s.sysinfo.game_time.count_down(10);
    s.record.inc(2, &mut crate::record::Plain);
    s.pms.unlock(5);
    s.mmdl.records[3].0[0] = 1;
    s.wfbc.ids[0] = 7;
    s.event_work.flags[1] = 0x80;
    s.battle_box.set_flags(4);
    s.daycare.put(0, &[9; crate::daycare::MON_SIZE]);
    s.mmdl_positions.positions[0] = [1, 2, 3];
    s.intrude.add_points_c(5);
    s.shortcut.set(4, true);
    s.zukan.set_seen(25, 0, 0);
    s.match_log.counters.inc(1, 2);
    s.pms_messages.set(1, &Pms::default());
    s.trial_house.log.add_count(0, 3);
    s.hall_of_fame.year = 2026;
    s.poke_records.records[1].pid = 0xDEAD;
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
    s.situation = Situation::from_bytes(&rnd(crate::situation::SIZE));
    s.trcard = TrCard::from_bytes(&rnd(crate::trcard::SIZE));
    s.perap = Perap::from_bytes(&rnd(crate::perapvoice::SIZE));
    s.sysinfo = SysInfo::from_bytes(&rnd(crate::sysinfo::SIZE));
    s.record = Record::from_bytes(&rnd(crate::record::SIZE));
    s.pms = PmsSave::from_bytes(&rnd(crate::pms_save::SIZE));
    s.mmdl = MmdlSave::from_bytes(&rnd(crate::mmdl_save::SIZE));
    s.wfbc = WfbcSave::from_bytes(&rnd(crate::wfbc_save::SIZE));
    s.event_work = EventWorkSave::from_bytes(&rnd(crate::field_save::EVENT_WORK_SIZE));
    s.battle_box = BattleBox::from_bytes(&rnd(crate::battle_box::SIZE));
    s.daycare = Daycare::from_bytes(&rnd(crate::daycare::SIZE));
    s.mmdl_positions = MmdlPositions::from_bytes(&rnd(crate::field_save::MMDL_POS_SIZE));
    s.intrude = IntrudeSave::from_bytes(&rnd(crate::intrude_save::SIZE));
    s.shortcut = Shortcut::from_bytes(&rnd(crate::shortcut::SIZE));
    s.zukan = Zukan::from_bytes(&rnd(crate::zukan_save::SIZE));
    s.match_log = MatchLog::from_bytes(&rnd(crate::mac_log::MATCH_SIZE));
    s.pms_messages = PmsMessages::from_bytes(&rnd(crate::pms_save::MESSAGES_SIZE));
    s.trial_house = TrialHouse::from_bytes(&rnd(crate::mac_log::SIZE));
    s.hall_of_fame = HofRecord::from_bytes(&rnd(crate::dendou_save::SIZE));
    s.poke_records = PokeRecords::from_bytes(&rnd(crate::poke_records::SIZE));
    // every S5b block model is lossless over arbitrary images
    type Conv = fn(&[u8]) -> Vec<u8>;
    let convs: [(usize, Conv); 20] = [
        (crate::situation::SIZE, |b| {
            Situation::from_bytes(b).to_bytes()
        }),
        (crate::trcard::SIZE, |b| TrCard::from_bytes(b).to_bytes()),
        (crate::perapvoice::SIZE, |b| Perap::from_bytes(b).to_bytes()),
        (crate::sysinfo::SIZE, |b| SysInfo::from_bytes(b).to_bytes()),
        (crate::record::SIZE, |b| Record::from_bytes(b).to_bytes()),
        (crate::pms_save::SIZE, |b| PmsSave::from_bytes(b).to_bytes()),
        (crate::mmdl_save::SIZE, |b| {
            MmdlSave::from_bytes(b).to_bytes()
        }),
        (crate::wfbc_save::SIZE, |b| {
            WfbcSave::from_bytes(b).to_bytes()
        }),
        (crate::field_save::EVENT_WORK_SIZE, |b| {
            EventWorkSave::from_bytes(b).to_bytes()
        }),
        (crate::battle_box::SIZE, |b| {
            BattleBox::from_bytes(b).to_bytes()
        }),
        (crate::daycare::SIZE, |b| Daycare::from_bytes(b).to_bytes()),
        (crate::field_save::MMDL_POS_SIZE, |b| {
            MmdlPositions::from_bytes(b).to_bytes()
        }),
        (crate::intrude_save::SIZE, |b| {
            IntrudeSave::from_bytes(b).to_bytes()
        }),
        (crate::shortcut::SIZE, |b| {
            Shortcut::from_bytes(b).to_bytes()
        }),
        (crate::zukan_save::SIZE, |b| Zukan::from_bytes(b).to_bytes()),
        (crate::mac_log::MATCH_SIZE, |b| {
            MatchLog::from_bytes(b).to_bytes()
        }),
        (crate::pms_save::MESSAGES_SIZE, |b| {
            PmsMessages::from_bytes(b).to_bytes()
        }),
        (crate::mac_log::SIZE, |b| {
            TrialHouse::from_bytes(b).to_bytes()
        }),
        (crate::dendou_save::SIZE, |b| {
            HofRecord::from_bytes(b).to_bytes()
        }),
        (crate::poke_records::SIZE, |b| {
            PokeRecords::from_bytes(b).to_bytes()
        }),
    ];
    for (size, conv) in convs {
        let img = rnd(size);
        assert_eq!(conv(&img), img, "block of {size:#x} bytes");
    }
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

#[test]
fn new_game_s5b_blocks() {
    let s = new_game();
    // block 28: +0x76 = 0x1AB, rest zero
    let b = s.situation.to_bytes();
    assert_eq!(&b[0x76..0x78], &[0xAB, 0x01]);
    assert_eq!(b.iter().filter(|&&x| x != 0).count(), 2);
    // block 37: game clock started (default RTC: zero), birthday / MAC zero
    assert_eq!(s.sysinfo.game_time.started, 1);
    assert_eq!(s.sysinfo.game_time.penalty, 0);
    // block 38: zero records (plaintext, key 0)
    assert_eq!(s.record.to_bytes(), vec![0; crate::record::SIZE]);
    // block 39: words 0xFFFF, group of kind 2 unlocked
    assert!(s.pms.is_unlocked(1));
    assert!(s.pms.sentences.iter().all(|p| p.words == [0xFFFF; 2]));
    // block 43: the town header and ids
    assert_eq!(s.wfbc.ids, [0xFFFF; 10]);
    // block 49: six cleared box Pokemon, the name of box 1
    let mut clear = vec![0u8; crate::battle_box::MON_SIZE];
    black_pokemon::create::clear_box_pokemon(&mut clear);
    assert!(s.battle_box.mons.iter().all(|m| m.0 == clear));
    assert_eq!(s.battle_box.name[0], u16::from(b'B'));
    // block 51: no kept positions
    assert!(s.mmdl_positions.to_bytes().iter().all(|&x| x == 0xFF));
    // block 53: +0x1A0 = 0x30
    assert_eq!(s.intrude.b1a0, 0x30);
    // block 54: all free
    assert_eq!(s.shortcut.to_bytes(), vec![25; crate::shortcut::SIZE]);
    // block 55: magic, nothing seen
    assert_eq!(s.zukan.magic, crate::zukan_save::MAGIC);
    assert_eq!(s.zukan.seen_count_national(), 0);
    // block 62: the default sentences
    assert_eq!(s.pms_messages.get(0).words[0], 0x58F);
    // zero blocks
    assert_eq!(s.trcard.to_bytes(), vec![0; crate::trcard::SIZE]);
    assert_eq!(s.daycare.to_bytes(), vec![0; crate::daycare::SIZE]);
    assert_eq!(
        s.event_work.to_bytes(),
        vec![0; crate::field_save::EVENT_WORK_SIZE]
    );
    assert_eq!(s.mmdl.to_bytes(), vec![0; crate::mmdl_save::SIZE]);
}

/// The SaveData of format 1 before S5b (the same fields, same order).
#[derive(Serialize)]
struct BeforeS5b<'a> {
    boxes: &'a BoxStorage,
    bag: &'a Bag,
    party: &'a Party,
    info: &'a SaveInfo,
    mailbox: &'a MailBox,
    regulation: &'a RegulationBlock,
    misc: &'a Misc,
    battle_rec: &'a BattleRecBlock,
    bsubway_play: &'a BsubwayPlay,
    bsubway_score: &'a BsubwayScore,
    symbols: &'a SymbolSave,
    battle_examination: &'a BattleExamination,
}

#[test]
fn format_1_save_without_s5b_blocks_still_loads() {
    let s = played();
    let old = BeforeS5b {
        boxes: &s.boxes,
        bag: &s.bag,
        party: &s.party,
        info: &s.info,
        mailbox: &s.mailbox,
        regulation: &s.regulation,
        misc: &s.misc,
        battle_rec: &s.battle_rec,
        bsubway_play: &s.bsubway_play,
        bsubway_score: &s.bsubway_score,
        symbols: &s.symbols,
        battle_examination: &s.battle_examination,
    };
    let payload = ron::to_string(&old).unwrap().into_bytes();
    let mut env = Vec::new();
    env.extend_from_slice(&MAGIC);
    env.extend_from_slice(&1u32.to_le_bytes());
    env.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    env.extend_from_slice(&crc32(&payload).to_le_bytes());
    env.extend_from_slice(&payload);
    let back = decode(&env).unwrap();
    assert_eq!(back.info, s.info);
    assert_eq!(back.symbols, s.symbols);
    // the S5b blocks come back in their init state (no message bank, clock or MAC)
    let fresh = SaveData::new_game(&mut NoNames, K);
    assert_eq!(back.zukan, fresh.zukan);
    assert_eq!(back.shortcut, fresh.shortcut);
    assert_eq!(back.battle_box, fresh.battle_box);
    assert_eq!(back.sysinfo, fresh.sysinfo);
    assert_eq!(back.mmdl_positions, fresh.mmdl_positions);
}
