use serde::Serialize;

use crate::plugins::Plugin;

const KNOWN: &[&str] = &[
    "Vulcan",
    "Matrix",
    "Grim",
    "GrimAC",
    "Spartan",
    "NoCheatPlus",
    "AAC",
    "AAC5",
    "Intave",
    "Themis",
    "Polar",
    "Karhu",
    "Verus",
    "Negativity",
    "Horizon",
    "Kauri",
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Anticheat {
    pub name: String,
    pub file: String,
    pub version: Option<String>,
}

pub fn detect(plugins: &[Plugin]) -> Vec<Anticheat> {
    plugins
        .iter()
        .filter(|p| KNOWN.iter().any(|k| p.name.eq_ignore_ascii_case(k)))
        .map(|p| Anticheat {
            name: p.name.clone(),
            file: p.file.clone(),
            version: p.version.clone(),
        })
        .collect()
}
