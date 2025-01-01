use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::os::windows::process::CommandExt as _;
use std::process::Command;
use std::time;
use std::u8;

type EnvHashMap = HashMap<String, Vec<String>>;


#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TaskManager {
    cur_env: EnvHashMap,
    tasks: Vec<TaskLog>,
}

impl TaskManager {
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

// ========================
// ========================

#[allow(unused_variables)]
trait ConsumeTask {
    fn forward(&self, map: &mut EnvHashMap) {
        ()
    }
    fn backword(&self, map: &mut EnvHashMap) {
        ()
    }
}

macro_rules! declare_task_log_data {
    ($name:ident, [ $( $item:ident: $ty:ident), * ]) => {
        #[derive(Serialize, Deserialize, Clone, Default, Debug)]
        pub struct $name {
            $( pub $item: $ty, )*
        }
    };
}
declare_task_log_data!(InitLog, []);
declare_task_log_data!(FlushLog, []);
declare_task_log_data!(RevertLog, []);

impl ConsumeTask for InitLog {}
impl ConsumeTask for FlushLog {}
impl ConsumeTask for RevertLog {}

// ========================

declare_task_log_data!(AddValueLog, [ variable: String, value: String ]);
impl ConsumeTask for AddValueLog {
    fn forward(&self, map: &mut EnvHashMap) {
        // 找有没有这个变量，没有直接panic
        if let Some(values) = map.get_mut(&self.variable) {
            values.push(self.value.clone());
        } else {
            panic!("[ConsumeTask AddValueLog forward] variable not found");
        }
    }
    fn backword(&self, map: &mut EnvHashMap) {
        if let Some(values) = map.get_mut(&self.variable) {
            // 看看最后一个是不是它
            let last = values.last().unwrap();
            if last == &self.value {
                values.pop();
                return;
            }
            panic!(
                "[ConsumeTask AddValueLog backword] self.value '{}' != values.last() '{}'",
                self.value, last
            );
        } else {
            panic!("[ConsumeTask AddValueLog backword] variable not found");
        }
    }
}

// ========================

declare_task_log_data!(DeleteValueLog, [ variable: String, index: usize, value: String ]);
impl ConsumeTask for DeleteValueLog {
    fn forward(&self, map: &mut EnvHashMap) {
        if let Some(values) = map.get_mut(&self.variable) {
            values.remove(self.index);
        } else {
            panic!("[ConsumeTask DeleteValueLog forward] variable not found");
        }
    }
    fn backword(&self, map: &mut EnvHashMap) {
        if let Some(values) = map.get_mut(&self.variable) {
            values.insert(self.index, self.value.clone());
        } else {
            panic!("[ConsumeTask DeleteValueLog backword] variable not found");
        }
    }
}

// ========================

declare_task_log_data!(UpdateValueLog, [ variable: String, index: usize, old_value: String, new_value: String ]);
impl ConsumeTask for UpdateValueLog {
    fn forward(&self, map: &mut EnvHashMap) {
        if let Some(values) = map.get_mut(&self.variable) {
            // 查看是否index位置的值和old value相等，没有直接panic
            if values[self.index] == self.old_value {
                values[self.index] = self.new_value.clone();
                return;
            }
            panic!("[ConsumeTask UpdateValueLog forward] self.old_value '{}' != values[self.index] '{}'", self.old_value, values[self.index]);
        }
        panic!("[ConsumeTask UpdateValueLog forward] variable not found");
    }
    fn backword(&self, map: &mut EnvHashMap) {
        if let Some(values) = map.get_mut(&self.variable) {
            if values[self.index] == self.new_value {
                values[self.index] = self.old_value.clone();
                return;
            }
            panic!("[ConsumeTask UpdateValueLog backword] self.new_value '{}' != values[self.index] '{}'", self.new_value, values[self.index]);
        }
        panic!("[ConsumeTask UpdateValueLog backword] variable not found");
    }
}

// ========================

declare_task_log_data!(OrderValueLog, [ variable: String, index_before: usize, index_after: usize, value: String ]);
impl ConsumeTask for OrderValueLog {
    fn forward(&self, map: &mut EnvHashMap) {
        if let Some(values) = map.get_mut(&self.variable) {
            values.swap(self.index_before, self.index_after);
            return;
        }
        panic!("[ConsumeTask OrderValueLog forward] variable not found");
    }
    fn backword(&self, map: &mut EnvHashMap) {
        if let Some(values) = map.get_mut(&self.variable) {
            values.swap(self.index_before, self.index_after);
            return;
        }
        panic!("[ConsumeTask OrderValueLog backword] variable not found");
    }
}

// ========================
declare_task_log_data!(AddVariableLog, [ variable: String ]);
impl ConsumeTask for AddVariableLog {
    fn forward(&self, map: &mut EnvHashMap) {
        // 如果已经存在这个变量，直接panic
        if map.contains_key(&self.variable) {
            panic!(
                "[ConsumeTask AddVariableLog forward] variable '{}' already exists",
                &self.variable
            );
        }
        map.insert(self.variable.clone(), vec![]);
    }
    fn backword(&self, map: &mut EnvHashMap) {
        // 如果不存在这个变量，直接panic
        if !map.contains_key(&self.variable) {
            panic!(
                "[ConsumeTask AddVariableLog backword] variable '{}' not found",
                &self.variable
            );
        }
        map.remove(&self.variable);
    }
}

// ========================
type VecString = Vec<String>;
declare_task_log_data!(DeleteVariableLog, [ variable: String, values: VecString ]);
impl ConsumeTask for DeleteVariableLog {
    fn forward(&self, map: &mut EnvHashMap) {
        // 如果不存在这个变量，直接panic
        if !map.contains_key(&self.variable) {
            panic!(
                "[ConsumeTask DeleteVariableLog forward] variable '{}' not found",
                &self.variable
            );
        }
        map.remove(&self.variable);
    }
    fn backword(&self, map: &mut EnvHashMap) {
        // 如果已经存在这个变量，直接panic
        if map.contains_key(&self.variable) {
            panic!(
                "[ConsumeTask DeleteVariableLog backword] variable '{}' already exists",
                &self.variable
            );
        }
        map.insert(self.variable.clone(), self.values.clone());
    }
}

// ========================
// ========================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TaskLogData {
    Init(InitLog),
    Flush(FlushLog),
    /// dont modify the names of these enum, compatiable with frontend
    AddVariable(AddVariableLog),
    DelVariable(DeleteVariableLog),
    AppendValue(AddValueLog),
    DeleteValue(DeleteValueLog),
    ModifyValue(UpdateValueLog),
    ReorderValue(OrderValueLog),
}

