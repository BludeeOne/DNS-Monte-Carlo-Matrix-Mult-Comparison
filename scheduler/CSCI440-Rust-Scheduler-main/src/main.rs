use std::env;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::time::Duration;
use clap::Parser;
use scheduler::{CLOCK, PCB, Schedule, simple::SimpleSchedule, simplerr::SimpleRRSchedule,mlrr::MLRRSchedule,simplemlf::SimpleMLFSchedule,mlf::MLFSchedule};

/// Simple args to set which scheduler to use and which input file to feed it
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of scheduler
    #[arg(short, long)]
    scheduler: String,

    /// input file
    #[arg(short, long)]
    input_file: String,
}

///Simple struct to track the input job information for the simulations
#[derive(Debug, Clone, Copy)]
struct Job {
    id: u32,
    time_inserted: u64,
    time_to_run: u32,
    priority: u32,
}

///Simulator for the MLF scheduler
fn mlf(lines: io::Lines<io::BufReader<File>>){
    let mut sched = MLFSchedule::new();
    //Initialize clock to 0
    CLOCK.set_now(Duration::from_millis(0));
    // HashMap keyed by ID
    let mut jobs_by_id: HashMap<u32, Job> = HashMap::new();

    // Optionally, a secondary index keyed by time_inserted
    let mut jobs_by_time: HashMap<u64, Vec<u32>> = HashMap::new(); // time_inserted -> IDs

    // Consumes the iterator, returns an (Optional) String
    // Parses input file into two HashMaps to make manipulation easier
    for line in lines.map_while(Result::ok) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let id: u32 = parts[0].parse().unwrap_or_else(|_| {
            eprintln!("Invalid ID on line: {}", line);
            std::process::exit(1);
        });
        let time_inserted: u64 = parts[1].parse().unwrap_or_else(|_| {
            eprintln!("Invalid time_inserted on line: {}", line);
            std::process::exit(1);
        });
        let time_to_run: u32 = parts[2].parse().unwrap_or_else(|_| {
            eprintln!("Invalid time_to_run on line: {}", line);
            std::process::exit(1);
        });

        let priority: u32 = parts[3].parse().unwrap_or_else(|_| {
            eprintln!("Invalid time_to_run on line: {}", line);
            std::process::exit(1);
        });

        let job = Job { id, time_inserted, time_to_run, priority };
        jobs_by_id.insert(id, job);

        // build secondary index for time_inserted
        jobs_by_time.entry(time_inserted).or_default().push(id);
    }
    //RUN Simulation
    while !jobs_by_id.is_empty() {
        let mut current_time = CLOCK.now().as_nanos();
        // println!("t = {} ", current_time);
        if let  Some(jobs) = jobs_by_time.get(&(current_time as u64)) {
            for job in jobs {
                let jid = jobs_by_id.clone();
                let pcb = PCB { id: *job, priority: jid.get(&job).unwrap().priority, time_added:None, time_scheduled:None};
                println!("Scheduled Process: {:?}, Priority:{}", pcb.id, pcb.priority);
                sched.add_process(pcb);
            }
        }
        while sched.has_process(){
            if let (Some(mut process), mut time) = sched.next_process() {
                let priority = match time {
                    0 => 0,
                    4 => 1,
                    1 => 2,
                    _ => 3, // default or handle other cases as needed
                };
                // println!("{:?}",process);
                let mut jid = jobs_by_id.clone();
                if let Some(job) = jobs_by_id.get_mut(&process.id) {
                    if time == 0 { //FCFS
                        loop {
                            println!("Process {} executed", process.id);
                            CLOCK.advance(Duration::from_nanos(1));
                            current_time = CLOCK.now().as_nanos();
                            // if current_time >=1800 {
                            //     println!("t = {} ", current_time);
                            // }
                            // println!("t = {} ", current_time);
                            if let Some(jobs) = jobs_by_time.get(&(current_time as u64)) {
                                // println!("Shouldn't be here");
                                for j in jobs {
                                    let jid2 = jid.clone();
                                    if let Some(tmp_job) = jid2.get(&j){
                                        let pcb = PCB { id: *j, priority: tmp_job.priority, time_added:None, time_scheduled:None};
                                        println!("Scheduled Process: {:?}, Priority:{}", pcb.id, pcb.priority);
                                        sched.add_process(pcb);
                                    }
                                }
                            }

                            if job.time_to_run <= 0 {
                                println!("Process {} Finished", process.id);
                                jobs_by_id.remove(&process.id);
                                break;
                            }
                            let pi = process.clone();
                            if sched.interrupt(pi, pi.priority){
                                break;
                            }
                            job.time_to_run -= 1;
                        }
                    }
                    else {
                        let mut interrupt = false;
                        loop {
                            println!("Process {} executed", process.id);
                            CLOCK.advance(Duration::from_nanos(1));
                            current_time = CLOCK.now().as_nanos();
                            // if current_time >=1800 {
                            //     println!("t = {} ", current_time);
                            // }
                            // println!("t = {} ", current_time);
                            if let Some(jobs) = jobs_by_time.get(&(current_time as u64)) {
                                // println!("Shouldn't be here");
                                for j in jobs {
                                    let jid2 = jid.clone();
                                    let pcb = PCB { id: *j, priority: jid2.get(&j).unwrap().priority, time_added:None, time_scheduled:None};
                                    println!("Scheduled Process: {:?}, Priority:{}", pcb.id, pcb.priority);
                                    sched.add_process(pcb);
                                }
                            }
                            time -= 1;
                            job.time_to_run -= 1;
                            if job.time_to_run <= 0 || time <= 0{
                                break;
                            }
                            let pi = process.clone();
                            if sched.interrupt(pi, pi.priority){
                                interrupt = true;
                                break;
                            }
                        }
                        if !interrupt {
                            if  job.time_to_run <= 0 {
                                println!("Process {} Finished", process.id);
                                jobs_by_id.remove(&process.id);
                            }
                            else {
                                sched.add_process(process);
                            }
                        }
                    }
                }
            }
            else{
                println!("None Process, something went wrong in your code.");
                std::process::exit(1);
            }
        }
    }
}

