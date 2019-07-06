use liberty_parse::parse_lib;

fn main() {
    let lib_str = r#"
library(sample) {
    cell(AND2) {
        area: 1;
        pin(o) {
            timing() {
                cell_rise(delay_temp_3x3) {
                    index_1 ("0.5, 1.0, 1.5");
                    index_2 ("10.0, 20.0, 30.0");
                    values ( "0.1, 0.2, 0.3", \
                                "0.11, 0.21, 0.31", \
                                "0.12, 0.22, 0.32" );
                }
            }
        }
    }
}
"#;

    for lib in parse_lib(lib_str).unwrap() {
        println!("Library '{}' has {} cells", lib.name, lib.cells.len());
        let area = lib
            .cells
            .get("AND2")
            .and_then(|c| c.simple_attributes.get("area"))
            .map_or(0.0, |v| v.float());
        println!("Cell AND2 has area: {}", area);

        let values = lib
            .cells
            .get("AND2")
            .and_then(|c| c.pins.get("o"))
            .and_then(|p| p.groups.iter().find(|g| g.type_ == "timing"))
            .and_then(|t| t.groups.iter().find(|g| g.type_ == "cell_rise"))
            .and_then(|rise| rise.complex_attributes.get("values"))
            .map_or(vec![], |values| {
                values.into_iter().map(|v| v.float_group()).collect()
            });
        println!("Pin AND2/o has cell_rise values: {:?}", values);
    }
}
