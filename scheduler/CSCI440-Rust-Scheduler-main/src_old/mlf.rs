use crate::{Schedule, PCB, CLOCK};// <-- Import Job from crate root

pub struct MLFSchedule {
    implemented: bool,
}

impl MLFSchedule {
    /// Creates a new, instance of the MLFscheduler.
    ///
    /// # Returns
    /// A new [`MLFSchedule`] with the elements in its struct set to initial values.
    ///
    pub fn new() -> Self {
        Self {
            implemented: false,
        }
    }
}

impl Schedule for MLFSchedule {
    /// Adds a new process to the scheduler.
    ///
    /// # Parameters
    /// - `process`: A mutable [`PCB`] (Process Control Block) representing
    ///   the process to be added.
    ///
    /// # Returns
    /// - `true` if the process was successfully added.
    /// - `false` if the operation failed (e.g., queue full or invalid process).
    ///
    /// # Behavior
    /// If the scheduler has not been implemented yet (`self.implemented == false`),
    /// this method prints `"Not Implemented"` and terminates the program.
    /// You do not need to maintain this struct element or functionality if you implement this
    /// scheduler, but if you don't this is the behavior it should have when submitted for
    /// grading if not implemented.
    fn add_process(&mut self, mut process: PCB) -> bool{
        if !self.implemented {
            println!("Not Implemented");
            std::process::exit(0);
        }
        true
    }

    /// Retrieves the next process to run from the scheduler.
    ///
    /// # Returns
    /// A tuple `(Option<PCB>, u32)` where:
    /// - The first element is the next process to run, or `None` if no process is available.
    /// - The second element is a `u32` value (for example, representing the time slice,
    ///   priority, or cycle count associated with the returned process).
    ///
    /// # Behavior
    /// If the scheduler has not been implemented yet (`self.implemented == false`),
    /// this method prints `"Not Implemented"` and terminates the program.
    /// You do not need to maintain this struct element or functionality if you implement this
    /// scheduler, but if you don't this is the behavior it should have when submitted for
    /// grading if not implemented.
    fn next_process(&mut self) -> (Option<PCB>, u32){
        if !self.implemented {
            println!("Not Implemented");
            std::process::exit(0);
        }
        (None,0)
    }
    /// Checks whether the scheduler currently has any processes pending.
    ///
    /// # Returns
    /// - `true` if there is at least one process waiting to be scheduled.
    /// - `false` if there are no processes.
    ///
    /// # Behavior
    /// If the scheduler has not been implemented yet (`self.implemented == false`),
    /// this method prints `"Not Implemented"` and terminates the program.
    /// You do not need to maintain this struct element or functionality if you implement this
    /// scheduler, but if you don't this is the behavior it should have when submitted for
    /// grading if not implemented.
    fn has_process(&self) -> bool{
        if !self.implemented {
            println!("Not Implemented");
            std::process::exit(0);
        }
        false
    }
}

impl MLFSchedule {
    /// Handles an interrupt for the given process.
    ///
    /// This method is intended to manage cases where a running process
    /// is preempted or interrupted — for example, due to a timer interrupt,
    /// I/O completion, or a higher-priority process becoming ready.
    ///
    /// # Parameters
    /// - `process`: A mutable [`PCB`] (Process Control Block) representing
    ///   the process that was interrupted.
    /// - `priority`: The priority level associated with the interrupt or the
    ///   process being interrupted.
    ///
    /// # Returns
    /// - `true` if the process is to be interrupted
    /// - `false` otherwise.
    ///
    /// # Behavior
    /// Currently, this method is not implemented and always returns `false`.
    /// Implementations should determine if a process has exceed the max running time
    /// and if so implement the reverse feedback and return true that it should be interrupted
    pub fn interrupt(&mut self, mut process: PCB, mut priority: u32) -> bool{
        false
    }
    //Any additional helper functions you'd like to have
}