//! The save data model and its persistence (S5).
//!
//! [`SaveData`] is the port's own save: one field per modelled save block of the original (the
//! DS layout is `savedata.c`'s table at 0x0209DA28: 69 blocks, `(id, size fn, init fn)`), each a
//! plain serde type built from the block's layout and accessors. The party, PC box and bag
//! blocks keep their original byte images because their ported logic (S1-S3) works on those
//! images. Blocks no item has modelled yet are not stored (see the crate README for the list).
//!
//! Persistence is not the DS save (no flash sectors, no block checksums, no mirror slots): a
//! [`SaveData`] is serialised to RON inside a versioned envelope ([`encode`] / [`decode`]) and
//! the envelope bytes go to a [`Storage`] under a slot key. [`FileStorage`] (native only) keeps
//! one file per slot; the browser storage (localStorage / IndexedDB) implements [`Storage`] in
//! `black-game` (G9); [`MemoryStorage`] is for tests and as a fallback.
//!
//! Envelope (little-endian): `b"BLKSAVE\0"`, `u32` format version, `u32` payload length,
//! `u32` CRC-32 (IEEE) of the payload, then the payload (UTF-8 RON of [`SaveData`]). A later
//! format bumps [`FORMAT_VERSION`] and adds a step to [`decode`]'s migration.

use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::battle_examination::BattleExamination;
use crate::battle_rec::BattleRecBlock;
use crate::box_data::{self, BoxEnv};
use crate::bsubway::{BsubwayPlay, BsubwayScore};
use crate::mail::{GameConsts, MailBox};
use crate::misc::{Misc, MiscTextHooks};
use crate::party::{self, PartyHooks};
use crate::regulation::RegulationBlock;
use crate::save_info::SaveInfo;
use crate::symbol_save::{Plain, SymbolSave};

/// Format version written by [`encode`].
pub const FORMAT_VERSION: u32 = 1;
/// Envelope magic.
pub const MAGIC: [u8; 8] = *b"BLKSAVE\0";
/// Envelope header length (magic, version, length, CRC).
pub const HEADER_LEN: usize = 20;
/// Bytes of the bag block (block 25, `sub_02007CB0`).
pub const BAG_SIZE: usize = 0x9C0;
/// Number of PC boxes (blocks 1..=24).
pub const BOXES: usize = box_data::MAX_BOXES as usize;

/// Byte images serialised as one lowercase hex string (compact in RON).
pub(crate) mod hex {
    use serde::{de::Error, Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(b: &[u8], s: S) -> Result<S::Ok, S::Error> {
        let mut out = String::with_capacity(b.len() * 2);
        for x in b {
            out.push_str(&format!("{x:02x}"));
        }
        s.serialize_str(&out)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let s = String::deserialize(d)?;
        if s.len() % 2 != 0 {
            return Err(D::Error::custom("odd hex length"));
        }
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(D::Error::custom))
            .collect()
    }
}

/// The PC boxes: block 0 (the 0x3E0-byte box manager: names, wallpapers, current box, unlocked
/// count) and blocks 1..=24 (one 0xFF0-byte box each), as the `box_data` port uses them.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoxStorage {
    /// Block 0.
    #[serde(with = "hex")]
    pub meta: Vec<u8>,
    /// Blocks 1..=24.
    pub trays: Vec<Tray>,
}

/// One PC box image (0xFF0 bytes).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tray(#[serde(with = "hex")] pub Vec<u8>);

/// The bag, block 25 (0x9C0 bytes, the `bag` port's image).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bag(#[serde(with = "hex")] pub Vec<u8>);

/// The party, block 26 (0x534 bytes, the `party` port's image).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Party(#[serde(with = "hex")] pub Vec<u8>);

/// What a new game needs from outside the logic crates.
pub trait NewGameHooks {
    /// The default name of PC box `i` (0..24), the 0x28-byte string `FUN_02045F5C` copies from
    /// message bank 9, entry `i + 6`; `None` when the bank is missing (the names stay empty, as
    /// in the original).
    fn box_name(&mut self, i: u32) -> Option<[u8; box_data::NAME_SIZE]>;
    /// Message `entry` of bank 0x2F as a 9-unit name (`FUN_02045EC0` + `FUN_02045A38`): the two
    /// default names of block 52 (entries 0x151 and 0x159).
    fn misc_name(&mut self, entry: u32) -> [u16; 9];
}

/// `MiscTextHooks` for the new game: a string handle is its message entry.
struct NewMisc<'a, H: NewGameHooks>(&'a mut H);

