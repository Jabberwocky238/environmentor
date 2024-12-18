use serde::{Deserialize, Serialize};
use tauri::ipc::IpcResponse;
use std::collections::HashMap;

macro_rules! declare_task {
    ($name:ident, $($param:ident), *) => {
        #[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
        pub struct $name {
            $(
                $param: String,
            )*
        }

        impl $name {
            pub fn new($($param: &str), *) -> Self {
                $name {
                    $(
                        $param: $param.to_string(),
                    )*
                }
            }
        }

        impl Into<Task> for $name {
            fn into(self) -> Task {
                Task::$name(self)
            }
        }
    };
}

declare_task!(AddValue, variable, value);
declare_task!(ModifyValue, variable, old_value, new_value);
declare_task!(DeleteValue, variable, value);

#[derive(Deserialize, PartialEq, Eq, Hash, Clone)]
pub enum Task {
    AddValue(AddValue),
    ModifyValue(ModifyValue),
    DeleteValue(DeleteValue),
    // AddVariable,
    // ModifyVariable,
    // DeleteVariable,
}

impl Serialize for Task {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Task::AddValue(task) => task.serialize(serializer),
            Task::ModifyValue(task) => task.serialize(serializer),
            Task::DeleteValue(task) => task.serialize(serializer),
        }
    }
}


#[derive(Serialize, Deserialize, Clone, Default)]
pub struct TaskQueue {
    tasks: Vec<Task>,
}

impl<TaskQueue> IpcResponse for TaskQueue {
    fn body(self) -> tauri::Result<tauri::ipc::InvokeResponseBody> {
        let json = serde_json::to_value(self).unwrap();
        Ok(tauri::ipc::InvokeResponseBody::Json(json))
    }
}

impl TaskQueue {
    pub fn new() -> Self {
        TaskQueue { tasks: Vec::new() }
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    pub fn optimise(&mut self) {
        // 向前合并
        let mut _tasks = vec![];

        while let Some(wrapped_task) = self.tasks.drain(..).next() {
            match wrapped_task.clone() {
                Task::AddValue(task) => {
                    // 向前寻找是否已经添加过值
                    _tasks.retain(|t| match t {
                        Task::AddValue(__t) => {
                            if __t.variable == task.variable && __t.value == task.value {
                                false
                            } else {
                                true
                            }
                        }
                        _ => true,
                    });
                    _tasks.push(wrapped_task);
                }
                Task::ModifyValue(task) => {
                    // 向前寻找add和modify操作
                    let mut flag = false;
                    _tasks.iter_mut().for_each(|v| match v {
                        Task::AddValue(__t) => {
                            if __t.variable == task.variable && __t.value == task.old_value {
                                __t.value = task.new_value.clone();
                                flag = true;
                            }
                        }
                        Task::ModifyValue(__t) => {
                            if __t.variable == task.variable && __t.new_value == task.old_value {
                                __t.new_value = task.new_value.clone();
                                flag = true;
                            }
                        }
                        _ => {}
                    });

                    if !flag {
                        _tasks.push(wrapped_task);
                    }
                }
                Task::DeleteValue(task) => {
                    // 寻找全部add和modify操作，并消除
                    let mut flag = false;
                    _tasks.retain(|t| match t {
                        Task::AddValue(__t) => {
                            if __t.variable == task.variable && __t.value == task.value {
                                flag = true;
                                false
                            } else {
                                true
                            }
                        }
                        Task::ModifyValue(__t) => {
                            if __t.variable == task.variable && __t.new_value == task.value {
                                flag = true;
                                false
                            } else {
                                true
                            }
                        }
                        _ => true,
                    });

                    if !flag {
                        _tasks.push(wrapped_task);
                    }
                }
            }
        }

        self.tasks = _tasks;
    }

    pub fn execute(&mut self) {
        self.tasks.clear();
    }
}
type EnvHashMap = HashMap<String, Vec<String>>;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct AppState {
    pub old_env: EnvHashMap,
    pub new_env: EnvHashMap,
    pub task_queue: TaskQueue,
}