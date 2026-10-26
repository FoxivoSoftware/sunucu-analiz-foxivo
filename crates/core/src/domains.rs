// Host names: pulling them out of text and telling well-known ones apart.

/// Domains plugins commonly talk to or mention in docs.
pub const KNOWN: &[&str] = &[
    "github.com",
    "githubusercontent.com",
    "github.io",
    "gitlab.com",
    "spigotmc.org",
    "papermc.io",
    "bstats.org",
    "modrinth.com",
    "mojang.com",
    "minecraft.net",
    "minecraftservices.com",
    "bukkit.org",
    "curseforge.com",
    "maven.org",
    "apache.org",
    "sonatype.org",
    "jitpack.io",
    "codemc.io",
    "codemc.org",
    "enginehub.org",
    "luckperms.net",
    "essentialsx.net",
    "placeholderapi.com",
    "helpch.at",
    "extendedclip.com",
    "w3.org",
    "xml.org",
    "json.org",
    "json-schema.org",
    "oracle.com",
    "java.com",
    "java.net",
    "sun.com",
    "openjdk.org",
    "kyori.net",
    "advntr.dev",
    "discord.gg",
    "discord.com",
    "discordapp.com",
    "google.com",
    "googleapis.com",
    "gstatic.com",
    "youtube.com",
    "mc-heads.net",
    "crafatar.com",
    "minotar.net",
    "mineskin.org",
    "geysermc.org",
    "viaversion.com",
    "spongepowered.org",
    "fabricmc.net",
    "jetbrains.com",
    "eclipse.org",
    "slf4j.org",
    "fasterxml.com",
    "yaml.org",
    "mozilla.org",
    "creativecommons.org",
    "opensource.org",
    "gnu.org",
    "wikipedia.org",
    "gamepedia.com",
    "minecraft.wiki",
    "example.com",
    "example.org",
    "localhost",
    "polymart.org",
    "builtbybit.com",
    "mc-market.org",
    "songoda.com",
    "craftaro.com",
    "lucko.me",
    "netty.io",
    "spiget.org",
    "intellectualsites.com",
    "athion.net",
    "schematic.cloud",
    "youtu.be",
    "planetminecraft.com",
    "protobuf.dev",
    "rabbitmq.com",
    "sk89q.com",
    "redis.io",
    "mongodb.com",
    "mysql.com",
    "mariadb.org",
    "postgresql.org",
    "h2database.com",
    "sqlite.org",
    "twitch.tv",
    "twitter.com",
    "x.com",
    "patreon.com",
    "paypal.me",
    "ko-fi.com",
    "buymeacoffee.com",
    "spdx.org",
    "imgur.com",
    "pastes.dev",
    "ashcon.app",
    "mcflags.emc.gs",
    "bintray.com",
    "jsoup.org",
    "fancyinnovations.com",
    "fancyspaces.net",
    "fancyanalytics.net",
    "lewisdev.fun",
    "zithium.net",
    "mcstats.org",
    "hastebin.com",
    "paste.gg",
    "pastebin.com",
    "neznamy.tech",
    "appspot.com",
    "medium.com",
    "cloudflare.com",
    "amazonaws.com",
    "microsoft.com",
    "xbox.com",
];

fn tld_ok(host: &str) -> bool {
    host.rsplit('.').next().is_some_and(|tld| {
        (2..=6).contains(&tld.len()) && tld.chars().all(|c| c.is_ascii_lowercase())
    })
}

/// Base domain, good enough for allowlisting: "api.foo.co.uk" isn't handled specially.
pub fn base(host: &str) -> &str {
    let mut dots = host.match_indices('.').map(|(i, _)| i).collect::<Vec<_>>();
    dots.pop();
    dots.last().map(|i| &host[i + 1..]).unwrap_or(host)
}

pub fn is_known(host: &str, extra: &[&str]) -> bool {
    let matches = |d: &str| host == d || host.ends_with(&format!(".{d}"));
    KNOWN.iter().any(|d| matches(d)) || extra.iter().any(|d| !d.is_empty() && matches(d))
}

/// Hosts of every http(s) URL in the text.
pub fn in_urls(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (pos, _) in text.match_indices("://") {
        if !text[..pos].ends_with("http") && !text[..pos].ends_with("https") {
            continue;
        }
        let host: String = text[pos + 3..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '-')
            .collect::<String>()
            .trim_matches('.')
            .to_ascii_lowercase();
        if host.contains('.') && tld_ok(&host) && !out.contains(&host) {
            out.push(host);
        }
    }
    out
}

/// Blanks out Minecraft color codes (&a, §a, &#RRGGBB, <#RRGGBB>) so they don't
/// stick to the word next to them.
fn strip_colors(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let hex6 = |i: usize| {
        chars
            .get(i..i + 6)
            .is_some_and(|s| s.iter().all(|c| c.is_ascii_hexdigit()))
    };
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        let skip = if (c == '&' || c == '§') && next == Some('#') && hex6(i + 2) {
            8
        } else if c == '<' && next == Some('#') && hex6(i + 2) && chars.get(i + 8) == Some(&'>') {
            9
        } else if (c == '&' || c == '§')
            && next.is_some_and(|n| "0123456789abcdefklmnorABCDEFKLMNOR".contains(n))
        {
            2
        } else {
            0
        };
        if skip > 0 {
            out.push(' ');
            i += skip;
        } else {
            out.push(c);
            i += 1;
        }
    }
    out
}

/// Host-like words without a scheme, e.g. "magaza.montana.com" in a scoreboard line.
pub fn bare(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let text = strip_colors(text);
    for word in
        text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '/'))
    {
        // case is kept so the value can be found again in the file as written
        let host = word
            .split('/')
            .next()
            .unwrap_or("")
            .trim_matches('.')
            .to_string();
        if host.matches('.').count() >= 1
            && tld_ok(&host.to_ascii_lowercase())
            && host
                .split('.')
                .all(|l| !l.is_empty() && !l.starts_with('-'))
            && host.chars().any(|c| c.is_ascii_alphabetic())
            && !out.contains(&host)
        {
            out.push(host);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hosts() {
        assert_eq!(
            in_urls("see https://cdn-update.xyz/a.jar and http://javax.xml.XMLConstants/p"),
            vec!["cdn-update.xyz"]
        );
        assert_eq!(
            bare("&7ᴍᴀɢᴀᴢᴀ &#FFBE6Cmagaza.montana.com"),
            vec!["magaza.montana.com"]
        );
        assert_eq!(base("magaza.montana.com"), "montana.com");
        assert!(is_known("api.github.com", &[]));
        assert!(is_known("vulcanac.net", &["vulcanac.net"]));
    }
}
