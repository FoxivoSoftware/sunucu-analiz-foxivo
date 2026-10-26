// Turns what the user approved into concrete file changes. Nothing here writes;
// `safety::apply` does that.
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::safety::Change;
use crate::{encoding, optimize, placeholders, walk};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    pub file: String,
    pub changes: usize,
    pub before: String,
    pub after: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub previews: Vec<Preview>,
    #[serde(skip)]
    pub changes: Vec<Change>,
}

fn clip(s: &str) -> String {
    let s = s.trim();
    if s.chars().count() > 100 {
        format!("{}…", s.chars().take(100).collect::<String>())
    } else {
        s.to_string()
    }
}

fn first_diff(old: &str, new: &str) -> (String, String) {
    old.lines()
        .zip(new.lines())
        .find(|(a, b)| a != b)
        .map(|(a, b)| (clip(a), clip(b)))
        .unwrap_or_default()
}

pub fn replace(root: &Path, map: &BTreeMap<String, String>) -> Plan {
    let mut plan = Plan {
        previews: Vec::new(),
        changes: Vec::new(),
    };
    let mut keys: Vec<String> = map
        .iter()
        .filter(|(_, v)| !v.is_empty())
        .map(|(k, _)| k.clone())
        .collect();
    keys.sort_by_key(|k| std::cmp::Reverse(k.chars().count()));
    for path in walk::text_files(root) {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let new = placeholders::replace(&text, map);
        if new == text {
            continue;
        }
        let rel = walk::rel(root, &path);
        let count: usize = placeholders::count_in(&text, &keys)
            .iter()
            .map(|(_, n)| n)
            .sum();
        let (before, after) = first_diff(&text, &new);
        plan.previews.push(Preview {
            file: rel.clone(),
            changes: count,
            before,
            after,
        });
        plan.changes.push(Change {
            rel,
            content: new.into_bytes(),
        });
    }
    plan
}

pub fn encoding(root: &Path, files: &[String]) -> Plan {
    let mut plan = Plan {
        previews: Vec::new(),
        changes: Vec::new(),
    };
    for rel in files {
        if walk::is_protected(rel) {
            continue;
        }
        let Ok(bytes) = fs::read(root.join(rel)) else {
            continue;
        };
        let (Some(issue), Some(content)) = (encoding::check(rel, &bytes), encoding::fixed(&bytes))
        else {
            continue;
        };
        plan.previews.push(Preview {
            file: rel.clone(),
            changes: issue.lines,
            before: issue.before,
            after: issue.after,
        });
        plan.changes.push(Change {
            rel: rel.clone(),
            content,
        });
    }
    plan
}

pub fn optimize(root: &Path, ids: &[String]) -> Plan {
    let mut plan = Plan {
        previews: Vec::new(),
        changes: Vec::new(),
    };
    let picked: Vec<_> = optimize::suggest(root)
        .into_iter()
        .filter(|s| s.applicable && ids.iter().any(|id| id == s.id))
        .collect();
    let Ok(original) = fs::read_to_string(root.join("server.properties")) else {
        return plan;
    };
    let mut text = original.clone();
    for s in &picked {
        text = optimize::set_property(&text, &s.key, &s.recommended);
        plan.previews.push(Preview {
            file: s.file.clone(),
            changes: 1,
            before: format!("{}={}", s.key, s.current),
            after: format!("{}={}", s.key, s.recommended),
        });
    }
    if text != original {
        plan.changes.push(Change {
            rel: "server.properties".into(),
            content: text.into_bytes(),
        });
    }
    plan
}
