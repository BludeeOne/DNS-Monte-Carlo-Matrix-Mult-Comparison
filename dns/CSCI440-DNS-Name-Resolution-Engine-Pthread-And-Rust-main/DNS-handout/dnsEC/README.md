CSCI340 (Operating Systems)
Spring 2014
California State University, Chico
Programming Assignment 4

Author: Erik Flores-Siemsen

Extra Credits attempted
1. Matching Number of Threads to Number of Cores
2. Multiple IP addresses
3. Benchmarks

1. Matching Threads to Cores
In the C version of the code Muli-lookup.c we were allowed to use the library unistd.h which contains the function sysconf(_SC_NRPROCESSORS_ONLN) which returns the number of processors that are currently online and available for use by the operating system. Then using the recommended min and max resolver threads (2 and 10) i used this code:
* long num_cores = sysconf(_SC_NPROCESSORS_ONLN);
* if (num_cores < 1) num_cores = 1;
* int num_resolvers = (int)num_cores;
I use the function call to determine how many cores are available, then if that numver happened to be less than 1 for some reason Its set to 1, otherise thats the number of resolvers this program will use. 

In the Rust iteration and part of its standard library, is the function std::thread::available_parallelism() 
which also returns an estiamte of available cores for parallel tasks to optimize performance
the code i used:
let num_cores = 
* std::thread::available_parallelism()
*     .map(|n| n.get())
*     .unwrap_or(1);
Since available parallelism returns a NonZeroUsize i converted it into a usize int and if it couldnt determine a count then it sets to 1 like the c code

2. Multiple IP Addresses
Many hostnames resolve to more than one IP address. However the orginal util.c intentionally only looked at the headresult, and overwrote everything that came after with a null "\0"
In Rust it was pretty easy lookup_host() already returns a Vec of all IPs. Updated the resolver to collect all entries and join them with commas; Code:
* ips.iter()
*    .map(|i: &std::net::IpAddr| i.to_string())
*    .collect::<Vec<String>>()
*    .join(",")
However in the c code, in keeping with the accordance of keeping the file backwards compatible and creating a new function
I added dnslookup_multi() to util.c and util.h alongside the existing dnslookup() to maintain backwards compatibility. The new function walks the full getaddrinfo linked list and appends each unique IPv4 address with strncat.

3. Benchmarks
-Specs and Setup
Machine: Apple M1 (8 cores)
Input: 5 name files (names1.txt through names5.txt)
Measurement: wall clock time (time command, total field)
small note: Dns lookup time depends on network and so the runs varied, Also dns timeouts on hostnames signifcantly impacted times
I decided to do each benchmark 5 times to get an avg time
-Results
Resolver threads        Avg Wall Time
2                       0.066
4                       0.075
6                       0.059   
8                       0.069
10                      0.061
12                      0.105
16                      0.074
sysconf(8cores)         0.097

-Analysis
The results show that core to core performance differences are so small and within the margin of just DNS network variance. Even 2 threads performed very similary to 16 even though the computer only has 8 cores. The program probably spent most of its time waiting for network responses rather than the cpu doing work.

The sysconf average of 0.097 is slightly higher due to a DNS timeout outlier in its first run of (0.238) Excluding that run it would get a 0.062 average becoming consistent with the other thread averages. I also noticed that After a period of not testing, returning to it also seemed to result in a time of 13+ seconds. Almost always 13 seconds but I also got a 16 and 20 second long dns timeout length. Then it would immedately drop to some time in the tenths(0.1+) followed by consistent tests in the hundredths(0.01+)

-Conclusion
For DNS-bound workloads, matching resolver threads to core count is a resonable approach. It avoids over-provisioning threads beyond what the hardware can actually run parallel, while still providing enough concurrency to keep all cores busy during dns waits. For my particular machine an ideal core count is between 6 and 10. With diminishing returns once threads begin to compete for cores