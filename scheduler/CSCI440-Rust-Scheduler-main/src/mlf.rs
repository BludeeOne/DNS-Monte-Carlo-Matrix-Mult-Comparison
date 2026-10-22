use crate::{Schedule, PCB, CLOCK};
use std::collections::VecDeque;

const QUANTA: [u32; 3] = [0, 4, 1];
const STARVE_LIMIT: u64 = 1000;
const DEMOTE_LIMIT: u64 = 1002;

pub struct MLFSchedule {
    queues: [VecDeque<PCB>; 3],
    demotion_occurred: bool,
}

impl MLFSchedule {
    pub fn new() -> Self {
        Self {
            queues: [
                VecDeque::new(),
                VecDeque::new(),
                VecDeque::new(),
            ],
            demotion_occurred: false,
        }
    }

    fn promote(&mut self) {
        let now = CLOCK.now_ns();
        for level in 1..3 {
            let mut i = 0;
            while i < self.queues[level].len() {
                let time_added = self.queues[level][i].time_added.unwrap_or(now);
                let starved = now.saturating_sub(time_added) >= STARVE_LIMIT;
                let just_added = time_added == now;
                let should_promote = !just_added && (starved || (self.demotion_occurred && level == 1));
                if should_promote {
                    let mut process = self.queues[level].remove(i).unwrap();
                    let new_level = level - 1;
                    if self.demotion_occurred {
                        println!("Process {} promoted to {}", process.id, new_level);
                    }
                    process.priority = new_level as u32;
                    process.time_added = Some(now);
                    process.time_scheduled = Some(u64::MAX);
                    self.queues[new_level].push_back(process);
                } else {
                    i += 1;
                }
            }
        }
        self.demotion_occurred = false;
    }
}

impl Schedule for MLFSchedule {
    fn add_process(&mut self, mut process: PCB) -> bool {
        let now = CLOCK.now_ns();
        let current = (process.priority as usize).min(2);

        let level = if process.time_scheduled == Some(u64::MAX) && current > 0 {
            // Was promoted and has now run its allotment - demote back
            let next = (current + 1).min(2);
            process.priority = next as u32;
            process.time_added = Some(now);
            next
        } else if process.time_scheduled.is_some() {
            // Normal run - reset starvation clock, stay at same level
            process.time_added = Some(now);
            current
        } else if process.time_added.is_none() {
            // Fresh process
            process.time_added = Some(now);
            current
        } else {
            current
        };

        process.time_scheduled = None;
        self.queues[level].push_back(process);
        true
    }

    fn next_process(&mut self) -> (Option<PCB>, u32) {
        self.promote();
        for level in 0..3 {
            if let Some(mut process) = self.queues[level].pop_front() {
                if process.time_scheduled != Some(u64::MAX) {
                    process.time_scheduled = Some(CLOCK.now_ns());
                }
                let quantum = QUANTA[process.priority as usize];
                return (Some(process), quantum);
            }
        }
        (None, 0)
    }

    fn has_process(&self) -> bool {
        self.queues.iter().any(|q| !q.is_empty())
    }
}

impl MLFSchedule {
    pub fn interrupt(&mut self, process: PCB, _priority: u32) -> bool {
        if let Some(ts) = process.time_scheduled {
            if ts != u64::MAX {
                let running_time = CLOCK.now_ns().saturating_sub(ts);
                if running_time >= DEMOTE_LIMIT {
                    let next_level = 1usize;
                    let mut demoted = process;
                    println!("Process {} demoted to {}", demoted.id, next_level);
                    demoted.priority = next_level as u32;
                    demoted.time_added = Some(CLOCK.now_ns());
                    demoted.time_scheduled = None;
                    self.queues[next_level].push_back(demoted);
                    self.demotion_occurred = true;
                    return true;
                }
            }
        }
        false
    }
}