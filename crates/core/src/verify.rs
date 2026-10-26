// The only check that gives a definite answer: is this jar byte for byte the file
// the author uploaded? Only the SHA-512 of each jar (plus the server software and
// Minecraft version, for update lookups) is sent to Modrinth, never the file or
// anything else about the server. Any network problem just means "not verified".
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

const API: &str = "https://api.modrinth.com/v2";
const USER_AGENT: &str = concat!(
    "foxivosoftware/sunucu-analiz-foxivo/",
    env!("CARGO_PKG_VERSION"),
    " (foxivo.com.tr)"
);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Official {
    pub source: &'static str,
    pub version: String,
    pub url: String,
    /// newer release for this server software and Minecraft version, if any
    pub latest: Option<String>,
}

#[derive(Deserialize)]
struct Version {
    project_id: String,
    version_number: String,
    #[serde(default)]
    version_type: String,
    #[serde(default)]
    date_published: String,
}

pub fn sha512(path: &Path) -> Option<String> {
    let mut file = File::open(path).ok()?;
    let mut hasher = Sha512::new();
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf).ok()?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Some(hex::encode(hasher.finalize()))
}

/// Modrinth loader names that can run plugins on this server software.
pub fn loaders(platform: &str) -> Vec<&'static str> {
    match platform {
        "Folia" => vec!["folia"],
        "Purpur" => vec!["purpur", "paper", "spigot", "bukkit"],
        "Paper" => vec!["paper", "spigot", "bukkit"],
        "Spigot" | "Bukkit" => vec!["spigot", "bukkit"],
        "Velocity" => vec!["velocity"],
        "BungeeCord" => vec!["bungeecord", "waterfall"],
        _ => vec!["paper", "spigot", "bukkit"],
    }
}

/// `None` on any network or parse failure.
fn post(
    agent: &ureq::Agent,
    path: &str,
    body: serde_json::Value,
) -> Option<HashMap<String, Version>> {
    agent
        .post(format!("{API}{path}"))
        .header("User-Agent", USER_AGENT)
        .send_json(&body)
        .ok()
        .and_then(|mut res| res.body_mut().read_json().ok())
}

// Answers per file hash for this run of the app, so the rescan after a fix doesn't
// ask Modrinth again about jars that didn't change. None = not on Modrinth.
static CACHE: LazyLock<Mutex<HashMap<String, Option<Official>>>> = LazyLock::new(Default::default);

pub fn modrinth(
    jars: &[PathBuf],
    platform: &str,
    mc_version: Option<&str>,
) -> HashMap<PathBuf, Official> {
    let hashes: Vec<(PathBuf, String)> = jars
        .iter()
        .filter_map(|j| Some((j.clone(), sha512(j)?)))
        .collect();

    let cached = |h: &str| CACHE.lock().ok().and_then(|c| c.get(h).cloned());
    let missing: Vec<String> = hashes
        .iter()
        .filter(|(_, h)| cached(h).is_none())
        .map(|(_, h)| h.clone())
        .collect();
    if !missing.is_empty() {
        let answers = lookup(&missing, platform, mc_version);
        // only remember answers from a lookup that reached Modrinth
        if let (Some(answers), Ok(mut cache)) = (answers, CACHE.lock()) {
            for h in missing {
                let official = answers.get(&h).cloned();
                cache.insert(h, official);
            }
        }
    }

    hashes
        .into_iter()
        .filter_map(|(path, hash)| Some((path, cached(&hash)??)))
        .collect()
}

/// `None` when Modrinth couldn't be reached, so nothing gets cached.
fn lookup(
    hashes: &[String],
    platform: &str,
    mc_version: Option<&str>,
) -> Option<HashMap<String, Official>> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(8)))
        .build()
        .into();

    let found = post(
        &agent,
        "/version_files",
        serde_json::json!({
            "hashes": hashes,
            "algorithm": "sha512",
        }),
    )?;
    if found.is_empty() {
        return Some(HashMap::new());
    }

    // newest release for the same project that fits this server
    let verified: Vec<&String> = hashes.iter().filter(|h| found.contains_key(*h)).collect();
    let mut filter = serde_json::json!({
        "hashes": verified,
        "algorithm": "sha512",
        "loaders": loaders(platform),
    });
    if let Some(v) = mc_version {
        filter["game_versions"] = serde_json::json!([v]);
    }
    let latest = post(&agent, "/version_files/update", filter).unwrap_or_default();

    // Only ever suggest a stable release that came out after the installed one.
    // The update endpoint can answer with alpha/beta builds, so fall back to the
    // project's version list and take its newest release.
    let newer_release = |current: &Version, hash: &str| -> Option<String> {
        let candidate = latest.get(hash)?;
        if candidate.version_number == current.version_number {
            return None;
        }
        let pick = if candidate.version_type == "release" {
            Some((
                candidate.version_number.clone(),
                candidate.date_published.clone(),
            ))
        } else {
            let mut req = agent
                .get(format!("{API}/project/{}/version", current.project_id))
                .header("User-Agent", USER_AGENT)
                .query("loaders", serde_json::json!(loaders(platform)).to_string());
            if let Some(v) = mc_version {
                req = req.query("game_versions", serde_json::json!([v]).to_string());
            }
            let list: Vec<Version> = req
                .call()
                .ok()
                .and_then(|mut res| res.body_mut().read_json().ok())
                .unwrap_or_default();
            list.into_iter()
                .find(|v| v.version_type == "release")
                .map(|v| (v.version_number, v.date_published))
        };
        // ISO 8601 timestamps compare correctly as strings
        pick.filter(|(number, date)| {
            *number != current.version_number && *date > current.date_published
        })
        .map(|(number, _)| number)
    };

    Some(
        hashes
            .iter()
            .filter_map(|hash| {
                let v = found.get(hash)?;
                Some((
                    hash.clone(),
                    Official {
                        source: "Modrinth",
                        version: v.version_number.clone(),
                        url: format!("https://modrinth.com/project/{}", v.project_id),
                        latest: newer_release(v, hash),
                    },
                ))
            })
            .collect(),
    )
}
