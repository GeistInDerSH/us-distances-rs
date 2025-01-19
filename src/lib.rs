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

    #[inline]
    fn antipode(&self) -> Point {
        Point {
            lat: self.lat * -1.0,
            lng: self.lng + if self.lng < 0.0 { 180.0 } else { -180.0 },
        }
    }

    pub fn haversine_distance(&self, other: &Point) -> f32 {
        let a = (other.lat - self.lat).to_radians().div(2.0).sin().powi(2);
        let b = (other.lng - self.lng).to_radians().div(2.0).sin().powi(2);
        let cos = self.lat.to_radians().cos() * other.lat.to_radians().cos() * b;
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

pub fn try_load(file_name: &str) -> std::io::Result<Vec<Point>> {
    let fp = std::fs::File::open(file_name)?;
    let reader = BufReader::new(fp);
    Ok(reader
        .lines()
        .map(|line| line.unwrap())
        .map(|line| Point::try_from(line).unwrap())
        .collect::<_>())
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

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&Point> {
        self.0.get(index)
    }

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

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}
