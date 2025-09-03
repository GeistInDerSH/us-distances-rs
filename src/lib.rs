use bincode::{Decode, Encode};
use kdtree::distance::squared_euclidean;
use kdtree::KdTree;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::io::Write;
use std::ops::Mul;
use std::path::Path;

type DistanceKm = f32;
type DistanceMi = f32;

const RADIUS_KM: DistanceKm = 6371.0;
const DIAMETER_KM: DistanceKm = RADIUS_KM * 2.0;
const KM_TO_MILE_RATIO: f32 = 0.621_371_2;

/// A Point on the Earth.
///
/// [latitude] & [longitude] are in radians.
#[derive(Clone, PartialEq, PartialOrd, Copy, Encode, Decode, Serialize, Deserialize)]
pub struct Point {
    latitude: f32,
    longitude: f32,
}

impl Point {
    #[inline]
    const fn new(latitude: f32, longitude: f32) -> Self {
        Self {
            latitude,
            longitude,
        }
    }

    /// The point directly opposite the current [Point] on a sphere
    #[inline]
    fn antipode(&self) -> Point {
        const PI: f32 = std::f32::consts::PI;

        Point {
            latitude: -self.latitude,
            longitude: self.longitude + if self.longitude < 0.0 { PI } else { -PI },
        }
    }

    /// Calculate the [Haversine Distance](https://en.wikipedia.org/wiki/Haversine_formula) between
    /// the current [Point] and the other Point.
    /// Returns the distance, in KM, between the two points on earth.
    ///
    /// ```markdown
    /// d = 2R × sin⁻¹(√(sin²((θ₂ - θ₁)/2) + cos(θ₁) × cos(θ₂) × sin²((φ₂ - φ₁)/2)))
    /// θ₁, φ₁= lat, lng of start
    /// θ₂, φ₂= lat, lng of end
    /// ```
    pub fn haversine_distance(&self, other: &Point) -> DistanceKm {
        let a = 0.5 - (other.latitude - self.latitude).cos().mul(0.5);
        let b = 0.5 - (other.longitude - self.longitude).cos().mul(0.5);
        let cos = self.latitude.cos() * other.latitude.cos() * b;
        let c = a + cos;
        let d = c.sqrt().asin();
        DIAMETER_KM * d
    }
}

impl fmt::Display for Point {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({:0.2}, {:0.2})",
            self.latitude.to_degrees(),
            self.longitude.to_degrees()
        )
    }
}

impl Default for Point {
    fn default() -> Self {
        Self::new(0.0, 0.0)
    }
}

impl TryFrom<&str> for Point {
    type Error = &'static str;

    /// Try to convert the string to a [Point]. Valid lines are two floats separated by a space
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut parts = value.split_whitespace();
        let lat = parts
            .next()
            .ok_or("Invalid point; Missing latitude")?
            .parse::<f32>()
            .map_err(|_| "Invalid point; Invalid latitude")?;
        let lng = parts
            .next()
            .ok_or("Invalid point; Missing latitude")?
            .parse::<f32>()
            .map_err(|_| "Invalid point; Invalid latitude")?;
        Ok(Point::new(lat, lng))
    }
}

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct Farthest {
    origin: Point,
    opposite: Point,
    closest: Point,
    distance: DistanceKm,
}

impl Default for Farthest {
    #[inline]
    fn default() -> Self {
        Self {
            origin: Default::default(),
            opposite: Point::default(),
            closest: Point::default(),
            distance: f32::MIN,
        }
    }
}

impl fmt::Display for Farthest {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let distance_km = self.opposite.haversine_distance(&self.closest);
        let distance_mi: DistanceMi = distance_km * KM_TO_MILE_RATIO;
        write!(
            f,
            "Starting Point: {}\nFarthest Point: {}\nClosest to Farthest: {}\nDistance to Closest: {:0.2}km / {:0.2}mi",
            self.origin, self.opposite, self.closest, distance_km, distance_mi
        )
    }
}

pub struct Points {
    distances: Vec<Farthest>,
}

impl Points {
    #[inline]
    pub fn new(distances: Vec<Farthest>) -> Self {
        Self { distances }
    }

    pub fn farthest(&self) -> Farthest {
        *self
            .distances
            .iter()
            .max_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap())
            .unwrap_or(&Farthest::default())
    }
}

#[derive(PartialOrd, PartialEq)]
struct Point3D([f32; 3]);

impl From<&Point> for Point3D {
    fn from(point: &Point) -> Self {
        let x = RADIUS_KM * point.latitude.cos() * point.longitude.cos();
        let y = RADIUS_KM * point.latitude.cos() * point.longitude.sin();
        let z = RADIUS_KM * point.latitude.sin();
        Point3D([x, y, z])
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

pub fn try_load_precalculated_distance_data(distance_data: &[u8]) -> Points {
    let kd: Vec<Farthest> =
        bincode::serde::decode_from_slice(distance_data, bincode::config::standard())
            .unwrap()
            .0;
    Points::new(kd)
}

pub struct Save {
    points: Vec<Point>,
    tree: KdTree<f32, Point, [f32; 3]>,
}

impl Save {
    #[inline]
    pub fn new(points: Vec<Point>, tree: KdTree<f32, Point, [f32; 3]>) -> Self {
        Self { points, tree }
    }

    pub fn save(&self) {
        let calculated = self
            .points
            .iter()
            .map(|point| {
                let opposite = point.antipode();
                let p3d = Point3D::from(&opposite);
                let nearest = self.tree.nearest(&p3d.0, 1, &squared_euclidean);
                let (dist, closest) = nearest.unwrap()[0];
                Farthest {
                    origin: *point,
                    opposite,
                    closest: *closest,
                    distance: dist,
                }
            })
            .collect::<Vec<_>>();
        let slice =
            bincode::serde::encode_to_vec(&calculated, bincode::config::standard()).unwrap();
        let mut fd = std::fs::File::create(Path::new("data/generated.bin")).unwrap();
        fd.write_all(&slice).unwrap();
    }
}
