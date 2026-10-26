mod installer;
mod tray;

use analiz_core::fix::{self, Plan, Preview};
use analiz_core::safety::{self, Backup};
use analiz_core::{Options, Progress, Report};
use std::collections::BTreeMap;
use std::path::PathBuf;
use tauri::Emitter;
use tauri_plugin_opener::OpenerExt;

type CmdResult<T> = Result<T, String>;

// scanning and writing touch the disk a lot, keep them off the UI thread
async fn blocking<T, F>(f: F) -> CmdResult<T>
where
    T: Send + 'static,
    F: FnOnce() -> CmdResult<T> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| e.to_string())?
}

/// Streams `scan-progress` events while it works so the UI can show real progress.
#[tauri::command]
async fn scan_server(app: tauri::AppHandle, path: String, online: bool) -> CmdResult<Report> {
    blocking(move || {
        let mut emit = |p: Progress| {
            let _ = app.emit("scan-progress", p);
        };
        let opts = Options {
            verify_online: online,
        };
        analiz_core::scan_with(&PathBuf::from(path), &opts, &mut emit).map_err(|e| e.to_string())
    })
    .await
}

#[tauri::command]
async fn is_running(path: String) -> CmdResult<bool> {
    blocking(move || Ok(analiz_core::running::is_running(&PathBuf::from(path)))).await
}

#[derive(serde::Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
enum FixRequest {
    Replace { values: BTreeMap<String, String> },
    Encoding { files: Vec<String> },
    Optimize { ids: Vec<String> },
}

impl FixRequest {
    fn label(&self) -> &'static str {
        match self {
            FixRequest::Replace { .. } => "replace",
            FixRequest::Encoding { .. } => "encoding",
            FixRequest::Optimize { .. } => "optimize",
        }
    }

    fn plan(&self, root: &std::path::Path) -> Plan {
        match self {
            FixRequest::Replace { values } => fix::replace(root, values),
            FixRequest::Encoding { files } => fix::encoding(root, files),
            FixRequest::Optimize { ids } => fix::optimize(root, ids),
        }
    }
}

/// What would change, without writing anything.
#[tauri::command]
async fn preview_fix(path: String, fix: FixRequest) -> CmdResult<Vec<Preview>> {
    blocking(move || Ok(fix.plan(&PathBuf::from(path)).previews)).await
}

/// Called only after the user approved the preview. The plan is rebuilt here
/// from the files on disk rather than trusting anything sent from the UI.
#[tauri::command]
async fn apply_fix(path: String, backup_dir: String, fix: FixRequest) -> CmdResult<Backup> {
    blocking(move || {
        let root = PathBuf::from(path);
        let plan = fix.plan(&root);
        if plan.changes.is_empty() {
            return Err("değiştirilecek bir şey kalmadı".into());
        }
        safety::apply(
            &root,
            &safety::expand_home(&backup_dir),
            fix.label(),
            &plan.changes,
        )
        .map_err(|e| e.to_string())
    })
    .await
}

#[tauri::command]
async fn list_backups(path: String, backup_dir: String) -> CmdResult<Vec<Backup>> {
    blocking(move || {
        Ok(safety::list(
            &safety::expand_home(&backup_dir),
            &PathBuf::from(path),
        ))
    })
    .await
}

#[tauri::command]
async fn restore_backup(path: String, backup_dir: String, id: String) -> CmdResult<Backup> {
    blocking(move || {
        safety::restore(&safety::expand_home(&backup_dir), &PathBuf::from(path), &id)
            .map_err(|e| e.to_string())
    })
    .await
}

#[tauri::command]
async fn cleanup_backups(backup_dir: String, days: u64) -> CmdResult<usize> {
    blocking(move || Ok(safety::cleanup(&safety::expand_home(&backup_dir), days))).await
}

#[tauri::command]
fn open_backup_dir(app: tauri::AppHandle, backup_dir: String) -> CmdResult<()> {
    let dir = safety::expand_home(&backup_dir);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    app.opener()
        .open_path(dir.to_string_lossy(), None::<&str>)
        .map_err(|e| e.to_string())
}

/// Writes a file the user picked in the save dialog (report, diagnostics).
#[tauri::command]
async fn save_text(path: String, contents: String) -> CmdResult<()> {
    blocking(move || {
        let ext = analiz_core::walk::ext(std::path::Path::new(&path));
        if !matches!(ext.as_str(), "md" | "txt" | "json") {
            return Err("yalnızca .md, .txt veya .json kaydedilebilir".into());
        }
        std::fs::write(path, contents).map_err(|e| e.to_string())
    })
    .await
}

/// Language picked in the setup wizard, used on first launch before the user
/// has saved any settings. `None` on macOS/Linux or for portable runs.
#[tauri::command]
fn installer_language() -> Option<String> {
    installer::language()
}

pub fn run() -> tauri::Result<()> {
    tauri::Builder::default()
        // must come first: a second launch just brings the running window back
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            tray::show_main(app)
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| Ok(tray::setup(app.handle())?))
        .on_window_event(tray::on_window_event)
        .invoke_handler(tauri::generate_handler![
            scan_server,
            is_running,
            preview_fix,
            apply_fix,
            list_backups,
            restore_backup,
            cleanup_backups,
            open_backup_dir,
            save_text,
            installer_language,
            tray::set_close_to_tray,
            tray::set_tray_labels
        ])
        .run(tauri::generate_context!())
}
