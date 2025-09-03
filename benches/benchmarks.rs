#[macro_use]
extern crate criterion;

use criterion::Criterion;
use usdist::try_load_precalculated_distance_data;

fn criterion_benchmark(c: &mut Criterion) {
    let generated = include_bytes!("../data/generated.bin");
    let points = try_load_precalculated_distance_data(generated);
    c.bench_function("farthest", |b| b.iter(|| points.farthest()));
    c.bench_function("try_load_precalculated_distance_data", |b| {
        b.iter(|| try_load_precalculated_distance_data(generated))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
