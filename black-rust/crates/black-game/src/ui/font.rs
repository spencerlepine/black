//! Bitmap font metrics (G5): the pure half of the text renderer.
//!
//! Reads the `fontN.metrics.ron` that `black extract-assets --only fonts` writes (see
//! `black-cli/src/cmd_extract_assets.rs::do_fonts`; the loader never touches the NFTR in the ROM,
//! D4). The file is a fixed, line-oriented RON subset, so it is parsed by hand and the app needs
//! no `ron`/`serde` dependency:
//!
//! ```text
//! ( atlas: "font0.png", columns: 64, cell_w: 12, cell_h: 15, baseline: 16, max_width: 13,
//!   bpp: 2, line_feed: 15,
//!   glyphs: [ (atlas index, UTF-16 code or 65535, left, glyph width, advance), ... ], )
//! ```
//!
//! Atlas cell `n` sits at column `n % columns`, row `n / columns`, each `cell_w x cell_h` pixels;
//! the glyph ink starts at the cell's left edge and is drawn at `pen_x + left`. The advance feeds
//! `black_text::layout::GlyphWidth`.

use std::collections::HashMap;

use black_text::layout::GlyphWidth;

/// Metrics of one glyph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Glyph {
    /// Cell number in the atlas.
    pub index: u32,
    /// Left bearing in pixels (added to the pen x when drawing).
    pub left: u8,
    /// Ink width in pixels.
    pub width: u8,
    /// Cursor advance in pixels.
    pub advance: u8,
}

/// One font's layout data.
#[derive(Debug, Clone)]
pub struct FontMetrics {
    /// Atlas PNG file name (next to the metrics file).
    pub atlas: String,
    pub columns: u32,
    pub cell_w: u32,
    pub cell_h: u32,
    pub baseline: u32,
    pub max_width: u32,
    pub bpp: u32,
    /// Line height in pixels (the font's own line feed).
    pub line_feed: u32,
    glyphs: HashMap<u16, Glyph>,
}

fn num(line: &str, key: &str) -> Option<u32> {
    let rest = line
        .trim()
        .strip_prefix(key)?
        .trim_start()
        .strip_prefix(':')?;
    rest.trim().trim_end_matches(',').trim().parse().ok()
}

impl FontMetrics {
    /// Parse the text of a `fontN.metrics.ron`.
    pub fn parse(text: &str) -> Result<FontMetrics, String> {
        let (mut atlas, mut columns, mut cell_w, mut cell_h) = (None, None, None, None);
        let (mut baseline, mut max_width, mut bpp, mut line_feed) = (None, None, None, None);
        let mut glyphs = HashMap::new();
        for line in text.lines() {
            let t = line.trim();
            if t.starts_with("//") {
                continue;
            }
            if let Some(r) = t.strip_prefix("atlas:") {
                atlas = Some(r.trim().trim_end_matches(',').trim_matches('"').to_string());
            } else if let Some(v) = t.strip_prefix('(').and_then(|r| r.strip_suffix("),")) {
                let f: Vec<u32> = v
                    .split(',')
                    .map(|x| x.trim().parse().map_err(|_| format!("bad glyph line: {t}")))
                    .collect::<Result<_, _>>()?;
                if let [index, code, left, width, advance] = f[..] {
                    if code != 0xFFFF {
                        let g = Glyph {
                            index,
                            left: left as u8,
                            width: width as u8,
                            advance: advance as u8,
                        };
                        glyphs.insert(code as u16, g);
                    }
                } else {
                    return Err(format!("glyph needs 5 fields: {t}"));
                }
            } else {
                columns = columns.or(num(t, "columns"));
                cell_w = cell_w.or(num(t, "cell_w"));
                cell_h = cell_h.or(num(t, "cell_h"));
                baseline = baseline.or(num(t, "baseline"));
                max_width = max_width.or(num(t, "max_width"));
                bpp = bpp.or(num(t, "bpp"));
                line_feed = line_feed.or(num(t, "line_feed"));
            }
        }
        let miss = |k: &str| format!("metrics: missing `{k}`");
        Ok(FontMetrics {
            atlas: atlas.ok_or_else(|| miss("atlas"))?,
            columns: columns.ok_or_else(|| miss("columns"))?,
            cell_w: cell_w.ok_or_else(|| miss("cell_w"))?,
            cell_h: cell_h.ok_or_else(|| miss("cell_h"))?,
            baseline: baseline.ok_or_else(|| miss("baseline"))?,
            max_width: max_width.ok_or_else(|| miss("max_width"))?,
            bpp: bpp.ok_or_else(|| miss("bpp"))?,
            line_feed: line_feed.ok_or_else(|| miss("line_feed"))?,
            glyphs,
        })
    }

