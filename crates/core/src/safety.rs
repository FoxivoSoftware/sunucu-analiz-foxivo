// Every write goes through here: protected-path check -> backup -> in-memory
// validation -> atomic write (temp + rename) -> read-back check. Any failure puts
// every touched file back from the backup. The manifest doubles as the journal.
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{Error, IoContext, Result};
use crate::walk;

#[derive(Debug, Clone)]
pub struct Change {
    pub rel: String,
    pub content: Vec<u8>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Status {
    Pending,
    Done,
    RolledBack,
    Restored,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Backup {
    pub id: String,
    pub root: String,
    pub label: String,
    pub created: u64,
    pub files: Vec<String>,
    pub status: Status,
    #[serde(skip)]
    pub dir: PathBuf,
}

pub fn expand_home(path: &str) -> PathBuf {
    match path.strip_prefix("~") {
        Some(rest) => dirs::home_dir()
            .unwrap_or_default()
            .join(rest.trim_start_matches(['/', '\\'])),
        None => PathBuf::from(path),
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// stable folder per server so two servers called "survival" don't mix
fn server_dir(backup_root: &Path, root: &Path) -> PathBuf {
    let name: String = crate::detect::server_name(root)
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let hash = root
        .to_string_lossy()
        .bytes()
        .fold(0xcbf29ce484222325u64, |h, b| {
            (h ^ b as u64).wrapping_mul(0x100000001b3)
        });
    backup_root.join(format!("{name}-{:08x}", hash as u32))
}

fn write_manifest(b: &Backup) -> Result<()> {
    let json = serde_json::to_vec_pretty(b)?;
    atomic_write(&b.dir.join("manifest.json"), &json)
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let tmp = path.with_file_name(format!(".{name}.analiz-tmp"));
    {
        let mut f = fs::File::create(&tmp).ctx(|| format!("{} oluşturulamadı", tmp.display()))?;
        f.write_all(bytes)
            .ctx(|| format!("{} yazılamadı", tmp.display()))?;
        f.sync_all()
            .ctx(|| format!("{} diske yazılamadı", tmp.display()))?;
    }
    fs::rename(&tmp, path).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        Error::Io {
            context: format!("{} değiştirilemedi", path.display()),
            source: e,
        }
    })
}

/// New content must still parse if the old one did. Files that were already
/// broken aren't blocked, we just don't make them worse.
fn valid(rel: &str, old: &[u8], new: &[u8]) -> bool {
    match walk::ext(Path::new(rel)).as_str() {
        "yml" | "yaml" => {
            let parses = |b: &[u8]| serde_yaml::from_slice::<serde_yaml::Value>(b).is_ok();
            !parses(old) || parses(new)
        }
        "json" => {
            let parses = |b: &[u8]| serde_json::from_slice::<serde_json::Value>(b).is_ok();
            !parses(old) || parses(new)
        }
        _ => std::str::from_utf8(new).is_ok() || std::str::from_utf8(old).is_err(),
    }
}

pub fn apply(root: &Path, backup_root: &Path, label: &str, changes: &[Change]) -> Result<Backup> {
    if crate::running::is_running(root) {
        return Err(Error::ServerRunning);
    }
    for c in changes {
        if walk::is_protected(&c.rel) {
            return Err(Error::Protected(c.rel.clone()));
        }
    }

    // 1. backup + journal
    let created = now();
    let base = server_dir(backup_root, root);
    let mut id = format!("{created}-{label}");
    let mut n = 1;
    while base.join(&id).exists() {
        n += 1;
        id = format!("{created}-{label}-{n}");
    }
    let mut backup = Backup {
        id: id.clone(),
        root: root.to_string_lossy().to_string(),
        label: label.to_string(),
        created,
        files: changes.iter().map(|c| c.rel.clone()).collect(),
        status: Status::Pending,
        dir: base.join(&id),
    };
    let mut originals = Vec::with_capacity(changes.len());
    for c in changes {
        let src = root.join(&c.rel);
        let old = fs::read(&src).ctx(|| format!("{} okunamadı", c.rel))?;
        let dst = backup.dir.join("files").join(&c.rel);
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)
                .ctx(|| format!("yedek klasörü oluşturulamadı: {}", parent.display()))?;
        }
        fs::write(&dst, &old).ctx(|| format!("{} yedeklenemedi", c.rel))?;
        originals.push(old);
    }
    write_manifest(&backup)?;

    // 2. validate everything before touching the server
    for (c, old) in changes.iter().zip(&originals) {
        if !valid(&c.rel, old, &c.content) {
            backup.status = Status::RolledBack;
            write_manifest(&backup)?;
            return Err(Error::Invalid {
                file: c.rel.clone(),
            });
        }
    }

    // 3. write, read back, roll back on any failure
    let mut written = 0;
    let mut failure = None;
    for c in changes {
        let path = root.join(&c.rel);
        let ok = atomic_write(&path, &c.content)
            .and_then(|_| fs::read(&path).ctx(|| format!("{} okunamadı", c.rel)))
            .map(|back| back == c.content);
        match ok {
            Ok(true) => written += 1,
            Ok(false) => {
                written += 1;
                failure = Some(Error::Invalid {
                    file: c.rel.clone(),
                });
                break;
            }
            Err(e) => {
                failure = Some(e);
                break;
            }
        }
    }
    if let Some(err) = failure {
        for (c, old) in changes.iter().zip(&originals).take(written) {
            atomic_write(&root.join(&c.rel), old)?;
        }
        backup.status = Status::RolledBack;
        write_manifest(&backup)?;
        return Err(err);
    }

