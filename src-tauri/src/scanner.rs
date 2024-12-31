use crossbeam::queue::{ArrayQueue, SegQueue};
use crossbeam::scope;
use dashmap::DashMap;
use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};

#[test]
fn test_scan() {
    let mut s1 = Storage::default();
    s1.deserialize("output.csv");
    s1.update();
    let cache = Some(&s1.path_map);
    // let cache = None;

    // let s = multi_thread_walk(None).unwrap();
    let s = single_thread_walk(cache).unwrap();

    s.serialize("output.csv");
}

#[derive(Debug, Default)]
pub struct Storage {
    path_map: HashMap<String, NodeRecord>,
}

#[derive(Debug, Default)]
struct NodeRecord {
    size: u64,
    last_scan: u64,
    last_modified: u64,
    has_envvar_count: u64,
}
type TyNodeRecord = (String, u64, u64, u64, u64);

impl Storage {
    fn serialize(&self, path: &str) {
        let mut keys = self.path_map.keys().cloned().collect::<Vec<String>>();
        keys.sort();
        let file = fs::File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .unwrap();
        let mut wtr = csv::Writer::from_writer(file);
        for k in keys {
            let v = self.path_map.get(&k).unwrap();
            wtr.write_record(&[
                k,
                v.size.to_string(),
                v.last_scan.to_string(),
                v.last_modified.to_string(),
                v.has_envvar_count.to_string(),
            ])
            .unwrap();
        }
    }
    fn deserialize(&mut self, path: &str) {
        let m: HashMap<String, NodeRecord> = csv::Reader::from_path(path)
            .unwrap()
            .deserialize()
            .map(|r: Result<TyNodeRecord, csv::Error>| {
                let (k, v1, v2, v3, v4) = r.unwrap();
                let v = NodeRecord {
                    size: v1,
                    last_scan: v2,
                    last_modified: v3,
                    has_envvar_count: v4,
                };
                (k, v)
            })
            .collect();
        self.path_map = m;
    }
}

impl Storage {
    pub fn update(&mut self) {
        // remove out-dated records
        self.tree_shaking();
        // update current storage
        self.scna_with_cache();
    }

    fn tree_shaking(&mut self) {
        let keys = self.path_map.keys().cloned().collect::<Vec<String>>();
        let mut modified_cnt = 0;
        let mut disappear_cnt = 0;
        for k in keys {
            if !PathBuf::from(&k).exists() {
                self.path_map.remove(&k);
                disappear_cnt += 1;
                continue;
            }
            let last_modified1 = get_modified(&k);
            let last_modified2 = self.path_map.get(&k).unwrap().last_modified;
            if last_modified1 != last_modified2 {
                self.path_map.remove(&k);
                modified_cnt += 1;
            }
        }
        println!(
            "tree_shaking: modified: {}, disappear: {}",
            modified_cnt, disappear_cnt
        );
    }

    fn scna_with_cache(&mut self) {
        let time1 = now();
        // let s = multi_thread_walk().unwrap();
        let s = single_thread_walk(Some(&self.path_map)).unwrap();
        let time2 = now();
        let diff = time2 - time1;
        println!(
            "scan_with_cache: {}s, {:.3}m",
            diff,
            diff as f64 / 60 as f64
        );
        self.path_map = s.path_map;
    }
}

trait Scan {
    fn consume(self) -> Storage;
}

// ==================== single thread ====================
struct _Storage {
    path_2_size: HashMap<String, u64>,
    path_2_scripts: HashMap<String, u64>,
}

impl Scan for _Storage {
    fn consume(self) -> Storage {
        let mut path_map = HashMap::new();
        path_map.reserve(self.path_2_size.len());
        let n = now();
        for (k, v) in self.path_2_size {
            path_map.insert(
                k.to_owned(),
                NodeRecord {
                    size: v,
                    last_scan: n,
                    last_modified: get_modified(&k),
                    has_envvar_count: *self.path_2_scripts.get(&k).unwrap_or(&0),
                },
            );
        }
        Storage { path_map }
    }
}

impl _Storage {
    pub fn new() -> Self {
        Self {
            path_2_size: HashMap::new(),
            path_2_scripts: HashMap::new(),
        }
    }
    pub fn load_cache(&mut self, cache: &HashMap<String, NodeRecord>) {
        for (k, v) in cache {
            self.path_2_size.insert(k.to_owned(), v.size);
            self.path_2_scripts.insert(k.to_owned(), v.has_envvar_count);
        }
    }
    pub fn accumulate(&mut self, path: &PathBuf, add_size: u64, add_scripts: u64) {
        let parent = match path.parent() {
            Some(p) => p,
            None => return, // root has no parent
        };
        let path_string = parent.to_str().unwrap().to_string();
        self.path_2_size
            .entry(path_string.to_owned())
            .and_modify(|e| *e += add_size)
            .or_insert_with(|| {
                let metadata = fs::metadata(&path_string).unwrap();
                add_size + metadata.len()
            });
        self.path_2_scripts
            .entry(path_string.to_owned())
            .and_modify(|e| *e += add_scripts)
            .or_insert(add_scripts);
        // recursive call parent to accumulate the size
        self.accumulate(&parent.to_path_buf(), add_size, add_scripts);
    }
}

