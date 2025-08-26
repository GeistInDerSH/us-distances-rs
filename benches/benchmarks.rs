#[macro_use]
extern crate criterion;
use criterion::Criterion;
use usdist::{try_load_points, try_load_points_bin_data, try_load_points_bin_data_tree};

fn criterion_benchmark(c: &mut Criterion) {
    let points = try_load_points("data/points.txt").expect("Failed to load points from the file");
    c.bench_function("farthest", |b| b.iter(|| points.farthest()));
    c.bench_function("try_load", |b| {
        b.iter(|| try_load_points("data/points.txt"))
    });
    c.bench_function("try_load_points_bin_data", |b| {
        let points = include_bytes!("../data/points.bin");
        b.iter(|| try_load_points_bin_data(points))
    });
    c.bench_function("try_load_points_bin_data_tree", |b| {
        let points = include_bytes!("../data/points.bin");
        let tree = include_bytes!("../data/tree.bin");
        b.iter(|| try_load_points_bin_data_tree(points, tree))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
