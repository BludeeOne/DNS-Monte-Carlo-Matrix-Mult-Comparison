/* Matrix Multiplicatoin but in RUST
 * Author: Erik FLores-Siemsen
 * Date: April 2026
 * 
* Descrition: Parallel matrix multiplication using pthreads. 
 * This file/code multiplies two square N x N matrices 
 * A and B to produe result matrix C. 
 * This formula: C[i][j] = sum of A[i][k] * B[k][j] for k = 0..N-1
 * Each thread works on a contiguous range of rows of the result matrix C
 * and does it works independently. Therefore no mutex is needed because theyre writing
 * to their own regions within the matrix C
 * 
 * since rust doesnt allow threads to share raw pointers cause no one has ownership
 * i got around this by splitting the matrix into nonOverlapping chunks with chunks_mut(),
 * satisfying rust by giving each thread exclusive ownership of its rows
 * 
 * references in README.md
 * 
* How To Run:
 *      cargo run --release -- <matrix_size> <num_threads>
 *      Example: cargo run --release -- 512 4
 *
 * data size benchmarks: 
 * 256
 * 512
 * 1024
 * 
  */


use std::env;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

fn main() {

    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <matrix_size> <num_threads>", args[0]);
        eprintln!("Example: {} 512 4", args[0]);
        std::process::exit(1);
    }

    let n: usize = args[1].parse().unwrap_or_else(|_| {
        eprintln!("ERROR: matrix_size must be a positive integer");
        std::process::exit(1);
    });

    let num_threads: usize = args[2].parse().unwrap_or_else(|_| {
        eprintln!("ERROR: num_threads must be a positive integer");
        std::process::exit(1);
    });

    if n < 1 {
        eprintln!("ERROR: matrix_size must be >= 1");
        std::process::exit(1);
    }

    if num_threads < 1 || num_threads > 64 {
        eprintln!("ERROR: num_threads must be between 1 and 64");
        std::process::exit(1);
    }

    if n < num_threads {
        eprintln!("ERROR: matrix_size must be >= num_threads");
        std::process::exit(1);
    }

    //allocate matrices
    let mut a = vec![0.0; n * n];
    let mut b = vec![0.0; n * n];
    let mut c = vec![0.0; n * n];

    //fill with random values
    for i in 0..(n * n) {
        a[i] = rand::random::<f64>();
        b[i] = rand::random::<f64>();
    }

    let a = Arc::new(a);
    let b = Arc::new(b);

    let start = Instant::now();

    let rows_per_thread = n / num_threads;
    let remainder = n % num_threads;

    let mut handles = Vec::new();

    //split rows across threads
    for i in 0..num_threads {

        let a_clone = Arc::clone(&a);
        let b_clone = Arc::clone(&b);

        let start_row = i * rows_per_thread;

        let end_row = if i == num_threads - 1 {
            start_row + rows_per_thread + remainder
        } else {
            start_row + rows_per_thread
        };

        let handle = thread::spawn(move || {

            let mut local_c = vec![0.0; (end_row - start_row) * n];

            for r in start_row..end_row {
                for col in 0..n {

                    let mut sum = 0.0;

                    for k in 0..n {
                        sum += a_clone[r * n + k] * b_clone[k * n + col];
                    }

                    local_c[(r - start_row) * n + col] = sum;
                }
            }

            (start_row, local_c)
        });

        handles.push(handle);
    }

    //collect results
    for handle in handles {
        let (start_row, local_c) = handle.join().unwrap();

        let rows = local_c.len() / n;

        for r in 0..rows {
            for col in 0..n {
                c[(start_row + r) * n + col] = local_c[r * n + col];
            }
        }
    }

    let elapsed = start.elapsed().as_secs_f64();

    println!("Matrix size: {} x {}", n, n);
    println!("Threads: {}", num_threads);
    println!("Elapsed time: {:.6} seconds", elapsed);

    println!("C[0][0]: {:.6}", c[0]);
}