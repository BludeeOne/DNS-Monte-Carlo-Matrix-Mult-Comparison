//! # CPU Scheduling Simulation Library
//!
//! This library provides the basic framework for implementing and comparing
//! different **CPU scheduling algorithms** as part of the Scheduling Assignment
//!
//! It defines shared data structures, traits, and helper modules used across
//! several scheduling strategies — including simple, round-robin, and
//! multi-level feedback queue (MLFQ) schedulers.
//!
//! ## Purpose
//! The goal of this library is to help students and developers understand how
//! different scheduling algorithms make decisions about process management,
//! fairness, and CPU utilization. Each module can be implemented independently
//! to explore how specific strategies behave under various workloads.
//!
//! ## Key Components
//! - [`PCB`]: Represents a **Process Control Block**, holding information about
//!   each simulated process.
//! - [`Schedule`]: A **trait** that defines the standard interface all schedulers
//!   must follow (adding, selecting, and checking for processes).
//! - [`Clock`]: A helper for tracking simulated time within your scheduler.
//!
//! ## Scheduler Modules
//! The following modules represent different scheduling strategies. You can
//! implement and experiment with them individually:
//! - [`simple`]: Basic scheduler to test the interface and structure.
//! - [`simplerr`]: A simple **Round Robin** scheduler.
//! - [`mlrr`]: **Multi-Level Round Robin** scheduler for layered priorities.
//! - [`simplemlf`]: Simplified **Multi-Level Feedback Queue (MLFQ)** scheduler.
//! - [`mlf`]: Full **MLFQ** scheduler for advanced scheduling experiments.


/// Represents a **Process Control Block (PCB)** for a simulated process.
///
/// Each `PCB` stores the essential attributes of a process, such as its ID,
/// priority level, and optional timing information. These values can be used
/// to determine scheduling order and performance metrics.
///
/// # Fields
/// - `id`: Unique identifier for the process.
/// - `priority`: Current priority level of the process.
/// - `time_added`: Time (in simulation ticks) when the process was added.
/// - `time_scheduled`: Time (in simulation ticks) when the process was last scheduled.
#[derive(Debug, Clone, Copy)]
pub struct PCB {
    pub id: u32,
    pub priority: u32,
    pub time_added: Option<u64>,
    pub time_scheduled: Option<u64>,
}

/// Defines the **common interface** for all CPU scheduling algorithms.
///
/// Every scheduler in this project implements the `Schedule` trait, which
/// provides the basic functions needed to manage a set of processes.
///
/// # Required Methods
/// - [`add_process`]: Adds a new process to the scheduler.
/// - [`next_process`]: Retrieves the next process to execute.
/// - [`has_process`]: Checks whether there are any remaining processes.
pub trait Schedule {
    fn add_process(&mut self, process: PCB) -> bool;
    fn next_process(&mut self) -> (Option<PCB>, u32);
    fn has_process(&self) -> bool;
}

/// Provides timing utilities for simulated scheduling operations.
///
/// The [`Clock`] module can be used to track the current simulation time,
/// record when processes are added, and measure CPU burst durations.
pub mod clock;
pub use clock::{CLOCK, Clock};

/// Contains a basic scheduler implementation template.
pub mod simple;

/// Contains a simplified **Round Robin** scheduler.
pub mod simplerr;

/// Contains a **Multi-Level Round Robin (MLRR)** scheduler implementation.
pub mod mlrr;

/// Contains a simplified **Multi-Level Feedback Queue (MLFQ)** scheduler.
pub mod simplemlf;

/// Contains a more complete **MLFQ** scheduler implementation.
pub mod mlf;
