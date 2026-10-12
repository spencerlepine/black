//! Gen 5 message banks (the text files inside `a/0/0/2` and `a/0/0/3`).
//!
//! Layout (all little-endian):
//!
//! ```text
//! 0x00 u16 section count   0x02 u16 entries per section   0x04 u32 largest section size
//! 0x08 u32 reserved (0)    0x0C u32 section offset * count
//! section: u32 size, then `entries` x { u32 offset (from section start), u16 chars, u16 flags },
//!          then the strings back to back
//! ```
//!
//! Every string is `chars` UTF-16 units including the trailing `0xFFFF`, each XORed with a rolling
//! key: entry `i` starts at `0x7C89 + 0x2983 * i` and the key is rotated left by 3 after every unit.
//! The `flags` field is kept verbatim (its meaning is unknown).
//!
//! A string whose first decoded unit is `0xF100` is compressed: the units after it are a bit stream
//! (LSB first) of 9-bit codes ending with `0x1FF`, zero padded to a whole unit, followed by the
//! `0xFFFF` terminator. When the end code leaves at most 9 bits in the last unit, that unit is
//! the terminator itself (all ones).
//!
//! Inside a string `0xFFFE` is a line break and `0xF000 id count params..` is a command (variable
//! insertion, page break `0xBE01`, ...). Unknown commands are kept as raw values.

use crate::error::{le16, le32, slice, Error, Result};

const KEY_START: u16 = 0x7C89;
const KEY_STEP: u16 = 0x2983;
const END: u16 = 0xFFFF;
const NEWLINE: u16 = 0xFFFE;
const CMD: u16 = 0xF000;
const COMPRESSED: u16 = 0xF100;
const CODE_END: u32 = 0x1FF;

fn bad(detail: impl Into<String>) -> Error {
    Error::invalid("message bank", detail)
}

/// One element of a decoded string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// Plain characters.
    Text(String),
    /// `0xFFFE`.
    NewLine,
    /// `0xF000 id count params..`.
    Cmd { id: u16, params: Vec<u16> },
    /// A UTF-16 unit that is not a valid character on its own (unpaired surrogate).
    Raw(u16),
}

/// One string of a bank.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Table `flags` field, verbatim.
    pub flags: u16,
    /// Stored with the 9-bit compression.
    pub compressed: bool,
    pub tokens: Vec<Token>,
}

impl Entry {
    /// Stable string form, see [`render`].
    pub fn render(&self) -> String {
        render(&self.tokens)
    }
}

/// A parsed message bank.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bank {
    /// Header word at 0x08 (always 0 in the game).
    pub reserved: u32,
    /// `sections[s][i]` is entry `i` of section `s` (the game has one section per bank).
    pub sections: Vec<Vec<Entry>>,
    /// Bytes between the last string and the end of each section: the game pads sections to 4
    /// bytes with a repeated byte whose value is not derivable. Zeros are added when missing.
    pub tails: Vec<Vec<u8>>,
}

fn step_key(key: u16) -> u16 {
    key.rotate_left(3)
}

fn entry_key(i: usize) -> u16 {
    KEY_START.wrapping_add(KEY_STEP.wrapping_mul(i as u16))
}

fn decrypt_units(data: &[u8], entry: usize) -> Vec<u16> {
    let mut key = entry_key(entry);
    data.as_chunks::<2>()
        .0
        .iter()
        .map(|c| {
            let u = u16::from_le_bytes(*c) ^ key;
            key = step_key(key);
            u
        })
        .collect()
}

fn encrypt_units(units: &[u16], entry: usize, out: &mut Vec<u8>) {
    let mut key = entry_key(entry);
    for u in units {
        out.extend_from_slice(&(u ^ key).to_le_bytes());
        key = step_key(key);
    }
}

