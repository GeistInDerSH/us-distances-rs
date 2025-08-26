use usdist::try_load_points;

fn main() {
    let all_points =
        try_load_points("data/points.txt").expect("Failed to load points from the file");
    all_points.save();
}
