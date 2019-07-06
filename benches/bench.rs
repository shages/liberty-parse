#[macro_use]
extern crate criterion;

use liberty_parse::Parser;

use criterion::Criterion;

macro_rules! my_bench_file {
    ($fname:ident) => {
        fn $fname(c: &mut Criterion) {
            let data = include_str!(concat!("../data/", stringify!($fname), ".lib"));
            c.bench_function(stringify!($fname), move |b| {
                b.iter(|| match Parser::new(data).parse() {
                    _ => {}
                })
            });
        }
    };
}

my_bench_file!(small);
my_bench_file!(cells);
my_bench_file!(cells_timing);

criterion_group!(benches, small, cells, cells_timing);
criterion_main!(benches);
