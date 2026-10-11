//! Comparison helpers (ports of `compare_arm9.py`, `compare_overlays.py`,
//! `compare_overlay_table.py`).  Each returns a report whose `Display` matches the scripts.

use crate::blz;
use crate::error::Result;
use crate::overlay::ENTRY_SIZE;
use std::fmt;

/// Load address of the decompressed ARM9, used to label diff runs.
pub const ARM9_LOAD: u32 = 0x0200_4000;

/// Runs of differing bytes as `(offset, length)`, over the common length.
pub fn diff_runs(built: &[u8], original: &[u8]) -> Vec<(usize, usize)> {
    let n = built.len().min(original.len());
    let mut runs = Vec::new();
    let mut i = 0;
    while i < n {
        if built[i] != original[i] {
            let start = i;
            while i < n && built[i] != original[i] {
                i += 1;
            }
            runs.push((start, i - start));
        } else {
            i += 1;
        }
    }
    runs
}

#[derive(Debug)]
pub struct Arm9Report {
    pub built_len: usize,
    pub original_len: usize,
    pub runs: Vec<(usize, usize)>,
}

impl Arm9Report {
    pub fn trailer(&self) -> isize {
        self.built_len as isize - self.original_len as isize
    }

    pub fn matches(&self) -> bool {
        self.runs.is_empty() && self.trailer() >= 0
    }
}

pub fn compare_arm9(built: &[u8], original: &[u8]) -> Arm9Report {
    Arm9Report {
        built_len: built.len(),
        original_len: original.len(),
        runs: diff_runs(built, original),
    }
}

impl fmt::Display for Arm9Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "built:    {} bytes", self.built_len)?;
        writeln!(f, "original: {} bytes", self.original_len)?;
        let trailer = self.trailer();
        if trailer > 0 {
            writeln!(
                f,
                "trailer:  {trailer} bytes of SDK footer appended by the linker"
            )?;
        } else if trailer < 0 {
            writeln!(f, "SHORT by {} bytes", -trailer)?;
        }
        if self.matches() {
            return write!(f, "\nMATCH");
        }
        let differing: usize = self.runs.iter().map(|r| r.1).sum();
        let coverage = if self.original_len == 0 {
            0.0
        } else {
            100.0 * (self.original_len - differing) as f64 / self.original_len as f64
        };
        writeln!(f, "\nNO MATCH")?;
        writeln!(f, "  matching:  {coverage:.4}% of the original")?;
        writeln!(f, "  diff runs: {}, {differing} bytes", self.runs.len())?;
        for (start, len) in self.runs.iter().take(20) {
            writeln!(f, "    0x{:08X}  +{len}", ARM9_LOAD as usize + start)?;
        }
        if self.runs.len() > 20 {
            write!(f, "    ... {} more", self.runs.len() - 20)?;
        }
        Ok(())
    }
}

/// One entry of the overlay comparison.
#[derive(Debug)]
pub struct OverlayFailure {
    pub index: usize,
    pub ram: u32,
    pub reason: String,
}

#[derive(Debug)]
pub struct OverlayReport {
    pub total: usize,
    pub matched: usize,
    pub failures: Vec<OverlayFailure>,
    pub show: usize,
}

impl OverlayReport {
    pub fn matches(&self) -> bool {
        self.failures.is_empty()
    }
}

fn verdict(built: Option<&[u8]>, want: &[u8], size: u32) -> Option<String> {
    let built = match built {
        None => return Some("not built".into()),
        Some(b) => b,
    };
    if built == want && built.len() == size as usize {
        return None;
    }
    if built.len() < want.len() {
        if built.iter().all(|&b| b == 0) {
            return Some(format!(
                "dropped by the linker, {} bytes of padding only",
                built.len()
            ));
        }
        return Some(format!("short by {} bytes", want.len() - built.len()));
    }
    if built.len() > want.len() {
        if &built[..want.len()] == want {
            return Some(format!(
                "{} trailing bytes past the original",
                built.len() - want.len()
            ));
        }
        return Some(format!(
            "long by {} bytes and content differs",
            built.len() - want.len()
        ));
    }
    let differing = built.iter().zip(want).filter(|(a, b)| a != b).count();
    Some(format!("{differing} content bytes differ"))
}

/// Compare built overlays with the originals.  `original(i)` yields the stored original
/// (`None` if not extracted), `built(i)` the linker output.
pub fn compare_overlays(
    table: &[u8],
    show: usize,
    mut original: impl FnMut(usize) -> Option<Vec<u8>>,
    mut built: impl FnMut(usize) -> Option<Vec<u8>>,
) -> Result<OverlayReport> {
    let overlays = crate::overlay::parse_table(table)?;
    let mut report = OverlayReport {
        total: 0,
        matched: 0,
        failures: Vec::new(),
        show,
    };
    for (index, ov) in overlays.iter().enumerate() {
        if ov.id as usize != index {
            return Err(crate::error::Error::invalid(
                "overlay table",
                format!("entry {index} reports id {}", ov.id),
            ));
        }
        report.total += 1;
        let Some(stored) = original(index) else {
            report.failures.push(OverlayFailure {
                index,
                ram: ov.ram_address,
                reason: "not extracted".into(),
            });
            continue;
        };
        let want = if ov.compressed_size != 0 {
            blz::decompress(&stored)?
        } else {
            stored
        };
        let b = built(index);
        match verdict(b.as_deref(), &want, ov.ram_size) {
            None => report.matched += 1,
            Some(reason) => report.failures.push(OverlayFailure {
                index,
                ram: ov.ram_address,
                reason,
            }),
        }
    }
    Ok(report)
}

