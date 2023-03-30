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

use std::borrow::Cow;
use std::collections::HashMap;

use regex::Regex;

/// 替换地址路径参数
pub fn replace_path_params<'t>(path: &'t str, params: &HashMap<&str, String>) -> Cow<'t, str> {
    let re = Regex::new(r":(\w+)").unwrap();
    re.replace_all(path, |caps: &regex::Captures| match params.get(&caps[1]) {
        Some(value) => value.to_string(),
        None => caps[0].to_string(),
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    #[test]
    fn test_replace_path_params() {
        {
            let path = "/v1/tenants/:tenant_id";
            let mut params = HashMap::default();
            params.insert("tenant_id", "123".to_string());

            let replaced_path = super::replace_path_params(path, &params);
            assert_eq!(replaced_path, "/v1/tenants/123");
        }

        {
            let path = "/v1/:id/:id2";
            let mut params = HashMap::default();
            params.insert("id", "1".to_string());
            params.insert("id2", "2".to_string());

            let replaced_path = super::replace_path_params(path, &params);
            assert_eq!(replaced_path, "/v1/1/2");
        }
    }
}
