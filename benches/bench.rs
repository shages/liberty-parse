#[macro_use]
extern crate criterion;

use liberty_parse::ast::LibertyAst;
use liberty_parse::parse_lib;

use criterion::Criterion;

macro_rules! my_bench_file_ast {
    ($benchname:ident, $fname:ident) => {
        fn $benchname(c: &mut Criterion) {
            let data = include_str!(concat!("../data/", stringify!($fname), ".lib"));
            c.bench_function(stringify!($benchname), move |b| {
                b.iter(|| match LibertyAst::from_string(data).unwrap() {
                    _ => {}
                })
            });
        }
    };
}

macro_rules! my_bench_file {
    ($fname:ident) => {
        fn $fname(c: &mut Criterion) {
            let data = include_str!(concat!("../data/", stringify!($fname), ".lib"));
            c.bench_function(stringify!($fname), move |b| {
                b.iter(|| match parse_lib(data).unwrap() {
                    _ => {}
                })
            });
        }
    };
}

my_bench_file_ast!(ast_small, small);
my_bench_file_ast!(ast_cells, cells);
my_bench_file_ast!(ast_cells_timing, cells_timing);

my_bench_file!(small);
my_bench_file!(cells);
my_bench_file!(cells_timing);

criterion_group!(
    benches,
    small,
    cells,
    cells_timing,
    ast_small,
    ast_cells,
    ast_cells_timing
);
criterion_main!(benches);
