use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::result::Result;
use std::os::windows::process::CommandExt as _;
use std::process::Command;
use std::u8;

use crate::task::{TaskLog, TaskLogData, TaskResolver};

pub type EnvHashMap = HashMap<String, Vec<String>>;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct AppState {
    cur_env: EnvHashMap,
    tasks: Vec<TaskLog>,
}

impl AppState {
    pub fn init(&mut self) -> Result<EnvHashMap, Box<dyn std::error::Error>> {
        dbg!("init");
        let data = get_environment_variables();
        self.cur_env.extend(data.clone());

        self.add_task(TaskLog::init());

        Ok(data)
    }

    pub fn flush(&mut self) -> Result<EnvHashMap, Box<dyn std::error::Error>> {
        let _tasks = self._since_last_flush_tasks();
        let new_env = TaskResolver::new(&self.cur_env, _tasks).forward();
        let _ = UpdateResolver::new(&self.cur_env, &new_env).resolve();

        self.add_task(TaskLog::flush());

        let data = get_environment_variables();
        self.cur_env.clear();
        self.cur_env.extend(data.clone());
        Ok(self.cur_env.clone())
    }

    pub fn add_task(&mut self, task: TaskLog) {
        self.tasks.push(task);
    }

    pub fn task_list(&self) -> Vec<TaskLog> {
        self.tasks.clone()
    }

    pub fn get_cur_env(&self) -> EnvHashMap {
        let _tasks = self._since_last_flush_tasks();
        let new_env = TaskResolver::new(&self.cur_env, _tasks).forward();
        new_env
    }

    pub fn is_dirty(&self) -> bool {
        let _tasks = self._since_last_flush_tasks();
        // dbg!(_tasks.len());
        if _tasks.len() > 0 {
            return true;
        } else {
            return false;
        }
    }

    pub fn try_undo(&mut self) -> Result<String, &str> {
        let _tasks = self._since_last_flush_tasks();
        let _task = match _tasks.last() {
            Some(_) => self.tasks.pop().unwrap(),
            None => return Err("[illigal calling] No task to undo"),
        };
        return match _task.data {
            TaskLogData::Flush(_) => Err("[illigal calling] Cannot undo TaskLog::Flush"),
            TaskLogData::Init(_) => Err("[illigal calling] Cannot undo TaskLog::Init"),
            TaskLogData::AddVariable(log) => {
                Ok(format!("Undo Task: 重新删除 '{}'", log.variable))
            },
            TaskLogData::DelVariable(log) => {
                Ok(format!("Undo Task: 恢复 '{}'", log.variable))
            },
            TaskLogData::AppendValue(log) => {
                Ok(format!("Undo Task: 重新删除变量 '{}' 中的 '{}", log.variable, log.value))
            },
            TaskLogData::DeleteValue(log) => {
                Ok(format!("Undo Task: 恢复变量 '{}' 中的 '{}'", log.variable, log.value))
            },
            TaskLogData::ModifyValue(log) => {
                Ok(format!("Undo Task: 恢复值 '{}' 为 '{}'", log.new_value, log.old_value))
            },
            TaskLogData::ReorderValue(log) => {
                Ok(format!("Undo Task: 恢复变量 '{}' 的排序", log.variable))
            },
        }
    }

    fn _since_last_flush_tasks(&self) -> &[TaskLog] {
        // 调用函数时，末尾处不应有flush任务
        let last_index = self.tasks.len();
        let mut first_index = last_index;
        for (index, task) in self.tasks.iter().rev().enumerate() {
            let index = last_index - index - 1;
            match task.data {
                TaskLogData::Flush(_) | TaskLogData::Init(_) => break,
                _ => first_index = index,
            }
        }
        // dbg!(first_index, last_index);
        &self.tasks[first_index..last_index]
    }
}

// 处理环境变量更新操作
struct UpdateResolver<'a> {
    old_env: &'a EnvHashMap,
    new_env: &'a EnvHashMap,
}

impl<'a> UpdateResolver<'a> {
    pub fn new(old_env: &'a EnvHashMap, new_env: &'a EnvHashMap) -> Self {
        Self { old_env, new_env }
    }
    pub fn resolve(&self) {
        self._resolve();
    }
    fn _resolve(&self) {
        let (updates, deletes) = self._filter();
        let tasks = self._create_tasks(updates, deletes);
        Command::new("powershell")
            .arg(tasks.join(";"))
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .output()
            .expect("failed to execute process");
    }

    // 过滤出需要更新和删除的环境变量
    fn _filter(&self) -> (Vec<&String>, Vec<&String>) {
        let mut updates = vec![];
        let mut deletes = vec![];

        for (k, v) in self.new_env.iter() {
            if self.old_env.get(k) != Some(v) {
                updates.push(k);
            }
        }
        for (k, _) in self.old_env.iter() {
            if self.new_env.get(k).is_none() {
                deletes.push(k);
            }
        }
        (updates, deletes)
    }

    // 生成更新环境变量的任务字符串
    fn _create_tasks(&self, updates: Vec<&String>, deletes: Vec<&String>) -> Vec<String> {
        let mut tasks = vec![];
        for k in updates.iter() {
            let v = self.new_env.get(*k).unwrap();
            println!("update '{}': '{:?}'", k, v);
            // extra ; is needed to avoid empty string, or it will be removed
            tasks.push(format!(
                "[Environment]::SetEnvironmentVariable(\"{}\", \";{}\", [EnvironmentVariableTarget]::User)",
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

// forces powershell to output UTF-8, or else it will output UTF-16, stdout cannot be decoded
const FORCE_UTF8: &str = r#"[console]::OutputEncoding = [System.Text.Encoding]::UTF8"#;
const GET_ENV: &str =
    "[Environment]::GetEnvironmentVariables([EnvironmentVariableTarget]::User) | ConvertTo-Json";

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
