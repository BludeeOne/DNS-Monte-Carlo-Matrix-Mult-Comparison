use std::collections::VecDeque;
use crate::{Schedule, PCB, CLOCK};

const NUM_LEVELS: usize = 3;
// Level 0 = FCFS (run to completion, quantum 0)
// Level 1 = RR quanta 4
// Level 2 = RR quanta 1
const QUANTA: [u32; NUM_LEVELS] = [0, 4, 1];
const STARVATION_THRESHOLD: u64 = 1000;

pub struct SimpleMLFSchedule {
    queues: Vec<VecDeque<PCB>>,
}

impl SimpleMLFSchedule {
    pub fn new() -> Self {
        Self {
            queues: (0..NUM_LEVELS).map(|_| VecDeque::new()).collect(),
        }
    }
}

impl Schedule for SimpleMLFSchedule {
    fn add_process(&mut self, mut process: PCB) -> bool {
        process.time_added = Some(CLOCK.now_ns());
        let level = (process.priority as usize).min(NUM_LEVELS - 1);
        self.queues[level].push_back(process);
        true
    }

    fn next_process(&mut self) -> (Option<PCB>, u32) {
        self.check_promotions();
        for level in 0..NUM_LEVELS {
            if !self.queues[level].is_empty() {
                let mut process = self.queues[level].pop_front().unwrap();
                process.time_scheduled = Some(CLOCK.now_ns());
                return (Some(process), QUANTA[level]);
            }
        }
        (None, 0)
    }

    fn has_process(&self) -> bool {
        self.queues.iter().any(|q| !q.is_empty())
    }
}

impl SimpleMLFSchedule {
    fn check_promotions(&mut self) {
        let now = CLOCK.now_ns();
        for level in (1..NUM_LEVELS).rev() {
            let mut i = 0;
            while i < self.queues[level].len() {
                let time_added = self.queues[level][i].time_added.unwrap_or(now);
                if now.saturating_sub(time_added) >= STARVATION_THRESHOLD {
                    let mut proc = self.queues[level].remove(i).unwrap();
                    proc.time_added = Some(CLOCK.now_ns());
                    self.queues[level - 1].push_back(proc);
                } else {
                    i += 1;
                }
            }
        }
    }

   
    pub fn interrupt(&mut self, process: PCB, priority: u32) -> bool {
        let current_level = priority as usize;
        // If any higher priority queue has a process, preempt
        for level in 0..current_level {
            if !self.queues[level].is_empty() {
                // Push current process back to front of its level to resume later
                self.queues[current_level].push_front(process);
                return true;
            }
        }
        false
    }
}