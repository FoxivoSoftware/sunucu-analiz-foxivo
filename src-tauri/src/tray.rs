// Tray icon: the app keeps running in the notification area, closing the window
// only hides it (unless the user turned that off), "Quit" really exits.
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Runtime, Window, WindowEvent};

pub struct TrayState<R: Runtime> {
    pub close_to_tray: AtomicBool,
    show: MenuItem<R>,
    quit: MenuItem<R>,
}

pub fn show_main<R: Runtime>(app: &AppHandle<R>) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
    }
}

pub fn setup<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Aç", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Çıkış", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    let mut tray = TrayIconBuilder::with_id("main")
        .tooltip("Sunucu Analiz - Foxivo")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_main(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;

    app.manage(TrayState {
        close_to_tray: AtomicBool::new(true),
        show,
        quit,
    });
    Ok(())
}

pub fn on_window_event<R: Runtime>(window: &Window<R>, event: &WindowEvent) {
    if let WindowEvent::CloseRequested { api, .. } = event {
        let hide = window
            .app_handle()
            .try_state::<TrayState<R>>()
            .is_some_and(|s| s.close_to_tray.load(Ordering::Relaxed));
        if hide && window.label() == "main" {
            api.prevent_close();
            let _ = window.hide();
        }
    }
}

#[tauri::command]
pub fn set_close_to_tray(app: AppHandle, enabled: bool) {
    if let Some(s) = app.try_state::<TrayState<tauri::Wry>>() {
        s.close_to_tray.store(enabled, Ordering::Relaxed);
    }
}

/// Menu texts follow the app language.
#[tauri::command]
pub fn set_tray_labels(app: AppHandle, show: String, quit: String) {
    if let Some(s) = app.try_state::<TrayState<tauri::Wry>>() {
        let _ = s.show.set_text(show);
        let _ = s.quit.set_text(quit);
    }
}
