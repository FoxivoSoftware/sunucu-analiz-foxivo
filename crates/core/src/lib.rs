//! Scans a Minecraft server folder without changing anything, and applies
//! approved fixes through a backup -> atomic write -> validate -> rollback pipeline.

pub mod anticheat;
pub mod classfile;
pub mod detect;
pub mod domains;
pub mod encoding;
pub mod error;
pub mod fix;
pub mod optimize;
pub mod placeholders;
pub mod plugins;
pub mod report;
pub mod running;
pub mod safety;
pub mod security;
pub mod verify;
pub mod walk;

use serde::Serialize;
use std::fs;
use std::path::Path;
use std::time::Instant;

pub use error::{Error, Result};
pub use report::Report;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    /// plugins | verify | files | done
    pub phase: &'static str,
    pub done: usize,
    pub total: usize,
    pub current: String,
}

pub struct Options {
    /// look the jars up on Modrinth by hash (only the hash leaves the machine)
    pub verify_online: bool,
}

pub fn scan(root: &Path) -> Result<Report> {
    scan_with(
        root,
        &Options {
            verify_online: false,
        },
        &mut |_| {},
    )
}

pub fn scan_with(
    root: &Path,
    opts: &Options,
    progress: &mut dyn FnMut(Progress),
) -> Result<Report> {
    let started = Instant::now();
    if !root.is_dir() {
        return Err(Error::NotFound(root.to_path_buf()));
    }
    if !detect::looks_like_server(root) {
        return Err(Error::NotAServer);
    }

    let info = detect::detect(root);
    let jars = plugins::jar_paths(root);
    let mut plugin_list = Vec::with_capacity(jars.len());
    let mut security = Vec::new();
    for (i, jar) in jars.iter().enumerate() {
        let plugin = plugins::read_plugin(jar);
        progress(Progress {
            phase: "plugins",
            done: i,
            total: jars.len(),
            current: plugin.name.clone(),
        });
        security.extend(security::scan_plugin(jar, &plugin));
        plugin_list.push(plugin);
    }

    if opts.verify_online && !jars.is_empty() {
        progress(Progress {
            phase: "verify",
            done: 0,
            total: jars.len(),
            current: String::new(),
        });
        let official = verify::modrinth(&jars, &info.platform, info.mc_version.as_deref());
        for (plugin, jar) in plugin_list.iter_mut().zip(&jars) {
            plugin.official = official.get(jar).cloned();
        }
        for f in &mut security {
            f.official = plugin_list
                .iter()
                .any(|p| p.file == f.file && p.official.is_some());
        }
    }

    let files = walk::text_files(root);
    let mut texts = Vec::new();
    let mut encoding_issues = Vec::new();
    for (i, path) in files.iter().enumerate() {
        let rel = walk::rel(root, path);
        if i % 25 == 0 {
            progress(Progress {
                phase: "files",
                done: i,
                total: files.len(),
                current: rel.clone(),
            });
        }
        let Ok(bytes) = fs::read(path) else { continue };
        if let Some(issue) = encoding::check(&rel, &bytes) {
            encoding_issues.push(issue);
        }
        if let Ok(text) = String::from_utf8(bytes) {
            texts.push((rel, text));
        }
    }

    security.sort_by_key(|f| (f.official, f.severity));

    let mut report = Report {
        root: root.to_string_lossy().to_string(),
        name: detect::server_name(root),
        platform: info.platform,
        mc_version: info.mc_version,
        anticheats: anticheat::detect(&plugin_list),
        plugins: plugin_list,
        security,
        placeholders: placeholders::collect(&texts),
        encoding: encoding_issues,
        optimize: optimize::suggest(root),
        running: running::is_running(root),
        score: 0,
        breakdown: Vec::new(),
        files_scanned: files.len(),
        duration_ms: 0,
    };
    report.compute_score();
    report.duration_ms = started.elapsed().as_millis() as u64;
    progress(Progress {
        phase: "done",
        done: 1,
        total: 1,
        current: String::new(),
    });
    Ok(report)
}
