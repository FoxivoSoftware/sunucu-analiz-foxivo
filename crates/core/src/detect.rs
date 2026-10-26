use std::fs;
use std::path::Path;

pub struct ServerInfo {
    pub platform: String,
    pub mc_version: Option<String>,
}

/// Looks at the server jar names and config files to guess Paper/Spigot/Purpur/Folia.
pub fn detect(root: &Path) -> ServerInfo {
    let jars: Vec<String> = fs::read_dir(root)
        .map(|it| {
            it.filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().to_ascii_lowercase())
                .filter(|n| n.ends_with(".jar"))
                .collect()
        })
        .unwrap_or_default();

    let has = |name: &str| root.join(name).exists();
    let jar_has = |word: &str| jars.iter().any(|j| j.contains(word));

    let platform = if jar_has("folia") {
        "Folia"
    } else if has("purpur.yml") || jar_has("purpur") {
        "Purpur"
    } else if has("config/paper-global.yml") || has("paper.yml") || jar_has("paper") {
        "Paper"
    } else if has("spigot.yml") || jar_has("spigot") {
        "Spigot"
    } else if has("bukkit.yml") || jar_has("bukkit") {
        "Bukkit"
    } else if has("velocity.toml") {
        "Velocity"
    } else if jar_has("bungee") || jar_has("waterfall") {
        "BungeeCord"
    } else {
        "Minecraft"
    };

    ServerInfo {
        platform: platform.to_string(),
        mc_version: version_from_history(root).or_else(|| version_from_jars(&jars)),
    }
}

// Paper keeps {"currentVersion":"git-Paper-123 (MC: 1.21.1)"} here
fn version_from_history(root: &Path) -> Option<String> {
    let text = fs::read_to_string(root.join("version_history.json")).ok()?;
    let start = text.find("(MC: ")? + 5;
    let end = text[start..].find(')')? + start;
    Some(text[start..end].trim().to_string())
}

fn version_from_jars(jars: &[String]) -> Option<String> {
    jars.iter().find_map(|j| {
        j.split(['-', '_'])
            .find(|part| {
                part.starts_with("1.")
                    && part[2..].chars().next().is_some_and(|c| c.is_ascii_digit())
            })
            .map(|v| v.trim_end_matches(".jar").to_string())
    })
}

/// A folder counts as a server if it has the usual files a server writes on first start.
pub fn looks_like_server(root: &Path) -> bool {
    [
        "server.properties",
        "plugins",
        "bukkit.yml",
        "spigot.yml",
        "velocity.toml",
        "eula.txt",
    ]
    .iter()
    .any(|f| root.join(f).exists())
}

pub fn server_name(root: &Path) -> String {
    root.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| root.to_string_lossy().to_string())
}
