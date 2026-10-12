//! Message box state machine (G5): paging + typewriter reveal. Pure, driven by `tick()` (one call
//! per 60 Hz frame) and `press()` (A or B).
//!
//! Text goes through `black-text`: `layout::wrap` (our own helper, the original never wraps) with
//! the font's `GlyphWidth`, `layout::paginate` into pages of N lines (`BE01` = clear page),
//! `token::parse` to split characters from control codes, `speed::{pace, Pacer}` for the reveal
//! rate. `BE00` ("wait for a button", kept in the page by `paginate`) pauses the reveal until
//! A/B. Other tags are not interpreted yet (colour, variables are expanded earlier by `wordset`).

use black_text::layout::{paginate, wrap, GlyphWidth};
use black_text::speed::{pace, Pacer};
use black_text::token::{parse, Token};

/// One element of a page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Char(u16),
    NewLine,
    /// `BE00`: stop revealing until the player presses A/B.
    Wait,
}

/// Where the box is in its life.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsgState {
    /// Revealing characters.
    Typing,
    /// All text of the page shown (`mid == false`, show the arrow) or stopped at a `BE00`
    /// (`mid == true`); waiting for A/B.
    Waiting { mid: bool },
    /// Last page acknowledged.
    Done,
}

/// What `press` did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsgEvent {
    None,
    /// Moved to the next page (clear the text, restart the reveal).
    NextPage,
    /// The last page was acknowledged: close the box.
    Finished,
}

/// Layout options.
#[derive(Debug, Clone, Copy)]
pub struct MsgOpts {
    /// Text width in pixels (wrap limit).
    pub width: i32,
    /// Lines per page (2 for the standard dialogue box).
    pub lines: usize,
    /// Extra pixels between characters (`spacing` of `line_width`).
    pub spacing: i32,
    /// Text speed value (`speed::option_speed(opt)`): `v >= 0` = a char every `v + 1` frames,
    /// `v < 0` = `-v` chars per frame.
    pub speed: i32,
}

impl Default for MsgOpts {
    fn default() -> Self {
        MsgOpts {
            width: 224,
            lines: 2,
            spacing: 0,
            speed: black_text::speed::option_speed(0),
        }
    }
}

/// Split `text` into pages of cells. Empty pages are dropped.
pub fn build_pages(text: &[u16], o: &MsgOpts, g: &dyn GlyphWidth) -> Vec<Vec<Cell>> {
    let wrapped = wrap(text, o.width, o.spacing, g);
    paginate(&wrapped, o.lines)
        .into_iter()
        .map(|units| {
            parse(&units)
                .into_iter()
                .filter_map(|t| match t {
                    Token::Char(c) => Some(Cell::Char(c)),
                    Token::NewLine => Some(Cell::NewLine),
                    Token::Tag { id: 0xBE00, .. } => Some(Cell::Wait),
                    _ => None,
                })
                .collect::<Vec<_>>()
        })
        .filter(|p| !p.is_empty())
        .collect()
}

/// The message box logic.
#[derive(Debug, Clone)]
pub struct MessageBox {
    pages: Vec<Vec<Cell>>,
    page: usize,
    shown: usize,
    pacer: Pacer,
    state: MsgState,
}

impl MessageBox {
    pub fn new(text: &[u16], o: &MsgOpts, g: &dyn GlyphWidth) -> MessageBox {
        let pages = build_pages(text, o, g);
        let state = if pages.is_empty() {
            MsgState::Done
        } else {
            MsgState::Typing
        };
        MessageBox {
            pages,
            page: 0,
            shown: 0,
            pacer: Pacer::new(pace(o.speed)),
            state,
        }
    }

