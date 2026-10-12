//! Text measurement and layout (F9c). Main ARM9 Thumb code at `0x0201D26C..0x0201D338`
//! (`printsys` helpers), verified per function with `tools/check-fn.sh black-text layout`
//! (spec `tools/rom-diff/specs/black-text/layout.spec`).
//!
//! The original has NO automatic word wrap: message strings carry their own `0xFFFE` line breaks
//! and the printer (`FUN_0201C974`) only moves to the next line on `0xFFFE`. What the engine does
//! measure is a line's pixel width (to centre or right-align text, tag group `0xBD`) and the
//! longest line (to size a window). Those are [`line_width`] / [`max_line_width`]. [`wrap`],
//! [`paginate`] and [`line_count`] are conveniences for strings that do not come with breaks
//! (new dialogue); they are NOT ports and are UNVERIFIED.
//!
//! Glyph advances come from the font through [`GlyphWidth`] (the original calls a function
//! pointer in the font object, `FUN_0201DC6C` -> `FUN_0201DB20`, a binary search of the glyph
//! ranges); the Bevy side implements it with `nds_rom::gffont::GfFont::char_width`.

use crate::token::{END, NEWLINE};
use crate::wordset::TAG;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[0x0201_D26C, 0x0201_D304];

/// Advance of one character code in the active font (`FUN_0201DC6C(font, code)`).
pub trait GlyphWidth {
    /// Cursor advance of `c` in pixels.
    fn width(&self, c: u16) -> u8;
}

const SPACES: [u16; 2] = [0x0020, 0x3000];

fn at(s: &[u16], i: usize) -> u16 {
    s.get(i).copied().unwrap_or(END)
}

/// C: `FUN_0201D26C(str, font, spacing, &next)`: pixel width of the line starting at `start`:
/// every character adds `spacing + glyph advance`, tags (`3 + n` units) add nothing, the line ends
/// at `0xFFFE` or `0xFFFF` (or the end of the slice). Returns the width and the index where the
/// next line starts (past the `0xFFFE`; at the `0xFFFF` otherwise).
pub fn line_width(s: &[u16], start: usize, spacing: i32, g: &dyn GlyphWidth) -> (i32, usize) {
    let mut p = start;
    let mut w = 0i32;
    loop {
        let c = at(s, p);
        if c == NEWLINE || c == END {
            break;
        }
        if c == TAG {
            p += 3 + at(s, p + 2) as usize;
        } else {
            w = w.wrapping_add(spacing.wrapping_add(g.width(c) as i32));
            p += 1;
        }
    }
    let next = if at(s, p) == NEWLINE { p + 1 } else { p };
    (w, next)
}

/// C: `FUN_0201D304(str, font, spacing)`: the width of the longest line of the string.
pub fn max_line_width(s: &[u16], spacing: i32, g: &dyn GlyphWidth) -> u32 {
    let mut p = 0;
    let mut best = 0u32;
    while at(s, p) != END {
        let (w, next) = line_width(s, p, spacing, g);
        best = best.max(w as u32);
        p = next;
    }
    best
}

/// Number of lines (`0xFFFE` count + 1) of a string, like `FUN_0201D42C` before it multiplies by
/// the line height. UNVERIFIED (no spec).
pub fn line_count(s: &[u16]) -> usize {
    let mut p = 0;
    let mut n = 1;
    while at(s, p) != END {
        match at(s, p) {
            NEWLINE => {
                n += 1;
                p += 1;
            }
            TAG => p += 3 + at(s, p + 2) as usize,
            _ => p += 1,
        }
    }
    n
}

/// Inserts `0xFFFE` so that no line is wider than `max_width` pixels: break at the last space
/// (`0x0020` or `0x3000`, which becomes the break) or, when a line has none, before the
/// overflowing character. Existing breaks and tags are kept. UNVERIFIED: the original never wraps
/// (see the module docs); the space codes are an assumption.
pub fn wrap(s: &[u16], max_width: i32, spacing: i32, g: &dyn GlyphWidth) -> Vec<u16> {
    let mut out: Vec<u16> = Vec::with_capacity(s.len() + 8);
    let mut cur = 0i32;
    let mut last_space: Option<usize> = None;
    let mut p = 0;
    while p < s.len() {
        let c = s[p];
        if c == END {
            out.push(END);
            break;
        }
        if c == NEWLINE {
            out.push(c);
            cur = 0;
            last_space = None;
            p += 1;
        } else if c == TAG {
            let end = (p + 3 + at(s, p + 2) as usize).min(s.len());
            out.extend_from_slice(&s[p..end]);
            p = end;
        } else {
            let w = spacing + g.width(c) as i32;
            if cur + w > max_width && cur > 0 {
                if SPACES.contains(&c) {
                    out.push(NEWLINE);
                    cur = 0;
                    last_space = None;
                    p += 1;
                    continue;
                }
                if let Some(si) = last_space {
                    out[si] = NEWLINE;
                    cur = line_width(&out, si + 1, spacing, g).0;
                } else {
                    out.push(NEWLINE);
                    cur = 0;
                }
                last_space = None;
            }
            out.push(c);
            cur += w;
            if SPACES.contains(&c) {
                last_space = Some(out.len() - 1);
            }
            p += 1;
        }
    }
    out
}

/// Splits a string into pages of at most `lines` lines: a page ends at a `BE01` tag (page clear;
/// the tag is dropped) or after `lines` lines (the `0xFFFE` is dropped). `BE00` (wait for a button)
/// stays in the text. UNVERIFIED: the command meanings are from `nds_rom` notes (A3) and the
/// printer's group `0xBE` switch (`FUN_0201D110`), not traced end to end.
pub fn paginate(s: &[u16], lines: usize) -> Vec<Vec<u16>> {
    let lines = lines.max(1);
    let mut pages = vec![Vec::new()];
    let mut n = 1;
    let mut p = 0;
    while p < s.len() && s[p] != END {
        let c = s[p];
        if c == TAG {
            let end = (p + 3 + at(s, p + 2) as usize).min(s.len());
            if at(s, p + 1) == 0xBE01 {
                pages.push(Vec::new());
                n = 1;
            } else {
                pages.last_mut().unwrap().extend_from_slice(&s[p..end]);
            }
            p = end;
        } else if c == NEWLINE && n == lines {
            pages.push(Vec::new());
            n = 1;
            p += 1;
        } else {
            if c == NEWLINE {
                n += 1;
            }
            pages.last_mut().unwrap().push(c);
            p += 1;
        }
    }
    pages
}

#[cfg(test)]
mod tests;
