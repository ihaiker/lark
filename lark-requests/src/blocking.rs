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

use std::time;

use reqwest::blocking::{Client as BlockingClient, ClientBuilder as BlockingClientBuilder};
use reqwest::Url;

use crate::utils::replace_path_params;
use crate::{LarkError, Request, Response};

#[derive(Debug, Clone)]
pub struct Client {
    client: BlockingClient,
}

impl Default for Client {
    fn default() -> Self {
        let client = BlockingClientBuilder::default()
            .connect_timeout(time::Duration::from_secs(3))
            .timeout(time::Duration::from_secs(7))
            .build()
            .expect("build blocking client");
        Self { client }
    }
}

impl Client {
    pub fn execute<R, T, P>(&self, req: R) -> crate::Result<T>
    where
        P: Response<Target = T> + serde::de::DeserializeOwned,
        R: Request<Target = P>,
        T: serde::de::DeserializeOwned,
    {
        let method = req.method();

        // 处理地址
        let mut address = String::from(req.address());
        if let Some(path_params) = req.path_params() {
            address = replace_path_params(&address, &path_params).to_string();
        }
        let mut address = Url::parse(address.as_str()).map_err(|e| LarkError::new(502, e.to_string()))?;

        // 处理查询参数
        if let Some(query_params) = req.query_params() {
            address.query_pairs_mut().extend_pairs(query_params);
        }

        let mut request = self.client.request(method, address);

        //处理请求头，主要添加请求头
        if let Some(headers) = req.headers() {
            for (header, value) in headers {
                request = request.header(header, value);
            }
        }

        // 处理请求体
        if let Some(body) = req.body() {
            request = request.body(body);
        }

        let resp = request.send()?;
        let bytes = resp.bytes()?;

        let resp = serde_json::from_slice::<R::Target>(&bytes)?;
        if !resp.is_success() {
            return Err(LarkError::from_response(&resp));
        }
        return resp
            .data()
            .ok_or_else(|| LarkError::new(502, "response data is null".to_string()));
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use serde::{Deserialize, Serialize};

    use crate::{Body, FlattenResponse, Request};

    use super::Client;

    #[derive(Serialize, Debug)]
    struct GetTenantAccessTokenRequest {
        app_id: String,
        app_secret: String,
    }

    impl Request for GetTenantAccessTokenRequest {
        type Target = FlattenResponse<TenantAccessToken>;

        fn method(&self) -> reqwest::Method {
            reqwest::Method::POST
        }

        fn address(&self) -> &str {
            "https://open.feishu.cn/open-apis/auth/v3/tenant_access_token/internal/"
        }

        fn body(&self) -> Option<Bytes> {
            let body = serde_json::to_string(self).unwrap();
            Some(Bytes::from(body))
        }
    }

    #[derive(Deserialize, Debug)]
    struct TenantAccessToken {
        tenant_access_token: String,
        expire: u64,
    }

    impl Body for TenantAccessToken {}

    #[test]
    fn blocking_request() {
        let client = Client::default();
        //from env
        let app_id = std::env::var("LARK_APP_ID").unwrap();
        let app_secret = std::env::var("LARK_APP_SECRET").unwrap();
        let req = GetTenantAccessTokenRequest { app_id, app_secret };
        let resp = client.execute(req);
        dbg!(&resp);
        assert!(resp.is_ok());
        let resp = resp.unwrap();
        assert!(resp.tenant_access_token.len() > 0);
        assert!(resp.expire > 0);
    }
}
