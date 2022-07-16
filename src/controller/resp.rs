use std::error::Error;
use std::fmt::{Display, Result};

use serde::{Deserialize, Serialize};
use serde::__private::Formatter;

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

impl Display for RespErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if let Some(err_str) = self.error.clone() {
            write!(f, "RespErr: code = {}, error = {}", self.code, err_str)
        } else {
            write!(f, "RespErr: code = {}, error = None", self.code)
        }
    }
}

impl Error for RespErr {}

impl RespErr {
    pub fn new(_code: i32, _error: Option<String>) -> Self {
        RespErr {
            code: _code,
            error: _error,
        }
    }
}