/// Units (without the trailing `0xFFFF`) to tokens.
pub fn tokens_from_units(units: &[u16]) -> Result<Vec<Token>> {
    let mut out: Vec<Token> = Vec::new();
    let mut text = String::new();
    let mut i = 0;
    let flush = |text: &mut String, out: &mut Vec<Token>| {
        if !text.is_empty() {
            out.push(Token::Text(std::mem::take(text)));
        }
    };
    while i < units.len() {
        let u = units[i];
        match u {
            CMD => {
                let id = *units.get(i + 1).ok_or_else(|| bad("truncated command"))?;
                let n = *units.get(i + 2).ok_or_else(|| bad("truncated command"))? as usize;
                let params = units
                    .get(i + 3..i + 3 + n)
                    .ok_or_else(|| bad("truncated command parameters"))?;
                flush(&mut text, &mut out);
                out.push(Token::Cmd {
                    id,
                    params: params.to_vec(),
                });
                i += 3 + n;
            }
            NEWLINE => {
                flush(&mut text, &mut out);
                out.push(Token::NewLine);
                i += 1;
            }
            END => return Err(bad("0xFFFF inside a string")),
            _ => {
                let rest = &units[i..units.len().min(i + 2)];
                match char::decode_utf16(rest.iter().copied()).next() {
                    Some(Ok(c)) => {
                        text.push(c);
                        i += c.len_utf16();
                    }
                    _ => {
                        flush(&mut text, &mut out);
                        out.push(Token::Raw(u));
                        i += 1;
                    }
                }
            }
        }
    }
    flush(&mut text, &mut out);
    Ok(out)
}

/// Tokens to units (without the trailing `0xFFFF`).
pub fn units_from_tokens(tokens: &[Token]) -> Result<Vec<u16>> {
    let mut out = Vec::new();
    for t in tokens {
        match t {
            Token::Text(s) => {
                for u in s.encode_utf16() {
                    if matches!(u, CMD | COMPRESSED | NEWLINE | END) {
                        return Err(bad(format!("text contains reserved unit {u:#06x}")));
                    }
                    out.push(u);
                }
            }
            Token::NewLine => out.push(NEWLINE),
            Token::Cmd { id, params } => {
                let n = u16::try_from(params.len()).map_err(|_| bad("too many parameters"))?;
                out.extend([CMD, *id, n]);
                out.extend_from_slice(params);
            }
            Token::Raw(u) => out.push(*u),
        }
    }
    Ok(out)
}

fn decompress(units: &[u16]) -> Result<Vec<u16>> {
    let mut out = Vec::new();
    let (mut acc, mut bits) = (0u32, 0u32);
    for &u in units {
        acc |= (u as u32) << bits;
        bits += 16;
        while bits >= 9 {
            let code = acc & 0x1FF;
            acc >>= 9;
            bits -= 9;
            if code == CODE_END {
                return Ok(out);
            }
            out.push(code as u16);
        }
    }
    Err(bad("compressed string without end code"))
}

fn compress(units: &[u16]) -> Result<Vec<u16>> {
    let mut out = Vec::new();
    let (mut acc, mut bits) = (0u32, 0u32);
    let mut put = |code: u32, out: &mut Vec<u16>| {
        acc |= code << bits;
        bits += 9;
        while bits >= 16 {
            out.push(acc as u16);
            acc >>= 16;
            bits -= 16;
        }
    };
    for &u in units {
        if u as u32 >= CODE_END {
            return Err(bad(format!("unit {u:#06x} cannot be compressed")));
        }
        put(u as u32, &mut out);
    }
    put(CODE_END, &mut out);
    // a partial unit holding only the tail of the end code doubles as the 0xFFFF terminator
    match bits {
        0..=9 => out.push(END),
        _ => out.extend([acc as u16, END]),
    }
    Ok(out)
}

