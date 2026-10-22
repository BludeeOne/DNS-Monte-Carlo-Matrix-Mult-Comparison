/* DNS implementation in RUST
 * Author: Erik flores-siemsen
 * Date: May 2026
 * 
 * Description: parallel dns resolution using requster resolver threads
 * I adapted from our previous assignment 
 * Requester threads pull hostnames from my hard coded list
 * and push them into a bounded buffer queue
 * resolvers pop hostnames and call dns lookup
 * results are written to an outfile
 * 
 * DNS is a good third system tester because unlike monte carlo and matrix mult.
 * dns is I/O bound. Threads spend most time waiting for netwrok responses or even waits
 * 
 * data sizes:
 * small: 50 hostnames
 * medium: 200 hostnames
 * large: 500 hostnames
 * 
 * references in readme.md
 *  */


use std::collections::VecDeque;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use dns_lookup::lookup_host;

const MAX_RESOLVER_THREADS: usize = 64;
const MIN_RESOLVER_THREADS: usize = 2;
const QUEUE_SIZE: usize = 50;
const MAX_SLEEP_US: u64 = 100;

//hardcoded domains for testing
const HOSTNAMES: &[&str] = &[
    "google.com",       "youtube.com",      "facebook.com",
    "amazon.com",       "wikipedia.org",    "twitter.com",
    "instagram.com",    "linkedin.com",     "reddit.com",
    "apple.com",        "microsoft.com",    "netflix.com",
    "github.com",       "stackoverflow.com","openai.com",
    "cloudflare.com",   "adobe.com",        "dropbox.com",
    "spotify.com",      "twitch.tv",        "discord.com",
    "slack.com",        "zoom.us",          "shopify.com",
    "paypal.com",       "ebay.com",         "pinterest.com",
    "tumblr.com",       "quora.com",        "medium.com",
    "nytimes.com",      "bbc.com",          "cnn.com",
    "theguardian.com",  "npr.org",          "espn.com",
    "weather.com",      "archive.org",      "wolframalpha.com",
    "nasa.gov",         "mit.edu",          "stanford.edu",
    "berkeley.edu",     "csuchico.edu",     "python.org",
    "rust-lang.org",    "golang.org",       "kernel.org",
    "debian.org",       "ubuntu.com",
];

//shared buffer between requesters and resolvers
struct BoundedBuffer {
    queue: Mutex<VecDeque<String>>,
    done: Mutex<bool>,
    condvar: Condvar,
}

//small random sleep when queue is full
fn rand_sleep() {
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() as u64;

    let micros = seed % (MAX_SLEEP_US + 1);

    thread::sleep(Duration::from_micros(micros));
}

fn requester(hostnames: Vec<String>, buf: Arc<BoundedBuffer>) {

    for hostname in hostnames {
        loop {
            let mut queue = buf.queue.lock().unwrap();

            if queue.len() < QUEUE_SIZE {
                queue.push_back(hostname.clone());
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

            loop {
                if let Some(h) = queue.pop_front() {
                    hostname = h;
                    break;
                }

                let done_flag = *buf.done.lock().unwrap();

                if done_flag {
                    return;
                }

                //wait until requester adds something
                queue = buf.condvar.wait(queue).unwrap();
            }
        }

        let ip = match lookup_host(&hostname) {
            Ok(ips) if !ips.is_empty() => {
                ips.iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            }

            _ => {
                eprintln!(
                    "WARNING: DNS lookup failed for hostname \"{}\"",
                    hostname
                );

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

    if args.len() != 4 {
        eprintln!(
            "Usage: {} <num_hosts> <num_threads> <outputfile>",
            args[0]
        );

        eprintln!(
            "Example: {} 200 4 results.txt",
            args[0]
        );

        std::process::exit(1);
    }

    let num_hosts: usize = args[1].parse().unwrap_or_else(|_| {
        eprintln!("ERROR: num_hosts must be a positive integer");
        std::process::exit(1);
    });

    let num_threads: usize = args[2].parse().unwrap_or_else(|_| {
        eprintln!("ERROR: num_threads must be a positive integer");
        std::process::exit(1);
    });

    let output_file = &args[3];

    if num_threads < MIN_RESOLVER_THREADS
        || num_threads > MAX_RESOLVER_THREADS {

        eprintln!(
            "ERROR: num_threads must be between {} and {}",
            MIN_RESOLVER_THREADS,
            MAX_RESOLVER_THREADS
        );

        std::process::exit(1);
    }

    //build hostname list
    let all_hostnames: Vec<String> = (0..num_hosts)
        .map(|i| HOSTNAMES[i % HOSTNAMES.len()].to_string())
        .collect();

    //open output file
    let file = File::create(output_file).unwrap_or_else(|e| {
        eprintln!(
            "ERROR: cannot open output file \"{}\": {}",
            output_file,
            e
        );

        std::process::exit(1);
    });

    let shared_output: Arc<Mutex<BufWriter<File>>> =
        Arc::new(Mutex::new(BufWriter::new(file)));

    //shared queue
    let shared_buf: Arc<BoundedBuffer> = Arc::new(BoundedBuffer {
        queue: Mutex::new(VecDeque::new()),
        done: Mutex::new(false),
        condvar: Condvar::new(),
    });

    let start = Instant::now();

    //start resolvers first
    let mut res_handles = Vec::new();

    for _ in 0..num_threads {
        let buf = Arc::clone(&shared_buf);
        let out = Arc::clone(&shared_output);

        res_handles.push(
            thread::spawn(move || resolver(buf, out))
        );
    }

    let mut req_handles = Vec::new();

    let hosts_per_requester = num_hosts / num_threads;
    let remainder = num_hosts % num_threads;

    //split work evenly across requesters
    for i in 0..num_threads {

        let start_idx = i * hosts_per_requester;

        let end_idx = if i == num_threads - 1 {
            start_idx + hosts_per_requester + remainder
        }
        else {
            start_idx + hosts_per_requester
        };

        let slice: Vec<String> =
            all_hostnames[start_idx..end_idx].to_vec();

        let buf = Arc::clone(&shared_buf);

        req_handles.push(
            thread::spawn(move || requester(slice, buf))
        );
    }

    //wait for requesters
    for handle in req_handles {
        if let Err(e) = handle.join() {
            eprintln!(
                "ERROR: requester thread panicked: {:?}",
                e
            );
        }
    }

    {
        let mut done = shared_buf.done.lock().unwrap();
        *done = true;
        shared_buf.condvar.notify_all();
    }

    //wait for resolvers
    for handle in res_handles {
        if let Err(e) = handle.join() {
            eprintln!(
                "ERROR: resolver thread panicked: {:?}",
                e
            );
        }
    }

    let elapsed = start.elapsed().as_secs_f64();

    //flush output file
    if let Err(e) = shared_output.lock().unwrap().flush() {
        eprintln!("ERROR: failed to flush output file: {}", e);
    }

    println!("Hostnames: {}", num_hosts);
    println!("Threads: {}", num_threads);
    println!("Output file: {}", output_file);
    println!("Elapsed time: {:.6} seconds", elapsed);
}