    pub fn state(&self) -> MsgState {
        self.state
    }
    pub fn page(&self) -> usize {
        self.page
    }
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }
    /// The revealed cells of the current page.
    pub fn visible(&self) -> &[Cell] {
        self.pages
            .get(self.page)
            .map_or(&[][..], |p| &p[..self.shown.min(p.len())])
    }
    /// The ▼ advance arrow is showing.
    pub fn arrow(&self) -> bool {
        matches!(self.state, MsgState::Waiting { .. })
    }

    /// Reveal up to `budget` characters; stops at `Wait` and at the end of the page.
    fn reveal(&mut self, mut budget: u32) {
        let Some(p) = self.pages.get(self.page) else {
            return;
        };
        while self.shown < p.len() {
            match p[self.shown] {
                Cell::Wait => {
                    self.state = MsgState::Waiting { mid: true };
                    return;
                }
                Cell::NewLine => self.shown += 1,
                Cell::Char(_) => {
                    if budget == 0 {
                        return;
                    }
                    budget -= 1;
                    self.shown += 1;
                }
            }
        }
        self.state = MsgState::Waiting { mid: false };
    }

    /// One 60 Hz frame of the typewriter.
    pub fn tick(&mut self) {
        if self.state == MsgState::Typing {
            let n = self.pacer.tick();
            if n > 0 {
                self.reveal(n as u32);
            }
        }
    }

    /// A or B pressed: while typing, show the rest of the page at once; at a wait / page end,
    /// continue.
    pub fn press(&mut self) -> MsgEvent {
        match self.state {
            MsgState::Typing => {
                self.reveal(u32::MAX);
                MsgEvent::None
            }
            MsgState::Waiting { mid: true } => {
                self.shown += 1; // past the Wait cell
                self.state = MsgState::Typing;
                MsgEvent::None
            }
            MsgState::Waiting { mid: false } => {
                if self.page + 1 < self.pages.len() {
                    self.page += 1;
                    self.shown = 0;
                    self.state = MsgState::Typing;
                    MsgEvent::NextPage
                } else {
                    self.state = MsgState::Done;
                    MsgEvent::Finished
                }
            }
            MsgState::Done => MsgEvent::None,
        }
    }
}

