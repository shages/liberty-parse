use liberty_parse::parse_lib;
use std::io::{stdin, Read};

fn main() {
    let mut buf = String::new();
    stdin().read_to_string(&mut buf).unwrap();

    let lib = parse_lib(&buf).unwrap();
    println!("{}", lib);
}
