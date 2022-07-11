use axum::response::Html;
use axum::{Router, Extension};
use axum::routing::get;
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::controller::oss::{State, SharedState};
use axum::extract::Path;
use wickdb::{Options, BytewiseComparator, WickDB};
use flexi_logger::AdaptiveFormat::WithThread;
use wickdb::file::FileStorage;


pub fn register_router() -> Router {
    // new web service router
    let r = Router::new()
        .nest("/oss", oss_routes());
    r
}

fn oss_routes() -> Router {
    async fn get_record_by_key(Path(key): Path<String>, Extension(state): Extension<SharedState>) {

    }

    // new leveldb state
    let opt = Options::<BytewiseComparator>::default();
    let state = Arc::new(RwLock::new(
        State {
            db: Option::from(
                WickDB::open_db(opt,
                                env::var("OSS_STORE_DIR").unwrap_or(String::from("oss_store")),
                                FileStorage::default())
                    .unwrap()),
        }
    ));

    Router::new()
        .route("/record/:key", get(get_record_by_key))
        .layer(Extension(state))
}