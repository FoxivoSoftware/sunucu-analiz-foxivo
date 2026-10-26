// Whole flow on a throwaway server: scan -> fix -> apply -> rescan -> restore.
use analiz_core::{fix, safety, scan};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

fn server() -> std::io::Result<PathBuf> {
    let dir = std::env::temp_dir().join(format!("analiz-flow-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let root = dir.join("survival");
    fs::create_dir_all(root.join("plugins/TAB"))?;
    fs::create_dir_all(root.join("world"))?;
    fs::write(
        root.join("server.properties"),
        "motd=sunucuadiniz.com\nview-distance=12\nserver-port=1\n",
    )?;
    fs::write(
        root.join("plugins/TAB/config.yml"),
        "header:\n  - '&bSunucu Adı'\n  - '&7sunucuadiniz.com'\nmsg: 'GÃ¼venli'\n",
    )?;
    fs::write(root.join("world/level.dat"), "sunucuadiniz.com")?;
    Ok(root)
}

#[test]
fn scan_fix_and_undo() -> Result<(), Box<dyn Error>> {
    let root = server()?;
    let backups = root.with_file_name("backups");
    let before = fs::read(root.join("plugins/TAB/config.yml"))?;

    let report = scan(&root)?;
    let spots: usize = report.placeholders.iter().map(|h| h.count).sum();
    assert_eq!(spots, 3, "world/ must not be counted");
    assert_eq!(report.encoding.len(), 1);
    assert!(report.optimize.iter().any(|s| s.id == "viewDistance"));

    let mut values = BTreeMap::new();
    values.insert(
        "sunucuadiniz.com".to_string(),
        "oyna.gokyuzu.net".to_string(),
    );
    let plan = fix::replace(&root, &values);
    assert_eq!(plan.changes.len(), 2);
    let backup = safety::apply(&root, &backups, "replace", &plan.changes)?;

    let after = scan(&root)?;
    assert!(
        after
            .placeholders
            .iter()
            .all(|h| h.value != "sunucuadiniz.com")
    );
    assert_eq!(
        fs::read_to_string(root.join("world/level.dat"))?,
        "sunucuadiniz.com"
    );

    safety::restore(&backups, &root, &backup.id)?;
    assert_eq!(fs::read(root.join("plugins/TAB/config.yml"))?, before);
    Ok(())
}
