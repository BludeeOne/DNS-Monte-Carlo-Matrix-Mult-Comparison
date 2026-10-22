use crate::{Schedule, PCB};
use std::collections::VecDeque;

pub struct SimpleSchedule {
    //decided on vecdeque
    queue: VecDeque<PCB>,
}

impl SimpleSchedule {
    /// Creates a new, instance of the SimpleScheduler.
    ///
    /// # Returns
    /// A new [`SimpleSchedule`] with the elements in its struct set to initial values.
    ///
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
         }
    }
}

impl Schedule for SimpleSchedule {
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
    fn add_process(&mut self, process: PCB) -> bool{
        //push onto back
        self.queue.push_back(process);
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
        //grab front
        (self.queue.pop_front(), 0)
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
        !self.queue.is_empty()
    }
}   
