mod core;

use core::{AddValue, AppState, DeleteValue, ModifyValue};
use std::{collections::HashMap, sync::Mutex};
use tauri::{Context, Manager, State, Window};

// #[tauri::command]
// fn greet(ctx: Context, window: Window, state: State<AppState>, name: &str) -> String {
//     format!("Hello, {}! You've been greeted from Rust!", name)
// }

#[tauri::command]
fn load(state: State<'_, Mutex<AppState>>) {
    let output = std::process::Command::new("powershell")
        .arg("[Environment]::GetEnvironmentVariables([EnvironmentVariableTarget]::User) | ConvertTo-Json")
        .output()
        .expect("failed to execute process");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let data: HashMap<String, String> =
        serde_json::from_str(&stdout).expect("Failed to deserialize JSON");

    let mut _state = state.lock().unwrap();
    _state.env.clear();
    _state.env.extend(
        data.into_iter()
            .map(|(k, v)| (k, v.split(";").map(|s| s.to_string()).collect())),
    );
}

#[tauri::command]
fn get_state(state: State<'_, Mutex<AppState>>) -> AppState {
    state.lock().unwrap().clone()
}

#[tauri::command]
fn add_task(
    state: State<'_, Mutex<AppState>>,
    task_type: &str,
    variable: &str,
    value1: Option<&str>,
    value2: Option<&str>,
) -> String {
    let mut _state = state.lock().unwrap();
    match task_type {
        "addvalue" => {
            let value1 = value1.expect("addvalue task require value");
            let task = AddValue::new(variable, value1);
            _state.task_queue.add_task(task.into());
            format!("Task added: addvalue {} {}", variable, value1)
        }
        "modvalue" => {
            let value1 = value1.expect("modvalue task require old value");
            let value2 = value2.expect("modvalue task require new value");
            let task = ModifyValue::new(variable, value1, value2);
            _state.task_queue.add_task(task.into());
            format!("Task added: modvalue {} {} {}", variable, value1, value2)
        }
        "delvalue" => {
            let value1 = value1.expect("delvalue task require value");
            let task = DeleteValue::new(variable, value1);
            _state.task_queue.add_task(task.into());
            format!("Task added: delvalue {} {}", variable, value1)
        }
        _ => format!("Task type {} not supported", task_type),
    }
}

#[tauri::command]
fn flush(ctx: Context, window: Window, state: State<AppState>, name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
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

// #[tauri::command]
// /// in case debugging accidentally set the value to empty
// fn get_one(var: &str) -> Vec<String> {
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
        .invoke_handler(tauri::generate_handler![load, get_state, add_task])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