impl ConsumeTask for TaskLogData {
    fn forward(&self, map: &mut EnvHashMap) {
        match self {
            TaskLogData::Init(log) => log.forward(map),
            TaskLogData::Flush(log) => log.forward(map),
            TaskLogData::AddVariable(log) => log.forward(map),
            TaskLogData::DelVariable(log) => log.forward(map),
            TaskLogData::AppendValue(log) => log.forward(map),
            TaskLogData::DeleteValue(log) => log.forward(map),
            TaskLogData::ModifyValue(log) => log.forward(map),
            TaskLogData::ReorderValue(log) => log.forward(map),
        }
    }
    fn backword(&self, map: &mut EnvHashMap) {
        match self {
            TaskLogData::Init(log) => log.backword(map),
            TaskLogData::Flush(log) => log.backword(map),
            TaskLogData::AddVariable(log) => log.backword(map),
            TaskLogData::DelVariable(log) => log.backword(map),
            TaskLogData::AppendValue(log) => log.backword(map),
            TaskLogData::DeleteValue(log) => log.backword(map),
            TaskLogData::ModifyValue(log) => log.backword(map),
            TaskLogData::ReorderValue(log) => log.backword(map),
        }
    }
}

// ========================
// ========================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskLog {
    pub timestamp: u128,
    pub data: TaskLogData,
}

impl TaskLog {
    pub fn init() -> Self {
        TaskLog {
            timestamp: now(),
            data: TaskLogData::Init(InitLog {}),
        }
    }
    pub fn flush() -> Self {
        TaskLog {
            timestamp: now(),
            data: TaskLogData::Flush(FlushLog {}),
        }
    }
}

impl Into<TaskLog> for TaskLogData {
    fn into(self) -> TaskLog {
        TaskLog {
            timestamp: now(),
            data: self,
        }
    }
}

// ========================
// ========================

pub struct TaskResolver<'a> {
    env: &'a EnvHashMap,
    tasks: &'a [TaskLog],
}

impl<'r> TaskResolver<'r> {
    pub fn new(env: &'r EnvHashMap, tasks: &'r [TaskLog]) -> Self {
        Self { env, tasks }
    }

    pub fn forward(&self) -> EnvHashMap {
        let mut map = self.env.clone();
        for task in self.tasks.iter() {
            task.data.forward(&mut map);
        }
        return map;
    }

    pub fn backword(&self) -> EnvHashMap {
        let mut map = self.env.clone();
        for task in self.tasks.iter().rev() {
            task.data.backword(&mut map);
        }
        return map;
    }
}

// ========================
// ========================

fn now() -> u128 {
    time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}
