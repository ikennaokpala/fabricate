use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Thread-safe auto-incrementing sequence generator.
///
/// Each named sequence maintains its own independent counter.
pub struct Sequence {
    counters: HashMap<String, AtomicU64>,
}

impl Sequence {
    pub fn new() -> Self {
        Self {
            counters: HashMap::new(),
        }
    }

    /// Get next value for a named sequence, starting from 1.
    pub fn next(&mut self, name: &str) -> u64 {
        let counter = self
            .counters
            .entry(name.to_string())
            .or_insert_with(|| AtomicU64::new(0));
        counter.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Generate a unique test email address.
    pub fn email(&mut self, prefix: &str) -> String {
        let n = self.next("email");
        format!("test_{prefix}_{n}@test.ridemate.app")
    }

    /// Generate a unique test phone number (555 area code).
    pub fn phone(&mut self) -> String {
        let n = self.next("phone");
        format!("+1555{n:07}")
    }

    /// Generate a unique full name.
    pub fn full_name(&mut self) -> String {
        let n = self.next("name");
        let first_names = [
            "Alice", "Bob", "Carol", "Dave", "Eve", "Frank", "Grace", "Hank", "Iris", "Jack",
        ];
        let last_names = [
            "Smith", "Jones", "Brown", "Davis", "Wilson", "Taylor", "Clark", "Hall", "Lee",
            "Young",
        ];
        let first = first_names[(n as usize - 1) % first_names.len()];
        let last = last_names[(n as usize - 1) % last_names.len()];
        format!("{first} {last}")
    }

    /// Generate a test license plate number.
    pub fn plate(&mut self) -> String {
        let n = self.next("plate");
        let letters = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'];
        let l1 = letters[(n as usize - 1) % letters.len()];
        let l2 = letters[((n as usize - 1) / letters.len()) % letters.len()];
        format!("{l1}{l2}{n:04}")
    }

    /// Reset all sequences to zero.
    pub fn reset(&mut self) {
        self.counters.clear();
    }
}

impl Default for Sequence {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequence_increments() {
        let mut seq = Sequence::new();
        assert_eq!(seq.next("test"), 1);
        assert_eq!(seq.next("test"), 2);
        assert_eq!(seq.next("other"), 1);
    }

    #[test]
    fn test_email_generation() {
        let mut seq = Sequence::new();
        assert_eq!(seq.email("rider"), "test_rider_1@test.ridemate.app");
        assert_eq!(seq.email("rider"), "test_rider_2@test.ridemate.app");
    }

    #[test]
    fn test_phone_generation() {
        let mut seq = Sequence::new();
        assert_eq!(seq.phone(), "+15550000001");
        assert_eq!(seq.phone(), "+15550000002");
    }
}
