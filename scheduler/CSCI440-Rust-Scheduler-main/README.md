# CSCI440-Rust-Scheduler
Scheduler Assignment for CSCI440 in Rust

California State University - Chico
By Bryan Dixon

## Introduction
The purpose of this assignment is for you to gain insight into how schedulers work on the system.

## Logistics
The only “hand-in” will be electronic. Any clarifications and revisions to the assignment will be made here and announced to the class during class or via the discussion board.

## Hand Out Instructions

I recommend you use an Ubuntu Linux virtual machine to complete this assignment. Alternatively, you can use your native Linux install.

Download this repository as a zip file and then extract it where you would like to store your project files. An example for downloading and extracting the zip file is below, assuming you are in your home directory (you may remove the main.zip file after unzipping it):

```bash
~$ wget https://github.com/CSUChico-CSCI440/CSCI440-Rust-Scheduler/archive/refs/heads/main.zip
~$ unzip main.zip
#Will now have folder CSCI440-Rust-Scheduler with files
$ cd CSCI440-Rust-Scheduler/
$ ls
Cargo.lock  Cargo.toml  images/  LICENSE  README.md  src/  tests/
```

As you can see there are three folders contained in this repo. The images folder you can ignore, as this is a holder for the images in the README. The tests folder contains input files for each simulator, along with the expected output for each input, for a given scheduler. The src folder holds the Rust source.

You only need to make changes to *simple.rs*, *simplerr.rs*, *mlrr.rs*, *simplemlf.rs*, and *mlf.rs*. You will only be submitting these five files; please do not modify any code in the other files, as you will not be submitting them.

Looking at the *main.rs* file, you will see that it allows you to specify which scheduler to run, along with the input test file that drives the simulation. The *lib.rs* file has the common elements used in all five scheduler implementations. Finally, the *clock.rs* file provides a global simulated system clock for schedulers that need to do things based on time since certain events occurred.

I recommend implementing these in the order discussed later in this document, as they will, to some extent, build on the concepts of the earlier schedule as they get more complicated.


## General Overview of Schedulers
Taken from Wikipedia[1]
In computer science, scheduling is the method by which threads, processes, or data flows are given access to system resources (e.g., processor time and communication bandwidth). This is usually done to load-balance and share system resources effectively, or to achieve a target quality of service.  The need for a scheduling algorithm arises from the requirement for most modern systems to perform multitasking (executing more than one process at a time) and multiplexing (transmitting multiple data streams simultaneously across a single physical channel).

The scheduler is concerned mainly with:

* Throughput - The total number of processes that complete their execution per time unit.
* Latency, specifically:
	* Turnaround time - total time between submission of a process and its completion.
	* Response time - amount of time it takes from when a request was submitted until the first response is produced.
* Fairness - Equal CPU time to each process (or more generally, appropriate times according to each process’ priority and workload).
* Waiting Time - The time the process remains in the ready queue.

In practice, these goals often conflict (e.g., throughput versus latency); thus, a scheduler will implement a suitable compromise. Preference is given to any one of the concerns as mentioned earlier, depending upon the user’s needs and objectives. In real-time environments, such as embedded systems for industrial automatic control (e.g., robotics), the scheduler must also ensure that processes meet deadlines; this is crucial for keeping the system stable. Scheduled tasks can also be distributed to remote devices across a network and managed through an administrative back end.


## Your Task
The task for this assignment is to implement the scheduler APIs provided to you in the *simple.rs*, *simplerr.rs*, *mlrr.rs*, *simplemlf.rs*, and *mlf.rs* files for the following schedulers:

* **Simple** - A simple FCFS scheduler
    * FCFS (first-come, first-served) is also known as a FIFO (first-in, first-out) scheduler.
    * Behaves like a queue: the first process added to the queue is the first to be removed and executed.
      
* **Simple Round Robin** - A simple Round Robin scheduler with a quanta of 4 time units.  	
    * Similar to the simple FCFS scheduler, but this one is sensitive to response time.
    * A scheduling quantum is also known as a time slice - each process runs for the given time slice before the scheduler switches to the next job. This process repeats until the job is finished.
      
* **Multi-Level Round Robin** - A variant of a Multi-Level priority scheduler using Round Robin schedulers.
    * The first time the scheduler runs, it should start at the highest priority level (priority 0). Each subsequent time it runs, it should move to the next lower priority level, cycling through all levels in order. Use a global index variable to track which priority level the scheduler is currently processing.
    * The scheduler should: iterate through all priority levels in order (starting from the tracked index), skip any empty queues, and at each non-empty level, select the following process in that queue and schedule it for the appropriate time quantum.
    * Higher-priority levels should be assigned larger time quanta. Your implementation should match the quanta and number of priority levels shown in Figure 1.

Figure 1: Multi Level Round Robin Priority Scheduler

