use crossbeam::queue::{ArrayQueue, SegQueue};
use crossbeam::scope;
use dashmap::DashMap;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};

#[test]
fn test_scan() {
    let s = multi_thread_walk().unwrap();
    // let s = single_thread_walk().unwrap();
    s.serialize("output.csv");
}
struct Storage {
    path_to_size: HashMap<String, u64>,
}

impl Storage {
    pub fn new() -> Self {
        let mut path_to_size = HashMap::new();
        path_to_size.reserve(50_0000);
        Self { path_to_size }
    }
    pub fn accumulate(&mut self, path: &PathBuf, add: u64) {
        let parent = match path.parent() {
            Some(p) => p,
            None => return, // root has no parent
        };
        let path_string = parent.to_str().unwrap().to_string();
        self.path_to_size
            .entry(path_string.to_owned())
            .and_modify(|e| *e += add)
            .or_insert_with(|| {
                let metadata = fs::metadata(path_string).unwrap();
                add + metadata.len()
            });
        // recursive call parent to accumulate the size
        self.accumulate(&parent.to_path_buf(), add);
    }

    pub fn serialize(&self, path: &str) {
        // open a file
        let mut file = std::fs::File::options()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)
            .unwrap();
        // sort the path
        let mut paths: Vec<&String> = self.path_to_size.keys().collect();
        paths.sort();
        // write to the file
        for k in paths {
            let v = self.path_to_size.get(k).unwrap();
            let line = format!("{},{}\n", k, v);
            file.write_all(line.as_bytes()).unwrap();
        }
    }
}

fn now() -> u64 {
    // current timestamp
    std::time::SystemTime::now()
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

// 递归改循环
fn single_thread_walk() -> Result<Storage, Box<dyn std::error::Error>> {
    let mut storage = Storage::new();
    let mut stack: Vec<PathBuf> = vec!["D:\\".into()];

    while let Some(path) = stack.pop() {
        if let Ok(entries) = fs::read_dir(&path) {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        // 如果是目录，则递归遍历
                        if path.is_dir() {
                            stack.push(path.to_owned());
                        }

                        let _size = if treat_as_file(&path)? {
                            pure_walk(&path)?
                        } else {
                            fs::metadata(&path)?.len()
                        };
                        storage.accumulate(&path, _size);
                    }
                    Err(e) => println!("Failed to read entry: {}", e),
                }
            }
        } else {
            println!("Failed to open directory: {:?}", &path);
        }
    }
    Ok(storage)
}

struct _RowLockStorage;
impl _RowLockStorage {
    pub fn into_inner(map: DashMap<String, u64>) -> Result<Storage, Box<dyn std::error::Error>> {
        let mut path_to_size = HashMap::new();
        for (k, v) in map {
            path_to_size.insert(k, v);
        }
        Ok(Storage { path_to_size })
    }
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

fn multi_thread_walk() -> Result<Storage, Box<dyn std::error::Error>> {
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
                        },
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
                                    _RowLockStorage::accumulate(Arc::clone(&row_lock_storage), &path, _size);
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
    if let Some(_map) = Arc::try_unwrap(row_lock_storage).ok() {
        return Ok(_RowLockStorage::into_inner(_map)?);
    }
    unreachable!()
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
