//! File name table (NitroFS directory tree) and file allocation table.

use crate::error::{le16, le32, slice, Error, Result};
use std::collections::BTreeMap;

/// Id of the root directory.
pub const ROOT_DIR_ID: u16 = 0xF000;

/// One entry in a directory's sub-table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FntEntry {
    File(String),
    Dir(String, u16),
}

/// One directory of the tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FntDir {
    /// File id of the first file in this directory; the following files are numbered
    /// consecutively.
    pub first_file_id: u16,
    pub entries: Vec<FntEntry>,
}

/// A parsed file name table.  `dirs[n]` is the directory with id `0xF000 + n`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Fnt {
    pub dirs: Vec<FntDir>,
}

impl Fnt {
    /// Parse a binary FNT.
    pub fn parse(data: &[u8]) -> Result<Fnt> {
        let count = le16(data, 6, "FNT root")? as usize;
        if count == 0 || count > 0x1000 {
            return Err(Error::invalid(
                "FNT",
                format!("bad directory count {count}"),
            ));
        }
        let mut dirs = Vec::with_capacity(count);
        for n in 0..count {
            let mut off = le32(data, n * 8, "FNT directory table")? as usize;
            let first_file_id = le16(data, n * 8 + 4, "FNT directory table")?;
            let mut entries = Vec::new();
            loop {
                let kind = *data.get(off).ok_or(Error::Truncated {
                    what: "FNT sub-table",
                    offset: off,
                    need: 1,
                    have: 0,
                })?;
                off += 1;
                if kind == 0 {
                    break;
                }
                let len = (kind & 0x7F) as usize;
                if len == 0 {
                    return Err(Error::invalid("FNT", "reserved entry type 0x80"));
                }
                let name = String::from_utf8(slice(data, off, len, "FNT name")?.to_vec())
                    .map_err(|_| Error::invalid("FNT", "non-UTF-8 name"))?;
                off += len;
                if kind & 0x80 != 0 {
                    let id = le16(data, off, "FNT sub-directory id")?;
                    off += 2;
                    if id < ROOT_DIR_ID || (id & 0xFFF) as usize >= count {
                        return Err(Error::invalid("FNT", format!("bad directory id {id:#x}")));
                    }
                    entries.push(FntEntry::Dir(name, id));
                } else {
                    entries.push(FntEntry::File(name));
                }
            }
            dirs.push(FntDir {
                first_file_id,
                entries,
            });
        }
        Ok(Fnt { dirs })
    }

    /// Serialise to the binary FNT layout: directory table, then sub-tables in directory order.
    pub fn build(&self) -> Result<Vec<u8>> {
        let count = self.dirs.len();
        let mut parents = vec![0u16; count];
        parents[0] = count as u16;
        for (n, dir) in self.dirs.iter().enumerate() {
            for entry in &dir.entries {
                if let FntEntry::Dir(_, id) = entry {
                    let child = (id & 0xFFF) as usize;
                    if child >= count || child == 0 {
                        return Err(Error::invalid("FNT", format!("bad directory id {id:#x}")));
                    }
                    parents[child] = ROOT_DIR_ID + n as u16;
                }
            }
        }
        let mut out = vec![0u8; count * 8];
        for (n, dir) in self.dirs.iter().enumerate() {
            let off = out.len() as u32;
            out[n * 8..n * 8 + 4].copy_from_slice(&off.to_le_bytes());
            out[n * 8 + 4..n * 8 + 6].copy_from_slice(&dir.first_file_id.to_le_bytes());
            out[n * 8 + 6..n * 8 + 8].copy_from_slice(&parents[n].to_le_bytes());
            for entry in &dir.entries {
                let (name, id) = match entry {
                    FntEntry::File(name) => (name, None),
                    FntEntry::Dir(name, id) => (name, Some(*id)),
                };
                if name.is_empty() || name.len() > 0x7F {
                    return Err(Error::invalid("FNT", format!("bad name length: {name:?}")));
                }
                out.push(name.len() as u8 | if id.is_some() { 0x80 } else { 0 });
                out.extend_from_slice(name.as_bytes());
                if let Some(id) = id {
                    out.extend_from_slice(&id.to_le_bytes());
                }
            }
            out.push(0);
        }
        Ok(out)
    }

    /// All files as `(file id, path)`, in table order (ascending by id for well-formed tables).
    pub fn files(&self) -> Vec<(u16, String)> {
        let mut out = Vec::new();
        self.walk(ROOT_DIR_ID, "", &mut out, 0);
        out
    }

    fn walk(&self, dir: u16, prefix: &str, out: &mut Vec<(u16, String)>, depth: usize) {
        let Some(d) = self.dirs.get((dir & 0xFFF) as usize) else {
            return;
        };
        if depth > 64 {
            return;
        }
        let mut id = d.first_file_id;
        for entry in &d.entries {
            match entry {
                FntEntry::File(name) => {
                    out.push((id, format!("{prefix}{name}")));
                    id = id.wrapping_add(1);
                }
                FntEntry::Dir(name, sub) => {
                    self.walk(*sub, &format!("{prefix}{name}/"), out, depth + 1)
                }
            }
        }
    }

