/* Monte Carlo algorithim in RUST
 * Author: Erik Flores-Siemsen
 * Date: May 2026
 * 
 * Description: This file uses the Monte carlo method to repeatedly
 * randomly generate numbers to estimate pi. 
 * First create a unit square and then a quarter circle inside of it,
 * By generating random points within the square and counting how many 
 * land within the circle and taking the ratio of inside/total we get pi/4. 
 * Multiplying that number by 4 would give you pi. 
 * 
 * Compile is inside the Readme
 * references is also in the readme 
 *
 * to run: 
 *      ./montecarlo <num_points> <num_threads>
 *      Example: ./montecarlo 1000000 4
 * 
 */



use std::env;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

fn random_number(seed: &mut u64) -> f64 {
    const A: u64 = 1664525;
    const C: u64 = 1013904223;
    const M: u64 = u32::MAX as u64 + 1;

    *seed = (A.wrapping_mul(*seed).wrapping_add(C)) % M;
    *seed as f64 / M as f64
}

fn main() {

    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <num_points> <num_threads>", args[0]);
        eprintln!("Example: {} 1000000 4", args[0]);
        std::process::exit(1);
    }

    let num_points: u64 = args[1].parse().unwrap_or_else(|_| {
        eprintln!("ERROR: num_points must be a positive integer");
        std::process::exit(1);
    });

    let num_threads: u64 = args[2].parse().unwrap_or_else(|_| {
        eprintln!("ERROR: num_threads must be a positive integer");
        std::process::exit(1);
    });

    if num_threads < 1 || num_threads > 64 {
        eprintln!("ERROR: num_threads must be between 1 and 64");
        std::process::exit(1);
    }

    if num_points < num_threads {
        eprintln!("ERROR: num_points must be >= num_threads");
        std::process::exit(1);
    }

    let global_inside = Arc::new(Mutex::new(0u64));

    let start = Instant::now();

    let points_per_thread = num_points / num_threads;
    let remainder = num_points % num_threads;

    let mut handles = Vec::new();

    //split work across threads
    for i in 0..num_threads {

        let global_inside_clone = Arc::clone(&global_inside);

        let points = points_per_thread +
                     if i == num_threads - 1 { remainder } else { 0 };

        let seed_base: u64 = 987654321;
        let initial_seed = seed_base ^ (i.wrapping_mul(1_000_003));

        let handle = thread::spawn(move || {

            let mut seed = initial_seed;
            let mut local_inside = 0u64;

            for _ in 0..points {

                let x = random_number(&mut seed);
                let y = random_number(&mut seed);

                if x * x + y * y <= 1.0 {
                    local_inside += 1;
                }
            }

            let mut global = global_inside_clone.lock().unwrap();
            *global += local_inside;
        });

        handles.push(handle);
    }

    //wait for threads
    for handle in handles {
        handle.join().unwrap();
    }

    let elapsed = start.elapsed().as_secs_f64();

    let inside = *global_inside.lock().unwrap();
    let pi_estimate = 4.0 * inside as f64 / num_points as f64;
    let actual_pi = std::f64::consts::PI;
    let error = (pi_estimate - actual_pi).abs() / actual_pi * 100.0;

    println!("Threads: {}", num_threads);
    println!("Points: {}", num_points);
    println!("Inside circle: {}", inside);
    println!("Pi estimate: {:.9}", pi_estimate);
    println!("Actual pi: {:.9}", actual_pi);
    println!("Error: {:.6}%", error);
    println!("Elapsed time: {:.6} seconds", elapsed);
}