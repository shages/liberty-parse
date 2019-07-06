# liberty-parse

Liberty file format parser for Rust

## Example usage

Parse libraries from a Liberty file

```rust
use liberty_parse::parse_lib;

let lib_str = r#"
library(sample) {
    cell(AND2) {
        area: 1;
    }
}
"#;

for lib in parse_lib(lib_str).unwrap() {
    println!("Library '{}' has {} cells", lib.name, lib.cells.len());
    if let Some(cell) = lib.cells.get("AND2") {
        let area = cell.simple_attributes.get("area").map_or(0.0, |v| v.float());
        println!("Cell AND2 has area: {}", area);
    } else {
        println!("Cell AND2 doesn't exist!");
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

