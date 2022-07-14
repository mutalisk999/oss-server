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

pub fn resp_to_json<T>(code: i32, result: T) -> String
    where T: Serialize
{
    let resp_ok = Resp::new(
        code,
        result,
    );
    serde_json::to_string(&resp_ok).unwrap()
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

pub fn resp_err_to_json(code: i32, error: Option<String>) -> String {
    let resp_err = RespErr::new(
        code,
        error,
    );
    serde_json::to_string(&resp_err).unwrap()
}