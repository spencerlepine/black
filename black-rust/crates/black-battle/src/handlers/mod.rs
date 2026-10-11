//! Battle event handlers (abilities, ...): code stored as `.byte` blocks in asm/battle_calc.s
//! and asm/battle_view_core.s, read from the full overlay disassembly.

pub mod abilities;

#[cfg(test)]
pub(crate) mod testkit;
