use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::result::Result;
use std::u8;

use crate::scanner::{Storage, StorageUpdater};
use crate::task::{TaskLog, TaskLogData, TaskManager, TaskResolver};

type EnvHashMap = HashMap<String, Vec<String>>;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SendState {
    env: EnvHashMap,
    dirty: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TreeNode {
    name: String,
    abs_path: String,
    size: u64,
    scripts_count: u64,
    is_dir: bool,
    is_allowed: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Notification {
    color: String,
    timestamp: i64,
    title: Option<String>,
    message: String,
}

impl Notification {
    pub fn new(color: &str, message: &str) -> Self {
        Self {
            color: color.to_string(),
            timestamp: 0,
            title: None,
            message: message.to_string(),
        }
    }
    pub fn success(message: &str) -> Self {
        Self::new("success", message)
    }
    pub fn error(message: &str) -> Self {
        Self::new("error", message)
    }
    pub fn warning(message: &str) -> Self {
        Self::new("warning", message)
    }
    pub fn info(message: &str) -> Self {
        Self::new("info", message)
    }
}

pub trait AppTaskAction {
    fn flush(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    fn send_state(&self) -> SendState;
    fn receive_state(&mut self, task: TaskLogData) -> ();
    fn undo(&mut self) -> Notification;
}

pub trait AppFSTAction {
    fn children(&self, abs_path: Option<&str>) -> Vec<TreeNode>;
    fn state(&mut self, option: Option<bool>) -> (bool, Notification);
    fn generater(&self) -> StorageUpdater;
    fn replace(&mut self, s: Storage); 
}


#[derive(Debug, Serialize, Deserialize)]
pub struct AppState {
    tm: TaskManager,
    s: Storage,
}

impl AppState {
    pub fn new() -> Self {
        let mut tm = TaskManager::default();
        tm.init().unwrap();

        Self { tm, s: Storage::load("output8.csv") }
    } 
}

impl AppTaskAction for AppState {
    fn flush(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.tm.flush()?;
        Ok(())
    }

    fn send_state(&self) -> SendState {
        let env = self.tm.get_cur_env();
        let dirty = self.tm.is_dirty();
        SendState { env, dirty }
    }

    fn receive_state(&mut self, task: TaskLogData) -> () {
        self.tm.add_task(task.into());
    }

    fn undo(&mut self) -> Notification {
        let notification = match self.tm.try_undo() {
            Ok(msg) => Notification::success(&msg),
            Err(msg) => Notification::warning(&msg),
        };
        notification
    }
}


impl AppFSTAction for AppState {
    fn children(&self, abs_path: Option<&str>) -> Vec<TreeNode> {
        let result = self.s.children(abs_path);
        result.into_iter().map(|(abspath, node)| {
            let abs_path = abspath.to_str().unwrap().to_string();
            TreeNode {
                name: if let Some(name) = abspath.file_name() {
                    name.to_str().unwrap().to_string()
                } else {
                    abs_path.clone()
                },
                abs_path,
                size: node.size,
                scripts_count: node.script_count,
                is_dir: abspath.is_dir(),
                is_allowed: node.is_allowed,
            }
        }).collect()
    }

    fn state(&mut self, option: Option<bool>) -> (bool, Notification) {
        if let Some(option) = option {
            self.s.updating = option;
            if option {
                return (option, Notification::info("Scanning started"));
            } else {
                return (option, Notification::success("Scanning complete"));
            }
        }
        let notification = if self.s.updating {
            Notification::warning("Scanning in progress, do not interrupt")
        } else {
            Notification::info("No Scanning planned")
        };
        (self.s.updating, notification)
    }

    fn generater(&self) -> StorageUpdater {
        self.s.clone().into()
    }

    fn replace(&mut self, s: Storage) {
        s.dump("output.csv");
        self.s.replace(s);
    }
}