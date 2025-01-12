mod persist;
mod utils;
mod walk;

use persist::Persist;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::ops::AddAssign;
use std::path::PathBuf;

#[tokio::test]
async fn test_scan() {
    let s1 = Storage::load("debug.csv");
    let s2 = StorageUpdater::from(s1);
    let s1: Storage = s2.consume();
    s1.dump("debug.csv");
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct NodeRecord {
    pub size: u64,
    pub last_modified: u64,
    pub script_count: u64,
    pub is_allowed: bool,
}

impl AddAssign for NodeRecord {
    fn add_assign(&mut self, other: Self) {
        self.size += other.size;
        self.script_count += other.script_count;
    }
}

impl NodeRecord {
    pub fn with(size: u64, last_modified: u64, script_count: u64, is_allowed: bool) -> Self {
        Self {
            size,
            last_modified,
            script_count,
            is_allowed,
        }
    }
    pub fn from_path(p: &PathBuf) -> Self {
        let _modified = utils::get_modified(p.to_str().unwrap());
        let _size = if utils::treat_as_file(&p) {
            utils::pure_walk(&p).unwrap()
        } else {
            fs::metadata(&p).unwrap().len()
        };
        let _script = if utils::treat_as_script(&p) { 1 } else { 0 };
        Self::with(_size, _modified, _script, true)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Storage {
    path_map: HashMap<String, NodeRecord>,
    pub updating: bool,
}

impl Storage {
    pub fn dump(&self, path: &str) {
        self._dump(path);
    }
    pub fn load(path: &str) -> Self {
        Self::_load(path)
    }
    /// return (abs_path, node_info, is_allowed)
    pub fn children(&self, abs_path: Option<&str>) -> Vec<(PathBuf, NodeRecord)> {
        if let None = abs_path {
            return utils::get_drives()
                .iter()
                .map(|d| {
                    let p: &str = d.to_str().unwrap();
                    let record = if let Some(n) = self.path_map.get(p) {
                        n.clone()
                    } else {
                        NodeRecord::default()
                    };
                    (d.to_owned(), record)
                })
                .collect();
        }
        let abs_path = PathBuf::from(abs_path.unwrap());
        let mut children = vec![];
        if let Ok(entries) = fs::read_dir(&abs_path) {
            let entries_iter = entries
                .filter(|e| e.is_ok())
                .map(|e| e.unwrap())
                .filter(|e| !utils::treat_as_ignore(&e.path()))
                .map(|e| e.path());
            for entry_pathbuf in entries_iter {
                if let Some(n) = self.path_map.get(entry_pathbuf.to_str().unwrap()) {
                    children.push((entry_pathbuf, n.clone()))
                } else {
                    children.push((entry_pathbuf, NodeRecord::default()))
                }
            }
        }
        children
    }
    pub fn replace(&mut self, s: Storage) {
        self.path_map = s.path_map;
    }
}

pub struct StorageUpdater {
    pub(crate) path_map: HashMap<String, NodeRecord>,
}

impl From<Storage> for StorageUpdater {
    fn from(s: Storage) -> Self {
        Self {
            path_map: s.path_map,
        }
    }
}

impl Into<Storage> for StorageUpdater {
    fn into(self) -> Storage {
        Storage {
            path_map: self.path_map,
            updating: false,
        }
    }
}

impl StorageUpdater {
    pub fn consume(mut self) -> Storage {
        println!("[StorageUpdater] update start");
        let time1 = utils::now();
        // remove out-dated records
        tree_shaking(&mut self.path_map);
        let time2 = utils::now();
        println!("[StorageUpdater] tree_shaking: {}s", time2 - time1);
        println!("[StorageUpdater] scan_with_cache start");
        // update current storage
        self._scna_with_cache();
        let time3 = utils::now();
        println!("[StorageUpdater] scan_with_cache: {}s", time3 - time2);
        return self.into();
    }

    fn _scna_with_cache(&mut self) {
        self.path_map = walk::walk_scan(Some(&self.path_map)).unwrap();
    }
}

/// pub(crate) for test walk
/// TODO: need a more efficient data structure to search parent
pub(crate) fn tree_shaking(path_map: &mut HashMap<String, NodeRecord>) {
    let mut modified_cnt = 0;
    let mut disappear_cnt = 0;

    let mut descendent_keys = path_map.keys().cloned().collect::<Vec<String>>();
    // sort by length, so that we can remove the shortest path first
    descendent_keys.sort_by(|a, b| b.len().cmp(&a.len()).reverse());

    let mut wait_list: Vec<PathBuf> = vec![];
    // former200
    for i in 0..200 {
        if let Some(k) = descendent_keys.get(i) {
            wait_list.push(PathBuf::from(k));
        }
    }
    while let Some(k) = wait_list.pop() {
        if !k.exists() {
            // everything start with k will be removed
            let prefix = k.to_str().unwrap();
            let keys_to_remove: Vec<String> = path_map
                .keys()
                .cloned()
                .filter(|s| s.starts_with(prefix))
                .collect();
            disappear_cnt += keys_to_remove.len();
            for key in keys_to_remove {
                path_map.remove(&key);
            }
            continue;
        }
        let p = k.to_str().unwrap();
        let last_modified1 = utils::get_modified(p);
        let last_modified2 = path_map.get(p).unwrap().last_modified;

        if last_modified1 != last_modified2 {
            path_map.remove(p);
            // 直系child
            let children: Vec<String> = path_map
                .keys()
                .cloned()
                .filter(|s| {
                    let p = PathBuf::from(s);
                    p.parent().unwrap() == k
                })
                .collect();
            for child in children {
                wait_list.push(child.into());
            }
            modified_cnt += 1;
        }
    }

    println!(
        "[StorageUpdater] tree_shaking: modified: {}, disappear: {}",
        modified_cnt, disappear_cnt
    );
}
