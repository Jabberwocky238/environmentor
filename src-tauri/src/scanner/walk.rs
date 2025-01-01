use crossbeam::queue::SegQueue;
use crossbeam::scope;
use dashmap::DashMap;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::time::Duration;
use std::{fs, thread};

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
            updating: false,
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
    storage._map.reserve(100_0000);
    storage.load_cache(cache);
    // let mut stack: Vec<PathBuf> = get_drives();
    let mut stack: Vec<PathBuf> = vec!["D:\\".into()];

    while let Some(__path) = stack.pop() {
        // 如果是缓存的目录，则直接累加
        if let Some(v) = storage._map.get(__path.to_str().unwrap()) {
            storage.accumulate(&__path, v.size, v.script_count);
            continue;
        }
        if let Ok(entries) = fs::read_dir(&__path) {
            let entries_iter = entries
                .filter(|e| e.is_ok())
                .map(|e| e.unwrap())
                .filter(|e| !treat_as_ignore(&e.path()))
                .map(|e| e.path());
            for entry_pathbuf in entries_iter {
                // 如果是缓存的目录，则直接累加
                if let Some(v) = storage._map.get(entry_pathbuf.to_str().unwrap()) {
                    storage.accumulate(&entry_pathbuf, v.size, v.script_count);
                    continue;
                }
                // 如果是目录，则递归遍历
                let _size = if treat_as_file(&entry_pathbuf) {
                    pure_walk(&entry_pathbuf)?
                } else {
                    // common file or directory
                    if entry_pathbuf.is_dir() {
                        stack.push(entry_pathbuf.to_owned());
                    }
                    fs::metadata(&entry_pathbuf)?.len()
                };
                let _script = if treat_as_script(&entry_pathbuf) {
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
type RowLockStorage = RwLock<HashMap<String, NodeRecord>>;
struct _RowLockStorageHelper;

impl _RowLockStorageHelper {
    pub fn accumulate(map: Arc<RowLockStorage>, path: &PathBuf, add_size: u64, add_scripts: u64) {
        // 先尝试添加自己
        // println!("accumulate: {:?}", path);
        let path_string = path.to_str().unwrap().to_string();
        let last_modified = get_modified(&path_string);

        map.write()
            .unwrap()
            .entry(path_string)
            .and_modify(|e| {
                e.size += add_size;
                e.last_modified = last_modified;
                e.script_count += add_scripts;
                e.is_allowed = true;
            })
            .or_insert(NodeRecord::with(add_size, last_modified, add_scripts, true));
        // println!("accumulate end: {:?}", path);
        // recursive call parent to accumulate the size
        let parent = match path.parent() {
            Some(p) => p,
            None => return, // root has no parent
        };
        // println!("accumulate parent: {:?}", parent);
        let len = map.read().unwrap().len();
        _RowLockStorageHelper::accumulate(map, &parent.to_path_buf(), add_size, add_scripts);
        if len % 1000 == 0 {
            println!("accumulate: {} records", len);
        }
    }
    pub fn load_cache(map: &mut RowLockStorage, cache: &HashMap<String, NodeRecord>) {
        let mut guard = map.write().unwrap();
        for (k, v) in cache {
            guard.insert(k.to_owned(), v.clone());
        }
        println!(
            "[_RowLockStorageHelper] load_cache: {} records",
            guard.len()
        );
    }
    pub fn not_allowed(map: Arc<RowLockStorage>, path: &PathBuf) {
        let path_string = path.to_str().unwrap().to_string();
        let last_modified = get_modified(&path_string);
        map.write()
            .unwrap()
            .entry(path_string.to_owned())
            .and_modify(|e| {
                e.size = 0;
                e.last_modified = last_modified;
                e.script_count = 0;
                e.is_allowed = false;
            })
            .or_insert(NodeRecord::with(0, last_modified, 0, false));
    }
    pub fn turn(map: RowLockStorage) -> Storage {
        let _map = map.into_inner().unwrap();
        Storage {
            path_map: _map,
            updating: false,
        }
    }
}

pub fn multi_thread_walk(
    cache: Option<&HashMap<String, NodeRecord>>,
) -> Result<Storage, Box<dyn std::error::Error>> {
    let cache = match cache {
        Some(c) => c,
        None => &HashMap::new(),
    };
    let cache = cache.clone();

    const THREADS: usize = 8;
    // let roots: Vec<PathBuf> = get_drives();
    let roots: Vec<PathBuf> = vec!["D:\\".into()];
    let q_walk = SegQueue::new();
    for root in roots {
        q_walk.push(root);
    }

    let mut row_lock_storage = RowLockStorage::default();
    _RowLockStorageHelper::load_cache(&mut row_lock_storage, &cache);
    let row_lock_storage = Arc::new(row_lock_storage);
    let cache = Arc::new(cache);
    let stop = Arc::new(Mutex::new(vec![false; THREADS]));
    let q_walk = Arc::new((Mutex::new(q_walk), Condvar::new()));

    scope(|scope| {
        for t_index in 0..THREADS {
            let _row_lock_storage: Arc<RowLockStorage> = Arc::clone(&row_lock_storage);
            let _cache = Arc::clone(&cache);
            let _stop = Arc::clone(&stop);
            let _q_walk = Arc::clone(&q_walk);
            
            scope.spawn(move |_| {
                loop {
                    let __path = {
                        let (lock, cvar) = &*_q_walk;
                        let mut q_walk = lock.lock().unwrap();
                        while q_walk.is_empty() {
                            let mut stop_flags = _stop.lock().unwrap();
                            stop_flags[t_index] = true;
                            println!("Thread {} stop", t_index);
                            if stop_flags.iter().all(|&x| x) {
                                drop(q_walk);
                                cvar.notify_one();
                                println!("Thread {} shutdown", t_index);
                                return;
                            }
                            drop(stop_flags);
                            q_walk = cvar.wait(q_walk).unwrap();
                            println!("Thread {} start", t_index);
                        }
                        cvar.notify_all();
                        q_walk.pop().unwrap()
                    };
                    // println!("Thread {} start walk", t_index);

                    // 如果是缓存的目录，则直接累加
                    let pstr = __path.to_str().unwrap();
                    // println!("Thread {} walk: {:?}", t_index, pstr);
                    if let Some(v) = _cache.get(pstr) {
                        // println!("Thread {} cache hitted: {:?}", t_index, pstr);
                        _RowLockStorageHelper::accumulate(
                            Arc::clone(&_row_lock_storage),
                            &__path,
                            v.size,
                            v.script_count,
                        );
                        continue;
                    }

                    if let Ok(entries) = fs::read_dir(&__path) {
                        let entries = entries
                            .filter(|e| e.is_ok())
                            .map(|e| e.unwrap())
                            .filter(|e| !treat_as_ignore(&e.path()))
                            .map(|e| e.path());
                        for entry_pathbuf in entries {
                            // println!("Thread {} traverse: {:?}", t_index, &entry_pathbuf);
                            // cache hitted
                            let pstr = entry_pathbuf.to_str().unwrap();
                            if let Some(v) = _cache.get(pstr) {
                                // println!("Thread {} cache hitted: {:?}", t_index, pstr);
                                _RowLockStorageHelper::accumulate(
                                    Arc::clone(&_row_lock_storage),
                                    &entry_pathbuf,
                                    v.size,
                                    v.script_count,
                                );
                                continue;
                            }
                            println!("Thread {} no cache hitted: {:?}", t_index, &entry_pathbuf);
                            // no cache hitted
                            let _size = if treat_as_file(&entry_pathbuf) {
                                pure_walk(&entry_pathbuf).unwrap()
                            } else {
                                // common file or directory
                                if entry_pathbuf.is_dir() {
                                    let (lock, cvar) = &*_q_walk;
                                    lock.lock().unwrap().push(entry_pathbuf.to_owned());
                                    // _q_walk.push(entry_pathbuf.to_owned());
                                }
                                fs::metadata(&entry_pathbuf).unwrap().len()
                            };
                            let _script = if treat_as_script(&entry_pathbuf) {
                                1
                            } else {
                                0
                            };
                            println!("Thread {} accumulate: {:?}", t_index, &entry_pathbuf);
                            _RowLockStorageHelper::accumulate(
                                Arc::clone(&_row_lock_storage),
                                &entry_pathbuf,
                                _size,
                                _script,
                            );
                            println!("Thread {} accumulate end: {:?}", t_index, &entry_pathbuf);
                        }
                    } else {
                        println!("Failed to open directory: {:?}", &__path);
                        _RowLockStorageHelper::not_allowed(Arc::clone(&_row_lock_storage), &__path);
                    }
                    println!("Thread {} end walk", t_index);
                }
            });
        }
    })
    .unwrap();
    // transform the storage

    if let Some(_map) = Arc::try_unwrap(row_lock_storage).ok() {
        let s = _RowLockStorageHelper::turn(_map);
        return Ok(s);
    }
    unreachable!()
}