    /// Glyph for a UTF-16 / game character code.
    pub fn glyph(&self, code: u16) -> Option<&Glyph> {
        self.glyphs.get(&code)
    }

    /// Top-left pixel `(x, y)` of atlas cell `index`.
    pub fn cell_origin(&self, index: u32) -> (u32, u32) {
        (
            (index % self.columns) * self.cell_w,
            (index / self.columns) * self.cell_h,
        )
    }

    /// Number of mapped codes.
    pub fn len(&self) -> usize {
        self.glyphs.len()
    }

    /// True when no code is mapped.
    pub fn is_empty(&self) -> bool {
        self.glyphs.is_empty()
    }
}

impl GlyphWidth for FontMetrics {
    /// The advance; unmapped codes advance 0 (nothing is drawn for them).
    fn width(&self, c: u16) -> u8 {
        self.glyph(c).map_or(0, |g| g.advance)
    }
}

/// A glyph positioned inside a text block (pixels from the block's top-left).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placed {
    pub code: u16,
    pub x: i32,
    pub y: i32,
}

impl FontMetrics {
    /// Pen positions for `units` (`0xFFFE` starts a new line `line_pitch` pixels lower; tags and
    /// the end marker draw nothing; each character advances by `spacing + advance`, exactly the
    /// measure `black_text::layout::line_width` uses). Characters without a glyph are skipped but
    /// still advance by 0.
    pub fn place(&self, units: &[u16], spacing: i32, line_pitch: i32) -> Vec<Placed> {
        let mut out = Vec::new();
        let (mut pen, mut y, mut p) = (0i32, 0i32, 0usize);
        while p < units.len() {
            let c = units[p];
            if c == black_text::token::END {
                break;
            }
            if c == black_text::token::NEWLINE {
                pen = 0;
                y += line_pitch;
                p += 1;
            } else if c == black_text::wordset::TAG {
                p += 3 + units.get(p + 2).copied().unwrap_or(0) as usize;
            } else {
                if let Some(g) = self.glyph(c) {
                    out.push(Placed {
                        code: c,
                        x: pen + g.left as i32,
                        y,
                    });
                }
                pen += spacing + self.width(c) as i32;
                p += 1;
            }
        }
        out
    }
}

/// Stand-in widths when no font is loaded (wasm before G3, or a missing `assets/`).
#[derive(Debug, Clone, Copy)]
pub struct FixedWidth(pub u8);

impl GlyphWidth for FixedWidth {
    fn width(&self, _c: u16) -> u8 {
        self.0
    }
}

