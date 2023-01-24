//! A module provides limited & safe Python-style dynamic(runtime) formatting.

use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;

/// Simple, dynamic, Python-styled string formatting (Only support `String`,
/// `{key}` patterns).
///
/// Escape like `{{` or `}}`.
///
/// ## Errors
///
/// 1. Raise `DynamicFormatError::KeyError` while the key in the brackets
///    is not found in `dictionary`.
/// 2. Raise `DynamicFormatError::TokenError` while there is any
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
) -> Result<String, DynamicFormatError> {
    if pattern.find('{') == None && pattern.find('}') == None {
        return Ok(pattern.to_string());
    }
    let chars: Vec<char> = pattern.chars().collect();
    let mut ans: String = String::with_capacity(pattern.len());
    let mut left_bracket = (false, 0usize);
    let mut right_bracket = (false, 0usize);
    let mut key = String::with_capacity(16);

    for (i, c) in chars.iter().enumerate() {
        if *c == '{' {
            if left_bracket.0 {
                if left_bracket.1 + 1 == i {
                    ans.push('{');
                    left_bracket = (false, 0);
                } else {
                    return Err(DynamicFormatError::TokenError {
                        pattern: pattern.to_owned(),
                        desc: "Unmatched token '{'".into(),
                        pos: left_bracket.1 + 1,
                    });
                }
            } else {
                left_bracket = (true, i);
            }
        } else if *c == '}' {
            if right_bracket.0 {
                if right_bracket.1 + 1 == i {
                    ans.push('}');
                    right_bracket = (false, 0);
                } else {
                    return Err(DynamicFormatError::TokenError {
                        pattern: pattern.to_owned(),
                        desc: "Unmatched token '}'".into(),
                        pos: right_bracket.1 + 1,
                    });
                }
            } else if left_bracket.0 {
                if let Some(s) = dictionary.get(key.as_str()) {
                    ans.push_str(s);
                } else {
                    return Err(DynamicFormatError::KeyError {
                        pattern: pattern.to_string(),
                        key,
                        dict: dictionary
                            .iter()
                            .map(|s| (s.0.to_string(), s.1.to_string()))
                            .collect(),
                        pos: left_bracket.1 + 1,
                    });
                }
                key.clear();
                left_bracket = (false, 0);
            } else {
                right_bracket = (true, i);
            }
        } else {
            if left_bracket.0 {
                key.push(*c);
            } else {
                ans.push(*c);
            }
        }
    }

    if left_bracket.0 {
        return Err(DynamicFormatError::TokenError {
            pattern: pattern.to_string(),
            desc: "Unmatched token '{'".into(),
            pos: left_bracket.1,
        });
    }
    if right_bracket.0 {
        return Err(DynamicFormatError::TokenError {
            pattern: pattern.to_string(),
            desc: "Unmatched token '}'".into(),
            pos: right_bracket.1,
        });
    }

    Ok(ans)
}

#[derive(Debug)]
pub enum DynamicFormatError {
    TokenError {
        pattern: String,
        desc: String,
        pos: usize,
    },
    KeyError {
        pattern: String,
        key: String,
        dict: Vec<(String, String)>,
        /// Start from 1
        pos: usize,
    },
}

impl Display for DynamicFormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TokenError { pattern, desc, pos } => write!(
                f,
                "Parse arguments failed: Token Error ({}) when \
                parsing pattern \"{}\" at pos {}.",
                desc, pattern, pos
            ),
            Self::KeyError {
                pattern,
                key,
                pos,
                dict,
            } => write!(
                f,
                "Parse arguments failed: Key Not Found \
                (key: \"{}\") when parsing pattern \"{}\" at pos {}.\n\
                Help: These are valid key-value pairs:\n{}",
                key,
                pattern,
                pos,
                dict.iter()
                    .map(|(key, value)| format!("{}: {}", key, value))
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
        }
    }
}

impl Error for DynamicFormatError {}

#[cfg(test)]
mod tests {
    use super::dynamic_format;
    use super::DynamicFormatError::*;

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
        match dynamic_format!("{abc}", [("abd", "1")]) {
            Err(KeyError {
                pattern,
                key,
                pos,
                dict,
            }) => {
                assert_eq!(pattern.as_str(), "{abc}");
                assert_eq!(key, "abc");
                assert_eq!(dict, vec![("abd".into(), "1".into())]);
                assert_eq!(pos, 1);
            }
            _ => unreachable!(),
        }
        match dynamic_format!("234{ac}{ab}", [("ac", "1"), ("aa", ".")]) {
            Err(KeyError { key, pos, .. }) => {
                assert_eq!(key, "ab");
                assert_eq!(pos, 8);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_token_error() {
        match dynamic_format!("{abc", [("abc", "1")]) {
            Err(TokenError { pattern, desc, pos }) => {
                assert_eq!(pattern.as_str(), "{abc");
                assert!(desc.contains("'{'"));
                assert_eq!(pos, 0);
            }
            _ => unreachable!(),
        }
        match dynamic_format!("{{a}}}324", []) {
            Err(TokenError { desc, pos, .. }) => {
                assert!(desc.contains("'}'"));
                assert_eq!(pos, 5);
            }
            _ => unreachable!(),
        }
        match dynamic_format!("{na{me}324", []) {
            Err(TokenError { desc, pos, .. }) => {
                assert!(desc.contains("'{'"));
                assert_eq!(pos, 1);
            }
            _ => unreachable!(),
        }
        match dynamic_format!("name}3}24", []) {
            Err(TokenError { desc, pos, .. }) => {
                assert!(desc.contains("'}'"));
                assert_eq!(pos, 5);
            }
            _ => unreachable!(),
        }
    }
}
