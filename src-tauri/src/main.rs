// hides the extra console window on Windows release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Err(err) = app_lib::run() {
        eprintln!("uygulama başlatılamadı: {err}");
        std::process::exit(1);
    }
}
