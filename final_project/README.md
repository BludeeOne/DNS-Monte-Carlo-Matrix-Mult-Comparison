C vs Rust Thread Performance Comparison
CSCI 440 - CSU CHICO
Erik Flores-Siemsen

This project benchmarks how threading performs in c (pthreads) vs Rust (std::thread). The goal is to see how each langauge handles different types of parallel workloads. 
Workloads such as: 
* CPU-heavy
* Memory-heavy
* I/O based

Main Question
Across Different workloads and programs utilizing threads, does C or Rust perform better?

What I tested
Algorithim          Type           Sizes
Monte Carlo         Cpu-bound      1m, 10m, 100m
Matrix Multiply     Memory Bound   256, 512, 1024
DNS resolution      I/O bound      200, 500 hostnames

I think ran each setup 50 times to get consistent results (Margin of error was around 5%)

How to build
    C files:
    $ gcc -O2 -pthread -o montecarlo src/c/montecarlo.c
    $ gcc -O2 -pthread -o matrix src/c/matrix.c
    $ gcc -O2 -pthread -o dns src/c/dns.c

    Rust Files:
    $ cd src/rust/montecarlo && cargo build --release
    $ cd src/rust/matrix && cargo build --release
    $ cd src/rust/dns && cargo build --release

Quick Summary of results(check report for full rundown)
Monte Carlo: C was a bit faster my rust threads slowed down I think from RNG overhead
Matrix Multiply: C was slightly ahead, but I had a feeling it could be optimized for Rust
DNS: They were basically the same - I only had a problem with network bottlenecking

References
see references.md for sources I used

