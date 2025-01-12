use crossbeam::queue::SegQueue;
use crossbeam::scope;
use dashmap::DashMap;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex};
use std::fs;

use super::utils::{
    get_drives, get_modified, treat_as_file, treat_as_ignore,
};
use super::NodeRecord;
const THREADS: usize = 4;

#[cfg(test)]
mod test_walk {
    use super::*;
    use crate::scanner::{tree_shaking, Storage};

    #[test]
    fn no_cache_single() {
        let start = vec!["D:\\Scoop".into()]; // 5.39s
        let s = single_thread_walk(start, None).unwrap();
        println!("no_cache_single done with {}", s["D:\\Scoop"].size);
        // 6634756838
    }

    #[test]
    fn no_cache_parallel() {
        let start = vec!["D:\\Scoop".into()]; // 2.14s
                                              // let start = vec!["D:\\".into()]; // 25s
        let s = multi_thread_walk(start, None).unwrap();
        println!("no_cache_parallel done with {}", s["D:\\Scoop"].size);
        // 6634756838
    }

    #[test]
    fn with_cache_single() {
        let cache_s = Storage::load("output.csv");
        let mut cache = cache_s.path_map;
        tree_shaking(&mut cache);

        let start = vec!["D:\\Scoop".into()]; // 3.25s
        let s = single_thread_walk(start, Some(&cache)).unwrap();
        println!("no_cache_single done with {}", s["D:\\Scoop"].size);
        // 10930490222 wrong
    }

    #[test]
    fn with_cache_parallel() {
        let cache_s = Storage::load("output.csv");
        let mut cache = cache_s.path_map;
        tree_shaking(&mut cache);

        let start = vec!["D:\\Scoop".into()]; // 4.66s
        let s = multi_thread_walk(start, Some(&cache)).unwrap();
        println!("no_cache_parallel done with {}", s["D:\\Scoop"].size);
        // 6634756838
    }
}

pub fn walk_scan(
    cache: Option<&HashMap<String, NodeRecord>>,
) -> Result<HashMap<String, NodeRecord>, Box<dyn std::error::Error>> {
    let drives = get_drives();
    let s = single_thread_walk(drives, cache).unwrap();
    // let s = multi_thread_walk(cache).unwrap();
    Ok(s)
}
// ==================== single thread ====================
#[derive(Debug, Default)]
struct _Storage {
    _map: HashMap<String, NodeRecord>,
}

