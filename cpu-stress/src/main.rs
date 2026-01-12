use rayon::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

fn main() {
    println!("🔥 CPU STRESS TEST - Using all 24 cores at 100%");

    rayon::ThreadPoolBuilder::new()
        .num_threads(24)
        .build_global()
        .unwrap();

    let counter = AtomicU64::new(0);
    let start = Instant::now();

    // Spawn 24 threads doing intensive work
    (0..24).into_par_iter().for_each(|thread_id| {
        println!("🚀 Thread {} starting intensive work", thread_id);

        loop {
            // CPU-intensive operations
            let mut hash = 0u64;
            for i in 0..1_000_000 {
                hash = hash.wrapping_mul(31).wrapping_add(i);
                hash ^= hash >> 16;
                hash = hash.wrapping_mul(0x85ebca6b);
                hash ^= hash >> 13;
                hash = hash.wrapping_mul(0xc2b2ae35);
                hash ^= hash >> 16;
            }

            let count = counter.fetch_add(1, Ordering::Relaxed);

            if count % 1000 == 0 {
                let elapsed = start.elapsed().as_secs();
                println!("Thread {}: {} iterations, {}s elapsed, hash: {}",
                    thread_id, count, elapsed, hash);
            }

            // Stop after 60 seconds
            if start.elapsed() > Duration::from_secs(60) {
                break;
            }
        }
    });

    println!("🔥 Stress test complete! Total iterations: {}", counter.load(Ordering::Relaxed));
}
