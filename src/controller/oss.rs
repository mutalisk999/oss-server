use std::env;
use std::sync::Arc;
use axum::{Extension, Router};
use axum::extract::{ContentLengthLimit, Path};
use axum::http::{header::{HeaderMap, HeaderName}, StatusCode};
use axum::routing::{get, post};
use tokio::sync::RwLock;
use wickdb::{BytewiseComparator, Options, WickDB, ReadOptions, DB, WriteOptions};
use wickdb::file::FileStorage;
use bytes::Bytes;
use bson;
use md5;
use serde::{Deserialize, Serialize};
use crate::controller::resp::{Resp, RespErr};


pub type SharedState = Arc<RwLock<State>>;

pub struct State {
    pub db: Option<wickdb::WickDB<FileStorage, BytewiseComparator>>
}


#[derive(Default, Debug, Clone, Serialize, Deserialize)]
struct OssRecordStore {
    origin_name: Option<String>,
    origin_type: Option<String>,
    content_data: Option<Vec<u8>>,
}

impl OssRecordStore {
    fn new(_name: Option<String>, _type: Option<String>, _data: Option<Vec<u8>>) -> Self {
        OssRecordStore {
            origin_name: _name,
            origin_type: _type,
            content_data: _data,
        }
    }
}


pub fn oss_routes() -> Router {
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


async fn get_record_by_key(Path(key): Path<String>,
                           Extension(state): Extension<SharedState>) {}


async fn store_record(headers: HeaderMap,
                      ContentLengthLimit(bytes): ContentLengthLimit<Bytes, { 10 * 1024 * 1024 }>,
                      Extension(state): Extension<SharedState>) -> Result<Bytes, StatusCode> {
    let mut rec_content_length = bytes.len();
    if let Some(content_length) = headers.get(HeaderName::from_static("content-length")) {
        if let Ok(content_length_str) = content_length.to_str() {
            if let Ok(content_length_usize) = content_length_str.parse::<usize>() {
                rec_content_length = content_length_usize
            }
        }
    }

    if rec_content_length > 10 * 1024 * 1024 {
        // size too big
        let resp_err = RespErr::new(-1, Some(String::from("invalid stored record [size is too big]")));
        let json_resp = serde_json::to_string(&resp_err).unwrap();
        return Ok(Bytes::from(json_resp));
    } else if rec_content_length == 0 {
        // size too small
        let resp_err = RespErr::new(-1, Some(String::from("invalid stored record [size is too small]")));
        let json_resp = serde_json::to_string(&resp_err).unwrap();
        return Ok(Bytes::from(json_resp));
    }

    let mut rec_origin_name: Option<String> = None;
    let mut rec_origin_type: Option<String> = None;
    let r = headers.get(HeaderName::from_static("record-origin-name"));
    if r.is_some() {
        rec_origin_name = Some(String::from(r.unwrap().to_str().unwrap()));
    }
    let r = headers.get(HeaderName::from_static("record-origin-type"));
    if r.is_some() {
        rec_origin_type = Some(String::from(r.unwrap().to_str().unwrap()));
    }

    let rec_store = OssRecordStore::new(rec_origin_name, rec_origin_type, Some(bytes.to_vec()));
    let bson_rec = bson::to_bson(&rec_store).unwrap();
    let bson_rec_md5 = md5::compute(bson_rec.as_str().unwrap().as_bytes());

    let r = state.read().await.db.as_ref().unwrap()
        .get(ReadOptions::default(), bson_rec_md5.as_ref());
    if r.is_err() {
        state.write().await.db.as_ref().unwrap()
            .put(WriteOptions::default(), bson_rec_md5.as_ref(), bson_rec.as_str().unwrap().as_bytes()).unwrap();
    }
    let resp_ok = Resp::new(0, Some(format!("{:x}", bson_rec_md5).to_string()));
    let json_resp = serde_json::to_string(&resp_ok).unwrap();
    return Ok(Bytes::from(json_resp));
}
