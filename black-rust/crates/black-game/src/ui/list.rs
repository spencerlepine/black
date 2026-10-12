//! Vertical list menu, yes/no box and number input (G5): pure cursor logic, no window.
//!
//! The caller decides what counts as a press: for the cursor keys pass
//! `DsButtons::just_pressed || DsButtons::repeated` (the DS repeats held directions), for A/B
//! pass `just_pressed`.

/// One frame of widget input (already repeat-filtered by the caller).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NavInput {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub a: bool,
    pub b: bool,
}

/// What a list did with the input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListEvent {
    None,
    /// The cursor moved (play the cursor SE).
    Moved,
    /// A on the item.
    Select(usize),
    /// B.
    Cancel,
}

/// Cursor + scroll window over `len` items, `visible` rows at a time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListState {
    len: usize,
    visible: usize,
    wrap: bool,
    cursor: usize,
    scroll: usize,
}

impl ListState {
    /// `visible` is clamped to `1..=len`; the cursor starts on item 0.
    pub fn new(len: usize, visible: usize, wrap: bool) -> ListState {
        ListState {
            len,
            visible: visible.clamp(1, len.max(1)),
            wrap,
            cursor: 0,
            scroll: 0,
        }
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }
    /// Index of the first visible item.
    pub fn scroll(&self) -> usize {
        self.scroll
    }
    pub fn visible(&self) -> usize {
        self.visible
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    /// Items exist above the window (draw an up arrow).
    pub fn more_above(&self) -> bool {
        self.scroll > 0
    }
    /// Items exist below the window (draw a down arrow).
    pub fn more_below(&self) -> bool {
        self.scroll + self.visible < self.len
    }
    /// Cursor row inside the window.
    pub fn cursor_row(&self) -> usize {
        self.cursor - self.scroll
    }

    /// Put the cursor on `i` (clamped) and scroll it into view.
    pub fn set_cursor(&mut self, i: usize) {
        self.cursor = i.min(self.len.saturating_sub(1));
        self.fit();
    }

    fn fit(&mut self) {
        if self.cursor < self.scroll {
            self.scroll = self.cursor;
        } else if self.cursor >= self.scroll + self.visible {
            self.scroll = self.cursor + 1 - self.visible;
        }
    }

    fn step(&mut self, down: bool) -> bool {
        if self.len < 2 {
            return false;
        }
        let old = self.cursor;
        if down {
            if self.cursor + 1 < self.len {
                self.cursor += 1;
            } else if self.wrap {
                self.cursor = 0;
            }
        } else if self.cursor > 0 {
            self.cursor -= 1;
        } else if self.wrap {
            self.cursor = self.len - 1;
        }
        self.fit();
        self.cursor != old
    }

    /// Left/Right jump one window's worth (clamped, no wrap).
    fn page(&mut self, down: bool) -> bool {
        let old = self.cursor;
        self.cursor = if down {
            (self.cursor + self.visible).min(self.len.saturating_sub(1))
        } else {
            self.cursor.saturating_sub(self.visible)
        };
        self.fit();
        self.cursor != old
    }

    /// Apply one frame of input. Up wins over Down, A over B.
    pub fn handle(&mut self, i: &NavInput) -> ListEvent {
        if i.a && self.len > 0 {
            return ListEvent::Select(self.cursor);
        }
        if i.b {
            return ListEvent::Cancel;
        }
        let moved = if i.up {
            self.step(false)
        } else if i.down {
            self.step(true)
        } else if i.left {
            self.page(false)
        } else if i.right {
            self.page(true)
        } else {
            false
        };
        if moved {
            ListEvent::Moved
        } else {
            ListEvent::None
        }
    }
}

/// A yes/no box is a two-item non-wrapping list; this maps its event: `Some(true)` = Yes,
/// `Some(false)` = No (item 1, or B), `None` = still open.
pub fn yes_no_answer(e: ListEvent) -> Option<bool> {
    match e {
        ListEvent::Select(0) => Some(true),
        ListEvent::Select(_) | ListEvent::Cancel => Some(false),
        _ => None,
    }
}

/// A new yes/no list (cursor on Yes).
pub fn yes_no_list() -> ListState {
    ListState::new(2, 2, false)
}

/// Result of a number-input frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberEvent {
    None,
    Changed,
    /// A: confirmed value.
    Done(u32),
    /// B.
    Cancel,
}

