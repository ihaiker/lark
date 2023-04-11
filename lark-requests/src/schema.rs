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

use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;

use serde::de::{DeserializeOwned, Error, IgnoredAny, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};

pub trait Body: DeserializeOwned {}

/// 通用的响应，包含 code, message, data，并且 data 可以是任意类型，只要实现了 Body 即可
/// 对应文档查看 https://open.feishu.cn/document/ukTMukTMukTM/ukDNz4SO0MjL5QzM/get-
pub trait Response {
    type Target: Body;
    fn code(&self) -> u64;
    fn message(&self) -> &String;
    fn data(self) -> Option<Self::Target>;
    fn is_success(&self) -> bool {
        self.code() == 0
    }
}

///
/// 通用相应实现，data 可以是任意类型，只要实现了 Body 即可。并且使用次返回类型的话，data内容不会放在 data区域，而是把内容展平了
/// 所有的返回内容均放在data区域中
/// 例如：
/// ```json
/// {
///     "code": 0,
///     "message": "ok",
///     "data": {
///         "tenant_access_token": "xxx",
///         "expire": 7200
///     }
/// }
/// ```
#[derive(Debug)]
pub struct BodyResponse<T> {
    code: u64,

    message: String,

    data: Option<T>,
}

impl<T: Body> Response for BodyResponse<T> {
    type Target = T;
    fn code(&self) -> u64 {
        self.code
    }

    fn message(&self) -> &String {
        &self.message
    }

    fn data(self) -> Option<Self::Target> {
        self.data
    }
}

///
/// 自定实现 BodyResponse 的反序列化.
/// 由于当返回错误信息的时候，data区域的内容是不需要的，
/// 但是json中缺包含了`"data": {}` 这样的内容。会导致`serde`反序列化失败，通过自定义实现反序列化，可以解决这个问题
///
impl<'de, T> Deserialize<'de> for BodyResponse<T>
where
    T: DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BodyResponseVisitor<T>(PhantomData<T>);

        impl<'de, T> Visitor<'de> for BodyResponseVisitor<T>
        where
            T: DeserializeOwned,
        {
            type Value = BodyResponse<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct BodyResponse")
            }

            fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut code: Option<u64> = None;
                let mut message: Option<String> = None;
                let mut data: Option<Option<T>> = None;

                while let Some(key) = access.next_key()? {
                    match key {
                        "code" => {
                            code = Some(access.next_value()?);
                        }
                        "msg" => {
                            message = Some(access.next_value()?);
                        }
                        "data" => {
                            if code == Some(0) {
                                // Only deserialize `data` if `code` is 0
                                data = Some(access.next_value()?);
                            } else {
                                // If `code` is not 0, skip over the `data` field without deserializing it.
                                access.next_value::<IgnoredAny>()?;
                            }
                        }
                        _ => {
                            return Err(Error::unknown_field(key, FIELDS));
                        }
                    }
                }
                let code = code.ok_or_else(|| Error::missing_field("code"))?;
                let message = message.ok_or_else(|| Error::missing_field("message"))?;
                Ok(BodyResponse { code, message, data: data.flatten() })
            }
        }

        const FIELDS: &'static [&'static str] = &["code", "message", "data"];
        deserializer.deserialize_struct("BodyResponse", FIELDS, BodyResponseVisitor(PhantomData))
    }
}

///
/// 通用相应实现，data 可以是任意类型，只要实现了 Body 即可。并且使用次返回类型的话，data内容会放在 data区域
/// 但是这种方式，data区域的内容必须是一个对象，不能是一个数组。并且所有的data内容字段会展平在返回中。
/// 例如：
/// ```json
/// {
///     "code": 0,
///     "message": "ok",
///     "tenant_access_token": "xxx",
///     "expire": 7200
/// }
/// ```
#[derive(Debug, serde::Deserialize)]
pub struct FlattenResponse<T> {
    code: u64,

    #[serde(rename = "msg")]
    message: String,

    #[serde(flatten)]
    data: Option<T>,
}

impl<T> Response for FlattenResponse<T>
where
    T: Body,
{
    type Target = T;

    fn code(&self) -> u64 {
        self.code
    }

    fn message(&self) -> &String {
        &self.message
    }

    fn data(self) -> Option<Self::Target> {
        self.data
    }
}

#[cfg(test)]
mod response_tests {
    use serde::{Deserialize, Serialize};

