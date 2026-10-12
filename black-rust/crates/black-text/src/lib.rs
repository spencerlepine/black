//! Message engine: string buffers, control codes, variable expansion, word wrap.
//!
//! Logic crate (D6): no Bevy dependency.

pub mod compress;
pub mod hooks;
pub mod layout;
pub mod speed;
pub mod strbuf;
pub mod token;
pub mod wordset;
