use std::borrow::Cow;
use std::env;
use std::sync::Arc;

use axum::{Extension, Router};
use axum::body::Bytes;
use axum::error_handling::HandleErrorLayer;
use axum::extract::{ContentLengthLimit, Path};
use axum::http::{header::{HeaderMap, HeaderName, HeaderValue}, Request, StatusCode};
use axum::routing::{get, post};
use axum::response::IntoResponse;
use tokio::sync::RwLock;
use wickdb::{BytewiseComparator, Options, WickDB};
use wickdb::file::FileStorage;
use tower::{BoxError, ServiceBuilder};

use crate::controller::oss::{SharedState, State};
use tokio::time::Duration;

pub fn register_router() -> Router {
    // new web service router
    let r = Router::new()
        .nest("/oss", oss_routes())
        .layer(
            ServiceBuilder::new()
                // Handle errors from middleware
                .layer(HandleErrorLayer::new(handle_error))
                .load_shed()
                .concurrency_limit(1024)
                .timeout(Duration::from_secs(10))
                .into_inner(),
        );
    r
}

fn oss_routes() -> Router {
    async fn get_record_by_key(Path(key): Path<String>,
                               Extension(state): Extension<SharedState>) {}

    async fn store_record(headers: HeaderMap,
                          ContentLengthLimit(bytes): ContentLengthLimit<Bytes, { 10 * 1024 * 1024 }>,
                          Extension(state): Extension<SharedState>) -> Result<Bytes, StatusCode> {
        let mut rec_content_length = bytes.len();
        if let Some(hv_content_length) = headers.get(HeaderName::from_static("content-length")) {
            if let Ok(hv_content_length_str) = hv_content_length.to_str() {
                if let Ok(hv_content_length_usize) = hv_content_length_str.parse::<usize>() {
                    rec_content_length = hv_content_length_usize
                }
            }
        }
        if rec_content_length > 10 * 1024 * 1024 {
            return Err(StatusCode::FORBIDDEN);
        }

        let rec_origin_name = headers.get(HeaderName::from_static("record-origin-name"))
            .unwrap_or(&HeaderValue::from_static(""));
        let rec_origin_type = headers.get(HeaderName::from_static("record-origin-type"))
            .unwrap_or(&HeaderValue::from_static(""));

        Ok(bytes)
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
        .route("/record", post(store_record))
        .layer(Extension(state))
}

async fn handle_error(error: BoxError) -> impl IntoResponse {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Cow::from(format!("Unhandled internal error: {}", error)),
    )
}