use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Folders we never read into or write to.
const SKIP_DIRS: &[&str] = &[
    "playerdata",
    "region",
    "logs",
    "cache",
    "libraries",
    "versions",
    "crash-reports",
    "bundler",
    ".git",
    "node_modules",
    ".paper-remapped",
    ".fabric",
];

pub const TEXT_EXTS: &[&str] = &[
    "yml",
    "yaml",
    "json",
    "properties",
    "txt",
    "sk",
    "conf",
    "toml",
    "cfg",
    "lang",
];

const NEVER_WRITE_EXTS: &[&str] = &["jar", "db", "sqlite", "sqlite3", "mca", "mcr", "dat"];

const MAX_TEXT_SIZE: u64 = 2 * 1024 * 1024;

/// `depth` 1 is a folder directly in the server root; worlds only live there,
/// so plugins/WorldEdit stays readable.
fn skip_dir(name: &str, depth: usize) -> bool {
    let lower = name.to_ascii_lowercase();
    (depth == 1 && lower.starts_with("world")) || SKIP_DIRS.contains(&lower.as_str())
}

pub fn ext(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default()
}

/// Path relative to the server root with forward slashes, used in reports and backups.
pub fn rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Config-like text files under the server root, skipping worlds, logs and binaries.
pub fn text_files(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            e.depth() == 0
                || !e.file_type().is_dir()
                || !skip_dir(&e.file_name().to_string_lossy(), e.depth())
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| TEXT_EXTS.contains(&ext(e.path()).as_str()))
        .filter(|e| {
            e.metadata()
                .map(|m| m.len() <= MAX_TEXT_SIZE)
                .unwrap_or(false)
        })
        .map(|e| e.into_path())
        .collect()
}

/// The golden rule: worlds, player data, jars and databases are never written.
pub fn is_protected(rel: &str) -> bool {
    let lower = rel.to_ascii_lowercase();
    let parts: Vec<&str> = lower.split('/').collect();
    let dir_hit = parts[..parts.len().saturating_sub(1)]
        .iter()
        .enumerate()
        .any(|(i, p)| skip_dir(p, i + 1));
    let ext_hit = lower
        .rsplit('.')
        .next()
        .is_some_and(|e| NEVER_WRITE_EXTS.contains(&e));
    dir_hit || ext_hit || parts.iter().any(|p| *p == ".." || p.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protects_worlds_and_binaries() {
        assert!(is_protected("world/level.dat"));
        assert!(is_protected("world_nether/region/r.0.0.mca"));
        assert!(is_protected("plugins/Essentials.jar"));
        assert!(is_protected("plugins/LuckPerms/luckperms-h2.db"));
        assert!(is_protected("../outside.yml"));
        assert!(!is_protected("plugins/Essentials/config.yml"));
        assert!(!is_protected("server.properties"));
        assert!(!is_protected("plugins/WorldEdit/config.yml"));
        assert!(!is_protected("plugins/WorldGuard/worlds/world/regions.yml"));
    }
}
