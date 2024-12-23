mod core;
mod task;

use core::AppState;
use core::EnvHashMap;
use std::sync::Mutex;
use task::TaskLog;
use task::TaskLogData;
use tauri::{AppHandle, Context, Manager, State, Window};

// #[tauri::command]
// fn greet(ctx: Context, window: Window, state: State<AppState>, name: &str) -> String {
//     format!("Hello, {}! You've been greeted from Rust!", name)
// }

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct SendState {
    env: EnvHashMap,
    dirty: bool,
}

#[tauri::command]
async fn flush(state: State<'_, Mutex<AppState>>) -> tauri::Result<SendState> {
    dbg!("flushing...");
    state.lock().unwrap().flush().unwrap();

    let state_guard = state.lock().unwrap();
    let (env, dirty) = (state_guard.get_cur_env(), state_guard.is_dirty());
    dbg!("flush END");
    let result = SendState { env, dirty };
    Ok(result)
}

#[tauri::command]
fn send_state(state: State<'_, Mutex<AppState>>) -> tauri::Result<SendState> {
    dbg!("send_state");
    let state_guard = state.lock().unwrap();

    let (env, dirty) = (state_guard.get_cur_env(), state_guard.is_dirty());
    let result = SendState { env, dirty };
    Ok(result)
}

#[tauri::command]
fn receive_state(state: State<'_, Mutex<AppState>>, task: TaskLogData) -> tauri::Result<()> {
    dbg!(&task);
    state.lock().unwrap().add_task(task.into());
    Ok(())
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
