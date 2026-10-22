//! # Clock Module
//!
//! This module provides a simple, thread-safe **simulation clock** used to
//! measure and manipulate time within CPU scheduling experiments.
//!
//! Instead of relying on the system clock, this clock maintains an internal
//! counter (in nanoseconds) that can be **manually advanced**. This makes it
//! ideal for educational simulations where time progression should be
//! deterministic and controlled.
//!
//! ## Purpose
//! In a real operating system, the scheduler interacts with hardware timers
//! to determine when to preempt processes or record CPU burst times.
//! This simulated clock allows you to:
//! - Track when processes are added or scheduled.
/// - Measure elapsed time between events.
/// - Control time manually for testing and debugging schedulers.
///
/// ## Example
/// ```
/// use your_crate_name::clock::{CLOCK, Clock};
/// use std::time::Duration;
///
/// // Reset the clock to zero
/// CLOCK.set_now(Duration::from_nanos(0));
///
/// // Advance the clock by 5 milliseconds
/// CLOCK.advance(Duration::from_millis(5));
///
/// // Get the current simulation time
/// let current = CLOCK.now();
/// println!("Simulated time: {:?}", current);
/// ```
///
/// ## Thread Safety
/// The [`Clock`] uses atomic operations internally, allowing multiple
/// threads to safely read or update the simulated time concurrently.
/// The assignment doesn't need this protection as it is not running
/// in a multi-threaded behavior but considered best practice for
/// use of a global static instance of the CLOCK

use std::sync::LazyLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// A thread-safe simulation clock that tracks virtual time in nanoseconds.
///
/// The `Clock` provides methods to read, update, and advance the simulated time.
/// It does **not** rely on the system clock; instead, it allows time to be
/// manually controlled for deterministic simulations.
///
/// # Fields
/// - `now_ns`: The current simulated time in nanoseconds, stored atomically.
///
/// # Example
/// ```
/// use your_crate_name::clock::Clock;
/// use std::time::Duration;
///
/// let clock = Clock::new();
/// clock.advance(Duration::from_millis(10));
/// assert_eq!(clock.now().as_millis(), 10);
/// ```
pub struct Clock {
    now_ns: AtomicU64,
}

impl Clock {
    /// Creates a new `Clock` instance initialized to zero nanoseconds.
    ///
    /// # Returns
    /// A new [`Clock`] starting at time `0`.
    ///
    /// # Example
    /// ```
    /// use your_crate_name::clock::Clock;
    /// let clock = Clock::new();
    /// assert_eq!(clock.now_ns(), 0);
    /// ```
    pub const fn new() -> Self {
        Self { now_ns: AtomicU64::new(0) }
    }

    /// Returns the current simulated time as a [`Duration`].
    ///
    /// # Example
    /// ```
    /// use your_crate_name::clock::Clock;
    /// use std::time::Duration;
    ///
    /// let clock = Clock::new();
    /// assert_eq!(clock.now(), Duration::from_nanos(0));
    /// ```
    pub fn now(&self) -> Duration {
        Duration::from_nanos(self.now_ns.load(Ordering::Relaxed))
    }

    /// Returns the current simulated time in nanoseconds as a `u64`.
    ///
    /// # Example
    /// ```
    /// use your_crate_name::clock::Clock;
    ///
    /// let clock = Clock::new();
    /// assert_eq!(clock.now_ns(), 0);
    /// ```
    pub fn now_ns(&self) -> u64 {
        self.now_ns.load(Ordering::Relaxed)
    }

    /// Sets the current simulated time to the given [`Duration`].
    ///
    /// # Parameters
    /// - `t`: The new time value to assign.
    ///
    /// # Example
    /// ```
    /// use your_crate_name::clock::Clock;
    /// use std::time::Duration;
    ///
    /// let clock = Clock::new();
    /// clock.set_now(Duration::from_nanos(500));
    /// assert_eq!(clock.now_ns(), 500);
    /// ```
    pub fn set_now(&self, t: Duration) {
        self.now_ns.store(t.as_nanos() as u64, Ordering::Relaxed);
    }

    /// Advances the simulated clock forward by the given [`Duration`].
    ///
    /// # Parameters
    /// - `dt`: The amount of time to advance.
    ///
    /// # Example
    /// ```
    /// use your_crate_name::clock::Clock;
    /// use std::time::Duration;
    ///
    /// let clock = Clock::new();
    /// clock.advance(Duration::from_millis(10));
    /// assert_eq!(clock.now().as_millis(), 10);
    /// ```
    pub fn advance(&self, dt: Duration) {
        self.now_ns.fetch_add(dt.as_nanos() as u64, Ordering::Relaxed);
    }

    /// Returns the [`Duration`] elapsed since a given start time (in nanoseconds).
    ///
    /// # Parameters
    /// - `start_ns`: The starting timestamp to measure from.
    ///
    /// # Returns
    /// The duration between the stored current time and the `start_ns` value.
    ///
    /// # Example
    /// ```
    /// use your_crate_name::clock::Clock;
    /// use std::time::Duration;
    ///
    /// let clock = Clock::new();
    /// clock.advance(Duration::from_micros(200));
    /// let elapsed = clock.elapsed_since_ns(100);
    /// assert!(elapsed.as_nanos() > 0);
    /// ```
    pub fn elapsed_since_ns(&self, start_ns: u64) -> Duration {
        let now = self.now_ns.load(Ordering::Relaxed);
        Duration::from_nanos(now - start_ns)
    }
}

/// A lazily initialized, global simulation clock instance.
///
/// [`CLOCK`] can be used across the entire project to represent a shared
/// notion of simulated time. It is safe for concurrent access.
///
/// # Example
/// ```
/// use your_crate_name::clock::CLOCK;
/// use std::time::Duration;
///
/// CLOCK.set_now(Duration::from_micros(500));
/// assert_eq!(CLOCK.now().as_micros(), 500);
/// ```
pub static CLOCK: LazyLock<Clock> = LazyLock::new(|| Clock::new());
