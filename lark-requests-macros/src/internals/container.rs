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

use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{DeriveInput, Expr, Token};

use crate::internals;

const REQUEST_ATTRIBUTE_ERROR: &'static str =
    "expected #[request(METHOD, \"url\", RESPONSE_DATA] or #[request(\"url\", RESPONSE_DATA)]";
const ALLOW_METHODS: [&'static str; 8] = ["OPTIONS", "GET", "POST", "PUT", "DELETE", "PATCH", "TRACE", "HEAD"];

macro_rules! err {
    ($($tt:tt)*) => {
        Err(syn::Error::new_spanned($($tt)*))
    }
}

#[derive(Default)]
pub struct RequestVariable {
    pub method: Option<Ident>,
    pub address: Option<String>,
    pub response: Option<Expr>,
    pub flatten: bool,
    pub body: Option<bool>,
}

impl RequestVariable {
    pub fn method(&self) -> proc_macro2::TokenStream {
        let method = self.method.as_ref().unwrap();
        quote! {
            reqwest::Method::#method
        }
    }

    pub fn address(&self) -> &String {
        self.address.as_ref().unwrap()
    }

    pub fn response(&self) -> proc_macro2::TokenStream {
        let data = self.response.as_ref().unwrap();
        if self.flatten {
            quote! {
                lark_requests::FlattenResponse<#data>
            }
        } else {
            quote! {
                lark_requests::BodyResponse<#data>
            }
        }
    }

    pub fn body(&self) -> proc_macro2::TokenStream {
        let body = self.body.unwrap_or_else(|| self.method.as_ref().unwrap().to_string() != "GET");
        if body {
            quote! {
                let body = serde_json::to_vec(self)?;
                Ok(Some(bytes::Bytes::from(body)))
            }
        } else {
            quote! {
                Ok(None)
            }
        }
    }
}

fn parse_method(arg: &Expr) -> Result<Ident, syn::Error> {
    let Expr::Path(id) = arg else {
        return Err(syn::Error::new(arg.span(), "unknown attribute"));
    };
    let ident = id.path.get_ident().unwrap();
    if ALLOW_METHODS.contains(&ident.to_string().as_str()) {
        return Ok(ident.clone());
    }
    Err(syn::Error::new(arg.span(), "unknown attribute"))
}

fn parse_address(arg: &Expr) -> Result<String, syn::Error> {
    let Expr::Lit(lit) = arg else {
        return Err(syn::Error::new(arg.span(), "unknown attribute"));
    };

    let syn::Lit::Str(address) = &lit.lit else {
        return Err(syn::Error::new(arg.span(), "unknown attribute"));
    };

    Ok(address.value())
}

fn parse_response_data(arg: &Expr) -> Result<Expr, syn::Error> {
    if let Expr::Path(_) = arg {
        return Ok(arg.clone());
    }
    err!(arg, REQUEST_ATTRIBUTE_ERROR)
}

///
/// 解析请求的属性
///
/// ### Example
/// ```ignore
/// #[request( GET|POST, "/api/v1/cluster/{cluster}/namespace/{namespace}/pod/{pod}/log", Response, flatten[ = true]]
/// ```
///
pub fn parse(input: &DeriveInput) -> Result<RequestVariable, syn::Error> {
    let attrs = &input.attrs;
    let attr = match internals::get_request_attr(attrs)? {
        Some(attr) => attr,
        None => return err!(input, REQUEST_ATTRIBUTE_ERROR),
    };

    let args = attr.parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)?;
    let mut args = args.iter().rev().collect::<Vec<_>>();

    let mut output = RequestVariable::default();

    let Some(expr) = args.pop() else {
        return err!(attr, REQUEST_ATTRIBUTE_ERROR);
    };

    if let Ok(method) = parse_method(expr) {
        output.method = Some(method);
    } else if let Ok(address) = parse_address(expr) {
        output.method = Some(format_ident!("POST"));
        output.address = Some(address);
    } else {
        return err!(expr, REQUEST_ATTRIBUTE_ERROR);
    }

    if output.address.is_none() {
        let Some(expr) = args.pop() else {
            return err!(attr, REQUEST_ATTRIBUTE_ERROR);
        };
        if let Ok(address) = parse_address(expr) {
            output.address = Some(address);
        } else {
            return err!(expr, REQUEST_ATTRIBUTE_ERROR);
        }
    }

    if output.response.is_none() {
        let Some(expr) = args.pop() else {
            return err!(attr, REQUEST_ATTRIBUTE_ERROR);
        };
        if let Ok(response) = parse_response_data(expr) {
            output.response = Some(response);
        } else {
            return err!(expr, REQUEST_ATTRIBUTE_ERROR);
        }
    }

    if let Some((expr, idx)) = internals::parse_bool_var(&args, "flatten")? {
        output.flatten = expr;
        args.remove(idx);
    }

    if let Some((expr, idx)) = internals::parse_bool_var(&args, "body")? {
        output.body = Some(expr);
        args.remove(idx);
    }

    if !args.is_empty() {
        return err!(args[0], REQUEST_ATTRIBUTE_ERROR);
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::parse;
    use proc_macro2::TokenStream;
    use quote::{format_ident, quote, ToTokens};
    use syn::DeriveInput;

    macro_rules! test {
        ($($tt:tt)*) => {{
            let expr = format!("{}", quote!($($tt)*));
            let input = format!("{}\n{}", expr, "struct Test;");
            let input = syn::parse_str::<TokenStream>(input.as_str()).unwrap();
            let input = syn::parse2::<DeriveInput>(input).expect("parse2");
            parse(&input).expect("parse")
        }}
    }

    #[test]
    #[should_panic]
    fn test_duplicate() {
        test!(
            #[request(GET, "https://exmaple.com/test")]
            #[request(GET, "https://exmaple.com/test")]
        );
    }

    #[test]
    fn test_container_parse() {
        let var = test!(
            #[request(GET, "https://exmaple.com/test", lark_requests::AssertToken)]
        );
        assert_eq!(var.method, Some(format_ident!("GET")));
        assert_eq!(var.address, Some(String::from("https://exmaple.com/test")));
        assert_eq!(
            var.response.unwrap().to_token_stream().to_string(),
            String::from("lark_requests :: AssertToken")
        );
        assert_eq!(var.flatten, false);

        let var = test! {
            #[request("https://exmaple.com/test", AssertToken)]
        };
        assert_eq!(var.method, Some(format_ident!("POST")));
        assert_eq!(var.address, Some(String::from("https://exmaple.com/test")));
        assert_eq!(var.response.unwrap().to_token_stream().to_string(), String::from("AssertToken"));
        assert_eq!(var.flatten, false);

        let var = test! {
            #[request("https://exmaple.com/test", AssertToken, flatten)]
        };
        assert_eq!(var.method, Some(format_ident!("POST")));
        assert_eq!(var.address, Some(String::from("https://exmaple.com/test")));
        assert_eq!(var.response.unwrap().to_token_stream().to_string(), String::from("AssertToken"));
        assert_eq!(var.flatten, true);

        let var = test! {
            #[request("https://exmaple.com/test", AssertToken, flatten = true)]
        };
        assert_eq!(var.method, Some(format_ident!("POST")));
        assert_eq!(var.address, Some(String::from("https://exmaple.com/test")));
        assert_eq!(var.response.unwrap().to_token_stream().to_string(), String::from("AssertToken"));
        assert_eq!(var.flatten, true);
    }

    #[test]
    #[should_panic]
    fn test_failed_parse() {
        test! {
            #[request("https://exmaple.com/test", AssertToken, flatten = true, unknown)]
        };
    }
}
