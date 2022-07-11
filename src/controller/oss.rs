use std::sync::Arc;
use tokio::sync::RwLock;
use wickdb::{WickDB, BytewiseComparator};
use wickdb::file::FileStorage;

pub type SharedState = Arc<RwLock<State>>;

pub struct State {
    pub db: Option<wickdb::WickDB<FileStorage, BytewiseComparator>>
}



