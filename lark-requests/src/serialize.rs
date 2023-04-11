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

pub trait RequestSerialize {
    fn serialize(&self) -> Option<String>;
}

macro_rules! impl_request_serialize {
    ($($t:ty),*) => {
        $(
            impl RequestSerialize for $t {
                fn serialize(&self) -> Option<String> {
                    Some(self.to_string())
                }
            }
        )*
    }
}
impl_request_serialize!(String, i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, bool, &str);

impl<T: RequestSerialize> RequestSerialize for Option<T> {
    fn serialize(&self) -> Option<String> {
        match self {
            Some(v) => v.serialize(),
            None => None,
        }
    }
}

impl<T: RequestSerialize> RequestSerialize for Vec<T> {
    fn serialize(&self) -> Option<String> {
        let mut s = String::new();
        for (i, v) in self.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            s.push_str(&v.serialize().unwrap());
        }
        Some(s)
    }
}

#[cfg(test)]
mod tests {
    use crate::RequestSerialize;

    macro_rules! test_case {
        ($expect:expr, $actual:expr) => {{
            let a = $expect;
            assert_eq!(a.serialize(), $actual);
        }};
    }

    #[test]
    fn test_se() {
        test_case!(&10, Some("10".to_string()));
        test_case!(vec![1, 2, 3], Some("1,2,3".to_string()));
        test_case!("hello", Some("hello".to_string()));
        test_case!(Some("hello".to_string()), Some("hello".to_string()));

        test_case!(Box::new(10), Some("10".to_string()));
        test_case!(Box::new(&10), Some("10".to_string()));

        test_case!(Box::new("10"), Some("10".to_string()));
        test_case!(Box::new("10".to_string()), Some("10".to_string()))
    }
}
