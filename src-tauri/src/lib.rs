use std::collections::HashMap;
mod core;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_all() -> HashMap<String, Vec<String>> {
    // cmd
    let output = std::process::Command::new("powershell")
        .arg("[Environment]::GetEnvironmentVariables([EnvironmentVariableTarget]::User) | ConvertTo-Json")
        .output()
        .expect("failed to execute process");
    let stdout = String::from_utf8(output.stdout).unwrap();

    let data: HashMap<String, String> =
        serde_json::from_str(&stdout).expect("Failed to deserialize JSON");
    let data = data
        .into_iter()
        .map(|(k, v)| {
            let v = v
                .trim_matches('"')
                .split(";")
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect::<Vec<String>>();
            (k, v)
        })
        .collect();
    data
}

// [Environment]::SetEnvironmentVariable($Name, $Value, [EnvironmentVariableTarget]::User)
#[tauri::command]
fn set_one(var: &str, value: &str) -> Vec<String> {
    // cmd
    let output = std::process::Command::new("powershell")
        .arg(format!("[Environment]::SetEnvironmentVariable(\"{}\", \"{}\", [EnvironmentVariableTarget]::User)", var, value))
        .output()
        .expect("failed to execute process");
    let output = std::process::Command::new("powershell")
        .arg(format!(
            "[Environment]::GetEnvironmentVariables(\"{}\", [EnvironmentVariableTarget]::User)",
            var
        ))
        .output()
        .expect("failed to execute process");
    let stdout = String::from_utf8(output.stdout).unwrap();
    stdout
        .trim_matches('"')
        .split(";")
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
}

#[tauri::command]
/// in case debugging accidentally set the value to empty
fn get_one(var: &str) -> Vec<String> {
    let output = std::process::Command::new("powershell")
        .arg(format!(
            "[Environment]::GetEnvironmentVariables(\"{}\", [EnvironmentVariableTarget]::User)",
            var
        ))
        .output()
        .expect("failed to execute process");
    let stdout = String::from_utf8(output.stdout).unwrap();
    stdout
        .trim_matches('"')
        .split(";")
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet, get_all, set_one, get_one])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
