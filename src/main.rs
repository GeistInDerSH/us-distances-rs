use std::cmp;
use std::io::{prelude::*, BufReader};
use std::ops::Div;
use std::sync::atomic::AtomicUsize;
use std::sync::mpsc::channel;
use std::sync::{atomic, Arc};
use std::{fmt, thread};

const DIAMETER_KM: f32 = 12_742.0;
const KM_TO_MILE_RATIO: f32 = 0.621_371_2;
const GROUP_SIZE: usize = 128;
const DEFAULT_POINT: Point = Point { lat: 0.0, lng: 0.0 };

#[derive(Clone, PartialEq, PartialOrd)]
struct Point {
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

    fn haversine_distance(&self, other: &Point) -> f32 {
        let a = (other.lat - self.lat).to_radians().div(2.0).sin().powi(2);
        let b = (other.lng - self.lng).to_radians().div(2.0).sin().powi(2);
        let cos = self.lat.to_radians().cos() * other.lat.to_radians().cos() * b;
        let c = cos + a;
        let c_inv = 1.0 - c;
        let d = c.sqrt().atan2(c_inv.sqrt());
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

#[derive(PartialEq)]
struct Farthest {
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

fn try_load(file_name: &str) -> std::io::Result<Vec<Point>> {
    let fp = std::fs::File::open(file_name)?;
    let reader = BufReader::new(fp);
    Ok(reader
        .lines()
        .map(|line| line.unwrap())
        .map(|line| Point::try_from(line).unwrap())
        .collect::<_>())
}

fn min_distance_for_point(point: &Point, all_points: &[Point]) -> Farthest {
    let opposite = point.antipode();
    let (closest, distance) = all_points
        .iter()
        .map(|point| (point, opposite.haversine_distance(point)))
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(cmp::Ordering::Equal))
        .unwrap_or((&DEFAULT_POINT, 0.0));
    Farthest {
        opposite,
        distance,
        closest: closest.clone(),
    }
}

fn main() {
    let points = try_load("points.txt").expect("Failed to load points from the file");
    let all_points = Arc::new(points);
    let number_of_points = all_points.len();

    let (sender, receiver) = channel::<Farthest>();
    let shared_sender = Arc::new(sender);

    let read_index = Arc::new(AtomicUsize::new(0));
    let cores = thread::available_parallelism().unwrap().get();
    let threads = (0..cores)
        .map(|_| {
            let chan = shared_sender.clone();
            let all_points = all_points.clone();
            let read_index = read_index.clone();
            thread::spawn(move || loop {
                let start = read_index.fetch_add(GROUP_SIZE, atomic::Ordering::SeqCst);
                if start > number_of_points {
                    return;
                }
                let point = all_points
                    .iter()
                    .skip(start)
                    .take(GROUP_SIZE)
                    .map(|point| min_distance_for_point(point, &all_points))
                    .max_by(|a, b| a.partial_cmp(b).unwrap_or(cmp::Ordering::Less))
                    .unwrap_or(Farthest::default());

                chan.send(point).unwrap();
            })
        })
        .collect::<Vec<_>>();
    drop(shared_sender);

    let farthest = receiver
        .into_iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(cmp::Ordering::Equal))
        .unwrap();
    println!("{farthest}");

    for thread in threads {
        thread.join().unwrap();
    }
}
