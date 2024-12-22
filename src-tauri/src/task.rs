use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time;
use std::u8;

pub type EnvHashMap = HashMap<String, Vec<String>>;

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
        // DO NOT INSERT AN EMPTY VECTOR, OR IT WON'T BE ADDED
        map.insert(self.variable.clone(), vec![";".to_string()]);
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
declare_task_log_data!(DeleteVariableLog, [variable: String ]);
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
        map.insert(self.variable.clone(), vec![]);
    }
}

// ========================
// ========================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TaskLogData {
    Init(InitLog),
    Flush(FlushLog),

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

#[derive(Serialize, Deserialize, Clone)]
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
