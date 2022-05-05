use std::sync::atomic::{AtomicUsize, Ordering};

/// An atomic counter which will wrap around at a specified value.
pub struct AtomicCounter {
    counter: AtomicUsize,
    counter_cap: usize,
    wrap_at: usize,
}

impl AtomicCounter {
    /// Cretae a new atomic counter.
    ///
    /// It will start at `0` and will wrap around at `wrap_at` and therefore never reach it.
    ///
    /// For example, if `wrap_at==4`, the highest value will be `3`, before starting over at `0`.
    pub fn new(wrap_at: usize) -> Self {
        Self {
            counter: AtomicUsize::new(0),
            counter_cap: usize::MAX - (usize::MAX % wrap_at),
            wrap_at,
        }
    }

    /// Adds `1` to the current value, returning the previous value.
    ///
    /// If the value was `wrap_at-1` the next value will be `0`.
    pub fn fetch_incr(&self) -> usize {
        // fetch the current value and increment the counter
        let current = self.counter.fetch_add(1, Ordering::AcqRel);

        // reset the counter if it reaches the cap
        let _ =
            self.counter
                .compare_exchange(self.counter_cap, 0, Ordering::AcqRel, Ordering::Relaxed);

        // ...
        current % self.wrap_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        let a = AtomicCounter::new(4);
        assert_eq!(a.fetch_incr(), 0);
        assert_eq!(a.fetch_incr(), 1);
        assert_eq!(a.fetch_incr(), 2);
        assert_eq!(a.fetch_incr(), 3);
        assert_eq!(a.fetch_incr(), 0);
    }
}
