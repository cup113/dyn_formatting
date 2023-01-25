//! A module provides limited & safe Python-style dynamic(runtime) formatting.

use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;

/// Lightweight, dynamic, Python-styled string formatting (Only support `String`,
/// `{key}` patterns). It only needs `std` to work.
///
/// Escape patterns are `{{` and `}}`.
///
/// It returns the formatted string.
///
/// ## Errors
///
/// 1. Error kind `DynamicFormatErrorKind::KeyError` if the key in the brackets
///    is not found in `dictionary`.
/// 2. Error kind `DynamicFormatErrorKind::TokenError` if there is any
///    unmatched bracket (`{` or `}`)
///
/// ## Examples
///
/// ```
/// use dyn_formatting::dynamic_format;
/// assert_eq!(
///     dynamic_format(
///         "I'm {name}. I'm {age} years old now.",
///         &[("name", "ABC"), ("age", "20")].into()
///     ).unwrap(),
///     "I'm ABC. I'm 20 years old now.".to_string()
/// );
/// ```
///
/// ```
/// use dyn_formatting::dynamic_format;
/// use std::collections::HashMap;
///
/// let value_age = (15).to_string(); // Make lifetime long enough
/// let dictionary = HashMap::from([
///     ("age", value_age.as_str()),
/// ]);
/// assert_eq!(
///     dynamic_format("{{{age} }}{age}", &dictionary).unwrap(),
///     "{15 }15"
/// )
/// ```
///
/// ```
/// use dyn_formatting::dynamic_format;
/// assert!(
///     dynamic_format(
///         "I'm {name}. I'm {age} years old now.",
///         &[("name", "ABC")].into()
///     ).is_err() // Key error
/// );
/// ```
///
/// ```
/// use dyn_formatting::dynamic_format;
/// assert!(
///     dynamic_format(
///         "I'm {name{name}}.",
///         &[("name", "ABC")].into()
///     ).is_err() // Token error: '{' unmatched.
/// );
/// ```

pub fn dynamic_format(
    pattern: &str,
    dictionary: &HashMap<&str, &str>,
) -> Result<String, Box<DynamicFormatError>> {
    if pattern.find('{') == None && pattern.find('}') == None {
        return Ok(pattern.to_string());
    }
    let chars: Vec<char> = pattern.chars().collect();
    let mut ans: String = String::with_capacity(pattern.len());
    let mut left_brace = (false, 0usize);
    let mut right_brace = (false, 0usize);
    let mut key = String::with_capacity(16);

    macro_rules! token_error {
        ($pos: expr, $msg: expr) => {
            return Err(Box::new(DynamicFormatError {
                pattern: pattern.to_string(),
                pos: $pos,
                kind: DynamicFormatErrorKind::TokenError { desc: $msg.into() },
            }));
        };
    }

    for (i, c) in chars.iter().enumerate() {
        if *c == '{' {
            if left_brace.0 {
                if left_brace.1 + 1 == i {
                    ans.push('{');
                    left_brace = (false, 0);
                } else {
                    token_error!(left_brace.1, "Unmatched token '{'");
                }
            } else {
                left_brace = (true, i);
            }
        } else if *c == '}' {
            if right_brace.0 {
                if right_brace.1 + 1 == i {
                    ans.push('}');
                    right_brace = (false, 0);
                } else {
                    token_error!(right_brace.1, "Unmatched token '}'");
                }
            } else if left_brace.0 {
                if let Some(s) = dictionary.get(key.as_str()) {
                    ans.push_str(s);
                } else {
                    return Err(Box::new(DynamicFormatError {
                        pattern: pattern.to_string(),
                        pos: left_brace.1,
                        kind: DynamicFormatErrorKind::KeyError {
                            key,
                            entries: dictionary
                                .iter()
                                .map(|s| (s.0.to_string(), s.1.to_string()))
                                .collect(),
                        },
                    }));
                }
                key.clear();
                left_brace = (false, 0);
            } else {
                right_brace = (true, i);
            }
        } else {
            if left_brace.0 {
                key.push(*c);
            } else {
                ans.push(*c);
            }
        }
    }

    if left_brace.0 {
        token_error!(left_brace.1, "Unmatched token '{'");
    }
    if right_brace.0 {
        token_error!(right_brace.1, "Unmatched token '}'");
    }

    Ok(ans)
}

/// Error types during dynamic formatting.
#[derive(Debug, Clone)]
pub struct DynamicFormatError {
    /// The pattern which causes error.
    pub pattern: String,
    /// The position (index) where the error occurs.
    /// Start from 0 itself, but from 1 when formatting.
    pub pos: usize,
    /// Error kind.
    pub kind: DynamicFormatErrorKind,
}

/// Kinds of error. Provide more information of the error.
#[derive(Debug, Clone)]
pub enum DynamicFormatErrorKind {
    /// The token (formatting grammar) is wrong.
    TokenError {
        /// The brief description of error.
        desc: String,
    },
    /// The key (in braces) is not found in the dictionary.
    KeyError {
        /// The key which is not found.
        key: String,
        /// The entries of the dictionary. It is used to provide help information.
        entries: Vec<(String, String)>,
    },
}

