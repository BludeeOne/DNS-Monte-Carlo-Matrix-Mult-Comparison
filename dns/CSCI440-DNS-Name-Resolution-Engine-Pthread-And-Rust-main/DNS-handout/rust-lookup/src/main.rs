use std::env;
use std::fs::File;
use std::io::{prelude::*, BufReader, BufWriter, Write};
use std::path::Path;
use dns_lookup::{lookup_host};



fn main() {
    //https://doc.rust-lang.org/book/ch12-01-accepting-command-line-arguments.html
    let args: Vec<String> = env::args().collect();
    let outfile = &args[args.len()-1];
    let path = Path::new(&outfile);
    let display = path.display();

    // Open a file in write-only mode, returns `io::Result<File>`
    let mut ofile = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(ofile) => ofile,
    };

    let mut of = BufWriter::new(ofile);

    let inputs: Vec<String> = args[1..args.len()-1].to_vec();
    for input in inputs {
        // Create a path to the desired file
        let path = Path::new(&input);
        let display = path.display();
        // Open the path in read-only mode, returns `io::Result<File>`
        let mut file = match File::open(&path) {
            Err(why) => panic!("couldn't open {}: {}", display, why),
            Ok(file) => file,
        };
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let hostname = line.unwrap();
            let mut ips: Vec<std::net::IpAddr> = vec![];
            let version = lookup_host(&hostname);
            match version {
                Ok(v) => ips = v,
                Err(e) => eprintln!("dnslookup error: {:?}", hostname),
            }
            let mut temp: String = "".to_string();
            if ips.len() > 0 {
                temp = ips[0].to_string();
            }
            of.write(format!("{} {}\n",  hostname, temp).as_bytes());
        }
    }
}