///Simulator for the Simple MLF scheduler that only promotes tasks
fn simplemlf(lines: io::Lines<io::BufReader<File>>){
    let mut sched = SimpleMLFSchedule::new();
    //Initialize clock to 0
    CLOCK.set_now(Duration::from_millis(0));
    // HashMap keyed by ID
    let mut jobs_by_id: HashMap<u32, Job> = HashMap::new();

    // Optionally, a secondary index keyed by time_inserted
    let mut jobs_by_time: HashMap<u64, Vec<u32>> = HashMap::new(); // time_inserted -> IDs

    // Consumes the iterator, returns an (Optional) String
    // Parses input file into two HashMaps to make manipulation easier
    for line in lines.map_while(Result::ok) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let id: u32 = parts[0].parse().unwrap_or_else(|_| {
            eprintln!("Invalid ID on line: {}", line);
            std::process::exit(1);
        });
        let time_inserted: u64 = parts[1].parse().unwrap_or_else(|_| {
            eprintln!("Invalid time_inserted on line: {}", line);
            std::process::exit(1);
        });
        let time_to_run: u32 = parts[2].parse().unwrap_or_else(|_| {
            eprintln!("Invalid time_to_run on line: {}", line);
            std::process::exit(1);
        });

        let priority: u32 = parts[3].parse().unwrap_or_else(|_| {
            eprintln!("Invalid time_to_run on line: {}", line);
            std::process::exit(1);
        });

        let job = Job { id, time_inserted, time_to_run, priority };
        jobs_by_id.insert(id, job);

        // build secondary index for time_inserted
        jobs_by_time.entry(time_inserted).or_default().push(id);
    }
    //RUN Simulation
    while !jobs_by_id.is_empty() {
        let mut current_time = CLOCK.now().as_nanos();
        // println!("t = {} ", current_time);
        if let  Some(jobs) = jobs_by_time.get(&(current_time as u64)) {
            for job in jobs {
                let jid = jobs_by_id.clone();
                let pcb = PCB { id: *job, priority: jid.get(&job).unwrap().priority, time_added:None, time_scheduled:None};
                println!("Scheduled Process: {:?}, Priority:{}", pcb.id, pcb.priority);
                sched.add_process(pcb);
            }
        }
        while sched.has_process(){
            if let (Some(process),mut time) = sched.next_process() {
                // println!("{:?}",process);
                let mut jid = jobs_by_id.clone();
                if let Some(job) = jobs_by_id.get_mut(&process.id) {
                    if time == 0 { //FCFS
                        loop {
                            println!("Process {} executed", process.id);
                            CLOCK.advance(Duration::from_nanos(1));
                            current_time = CLOCK.now().as_nanos();
                            // if current_time >=1800 {
                            //     println!("t = {} ", current_time);
                            // }
                            // println!("t = {} ", current_time);
                            if let Some(jobs) = jobs_by_time.get(&(current_time as u64)) {
                                // println!("Shouldn't be here");
                                for j in jobs {
                                    let jid2 = jid.clone();
                                    if let Some(tmp_job) = jid2.get(&j){
                                        let pcb = PCB { id: *j, priority: tmp_job.priority, time_added:None, time_scheduled:None};
                                        println!("Scheduled Process: {:?}, Priority:{}", pcb.id, pcb.priority);
                                        sched.add_process(pcb);
                                    }
                                }
                            }
                            if job.time_to_run <= 0 {
                                println!("Process {} Finished", process.id);
                                jobs_by_id.remove(&process.id);
                                break;
                            }
                            job.time_to_run -= 1;

                        }
                    }
                    else {
                        loop {
                            println!("Process {} executed", process.id);
                            CLOCK.advance(Duration::from_nanos(1));
                            current_time = CLOCK.now().as_nanos();
                            // if current_time >=1800 {
                            //     println!("t = {} ", current_time);
                            // }
                            // println!("t = {} ", current_time);
                            if let Some(jobs) = jobs_by_time.get(&(current_time as u64)) {
                                // println!("Shouldn't be here");
                                for j in jobs {
                                    let jid2 = jid.clone();
                                    let pcb = PCB { id: *j, priority: jid2.get(&j).unwrap().priority, time_added:None, time_scheduled:None};
                                    println!("Scheduled Process: {:?}, Priority:{}", pcb.id, pcb.priority);
                                    sched.add_process(pcb);
                                }
                            }
                            time -= 1;
                            job.time_to_run -= 1;
                            if job.time_to_run <= 0 || time <= 0{
                                break;
                            }

                        }
                        if job.time_to_run <= 0 {
                            println!("Process {} Finished", process.id);
                            jobs_by_id.remove(&process.id);
                        }
                        else {
                            sched.add_process(process);
                        }
                    }
                }
            }
            else{
                println!("None Process, something went wrong in your code.");
                std::process::exit(1);
            }
        }
    }
}

