use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub data_dir: PathBuf,
}

impl AppState {
    pub fn new(conn: Connection, data_dir: PathBuf) -> Self {
        Self {
            db: Arc::new(Mutex::new(conn)),
            data_dir,
        }
    }
}
