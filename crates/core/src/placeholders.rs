use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

use crate::domains;

/// Sample values that ready-made packs ship with.
pub const SAMPLES: &[&str] = &[
    "magaza.sunucuadiniz.com",
    "store.sunucuadiniz.com",
    "play.sunucuadiniz.com",
    "www.sunucuadiniz.com",
    "sunucuadiniz.com",
    "sunucuadi.com",
    "magaza.ornek.net",
    "magaza.ornek.com",
    "www.ornek.net",
    "www.ornek.com",
    "ornek.net",
    "ornek.com",
    "discord.gg/sunucuadiniz",
    "discord.gg/ornek",
    "discord.gg/example",
    "discord.gg/xxxx",
    "discord.gg/yourserver",
    "play.example.com",
    "store.example.com",
    "play.yourserver.com",
    "yourserver.com",
    "SunucuAdınız",
    "Sunucu Adınız",
    "SunucuAdı",
    "Sunucu Adı",
    "YourServerName",
    "YourServer",
    "ServerName",
    "Server Name",
];

// first labels that mark an address as the server's own: www.x, magaza.x, play.x...
const BRAND_PREFIXES: &[&str] = &[
    "www", "magaza", "market", "store", "shop", "play", "mc", "oyna", "discord", "dc", "site",
    "web", "forum", "vote", "oy", "destek", "support", "panel", "giris",
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Hit {
    pub value: String,
    pub count: usize,
    pub files: Vec<String>,
    /// found by looking at the pack's own addresses rather than the sample list
    pub detected: bool,
}

fn value_lines(text: &str) -> impl Iterator<Item = &str> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.starts_with('#') && !l.starts_with("//"))
}

/// The pack's own domains, e.g. "montana.com" from "magaza.montana.com".
pub fn brand_domains<'a>(texts: impl Iterator<Item = &'a str>) -> BTreeSet<String> {
    let mut bases = BTreeSet::new();
    for text in texts {
        for line in value_lines(text) {
            for host in domains::bare(line) {
                let lower = host.to_ascii_lowercase();
                let first = lower.split('.').next().unwrap_or("");
                let base = domains::base(&lower).to_string();
                let sample = SAMPLES.iter().any(|s| s.eq_ignore_ascii_case(&lower));
                if BRAND_PREFIXES.contains(&first)
                    && lower != base
                    && !domains::is_known(&lower, &[])
                    && !sample
                {
                    bases.insert(base);
                }
            }
        }
    }
    bases
}

/// Every address on one of the brand domains, as written in the files.
pub fn brand_values<'a>(
    texts: impl Iterator<Item = &'a str>,
    bases: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for text in texts {
        for line in value_lines(text) {
            for host in domains::bare(line) {
                if bases.contains(domains::base(&host.to_ascii_lowercase())) {
                    out.insert(host);
                }
            }
        }
    }
    out
}

/// Counts every value once, longest first, masking matches so "www.x.com" isn't
/// also counted as "x.com".
pub fn count_in<'a>(text: &str, values: &'a [String]) -> Vec<(&'a str, usize)> {
    let mut work = text.to_string();
    let mut found = Vec::new();
    for v in values {
        let n = work.matches(v.as_str()).count();
        if n > 0 {
            found.push((v.as_str(), n));
            work = work.replace(v.as_str(), &"\u{1}".repeat(v.chars().count()));
        }
    }
    found
}

/// Sample list plus the detected values, longest first.
pub fn search_list(detected: &BTreeSet<String>) -> Vec<String> {
    let mut all: Vec<String> = SAMPLES.iter().map(|s| s.to_string()).collect();
    for d in detected {
        if !all.contains(d) {
            all.push(d.clone());
        }
    }
    all.sort_by_key(|v| std::cmp::Reverse(v.chars().count()));
    all
}

pub fn collect(files: &[(String, String)]) -> Vec<Hit> {
    let bases = brand_domains(files.iter().map(|(_, t)| t.as_str()));
    let detected = brand_values(files.iter().map(|(_, t)| t.as_str()), &bases);
    let values = search_list(&detected);

    let mut hits: BTreeMap<&str, (usize, Vec<String>)> = BTreeMap::new();
    for (rel, text) in files {
        for (v, n) in count_in(text, &values) {
            let e = hits.entry(v).or_default();
            e.0 += n;
            e.1.push(rel.clone());
        }
    }
    let mut out: Vec<Hit> = hits
        .into_iter()
        .map(|(value, (count, files))| Hit {
            detected: detected.contains(value),
            value: value.to_string(),
            count,
            files,
        })
        .collect();
    out.sort_by_key(|h| std::cmp::Reverse(h.count));
    out
}

/// Applies the user's replacements longest-first, same order as counting.
pub fn replace(text: &str, map: &BTreeMap<String, String>) -> String {
    let mut keys: Vec<&String> = map
        .iter()
        .filter(|(_, v)| !v.is_empty())
        .map(|(k, _)| k)
        .collect();
    keys.sort_by_key(|k| std::cmp::Reverse(k.chars().count()));
    let mut out = text.to_string();
    for k in keys {
        out = out.replace(k.as_str(), &map[k]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn longer_value_is_not_double_counted() {
        let values = s(&["magaza.sunucuadiniz.com", "sunucuadiniz.com"]);
        let found = count_in(
            "ip: sunucuadiniz.com\nstore: magaza.sunucuadiniz.com",
            &values,
        );
        assert_eq!(
            found,
            vec![("magaza.sunucuadiniz.com", 1), ("sunucuadiniz.com", 1)]
        );
    }

    #[test]
    fn finds_the_packs_own_addresses() {
        let files = vec![
            ("config.yml".to_string(), "# docs: https://docs.zithium.net\n- '&7ᴍᴀɢᴀᴢᴀ &#FFBE6Cmagaza.montana.com'\n- 'www.montana.com'\nurl: \"https://schem.intellectualsites.com/fawe/\"\n".to_string()),
            ("motd.yml".to_string(), "line: montana.com | discord.montana.com\n".to_string()),
        ];
        let hits = collect(&files);
        let values: Vec<&str> = hits.iter().map(|h| h.value.as_str()).collect();
        assert!(values.contains(&"magaza.montana.com"));
        assert!(values.contains(&"discord.montana.com"));
        assert!(values.contains(&"montana.com"));
        assert!(
            !values
                .iter()
                .any(|v| v.contains("intellectualsites") || v.contains("zithium"))
        );
    }

    #[test]
    fn keeps_color_codes() {
        let mut map = BTreeMap::new();
        map.insert("Sunucu Adı".to_string(), "Gökyüzü".to_string());
        let out = replace("prefix: '&6Sunucu Adı &7» '", &map);
        assert_eq!(out, "prefix: '&6Gökyüzü &7» '");
    }
}
