//! Reusable UI kit (G5): bitmap-font text, window frames, message box, list menu, yes/no box and
//! number input.
//!
//! * `font`, `list`, `msgbox` are pure (no window) and unit-tested.
//! * `widgets` is the Bevy half: it loads the font atlas, draws glyph sprites pixel-exact and runs
//!   the widgets from `DsButtons` (plus the `BLACK_SCRIPT` injection of `screens::Script`).
//!
//! Fonts come from `assets/fonts/fontN.png` + `fontN.metrics.ron` written by
//! `black extract-assets --only fonts` (D4: nothing is read from the ROM at runtime). The assets
//! directory is `$BLACK_ASSETS` or `assets/`.

// The kit's public API is consumed by later phases; the binary cannot see that yet.
#![allow(dead_code)]

pub mod demo;
pub mod font;
pub mod list;
pub mod msgbox;
mod widgets;

pub use widgets::*;
