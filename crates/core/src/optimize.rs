use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Suggestion {
    /// i18n key under optimize.tips
    pub id: &'static str,
    pub file: String,
    pub key: String,
    pub current: String,
    pub recommended: String,
    /// changes how the game plays, never preselected in the UI
    pub gameplay: bool,
    /// false for things we only explain (start script flags)
    pub applicable: bool,
}

pub const AIKAR_FLAGS: &str = "-XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:G1HeapWastePercent=5 -XX:G1MixedGCCountTarget=4 -XX:InitiatingHeapOccupancyPercent=15 -XX:G1MixedGCLiveThresholdPercent=90 -XX:G1RSetUpdatingPauseTimePercent=5 -XX:SurvivorRatio=32 -XX:+PerfDisableSharedMem -XX:MaxTenuringThreshold=1 -Dusing.aikars.flags=https://mcflags.emc.gs -Daikars.new.flags=true";

pub fn property(text: &str, key: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.starts_with('#'))
        .find_map(|l| {
            let (k, v) = l.split_once('=')?;
            (k.trim() == key).then(|| v.trim().to_string())
        })
}

/// Rewrites one `key=value` line, keeping everything else byte for byte.
pub fn set_property(text: &str, key: &str, value: &str) -> String {
    let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut out: Vec<String> = text
        .lines()
        .map(|l| match l.split_once('=') {
            Some((k, _)) if k.trim() == key && !l.trim_start().starts_with('#') => {
                format!("{key}={value}")
            }
            _ => l.to_string(),
        })
        .collect();
    if property(text, key).is_none() {
        out.push(format!("{key}={value}"));
    }
    let mut joined = out.join(newline);
    if text.ends_with('\n') {
        joined.push_str(newline);
    }
    joined
}

pub fn suggest(root: &Path) -> Vec<Suggestion> {
    let mut out = Vec::new();

    if let Ok(props) = fs::read_to_string(root.join("server.properties")) {
        let num = |k: &str| property(&props, k).and_then(|v| v.parse::<u32>().ok());
        let mut tip = |id, key: &str, current: u32, recommended: u32, gameplay| {
            out.push(Suggestion {
                id,
                file: "server.properties".into(),
                key: key.into(),
                current: current.to_string(),
                recommended: recommended.to_string(),
                gameplay,
                applicable: true,
            })
        };
        if let Some(v) = num("view-distance").filter(|v| *v > 8) {
            tip("viewDistance", "view-distance", v, 8, false);
        }
        if let Some(v) = num("simulation-distance").filter(|v| *v > 6) {
            tip("simulationDistance", "simulation-distance", v, 6, true);
        }
        if property(&props, "sync-chunk-writes").as_deref() == Some("true") {
            out.push(Suggestion {
                id: "syncChunkWrites",
                file: "server.properties".into(),
                key: "sync-chunk-writes".into(),
                current: "true".into(),
                recommended: "false".into(),
                gameplay: false,
                applicable: true,
            });
        }
    }

    // a start script that runs java without G1 tuning
    let scripts = fs::read_dir(root)
        .map(|it| {
            it.filter_map(|e| e.ok())
                .map(|e| e.path())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for script in scripts {
        let ext = crate::walk::ext(&script);
        if !matches!(ext.as_str(), "bat" | "cmd" | "sh" | "command") {
            continue;
        }
        let Ok(text) = fs::read_to_string(&script) else {
            continue;
        };
        if text.contains("java") && text.contains("-jar") && !text.contains("MaxGCPauseMillis") {
            out.push(Suggestion {
                id: "aikarFlags",
                file: crate::walk::rel(root, &script),
                key: "JVM".into(),
                current: "-".into(),
                recommended: AIKAR_FLAGS.into(),
                gameplay: false,
                applicable: false,
            });
            break;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edits_only_the_key() {
        let text = "#comment\r\nview-distance=12\r\nmotd=Merhaba\r\n";
        let out = set_property(text, "view-distance", "8");
        assert_eq!(out, "#comment\r\nview-distance=8\r\nmotd=Merhaba\r\n");
    }
}