impl _Storage {
    fn consume(self) -> HashMap<String, NodeRecord> {
        self._map
    }
    pub fn merge(&mut self, cache: &HashMap<String, NodeRecord>) {
        for (k, v) in cache {
            self._map.insert(k.to_owned(), v.clone());
        }
    }
    pub fn accumulate(&mut self, path: &PathBuf, adder: NodeRecord) {
        // 先尝试添加自己
        let path_string = path.to_str().unwrap().to_string();
        self._map
            .entry(path_string)
            .and_modify(|e| {
                *e += adder.clone();
            })
            .or_insert(adder.clone());

        // recursive call parent to accumulate the size
        let parent = match path.parent() {
            Some(p) => p,
            None => return, // root has no parent
        };
        self.accumulate(&parent.to_path_buf(), adder);

        #[cfg(debug_assertions)]
        {
            if self._map.len() % 10000 == 0 {
                println!("accumulate: {} records", self._map.len());
            }
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
fn single_thread_walk(
    start_points: Vec<PathBuf>,
    cache: Option<&HashMap<String, NodeRecord>>,
) -> Result<HashMap<String, NodeRecord>, Box<dyn std::error::Error>> {
    let readonly_cache = match cache {
        Some(c) => c,
        None => &HashMap::new(),
    };
    let mut storage = _Storage::default();
    storage.merge(readonly_cache);
    let mut stack: Vec<PathBuf> = start_points;

    while let Some(__path) = stack.pop() {
        // 如果是缓存的目录，则直接累加
        if let Some(v) = readonly_cache.get(__path.to_str().unwrap()) {
            storage.accumulate(&__path, v.clone());
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
                if let Some(v) = readonly_cache.get(entry_pathbuf.to_str().unwrap()) {
                    storage.accumulate(&entry_pathbuf, v.clone());
                    continue;
                }
                if !treat_as_file(&entry_pathbuf) && entry_pathbuf.is_dir() {
                    stack.push(entry_pathbuf.clone());
                    continue;
                }
                // 如果是目录，则递归遍历
                let adder = NodeRecord::from_path(&entry_pathbuf);
                storage.accumulate(&entry_pathbuf, adder);
            }
        } else {
            println!("Failed to open directory: {:?}", &__path);
            storage.not_allowed(&__path);
        }
    }
    Ok(storage.consume())
}

// ==================== multiple threads ====================

// 45.07s
// type RowLockStorage = RwLock<HashMap<String, NodeRecord>>;
// 48.10s
type RowLockStorage = DashMap<String, NodeRecord>;

struct _RowLockStorageHelper;

impl _RowLockStorageHelper {
    pub fn accumulate(map: Arc<RowLockStorage>, path: &PathBuf, adder: NodeRecord) {
        // #[cfg(debug_assertions)]
        // {
        //     let len = map.len();
        //     if len % 1000 == 0 {
        //         println!("accumulate: {} records", len);
        //     }
        // }
        let path_string = path.to_str().unwrap().to_string();

        map.entry(path_string)
            .and_modify(|e| {
                *e += adder.clone();
            })
            .or_insert(adder.clone());
        // println!("accumulate end: {:?}", path);
        // recursive call parent to accumulate the size
        let parent = match path.parent() {
            Some(p) => p,
            None => return, // root has no parent
        };
        _RowLockStorageHelper::accumulate(map, &parent.to_path_buf(), adder);
    }
    pub fn load_cache(map: &mut RowLockStorage, cache: &HashMap<String, NodeRecord>) {
        // let mut guard = map.write().unwrap();
        let guard = map;
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
        map.entry(path_string.to_owned())
            .and_modify(|e| {
                e.size = 0;
                e.last_modified = last_modified;
                e.script_count = 0;
                e.is_allowed = false;
            })
            .or_insert(NodeRecord::with(0, last_modified, 0, false));
    }
    pub fn turn(map: RowLockStorage) -> HashMap<String, NodeRecord> {
        // let _map = map.into_inner().unwrap();
        map.into_iter().collect()
    }
}

fn multi_thread_walk(
    start_points: Vec<PathBuf>,
    cache: Option<&HashMap<String, NodeRecord>>,
) -> Result<HashMap<String, NodeRecord>, Box<dyn std::error::Error>> {
    let cache = match cache {
        Some(c) => c,
        None => &HashMap::new(),
    };
    let cache = cache.clone();
    let start_points: Vec<PathBuf> = start_points;
    let q_walk = SegQueue::new();
    for root in start_points {
        q_walk.push(root);
    }

    let mut row_lock_storage = RowLockStorage::default();
    _RowLockStorageHelper::load_cache(&mut row_lock_storage, &cache);
    let row_lock_storage = Arc::new(row_lock_storage);

    // readonly cache is used for searching, it will not be modified
    // if we use row_lock_storage as cache directly, we either need to clone the item or cause deadlock.
    // to avoid scattering additional clone, readonly_cache takes place.
    let readonly_cache = Arc::new(cache);
    let stop = Arc::new(Mutex::new(vec![false; THREADS]));
    let q_walk = Arc::new((Mutex::new(q_walk), Condvar::new()));

    scope(|scope| {
        for t_index in 0..THREADS {
            let _row_lock_storage: Arc<RowLockStorage> = Arc::clone(&row_lock_storage);
            let _readonly_cache = Arc::clone(&readonly_cache);
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
                    if let Some(v) = _readonly_cache.get(pstr) {
                        // println!("Thread {} cache hitted: {:?}", t_index, pstr);
                        _RowLockStorageHelper::accumulate(
                            Arc::clone(&_row_lock_storage),
                            &__path,
                            v.clone(),
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
                            if let Some(v) = _readonly_cache.get(pstr) {
                                // println!("Thread {} cache hitted: {:?}", t_index, pstr);
                                _RowLockStorageHelper::accumulate(
                                    Arc::clone(&_row_lock_storage),
                                    &entry_pathbuf,
                                    v.clone(),
                                );
                                continue;
                            }
                            // println!("Thread {} no cache hitted: {:?}", t_index, &entry_pathbuf);
                            // no cache hitted
                            if !treat_as_file(&entry_pathbuf) && entry_pathbuf.is_dir() {
                                let (lock, cvar) = &*_q_walk;
                                lock.lock().unwrap().push(entry_pathbuf.to_owned());
                                continue;
                            }
                            let adder = NodeRecord::from_path(&entry_pathbuf);
                            // println!("Thread {} accumulate: {:?}", t_index, &entry_pathbuf);
                            _RowLockStorageHelper::accumulate(
                                Arc::clone(&_row_lock_storage),
                                &entry_pathbuf,
                                adder,
                            );
                            // println!("Thread {} accumulate end: {:?}", t_index, &entry_pathbuf);
                        }
                    } else {
                        println!("Failed to open directory: {:?}", &__path);
                        _RowLockStorageHelper::not_allowed(Arc::clone(&_row_lock_storage), &__path);
                    }
                    // println!("Thread {} end walk", t_index);
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