/// The visible cells as a unit string for `FontMetrics::place` (`Wait` draws nothing).
pub fn cells_to_units(cells: &[Cell]) -> Vec<u16> {
    cells
        .iter()
        .filter_map(|c| match c {
            Cell::Char(u) => Some(*u),
            Cell::NewLine => Some(black_text::token::NEWLINE),
            Cell::Wait => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::font::{encode, FixedWidth};

    fn opts(speed: i32) -> MsgOpts {
        MsgOpts {
            width: 100,
            lines: 2,
            spacing: 0,
            speed,
        }
    }
    fn text(c: &[Cell]) -> String {
        c.iter()
            .map(|c| match c {
                Cell::Char(u) => char::from_u32(*u as u32).unwrap(),
                Cell::NewLine => '/',
                Cell::Wait => '~',
            })
            .collect()
    }

    #[test]
    fn pages_split_by_lines_and_clear() {
        let g = FixedWidth(5);
        // 4 lines -> two pages of 2; \x0c forces a new page
        let b = MessageBox::new(&encode("a\nb\nc\nd\x0ce"), &opts(0), &g);
        assert_eq!(b.page_count(), 3);
        let pages = build_pages(&encode("a\nb\nc\nd\x0ce"), &opts(0), &g);
        assert_eq!(text(&pages[0]), "a/b");
        assert_eq!(text(&pages[1]), "c/d");
        assert_eq!(text(&pages[2]), "e");
    }

    #[test]
    fn wraps_at_width_using_glyph_widths() {
        let g = FixedWidth(10); // 100 px = 10 chars
        let pages = build_pages(&encode("aaaa bbbb cccc"), &opts(0), &g);
        assert_eq!(text(&pages[0]), "aaaa bbbb/cccc");
    }

    #[test]
    fn typewriter_pace_one_char_per_frame_at_speed_zero() {
        let g = FixedWidth(5);
        // speed 0 = a char every 1 frame (delay 0, 1 per tick)
        let mut b = MessageBox::new(&encode("abc"), &opts(0), &g);
        assert_eq!(b.visible().len(), 0);
        b.tick();
        assert_eq!(text(b.visible()), "a");
        b.tick();
        b.tick();
        assert_eq!(text(b.visible()), "abc");
        assert_eq!(b.state(), MsgState::Waiting { mid: false });
        assert!(b.arrow());
    }

    #[test]
    fn slow_speed_waits_between_chars() {
        let g = FixedWidth(5);
        let mut b = MessageBox::new(&encode("abc"), &opts(1), &g); // every 2 frames
        b.tick();
        assert_eq!(text(b.visible()), "a");
        b.tick();
        assert_eq!(text(b.visible()), "a");
        b.tick();
        assert_eq!(text(b.visible()), "ab");
    }

    #[test]
    fn fast_speed_reveals_several_per_frame() {
        let g = FixedWidth(5);
        let mut b = MessageBox::new(&encode("abcdef"), &opts(-4), &g);
        b.tick();
        assert_eq!(text(b.visible()), "abcd");
    }

    #[test]
    fn newlines_are_free() {
        let g = FixedWidth(5);
        let mut b = MessageBox::new(&encode("a\nb"), &opts(0), &g);
        b.tick();
        b.tick();
        assert_eq!(text(b.visible()), "a/b");
    }

    #[test]
    fn press_while_typing_completes_page_then_advances() {
        let g = FixedWidth(5);
        let mut b = MessageBox::new(&encode("a\nb\nc\nd"), &opts(1), &g);
        assert_eq!(b.press(), MsgEvent::None);
        assert_eq!(text(b.visible()), "a/b");
        assert!(b.arrow());
        assert_eq!(b.press(), MsgEvent::NextPage);
        assert_eq!(b.page(), 1);
        assert_eq!(b.visible().len(), 0);
        assert_eq!(b.state(), MsgState::Typing);
        b.press();
        assert_eq!(b.press(), MsgEvent::Finished);
        assert_eq!(b.state(), MsgState::Done);
        assert_eq!(b.press(), MsgEvent::None);
    }

    #[test]
    fn wait_tag_pauses_mid_page() {
        let g = FixedWidth(5);
        let mut b = MessageBox::new(&encode("ab\x07cd"), &opts(-8), &g);
        b.tick();
        assert_eq!(text(b.visible()), "ab");
        assert_eq!(b.state(), MsgState::Waiting { mid: true });
        b.tick(); // no progress while waiting
        assert_eq!(text(b.visible()), "ab");
        // first press from the mid-wait resumes
        assert_eq!(b.press(), MsgEvent::None);
        assert_eq!(b.state(), MsgState::Typing);
        b.tick();
        assert_eq!(text(b.visible()), "ab~cd");
        assert_eq!(cells_to_units(b.visible()), encode("abcd"));
        assert_eq!(b.press(), MsgEvent::Finished);
    }

    #[test]
    fn press_skip_stops_at_wait() {
        let g = FixedWidth(5);
        let mut b = MessageBox::new(&encode("ab\x07cd"), &opts(1), &g);
        b.press();
        assert_eq!(b.state(), MsgState::Waiting { mid: true });
        assert_eq!(text(b.visible()), "ab");
    }

    #[test]
    fn empty_text_is_done() {
        let g = FixedWidth(5);
        let b = MessageBox::new(&[], &opts(0), &g);
        assert_eq!(b.state(), MsgState::Done);
        assert_eq!(b.page_count(), 0);
        assert!(b.visible().is_empty());
    }

    #[test]
    fn unknown_tags_are_dropped() {
        let g = FixedWidth(5);
        let mut units = encode("a");
        units.extend_from_slice(&[0xF000, 0xFF00, 1, 7]);
        units.extend(encode("b"));
        let pages = build_pages(&units, &opts(0), &g);
        assert_eq!(text(&pages[0]), "ab");
    }
}
