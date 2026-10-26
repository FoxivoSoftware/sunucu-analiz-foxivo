use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Plugin {
    pub file: String,
    pub name: String,
    pub version: Option<String>,
    pub api_version: Option<String>,
    pub authors: Vec<String>,
    pub depend: Vec<String>,
    pub website: Option<String>,
    /// byte-identical to an official release, see verify.rs
    pub official: Option<crate::verify::Official>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
struct PluginYml {
    name: Option<String>,
    version: Option<serde_yaml::Value>,
    api_version: Option<serde_yaml::Value>,
    author: Option<String>,
    #[serde(default)]
    authors: Vec<String>,
    #[serde(default)]
    depend: Vec<String>,
    website: Option<String>,
}

fn value_str(v: Option<serde_yaml::Value>) -> Option<String> {
    match v? {
        serde_yaml::Value::String(s) => Some(s),
        serde_yaml::Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

pub fn jar_paths(root: &Path) -> Vec<PathBuf> {
    let mut jars: Vec<PathBuf> = fs::read_dir(root.join("plugins"))
        .map(|it| {
            it.filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("jar")))
                .collect()
        })
        .unwrap_or_default();
    jars.sort();
    jars
}

/// Reads plugin.yml (or paper-plugin.yml) out of the jar. The jar is never executed.
pub fn read_plugin(jar: &Path) -> Plugin {
    let file = jar
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let fallback_name = file.trim_end_matches(".jar").to_string();

    let meta = File::open(jar)
        .ok()
        .and_then(|f| zip::ZipArchive::new(f).ok())
        .and_then(|mut zip| {
            ["plugin.yml", "paper-plugin.yml", "bungee.yml"]
                .iter()
                .find_map(|name| {
                    let mut entry = zip.by_name(name).ok()?;
                    let mut text = String::new();
                    entry.read_to_string(&mut text).ok()?;
                    serde_yaml::from_str::<PluginYml>(&text).ok()
                })
        })
        .unwrap_or_default();

    let mut authors = meta.authors;
    if let Some(a) = meta.author {
        authors.insert(0, a);
    }

    Plugin {
        file,
        name: meta.name.unwrap_or(fallback_name),
        version: value_str(meta.version),
        api_version: value_str(meta.api_version),
        authors,
        depend: meta.depend,
        website: meta.website,
        official: None,
    }
}