/// Fixed-width digit editor (quantity / amount prompts): Left/Right pick a digit, Up/Down change it
/// (wrapping 9 -> 0), the value is clamped to `min..=max`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberInput {
    digits: usize,
    pos: usize,
    value: u32,
    min: u32,
    max: u32,
}

impl NumberInput {
    /// `digits` wide (1..=9), starting at `start` (clamped), the cursor on the last digit.
    pub fn new(digits: usize, min: u32, max: u32, start: u32) -> NumberInput {
        let digits = digits.clamp(1, 9);
        let cap = 10u32.pow(digits as u32) - 1;
        let max = max.min(cap);
        let min = min.min(max);
        NumberInput {
            digits,
            pos: digits - 1,
            value: start.clamp(min, max),
            min,
            max,
        }
    }

    pub fn value(&self) -> u32 {
        self.value
    }
    pub fn digits(&self) -> usize {
        self.digits
    }
    /// Selected digit, 0 = leftmost.
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// The digits, most significant first (zero padded).
    pub fn digit_chars(&self) -> Vec<u8> {
        (0..self.digits)
            .map(|i| ((self.value / 10u32.pow((self.digits - 1 - i) as u32)) % 10) as u8)
            .collect()
    }

    fn bump(&mut self, up: bool) -> bool {
        let p = 10u32.pow((self.digits - 1 - self.pos) as u32);
        let d = (self.value / p) % 10;
        let nd = if up { (d + 1) % 10 } else { (d + 9) % 10 };
        let v = self.value - d * p + nd * p;
        let v = v.clamp(self.min, self.max);
        let changed = v != self.value;
        self.value = v;
        changed
    }

