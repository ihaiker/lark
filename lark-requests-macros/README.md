# lark-requests-macros

使用`#[lark_request_macros::request]`宏来简化`lark-requests`的使用。

## 使用

```rust
use lark_request_macros::request;

#[request(POST, "https://open.feishu.cn/open-apis/auth/v3/tenant_access_token/internal")]
pub struct GetAccessTokenRequest {
    pub app_id: String,
    pub app_secret: String,
}
```


