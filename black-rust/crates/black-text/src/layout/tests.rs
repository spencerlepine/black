use super::*;
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};
use std::cell::RefCell;

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0201D26C, "b04ed26f00dd0d4f"),
    (0x0201D304, "73c86fc52868c21a"),
];
// fndiff-recorded-end

/// Address of pointer argument 0 in the harness.
const ARG_BASE: u32 = 0x02E1_0000;

/// `FUN_0201DC6C(font, code)` routed to the spec's stub line.
struct StubGlyph<'a> {
    env: RefCell<&'a mut StubEnv>,
    font: u32,
}

impl GlyphWidth for StubGlyph<'_> {
    fn width(&self, c: u16) -> u8 {
        self.env
            .borrow_mut()
            .call(0x0201_DC6C, &mut [Arg::V(self.font), Arg::V(c as u32)]) as u8
    }
}

fn units(b: &[u8]) -> Vec<u16> {
    (0..b.len() / 2)
        .map(|i| u16::from_le_bytes([b[2 * i], b[2 * i + 1]]))
        .collect()
}

fn call(addr: u32, c: &mut Case) -> u64 {
    match addr {
        0x0201_D26C => {
            let s = units(&c.bufs[0]);
            let (font, spacing) = (c.vals[1], c.vals[2] as i32);
            let g = StubGlyph {
                env: RefCell::new(&mut c.env),
                font,
            };
            let (w, next) = line_width(&s, 0, spacing, &g);
            c.bufs[3].copy_from_slice(&(ARG_BASE + 2 * next as u32).to_le_bytes());
            w as u32 as u64
        }
        0x0201_D304 => {
            let s = units(&c.bufs[0][crate::strbuf::CHARS_OFS..]);
            let (font, spacing) = (c.vals[1], c.vals[2] as i32);
            let g = StubGlyph {
                env: RefCell::new(&mut c.env),
                font,
            };
            max_line_width(&s, spacing, &g) as u64
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-text layout`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "layout",
            include_str!("../../../../tools/rom-diff/specs/black-text/layout.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

/// 'a'..'z' are 6 wide, space 3, everything else 8.
struct Fixed;
impl GlyphWidth for Fixed {
    fn width(&self, c: u16) -> u8 {
        match c {
            0x61..=0x7A => 6,
            0x20 => 3,
            _ => 8,
        }
    }
}

fn u(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

#[test]
fn line_width_stops_at_breaks_and_skips_tags() {
    let mut s = u("ab");
    s.extend([TAG, 0x0100, 2, 7, 7]);
    s.extend(u("c"));
    s.push(NEWLINE);
    s.extend(u("abcd"));
    s.push(END);
    assert_eq!(line_width(&s, 0, 1, &Fixed), (21, 9));
    assert_eq!(line_width(&s, 9, 0, &Fixed), (24, 13));
    assert_eq!(max_line_width(&s, 0, &Fixed), 24);
    assert_eq!(max_line_width(&[END], 0, &Fixed), 0);
    assert_eq!(line_count(&s), 2);
}

#[test]
fn wrap_breaks_at_spaces() {
    let s = u("aa bb cc dd");
    let w = wrap(&s, 30, 0, &Fixed);
    let lines: Vec<String> = w
        .split(|&c| c == NEWLINE)
        .map(|l| String::from_utf16(l).unwrap())
        .collect();
    assert_eq!(lines, ["aa bb", "cc dd"]);
    for l in w.split(|&c| c == NEWLINE) {
        assert!(line_width(l, 0, 0, &Fixed).0 <= 30);
    }
    // no space: break before the overflowing character
    let w = wrap(&u("aaaaaaa"), 20, 0, &Fixed);
    assert_eq!(
        w,
        [u("aaa"), vec![NEWLINE], u("aaa"), vec![NEWLINE], u("a")].concat()
    );
}

#[test]
fn paginate_splits_on_clear_and_line_count() {
    let mut s = u("a");
    s.push(NEWLINE);
    s.extend(u("b"));
    s.push(NEWLINE);
    s.extend(u("c"));
    s.extend([TAG, 0xBE01, 0]);
    s.extend(u("d"));
    s.push(END);
    let p = paginate(&s, 2);
    assert_eq!(
        p,
        [[u("a"), vec![NEWLINE], u("b")].concat(), u("c"), u("d")]
    );
}
