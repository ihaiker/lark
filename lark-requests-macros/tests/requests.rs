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

use serde::{Deserialize, Serialize};

use lark_requests::Request;
use lark_requests_macros::{Request, Response};

#[derive(Deserialize, Response, Debug)]
pub struct AccessToken {
    pub access_token: String,
    pub expire: u64,
}

#[test]
fn bases_request() {
    #[derive(Request, Serialize)]
    #[request(GET, "https://exmaple.com", AccessToken)]
    struct BasesRequest {}

    let req = BasesRequest {};
    assert_eq!(req.method(), reqwest::Method::GET);
    let url = req.address();
    assert_eq!("https://exmaple.com", url);

    let body = req.body();
    assert!(body.is_ok());
}

#[test]
fn request_with_body() {
    #[derive(Request, Serialize)]
    #[request(POST, "https://exmaple.com/user/:id", AccessToken)]
    struct RequestWithBody {
        #[request(header, rename = "Authorization", with = "Bearer ")]
        #[serde(skip)]
        request_id: String,

        #[request(header, rename = "User")]
        #[serde(skip)]
        user: String,

        #[request(path = "id")]
        #[serde(skip)]
        id: u64,

        #[request(query)]
        #[serde(skip)]
        page: u32,

        #[request(query)]
        #[serde(skip)]
        limit: u32,

        pub f1: String,

        pub f2: String,
    }

    let req = RequestWithBody {
        id: 1,
        page: 1,
        limit: 10,
        request_id: "tests".to_string(),
        user: "ihaiker".to_string(),

        f1: "v1".to_string(),
        f2: "v2".to_string(),
    };
    assert_eq!(req.method(), reqwest::Method::POST);

    let url = req.address();
    assert_eq!("https://exmaple.com/user/:id", url);

    let paths = req.path_params();
    assert!(paths.is_some());
    let paths = paths.unwrap();
    assert_eq!(paths.len(), 1);
    assert_eq!(paths.get("id"), Some(&"1".to_string()));

    let queries = req.query_params();
    assert!(queries.is_some());
    let queries = queries.unwrap();
    assert_eq!(queries.len(), 2);
    assert_eq!(queries.get(0), Some(&("page".to_string(), "1".to_string())));
    assert_eq!(queries.get(1), Some(&("limit".to_string(), "10".to_string())));

    let headers = req.headers();
    assert!(headers.is_some());
    let headers = headers.unwrap();
    assert_eq!(headers.len(), 2);
    assert_eq!(headers.get(0), Some(&("Authorization".to_string(), "Bearer tests".to_string())));
    assert_eq!(headers.get(1), Some(&("User".to_string(), "ihaiker".to_string())));

    let body = req.body().expect("result").expect("body");
    assert_eq!(String::from_utf8(body.to_vec()), Ok(r#"{"f1":"v1","f2":"v2"}"#.to_string()));
}
