use std::sync::atomic::AtomicUsize;
use std::sync::mpsc::channel;
use std::sync::{atomic, Arc};
use std::{cmp, thread};
use usdist::try_load_points;

const GROUP_SIZE: usize = 8;

fn main() {
    let all_points =
        Arc::new(try_load_points("points.txt").expect("Failed to load points from the file"));
    let number_of_points = all_points.len();

    let (sender, receiver) = channel();
    let shared_sender = Arc::new(sender);

    // Share a read offset, and read the GROUP_SIZE for each farthest point.
    // This allows for work-stealing between threads
    let read_index = Arc::new(AtomicUsize::new(0));
    let cores = match thread::available_parallelism() {
        Ok(n) => n.get(),
        Err(_) => 1,
    };
    let threads = (0..cores)
        .map(|_| {
            let sender = shared_sender.clone();
            let all_points = all_points.clone();
            let read_index = read_index.clone();
            thread::spawn(move || loop {
                let start = read_index.fetch_add(GROUP_SIZE, atomic::Ordering::SeqCst);
                if start > number_of_points {
                    return;
                }
                let point = all_points.farthest_point_with_offset(start, GROUP_SIZE);
                sender
                    .send(point)
                    .expect("Failed to send farthest point; Channel closed?");
            })
        })
        .collect::<Vec<_>>();
    drop(shared_sender); // Drop the parent sender channel

    let farthest = receiver
        .into_iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(cmp::Ordering::Less))
        .expect("Failed to find the farthest point");
    println!("{farthest}");

    // The actual joins should be a no-op as all threads should already be finished
    // because all sender channels have been sent
    for thread in threads {
        thread.join().expect("Failed to join thread");
    }
}
