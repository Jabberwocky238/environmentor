mod utils;
mod walk;
mod persist;

use persist::Persist;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[tokio::test]
async fn test_scan() {
    let mut s1 = Storage::load("output.csv");
    s1.update().await;
    s1.dump("output.csv");
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Storage {
    path_map: HashMap<String, NodeRecord>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct NodeRecord {
    pub size: u64,
    pub last_modified: u64,
    pub script_count: u64,
    pub is_allowed: bool,
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
}

impl Storage {
    pub async fn update(&mut self) {
        println!("[Storage] update start");
        let time1 = utils::now();
        // remove out-dated records
        self._tree_shaking();
        let time2 = utils::now();
        println!("[Storage] tree_shaking: {}s", time2 - time1);
        println!("[Storage] scan_with_cache start");
        // update current storage
        self._scna_with_cache();
        let time3 = utils::now();
        println!("[Storage] scan_with_cache: {}s", time3 - time2);
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
                };
            }
        }
        children
    }

    fn _tree_shaking(&mut self) {
        let mut modified_cnt = 0;
        let mut disappear_cnt = 0;

        let mut descendent_keys = self.path_map.keys().cloned().collect::<Vec<String>>();
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
                let keys_to_remove: Vec<String> = self
                    .path_map
                    .keys()
                    .cloned()
                    .filter(|s| s.starts_with(prefix))
                    .collect();
                disappear_cnt += keys_to_remove.len();
                for key in keys_to_remove {
                    self.path_map.remove(&key);
                }
                continue;
            }
            let p = k.to_str().unwrap();
            let last_modified1 = utils::get_modified(p);
            let last_modified2 = self.path_map.get(p).unwrap().last_modified;

            if last_modified1 != last_modified2 {
                self.path_map.remove(p);
                // 直系child
                let children: Vec<String> = self
                    .path_map
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
            "[Storage] tree_shaking: modified: {}, disappear: {}",
            modified_cnt, disappear_cnt
        );
    }

    fn _scna_with_cache(&mut self) {
        // let s = multi_thread_walk().unwrap();
        let s = walk::single_thread_walk(Some(&self.path_map)).unwrap();
        self.path_map = s.path_map;
    }

    pub fn dump(&self, path: &str) {
        self._dump(path);
    }
    pub fn load(path: &str) -> Self {
        Self::_load(path)
    }
}