///Simulator for the MLRR scheduler
fn mlrr(lines: io::Lines<io::BufReader<File>>){
    let mut sched = MLRRSchedule::new();
    //Initialize clock to 0
    CLOCK.set_now(Duration::from_millis(0));
    // HashMap keyed by ID
    let mut jobs_by_id: HashMap<u32, Job> = HashMap::new();

    // Optionally, a secondary index keyed by time_inserted
    let mut jobs_by_time: HashMap<u64, Vec<u32>> = HashMap::new(); // time_inserted -> IDs

    // Consumes the iterator, returns an (Optional) String
    // Parses input file into two HashMaps to make manipulation easier
    for line in lines.map_while(Result::ok) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let id: u32 = parts[0].parse().unwrap_or_else(|_| {
            eprintln!("Invalid ID on line: {}", line);
            std::process::exit(1);
        });
        let time_inserted: u64 = parts[1].parse().unwrap_or_else(|_| {
            eprintln!("Invalid time_inserted on line: {}", line);
            std::process::exit(1);
        });
        let time_to_run: u32 = parts[2].parse().unwrap_or_else(|_| {
            eprintln!("Invalid time_to_run on line: {}", line);
            std::process::exit(1);
        });
        let priority: u32 = parts[3].parse().unwrap_or_else(|_| {
            eprintln!("Invalid time_to_run on line: {}", line);
            std::process::exit(1);
        });

        let job = Job { id, time_inserted, time_to_run, priority };
        jobs_by_id.insert(id, job);

        // build secondary index for time_inserted
        jobs_by_time.entry(time_inserted).or_default().push(id);
    }

    //RUN Simulation
    while !jobs_by_id.is_empty() {
        let mut current_time = CLOCK.now().as_nanos();
        // println!("t = {} ", current_time);
        if let Some(jobs) = jobs_by_time.get(&(current_time as u64)) {
            for j in jobs {
                let pcb = PCB { id: *j, priority: jobs_by_id.get(&j).unwrap().priority, time_added:None, time_scheduled:None};
                println!("Scheduled Process: {:?}, Priority:{}", pcb.id, pcb.priority);
                sched.add_process(pcb);

            }
        }
        while sched.has_process(){
            if let (Some(process),mut time) = sched.next_process() {
                // println!("{:?}",process);
                if let Some(job) = jobs_by_id.get_mut(&process.id) {
                    loop {
                        println!("Process {} executed", process.id);
                        CLOCK.advance(Duration::from_nanos(1));
                        time -= 1;
                        job.time_to_run -= 1;
                        if job.time_to_run <= 0 || time <= 0{
                            break;
                        }
                    }
                    if job.time_to_run <= 0 {
                        println!("Process {} Finished", process.id);
                        jobs_by_id.remove(&process.id);
                    }
                    else {
                        sched.add_process(process);
                    }
                    // println!("HERE");
                    current_time = CLOCK.now().as_nanos();
                    // println!("t = {} ", current_time);
                    if let Some(jobs) = jobs_by_time.get(&(current_time as u64)) {
                        // println!("Shouldn't be here");
                        for j in jobs {
                            let pcb = PCB { id: *j, priority: jobs_by_id.get(&j).unwrap().priority, time_added:None, time_scheduled:None};
                            println!("Scheduled Process: {:?}, Priority:{}", pcb.id, pcb.priority);
                            sched.add_process(pcb);

                        }
                    }
                    else{
                        // println!("Should be here");
                        continue;
                    }
                }
            }
            else{
                println!("None Process, something went wrong in your code.");
                std::process::exit(1);
            }
        }
    }
}

