
/// Helper struct for `step_by`.
/// In a cycle, emits a boolean `true` value, then <skip_amount> `false` values.
pub struct StepByEmitter {
    counter: usize,
    skip_amount: usize,
}

impl StepByEmitter {
    pub fn new(skip_amount: usize) -> Self {
        Self { counter: 0, skip_amount }
    }

    pub fn step(&mut self) -> bool {
        if self.counter == 0 {
            // Reset the counter, and emit true.
            self.counter = self.skip_amount;
            true
        }
        else {
            // Decrement the counter, and emit false.
            self.counter -= 1;
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step() {
        for skip_amount in 0usize..100 {
            let mut emitter = StepByEmitter::new(skip_amount);
            for i in 0..1000 {
                if i % (skip_amount + 1) == 0 { assert!(emitter.step()); }
                else { assert!(!emitter.step()); }
            }
        }
    }
}