![MultiLevel Queue](https://github.com/CSUChico-CSCI440/CSCI440-Rust-Scheduler/blob/0a374cc14826c5741e5e4958e0bc1c9585de967a/images/multilevel.png "MultiLevel Queue")

* **Simple Multi Level Feedback** - A Multi-Level priority scheduler with feedback.
    * This scheduler consists of three queues: a FCFS scheduler for the highest priority tasks and two round robin queues for lower priority tasks.
    * The highest priority is represented with 0, and higher numbers represent lower priorities. The scheduler should always check the highest priority queue first.
    * All high-priority tasks (priority 0) should run to completion. All lower-priority tasks (priorities 1 and 2) are assigned a time quantum. Your implementation should mirror the number of priorities and implementation in Figure 2.
    * If a process has not been scheduled for 1000 time cycles, it should be promoted to the next higher-priority queue (its *time_added* should be reset to the current time when it is placed in the new queue).
      
Figure 2: Simple Multi-Level Feedback Priority Scheduler

![MultiLevel Feedback Queue](https://github.com/CSUChico-CSCI440/CSCI440-Rust-Scheduler/blob/0a374cc14826c5741e5e4958e0bc1c9585de967a/images/simplemultilevelfeedback.png "Simple MultiLevel Feedback Queue")

* **Multi Level Feedback** - A Multi-Level priority scheduler with feedback.
    * This scheduler consists of three queues: a FCFS scheduler for the highest priority tasks and two round robin queues for lower priority tasks.
    * The highest priority is represented with 0, and higher numbers represent lower priorities. The scheduler should always check the highest priority queue first.
    * All high-priority tasks (priority 0) should run to completion. All lower-priority tasks (priorities 1 and 2) are assigned a time quantum. Your implementation should mirror the number of priorities and implementation in Figure 2.
    * If a process has not been scheduled for 1000 time cycles, it should be promoted to the next higher-priority queue (its *time_added* should be reset to the current time when it is placed in the new queue).
    * If a process has been running for over 1000 time cycles, it should be demoted to the next lower-priority queue (its *time_added* should be reset to the current time when it is placed in the new queue).
      
Figure 3: Multi-Level Feedback Priority Scheduler

![MultiLevel Feedback Queue](https://github.com/CSUChico-CSCI440/CSCI440-Rust-Scheduler/blob/0a374cc14826c5741e5e4958e0bc1c9585de967a/images/multilevelfeedback.png "MultiLevel Feedback Queue")


You are not allowed to import any additional libraries, except those in the *std* library.

## Data Queue in Rust
If you would like to refresh yourself on building basic data structures like a linked list and queue, here is a simple implementation of a linked list-based Queue in Rust using structs and the Box class:

```Rust
pub struct List<T> {
    head: Link<T>,
}

type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List { head: None }
    }

    pub fn push(&mut self, elem: T) {
        let mut link = &mut self.head;

        loop {
            match link {
                Some(node) => {
                    link = &mut node.next;
                }
                None => {
                    *link = Some(Box::new(Node { elem, next: None }));
                    break;
                }
            }
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        self.head.take().map(|mut head| {
            self.head = head.next.take();
            head.elem
        })
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            next: self.head.as_deref(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut cur = self.head.take();
        while let Some(mut node) = cur {
            cur = node.next.take();
        }
    }
}

pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.as_deref();
            &node.elem
        })
    }
}

#[cfg(test)]
mod test {
    use super::List;
    #[test]
    fn basics() {
        let mut list = List::new();
        assert_eq!(list.is_empty(), true);
        assert_eq!(list.pop(), None);
        list.push(1);
        list.push(2);
        list.push(3);
        assert_eq!(list.pop(), Some(1));
    }
    #[test]
    fn iter() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);
        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
    }
}
```
Since it is challenging to do **safe** data structures, you can alternatively use the [VecDequeue](https://doc.rust-lang.org/std/collections/struct.VecDeque.html
) library in *std* Rust. 

## Checking Your Work
I have provided some tools to help you check your work.

* **Reference solution.** - I’ve included a reference output file with the expected solution for each scheduler you need to write. Your program should produce identical output to that of the ref.out file.
* You can:
    * run the project and redirect the output to your own output file, e.g.
      ```bash
      cargo run -- -s simple -i tests/simple/t1.in > my.out
      ```
    * use `diff` or `vimdiff` to compare the corresponding test output file with your output file, e.g.
      ```bash
      diff tests/simple/t1.out my.out
      ``` 


## Hints

* Read the [CPU Scheduling](http://pages.cs.wisc.edu/~remzi/OSTEP/cpu-sched.pdf) and [Multi-Level Feedback Scheduling](http://pages.cs.wisc.edu/~remzi/OSTEP/cpu-sched-mlfq.pdf) sections from the online textbook.
* You should potentially enable the RUST_BACKTRACE to troubleshoot errors:
  ```bash
   export RUST_BACKTRACE=1
  ```
* The code likely will have warnings when built; hopefully, over time, I'll work on removing these, but initially, they can be safely ignored. Should likely look at what the initial warnings are, as the code will build out of the box, so you can determine which warnings are for your code vs the provided code.
  


## Evaluation
You will be tested against the reference outputs. You will get full credit if your assignment reproduces the reference output. Grades for this assignment will be assigned as follows:

* 5% - Simple Scheduler working
* 5% - Simple Round Robin working
* 30% - Multi-Level Round Robin working
* 30% - Simple Multi-Level Feedback working
* 30% - Multi-Level Feedback working


So if you get all of them working, you'll get 100%.

## Hand In Instructions
You need to upload your *simple.rs*, *simplerr.rs*, *mlrr.rs*, *simplemlf.rs*, and *mlf.rs* files to the [INGInious submission](https://inginious.csuchico.edu/) to mark your completion time and have it test your submission. Just a reminder, if you didn't implement one of the schedulers, please make sure the initial code for that scheduler is left in that file, or you will likely cause the autograder to fail to grade your assignment, which is a 0.

## References
1. Wikipedia. “Scheduling (computing)”. Wikipedia, The Free Encyclopedia. 2012. http://en.wikipedia.org/wiki/Process_scheduler. Online; accessed 16-February-2014.
