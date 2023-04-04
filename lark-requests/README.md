# lark-requests

飞书开放平台Rust SDK请求部分实现。通过 `lark-requests` 可以方便的编写飞书开放平台的api.

## 安装

```toml
[dependencies]
lark-requests = "0.1.0"
```

## 使用

```rust
 use super::Client;
    use lark_requests::{Body, FlattenResponse, Request};
    use bytes::Bytes;

    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Debug)]
    struct GetTenantAccessTokenRequest {
        app_id: String,
        app_secret: String,
    }

    /// 通过实现 `Request` trait 来实现请求
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

    #[tokio::test]
    async fn test_async_request() {
        let client = Client::default();
        //from env
        let app_id = std::env::var("LARK_APP_ID").unwrap();
        let app_secret = std::env::var("LARK_APP_SECRET").unwrap();
        let req = GetTenantAccessTokenRequest { app_id, app_secret };

        let resp = client.execute(req).await;
        
        assert!(resp.is_ok());

        let resp = resp.unwrap();
        assert!(resp.tenant_access_token.len() > 0);
        assert!(resp.expire > 0);
    }
```

