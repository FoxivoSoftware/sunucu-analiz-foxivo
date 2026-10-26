use serde::Serialize;

use crate::anticheat::Anticheat;
use crate::encoding::Issue;
use crate::optimize::Suggestion;
use crate::placeholders::Hit;
use crate::plugins::Plugin;
use crate::security::{Finding, Severity};

/// Points a module takes off the score, so the UI can say what to fix for how much.
#[derive(Debug, Clone, Serialize)]
pub struct Penalty {
    /// module id: security | replace | encoding | optimize | anticheat
    pub id: &'static str,
    pub points: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub root: String,
    pub name: String,
    pub platform: String,
    pub mc_version: Option<String>,
    pub plugins: Vec<Plugin>,
    pub security: Vec<Finding>,
    pub placeholders: Vec<Hit>,
    pub encoding: Vec<Issue>,
    pub optimize: Vec<Suggestion>,
    pub anticheats: Vec<Anticheat>,
    pub running: bool,
    pub score: u8,
    pub breakdown: Vec<Penalty>,
    pub files_scanned: usize,
    pub duration_ms: u64,
}

impl Report {
    /// Rough 0-100 summary. Weights are deliberately simple so they're easy to explain.
    pub fn compute_score(&mut self) {
        // findings inside a byte-identical official release are the author's own code
        let sec = |s: Severity| {
            self.security
                .iter()
                .filter(|f| f.severity == s && !f.official)
                .count() as u32
        };
        let security = (sec(Severity::High) * 15).min(45)
            + (sec(Severity::Medium) * 6).min(18)
            + (sec(Severity::Low) * 2).min(6);
        let replace = if self.placeholders.is_empty() {
            0
        } else {
            let spots: usize = self.placeholders.iter().map(|h| h.count).sum();
            (6 + spots as u32 / 20).min(14)
        };
        let encoding = (self.encoding.len() as u32 * 2).min(10);
        let optimize = (self.optimize.len() as u32 * 3).min(9);
        let anticheat = if self.anticheats.len() > 1 { 6 } else { 0 };

        self.breakdown = [
            ("security", security),
            ("replace", replace),
            ("encoding", encoding),
            ("optimize", optimize),
            ("anticheat", anticheat),
        ]
        .into_iter()
        .filter(|(_, p)| *p > 0)
        .map(|(id, points)| Penalty { id, points })
        .collect();
        self.breakdown.sort_by_key(|p| std::cmp::Reverse(p.points));

        let total: u32 = self.breakdown.iter().map(|p| p.points).sum();
        self.score = 100u32.saturating_sub(total) as u8;
    }
}