impl fmt::Display for OverlayReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "overlays: {}", self.total)?;
        writeln!(f, "matching: {}", self.matched)?;
        if self.failures.is_empty() {
            return write!(f, "\nMATCH");
        }
        writeln!(f, "\nNO MATCH")?;
        writeln!(
            f,
            "  {:.2}% of overlays, {} failing",
            100.0 * self.matched as f64 / self.total as f64,
            self.failures.len()
        )?;
        for fail in self.failures.iter().take(self.show) {
            writeln!(
                f,
                "    OVY_{} @ 0x{:08X}  {}",
                fail.index, fail.ram, fail.reason
            )?;
        }
        if self.failures.len() > self.show {
            write!(f, "    ... {} more", self.failures.len() - self.show)?;
        }
        Ok(())
    }
}

const FIELDS: [&str; 8] = [
    "id",
    "ram_address",
    "ram_size",
    "bss_size",
    "sinit_start",
    "sinit_end",
    "file_id",
    "compressed",
];

#[derive(Debug)]
pub struct TableReport {
    pub built_entries: usize,
    pub original_entries: usize,
    /// Per field: `(entry, built, original)` mismatches.
    pub wrong: Vec<Vec<(usize, u32, u32)>>,
    pub show: usize,
}

impl TableReport {
    pub fn matches(&self) -> bool {
        self.built_entries == self.original_entries && self.wrong.iter().all(Vec::is_empty)
    }
}

fn words(data: &[u8], entry: usize) -> [u32; 8] {
    let mut w = [0u32; 8];
    for (i, v) in w.iter_mut().enumerate() {
        let o = entry * ENTRY_SIZE + i * 4;
        *v = u32::from_le_bytes([data[o], data[o + 1], data[o + 2], data[o + 3]]);
    }
    w
}

pub fn compare_table(built: &[u8], original: &[u8], show: usize) -> TableReport {
    let mut wrong = vec![Vec::new(); 8];
    if built.len() == original.len() {
        for entry in 0..original.len() / ENTRY_SIZE {
            let (a, b) = (words(built, entry), words(original, entry));
            for k in 0..8 {
                if a[k] != b[k] {
                    wrong[k].push((entry, a[k], b[k]));
                }
            }
        }
    }
    TableReport {
        built_entries: built.len() / ENTRY_SIZE,
        original_entries: original.len() / ENTRY_SIZE,
        wrong,
        show,
    }
}

impl fmt::Display for TableReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "built:    {} bytes, {} entries",
            self.built_entries * ENTRY_SIZE,
            self.built_entries
        )?;
        writeln!(
            f,
            "original: {} bytes, {} entries",
            self.original_entries * ENTRY_SIZE,
            self.original_entries
        )?;
        if self.built_entries != self.original_entries {
            let diff = (self.built_entries as isize - self.original_entries as isize)
                * ENTRY_SIZE as isize;
            return write!(f, "\nNO MATCH\n  size differs by {diff} bytes");
        }
        if self.matches() {
            return write!(f, "\nMATCH");
        }
        write!(f, "\nNO MATCH")?;
        for (name, bad) in FIELDS.iter().zip(&self.wrong) {
            if bad.is_empty() {
                write!(f, "\n  {name:<12} all {} match", self.original_entries)?;
                continue;
            }
            write!(
                f,
                "\n  {name:<12} {} of {} differ",
                bad.len(),
                self.original_entries
            )?;
            for (i, x, y) in bad.iter().take(self.show) {
                write!(f, "\n      OVY_{i:<4} built 0x{x:08X}  rom 0x{y:08X}")?;
            }
            if bad.len() > self.show {
                write!(f, "\n      ... {} more", bad.len() - self.show)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs() {
        assert_eq!(
            diff_runs(&[1, 2, 3, 4, 5], &[1, 9, 9, 4, 0]),
            vec![(1, 2), (4, 1)]
        );
        let r = compare_arm9(&[1, 2, 3], &[1, 2]);
        assert!(r.matches());
        assert!(r.to_string().contains("trailer:  1 bytes"));
    }

    #[test]
    fn table_cmp() {
        let a = vec![0u8; 64];
        let mut b = a.clone();
        b[32 + 8] = 1;
        let r = compare_table(&a, &b, 6);
        assert!(!r.matches());
        assert_eq!(r.wrong[2], vec![(1, 0, 1)]);
        assert!(compare_table(&a, &a, 6).matches());
    }
}
