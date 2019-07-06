use liberty_parse::{Error, GroupItem, Parser};
use std::{env, fs, result};

fn parse<'a>(parser: &'a Parser) -> result::Result<(), Error<'a>> {
    for lib in parser.parse()? {
        match lib {
            GroupItem::Group(_group_type, name, groups) => {
                println!("Parsed library '{}'", name);
                for group in groups {
                    match group {
                        GroupItem::Group(type_, name, _) => {
                            if type_ == "cell" {
                                println!("Cell: {}", name);
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
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
    let parser = Parser::new(&contents);
    parse(&parser);
}
