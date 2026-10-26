// sunucu-analiz-cli scan <folder> [--json] [--online] [--fail-on high|medium|low]
use analiz_core::security::Severity;
use std::path::PathBuf;
use std::process::ExitCode;

fn usage() -> ExitCode {
    eprintln!(
        "kullanım: sunucu-analiz-cli scan <sunucu-klasörü> [--json] [--online] [--fail-on high|medium|low]"
    );
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) != Some("scan") {
        return usage();
    }
    let Some(path) = args.get(1).map(PathBuf::from) else {
        return usage();
    };
    let json = args.iter().any(|a| a == "--json");
    let online = args.iter().any(|a| a == "--online");
    let fail_on = args
        .iter()
        .position(|a| a == "--fail-on")
        .and_then(|i| args.get(i + 1))
        .map(|s| match s.as_str() {
            "high" => Severity::High,
            "medium" => Severity::Medium,
            _ => Severity::Low,
        });

    let opts = analiz_core::Options {
        verify_online: online,
    };
    let report = match analiz_core::scan_with(&path, &opts, &mut |_| {}) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("hata: {e}");
            return ExitCode::from(1);
        }
    };

    if json {
        match serde_json::to_string_pretty(&report) {
            Ok(s) => println!("{s}"),
            Err(e) => {
                eprintln!("hata: {e}");
                return ExitCode::from(1);
            }
        }
    } else {
        println!(
            "{} · {} {}",
            report.name,
            report.platform,
            report.mc_version.as_deref().unwrap_or("")
        );
        println!("sağlık skoru: {}/100", report.score);
        println!("eklenti: {}", report.plugins.len());
        println!("güvenlik bulgusu: {}", report.security.len());
        for f in &report.security {
            println!(
                "  [{:?}] {} · {} ({})",
                f.severity, f.file, f.rule, f.detail
            );
        }
        let spots: usize = report.placeholders.iter().map(|h| h.count).sum();
        println!("örnek değer: {} yerde", spots);
        println!("bozuk karakterli dosya: {}", report.encoding.len());
        println!("optimizasyon önerisi: {}", report.optimize.len());
    }

    let failed = fail_on.is_some_and(|limit| {
        report
            .security
            .iter()
            .any(|f| !f.official && f.severity <= limit)
    });
    if failed {
        ExitCode::from(3)
    } else {
        ExitCode::SUCCESS
    }
}
