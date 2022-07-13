use std::env;
use std::sync::Arc;

use axum::{Extension, Router};
use axum::extract::{BodyStream, Path};
use axum::routing::{get, post};
use bincode;
use bytes::Bytes;
use hyper::body;
use hyper::body::Body;
use hyper::http::{HeaderMap, HeaderValue, StatusCode};
use hyper::http::header::HeaderName;
use md5;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use wickdb::{BytewiseComparator, DB, Options, ReadOptions, WickDB, WriteOptions};
use wickdb::file::FileStorage;

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
    let state = Arc::new(
        RwLock::new(
            State {
                db: Option::from(
                    WickDB::open_db(
                        opt,
                        env::var("OSS_STORE_DIR").unwrap_or(String::from("oss_store")),
                        FileStorage::default(),
                    ).unwrap()),
            }
        )
    );

    Router::new()
        .route("/record/:key", get(get_record_by_key))
        .route("/record", post(store_record))
        .layer(Extension(state))
}


async fn get_record_by_key(Path(key): Path<String>,
                           Extension(state): Extension<SharedState>) -> Result<(HeaderMap, Bytes), StatusCode> {
    let mut headers = HeaderMap::new();
    let rec_bin_md5 = hex::decode(&key.as_str());

    if rec_bin_md5.is_err() {
        // invalid hex string
        let resp_err = RespErr::new(
            -1,
            Some(String::from("invalid hex string [record key]")),
        );
        let json_resp = serde_json::to_string(&resp_err)
            .unwrap();

        return Ok((headers, Bytes::from(json_resp)));
    }

    let rec_bin_vec = state.read()
        .await
        .db
        .as_ref()
        .unwrap()
        .get(
            ReadOptions::default(),
            &rec_bin_md5.unwrap().as_slice(),
        );

    if rec_bin_vec
        .is_err() {
        // get error
        let resp_err = RespErr::new(
            -1,
            Some(String::from("get error [record key]")),
        );
        let json_resp = serde_json::to_string(&resp_err)
            .unwrap();

        return Ok((headers, Bytes::from(json_resp)));
    } else if rec_bin_vec
        .as_ref()
        .unwrap()
        .is_none() {
        // not found in store
        let resp_err = RespErr::new(
            -1,
            Some(String::from("not found [record key]")),
        );
        let json_resp = serde_json::to_string(&resp_err)
            .unwrap();

        return Ok((headers, Bytes::from(json_resp)));
    }

    let record_store: OssRecordStore = bincode::deserialize(
        &rec_bin_vec
            .unwrap()
            .unwrap()
            .as_slice()
    ).unwrap();

    headers.insert(
        HeaderName::from_static("record-origin-name"),
        HeaderValue::from_str(&record_store
            .origin_name
            .unwrap_or(String::default()).as_str())
            .unwrap(),
    );
    headers.insert(
        HeaderName::from_static("record-origin-type"),
        HeaderValue::from_str(&record_store
            .origin_type
            .unwrap_or(String::default()).as_str())
            .unwrap(),
    );
    return Ok((headers, Bytes::from(record_store.content_data.unwrap())));
}


async fn store_record(headers: HeaderMap,
                      stream: BodyStream,
                      Extension(state): Extension<SharedState>) -> Result<Bytes, StatusCode> {
    let mut rec_content_length = 0usize;
    let mut header_found = false;

    if let Some(content_length) = headers.get(HeaderName::from_static("content-length")) {
        if let Ok(content_length_str) = content_length.to_str() {
            if let Ok(content_length_usize) = content_length_str.parse::<usize>() {
                rec_content_length = content_length_usize;
                header_found = true;
            }
        }
    }

    if !header_found {
        let resp_err = RespErr::new(
            -1,
            Some(String::from("not found valid header [content-length]")),
        );
        let json_resp = serde_json::to_string(&resp_err)
            .unwrap();

        return Ok(Bytes::from(json_resp));
    }

    if rec_content_length > 100 * 1024 * 1024 {
        // size too big
        let resp_err = RespErr::new(
            -1,
            Some(String::from("invalid stored record [size is too big]")),
        );
        let json_resp = serde_json::to_string(&resp_err)
            .unwrap();

        return Ok(Bytes::from(json_resp));
    } else if rec_content_length == 0 {
        // size too small
        let resp_err = RespErr::new(
            -1,
            Some(String::from("invalid stored record [size is too small]")),
        );
        let json_resp = serde_json::to_string(&resp_err)
            .unwrap();

        return Ok(Bytes::from(json_resp));
    }

    let mut rec_origin_name: Option<String> = None;
    let mut rec_origin_type: Option<String> = None;

    let origin_name = headers.get(
        HeaderName::from_static("record-origin-name")
    );
    if origin_name.is_some() {
        rec_origin_name = Some(String::from(origin_name.unwrap().to_str().unwrap()));
    }

    let origin_type = headers.get(
        HeaderName::from_static("record-origin-type")
    );
    if origin_type.is_some() {
        rec_origin_type = Some(String::from(origin_type.unwrap().to_str().unwrap()));
    }

    let mut bytes_resp = vec![];
    match body::to_bytes(Body::wrap_stream(stream)).await {
        Ok(v) => {
            bytes_resp.extend_from_slice(v.to_vec().as_slice());
        }
        Err(err) => {
            let resp_err = RespErr::new(
                -1,
                Some(format!("read body stream error: {}", err.to_string())),
            );
            let json_resp = serde_json::to_string(&resp_err)
                .unwrap();

            return Ok(Bytes::from(json_resp));
        }
    }

    let rec_store = OssRecordStore::new(
        rec_origin_name,
        rec_origin_type,
        Some(bytes_resp),
    );
    let rec_bin_vec = bincode::serialize(&rec_store)
        .unwrap();
    let rec_bin_md5 = md5::compute(&rec_bin_vec.as_slice());

    let res_get = state
        .read()
        .await
        .db
        .as_ref()
        .unwrap()
        .get(
            ReadOptions::default(),
            &rec_bin_md5.as_ref(),
        );

    if res_get.is_err() {
        // get error
        let resp_err = RespErr::new(
            -1,
            Some(String::from("get error")),
        );
        let json_resp = serde_json::to_string(&resp_err)
            .unwrap();

        return Ok(Bytes::from(json_resp));
    } else if res_get.unwrap().is_none() {
        // not found in store
        let res_put = state
            .write()
            .await
            .db
            .as_ref()
            .unwrap()
            .put(
                WriteOptions::default(),
                &rec_bin_md5.as_ref(),
                &rec_bin_vec.as_ref(),
            );

        if res_put.is_err() {
            // put error
            let resp_err = RespErr::new(
                -1,
                Some(String::from("put error")),
            );
            let json_resp = serde_json::to_string(&resp_err)
                .unwrap();

            return Ok(Bytes::from(json_resp));
        }
    }

    let resp_ok = Resp::new(
        0,
        Some(format!("{:x}", rec_bin_md5).to_string()),
    );
    let json_resp = serde_json::to_string(&resp_ok)
        .unwrap();

    return Ok(Bytes::from(json_resp));
}
