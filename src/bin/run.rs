use usdist::try_load_points_bin_data_tree;

fn main() {
    let data = include_bytes!("../../data/points.bin");
    let tree = include_bytes!("../../data/tree.bin");
    let all_points = try_load_points_bin_data_tree(data, tree);
    let farthest = all_points.farthest();
    println!("{farthest}");
}
