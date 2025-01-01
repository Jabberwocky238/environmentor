mod app;
mod scanner;
mod task;

use app::AppAction;
use app::AppState;
use app::SendState;
use app::TreeNode;

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use task::TaskLogData;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::Emitter;
use tauri::{AppHandle, Context, Manager, State, Window};
// #[tauri::command]
// fn greet(ctx: Context, window: Window, state: State<AppState>, name: &str) -> String {
//     format!("Hello, {}! You've been greeted from Rust!", name)
// }

#[tauri::command]
async fn flush(state: State<'_, Mutex<AppState>>) -> tauri::Result<()> {
    dbg!("flushing...");
    state.lock().unwrap().flush().unwrap();
    Ok(())
}

#[tauri::command]
async fn send_state(state: State<'_, Mutex<AppState>>) -> tauri::Result<SendState> {
    dbg!("send_state");
    let send_state = state.lock().unwrap().send_state();
    Ok(send_state)
}

#[tauri::command]
async fn receive_state(state: State<'_, Mutex<AppState>>, task: TaskLogData) -> tauri::Result<()> {
    dbg!(&task);
    state.lock().unwrap().receive_state(task);
    Ok(())
}

#[tauri::command]
async fn undo(app_handle: AppHandle, state: State<'_, Mutex<AppState>>) -> tauri::Result<()> {
    dbg!("undo");
    let notification = state.lock().unwrap().undo();
    app_handle
        .emit("notification", notification)
        .expect("failed to emit notification");
    Ok(())
}

#[tauri::command]
async fn FST_get_children(state: State<'_, Mutex<AppState>>, abs_path: Option<&str>) -> tauri::Result<Vec<TreeNode>> {
    dbg!("FST_get_children");
    let result = state.lock().unwrap().FST_get(abs_path);
    Ok(result)
}

#[tauri::command]
async fn FST_scan(state: State<'_, Mutex<AppState>>) -> tauri::Result<()> {
    dbg!("FST_scan");
    state.lock().unwrap().FST_scan();
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            app.manage(Mutex::new(AppState::new()));

            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_i])?;
            let tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        println!("quit menu item was clicked");
                        app.exit(0);
                    }
                    _ => {
                        println!("menu item {:?} not handled", event.id);
                    }
                })
                .build(app)?;
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            flush,
            send_state,
            receive_state,
            undo,
            FST_get_children,
            FST_scan
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
