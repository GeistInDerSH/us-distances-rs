use std::fmt;
use std::io::{prelude::*, BufReader};
use std::ops::Div;
use std::{cmp, ops};

const DIAMETER_KM: f32 = 12_742.0;
const KM_TO_MILE_RATIO: f32 = 0.621_371_2;
const DEFAULT_POINT: Point = Point { lat: 0.0, lng: 0.0 };

#[derive(Clone, PartialEq, PartialOrd)]
pub struct Point {
    lat: f32,
    lng: f32,
}

impl Point {
    #[inline]
    fn new(lat: f32, lng: f32) -> Self {
        Self { lat, lng }
    }

    /// The point directly opposite the current [Point] on a sphere
    #[inline]
    fn antipode(&self) -> Point {
        Point {
            lat: self.lat * -1.0,
            lng: self.lng + if self.lng < 0.0 { 180.0 } else { -180.0 },
        }
    }

    /// d = 2R × sin⁻¹(√(sin²((θ₂ - θ₁)/2) + cosθ₁ × cosθ₂ × sin²((φ₂ - φ₁)/2)))
    /// θ₁, φ₁= lat, lng of start
    /// θ₂, φ₂= lat, lng of end
    pub fn haversine_distance(&self, other: &Point) -> f32 {
        let s_lat_rad = self.lat.to_radians();
        let o_lat_rad = other.lat.to_radians();
        let a = (o_lat_rad - s_lat_rad).div(2.0).sin().powi(2);
        let b = (other.lng - self.lng).to_radians().div(2.0).sin().powi(2);
        let cos = s_lat_rad.cos() * o_lat_rad.cos() * b;
        let c = cos + a;
        let d = c.sqrt().asin();
        DIAMETER_KM * d
    }
}

impl fmt::Display for Point {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:0.2}, {:0.2})", self.lat, self.lng)
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
            Ok(lat) => lat,
            Err(_) => return Err("Failed to parse latitude"),
        };
        let lng = match value[space + 1..value.len()].parse::<f32>() {
            Ok(lng) => lng,
            Err(_) => return Err("Failed to parse longitude"),
        };
        Ok(Point::new(lat, lng))
    }
}

#[derive(PartialEq)]
pub struct Farthest {
    opposite: Point,
    closest: Point,
    distance: f32,
}

impl Farthest {
    #[inline]
    fn distance_km(&self) -> f32 {
        self.distance
    }

    #[inline]
    fn distance_mi(&self) -> f32 {
        self.distance * KM_TO_MILE_RATIO
    }
}

impl Default for Farthest {
    #[inline]
    fn default() -> Self {
        Self {
            opposite: DEFAULT_POINT,
            closest: DEFAULT_POINT,
            distance: f32::MIN,
        }
    }
}

impl fmt::Display for Farthest {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let start = self.opposite.antipode();
        write!(
            f,
            "Starting Point: {}\nFarthest Point: {}\nClosest to Farthest: {}\nDistance to Closest: {:0.2}km / {:0.2}mi",
            start, self.opposite, self.closest, self.distance_km(), self.distance_mi()
        )
    }
}

impl PartialOrd for Farthest {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.distance.partial_cmp(&other.distance)
    }
}

pub struct Points(Vec<Point>);

impl Points {
    #[inline]
    pub fn new(points: Vec<Point>) -> Self {
        Self(points)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&Point> {
        self.0.get(index)
    }

    /// Get the [Farthest] by looking at each of the [Point]s in the range of
    /// [start] to [start] + [count], against all other Points.
    pub fn farthest_point_with_offset(&self, start: usize, count: usize) -> Farthest {
        let mut farthest = Farthest::default();
        for current_point in self.0.iter().skip(start).take(count) {
            let opposite = current_point.antipode();
            let (closest, distance) = self
                .0
                .iter()
                .map(|point| (point, opposite.haversine_distance(point)))
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(cmp::Ordering::Equal))
                .unwrap_or((&DEFAULT_POINT, 0.0));
            if distance > farthest.distance {
                farthest.distance = distance;
                farthest.opposite = opposite;
                farthest.closest = closest.clone();
            }
        }
        farthest
    }
}

impl ops::Index<usize> for Points {
    type Output = Point;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

/// Attempt to load [Points] from the name of a file. This operation is buffered, but should
/// consume the whole file before returning.
pub fn try_load_points(file_name: &str) -> std::io::Result<Points> {
    let fp = std::fs::File::open(file_name)?;
    let reader = BufReader::new(fp);
    let points = reader
        .lines()
        .map(|line| line.expect("Failed to read line"))
        .map(|line| Point::try_from(line).expect("Failed to parse point from line"))
        .collect::<_>();
    Ok(Points::new(points))
}
