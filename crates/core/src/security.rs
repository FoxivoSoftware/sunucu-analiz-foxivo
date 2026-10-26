// Static look at plugin bytecode. Each class's constant pool is parsed so we see
// real method calls and string literals, and the strong rules only fire when the
// signals sit together in the same class. Nothing is loaded or run.
use serde::Serialize;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::classfile::{self, ClassInfo};
use crate::domains;
use crate::plugins::Plugin;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub plugin: String,
    pub file: String,
    /// i18n key under security.rules, the UI owns the wording
    pub rule: &'static str,
    pub severity: Severity,
    pub detail: String,
    /// set when the jar is byte-identical to the official release
    pub official: bool,
}

const MAX_CLASS: u64 = 4 * 1024 * 1024;
const MAX_CLASSES: usize = 20_000;

// Shaded libraries that legitimately do class loading and process calls.
const LIBRARIES: &[&str] = &[
    "net/bytebuddy/",
    "org/objectweb/",
    "kotlin/",
    "org/apache/",
    "com/google/",
    "io/netty/",
    "org/slf4j/",
    "com/fasterxml/",
    "org/yaml/",
    "net/kyori/",
    "com/github/benmanes/",
    "org/bstats/",
    "me/lucko/",
    "org/jetbrains/",
    "com/zaxxer/",
    "org/mariadb/",
    "com/mysql/",
    "org/h2/",
    "org/sqlite/",
    "okhttp3/",
    "okio/",
    "javassist/",
    "org/checkerframework/",
    "it/unimi/",
    "org/json/",
    "com/squareup/",
    "net/byteflux/",
    "com/alessiodp/",
    "org/jsoup/",
    "com/mongodb/",
    "org/postgresql/",
    "redis/",
    "io/lettuce/",
    "org/xerial/",
    "com/sk89q/",
    "org/enginehub/",
];

fn is_library(class: &str) -> bool {
    LIBRARIES
        .iter()
        .any(|p| class.starts_with(p) || class.contains(&format!("/{p}")))
}

fn website_host(site: Option<&str>) -> String {
    let s = site.unwrap_or_default().trim();
    let s = s.split("://").nth(1).unwrap_or(s);
    s.split('/')
        .next()
        .unwrap_or("")
        .trim_start_matches("www.")
        .to_ascii_lowercase()
}

#[derive(Default)]
struct Signals {
    hidden_loader: Vec<String>,
    remote_code: Vec<String>,
    exec_download: Vec<String>,
    chat_op: Vec<String>,
    jar_writer: Vec<String>,
    exec: Vec<String>,
    webhook: bool,
    unknown: Vec<String>,
    short_names: usize,
    classes: usize,
}

fn push(list: &mut Vec<String>, items: impl IntoIterator<Item = String>) {
    for i in items {
        if !list.contains(&i) {
            list.push(i);
        }
    }
}

fn inspect(c: &ClassInfo, known: &[&str], s: &mut Signals) {
    let unknown: Vec<String> = c
        .strings
        .iter()
        .flat_map(|t| domains::in_urls(t))
        .filter(|h| !domains::is_known(h, known))
        .collect();
    let jar_urls: Vec<String> = c
        .strings
        .iter()
        .filter(|t| t.contains(".jar") || t.contains(".class"))
        .flat_map(|t| domains::in_urls(t))
        .filter(|h| !domains::is_known(h, known))
        .collect();

    let exec = c.calls("java/lang/Runtime", "exec") || c.calls("java/lang/ProcessBuilder", "start");
    let define = c.calls_named("defineClass");
    let decode = c.calls("java/util/Base64$Decoder", "decode");
    let url_loader = c.calls("java/net/URLClassLoader", "<init>")
        || c.calls("java/net/URLClassLoader", "newInstance");
    let set_op = c.calls_named("setOp");
    let chat = c.uses_class("AsyncPlayerChatEvent")
        || c.uses_class("PlayerCommandPreprocessEvent")
        || c.uses_class("AsyncChatEvent");
    let lib = is_library(&c.name);

    if define && decode && !lib {
        push(&mut s.hidden_loader, [c.name.clone()]);
    }
    if url_loader && !jar_urls.is_empty() {
        push(&mut s.remote_code, jar_urls);
    }
    if exec && !unknown.is_empty() && !lib {
        push(&mut s.exec_download, unknown.clone());
    }
    if set_op && chat && !lib {
        push(&mut s.chat_op, [c.name.clone()]);
    }
    // how self-spreading malware infects other plugins: build a new jar and
    // write it over an existing one
    let writes_jar = c.calls("java/util/jar/JarOutputStream", "<init>")
        || c.calls("java/util/zip/ZipOutputStream", "<init>");
    if writes_jar && c.strings.iter().any(|t| t.contains(".jar")) && !lib {
        push(&mut s.jar_writer, [c.name.clone()]);
    }
    if exec && !lib {
        push(&mut s.exec, [c.name.clone()]);
    }
    s.webhook |= c.strings.iter().any(|t| {
        t.contains("discord.com/api/webhooks") || t.contains("discordapp.com/api/webhooks")
    });
    push(&mut s.unknown, unknown);
}