impl Bank {
    /// Parse a bank file.
    pub fn parse(bytes: &[u8]) -> Result<Bank> {
        const W: &str = "message bank header";
        let nsec = le16(bytes, 0, W)? as usize;
        let nent = le16(bytes, 2, W)? as usize;
        let reserved = le32(bytes, 8, W)?;
        let mut sections = Vec::with_capacity(nsec);
        let mut tails = Vec::with_capacity(nsec);
        for s in 0..nsec {
            let base = le32(bytes, 12 + s * 4, W)? as usize;
            let size = le32(bytes, base, "message section")? as usize;
            let sect = slice(bytes, base, size.max(4 + nent * 8), "message section")?;
            let mut entries = Vec::with_capacity(nent);
            let mut end = 4 + nent * 8;
            for i in 0..nent {
                let off = le32(sect, 4 + i * 8, "message table")? as usize;
                let chars = le16(sect, 8 + i * 8, "message table")? as usize;
                let flags = le16(sect, 10 + i * 8, "message table")?;
                let raw = slice(sect, off, chars * 2, "message string")?;
                end = end.max(off + chars * 2);
                let mut units = decrypt_units(raw, i);
                if units.last() != Some(&END) {
                    return Err(bad(format!("entry {i} has no terminator")));
                }
                let compressed = units.first() == Some(&COMPRESSED);
                if compressed {
                    units = decompress(&units[1..])?;
                } else {
                    units.pop();
                }
                let tokens = tokens_from_units(&units)
                    .map_err(|e| bad(format!("section {s} entry {i}: {e}")))?;
                entries.push(Entry {
                    flags,
                    compressed,
                    tokens,
                });
            }
            sections.push(entries);
            tails.push(sect.get(end..size).unwrap_or_default().to_vec());
        }
        Ok(Bank {
            reserved,
            sections,
            tails,
        })
    }

    /// Serialise (inverse of [`Bank::parse`]).
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let nent = self.sections.first().map_or(0, Vec::len);
        let mut bodies = Vec::new();
        for (sec, entries) in self.sections.iter().enumerate() {
            if entries.len() != nent {
                return Err(bad("sections have different entry counts"));
            }
            let mut table = Vec::new();
            let mut strings = Vec::new();
            for (i, e) in entries.iter().enumerate() {
                let mut units = units_from_tokens(&e.tokens)?;
                if e.compressed {
                    units = compress(&units)?;
                    units.insert(0, COMPRESSED);
                } else {
                    units.push(END);
                }
                let off = 4 + nent * 8 + strings.len();
                table.extend_from_slice(&(off as u32).to_le_bytes());
                let chars = u16::try_from(units.len()).map_err(|_| bad("string too long"))?;
                table.extend_from_slice(&chars.to_le_bytes());
                table.extend_from_slice(&e.flags.to_le_bytes());
                encrypt_units(&units, i, &mut strings);
            }
            strings.extend_from_slice(self.tails.get(sec).map_or(&[][..], Vec::as_slice));
            while (4 + table.len() + strings.len()) % 4 != 0 {
                strings.push(0);
            }
            let size = 4 + table.len() + strings.len();
            let mut body = (size as u32).to_le_bytes().to_vec();
            body.extend(table);
            body.extend(strings);
            bodies.push(body);
        }
        let largest = bodies.iter().map(Vec::len).max().unwrap_or(0);
        let nsec = u16::try_from(bodies.len()).map_err(|_| bad("too many sections"))?;
        let mut out = Vec::new();
        out.extend_from_slice(&nsec.to_le_bytes());
        out.extend_from_slice(&(nent as u16).to_le_bytes());
        out.extend_from_slice(&(largest as u32).to_le_bytes());
        out.extend_from_slice(&self.reserved.to_le_bytes());
        let mut off = 12 + bodies.len() * 4;
        for b in &bodies {
            out.extend_from_slice(&(off as u32).to_le_bytes());
            off += b.len();
        }
        for b in bodies {
            out.extend(b);
        }
        Ok(out)
    }

    /// Entries of the first section (the only one in the game's banks).
    pub fn entries(&self) -> &[Entry] {
        self.sections.first().map_or(&[], Vec::as_slice)
    }

    /// JSON array of `{"i","flags","compressed","text"}` for the first section.
    pub fn to_json(&self) -> String {
        let mut s = String::from("[");
        for (i, e) in self.entries().iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            s.push_str(&format!(
                "\n  {{\"i\":{i},\"flags\":{},\"compressed\":{},\"text\":",
                e.flags, e.compressed
            ));
            json_string(&e.render(), &mut s);
            s.push('}');
        }
        s.push_str("\n]\n");
        s
    }
}

