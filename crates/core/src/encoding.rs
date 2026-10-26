// Turkish text that went UTF-8 -> Windows-1254/1252 -> UTF-8 ends up as "Ã¼", "ÅŸ"...
// We only undo those exact sequences, so a fix can't make things worse.
use serde::Serialize;

const MOJIBAKE: &[(&str, &str)] = &[
    ("Ã¼", "ü"),
    ("Ãœ", "Ü"),
    ("Ã¶", "ö"),
    ("Ã–", "Ö"),
    ("Ã§", "ç"),
    ("Ã‡", "Ç"),
    ("Ä±", "ı"),
    ("Ä°", "İ"),
    ("ÅŸ", "ş"),
    ("Åž", "Ş"),
    ("ÄŸ", "ğ"),
    ("Äž", "Ğ"),
];

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// valid UTF-8 that contains double-encoded Turkish letters
    Mojibake,
    /// not UTF-8 at all, saved as Windows-1254
    Legacy,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    pub file: String,
    pub kind: Kind,
    pub lines: usize,
    pub before: String,
    pub after: String,
}

pub fn fix_mojibake(text: &str) -> String {
    MOJIBAKE
        .iter()
        .fold(text.to_string(), |acc, (bad, good)| acc.replace(bad, good))
}

fn has_mojibake(line: &str) -> bool {
    MOJIBAKE.iter().any(|(bad, _)| line.contains(bad))
}

fn has_turkish(text: &str) -> bool {
    text.chars().any(|c| "ğĞşŞıİçÇöÖüÜ".contains(c))
}

fn clip(s: &str) -> String {
    let s = s.trim();
    if s.chars().count() > 90 {
        format!("{}…", s.chars().take(90).collect::<String>())
    } else {
        s.to_string()
    }
}

/// Decodes the file the way we'd fix it. `None` means nothing to do.
pub fn check(rel: &str, bytes: &[u8]) -> Option<Issue> {
    match std::str::from_utf8(bytes) {
        Ok(text) => {
            let bad: Vec<&str> = text.lines().filter(|l| has_mojibake(l)).collect();
            let first = bad.first()?;
            Some(Issue {
                file: rel.to_string(),
                kind: Kind::Mojibake,
                lines: bad.len(),
                before: clip(first),
                after: clip(&fix_mojibake(first)),
            })
        }
        Err(_) => {
            // .properties files are often Latin-1 on purpose, leave them alone
            if rel.ends_with(".properties") {
                return None;
            }
            let (decoded, _, had_errors) = encoding_rs::WINDOWS_1254.decode(bytes);
            if had_errors || !has_turkish(&decoded) {
                return None;
            }
            let lines: Vec<&str> = decoded.lines().filter(|l| has_turkish(l)).collect();
            let first = lines.first()?;
            let before = String::from_utf8_lossy(
                bytes
                    .split(|b| *b == b'\n')
                    .find(|l| encoding_rs::WINDOWS_1254.decode(l).0.trim() == first.trim())
                    .unwrap_or_default(),
            );
            Some(Issue {
                file: rel.to_string(),
                kind: Kind::Legacy,
                lines: lines.len(),
                before: clip(&before),
                after: clip(first),
            })
        }
    }
}

/// New UTF-8 content for a file reported by `check`. Keeps an existing BOM.
pub fn fixed(bytes: &[u8]) -> Option<Vec<u8>> {
    match std::str::from_utf8(bytes) {
        Ok(text) => {
            let out = fix_mojibake(text);
            (out != text).then(|| out.into_bytes())
        }
        Err(_) => {
            let (decoded, _, had_errors) = encoding_rs::WINDOWS_1254.decode(bytes);
            (!had_errors).then(|| decoded.into_owned().into_bytes())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repairs_and_is_idempotent() {
        let broken = "mesaj: 'GÃ¼venli Ã§Ä±kÄ±ÅŸ yapÄ±ldÄ±'";
        let once = fix_mojibake(broken);
        assert_eq!(once, "mesaj: 'Güvenli çıkış yapıldı'");
        assert_eq!(fix_mojibake(&once), once);
    }

    #[test]
    fn leaves_clean_text() {
        assert!(check("a.yml", "isim: Şükrü Öğün".as_bytes()).is_none());
    }

    #[test]
    fn converts_windows_1254() {
        let (bytes, _, _) = encoding_rs::WINDOWS_1254.encode("başlık: Güneş");
        let issue = check("menu.yml", &bytes);
        assert!(issue.is_some_and(|i| i.kind == Kind::Legacy));
        assert_eq!(fixed(&bytes), Some("başlık: Güneş".as_bytes().to_vec()));
    }
}
