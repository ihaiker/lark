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

#[allow(unused_extern_crates)]
extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;

use proc_macro::TokenStream;

use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod internals;
mod request;
mod response;

///
/// 生成请求实现
///
/// ```rust
/// use serde::{Deserialize, Serialize};
/// use lark_requests_macros::Response;
///
/// #[derive(Response,Deserialize)]
/// pub struct AccessToken {
///     pub access_token: String,
///     pub expire: u64,
/// }
///
/// #[derive(lark_requests_macros::Request,Serialize)]
/// #[request(GET, "https://open.feishu.cn/open-apis/auth/v3/tenant_access_token/internal", AccessToken, flatten)]
/// pub struct GetAccessTokenRequest {
///     pub name: String,
/// }
/// ```
#[proc_macro_derive(Request, attributes(request, response))]
pub fn derive_request(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    request::derive(&mut input)
        .unwrap_or_else(|es| to_compile_errors(es).into())
        .into()
}

///
/// 生成相应实现
/// ```rust
/// use lark_requests::Body;
/// use lark_requests_macros::Response;
/// use serde::Deserialize;
///
/// #[derive(Deserialize, Response, Debug)]
/// pub struct AccessToken {
///     pub access_token: String,
///     pub expire: u64,
/// }
/// ```
#[proc_macro_derive(Response)]
pub fn derive_response(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    response::derive(&mut input)
        .unwrap_or_else(|es| to_compile_errors(es).into())
        .into()
}

fn to_compile_errors(err: syn::Error) -> proc_macro2::TokenStream {
    let errors = vec![err.to_compile_error()];
    quote! {#(#errors)*}
}
