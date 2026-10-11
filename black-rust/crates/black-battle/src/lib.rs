//! Safe Rust port of battle logic from overlay 93 of the Pokemon Black decompilation.
//!
//! Only `.s` (undecompiled) sources exist for this code. Each item keeps the original symbol in
//! its doc comment. Routines outside the ported set are traits in [`hooks`] so the logic can run
//! and be tested on a host. See `README.md` for the symbol mapping and deviations.

pub mod battle_mon;
pub mod calc;
pub mod event_factor;
pub mod event_var;
pub mod flow;
pub mod handlers;
pub mod hooks;
pub mod main_util;
pub mod poke_type;
pub mod sick_cont;

#[cfg(test)]
mod testutil;