impl Display for DynamicFormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            DynamicFormatErrorKind::TokenError { desc } => write!(
                f,
                "Parse arguments failed: Token Error ({}) when \
                parsing pattern \"{}\" at pos {}.",
                desc, self.pattern, self.pos
            ),
            DynamicFormatErrorKind::KeyError { key, entries } => write!(
                f,
                "Parse arguments failed: Key Not Found \
                (key: \"{}\") when parsing pattern \"{}\" at pos {}.\n\
                Help: These are valid key-value pairs:\n{}",
                key,
                self.pattern,
                self.pos + 1,
                entries
                    .iter()
                    .map(|(key, value)| format!("- {}: {}", key, value))
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
        }
    }
}

impl Error for DynamicFormatError {}

#[cfg(test)]
mod tests {
    use super::*;
    use DynamicFormatErrorKind::*;

    macro_rules! dynamic_format {
        ($pattern: expr, $dict_list: expr) => {
            dynamic_format(&$pattern.to_string(), &$dict_list.into())
        };
    }

    #[test]
    fn test_no_replace() {
        assert_eq!(dynamic_format!("", []).unwrap(), "".to_string());
        assert_eq!(
            dynamic_format!("abcdefg", []).unwrap(),
            "abcdefg".to_string()
        );
        assert_eq!(
            dynamic_format!("abc", [("abc", "")]).unwrap(),
            "abc".to_string()
        );
        assert_eq!(
            dynamic_format!("we-have", [("we", "")]).unwrap(),
            "we-have".to_string()
        );
    }

    #[test]
    fn test_escape() {
        assert_eq!(dynamic_format!("}}", []).unwrap(), "}".to_string());
        assert_eq!(
            dynamic_format!("{{ab}}", [("ab", "1")]).unwrap(),
            "{ab}".to_string()
        );
        assert_eq!(dynamic_format!("{{234", []).unwrap(), "{234".to_string());
        assert_eq!(dynamic_format!("{{{{a}}", []).unwrap(), "{{a}".to_string());
    }

    #[test]
    fn test_replace() {
        assert_eq!(
            dynamic_format!("{ab}", [("ab", "1")]).unwrap(),
            "1".to_string()
        );
        assert_eq!(
            dynamic_format!("1{a}32{a}4", [("a", "555"), ("b", "")]).unwrap(),
            "1555325554".to_string()
        );
        assert_eq!(
            dynamic_format!("{key1}-{key2}", [("key1", "0"), ("key2", "a")]).unwrap(),
            "0-a".to_string()
        );
    }

    #[test]
    fn test_mixed() {
        assert_eq!(
            dynamic_format!("{{{a}", [("a", "1")]).unwrap(),
            "{1".to_string()
        );
        assert_eq!(
            dynamic_format!("{{|{k}}}", [("k", "x123")]).unwrap(),
            "{|x123}".to_string()
        );
        assert_eq!(
            dynamic_format!("{{{key1}}}-}}}}{key2}", [("key1", "0"), ("key2", "a")]).unwrap(),
            "{0}-}}a".to_string()
        );
    }

    #[test]
    fn test_key_error() {
        match *dynamic_format!("{abc}", [("abd", "1")]).unwrap_err() {
            DynamicFormatError {
                pattern,
                pos,
                kind: KeyError { key, entries },
            } => {
                assert_eq!(pattern.as_str(), "{abc}");
                assert_eq!(key, "abc");
                assert_eq!(entries, vec![("abd".into(), "1".into())]);
                assert_eq!(pos, 0);
            }
            _ => unreachable!(),
        }
        match *dynamic_format!("234{ac}{ab}", [("ac", "1"), ("aa", ".")]).unwrap_err() {
            DynamicFormatError {
                pos,
                kind: KeyError { key, .. },
                ..
            } => {
                assert_eq!(key, "ab");
                assert_eq!(pos, 7);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_token_error() {
        match *dynamic_format!("{abc", [("abc", "1")]).unwrap_err() {
            DynamicFormatError {
                pattern,
                pos,
                kind: TokenError { desc },
            } => {
                assert_eq!(pattern.as_str(), "{abc");
                assert!(desc.contains("'{'"));
                assert_eq!(pos, 0);
            }
            _ => unreachable!(),
        }
        match *dynamic_format!("{{a}}}324", []).unwrap_err() {
            DynamicFormatError {
                pos,
                kind: TokenError { desc },
                ..
            } => {
                assert!(desc.contains("'}'"));
                assert_eq!(pos, 5);
            }
            _ => unreachable!(),
        }
        match *dynamic_format!("{na{me}324", []).unwrap_err() {
            DynamicFormatError {
                pos,
                kind: TokenError { desc },
                ..
            } => {
                assert!(desc.contains("'{'"));
                assert_eq!(pos, 0);
            }
            _ => unreachable!(),
        }
        match *dynamic_format!("name}3}24", []).unwrap_err() {
            DynamicFormatError {
                pos,
                kind: TokenError { desc },
                ..
            } => {
                assert!(desc.contains("'}'"));
                assert_eq!(pos, 4);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_error_display() {
        println!("{}", dynamic_format!("name}3}24", []).unwrap_err());
        println!(
            "{}",
            dynamic_format!("234{ac}{ab}", [("ac", "1"), ("aa", ".")]).unwrap_err()
        );
    }
}
