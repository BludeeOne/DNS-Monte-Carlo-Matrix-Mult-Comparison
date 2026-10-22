//Multi-lookup DNS 
//Author: Erik Flores-Siemsen
// Requester thread pool -> bounded buffer -> resolver thread pool -> results file

// Resources:
// https://doc.rust-lang.org/book/ch16-00-concurrency.html
// https://doc.rust-lang.org/std/sync/struct.Mutex.html
// https://doc.rust-lang.org/std/sync/struct.Condvar.html - used to save cpu time while a thread is sent to wait


use std::collections::VecDeque;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;
use dns_lookup::lookup_host;

const MAX_INPUT_FILES: usize      = 10;
const MAX_RESOLVER_THREADS: usize = 10;
const MIN_RESOLVER_THREADS: usize = 2;
const QUEUE_SIZE: usize           = 50;
const MAX_SLEEP_US: u64           = 100;



struct BoundedBuffer {
    queue: Mutex<VecDeque<String>>,
    done: Mutex<bool>,
    condvar: Condvar,
}


fn rand_sleep() {
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() as u64;
    let micros = seed % (MAX_SLEEP_US +1);
    thread::sleep(Duration::from_micros(micros));
}

fn requester(filename: String, buf: Arc<BoundedBuffer>) {
    let file = match File::open(&filename) {
        Err(e) => {
            eprintln!("ERROR: Cannot open input file\"{}\": {}", filename, e);
            return;
        }
        Ok(f) => f,
    };

    for line in BufReader::new(file).lines() {
        let hostname = match line {
            Err(e) => {
                eprintln!("ERROR: I/O error reading \"{}\": {}", filename, e);
            continue;
            }
            Ok(h) => h,
        };
    

    let hostname = hostname.trim().to_string();
    if hostname.is_empty() {
        continue;
    }

    loop {
        let mut queue = buf.queue.lock().unwrap();
        if queue.len() < QUEUE_SIZE {
            queue.push_back(hostname);
            buf.condvar.notify_one();
            break;
        }
        drop(queue);
        rand_sleep();
        }
    }
}

fn resolver(buf: Arc<BoundedBuffer>, output: Arc<Mutex<BufWriter<File>>>) {
    loop {
        let hostname;
        {
            let mut queue = buf.queue.lock().unwrap();
            loop{
                if let Some(h) = queue.pop_front() {
                    hostname = h;
                    break;
                }

                let done_flag = *buf.done.lock().unwrap();
                if done_flag {
                    return;
                }
            queue = buf.condvar.wait(queue).unwrap();
            }
        }

        let ip = match lookup_host(&hostname) {
            Ok(ips) => {
                if ips.is_empty() {
                    eprintln!("WARNING: DNS lookup failed for hostname \"{}\"", hostname);
                    String::new()
                } else {
                    ips.iter()
                        .map(|i: &std::net::IpAddr| i.to_string())
                        .collect::<Vec<String>>()
                        .join(",")
                }
            }
            Err(_) => {
                eprintln!("WARNING: DNS lookup failed for hostname \"{}\"", hostname);
                String::new()
            }
        };

        let mut out = output.lock().unwrap();
        if let Err(e) = writeln!(out, "{},{}", hostname, ip) {
            eprintln!("ERROR: failed writing to output file: {}", e);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
 
    if args.len() < 3 {
        eprintln!("Usage: {} <namefile1> [namefile2 ...] <outputfile>", args[0]);
        eprintln!("  Maximum input files : {}", MAX_INPUT_FILES);
        eprintln!("  Resolver threads    : {} - {}", MIN_RESOLVER_THREADS, MAX_RESOLVER_THREADS);
        std::process::exit(1);
    }
 
    let num_input_files = args.len() - 2;
    let input_files: Vec<String> = args[1..args.len()-1].to_vec();
    let output_file = &args[args.len()-1];
 
    if num_input_files > MAX_INPUT_FILES {
        eprintln!("ERROR: Too many input files ({}). Maximum allowed: {}.", num_input_files, MAX_INPUT_FILES);
        std::process::exit(1);
    }
 
    let file = match File::create(output_file) {
        Err(e) => {
            eprintln!("ERROR: Cannot open output file \"{}\": {}", output_file, e);
            std::process::exit(1);
        }
        Ok(f) => f,
    };
 
    let shared_output: Arc<Mutex<BufWriter<File>>> =
        Arc::new(Mutex::new(BufWriter::new(file)));
 
    let shared_buf: Arc<BoundedBuffer> = Arc::new(BoundedBuffer {
        queue:   Mutex::new(VecDeque::new()),
        done:    Mutex::new(false),
        condvar: Condvar::new(),
    });
 
    let num_cores = std::thread::available_parallelism()
    .map(|n| n.get())
    .unwrap_or(1); 

    let num_resolvers = num_cores.max(MIN_RESOLVER_THREADS).min(MAX_RESOLVER_THREADS); 
    let mut res_handles = Vec::new();

    for _ in 0..num_resolvers {
        let buf  = Arc::clone(&shared_buf);
        let out  = Arc::clone(&shared_output);
        res_handles.push(thread::spawn(move || resolver(buf, out)));
    }
 
    let mut req_handles = Vec::new();
    for filename in input_files {
        let buf = Arc::clone(&shared_buf);
        req_handles.push(thread::spawn(move || requester(filename, buf)));
    }
 
    for handle in req_handles {
        if let Err(e) = handle.join() {
            eprintln!("ERROR: requester thread panicked: {:?}", e);
        }
    }
 
    {
        let mut done = shared_buf.done.lock().unwrap();
        *done = true;
        shared_buf.condvar.notify_all();
    }
 
    for handle in res_handles {
        if let Err(e) = handle.join() {
            eprintln!("ERROR: resolver thread panicked: {:?}", e);
        }
    }
 
    if let Err(e) = shared_output.lock().unwrap().flush() {
        eprintln!("ERROR: failed to flush output file: {}", e);
    };
 
}
 