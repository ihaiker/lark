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

use quote::quote;
use syn::DeriveInput;

use crate::internals;

pub fn derive(input: &mut DeriveInput) -> Result<proc_macro2::TokenStream, syn::Error> {
    let name = &input.ident;

    let var = internals::container::parse(&input)?;
    let response = var.response();
    let method = var.method();
    let address = var.address();
    let body = var.body();

    let (headers, paths, queries) = internals::fields::parse(input)?;

    Ok(quote! {
        impl lark_requests::Request for #name {
            type Target =  #response;

            fn method(&self) -> reqwest::Method {
                #method
            }

            fn address(&self) -> &str {
                #address
            }

            fn body(&self) -> lark_requests::Result<Option<bytes::Bytes>> {
                #body
            }

            /// 地址路径上的参数对
            fn path_params(&self) -> Option<std::collections::HashMap<String, String>> {
                #paths
            }

            /// 请求需要添加的查询参数
            fn query_params(&self) -> Option<Vec<(String, String)>> {
                #queries
            }

            /// 请求需要添加的头信息
            fn headers(&self) -> Option<Vec<(String, String)>> {
                #headers
            }
        }
    })
}