impl<H: NewGameHooks> MiscTextHooks for NewMisc<'_, H> {
    fn msg_open(&mut self) -> u32 {
        1
    }
    fn msg_string(&mut self, _msg: u32, id: u32) -> u32 {
        id
    }
    fn str_to_name(&mut self, s: u32, dst: &mut [u16; 9]) {
        *dst = self.0.misc_name(s);
    }
    fn str_free(&mut self, _s: u32) {}
    fn msg_close(&mut self, _msg: u32) {}
}

/// The whole save.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveData {
    /// Blocks 0..=24.
    pub boxes: BoxStorage,
    /// Block 25.
    pub bag: Bag,
    /// Block 26.
    pub party: Party,
    /// Block 27: config, player status, play time.
    pub info: SaveInfo,
    /// Block 40.
    pub mailbox: MailBox,
    /// Block 47.
    pub regulation: RegulationBlock,
    /// Block 52: money, badges, counters.
    pub misc: Misc,
    /// Block 48.
    pub battle_rec: BattleRecBlock,
    /// Block 57.
    pub bsubway_play: BsubwayPlay,
    /// Block 58.
    pub bsubway_score: BsubwayScore,
    /// Block 61.
    pub symbols: SymbolSave,
    /// Extra save area 5.
    pub battle_examination: BattleExamination,
}

/// `PartyHooks` for the new-game party: the record clear is `black-pokemon`'s port.
struct NewParty;

impl PartyHooks for NewParty {
    fn mon_param(&mut self, _mon_addr: u32, _id: u32) -> u32 {
        unreachable!("not called by the party init")
    }
    fn clear_mon(&mut self, mon: &mut [u8]) {
        black_pokemon::create::clear_pokemon(mon);
    }
    fn alloc(&mut self, _args: [u32; 5]) -> u32 {
        unreachable!("not called by the party init")
    }
    fn free(&mut self, _ptr: u32) {
        unreachable!("not called by the party init")
    }
}

/// `BoxEnv` for the new-game box manager: address 0 is the meta image, the names come from the
/// hooks.
struct NewMeta<'a, H: NewGameHooks> {
    meta: &'a mut [u8],
    h: &'a mut H,
    names: Option<Vec<[u8; box_data::NAME_SIZE]>>,
}

impl<H: NewGameHooks> BoxEnv for NewMeta<'_, H> {
    fn meta_addr(&mut self, _mgr: u32) -> u32 {
        0
    }
    fn tray_addr(&mut self, _mgr: u32, _tray: u32) -> u32 {
        unreachable!("not called by the meta init")
    }
    fn mem(&mut self, addr: u32) -> &mut [u8] {
        &mut self.meta[addr as usize..]
    }
    fn save_block(&mut self, _savedata: u32, _block: u32) -> u32 {
        unreachable!("not called by the meta init")
    }
    fn poke_get(&mut self, _mon: u32, _id: u32, _buf: u32) -> u32 {
        unreachable!("not called by the meta init")
    }
    fn clear_mon(&mut self, _mon: u32) {
        unreachable!("not called by the meta init")
    }
    fn fixup_mon(&mut self, _mon: u32) {
        unreachable!("not called by the meta init")
    }
    fn meta_init(&mut self, _meta: u32) {
        unreachable!("not called by the meta init")
    }
    fn put_tray(&mut self, _mgr: u32, _tray: u32, _mon: u32) -> u32 {
        unreachable!("not called by the meta init")
    }
    fn occupied_count(&mut self, _mgr: u32, _tray: u32) -> u32 {
        unreachable!("not called by the meta init")
    }
    fn non_egg_count(&mut self, _mgr: u32, _tray: u32) -> u32 {
        unreachable!("not called by the meta init")
    }
    fn name_to_str(&mut self, _dst: u32, _name: &[u8]) {
        unreachable!("not called by the meta init")
    }
    fn str_to_name(&mut self, _src: u32, _name: &mut [u8], _n: u32) {
        unreachable!("not called by the meta init")
    }
    fn msg_string(&mut self, _msg: u32, idx: u32, name: &mut [u8], _n: u32) {
        if let Some(n) = &self.names {
            name.copy_from_slice(&n[(idx - 6) as usize]);
        }
    }
    fn msg_bank(&mut self) -> u32 {
        let names: Option<Vec<_>> = (0..BOXES as u32).map(|i| self.h.box_name(i)).collect();
        let ok = names.is_some() as u32;
        self.names = names;
        ok
    }
    fn msg_free(&mut self, _msg: u32) {}
    fn alloc(&mut self, _args: [u32; 5]) -> u32 {
        unreachable!("not called by the meta init")
    }
    fn free(&mut self, _ptr: u32) {
        unreachable!("not called by the meta init")
    }
    fn assert_fail(&mut self) -> ! {
        unreachable!("not called by the meta init")
    }
}

