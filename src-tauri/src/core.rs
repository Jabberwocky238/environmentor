use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
struct AddValueTask {
    variable: String,
    value: String,
}

impl AddValueTask {
    pub fn new(variable: &str, value: &str) -> Self {
        AddValueTask {
            variable: variable.to_string(),
            value: value.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
struct ModifyValueTask {
    variable: String,
    old_value: String,
    new_value: String,
}

impl ModifyValueTask {
    pub fn new(variable: &str, old_value: &str, new_value: &str) -> Self {
        ModifyValueTask {
            variable: variable.to_string(),
            old_value: old_value.to_string(),
            new_value: new_value.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
struct DeleteValueTask {
    variable: String,
    value: String,
}

impl DeleteValueTask {
    pub fn new(variable: &str, value: &str) -> Self {
        DeleteValueTask {
            variable: variable.to_string(),
            value: value.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
enum Task {
    AddValue(AddValueTask),
    ModifyValue(ModifyValueTask),
    DeleteValue(DeleteValueTask),
    // AddVariable,
    // ModifyVariable,
    // DeleteVariable,
}


macro_rules! createTask {
    (addvalue, $variable:expr, $value:expr) => {
        Task::AddValue(AddValueTask::new($variable, $value))
    };
    (modvalue, $variable:expr, $old_value:expr, $new_value:expr) => {
        Task::ModifyValue(ModifyValueTask::new($variable, $old_value, $new_value))
    };
    (delvalue, $variable:expr, $value:expr) => {
        Task::DeleteValue(DeleteValueTask::new($variable, $value))
    };
}

#[derive(Serialize, Deserialize)]
struct TaskQueue {
    tasks: Vec<Task>,
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
