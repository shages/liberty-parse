use liberty_parse::{parse_lib, Error};
use std::{env, fs, result};

fn parse<'a>(contents: &'a str) -> result::Result<(), Error<'a>> {
    for lib in parse_lib(contents)? {
        println!("Parsed library '{}'", lib.name);
        for cell in lib.iter_cells() {
            println!("Cell: {}", cell.name);
            if let Some(area) = cell.simple_attribute("area") {
                println!("Cell has area: {:?}", area.float());
            }
        }
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Missing LIB file argument");
    }
    let contents = fs::read_to_string(&args[1]).expect("Unable to read LIB file");
    parse(&contents).unwrap();
}
