use rusqlite::Connection;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

#[derive(Clone, Serialize, Default)]
pub struct IndexingStatus {
    pub is_running: bool,
    pub current_file: String,
    pub scanned_count: usize,
    pub total_files: usize,
    pub percent: usize,
    pub message: String,
    pub stopped: bool,
}

pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub data_dir: PathBuf,
    pub indexing_status: Arc<Mutex<IndexingStatus>>,
    pub stop_flag: Arc<AtomicBool>,
}

impl AppState {
    pub fn new(conn: Connection, data_dir: PathBuf) -> Self {
        Self {
            db: Arc::new(Mutex::new(conn)),
            data_dir,
            indexing_status: Arc::new(Mutex::new(IndexingStatus::default())),
            stop_flag: Arc::new(AtomicBool::new(false)),
        }
    }
}
