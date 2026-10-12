//! Shared IDs and enum values used across the logic crates.
//!
//! Plain integers and constants, matching how the original stores them. Type IDs are verified
//! through the battle type chart (`black-battle::poke_type`); the other orders follow the
//! standard Gen 5 data layout and are not yet checked against the ROM (§8).

pub type SpeciesId = u16;
pub type MoveId = u16;
pub type ItemId = u16;
pub type AbilityId = u16;
/// Index into a message bank.
pub type MsgId = u16;
/// Message bank (archive file) number.
pub type MsgBankId = u16;

/// Type IDs (Gen 5 order).
pub mod types {
    pub const NORMAL: u8 = 0;
    pub const FIGHTING: u8 = 1;
    pub const FLYING: u8 = 2;
    pub const POISON: u8 = 3;
    pub const GROUND: u8 = 4;
    pub const ROCK: u8 = 5;
    pub const BUG: u8 = 6;
    pub const GHOST: u8 = 7;
    pub const STEEL: u8 = 8;
    pub const FIRE: u8 = 9;
    pub const WATER: u8 = 10;
    pub const GRASS: u8 = 11;
    pub const ELECTRIC: u8 = 12;
    pub const PSYCHIC: u8 = 13;
    pub const ICE: u8 = 14;
    pub const DRAGON: u8 = 15;
    pub const DARK: u8 = 16;
    /// Number of real types (rows/columns of the type chart).
    pub const COUNT: u8 = 17;
    /// "No type" (`0x11`): always neutral and never matched by `has_type`.
    pub const NONE: u8 = 17;
}

/// Gender values (UNVERIFIED order).
pub mod gender {
    pub const MALE: u8 = 0;
    pub const FEMALE: u8 = 1;
    pub const GENDERLESS: u8 = 2;
}

/// Stat indices in personal-data order: HP, Attack, Defense, Speed, Sp. Atk, Sp. Def (UNVERIFIED).
pub mod stat {
    pub const HP: u8 = 0;
    pub const ATTACK: u8 = 1;
    pub const DEFENSE: u8 = 2;
    pub const SPEED: u8 = 3;
    pub const SP_ATTACK: u8 = 4;
    pub const SP_DEFENSE: u8 = 5;
    pub const COUNT: u8 = 6;
}

/// Natures, `PID % 25` (UNVERIFIED order).
pub mod nature {
    pub const HARDY: u8 = 0;
    pub const LONELY: u8 = 1;
    pub const BRAVE: u8 = 2;
    pub const ADAMANT: u8 = 3;
    pub const NAUGHTY: u8 = 4;
    pub const BOLD: u8 = 5;
    pub const DOCILE: u8 = 6;
    pub const RELAXED: u8 = 7;
    pub const IMPISH: u8 = 8;
    pub const LAX: u8 = 9;
    pub const TIMID: u8 = 10;
    pub const HASTY: u8 = 11;
    pub const SERIOUS: u8 = 12;
    pub const JOLLY: u8 = 13;
    pub const NAIVE: u8 = 14;
    pub const MODEST: u8 = 15;
    pub const MILD: u8 = 16;
    pub const QUIET: u8 = 17;
    pub const BASHFUL: u8 = 18;
    pub const RASH: u8 = 19;
    pub const CALM: u8 = 20;
    pub const GENTLE: u8 = 21;
    pub const SASSY: u8 = 22;
    pub const CAREFUL: u8 = 23;
    pub const QUIRKY: u8 = 24;
    pub const COUNT: u8 = 25;
}
