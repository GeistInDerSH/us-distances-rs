#[macro_use]
extern crate criterion;
use criterion::Criterion;
use usdist::{min_distance_for_point, try_load};

fn criterion_benchmark(c: &mut Criterion) {
    let points = try_load("points.txt").expect("Failed to load points from the file");
    c.bench_function("haversine_distance", |b| {
        b.iter(|| points[0].haversine_distance(&points[1024]))
    });
    c.bench_function("min_distance_for_point", |b| {
        b.iter(|| min_distance_for_point(&points[0], &points))
    });
    c.bench_function("try_load", |b| b.iter(|| try_load("points.txt")));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
