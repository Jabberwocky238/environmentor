use std::fs;
use std::path::PathBuf;

// ==================== common functions ====================
pub fn now() -> u64 {
    // current timestamp
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn get_modified(path: &str) -> u64 {
    fs::metadata(path)
        .unwrap()
        .modified()
        .unwrap()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn treat_as_file(path: &PathBuf) -> Result<bool, Box<dyn std::error::Error>> {
    // let filename = path.to_str().unwrap().to_string();
    // if filename.starts_with(".") {
    //     return Ok(true);
    // }
    // if filename.ends_with("$RECYCLE.BIN") {
    //     return Ok(true);
    // }
    Ok(false)
}

pub fn treat_as_ignore(path: &PathBuf) -> bool {
    let filename = path.file_name().unwrap().to_str().unwrap().to_string();
    if filename.starts_with("$") {
        return true;
    }
    false
}

#[test]
fn test_treat_as_ignore() {
    let path = PathBuf::from("D:\\$RECYCLE.BIN");
    assert_eq!(treat_as_ignore(&path), true);
}

pub fn treat_as_script(path: &PathBuf) -> Result<bool, Box<dyn std::error::Error>> {
    const SCRIPT_EXTENSIONS: [&str; 5] = [".exe", ".dll", ".bat", ".vbs", ".ps1"];
    let filename = path.to_str().unwrap().to_string();
    for ext in SCRIPT_EXTENSIONS.iter() {
        if filename.ends_with(ext) {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn pure_walk(path: &PathBuf) -> Result<u64, Box<dyn std::error::Error>> {
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

pub fn get_drives() -> Vec<PathBuf> {
    let mut drives = vec![];
    for i in b'C'..=b'Z' {
        let drive = format!("{}:\\", i as char);
        if fs::metadata(&drive).is_ok() {
            drives.push(drive.into());
        } else {
            break;
        }
    }
    drives
}

#[test]
fn test_get_drives() {
    let drives = get_drives();
    for d in drives {
        println!("{:?}", d);
    }
}