    /// Apply one frame of input.
    pub fn handle(&mut self, i: &NavInput) -> NumberEvent {
        if i.a {
            return NumberEvent::Done(self.value);
        }
        if i.b {
            return NumberEvent::Cancel;
        }
        if i.up || i.down {
            return if self.bump(i.up) {
                NumberEvent::Changed
            } else {
                NumberEvent::None
            };
        }
        if i.left && self.pos > 0 {
            self.pos -= 1;
            return NumberEvent::Changed;
        }
        if i.right && self.pos + 1 < self.digits {
            self.pos += 1;
            return NumberEvent::Changed;
        }
        NumberEvent::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(f: impl Fn(&mut NavInput)) -> NavInput {
        let mut i = NavInput::default();
        f(&mut i);
        i
    }
    fn up() -> NavInput {
        press(|i| i.up = true)
    }
    fn down() -> NavInput {
        press(|i| i.down = true)
    }

    #[test]
    fn cursor_wraps_both_ways() {
        let mut l = ListState::new(3, 3, true);
        assert_eq!(l.handle(&up()), ListEvent::Moved);
        assert_eq!(l.cursor(), 2);
        assert_eq!(l.handle(&down()), ListEvent::Moved);
        assert_eq!(l.cursor(), 0);
    }

    #[test]
    fn no_wrap_stops_at_ends() {
        let mut l = ListState::new(3, 3, false);
        assert_eq!(l.handle(&up()), ListEvent::None);
        l.set_cursor(2);
        assert_eq!(l.handle(&down()), ListEvent::None);
        assert_eq!(l.cursor(), 2);
    }

    #[test]
    fn scrolls_to_keep_cursor_visible() {
        let mut l = ListState::new(7, 3, false);
        assert!(!l.more_above() && l.more_below());
        for _ in 0..4 {
            l.handle(&down());
        }
        assert_eq!((l.cursor(), l.scroll(), l.cursor_row()), (4, 2, 2));
        assert!(l.more_above() && l.more_below());
        for _ in 0..2 {
            l.handle(&down());
        }
        assert_eq!((l.cursor(), l.scroll()), (6, 4));
        assert!(!l.more_below());
        for _ in 0..6 {
            l.handle(&up());
        }
        assert_eq!((l.cursor(), l.scroll()), (0, 0));
    }

    #[test]
    fn wrap_scrolls_to_ends() {
        let mut l = ListState::new(7, 3, true);
        l.handle(&up());
        assert_eq!((l.cursor(), l.scroll()), (6, 4));
        l.handle(&down());
        assert_eq!((l.cursor(), l.scroll()), (0, 0));
    }

    #[test]
    fn left_right_page() {
        let mut l = ListState::new(10, 4, true);
        l.handle(&press(|i| i.right = true));
        assert_eq!(l.cursor(), 4);
        l.handle(&press(|i| i.right = true));
        l.handle(&press(|i| i.right = true));
        assert_eq!(l.cursor(), 9);
        l.handle(&press(|i| i.left = true));
        assert_eq!(l.cursor(), 5);
    }

    #[test]
    fn select_and_cancel() {
        let mut l = ListState::new(3, 3, true);
        l.handle(&down());
        assert_eq!(l.handle(&press(|i| i.a = true)), ListEvent::Select(1));
        assert_eq!(l.handle(&press(|i| i.b = true)), ListEvent::Cancel);
        let mut empty = ListState::new(0, 3, true);
        assert_eq!(empty.handle(&press(|i| i.a = true)), ListEvent::None);
    }

    #[test]
    fn single_item_does_not_move() {
        let mut l = ListState::new(1, 5, true);
        assert_eq!(l.handle(&down()), ListEvent::None);
    }

    #[test]
    fn yes_no() {
        let mut l = yes_no_list();
        assert_eq!(yes_no_answer(l.handle(&press(|i| i.a = true))), Some(true));
        l.handle(&down());
        assert_eq!(yes_no_answer(l.handle(&press(|i| i.a = true))), Some(false));
        assert_eq!(yes_no_answer(l.handle(&press(|i| i.b = true))), Some(false));
        assert_eq!(yes_no_answer(l.handle(&up())), None);
        assert_eq!(l.cursor(), 0);
    }

    #[test]
    fn number_digits_wrap_and_clamp() {
        let mut n = NumberInput::new(3, 1, 120, 1);
        assert_eq!(n.digit_chars(), vec![0, 0, 1]);
        // ones digit: 1 -> 2 ... 9 -> 0 wraps to 120's clamp? 1..9 then wrap to 0 -> clamped up to min 1
        for _ in 0..8 {
            n.handle(&up());
        }
        assert_eq!(n.value(), 9);
        n.handle(&up());
        assert_eq!(n.value(), 1, "0 is below min and clamps to 1");
        // tens digit
        n.handle(&press(|i| i.left = true));
        for _ in 0..3 {
            n.handle(&up());
        }
        assert_eq!(n.value(), 31);
        // hundreds digit: 131 > max 120 clamps
        n.handle(&press(|i| i.left = true));
        n.handle(&up());
        assert_eq!(n.value(), 120);
        assert_eq!(n.digit_chars(), vec![1, 2, 0]);
    }

    #[test]
    fn number_cursor_bounds_and_confirm() {
        let mut n = NumberInput::new(2, 0, 99, 42);
        assert_eq!(n.handle(&press(|i| i.right = true)), NumberEvent::None);
        assert_eq!(n.handle(&press(|i| i.left = true)), NumberEvent::Changed);
        assert_eq!(n.handle(&press(|i| i.left = true)), NumberEvent::None);
        assert_eq!(n.handle(&down()), NumberEvent::Changed);
        assert_eq!(n.value(), 32);
        assert_eq!(n.handle(&press(|i| i.a = true)), NumberEvent::Done(32));
        assert_eq!(n.handle(&press(|i| i.b = true)), NumberEvent::Cancel);
    }

    #[test]
    fn number_down_wraps_below_zero_digit() {
        let mut n = NumberInput::new(2, 0, 99, 5);
        n.handle(&press(|i| i.left = true));
        n.handle(&down());
        assert_eq!(n.value(), 95, "tens 0 -> 9");
    }

    #[test]
    fn number_start_and_max_are_clamped_to_width() {
        let n = NumberInput::new(2, 0, 5000, 777);
        assert_eq!(n.value(), 99);
    }
}
