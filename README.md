# liberty-parse

Liberty file format parser

## Example usage

Parse a liberty file into the Library

```rust
use liberty_parse;

let lib_str = "...";
let library = liberty_parse::parse_libs(lib_str);

println!("Found {} libraries", library.len());
for lib in library {
    match lib {
        liberty_parse::Item::Group(name, items) => {
            let groups = items
                .iter()
                .filter(|i| match i {
                    liberty_parse::Item::Group => true,
                    _ => false
                })
                .collect();
            println!("Library {} has {} groups", name, groups.len());
        }
        _ => {}
    }
}
```

## Limitations

- Doesn't automatically parse files from `include` statements

## Use cases

- (1) Read selective information for scripting: Cell names, cell attributes (ex: function, footprint, etc.), cell/pin names
- (2) Read most information for timing and power calculations (better stored in a database format?)
- (N - not focus) Construct entire Liberty file: Write Liberty from characterization or design database
- (N - niche use) Read, modify, and write-back: Scale values, or similar?

