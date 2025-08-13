use usdist::try_load_points_bin_data;

fn main() {
    // let all_points = try_load_points("points.txt").expect("Failed to load points from the file");
    // all_points.save();
    let data = include_bytes!("../points.bincode");
    let all_points = try_load_points_bin_data(data);
    let farthest = all_points.farthest();
    println!("{farthest}");
}