    backup.status = Status::Done;
    write_manifest(&backup)?;
    Ok(backup)
}

pub fn list(backup_root: &Path, root: &Path) -> Vec<Backup> {
    let base = server_dir(backup_root, root);
    let mut out: Vec<Backup> = fs::read_dir(&base)
        .map(|it| {
            it.filter_map(|e| e.ok())
                .filter_map(|e| {
                    let dir = e.path();
                    let text = fs::read(dir.join("manifest.json")).ok()?;
                    let mut b: Backup = serde_json::from_slice(&text).ok()?;
                    b.dir = dir;
                    Some(b)
                })
                .filter(|b| b.status != Status::RolledBack)
                .collect()
        })
        .unwrap_or_default();
    out.sort_by(|a, b| b.created.cmp(&a.created).then(b.id.cmp(&a.id)));
    out
}

/// Puts the files from a backup back in place.
pub fn restore(backup_root: &Path, root: &Path, id: &str) -> Result<Backup> {
    if crate::running::is_running(root) {
        return Err(Error::ServerRunning);
    }
    let mut backup = list(backup_root, root)
        .into_iter()
        .find(|b| b.id == id)
        .ok_or_else(|| Error::BackupMissing(id.to_string()))?;
    for rel in &backup.files {
        if walk::is_protected(rel) {
            return Err(Error::Protected(rel.clone()));
        }
        let saved = fs::read(backup.dir.join("files").join(rel))
            .ctx(|| format!("yedekteki {rel} okunamadı"))?;
        atomic_write(&root.join(rel), &saved)?;
    }
    backup.status = Status::Restored;
    write_manifest(&backup)?;
    Ok(backup)
}

/// Deletes backups older than `days`. Returns how many were removed.
pub fn cleanup(backup_root: &Path, days: u64) -> usize {
    let limit = now().saturating_sub(days * 86_400);
    let Ok(servers) = fs::read_dir(backup_root) else {
        return 0;
    };
    let mut removed = 0;
    for server in servers.filter_map(|e| e.ok()) {
        let Ok(items) = fs::read_dir(server.path()) else {
            continue;
        };
        for item in items.filter_map(|e| e.ok()) {
            let old = fs::read(item.path().join("manifest.json"))
                .ok()
                .and_then(|t| serde_json::from_slice::<Backup>(&t).ok())
                .is_some_and(|b| b.created < limit);
            if old && fs::remove_dir_all(item.path()).is_ok() {
                removed += 1;
            }
        }
    }
    removed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("analiz-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(dir.join("server/plugins/Shop"));
        dir
    }

    fn server(dir: &Path) -> PathBuf {
        let root = dir.join("server");
        let _ = fs::write(root.join("server.properties"), "server-port=1\n");
        let _ = fs::write(
            root.join("plugins/Shop/config.yml"),
            "ip: sunucuadiniz.com\n",
        );
        let _ = fs::write(root.join("plugins/Shop/data.json"), "{\"a\": 1}");
        root
    }

    #[test]
    fn apply_then_restore_is_byte_identical() -> Result<()> {
        let dir = temp("roundtrip");
        let root = server(&dir);
        let before = fs::read(root.join("plugins/Shop/config.yml")).ctx(|| "read".into())?;
        let changes = [Change {
            rel: "plugins/Shop/config.yml".into(),
            content: b"ip: oyna.ornek.net\n".to_vec(),
        }];
        let b = apply(&root, &dir.join("backups"), "replace", &changes)?;
        assert_eq!(b.status, Status::Done);
        assert_eq!(
            fs::read(root.join("plugins/Shop/config.yml")).ctx(|| "read".into())?,
            b"ip: oyna.ornek.net\n"
        );

        restore(&dir.join("backups"), &root, &b.id)?;
        assert_eq!(
            fs::read(root.join("plugins/Shop/config.yml")).ctx(|| "read".into())?,
            before
        );
        assert_eq!(
            list(&dir.join("backups"), &root)[0].status,
            Status::Restored
        );
        Ok(())
    }

    #[test]
    fn broken_yaml_is_refused_and_nothing_changes() -> Result<()> {
        let dir = temp("invalid");
        let root = server(&dir);
        let changes = [
            Change {
                rel: "plugins/Shop/data.json".into(),
                content: b"{\"a\": 2}".to_vec(),
            },
            Change {
                rel: "plugins/Shop/config.yml".into(),
                content: b"ip: [unclosed\n".to_vec(),
            },
        ];
        let res = apply(&root, &dir.join("backups"), "replace", &changes);
        assert!(matches!(res, Err(Error::Invalid { .. })));
        assert_eq!(
            fs::read(root.join("plugins/Shop/data.json")).ctx(|| "read".into())?,
            b"{\"a\": 1}"
        );
        assert!(list(&dir.join("backups"), &root).is_empty());
        Ok(())
    }

    #[test]
    fn protected_paths_are_refused() {
        let dir = temp("protected");
        let root = server(&dir);
        let changes = [Change {
            rel: "world/level.dat".into(),
            content: vec![1],
        }];
        assert!(matches!(
            apply(&root, &dir.join("backups"), "x", &changes),
            Err(Error::Protected(_))
        ));
    }
}
