use std::{collections::{HashMap, HashSet}, time};

use serde::{de::value, Deserialize, Serialize};

macro_rules! declare_task_log_data {
    ($name:ident, [ $( $item:ident: $ty:ident, )* ]) => {
        #[derive(Serialize, Deserialize, Clone, Default)]
        pub struct $name {
            $( pub $item: $ty, )*
        }
    };
}

declare_task_log_data!(AddValueLogData, [
    variable: String,
    index: usize,
    value: String,
]);

declare_task_log_data!(DeleteValueLogData, [
    variable: String,
    index: usize,
    value: String,
]);

declare_task_log_data!(UpdateValueLogData, [
    variable: String,
    index: usize,
    old_value: String,
    new_value: String,
]);

declare_task_log_data!(OrderValueLogData, [
    variable: String,
    candidate1: usize,
    candidate2: usize,
]);

declare_task_log_data!(AddVariableLogData, [
    variable: String,
]);

declare_task_log_data!(DeleteVariableLogData, [
    variable: String,
]);

declare_task_log_data!(FlushLogData, []);
declare_task_log_data!(InitLogData, []);
// ========================
#[derive(Serialize, Deserialize, Clone)]
enum TaskLogData {
    Init(InitLogData),
    Flush(FlushLogData),

    AddValue(AddValueLogData),
    DeleteValue(DeleteValueLogData),
    UpdateValue(UpdateValueLogData),
    OrderValue(OrderValueLogData),
    AddVariable(AddVariableLogData),
    DeleteVariable(DeleteVariableLogData),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TaskLog {
    timestamp: u128,
    data: TaskLogData,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Recorder {
    pub tasks: Vec<TaskLog>,
}

impl Recorder {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn add_task(&mut self, task: TaskLog) {
        self.tasks.push(task);
    }

    pub fn load(&mut self, rawtext: &str) {
        let logs: Vec<TaskLog> = serde_json::from_str(rawtext).unwrap();
        self.tasks.extend(logs);
    }
}

pub struct TaskBuilder;
type EnvHashMap = HashMap<String, Vec<String>>;

impl TaskBuilder {
    pub fn make(env: &EnvHashMap, variable: &str, values: Option<Vec<String>>) -> TaskLog {
        // judge if DeleteVariableLogData
        if let None = values {
            return TaskLog {
                timestamp: now(),
                data: TaskLogData::DeleteVariable(DeleteVariableLogData {
                    variable: variable.to_string(),
                }),
            };
        }

        // judge if AddValueLogData
        let values = values.unwrap();
        if values.len() == 0 {
            return TaskLog {
                timestamp: now(),
                data: TaskLogData::AddVariable(AddVariableLogData {
                    variable: variable.to_string(),
                }),
            };
        }

        let what_variable_has = env[variable].clone();
        let (old_len, new_len) = (what_variable_has.len(), values.len());

        if old_len < new_len {
            // judge if AddValueLogData
            return TaskLog {
                timestamp: now(),
                data: TaskLogData::AddValue(AddValueLogData {
                    variable: variable.to_string(),
                    index: old_len,
                    value: values[old_len].clone(),
                }),
            };
        } else if old_len > new_len {
            // judge if DeleteValueLogData
            return TaskLog {
                timestamp: now(),
                data: TaskLogData::DeleteValue(DeleteValueLogData {
                    variable: variable.to_string(),
                    index: old_len - 1,
                    value: what_variable_has[old_len - 1].clone(),
                }),
            };
        } else { 
            // old_len == new_len            
            let mut old_set = HashSet::new();
            let mut new_set = HashSet::new();
            for i in 0..old_len {
                old_set.insert(&what_variable_has[i]);
            }
            for i in 0..new_len {
                new_set.insert(&values[i]);
            }
            if old_set == new_set {
                // judge if OrderValueLogData
                let mut diff_indice = vec![];
                for i in 0..old_len {
                    if what_variable_has[i] != values[i] {
                        diff_indice.push(i);
                    }
                }
                if diff_indice.len() == 0 {
                    panic!("unknown task, judge if OrderValueLogData, diff_indice.len() == 0");
                }
                // we can never know which one is the new one, so I call this action OrderValueLogData as a type of SWITCH
                return TaskLog {
                    timestamp: now(),
                    data: TaskLogData::OrderValue(OrderValueLogData {
                        variable: variable.to_string(),
                        candidate1: diff_indice[0],
                        candidate2: diff_indice[1],
                    }),
                };
            } else {
                // judge if UpdateValueLogData
                for i in 0..old_len {
                    if what_variable_has[i] != values[i] {
                        return TaskLog {
                            timestamp: now(),
                            data: TaskLogData::UpdateValue(UpdateValueLogData {
                                variable: variable.to_string(),
                                index: i,
                                old_value: what_variable_has[i].clone(),
                                new_value: values[i].clone(),
                            }),
                        };
                    }
                }
            }
        }
        unreachable!()
    }
    pub fn action(what: &str) -> TaskLog {
        match what {
            "init" => TaskLog {
                timestamp: now(),
                data: TaskLogData::Init(InitLogData {}),
            },
            "flush" => TaskLog {
                timestamp: now(),
                data: TaskLogData::Flush(FlushLogData {}),
            },
            _ => panic!("unknown action: {}", what),
        }
    }
}

fn now() -> u128 {
    time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

#[test]
fn test_task_log_batch_load() {
    let rawtext = r#"
    [
        {
            "timestamp": 0,
            "data": {
                "AddValue": {
                    "variable": "test",
                    "index": 0,
                    "value": "a"
                }
            }
        },
        {
            "timestamp": 1,
            "data": {
                "AddValue": {
                    "variable": "test",
                    "index": 1,
                    "value": "b"
                }
            }
        },
        {
            "timestamp": 2,
            "data": {
                "AddValue": {
                    "variable": "test",
                    "index": 2,
                    "value": "c"
                }
            }
        }
    ]"#;
    let logs: Vec<TaskLog> = serde_json::from_str(rawtext).unwrap();
    assert_eq!(logs.len(), 3);
}
