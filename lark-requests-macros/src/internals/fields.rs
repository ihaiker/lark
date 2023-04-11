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

use std::ops::{Deref, DerefMut};

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::{Data, DataStruct, DeriveInput, Expr, Token};

use crate::internals;

pub enum VariableMode {
    /// 用于请求头的字段
    Header,
    /// 用于请求路径的字段
    Path,
    /// 用户请求参数的字段
    Query,
}

pub struct FieldVariable {
    pub field: String,
    pub with: Option<String>,
    pub mode: Option<VariableMode>,
    pub rename: Option<String>,
    pub serialize_with: Option<String>,
}

impl FieldVariable {
    pub fn new(field: String) -> Self {
        FieldVariable { field, with: None, mode: None, rename: None, serialize_with: None }
    }
}

impl ToTokens for FieldVariable {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        //key
        let name = match self.rename {
            Some(ref rename) => rename,
            None => &self.field,
        };
        //Punct::new(',', Spacing::Alone);

        //value
        let value = match self.serialize_with {
            Some(ref serialize_with) => {
                let field = format_ident!("{}", self.field);
                let serialize_with = syn::parse_str::<Expr>(serialize_with).unwrap();
                quote! { #serialize_with(&self.#field) }
            }
            None => {
                let field = format_ident!("{}", self.field);
                quote! { lark_requests::RequestSerialize::serialize(&self.#field) }
            }
        };

        //value append with
        let value = match self.with {
            Some(ref with) => {
                quote! {
                    match #value {
                        Some(val) => Some(format!("{}{}", #with, val)),
                        None => None
                    }
                }
            }
            None => value,
        };
        let token = quote! {
            String::from(#name), #value
        };
        token.to_tokens(tokens);
    }
}

#[derive(Default)]
pub struct FieldsVariable(Vec<FieldVariable>);

impl Deref for FieldsVariable {
    type Target = Vec<FieldVariable>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FieldsVariable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ToTokens for FieldsVariable {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.0.is_empty() {
            quote!(None).to_tokens(tokens);
            return;
        }

        if let Some(VariableMode::Path) = self.0.first().unwrap().mode.as_ref() {
            let fields = &self.0;
            let st = quote! {
                let mut headers = std::collections::HashMap::new();
                #(
                    if let (key, Some(val)) = (#fields) {
                        headers.insert(key, val);
                    }
                )*;
                Some(headers)
            };
            st.to_tokens(tokens);
        } else {
            let fields = &self.0;
            let st = quote! {
                let mut items = Vec::new();
                #(
                    if let (key, Some(val)) = (#fields) {
                        items.push((key, val));
                    }
                )*
                Some(items)
            };
            st.to_tokens(tokens);
        }
    }
}

///
/// 解析请求的属性
///
/// ### Example
/// ```ignore
/// #[request(header, rename = "X-Request-Id", serialize_with = "uuid::Uuid::to_string")
/// or
/// #[request(path, rename = "id", serialize_with = "uuid::Uuid::to_string")
/// or
/// #[request(query, rename = "id", serialize_with = "uuid::Uuid::to_string")
/// or
/// #[request(header = "id", serialize_with = "uuid::Uuid::to_string")
/// or
/// #[request(path = "id", serialize_with = "uuid::Uuid::to_string")
/// or
/// #[request(query = "id")
/// ```
///
pub fn parse(input: &DeriveInput) -> Result<(FieldsVariable, FieldsVariable, FieldsVariable), syn::Error> {
    let Data::Struct(DataStruct { fields, .. }) = &input.data else {
        return Err(syn::Error::new_spanned(input, "only support struct"));
    };

    let mut headers = FieldsVariable::default();
    let mut paths = FieldsVariable::default();
    let mut queries = FieldsVariable::default();

    for field in fields.iter() {
        let attrs = &field.attrs;
        let Some(attr) = internals::get_request_attr(attrs)? else {
            continue;
        };
        let field_name = field.ident.as_ref().unwrap().to_string();

        let mut var = FieldVariable::new(field_name);

        let args = attr.parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)?;
        let mut args = args.iter().rev().collect::<Vec<_>>();

        if let Some((_header, rename, index)) = internals::parse_bool_or_string_var(&args, "header")? {
            var.mode = Some(VariableMode::Header);
            var.rename = rename;
            args.remove(index);
        } else if let Some((_path, rename, index)) = internals::parse_bool_or_string_var(&args, "path")? {
            var.mode = Some(VariableMode::Path);
            var.rename = rename;
            args.remove(index);
        } else if let Some((_query, rename, index)) = internals::parse_bool_or_string_var(&args, "query")? {
            var.mode = Some(VariableMode::Query);
            var.rename = rename;
            args.remove(index);
        }

        if let Some((rename, index)) = internals::parse_string_var(&args, "rename")? {
            var.rename = Some(rename);
            args.remove(index);
        }
        if let Some((serialize_with, index)) = internals::parse_string_var(&args, "serialize_with")? {
            var.serialize_with = Some(serialize_with);
            args.remove(index);
        }

        if let Some((with, index)) = internals::parse_string_var(&args, "with")? {
            var.with = Some(with);
            args.remove(index);
        }

        if !args.is_empty() {
            return Err(syn::Error::new_spanned(args[0], "invalid attribute"));
        }

        match var.mode {
            Some(VariableMode::Header) => headers.push(var),
            Some(VariableMode::Path) => paths.push(var),
            Some(VariableMode::Query) => queries.push(var),
            None => return Err(syn::Error::new_spanned(field, "missing attribute")),
        }
    }
    Ok((headers, paths, queries))
}

#[cfg(test)]
mod tests {

    use quote::{quote, ToTokens};
    use syn::DeriveInput;

    use super::parse;

    macro_rules! test {
        ($($tt:tt)*) => {{
            let input = quote!{$($tt)*};
            let input = syn::parse2::<DeriveInput>(input).expect("parse2");
            parse(&input).expect("parse")
        }}
    }

    #[test]
    fn test_parse() {
        let (headers, paths, queries) = test! {
            #[request("https://exmaple.com/:id/", AssertToken)]
            struct Test {
                #[request(header, rename = "Authorization", with = "Bearer ")]
                request_id: String,

                 #[request(header, rename = "X-Request-Type")]
                request_type: String,

                #[request(path = "id")]
                id: u64,

                #[request(query, with = "zt_")]
                page: usize,

                #[request(query)]
                limit: String,

                #[request(query, serialize_with = "Mode::to_string")]
                mode: Mode,
            }
        };

        assert_eq!(headers.len(), 2);
        assert_eq!(headers[0].field.as_str(), "request_id");
        assert_eq!(headers[0].rename, Some(String::from("Authorization")));

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].field.as_str(), "id");
        assert_eq!(paths[0].rename, Some(String::from("id")));

        assert_eq!(queries.len(), 3);
        assert_eq!(queries[0].field.as_str(), "page");
        assert_eq!(queries[0].rename, None);
        assert_eq!(queries[1].field.as_str(), "limit");
        assert_eq!(queries[1].rename, None);

        assert_eq!(queries[2].field.as_str(), "mode");
        assert_eq!(queries[2].rename, None);
        assert_eq!(queries[2].serialize_with, Some(String::from("Mode::to_string")));

        println!("{}", headers.to_token_stream());
        println!("{}", paths.to_token_stream());
        println!("{}", queries.to_token_stream());
    }
}
