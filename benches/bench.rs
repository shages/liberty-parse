#[macro_use]
extern crate criterion;

use liberty_parse::parse_libs;
use nom::error::ErrorKind;

use criterion::black_box;
use criterion::Criterion;

macro_rules! my_bench_file {
    ($fname:ident) => {
        fn $fname(c: &mut Criterion) {
            let data = include_str!(concat!("../data/", stringify!($fname), ".lib"));
            c.bench_function(stringify!($fname), move |b| {
                b.iter(|| parse_libs::<(&str, ErrorKind)>(data))
            });
        }
    };
}

my_bench_file!(small);
my_bench_file!(cells);
my_bench_file!(cells_timing);

criterion_group!(benches, small, cells, cells_timing);
criterion_main!(benches);