impl SaveData {
    /// A new game: every modelled block as its init function leaves a zeroed block (the
    /// original clears the whole save before calling the inits): box manager `FUN_02007464`,
    /// boxes `sub_02007AD0` (30 `clear_box_pokemon` each), bag `sub_02007CB8` (zero), party
    /// `FUN_0201A948`, block 27 `FUN_02008704`, mailbox `FUN_02009004`, regulation
    /// `sub_0200B348`, block 52 `sub_0200C058`, battle recorder `sub_0200B368`, Battle Subway `FUN_0200D85C` /
    /// `sub_0200DA6C`, symbols `sub_0200E098`, battle examination `sub_0200E590`.
    pub fn new_game<H: NewGameHooks>(h: &mut H, k: GameConsts) -> Self {
        let mut meta = vec![0u8; box_data::META_SIZE];
        box_data::init_meta(
            &mut NewMeta {
                meta: &mut meta,
                h,
                names: None,
            },
            0,
        );
        let trays = (0..BOXES)
            .map(|_| {
                let mut t = vec![0u8; box_data::TRAY_SIZE];
                for s in 0..box_data::SLOTS {
                    let o = s * box_data::MON_SIZE;
                    black_pokemon::create::clear_box_pokemon(&mut t[o..o + box_data::MON_SIZE]);
                }
                Tray(t)
            })
            .collect();
        let mut party = vec![0u8; party::PARTY_SIZE];
        party::init_default(&mut party, &mut NewParty);
        let mut misc = Misc::default();
        misc.init(&mut NewMisc(h));
        let mut symbols = SymbolSave::default();
        symbols.init(&mut Plain);
        SaveData {
            boxes: BoxStorage { meta, trays },
            bag: Bag(vec![0; BAG_SIZE]),
            party: Party(party),
            info: SaveInfo::new_game(),
            mailbox: MailBox::new_game(k),
            regulation: RegulationBlock::default(),
            misc,
            battle_rec: BattleRecBlock::new_game(),
            bsubway_play: BsubwayPlay::default(),
            bsubway_score: BsubwayScore::new_game(),
            symbols,
            battle_examination: BattleExamination::default(),
        }
    }
}

/// Why a save could not be read or written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SaveError {
    /// Not an envelope (too short or wrong magic).
    NotASave,
    /// Written by a newer format than this build knows.
    TooNew(u32),
    /// A format version no migration exists for.
    UnknownVersion(u32),
    /// Payload length or CRC mismatch (a truncated or damaged save).
    Corrupt,
    /// The payload does not parse as this version's model.
    Format(String),
    /// The storage backend failed.
    Storage(String),
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaveError::NotASave => write!(f, "not a save file"),
            SaveError::TooNew(v) => write!(f, "save format {v} is newer than this build"),
            SaveError::UnknownVersion(v) => write!(f, "unknown save format {v}"),
            SaveError::Corrupt => write!(f, "save data is damaged"),
            SaveError::Format(e) => write!(f, "save data does not parse: {e}"),
            SaveError::Storage(e) => write!(f, "storage error: {e}"),
        }
    }
}

impl std::error::Error for SaveError {}

const fn crc_table() -> [u32; 256] {
    let mut t = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        let mut c = i as u32;
        let mut k = 0;
        while k < 8 {
            c = if c & 1 != 0 {
                0xEDB8_8320 ^ (c >> 1)
            } else {
                c >> 1
            };
            k += 1;
        }
        t[i] = c;
        i += 1;
    }
    t
}

const CRC_TABLE: [u32; 256] = crc_table();

/// CRC-32 (IEEE 802.3, reflected, init and final xor 0xFFFFFFFF).
pub fn crc32(data: &[u8]) -> u32 {
    !data.iter().fold(!0u32, |c, &b| {
        CRC_TABLE[((c ^ b as u32) & 0xFF) as usize] ^ (c >> 8)
    })
}

/// Serialises a save into the versioned envelope.
pub fn encode(save: &SaveData) -> Vec<u8> {
    let payload = ron::to_string(save)
        .expect("SaveData serialises")
        .into_bytes();
    let mut out = Vec::with_capacity(HEADER_LEN + payload.len());
    out.extend_from_slice(&MAGIC);
    out.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    out.extend_from_slice(&crc32(&payload).to_le_bytes());
    out.extend_from_slice(&payload);
    out
}

