use std::io::{BufRead, BufReader};
use std::ops::{Index, Mul};
use std::{cmp, fmt};

const DIAMETER_KM: f32 = 12_742.0;
const KM_TO_MILE_RATIO: f32 = 0.621_371_2;

/// A Point on the Earth.
///
/// [latitude] & [longitude] are in degrees.
#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub struct Point {
    latitude: f32,
    longitude: f32,
    latitude_cos: f32,
}

impl Point {
    #[inline]
    const fn new(latitude: f32, longitude: f32, latitude_cos: f32) -> Self {
        Self {
            latitude,
            longitude,
            latitude_cos,
        }
    }

    /// The point directly opposite the current [Point] on a sphere
    #[inline]
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
        Self::new(0.0, 0.0, 1.0)
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

#[derive(PartialEq)]
pub struct Farthest {
    origin: Point,
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
        write!(
            f,
            "Starting Point: {}\nFarthest Point: {}\nClosest to Farthest: {}\nDistance to Closest: {:0.2}km / {:0.2}mi",
            self.origin, self.opposite, self.closest, self.distance_km(), self.distance_mi()
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
    pub const fn new(points: Vec<Point>) -> Self {
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
        let default = Point::default();
        for current_point in self.0.iter().skip(start).take(count) {
            let opposite = current_point.antipode();
            let (closest, distance) = self
                .0
                .iter()
                .map(|point| (point, opposite.haversine_distance(point)))
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(cmp::Ordering::Greater))
                .unwrap_or((&default, 0.0));
            if distance > farthest.distance {
                farthest.origin = current_point.clone();
                farthest.distance = distance;
                farthest.opposite = opposite;
                farthest.closest = closest.clone();
            }
        }
        farthest
    }
}

impl Index<usize> for Points {
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
    let mut points: Vec<Point> = reader
        .lines()
        .map(|line| line.expect("Failed to read line"))
        .map(|line| Point::try_from(line).expect("Failed to parse point from line"))
        .collect::<_>();
    points.sort_by(|lhs, rhs| lhs.longitude.total_cmp(&rhs.longitude));
    Ok(Points::new(points))
}
