use serde::{Serialize, Deserialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Resp<T> {
    pub code: i32,
    pub result: T,
}

impl<T> Resp<T> {
    pub fn new(_code: i32, _result: T) -> Self {
        Resp {
            code: _code,
            result: _result,
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RespErr {
    pub code: i32,
    pub error: Option<String>,
}

impl RespErr {
    pub fn new(_code: i32, _error: Option<String>) -> Self {
        RespErr {
            code: _code,
            error: _error,
        }
    }
}