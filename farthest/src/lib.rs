use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Mul;

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
    latitude: i16,
    longitude: i16,
}

impl Point {
    #[inline]
    pub const fn new(latitude: f32, longitude: f32) -> Self {
        Self {
            latitude: (latitude * 1000.0) as i16,
            longitude: (longitude * 1000.0) as i16,
        }
    }

    fn lat(&self) -> f32 {
        self.latitude as f32 / 1000.0
    }

    fn long(&self) -> f32 {
        self.longitude as f32 / 1000.0
    }

    /// The point directly opposite the current [Point] on a sphere
    #[inline]
    pub fn antipode(&self) -> Point {
        const PI: i16 = (std::f32::consts::PI * 1000.) as i16;

        Point {
            latitude: -self.latitude,
            longitude: self.longitude + if self.longitude < 0 { PI } else { -PI },
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
        let a = 0.5 - (other.lat() - self.lat()).cos().mul(0.5);
        let b = 0.5 - (other.long() - self.long()).cos().mul(0.5);
        let cos = self.lat().cos() * other.lat().cos() * b;
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
            self.lat().to_degrees(),
            self.long().to_degrees()
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
    pub origin: Point,
    pub opposite: Point,
    pub closest: Point,
    pub distance: DistanceKm,
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
        let distance_km = self.distance;
        let distance_mi: DistanceMi = distance_km * KM_TO_MILE_RATIO;
        write!(
            f,
            "Starting Point: {}\nFarthest Point: {}\nClosest to Farthest: {}\nDistance to Closest: {:0.2}km / {:0.2}mi",
            self.origin, self.opposite, self.closest, distance_km, distance_mi
        )
    }
}

#[derive(PartialOrd, PartialEq)]
pub struct Point3D(pub [f32; 3]);

impl From<&Point> for Point3D {
    fn from(point: &Point) -> Self {
        let x = RADIUS_KM * point.lat().cos() * point.long().cos();
        let y = RADIUS_KM * point.lat().cos() * point.long().sin();
        let z = RADIUS_KM * point.lat().sin();
        Point3D([x, y, z])
    }
}

pub fn encoding_config()
-> bincode::config::Configuration<bincode::config::LittleEndian, bincode::config::Fixint> {
    bincode::config::standard().with_fixed_int_encoding()
}
