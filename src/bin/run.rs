use usdist::try_load_precalculated_distance_data;

fn main() {
    let distance = include_bytes!("../../data/generated.bin");
    let all_points = try_load_precalculated_distance_data(distance);
    let farthest = all_points.farthest();
    println!("{farthest}");
}