    use super::{BodyResponse, FlattenResponse, Response};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TenantAccessToken {
        tenant_access_token: String,
        expire: u64,
    }

    impl super::Body for TenantAccessToken {}

    #[test]
    fn body_response() {
        let json = r#"
           {
                "code": 0,
                "msg": "ok",
                "data": {
                    "tenant_access_token": "xxx",
                    "expire": 7200
                }
           }
        "#;

        let resp = serde_json::from_str::<BodyResponse<TenantAccessToken>>(json);
        assert!(resp.is_ok());
        let resp = resp.unwrap();
        let data = resp.data();
        assert!(data.is_some());
        let data: TenantAccessToken = data.unwrap();
        assert_eq!("xxx", data.tenant_access_token);
        assert_eq!(7200, data.expire);
    }

    #[test]
    fn body_error_response() {
        let json = r#"
           {
                "code": 100000,
                "msg": "invalid tenant_access_token",
                "data": {}
           }
        "#;
        let resp = serde_json::from_str::<BodyResponse<TenantAccessToken>>(json);
        assert!(resp.is_ok());
        let resp = resp.unwrap();
        assert_eq!(false, resp.is_success());
        assert_eq!(100000, resp.code());
        assert_eq!("invalid tenant_access_token", resp.message());

        let data = resp.data();
        assert!(data.is_none());
    }

    #[test]
    fn flatten_response() {
        let json = r#"
           {
                "code": 0,
                "msg": "ok",
                "tenant_access_token": "xxx",
                "expire": 7200
           }
        "#;
        let resp = serde_json::from_str::<FlattenResponse<TenantAccessToken>>(json);
        assert!(resp.is_ok());
        let resp = resp.unwrap();
        let data = resp.data();
        assert!(data.is_some());
        let data = data.unwrap();
        assert_eq!("xxx", data.tenant_access_token);
    }

    #[test]
    fn flatten_body_error_response() {
        let json = r#"
           {
                "code": 100000,
                "msg": "invalid tenant_access_token",
                "data": {}
           }
        "#;
        let resp = serde_json::from_str::<FlattenResponse<TenantAccessToken>>(json);
        assert!(resp.is_ok());
        let resp = resp.unwrap();
        assert_eq!(100000, resp.code());
        assert_eq!("invalid tenant_access_token", resp.message());

        let data = resp.data();
        assert!(data.is_none());
    }
}

///
/// 请求接口
/// 可以通过实现该接口，来实现自定义的请求。其中 Target 为响应的类型。
/// 例如：
/// ```rust
///     use bytes::Bytes;
///     use lark_requests::{Body, FlattenResponse, Request,Result};
///
///     #[derive(serde::Serialize, Debug)]
///     struct GetTenantAccessTokenRequest {
///         app_id: String,
///         app_secret: String,
///     }
///
///     impl Request for GetTenantAccessTokenRequest {
///         type Target = FlattenResponse<TenantAccessToken>;
///
///         fn method(&self) -> reqwest::Method {
///             reqwest::Method::POST
///         }
///
///         fn address(&self) -> &str {
///             "https://open.feishu.cn/open-apis/auth/v3/tenant_access_token/internal/"
///         }
///
///         fn body(&self) -> Result<Option<bytes::Bytes>> {
///             let body = serde_json::to_string(self).unwrap();
///             Ok(Some(Bytes::from(body)))
///         }
///     }
///
///     #[derive(serde::Deserialize, Debug)]
///     struct TenantAccessToken {
///         tenant_access_token: String,
///         expire: u64,
///     }
///
///     impl Body for TenantAccessToken {}
///
///
/// ```
pub trait Request: serde::Serialize {
    /// 响应的类型
    type Target: Response;

    fn method(&self) -> reqwest::Method {
        reqwest::Method::POST
    }

    fn address(&self) -> &str;

    /// 地址路径上的参数对
    fn path_params(&self) -> Option<HashMap<String, String>> {
        None
    }

    /// 请求需要添加的查询参数
    fn query_params(&self) -> Option<Vec<(String, String)>> {
        None
    }

    /// 请求需要添加的头信息
    fn headers(&self) -> Option<Vec<(String, String)>> {
        None
    }

    /// 请求需要添加的body
    fn body(&self) -> crate::Result<Option<bytes::Bytes>> {
        Ok(None)
    }
}
