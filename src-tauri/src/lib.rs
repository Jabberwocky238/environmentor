mod core;
mod record;

use core::AppState;
use core::EnvHashMap;
use record::TaskLog;
use std::sync::Mutex;
use tauri::{Context, Manager, State, Window};

// #[tauri::command]
// fn greet(ctx: Context, window: Window, state: State<AppState>, name: &str) -> String {
//     format!("Hello, {}! You've been greeted from Rust!", name)
// }

#[tauri::command]
async fn flush(
    app_handle: tauri::AppHandle,
    state: State<'_, Mutex<AppState>>,
) -> tauri::Result<EnvHashMap> {
    dbg!("flushing...");
    state.lock().unwrap().flush().unwrap();
    let result = state.lock().unwrap().reload().unwrap();
    dbg!("flush END");
    Ok(result)
}

#[tauri::command]
fn receive_state(
    state: State<'_, Mutex<AppState>>,
    variable: &str,
    values: Option<Vec<String>>,
) -> tauri::Result<()> {
    let mut state_guard = state.lock().unwrap();
    state_guard.sync_state(variable, values);
    dbg!(variable, &state_guard.new_env.get(variable));
    Ok(())
}

#[tauri::command]
fn send_state(state: State<'_, Mutex<AppState>>) -> tauri::Result<EnvHashMap> {
    let state_guard = state.lock().unwrap();
    Ok(state_guard.new_env.clone())
}

#[tauri::command]
fn task_list(state: State<'_, Mutex<AppState>>) -> tauri::Result<Vec<TaskLog>> {
    let state_guard = state.lock().unwrap();
    Ok(state_guard.task_list())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let mut app_state = AppState::default();
            app_state.init()?;
            app.manage(Mutex::new(app_state));
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            flush,
            send_state,
            receive_state,
            task_list
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