// 递归改循环
fn single_thread_walk(
    cache: Option<&HashMap<String, NodeRecord>>,
) -> Result<Storage, Box<dyn std::error::Error>> {
    let cache = match cache {
        Some(c) => c,
        None => &HashMap::new(),
    };
    let mut storage = _Storage::new();
    storage.load_cache(cache);
    let mut stack: Vec<PathBuf> = vec!["D:\\".into()];

    while let Some(path) = stack.pop() {
        if let Ok(entries) = fs::read_dir(&path) {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        // 如果是缓存的目录，则直接累加
                        if let Some(v) = cache.get(&path.to_str().unwrap().to_string()) {
                            storage.accumulate(&path, v.size, v.has_envvar_count);
                            continue;
                        }
                        // 如果是目录，则递归遍历
                        if path.is_dir() {
                            stack.push(path.to_owned());
                        }

                        let _size = if treat_as_file(&path)? {
                            pure_walk(&path)?
                        } else {
                            fs::metadata(&path)?.len()
                        };
                        let _script = if treat_as_script(&path)? { 1 } else { 0 };
                        storage.accumulate(&path, _size, _script);
                    }
                    Err(e) => println!("Failed to read entry: {}", e),
                }
            }
        } else {
            println!("Failed to open directory: {:?}", &path);
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

fn multi_thread_walk(
    cache: Option<&HashMap<String, NodeRecord>>,
) -> Result<HashMap<String, NodeRecord>, Box<dyn std::error::Error>> {
    const THREADS: usize = 4;
    let root: PathBuf = "D:\\".into();
    let row_lock_storage = Arc::new(DashMap::<String, u64>::new());

    let stop = Arc::new(RwLock::new(vec![false; THREADS]));

    let q_walk = SegQueue::new();
    q_walk.push(root);
    let q_walk = Arc::new(Mutex::new(q_walk));

    scope(|scope| {
        for t_index in 0..THREADS {
            let row_lock_storage = Arc::clone(&row_lock_storage);
            let _q_walk = Arc::clone(&q_walk);
            let stop = Arc::clone(&stop);

            scope.spawn(move |_| {
                loop {
                    let path = match _q_walk.lock().unwrap().pop() {
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
                                    if path.is_dir() {
                                        _q_walk.lock().unwrap().push(path.to_owned());
                                    }

                                    let _size = if treat_as_file(&path).unwrap() {
                                        pure_walk(&path).unwrap()
                                    } else {
                                        // common file or directory
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

// ==================== common functions ====================

fn now() -> u64 {
    // current timestamp
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn get_modified(path: &str) -> u64 {
    fs::metadata(path)
        .unwrap()
        .modified()
        .unwrap()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn treat_as_file(path: &PathBuf) -> Result<bool, Box<dyn std::error::Error>> {
    let filename = path.to_str().unwrap().to_string();
    if filename.starts_with(".") {
        return Ok(true);
    }
    // if filename.ends_with("$RECYCLE.BIN") {
    //     return Ok(true);
    // }
    Ok(false)
}

const SCRIPT_EXTENSIONS: [&str; 5] = [".exe", ".dll", ".bat", ".vbs", ".ps1"];

fn treat_as_script(path: &PathBuf) -> Result<bool, Box<dyn std::error::Error>> {
    let filename = path.to_str().unwrap().to_string();
    for ext in SCRIPT_EXTENSIONS.iter() {
        if filename.ends_with(ext) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn pure_walk(path: &PathBuf) -> Result<u64, Box<dyn std::error::Error>> {
    let mut stack: Vec<PathBuf> = vec![path.into()];
    let mut size = fs::metadata(path)?.len();

    while let Some(path) = stack.pop() {
        if let Ok(entries) = fs::read_dir(&path) {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        size += fs::metadata(&path)?.len();
                        if path.is_dir() {
                            stack.push(path.to_owned());
                        }
                    }
                    Err(e) => println!("Failed to read entry: {}", e),
                }
            }
        } else {
            println!("Failed to open directory: {:?}", &path);
        }
    }
    Ok(size)
}