fn json_string(text: &str, out: &mut String) {
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

/// Stable text form: plain text as is, `\\`, `\[` and `\xHH` (controls) escaped, line breaks as
/// a real `\n`, commands as `[VAR 0100(0001,0002)]` (`[VAR 0100]` without parameters, ids and
/// parameters in 4-digit upper-case hex) and stray units as `[RAW D800]`.
pub fn render(tokens: &[Token]) -> String {
    let mut s = String::new();
    for t in tokens {
        match t {
            Token::Text(t) => {
                for c in t.chars() {
                    match c {
                        '\\' => s.push_str("\\\\"),
                        '[' => s.push_str("\\["),
                        c if (c as u32) < 0x20 || c as u32 == 0x7F => {
                            s.push_str(&format!("\\x{:02X}", c as u32))
                        }
                        c => s.push(c),
                    }
                }
            }
            Token::NewLine => s.push('\n'),
            Token::Cmd { id, params } => {
                s.push_str(&format!("[VAR {id:04X}"));
                if !params.is_empty() {
                    let p: Vec<String> = params.iter().map(|p| format!("{p:04X}")).collect();
                    s.push_str(&format!("({})", p.join(",")));
                }
                s.push(']');
            }
            Token::Raw(u) => s.push_str(&format!("[RAW {u:04X}]")),
        }
    }
    s
}

/// Inverse of [`render`].
pub fn parse_rendered(text: &str) -> Result<Vec<Token>> {
    let hex = |s: &str| u16::from_str_radix(s, 16).map_err(|_| bad(format!("bad hex {s:?}")));
    let mut out: Vec<Token> = Vec::new();
    let mut plain = String::new();
    let mut chars = text.chars().peekable();
    let flush = |plain: &mut String, out: &mut Vec<Token>| {
        if !plain.is_empty() {
            out.push(Token::Text(std::mem::take(plain)));
        }
    };
    while let Some(c) = chars.next() {
        match c {
            '\n' => {
                flush(&mut plain, &mut out);
                out.push(Token::NewLine);
            }
            '\\' => match chars.next() {
                Some('x') => {
                    let h: String = chars.by_ref().take(2).collect();
                    let v = hex(&h)?;
                    plain.push(char::from_u32(v as u32).ok_or_else(|| bad("bad escape"))?);
                }
                Some(e @ ('\\' | '[')) => plain.push(e),
                _ => return Err(bad("bad escape")),
            },
            '[' => {
                flush(&mut plain, &mut out);
                let mut body = String::new();
                loop {
                    match chars.next() {
                        Some(']') => break,
                        Some(c) => body.push(c),
                        None => return Err(bad("unterminated [")),
                    }
                }
                if let Some(h) = body.strip_prefix("RAW ") {
                    out.push(Token::Raw(hex(h)?));
                } else if let Some(rest) = body.strip_prefix("VAR ") {
                    let (id, params) = match rest.split_once('(') {
                        Some((id, p)) => {
                            let p = p.strip_suffix(')').ok_or_else(|| bad("missing )"))?;
                            let params: Result<Vec<u16>> = p.split(',').map(hex).collect();
                            (id, params?)
                        }
                        None => (rest, Vec::new()),
                    };
                    out.push(Token::Cmd {
                        id: hex(id)?,
                        params,
                    });
                } else {
                    return Err(bad(format!("unknown tag [{body}]")));
                }
            }
            c => plain.push(c),
        }
    }
    flush(&mut plain, &mut out);
    Ok(out)
}

#[cfg(test)]
mod tests;