/// Reads an envelope: checks magic, length and CRC, then parses the payload of its format
/// version (migrating older formats when there are any).
pub fn decode(bytes: &[u8]) -> Result<SaveData, SaveError> {
    if bytes.len() < HEADER_LEN || bytes[..8] != MAGIC {
        return Err(SaveError::NotASave);
    }
    let word = |o: usize| u32::from_le_bytes(bytes[o..o + 4].try_into().unwrap());
    let (version, len, crc) = (word(8), word(12) as usize, word(16));
    let payload = &bytes[HEADER_LEN..];
    if payload.len() != len || crc32(payload) != crc {
        return Err(SaveError::Corrupt);
    }
    let text = std::str::from_utf8(payload).map_err(|e| SaveError::Format(e.to_string()))?;
    match version {
        FORMAT_VERSION => ron::from_str(text).map_err(|e| SaveError::Format(e.to_string())),
        v if v > FORMAT_VERSION => Err(SaveError::TooNew(v)),
        v => Err(SaveError::UnknownVersion(v)),
    }
}

/// Byte storage by slot key (`"main"`, `"backup"`, ...). Keys are `[A-Za-z0-9_-]{1,64}`
/// ([`valid_slot`]). Implemented natively by [`FileStorage`], in memory by [`MemoryStorage`],
/// and in the browser by `black-game` (localStorage / IndexedDB).
pub trait Storage {
    /// The bytes stored under `slot`, `None` if there are none.
    fn load(&self, slot: &str) -> Result<Option<Vec<u8>>, SaveError>;
    /// Stores `bytes` under `slot`, replacing what was there (atomically where the backend can).
    fn save(&mut self, slot: &str, bytes: &[u8]) -> Result<(), SaveError>;
    /// Deletes `slot` (no error when it does not exist).
    fn remove(&mut self, slot: &str) -> Result<(), SaveError>;
}

/// Is `slot` a valid slot key.
pub fn valid_slot(slot: &str) -> bool {
    (1..=64).contains(&slot.len())
        && slot
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

fn check_slot(slot: &str) -> Result<(), SaveError> {
    if valid_slot(slot) {
        Ok(())
    } else {
        Err(SaveError::Storage(format!("invalid slot key {slot:?}")))
    }
}

/// Encodes `save` and stores it under `slot`.
pub fn write_save<S: Storage + ?Sized>(
    storage: &mut S,
    slot: &str,
    save: &SaveData,
) -> Result<(), SaveError> {
    storage.save(slot, &encode(save))
}

/// Loads and decodes the save under `slot`; `None` if the slot is empty.
pub fn read_save<S: Storage + ?Sized>(
    storage: &S,
    slot: &str,
) -> Result<Option<SaveData>, SaveError> {
    storage.load(slot)?.map(|b| decode(&b)).transpose()
}

/// In-memory storage.
#[derive(Clone, Debug, Default)]
pub struct MemoryStorage {
    slots: HashMap<String, Vec<u8>>,
}

impl Storage for MemoryStorage {
    fn load(&self, slot: &str) -> Result<Option<Vec<u8>>, SaveError> {
        check_slot(slot)?;
        Ok(self.slots.get(slot).cloned())
    }
    fn save(&mut self, slot: &str, bytes: &[u8]) -> Result<(), SaveError> {
        check_slot(slot)?;
        self.slots.insert(slot.to_string(), bytes.to_vec());
        Ok(())
    }
    fn remove(&mut self, slot: &str) -> Result<(), SaveError> {
        check_slot(slot)?;
        self.slots.remove(slot);
        Ok(())
    }
}

/// One file per slot (`<dir>/<slot>.sav`), written through a temporary file and a rename.
/// Native only (std::fs).
#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Debug)]
pub struct FileStorage {
    dir: std::path::PathBuf,
}

#[cfg(not(target_arch = "wasm32"))]
impl FileStorage {
    /// Storage in `dir` (created on the first save).
    pub fn new(dir: impl Into<std::path::PathBuf>) -> Self {
        FileStorage { dir: dir.into() }
    }

    fn path(&self, slot: &str) -> Result<std::path::PathBuf, SaveError> {
        check_slot(slot)?;
        Ok(self.dir.join(format!("{slot}.sav")))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn io_err(e: std::io::Error) -> SaveError {
    SaveError::Storage(e.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
impl Storage for FileStorage {
    fn load(&self, slot: &str) -> Result<Option<Vec<u8>>, SaveError> {
        match std::fs::read(self.path(slot)?) {
            Ok(b) => Ok(Some(b)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(io_err(e)),
        }
    }
    fn save(&mut self, slot: &str, bytes: &[u8]) -> Result<(), SaveError> {
        let path = self.path(slot)?;
        std::fs::create_dir_all(&self.dir).map_err(io_err)?;
        let tmp = self.dir.join(format!("{slot}.sav.tmp"));
        std::fs::write(&tmp, bytes).map_err(io_err)?;
        std::fs::rename(&tmp, &path).map_err(io_err)
    }
    fn remove(&mut self, slot: &str) -> Result<(), SaveError> {
        match std::fs::remove_file(self.path(slot)?) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(io_err(e)),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests;
