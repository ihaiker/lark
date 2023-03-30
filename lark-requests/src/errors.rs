/*
 * MIT License
 *
 * Copyright (c) 2023  ihaiker (ni@renzhen.la) .
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

use std::fmt;

use reqwest::{Error as ReqWestError, StatusCode};
use serde_json::Error as JsonError;

pub type Result<T> = std::result::Result<T, LarkError>;

///
/// 通用的错误类型，包含 code, message
///
/// 查阅文档：https://open.feishu.cn/document/ukTMukTMukTM/ugjM14COyUjL4ITN
///
#[derive(Debug)]
pub struct LarkError {
    code: u64,
    message: String,
}

impl LarkError {
    pub fn new(code: u64, message: String) -> Self {
        LarkError { code, message }
    }

    pub fn from_response<T>(response: &T) -> Self
    where
        T: super::Response,
    {
        Self::new(response.code(), response.message().clone())
    }

    pub(crate) fn from_req_west(err: ReqWestError) -> LarkError {
        if err.is_connect() {
            LarkError::new(500, format!("connect error: {}", err))
        } else if err.is_timeout() {
            LarkError::new(500, format!("timeout error: {}", err))
        } else if err.is_status() {
            let status = match err.status() {
                Some(status) => status,
                None => StatusCode::INTERNAL_SERVER_ERROR,
            };
            LarkError::new(status.as_u16() as u64, format!("status error: {}", err))
        } else {
            LarkError::new(500, format!("unknown error: {}", err))
        }
    }

    pub(crate) fn from_json_serde(err: JsonError) -> LarkError {
        LarkError::new(500, format!("json serde error: {}", err))
    }
}

impl fmt::Display for LarkError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]: {}", self.code, self.message)
    }
}

impl From<reqwest::Error> for LarkError {
    fn from(err: reqwest::Error) -> Self {
        LarkError::from_req_west(err)
    }
}

impl From<serde_json::Error> for LarkError {
    fn from(err: serde_json::Error) -> Self {
        LarkError::from_json_serde(err)
    }
}