pub fn scan_plugin(jar: &Path, plugin: &Plugin) -> Vec<Finding> {
    let Some(mut zip) = File::open(jar)
        .ok()
        .and_then(|f| zip::ZipArchive::new(f).ok())
    else {
        return Vec::new();
    };
    let own = website_host(plugin.website.as_deref());
    let own_base = domains::base(&own).to_string();
    let name = plugin.name.to_ascii_lowercase();
    let mut s = Signals::default();

    for i in 0..zip.len() {
        let Ok(mut entry) = zip.by_index(i) else {
            continue;
        };
        let Ok(path) = entry.name().map(|n| n.to_string()) else {
            continue;
        };
        if !path.ends_with(".class") || entry.size() > MAX_CLASS {
            continue;
        }
        s.classes += 1;
        if s.classes > MAX_CLASSES {
            break;
        }
        let simple = path
            .rsplit('/')
            .next()
            .unwrap_or(&path)
            .trim_end_matches(".class");
        if simple.split('$').next().is_some_and(|n| n.len() <= 2) {
            s.short_names += 1;
        }
        let mut bytes = Vec::with_capacity(entry.size() as usize);
        if entry.read_to_end(&mut bytes).is_err() {
            continue;
        }
        if let Some(info) = classfile::parse(&bytes) {
            inspect(&info, &[own.as_str(), own_base.as_str()], &mut s);
        }
    }

    // a domain carrying the plugin's own name (vulcanac.net for Vulcan) is its own
    let named = |h: &String| name.len() >= 4 && h.contains(&name);
    s.unknown.retain(|h| !named(h));
    s.exec_download.retain(|h| !named(h));
    s.remote_code.retain(|h| !named(h));

    let file = jar
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let mut out = Vec::new();
    let mut add = |rule, severity, detail: String| {
        out.push(Finding {
            plugin: plugin.name.clone(),
            file: file.clone(),
            rule,
            severity,
            detail,
            official: false,
        })
    };
    let short = |v: &[String]| v.iter().take(3).cloned().collect::<Vec<_>>().join(", ");

    if !s.hidden_loader.is_empty() {
        add(
            "hiddenLoader",
            Severity::High,
            short(&s.hidden_loader).replace('/', "."),
        );
    }
    if !s.remote_code.is_empty() {
        add("remoteCode", Severity::High, short(&s.remote_code));
    }
    if !s.exec_download.is_empty() {
        add("execAndDownload", Severity::High, short(&s.exec_download));
    }
    if !s.chat_op.is_empty() {
        add(
            "chatOp",
            Severity::Medium,
            short(&s.chat_op).replace('/', "."),
        );
    }
    if !s.jar_writer.is_empty() {
        add(
            "jarWriter",
            Severity::Medium,
            short(&s.jar_writer).replace('/', "."),
        );
    }
    if s.webhook {
        add("webhook", Severity::Low, "discord.com/api/webhooks".into());
    }
    if !s.exec.is_empty() && s.exec_download.is_empty() {
        add("exec", Severity::Info, short(&s.exec).replace('/', "."));
    }
    if !s.unknown.is_empty() && s.exec_download.is_empty() && s.remote_code.is_empty() {
        add("unknownDomain", Severity::Info, short(&s.unknown));
    }
    if s.classes >= 30 && s.short_names * 10 >= s.classes * 7 {
        add(
            "obfuscated",
            Severity::Low,
            format!("{}/{}", s.short_names, s.classes),
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classfile::build::class;

    fn signals(classes: &[Vec<u8>]) -> Signals {
        let mut s = Signals::default();
        for c in classes {
            if let Some(info) = classfile::parse(c) {
                inspect(&info, &[], &mut s);
            }
        }
        s
    }

    #[test]
    fn exec_and_url_in_different_classes_is_not_a_dropper() {
        let s = signals(&[
            class("a/Main", &[("java/lang/Runtime", "exec")], &[]),
            class("a/Update", &[], &["https://cdn-update.xyz/v.json"]),
        ]);
        assert!(s.exec_download.is_empty());
        assert_eq!(s.exec.len(), 1);
    }

    #[test]
    fn exec_and_url_in_the_same_class_is_flagged() {
        let s = signals(&[class(
            "a/Main",
            &[("java/lang/Runtime", "exec")],
            &["https://cdn-update.xyz/p.sh"],
        )]);
        assert_eq!(s.exec_download, vec!["cdn-update.xyz"]);
    }

    #[test]
    fn similar_names_dont_count() {
        let s = signals(&[class(
            "a/Main",
            &[
                ("java/util/concurrent/Executor", "execute"),
                ("a/Ui", "setOpacity"),
            ],
            &[],
        )]);
        assert!(s.exec.is_empty() && s.chat_op.is_empty());
    }

    #[test]
    fn writing_jars_is_flagged_only_with_a_jar_target() {
        let infector = signals(&[class(
            "a/Spread",
            &[("java/util/jar/JarOutputStream", "<init>")],
            &["plugins/", ".jar"],
        )]);
        assert_eq!(infector.jar_writer.len(), 1);

        let exporter = signals(&[class(
            "a/Export",
            &[("java/util/zip/ZipOutputStream", "<init>")],
            &["backup.zip"],
        )]);
        assert!(exporter.jar_writer.is_empty());
    }

    #[test]
    fn shaded_libraries_are_ignored_for_loaders() {
        let s = signals(&[class(
            "com/viaversion/libs/net/bytebuddy/Loader",
            &[
                ("java/lang/ClassLoader", "defineClass"),
                ("java/util/Base64$Decoder", "decode"),
            ],
            &[],
        )]);
        assert!(s.hidden_loader.is_empty());
    }
}
