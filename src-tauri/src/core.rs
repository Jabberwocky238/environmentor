use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::os::windows::process::CommandExt as _;
use std::process::Command;
use std::u8;

// let decoded_bytes = decode_config(&base64_string, STANDARD)?;

pub type EnvHashMap = HashMap<String, Vec<String>>;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct AppState {
    pub old_env: EnvHashMap,
    pub new_env: EnvHashMap,
}

impl AppState {
    pub fn init(&mut self) -> Result<EnvHashMap, Box<dyn std::error::Error>> {
        let data = get_environment_variables();
        self.old_env.extend(data.clone());
        self.new_env.extend(data.clone());
        Ok(self._encode_base64(true))
    }
    pub fn reload(&mut self) -> Result<EnvHashMap, Box<dyn std::error::Error>> {
        let data = get_environment_variables();
        self.old_env.clear();
        self.old_env.extend(data);
        Ok(self._encode_base64(false))
    }
    pub fn take_snapshot(&self, new: bool) -> EnvHashMap {
        self._encode_base64(new)
    }

    pub fn flush(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let resolver = UpdateResolver::new(self.old_env.clone(), self.new_env.clone());
        resolver.resolve();
        Ok(())
    }

    fn _encode_base64(&self, new: bool) -> EnvHashMap {
        let map = if new {
            self.new_env.clone()
        } else {
            self.old_env.clone()
        };
        map
    }
}

struct UpdateResolver {
    old_env: EnvHashMap,
    new_env: EnvHashMap,
}

impl UpdateResolver {
    pub fn new(old_env: EnvHashMap, new_env: EnvHashMap) -> Self {
        Self { old_env, new_env }
    }
    pub fn resolve(&self) {
        self._resolve();
    }
    fn _resolve(&self) {
        let tasks = self._create_tasks();
        Command::new("powershell")
            .arg(tasks.join(";"))
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .output()
            .expect("failed to execute process");
    }

    fn _filter(&self) -> (EnvHashMap, Vec<String>) {
        let mut updates = EnvHashMap::new();
        let mut deletes = vec![];

        for (k, v) in self.new_env.iter() {
            if self.old_env.get(k) != Some(v) {
                updates.insert(k.clone(), v.clone());
            }
        }
        for (k, _) in self.old_env.iter() {
            if self.new_env.get(k).is_none() {
                deletes.push(k.clone());
            }
        }
        (updates, deletes)
    }

    fn _create_tasks(&self) -> Vec<String> {
        let (updates, deletes) = self._filter();
        let mut tasks = vec![];
        for (k, v) in updates.iter() {
            println!("update '{}': '{:?}'", k, v);
            tasks.push(format!(
                "[Environment]::SetEnvironmentVariable(\"{}\", \"{}\", [EnvironmentVariableTarget]::User)",
                k,
                v.join(";")
            ));
        }
        for k in deletes.iter() {
            println!("delete '{}'", k);
            tasks.push(format!(
                "[Environment]::SetEnvironmentVariable(\"{}\", $null, [EnvironmentVariableTarget]::User)",
                k
            ));
        }
        tasks
    }
}

// forces powershell to output UTF-8, or else it will output UTF-16, stdout will be garbled
const FORCE_UTF8: &str = r#"[console]::OutputEncoding = [System.Text.Encoding]::UTF8"#;
const GET_ENV: &str =
    r#"[Environment]::GetEnvironmentVariables([EnvironmentVariableTarget]::User) | ConvertTo-Json"#;

fn get_environment_variables() -> EnvHashMap {
    let output = Command::new("powershell")
        .arg([FORCE_UTF8, GET_ENV].join(";"))
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
        .expect("failed to execute process");
    let stdout = String::from_utf8(output.stdout).expect("Failed to decode UTF-8");

    let data: HashMap<String, String> =
        serde_json::from_str(&stdout).expect("Failed to deserialize JSON");

    let mut env = HashMap::new();
    env.extend(data.into_iter().map(|(k, v)| {
        let v = v
            .split(";")
            .map(|s| s.to_string())
            .filter(|x| if x.trim().is_empty() { false } else { true })
            .collect();
        return (k, v);
    }));
    env
}