/// Encode a Rust string into game character codes: ASCII/BMP chars map to themselves;
/// `\n` = line break (`0xFFFE`), `\x0c` = page clear (`BE01` tag), `\x07` = wait for button
/// (`BE00` tag). Used for the built-in demo strings; real text comes from the extracted banks.
pub fn encode(s: &str) -> Vec<u16> {
    let mut out = Vec::new();
    for ch in s.chars() {
        match ch {
            '\n' => out.push(black_text::token::NEWLINE),
            '\x0c' => out.extend_from_slice(&[black_text::wordset::TAG, 0xBE01, 0]),
            '\x07' => out.extend_from_slice(&[black_text::wordset::TAG, 0xBE00, 0]),
            c => out.extend_from_slice(c.encode_utf16(&mut [0; 2])),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "(\n    atlas: \"font0.png\",\n    columns: 64,\n    cell_w: 12,\n    cell_h: 15,\n    baseline: 16,\n    max_width: 13,\n    bpp: 2,\n    line_feed: 15,\n    // (atlas index, ...)\n    glyphs: [\n        (0, 32, 4, 0, 4),\n        (1, 33, 2, 2, 5),\n        (2, 65535, 0, 6, 6),\n        (70, 65, 0, 6, 6),\n    ],\n)\n";

    #[test]
    fn parses_header_and_glyphs() {
        let f = FontMetrics::parse(SAMPLE).unwrap();
        assert_eq!(
            (f.atlas.as_str(), f.columns, f.cell_w, f.cell_h),
            ("font0.png", 64, 12, 15)
        );
        assert_eq!(
            (f.baseline, f.max_width, f.bpp, f.line_feed),
            (16, 13, 2, 15)
        );
        assert_eq!(f.len(), 3, "the unmapped (65535) glyph is skipped");
        let ex = f.glyph(33).unwrap();
        assert_eq!((ex.index, ex.left, ex.width, ex.advance), (1, 2, 2, 5));
        assert_eq!(f.width(33), 5);
        assert_eq!(f.width(0x4242), 0);
        assert_eq!(f.cell_origin(70), (6 * 12, 15));
    }

    #[test]
    fn rejects_missing_fields_and_bad_glyphs() {
        assert!(FontMetrics::parse("(\n atlas: \"a.png\",\n)").is_err());
        let bad = SAMPLE.replace("(1, 33, 2, 2, 5),", "(1, 33, 2, 2),");
        assert!(FontMetrics::parse(&bad).is_err());
    }

    #[test]
    fn place_follows_line_width() {
        let f = FontMetrics::parse(SAMPLE).unwrap();
        // ' ' (adv 4, left 4, no ink), '!' (adv 5, left 2), newline, 'A' (adv 6)
        let units = [32, 33, 0xFFFE, 65, 0xF000, 0xBE00, 0, 33, 0xFFFF, 65];
        let p = f.place(&units, 1, 16);
        let at = |code, x, y| Placed { code, x, y };
        assert_eq!(
            p,
            vec![
                at(32, 4, 0),
                at(33, 5 + 2, 0),
                at(65, 0, 16),
                at(33, 7 + 2, 16)
            ]
        );
        let (w, _) = black_text::layout::line_width(&units, 0, 1, &f);
        assert_eq!(w, 5 + 6);
    }

    /// With `BLACK_ASSETS` pointing at a full extraction, the real font0 parses and has ASCII.
    #[test]
    fn real_extracted_font0_if_present() {
        let Some(dir) = std::env::var_os("BLACK_ASSETS") else {
            return;
        };
        let Ok(text) =
            std::fs::read_to_string(std::path::Path::new(&dir).join("fonts/font0.metrics.ron"))
        else {
            return;
        };
        let f = FontMetrics::parse(&text).unwrap();
        assert_eq!((f.cell_w, f.cell_h, f.bpp), (12, 15, 2));
        assert!(f.len() > 1000);
        for c in b'A'..=b'z' {
            assert!(f.glyph(u16::from(c)).is_some(), "missing {}", c as char);
        }
        assert_eq!(f.width(0x20), 4);
    }

    #[test]
    fn encode_controls() {
        use black_text::wordset::TAG;
        assert_eq!(encode("Hi\n!"), vec![72, 105, 0xFFFE, 33]);
        assert_eq!(encode("a\x0cb"), vec![97, TAG, 0xBE01, 0, 98]);
        assert_eq!(encode("\x07"), vec![TAG, 0xBE00, 0]);
    }
}
