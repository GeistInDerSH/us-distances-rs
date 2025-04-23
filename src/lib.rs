use rayon::prelude::*;
use std::cmp::Ordering;
use std::fmt;
use std::io::{BufRead, BufReader};
use std::ops::Mul;

const DIAMETER_KM: f32 = 12_742.0;
const KM_TO_MILE_RATIO: f32 = 0.621_371_2;

/// A Point on the Earth.
///
/// [latitude] & [longitude] are in degrees.
#[derive(Clone, Copy)]
pub struct Point {
    latitude: f32,
    longitude: f32,
    latitude_cos: f32,
}

impl Point {
    const fn new(latitude: f32, longitude: f32, latitude_cos: f32) -> Self {
        Self {
            latitude,
            longitude,
            latitude_cos,
        }
    }

    /// The point directly opposite the current [Point] on a sphere
    fn antipode(&self) -> Point {
        const PI: f32 = std::f32::consts::PI;

        Point {
            latitude: self.latitude * -1.0,
            longitude: self.longitude + if self.longitude < 0.0 { PI } else { -PI },
            latitude_cos: self.latitude_cos,
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
    pub fn haversine_distance(&self, other: &Point) -> f32 {
        let a = 0.5 - (other.latitude - self.latitude).cos().mul(0.5);
        let b = 0.5 - (other.longitude - self.longitude).cos().mul(0.5);
        let cos = self.latitude_cos * other.latitude_cos * b;
        let c = a + cos;
        let d = c.sqrt().asin();
        DIAMETER_KM * d
    }
}

impl TryFrom<String> for Point {
    type Error = &'static str;

    /// Try to convert the string to a [Point]. Valid lines are two floats separated by a space
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let space = match value.find(' ') {
            None => return Err("Invalid point; Missing space"),
            Some(i) => i,
        };
        let lat = match value[0..space].parse::<f32>() {
            Ok(lat) => lat.to_radians(),
            Err(_) => return Err("Failed to parse latitude"),
        };
        let lng = match value[space + 1..value.len()].parse::<f32>() {
            Ok(lng) => lng.to_radians(),
            Err(_) => return Err("Failed to parse longitude"),
        };
        Ok(Point::new(lat, lng, lat.cos()))
    }
}

pub struct Farthest {
    origin_latitude: f32,
    origin_longitude: f32,
    opposite_latitude: f32,
    opposite_longitude: f32,
    end_latitude: f32,
    end_longitude: f32,
    distance: f32,
}

impl Farthest {
    fn distance_km(&self) -> f32 {
        self.distance
    }

    fn distance_mi(&self) -> f32 {
        self.distance * KM_TO_MILE_RATIO
    }
}

impl fmt::Display for Farthest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Starting Point: ({:0.2}, {:0.2})\nFarthest Point: ({:0.2}, {:0.2})\nClosest to Farthest: ({:0.2}, {:0.2})\nDistance to Closest: {:0.2}km / {:0.2}mi",
            self.origin_latitude.to_degrees(),
            self.origin_longitude.to_degrees(),
            self.opposite_latitude.to_degrees(),
            self.opposite_longitude.to_degrees(),
            self.end_latitude.to_degrees(),
            self.end_longitude.to_degrees(),
            self.distance_km(),
            self.distance_mi()
        )
    }
}

pub fn farthest(points: Vec<Point>) -> Farthest {
    let (origin, end, distance) = points
        .par_iter()
        .map(|origin| {
            let opposite = origin.antipode();
            let (end, dist) = points
                .iter()
                .map(|other| (other, opposite.haversine_distance(other)))
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Greater))
                .unwrap();
            (origin, end, dist)
        })
        .max_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(Ordering::Less))
        .unwrap();

    let opposite = origin.antipode();
    Farthest {
        origin_latitude: origin.latitude,
        origin_longitude: origin.longitude,
        opposite_longitude: opposite.longitude,
        opposite_latitude: opposite.latitude,
        end_latitude: end.latitude,
        end_longitude: end.longitude,
        distance,
    }
}

/// Attempt to load [Points] from the name of a file. This operation is buffered, but should
/// consume the whole file before returning.
pub fn try_load_points(file_name: &str) -> std::io::Result<Vec<Point>> {
    let fp = std::fs::File::open(file_name)?;
    let reader = BufReader::new(fp);
    let mut points: Vec<Point> = reader
        .lines()
        .map(Result::unwrap)
        .map(Point::try_from)
        .map(Result::unwrap)
        .collect::<_>();
    points.sort_by(|lhs, rhs| lhs.longitude.total_cmp(&rhs.longitude));
    Ok(points)
}
