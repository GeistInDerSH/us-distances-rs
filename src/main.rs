use usdist::{farthest, try_load_points};

fn main() {
    let all_points = try_load_points("points.txt").expect("Failed to load points from the file");
    let farthest = farthest(all_points);
    println!("{farthest}");
}
