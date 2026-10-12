//! Control-code tokens of a message string (F9c). A message is a `u16` string; this module splits
//! it into [`Token`]s and joins them again. Independent of `nds_rom::msg` (D6): the format is the
//! one in `README.md` ("Control-code format"), `nds_rom::msg` is only used by the tests.
//!
//! A tag is `{0xF000, group << 8 | cmd, n, args[n]}` (`3 + n` units), `0xFFFE` is a line break,
//! `0xFFFF` ends the string. Everything else is one character per unit (the engine is not
//! surrogate-aware: a unit is a glyph code).

use crate::compress;
use crate::wordset::{COMPRESSED, TAG};

/// Line break unit.
pub const NEWLINE: u16 = 0xFFFE;
/// End-of-string unit.
pub const END: u16 = 0xFFFF;

/// One element of a message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// A character code (any unit that is not `0xF000`, `0xFFFE` or `0xFFFF`).
    Char(u16),
    /// `0xFFFE`.
    NewLine,
    /// `0xF000 id n args..`: `id = group << 8 | cmd`.
    Tag { id: u16, args: Vec<u16> },
    /// `0xFFFF`; [`parse`] stops here.
    End,
    /// A tag whose `n` arguments run past the end of the slice: the remaining units, verbatim.
    Truncated(Vec<u16>),
}

impl Token {
    /// The group (`id >> 8`) of a tag.
    pub fn group(&self) -> Option<u8> {
        match self {
            Token::Tag { id, .. } => Some((id >> 8) as u8),
            _ => None,
        }
    }

    /// The command (`id & 0xFF`) of a tag.
    pub fn cmd(&self) -> Option<u8> {
        match self {
            Token::Tag { id, .. } => Some(*id as u8),
            _ => None,
        }
    }
}

/// Splits a message into tokens. A string starting with `0xF100` is 9-bit compressed and is
/// unpacked first ([`compress::decode`], which applies the insertion table); parsing stops at the
/// first `0xFFFF` (emitted as [`Token::End`]) or at the end of the slice.
pub fn parse(units: &[u16]) -> Vec<Token> {
    let unpacked;
    let units = if units.first() == Some(&COMPRESSED) {
        unpacked = compress::decode(units);
        &unpacked[..]
    } else {
        units
    };
    let mut out = Vec::new();
    let mut i = 0;
    while i < units.len() {
        match units[i] {
            END => {
                out.push(Token::End);
                break;
            }
            NEWLINE => {
                out.push(Token::NewLine);
                i += 1;
            }
            TAG => {
                let end = units.get(i + 2).map(|&n| i + 3 + n as usize);
                match end {
                    Some(end) if end <= units.len() => {
                        out.push(Token::Tag {
                            id: units[i + 1],
                            args: units[i + 3..end].to_vec(),
                        });
                        i = end;
                    }
                    _ => {
                        out.push(Token::Truncated(units[i..].to_vec()));
                        break;
                    }
                }
            }
            c => {
                out.push(Token::Char(c));
                i += 1;
            }
        }
    }
    out
}

/// Joins tokens into units; the inverse of [`parse`] for a string that is not compressed
/// (a tag with more than `0xFFFF` arguments is cut to its length modulo 2^16, as `n` is a `u16`).
pub fn encode(tokens: &[Token]) -> Vec<u16> {
    let mut out = Vec::new();
    for t in tokens {
        match t {
            Token::Char(c) => out.push(*c),
            Token::NewLine => out.push(NEWLINE),
            Token::Tag { id, args } => {
                out.extend([TAG, *id, args.len() as u16]);
                out.extend_from_slice(args);
            }
            Token::End => out.push(END),
            Token::Truncated(u) => out.extend_from_slice(u),
        }
    }
    out
}

#[cfg(test)]
pub(crate) mod tests;
