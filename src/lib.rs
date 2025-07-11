use kdtree::distance::squared_euclidean;
use kdtree::KdTree;
use std::fmt;
use std::io::{BufRead, BufReader};
use std::ops::{Index, Mul};

const RADIUS_KM: f32 = 6371.0;
const DIAMETER_KM: f32 = RADIUS_KM * 2.0;
const KM_TO_MILE_RATIO: f32 = 0.621_371_2;

/// A Point on the Earth.
///
/// [latitude] & [longitude] are in radians.
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

impl From<&Point3D> for Point {
    fn from(point: &Point3D) -> Self {
        let long = point.y.atan2(point.x);
        let hyp = (point.x.powi(2) + point.y.powi(2)).sqrt();
        let lat = point.z.atan2(hyp);
        Point {
            latitude: lat,
            longitude: long,
            latitude_cos: lat.cos(),
        }
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

pub struct Points(Vec<Point>);

impl Points {
    #[inline]
    pub const fn new(points: Vec<Point>) -> Self {
        Self(points)
    }

    pub fn farthest(&self) -> Farthest {
        let mut kd = KdTree::with_capacity(3, 1 << 7);
        for p in self.0.iter() {
            let p3d = Point3D::from(p);
            let _ = kd.add(p3d.to_array(), p);
        }

        let mut farthest = Farthest::default();
        for point in self.0.iter() {
            let opposite = point.antipode();
            let o_3d = Point3D::from(&opposite);

            let n = kd.nearest(&o_3d.to_array(), 1, &squared_euclidean);
            if n.is_err() {
                continue;
            }
            let closest = n.unwrap()[0].1;
            let dist = opposite.haversine_distance(closest);
            if dist > farthest.distance {
                farthest.distance = dist;
                farthest.origin = point.clone();
                farthest.opposite = opposite;
                farthest.closest = (**closest).clone();
            }
        }

        farthest
    }
}

#[derive(PartialOrd, PartialEq)]
struct Point3D {
    x: f32,
    y: f32,
    z: f32,
}

impl Point3D {
    fn to_array(&self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }
}

impl From<&Point> for Point3D {
    fn from(point: &Point) -> Self {
        Point3D {
            x: RADIUS_KM * point.latitude_cos * point.longitude.cos(),
            y: RADIUS_KM * point.latitude_cos * point.longitude.sin(),
            z: RADIUS_KM * point.latitude.sin(),
        }
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
    let points: Vec<Point> = reader
        .lines()
        .map(|line| line.expect("Failed to read line"))
        .map(|line| Point::try_from(line).expect("Failed to parse point from line"))
        .collect::<_>();
    Ok(Points::new(points))
}
