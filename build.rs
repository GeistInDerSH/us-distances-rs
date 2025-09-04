use farthest::{encoding_config, Farthest, Point, Point3D};
use kdtree::distance::squared_euclidean;
use kdtree::KdTree;
use std::io;
use std::io::Write;
use std::path::Path;

pub struct Save {
    points: Vec<Point>,
    tree: KdTree<f32, Point, [f32; 3]>,
}

impl Save {
    #[inline]
    pub fn new(points: Vec<Point>, tree: KdTree<f32, Point, [f32; 3]>) -> Self {
        Self { points, tree }
    }

    pub fn save(&self) -> io::Result<()> {
        let mut calculated = self
            .points
            .iter()
            .map(|point| {
                let opposite = point.antipode();
                let p3d = Point3D::from(&opposite);
                let nearest = self.tree.nearest(&p3d.0, 1, &squared_euclidean);
                let closest = nearest.unwrap()[0].1;
                let distance = opposite.haversine_distance(closest);
                Farthest {
                    origin: *point,
                    closest: *closest,
                    opposite,
                    distance,
                }
            })
            .collect::<Vec<_>>();
        calculated.sort_by(|a, b| b.distance.partial_cmp(&a.distance).unwrap());
        let slice = match bincode::serde::encode_to_vec(&calculated, encoding_config()) {
            Ok(vec) => vec,
            Err(_) => Err(io::Error::from(io::ErrorKind::Other))?,
        };
        let mut fd = std::fs::File::create(Path::new("data/generated.bin"))?;
        fd.write_all(&slice)
    }
}

/// Attempt to load [Points] from the name of a file. This operation is buffered, but should
/// consume the whole file before returning.
pub fn try_load_points(file_name: &str) -> std::io::Result<Save> {
    let contents = std::fs::read_to_string(file_name)?;
    let mut kd = KdTree::with_capacity(3, 1 << 7);
    let mut points = Vec::with_capacity(12_033);
    for line in contents.lines() {
        let point = Point::try_from(line).expect("Failed to parse point from line");
        points.push(point);
        let point_3d = Point3D::from(&point);
        let _ = kd.add(point_3d.0, point);
    }
    Ok(Save::new(points, kd))
}

fn main() -> io::Result<()> {
    let points = try_load_points("data/points.txt")?;
    points.save()
}
