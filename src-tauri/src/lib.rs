mod core;

use core::AppState;
use core::EnvHashMap;
use std::sync::Mutex;
use tauri::{Context, Manager, State, Window};

// #[tauri::command]
// fn greet(ctx: Context, window: Window, state: State<AppState>, name: &str) -> String {
//     format!("Hello, {}! You've been greeted from Rust!", name)
// }

#[tauri::command]
async fn load(state: State<'_, Mutex<AppState>>) -> tauri::Result<EnvHashMap> {
    let result = state.lock().unwrap().init().unwrap();
    Ok(result)
}

#[tauri::command]
async fn flush(state: State<'_, Mutex<AppState>>) -> tauri::Result<EnvHashMap> {
    dbg!("flushing...");
    state.lock().unwrap().flush().unwrap();
    let result = state.lock().unwrap().reload().unwrap();
    dbg!("flush END");
    Ok(result)
}

#[tauri::command]
fn get_old_state(state: State<'_, Mutex<AppState>>) -> EnvHashMap {
    let state_guard = state.lock().unwrap();
    state_guard.take_snapshot(false).clone()
}

#[tauri::command]
fn get_new_state(state: State<'_, Mutex<AppState>>) -> EnvHashMap {
    let state_guard = state.lock().unwrap();
    state_guard.take_snapshot(true).clone()
}

#[tauri::command]
fn sync_state(state: State<'_, Mutex<AppState>>, variable: &str, values: Option<Vec<String>>) -> tauri::Result<()> {
    let mut state_guard = state.lock().unwrap();
    if let Some(values) = values {
        state_guard.new_env.insert(variable.to_string(), values);
    } else {
        state_guard.new_env.remove(variable);
    }
    dbg!(variable, &state_guard.new_env[variable]);
    Ok(())
}



// [Environment]::SetEnvironmentVariable($Name, $Value, [EnvironmentVariableTarget]::User)
// #[tauri::command]
// fn set_one(var: &str, value: &str) -> Vec<String> {
//     // cmd
//     let output = std::process::Command::new("powershell")
//         .arg(format!("[Environment]::SetEnvironmentVariable(\"{}\", \"{}\", [EnvironmentVariableTarget]::User)", var, value))
//         .output()
//         .expect("failed to execute process");
//     let output = std::process::Command::new("powershell")
//         .arg(format!(
//             "[Environment]::GetEnvironmentVariables(\"{}\", [EnvironmentVariableTarget]::User)",
//             var
//         ))
//         .output()
//         .expect("failed to execute process");
//     let stdout = String::from_utf8(output.stdout).unwrap();
//     stdout
//         .trim_matches('"')
//         .split(";")
//         .filter(|s| !s.is_empty())
//         .map(|s| s.to_string())
//         .collect::<Vec<String>>()
// }

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(Mutex::new(AppState::default()));
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![load, flush, get_new_state, get_old_state, sync_state])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
