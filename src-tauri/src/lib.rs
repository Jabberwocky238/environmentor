mod core;
mod task;
mod scanner;

use core::AppState;
use core::EnvHashMap;
use std::sync::Mutex;
use task::TaskLog;
use task::TaskLogData;
use tauri::Emitter;
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

enum NotificationColor {
    Success,
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Notification {
    color: String,
    timestamp: i64,
    title: Option<String>,
    message: String,
}

impl Notification {
    pub fn new(color: NotificationColor, message: &str) -> Self {
        let color = match color {
            NotificationColor::Success => "success",
            NotificationColor::Error => "error",
            NotificationColor::Warning => "warning",
            NotificationColor::Info => "info",
        };
        Self {
            color: color.to_string(),
            timestamp: 0,
            title: None,
            message: message.to_string(),
        }
    }
}

#[tauri::command]
fn undo(app_handle: AppHandle, state: State<'_, Mutex<AppState>>) -> tauri::Result<SendState> {
    dbg!("undo");
    let mut state_guard = state.lock().unwrap();
    let notification = match state_guard.try_undo() {
        Ok(msg) => Notification::new(NotificationColor::Success, &msg),
        Err(msg) => Notification::new(NotificationColor::Warning, &msg),
    };
    app_handle
        .emit("notification", notification)
        .expect("failed to emit notification");

    let (env, dirty) = (state_guard.get_cur_env(), state_guard.is_dirty());
    let result = SendState { env, dirty };
    Ok(result)
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
            undo
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
