//! `assets/manifest.json` as written by `black extract-assets` (A11): `{"format": 1,
//! "converter_version": "...", "files": [{"path", "category", "size", "sha1"}, ...]}`, one file
//! entry per line. The format is fixed and tiny, so it is scanned here rather than pulling in a
//! JSON crate (none is in the lockfile). Pure code: no Bevy.

/// One file of the extracted tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub path: String,
    pub category: String,
    pub size: u64,
    pub sha1: String,
}

/// The parsed manifest.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Manifest {
    pub converter_version: String,
    pub files: Vec<Entry>,
}

impl Manifest {
    /// Parses the manifest text. Errors on an unknown `format` or a malformed entry.
    pub fn parse(text: &str) -> Result<Manifest, String> {
        let mut m = Manifest::default();
        let mut format_ok = false;
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with("\"format\"") {
                format_ok = number_field(line, "format") == Some(1);
            } else if line.starts_with("\"converter_version\"") {
                m.converter_version = string_field(line, "converter_version").unwrap_or_default();
            } else if line.starts_with('{') && line.contains("\"path\"") {
                m.files.push(Entry {
                    path: string_field(line, "path").ok_or("entry without path")?,
                    category: string_field(line, "category").unwrap_or_default(),
                    size: number_field(line, "size").ok_or("entry without size")?,
                    sha1: string_field(line, "sha1").unwrap_or_default(),
                });
            }
        }
        if !format_ok {
            return Err("manifest.json: missing or unsupported \"format\" (want 1)".into());
        }
        Ok(m)
    }

    /// True if `path` is listed.
    pub fn contains(&self, path: &str) -> bool {
        self.files.iter().any(|e| e.path == path)
    }

    /// Paths with the given directory prefix and file extension, in manifest order.
    pub fn paths_in(&self, prefix: &str, ext: &str) -> Vec<&str> {
        self.files
            .iter()
            .map(|e| e.path.as_str())
            .filter(|p| p.starts_with(prefix) && p.ends_with(ext))
            .collect()
    }
}

/// Text value of `"key": "..."` on this line (JSON escapes `\"`, `\\`, `\uXXXX` handled).
fn string_field(line: &str, key: &str) -> Option<String> {
    let pat = format!("\"{key}\":");
    let rest = line[line.find(&pat)? + pat.len()..].trim_start();
    let mut chars = rest.strip_prefix('"')?.chars();
    let mut out = String::new();
    loop {
        match chars.next()? {
            '"' => return Some(out),
            '\\' => match chars.next()? {
                'u' => {
                    let hex: String = chars.by_ref().take(4).collect();
                    out.push(char::from_u32(u32::from_str_radix(&hex, 16).ok()?)?);
                }
                'n' => out.push('\n'),
                't' => out.push('\t'),
                c => out.push(c),
            },
            c => out.push(c),
        }
    }
}

/// Unsigned integer value of `"key": N` on this line.
fn number_field(line: &str, key: &str) -> Option<u64> {
    let pat = format!("\"{key}\":");
    let rest = line[line.find(&pat)? + pat.len()..].trim_start();
    let end = rest
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(rest.len());
    rest[..end].parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "{\n  \"format\": 1,\n  \"converter_version\": \"0.1.0\",\n  \"files\": [\n    {\"path\": \"data/growth.ron\", \"category\": \"data\", \"size\": 12, \"sha1\": \"ab\"},\n    {\"path\": \"text/en/sys_000.json\", \"category\": \"text\", \"size\": 5, \"sha1\": \"cd\"},\n    {\"path\": \"a\\\"b\\\\c.png\", \"category\": \"sprites\", \"size\": 1, \"sha1\": \"ef\"}\n  ]\n}\n";

    #[test]
    fn parses_entries() {
        let m = Manifest::parse(SAMPLE).unwrap();
        assert_eq!(m.converter_version, "0.1.0");
        assert_eq!(m.files.len(), 3);
        assert_eq!(m.files[0].size, 12);
        assert_eq!(m.files[1].category, "text");
        assert_eq!(m.files[2].path, "a\"b\\c.png");
        assert!(m.contains("data/growth.ron"));
        assert_eq!(
            m.paths_in("text/en/", ".json"),
            vec!["text/en/sys_000.json"]
        );
    }

    #[test]
    fn rejects_bad_format() {
        assert!(Manifest::parse("{}").is_err());
        assert!(Manifest::parse("{\n  \"format\": 2\n}").is_err());
    }
}
