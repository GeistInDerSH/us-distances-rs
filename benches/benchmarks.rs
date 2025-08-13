#[macro_use]
extern crate criterion;
use criterion::Criterion;
use usdist::try_load_points;

fn criterion_benchmark(c: &mut Criterion) {
    let points = try_load_points("data/points.txt").expect("Failed to load points from the file");
    c.bench_function("farthest", |b| b.iter(|| points.farthest()));
    c.bench_function("try_load", |b| b.iter(|| try_load_points("points.txt")));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
