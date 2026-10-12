//! Battle event handlers (abilities, held items, ...): code stored as `.byte` blocks in asm/battle_calc.s
//! and asm/battle_view_core.s, read from the full overlay disassembly.

pub mod abilities;
pub mod effects;
pub mod items;
pub mod moves;
pub mod registry;

#[cfg(test)]
pub(crate) mod hints;
#[cfg(test)]
pub(crate) mod testkit;