///Simulator for the SimpleRR scheduler
fn simplerr(lines: io::Lines<io::BufReader<File>>){
    let mut sched = SimpleRRSchedule::new();
    //Initialize clock to 0
    CLOCK.set_now(Duration::from_millis(0));
    // HashMap keyed by ID
    let mut jobs_by_id: HashMap<u32, Job> = HashMap::new();

    // Optionally, a secondary index keyed by time_inserted
    let mut jobs_by_time: HashMap<u64, Vec<u32>> = HashMap::new(); // time_inserted -> IDs

    // Consumes the iterator, returns an (Optional) String
    // Parses input file into two HashMaps to make manipulation easier
    for line in lines.map_while(Result::ok) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let id: u32 = parts[0].parse().unwrap_or_else(|_| {
            eprintln!("Invalid ID on line: {}", line);
            std::process::exit(1);
        });
        let time_inserted: u64 = parts[1].parse().unwrap_or_else(|_| {
            eprintln!("Invalid time_inserted on line: {}", line);
            std::process::exit(1);
        });
        let time_to_run: u32 = parts[2].parse().unwrap_or_else(|_| {
            eprintln!("Invalid time_to_run on line: {}", line);
            std::process::exit(1);
        });

        let priority: u32 = 0;
        let job = Job { id, time_inserted, time_to_run, priority };
        jobs_by_id.insert(id, job);

        // build secondary index for time_inserted
        jobs_by_time.entry(time_inserted).or_default().push(id);
    }

    //RUN Simulation
    while !jobs_by_id.is_empty() {
        let current_time = CLOCK.now().as_nanos();
        // println!("t = {} ", current_time);
        if let Some(jobs) = jobs_by_time.get(&(current_time as u64)) {
            for job in jobs {
                let pcb = PCB { id: *job, priority: 0, time_added:None, time_scheduled:None};
                println!("Scheduled Process: {:?}", pcb.id);
                sched.add_process(pcb);

            }
        }
        while sched.has_process(){
            if let (Some(process),mut time) = sched.next_process() {
                // println!("{:?}",process);
                if let Some(job) = jobs_by_id.get_mut(&process.id) {
                    loop {
                        println!("Process {} executed", process.id);
                        CLOCK.advance(Duration::from_nanos(1));
                        time -= 1;
                        job.time_to_run -= 1;
                        if job.time_to_run <= 0 || time <= 0{
                            break;
                        }
                    }
                    if job.time_to_run <= 0 {
                        println!("Process {} Finished", process.id);
                        jobs_by_id.remove(&process.id);
                    }
                    else {
                        sched.add_process(process);
                    }
                }
            }
            else{
                println!("None Process, something went wrong in your code.");
                std::process::exit(1);
            }
        }
    }
}

