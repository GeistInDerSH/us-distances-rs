use std::sync::atomic::AtomicUsize;
use std::sync::mpsc::channel;
use std::sync::{atomic, Arc};
use std::{cmp, thread};
use usdist::{try_load, Farthest, Points};

const GROUP_SIZE: usize = 128;

fn main() {
    let points = try_load("points.txt").expect("Failed to load points from the file");
    let all_points = Arc::new(Points::new(points));
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
                let point = all_points.farthest_point_with_offset(start, GROUP_SIZE);
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
