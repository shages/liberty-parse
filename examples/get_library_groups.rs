use liberty_parse;
use nom::{
    error::{convert_error, VerboseError},
    Err,
};

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let contents = fs::read_to_string(filename).expect("Unable to read LIB file");

    match liberty_parse::parse_libs::<VerboseError<&str>>(&contents) {
        Ok((_, libraries)) => {
            println!("Found {} libraries", libraries.len());
            for lib in libraries {
                match lib {
                    liberty_parse::GroupItem::Group(_group_type, name, items) => {
                        let groups: Vec<_> = items
                            .iter()
                            .filter(|i| match i {
                                liberty_parse::GroupItem::Group(_, _, _) => true,
                                _ => false,
                            })
                            .collect();
                        println!("Library '{}' has {} groups", name, groups.len());
                    }
                    _ => {}
                }
            }
        }
        Err(Err::Error(err)) | Err(Err::Failure(err)) => {
            println!("{}", convert_error(&contents, err));
        }
        _ => {}
    }
}