///Simulator for the Simple FIFO scheduler
fn simple(lines: io::Lines<io::BufReader<File>>){
    let mut sched = SimpleSchedule::new();
    //Initialize clock to 0
    CLOCK.set_now(Duration::from_millis(0));
    // HashMap keyed by ID
    let mut jobs_by_id: HashMap<u32, Job> = HashMap::new();

    // Optionally, a secondary index keyed by time_inserted
    let mut jobs_by_time: HashMap<u64, Vec<u32>> = HashMap::new(); // time_inserted -> IDs

    // Consumes the iterator, returns an (Optional) String
    // Parses input file into two HashMaps to make manipulation easier
    for line in lines.map_while(Result::ok) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let id: u32 = parts[0].parse().unwrap_or_else(|_| {
            eprintln!("Invalid ID on line: {}", line);
            std::process::exit(1);
        });
        let time_inserted: u64 = parts[1].parse().unwrap_or_else(|_| {
            eprintln!("Invalid time_inserted on line: {}", line);
            std::process::exit(1);
        });
        let time_to_run: u32 = parts[2].parse().unwrap_or_else(|_| {
            eprintln!("Invalid time_to_run on line: {}", line);
            std::process::exit(1);
        });

        let priority: u32 = 0;
        let job = Job { id, time_inserted, time_to_run, priority };
        jobs_by_id.insert(id, job);

        // build secondary index for time_inserted
        jobs_by_time.entry(time_inserted).or_default().push(id);
    }

    //RUN Simulation
    while !jobs_by_id.is_empty() {
        let current_time = CLOCK.now().as_nanos();
        // println!("t = {} ", current_time);
        if let Some(jobs) = jobs_by_time.get(&(current_time as u64)) {
            for job in jobs {
                let pcb = PCB { id: *job, priority: 0, time_added:None,time_scheduled:None};
                println!("Scheduled Process: {:?}", pcb.id);
                sched.add_process(pcb);

            }
        }
        while sched.has_process(){
            if let (Some(process),_) = sched.next_process() {
                // println!("{:?}",process);
                if let Some(job) = jobs_by_id.get_mut(&process.id) {
                    loop {
                        println!("Process {} executed", process.id);
                        CLOCK.advance(Duration::from_nanos(1));
                        if job.time_to_run <= 0 {
                            break;
                        }
                        job.time_to_run -= 1;
                    }
                    println!("Process {} Finished", process.id);
                    jobs_by_id.remove(&process.id);
                }
            }
            else{
                println!("None Process, something went wrong in your code.");
                std::process::exit(1);
            }
        }
    }
}

fn main() {
    //Parse the inputs for which scheduler and which input file to use
    let args = Args::parse();
    //Assuming input file exists, read all the lines from the input file
    if let Ok(lines) = read_lines(args.input_file) {
        //Now determine what scheduler to run the inputs on
        match args.scheduler.as_str() {
            "simple" => simple(lines),
            "simplerr" => simplerr(lines),
            "mlrr" => mlrr(lines),
            "simplemlf"=> simplemlf(lines),
            "mlf"=> mlf(lines),
            other => {
                eprintln!("Error: unknown scheduler '{}'", other);
                std::process::exit(1);
            }
        }
    }
}


// The output is wrapped in a Result to allow matching on errors.
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
