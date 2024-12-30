use crossbeam::queue::ArrayQueue;
use crossbeam::scope;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[test]
fn test_scan() {
    // scan();
    // let s = multi_thread_walk().unwrap();
    let s = single_thread_walk().unwrap();
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
    pub fn insert(&mut self, path: &PathBuf) {
        let metadata = fs::metadata(path).unwrap();
        let path_string = path.to_str().unwrap().to_string();
        self.path_to_size.insert(path_string, metadata.len());
    }
    pub fn accumulate(&mut self, path: &PathBuf, add: u64) {
        let parent = path.parent().unwrap();
        let path_string = parent.to_str().unwrap().to_string();
        self.path_to_size
            .entry(path_string)
            .and_modify(|e| *e += add);
    }

    pub fn serialize(&self, path: &str) {
        // open a file
        let mut file = std::fs::File::options()
            .write(true)
            .truncate(true)
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

const EXCLUDE_FILENAME: [&str; 4] = [
    "$RECYCLE.BIN",
    "System Volume Information",
    "pagefile.sys",
    "Config.Msi",
];

fn treat_as_file(path: &PathBuf) -> Result<bool, Box<dyn std::error::Error>> {
    let filename = path.to_str().unwrap().to_string();
    if filename.starts_with(".") {
        return Ok(true);
    }
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
                        storage.insert(&path);

                        if treat_as_file(&path)? {
                            let _size = pure_walk(&path)?;
                            storage.accumulate(&path, _size);
                        } else if path.is_dir() {
                            // println!("Directory: {:?}", path);
                            stack.push(path.to_owned());
                        } else {
                            // println!("File: {:?}", path);
                            let _size = fs::metadata(&path)?.len();
                            storage.accumulate(&path, _size);
                        }
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

fn multi_thread_walk() -> Result<Storage, Box<dyn std::error::Error>> {
    let storage = Arc::new(Mutex::new(Storage::new()));

    const THREADS: usize = 4;
    let q = ArrayQueue::<PathBuf>::new(200);
    let _ = q.push("D:\\".into());

    scope(|scope| {
        for _ in 0..THREADS {
            scope.spawn(|_| {
                while let Some(path) = q.pop() {
                    if let Ok(entries) = fs::read_dir(&path) {
                        for entry in entries {
                            match entry {
                                Ok(entry) => {
                                    let path = entry.path();
                                    // 如果是目录，则递归遍历
                                    storage.lock().unwrap().insert(&path);
                                    if path.is_dir() {
                                        // println!("Directory: {:?}", path);
                                        let _ = q.push(path.to_owned());
                                    } else {
                                        // println!("File: {:?}", path);
                                        storage.lock().unwrap().accumulate(&path, 1);
                                    }
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
    if let Some(storage) = Arc::try_unwrap(storage).ok() {
        return Ok(storage.into_inner()?);
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
