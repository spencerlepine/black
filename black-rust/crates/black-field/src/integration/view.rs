//! The headless message windows (D3: `field_msgbg.c` is the G5 / U2 rewrite; like B16's headless battle view).
//! The script commands (the black-script `scrcmd_talk` ports) open, print into, poll and close the field's
//! windows by address; here every window is a placeholder block, every print is recorded (the STRBUF the
//! command expanded, decoded with `black_text::token`), every "finished printing?" / "closed?" answers yes and
//! the yes / no menu answers yes. The key waits stay with the scripts (they poll the pad).

use super::Env;

/// `(address, what)` of the window functions modelled here (all overlay 21, `field_msgbg.c`).
pub const WINDOWS: &[(u32, &str)] = &[
    (0x0219_04DC, "msgbg flag (msgbg, on)"),
    (0x0219_0708, "info board create"),
    (0x0219_0728, "info board delete"),
    (0x0219_0758, "info board print (board, x, y, text)"),
    (0x0219_0788, "info board clear"),
    (0x0219_09BC, "menu close"),
    (0x0219_0BE4, "yes / no open"),
    (0x0219_0C78, "yes / no poll: 0 = yes"),
    (0x0219_0FA0, "talk window create"),
    (0x0219_1008, "talk window delete"),
    (0x0219_1084, "talk window print (window, x, y, text)"),
    (0x0219_10C0, "talk window printed: 1"),
    (0x0219_1134, "talk window key cursor"),
    (0x0219_1164, "talk window clear"),
    (
        0x0219_1184,
        "talk window print at once (window, x, y, text)",
    ),
    (0x0219_11B4, "talk window BG window"),
    (
        0x0219_129C,
        "balloon create (msgbg, tail, &pos, text, a, b)",
    ),
    (0x0219_12D8, "balloon close"),
    (0x0219_12F8, "balloon closed: 1"),
    (0x0219_131C, "balloon delete"),
    (0x0219_1348, "balloon next page (window, text)"),
    (0x0219_1388, "balloon printed: 1"),
    (0x0219_14CC, "balloon key cursor"),
    (0x0219_1558, "system window create"),
    (0x0219_15B4, "system window delete"),
    (0x0219_15EC, "system window clear"),
    (0x0219_16A8, "system window print (window, x, y, text)"),
    (0x0219_16E0, "system window printed: 1"),
    (0x0219_1794, "system window key cursor"),
    (0x0219_1830, "text size (msgbg, text, &w, &h): 1 x 2"),
    (0x0219_18DC, "sized info window open"),
    (0x0219_196C, "command 59 set"),
    (0x0219_1978, "sized info window close"),
    (0x0219_19AC, "sized info window gone: 0"),
    (0x0219_1A04, "window 67 create"),
    (0x0219_1A10, "window 579 create"),
    (0x0219_1B14, "window 67 / 579 delete"),
    (0x0219_1BE8, "window 67 / 579 print (window, text): 1"),
    (0x0219_2570, "window 69 create"),
    (0x0219_2628, "window 69 delete"),
    (0x0219_269C, "window 69 print (window, x, y, text)"),
    (0x0219_26D8, "window 69 printed: 1"),
    (0x0219_270C, "text width: 0"),
    (0x0219_272C, "text height: 0"),
];

/// A message the session printed.
#[derive(Clone, Debug)]
pub struct Printed {
    pub frame: u32,
    pub window: &'static str,
    /// The STRBUF's units (up to its length).
    pub units: Vec<u16>,
}

impl Printed {
    /// The text with `black_text::token`: characters, `\n` for a line break, `{group:cmd args}` for a tag.
    pub fn text(&self) -> String {
        use black_text::token::{parse, Token};
        let mut s = String::new();
        for t in parse(&self.units) {
            match t {
                Token::Char(c) => s.push(char::from_u32(c as u32).unwrap_or('?')),
                Token::NewLine => s.push('\n'),
                Token::Tag { id, args } => {
                    s.push_str(&format!("{{{:x}:{:x}", id >> 8, id & 0xFF));
                    for a in args {
                        s.push_str(&format!(" {a:x}"));
                    }
                    s.push('}');
                }
                Token::End => break,
                Token::Truncated(v) => s.push_str(&format!("{{truncated {v:x?}}}")),
            }
        }
        s
    }
}

fn record(env: &mut Env, window: &'static str, text: u32) {
    let len = env.r16(text + 2) as usize;
    let units = (0..len)
        .map(|i| env.r16(text + 8 + 2 * i as u32) as u16)
        .collect();
    let frame = env.host.frame;
    env.host.printed.push(Printed {
        frame,
        window,
        units,
    });
}

fn placeholder(env: &mut Env) -> u32 {
    env.cv(crate::hooks::HEAP_ALLOC, &[0, 0x40, 0, 0, 0])
}

/// The model of window function `a` (`WINDOWS`) with the argument words `w`.
pub fn call(env: &mut Env, a: u32, w: &[u32]) -> u64 {
    let arg = |i: usize| w.get(i).copied().unwrap_or(0);
    (match a {
        0x0219_0708 | 0x0219_0BE4 | 0x0219_0FA0 | 0x0219_11B4 | 0x0219_1558 | 0x0219_1A04
        | 0x0219_1A10 | 0x0219_2570 => placeholder(env),
        0x0219_0758 => {
            record(env, "board", arg(3));
            0
        }
        0x0219_1084 | 0x0219_1184 => {
            record(env, "talk", arg(3));
            0
        }
        0x0219_129C => {
            record(env, "balloon", arg(3));
            placeholder(env)
        }
        0x0219_1348 => {
            record(env, "balloon", arg(1));
            0
        }
        0x0219_16A8 => {
            record(env, "system", arg(3));
            0
        }
        0x0219_1BE8 => {
            record(env, "window 67", arg(1));
            1
        }
        0x0219_269C => {
            record(env, "window 69", arg(3));
            0
        }
        0x0219_10C0 | 0x0219_12F8 | 0x0219_1388 | 0x0219_16E0 | 0x0219_26D8 => 1,
        0x0219_1830 => {
            env.w8(arg(2), 1);
            env.w8(arg(3), 2);
            0
        }
        _ => 0,
    }) as u64
}
