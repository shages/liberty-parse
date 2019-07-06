# liberty-parse

Liberty file format parser for Rust

## Example usage

Parse libraries from a Liberty file

```rust
use liberty_parse::{Parser, GroupItem};

let lib_str = r#"
library(sample) {
    cell(AND2) {
        area: 1;
    }
}
"#;

for lib in Parser::new(lib_str).parse()? {
    match lib {
        GroupItem::Group(type_, name, items) => {
            println!(
                "Library '{}' has {} cells", 
                name, 
                items
                    .iter()
                    .filter(|g| match g {
                        GroupItem::Group(type_, _ ,_) => type_ == "cell",
                        _ => false
                    })
                    .count()
            );
        }
        _ => {}
    }
}
```

## Limitations

- Doesn't automatically parse files from `include` statements
- Doesn't parse bus syntax in pin names. For example:

  ```
      pin (X[0:3]){
      }
  ```

