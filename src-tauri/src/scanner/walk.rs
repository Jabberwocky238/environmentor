use crossbeam::queue::SegQueue;
use crossbeam::scope;
use dashmap::DashMap;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use super::utils::{get_modified, pure_walk, treat_as_file, treat_as_ignore, treat_as_script};
use super::{NodeRecord, Storage};

// ==================== single thread ====================
#[derive(Debug, Default)]
struct _Storage {
    _map: HashMap<String, NodeRecord>,
}

impl _Storage {
    fn consume(self) -> Storage {
        println!("consume: {} records", self._map.len());
        Storage {
            path_map: self._map,
        }
    }
    pub fn load_cache(&mut self, cache: &HashMap<String, NodeRecord>) {
        for (k, v) in cache {
            self._map.insert(k.to_owned(), v.clone());
        }
    }
    pub fn accumulate(&mut self, path: &PathBuf, add_size: u64, add_scripts: u64) {
        // 先尝试添加自己
        let path_string = path.to_str().unwrap().to_string();
        let last_modified = get_modified(&path_string);
        self._map
            .entry(path_string)
            .and_modify(|e| {
                e.size += add_size;
                e.last_modified = last_modified;
                e.script_count += add_scripts;
                e.is_allowed = true;
            })
            .or_insert(NodeRecord::with(add_size, last_modified, add_scripts, true));

        // recursive call parent to accumulate the size
        let parent = match path.parent() {
            Some(p) => p,
            None => return, // root has no parent
        };
        self.accumulate(&parent.to_path_buf(), add_size, add_scripts);
        if self._map.len() % 1000 == 0 {
            println!("accumulate: {} records", self._map.len());
        }
    }
    pub fn not_allowed(&mut self, path: &PathBuf) {
        let path_string = path.to_str().unwrap().to_string();
        let last_modified = get_modified(&path_string);
        self._map
            .entry(path_string.to_owned())
            .and_modify(|e| {
                e.size = 0;
                e.last_modified = last_modified;
                e.script_count = 0;
                e.is_allowed = false;
            })
            .or_insert(NodeRecord::with(0, last_modified, 0, false));
    }
}

// 递归改循环
pub fn single_thread_walk(
    cache: Option<&HashMap<String, NodeRecord>>,
) -> Result<Storage, Box<dyn std::error::Error>> {
    let cache = match cache {
        Some(c) => c,
        None => &HashMap::new(),
    };
    let mut storage = _Storage::default();
    storage.load_cache(cache);
    // let mut stack: Vec<PathBuf> = get_drives();
    let mut stack: Vec<PathBuf> = vec!["D:\\".into()];

    while let Some(__path) = stack.pop() {
        if let Ok(entries) = fs::read_dir(&__path) {
            let entries_iter = entries
                .filter(|e| e.is_ok())
                .map(|e| e.unwrap())
                .filter(|e| !treat_as_ignore(&e.path()))
                .map(|e| e.path());
            for entry_pathbuf in entries_iter {
                // 如果是缓存的目录，则直接累加
                if let Some(v) = cache.get(&entry_pathbuf.to_str().unwrap().to_string()) {
                    storage.accumulate(&entry_pathbuf, v.size, v.script_count);
                    continue;
                }
                // 如果是目录，则递归遍历
                let _size = if treat_as_file(&entry_pathbuf)? {
                    pure_walk(&entry_pathbuf)?
                } else {
                    // common file or directory
                    if entry_pathbuf.is_dir() {
                        stack.push(entry_pathbuf.to_owned());
                    }
                    fs::metadata(&entry_pathbuf)?.len()
                };
                let _script = if treat_as_script(&entry_pathbuf)? {
                    1
                } else {
                    0
                };
                storage.accumulate(&entry_pathbuf, _size, _script);
            }
        } else {
            println!("Failed to open directory: {:?}", &__path);
            storage.not_allowed(&__path);
        }
    }
    Ok(storage.consume())
}

// ==================== multiple threads ====================
struct _RowLockStorage;
impl _RowLockStorage {
    pub fn accumulate(map: Arc<DashMap<String, u64>>, path: &PathBuf, add: u64) {
        let parent = match path.parent() {
            Some(p) => p,
            None => return, // root has no parent
        };
        let path_string = parent.to_str().unwrap().to_string();
        map.entry(path_string.to_owned())
            .and_modify(|e| *e += add)
            .or_insert_with(|| {
                let metadata = fs::metadata(path_string).unwrap();
                add + metadata.len()
            });
        // recursive call parent to accumulate the size
        _RowLockStorage::accumulate(map, &parent.to_path_buf(), add);
    }
}

pub fn multi_thread_walk(
    cache: Option<&HashMap<String, NodeRecord>>,
) -> Result<HashMap<String, NodeRecord>, Box<dyn std::error::Error>> {
    const THREADS: usize = 8;
    // let roots: Vec<PathBuf> = get_drives();
    let roots: Vec<PathBuf> = vec!["D:\\".into()];

    let row_lock_storage = Arc::new(DashMap::<String, u64>::new());

    let stop = Arc::new(RwLock::new(vec![false; THREADS]));

    let q_walk = SegQueue::new();
    for root in roots {
        q_walk.push(root);
    }
    let q_walk = Arc::new(q_walk);

    scope(|scope| {
        for t_index in 0..THREADS {
            let row_lock_storage = Arc::clone(&row_lock_storage);
            let _q_walk = Arc::clone(&q_walk);
            let stop = Arc::clone(&stop);

            scope.spawn(move |_| {
                loop {
                    let path = match _q_walk.pop() {
                        Some(p) => {
                            if stop.read().unwrap()[t_index] {
                                stop.write().unwrap()[t_index] = false;
                            }
                            p
                        }
                        None => {
                            println!("Thread {} try stop", t_index);
                            // if all decide to stop, then stop
                            stop.write().unwrap()[t_index] = true;
                            let all_stop = stop.read().unwrap().iter().all(|&x| x);
                            if all_stop {
                                return;
                            } else {
                                continue;
                            }
                        }
                    };

                    // println!("Thread {} start walk", t_index);
                    if let Ok(entries) = fs::read_dir(&path) {
                        for entry in entries {
                            match entry {
                                Ok(entry) => {
                                    let path = entry.path();

                                    let _size = if treat_as_ignore(&path) {
                                        continue;
                                    } else if treat_as_file(&path).unwrap() {
                                        pure_walk(&path).unwrap()
                                    } else {
                                        // common file or directory
                                        if path.is_dir() {
                                            _q_walk.push(path.to_owned());
                                        }
                                        fs::metadata(&path).unwrap().len()
                                    };
                                    _RowLockStorage::accumulate(
                                        Arc::clone(&row_lock_storage),
                                        &path,
                                        _size,
                                    );
                                }
                                Err(e) => println!("Failed to read entry: {}", e),
                            }
                        }
                    } else {
                        println!("Failed to open directory: {:?}", &path);
                    }
                }
            });
        }
    })
    .unwrap();
    // transform the storage
    todo!();
    // if let Some(_map) = Arc::try_unwrap(row_lock_storage).ok() {
    //     return Ok(_RowLockStorage::(_map)?);
    // }
    unreachable!()
}

