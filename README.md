# Dynamic Formatting README

Simple, dynamic, Python-styled string formatting (Only support `String`,
`{key}` patterns).

Escape like `{{` or `}}`.

## Errors

1. Raise `DynamicFormatError::KeyError` while the key in the brackets
   is not found in `dictionary`.
2. Raise `DynamicFormatError::TokenError` while there is any
   unmatched bracket (`{` or `}`)

## Examples

```Rust
use dyn_formatting::dynamic_format;
assert_eq!(
    dynamic_format("I'm {name}", &[("name", "ABC")].into()).unwrap(),
    "I'm ABC".to_string()
);
```
