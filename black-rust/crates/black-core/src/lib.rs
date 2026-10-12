//! Safe Rust port of `battle_record.c` and `unk_02008574.c` from the Pokemon Black decompilation.
//!
//! Undecompiled / platform routines (heap, battle entry initialisers) are modelled as traits so
//! the logic can be tested on a host machine.

pub mod battle_common;
pub mod battle_record;
pub mod heap;
pub mod ids;
pub mod mi_memory;
pub mod poke_crypt;
pub mod rng;
pub mod softfloat;
pub mod types;
pub mod unk_02008574;
