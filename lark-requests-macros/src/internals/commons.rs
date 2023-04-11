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

use syn::spanned::Spanned;
use syn::{Attribute, Expr};

///
/// 解析参数中的bool类型变量
///
pub fn parse_bool_var(args: &Vec<&Expr>, ident: &str) -> Result<Option<(bool, usize)>, syn::Error> {
    for (idx, &expr) in args.iter().enumerate() {
        match expr {
            Expr::Path(path) if path.path.is_ident(ident) => {
                return Ok(Some((true, idx)));
            }
            Expr::Assign(assign) => {
                let Expr::Path(path) = &*assign.left else {
                    continue;
                };
                if !path.path.is_ident(ident) {
                    continue;
                }

                let Expr::Lit(lit) = &*assign.right else {
                    return Err(syn::Error::new_spanned(expr, "invalid value"));
                };
                let syn::Lit::Bool(flatten) = &lit.lit else {
                    return Err(syn::Error::new_spanned(expr, "invalid value"));
                };
                return Ok(Some((flatten.value, idx)));
            }
            _ => {}
        }
    }
    Ok(None)
}

pub fn parse_string_var(args: &Vec<&Expr>, ident: &str) -> Result<Option<(String, usize)>, syn::Error> {
    for (idx, &expr) in args.iter().enumerate() {
        match expr {
            Expr::Assign(assign) => {
                let Expr::Path(path) = &*assign.left else {
                    continue;
                };
                if !path.path.is_ident(ident) {
                    continue;
                }

                let Expr::Lit(lit) = &*assign.right else {
                    return Err(syn::Error::new_spanned(expr, "invalid value"));
                };
                let syn::Lit::Str(flatten) = &lit.lit else {
                    return Err(syn::Error::new_spanned(expr, "invalid value"));
                };
                return Ok(Some((flatten.value(), idx)));
            }
            _ => {}
        }
    }
    Ok(None)
}

pub fn parse_bool_or_string_var(
    args: &Vec<&Expr>,
    ident: &str,
) -> Result<Option<(bool, Option<String>, usize)>, syn::Error> {
    for (idx, &expr) in args.iter().enumerate() {
        match expr {
            Expr::Path(path) if path.path.is_ident(ident) => {
                return Ok(Some((true, None, idx)));
            }
            Expr::Assign(assign) => {
                let Expr::Path(path) = &*assign.left else {
                    continue;
                };
                if !path.path.is_ident(ident) {
                    continue;
                }

                let Expr::Lit(lit) = &*assign.right else {
                    return Err(syn::Error::new_spanned(expr, "invalid value"));
                };

                match &lit.lit {
                    syn::Lit::Bool(flatten) => {
                        return Ok(Some((flatten.value, None, idx)));
                    }
                    syn::Lit::Str(str) => {
                        return Ok(Some((true, Some(str.value()), idx)));
                    }
                    _ => {
                        return Err(syn::Error::new_spanned(expr, "invalid value"));
                    }
                }
            }

            Expr::Lit(lit) => {
                let syn::Lit::Str(str) = &lit.lit else {
                    return Err(syn::Error::new_spanned(expr, "invalid value"));
                };
                return Ok(Some((true, Some(str.value()), idx)));
            }
            _ => {}
        }
    }
    Ok(None)
}

pub fn get_request_attr(attrs: &Vec<Attribute>) -> Result<Option<&Attribute>, syn::Error> {
    let attrs = attrs.iter().filter(|attr| attr.path().is_ident("request")).collect::<Vec<_>>();
    if attrs.len() > 1 {
        return Err(syn::Error::new(attrs[1].span(), "duplicate attribute `request`"));
    }
    if attrs.is_empty() {
        return Ok(None);
    }
    Ok(Some(attrs[0]))
}
