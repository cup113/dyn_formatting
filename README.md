# Dynamic Formatting README

Simple, dynamic, Python-styled string formatting (Only support `String`,
`{key}` patterns).

Escape like `{{` or `}}`.

## Install

```bash
cargo add dyn_formatting@2.0.0
```

## Errors

1. Raise `DynamicFormatError::KeyError` while the key in the brackets
   is not found in `dictionary`.
2. Raise `DynamicFormatError::TokenError` while there is any
   unmatched bracket (`{` or `}`)

## Examples

```Rust
use dyn_formatting::dynamic_format;
assert_eq!(
    dynamic_format(
        "I'm {name}. I'm {age} years old now.",
        &[("name", "ABC"), ("age", "20")].into()
    ).unwrap(),
    "I'm ABC. I'm 20 years old now.".to_string()
);
```

```Rust
use dyn_formatting::dynamic_format;
use std::collections::HashMap;
let value_age = (15).to_string(); // Make lifetime long enough
let dictionary = HashMap::from([
    ("age", value_age.as_str()),
]);
assert_eq!(
    dynamic_format("{{{age} }}{age}", &dictionary).unwrap(),
    "{15 }15"
)
```

```Rust
use dyn_formatting::dynamic_format;
assert!(
    dynamic_format(
        "I'm {name}. I'm {age} years old now.",
        &[("name", "ABC")].into()
    ).is_err() // Key error
);
```

```Rust
use dyn_formatting::dynamic_format;
assert!(
    dynamic_format(
        "I'm {name{name}}.",
        &[("name", "ABC")].into()
    ).is_err() // Token error: '{' unmatched.
);
```
