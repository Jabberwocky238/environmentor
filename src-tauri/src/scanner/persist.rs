use crossbeam::queue::SegQueue;
use crossbeam::scope;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

use crate::scanner::NodeRecord;

use super::Storage;

type TyNodeRecord = (String, u64, u64, u64, bool);

pub trait Persist {
    fn _dump(&self, path: &str);
    fn _load(path: &str) -> Self;
}

impl Persist for Storage {
    fn _dump(&self, path: &str) {
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
                v.last_modified.to_string(),
                v.script_count.to_string(),
                v.is_allowed.to_string(),
            ])
            .unwrap();
        }
    }
    fn _load(path: &str) -> Self {
        if fs::metadata(path).is_err() {
            println!("[Storage] load: not found");
            return Self::default();
        }

        let m: HashMap<String, NodeRecord> = csv::Reader::from_path(path)
            .unwrap()
            .deserialize()
            .map(|r: Result<TyNodeRecord, csv::Error>| {
                let (k, v1, v3, v4, v5) = r.unwrap();
                let v = NodeRecord::with(v1, v3, v4, v5);
                (k, v)
            })
            .collect();

        println!("[Storage] load: found {} records", m.len());
        Self { path_map: m, updating: false }
    }
}