    /// Build a table from paths given in file-id order, the first having id `first_file_id`.
    ///
    /// Matches NitroFS as laid out by the SDK tools: directory and file ids are assigned in a
    /// depth-first walk, a directory's own files come before its sub-directories, and
    /// sub-directories keep the order in which they first appear in `paths`.
    pub fn from_paths<S: AsRef<str>>(first_file_id: u16, paths: &[S]) -> Result<Fnt> {
        #[derive(Default)]
        struct Node {
            files: Vec<String>,
            dirs: Vec<(String, usize)>,
        }
        let mut nodes = vec![Node::default()];
        let mut index: BTreeMap<(usize, String), usize> = BTreeMap::new();
        for path in paths {
            let mut parts: Vec<&str> = path.as_ref().split('/').collect();
            let file = parts.pop().unwrap_or_default();
            let mut cur = 0usize;
            for part in parts {
                cur = match index.get(&(cur, part.to_string())) {
                    Some(&n) => n,
                    None => {
                        nodes.push(Node::default());
                        let n = nodes.len() - 1;
                        nodes[cur].dirs.push((part.to_string(), n));
                        index.insert((cur, part.to_string()), n);
                        n
                    }
                };
            }
            nodes[cur].files.push(file.to_string());
        }
        // depth-first: assign directory ids and file id ranges
        let mut order = Vec::new();
        let mut stack = vec![0usize];
        while let Some(n) = stack.pop() {
            order.push(n);
            for (_, child) in nodes[n].dirs.iter().rev() {
                stack.push(*child);
            }
        }
        let mut dir_id = vec![0u16; nodes.len()];
        let mut first = vec![0u16; nodes.len()];
        let mut next_file = first_file_id;
        for (i, &n) in order.iter().enumerate() {
            dir_id[n] = ROOT_DIR_ID + i as u16;
            first[n] = next_file;
            next_file = next_file.wrapping_add(nodes[n].files.len() as u16);
        }
        let mut dirs = vec![
            FntDir {
                first_file_id: 0,
                entries: Vec::new()
            };
            nodes.len()
        ];
        for &n in &order {
            let mut entries: Vec<FntEntry> =
                nodes[n].files.iter().cloned().map(FntEntry::File).collect();
            entries.extend(
                nodes[n]
                    .dirs
                    .iter()
                    .map(|(name, c)| FntEntry::Dir(name.clone(), dir_id[*c])),
            );
            dirs[(dir_id[n] & 0xFFF) as usize] = FntDir {
                first_file_id: first[n],
                entries,
            };
        }
        Ok(Fnt { dirs })
    }
}

/// A FAT entry: the byte range of a file inside the ROM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FatEntry {
    pub start: u32,
    pub end: u32,
}

impl FatEntry {
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start) as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Parse a FAT.
pub fn parse_fat(data: &[u8]) -> Result<Vec<FatEntry>> {
    if !data.len().is_multiple_of(8) {
        return Err(Error::invalid("FAT", "size is not a multiple of 8"));
    }
    (0..data.len() / 8)
        .map(|i| {
            Ok(FatEntry {
                start: le32(data, i * 8, "FAT")?,
                end: le32(data, i * 8 + 4, "FAT")?,
            })
        })
        .collect()
}

/// Serialise a FAT.
pub fn build_fat(entries: &[FatEntry]) -> Vec<u8> {
    let mut out = Vec::with_capacity(entries.len() * 8);
    for e in entries {
        out.extend_from_slice(&e.start.to_le_bytes());
        out.extend_from_slice(&e.end.to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const PATHS: [&str; 6] = ["a.bin", "b.bin", "x/0", "x/1", "x/y/0", "z/q"];

    #[test]
    fn from_paths_roundtrip() {
        let fnt = Fnt::from_paths(10, &PATHS).unwrap();
        let bytes = fnt.build().unwrap();
        let parsed = Fnt::parse(&bytes).unwrap();
        assert_eq!(parsed, fnt);
        assert_eq!(parsed.build().unwrap(), bytes);
        let files: Vec<_> = parsed.files();
        let names: Vec<&str> = files.iter().map(|f| f.1.as_str()).collect();
        assert_eq!(names, PATHS);
        assert_eq!(files[0].0, 10);
        assert_eq!(files[5].0, 15);
        // root's parent field holds the directory count
        assert_eq!(u16::from_le_bytes([bytes[6], bytes[7]]), 4);
    }

    #[test]
    fn fat_roundtrip() {
        let fat = vec![
            FatEntry {
                start: 0x200,
                end: 0x300,
            },
            FatEntry {
                start: 0x400,
                end: 0x400,
            },
        ];
        assert_eq!(parse_fat(&build_fat(&fat)).unwrap(), fat);
        assert!(fat[1].is_empty());
        assert!(parse_fat(&[0; 7]).is_err());
    }

    #[test]
    fn rejects_garbage() {
        assert!(Fnt::parse(&[0; 4]).is_err());
        assert!(Fnt::parse(&[0; 16]).is_err());
    }
}
