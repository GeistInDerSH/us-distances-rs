use usdist::try_load_points;

fn main() {
    let all_points = try_load_points("points.txt").expect("Failed to load points from the file");
    let farthest = all_points.farthest();
    println!("{}", farthest);
}
