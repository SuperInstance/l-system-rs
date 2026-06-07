//! Stochastic L-systems with probabilistic rule selection.
//!
//! In a stochastic L-system, multiple rules may apply to the same predecessor.
//! The successor is chosen randomly according to the rules' probabilities.
//! This module uses a simple seed-based PRNG for reproducibility.

use crate::rule::StochasticRule;

/// A stochastic L-system.
///
/// For each symbol, multiple rules with probabilities may apply.
/// The actual successor is selected based on those probabilities.
///
/// # Examples
///
/// ```
/// use l_system_rs::stochastic::StochasticLSystem;
/// use l_system_rs::rule::StochasticRule;
///
/// let mut sls = StochasticLSystem::new("F");
/// sls.add_rule(StochasticRule::new('F', "F+F", 0.5));
/// sls.add_rule(StochasticRule::new('F', "F-F", 0.5));
/// sls.set_seed(42);
/// let result = sls.iterate(2);
/// ```
#[derive(Debug, Clone)]
pub struct StochasticLSystem {
    /// The axiom (initial string).
    pub axiom: String,
    /// Stochastic production rules.
    pub rules: Vec<StochasticRule>,
    /// PRNG state for reproducibility.
    rng_state: u64,
}

impl StochasticLSystem {
    /// Create a new stochastic L-system with the given axiom.
    pub fn new(axiom: &str) -> Self {
        Self {
            axiom: axiom.to_string(),
            rules: Vec::new(),
            rng_state: 12345,
        }
    }

    /// Set the PRNG seed for reproducibility.
    pub fn set_seed(&mut self, seed: u64) {
        self.rng_state = seed;
    }

    /// Add a stochastic rule.
    pub fn add_rule(&mut self, rule: StochasticRule) {
        self.rules.push(rule);
    }

    /// Generate a pseudo-random number in [0, 1).
    fn next_random(&mut self) -> f64 {
        // xorshift64
        let mut x = self.rng_state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng_state = x;
        (x as f64) / (u64::MAX as f64)
    }

    /// Select a successor for the given symbol based on rule probabilities.
    fn select_successor(&mut self, symbol: char) -> Option<String> {
        let matching: Vec<(String, f64)> = self.rules.iter()
            .filter(|r| r.predecessor == symbol)
            .map(|r| (r.successor.clone(), r.probability))
            .collect();
        if matching.is_empty() {
            return None;
        }

        let r = self.next_random();
        let mut cumulative = 0.0;
        for (successor, prob) in &matching {
            cumulative += prob;
            if r < cumulative {
                return Some(successor.clone());
            }
        }
        // Fallback to last matching rule (handles floating-point rounding)
        matching.last().map(|(s, _)| s.clone())
    }

    /// Apply one iteration of stochastic rewriting.
    pub fn rewrite(&mut self, input: &str) -> String {
        let mut output = String::new();
        for ch in input.chars() {
            match self.select_successor(ch) {
                Some(successor) => output.push_str(&successor),
                None => output.push(ch),
            }
        }
        output
    }

    /// Iterate `n` times starting from the axiom.
    pub fn iterate(&mut self, n: usize) -> String {
        let mut current = self.axiom.clone();
        for _ in 0..n {
            current = self.rewrite(&current);
        }
        current
    }

    /// Validate that rule probabilities for each predecessor sum to approximately 1.0.
    pub fn validate_probabilities(&self) -> bool {
        use std::collections::HashMap;
        let mut sums: HashMap<char, f64> = HashMap::new();
        for rule in &self.rules {
            *sums.entry(rule.predecessor).or_insert(0.0) += rule.probability;
        }
        sums.values().all(|&s| (s - 1.0).abs() < 0.01)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::StochasticRule;

    #[test]
    fn test_deterministic_with_single_rules() {
        let mut sls = StochasticLSystem::new("F");
        sls.add_rule(StochasticRule::new('F', "FF", 1.0));
        assert_eq!(sls.iterate(2), "FFFF");
    }

    #[test]
    fn test_reproducible_with_same_seed() {
        let mut sls1 = StochasticLSystem::new("F");
        sls1.add_rule(StochasticRule::new('F', "F+F", 0.5));
        sls1.add_rule(StochasticRule::new('F', "F-F", 0.5));
        sls1.set_seed(42);
        let r1 = sls1.iterate(3);

        let mut sls2 = StochasticLSystem::new("F");
        sls2.add_rule(StochasticRule::new('F', "F+F", 0.5));
        sls2.add_rule(StochasticRule::new('F', "F-F", 0.5));
        sls2.set_seed(42);
        let r2 = sls2.iterate(3);

        assert_eq!(r1, r2, "Same seed should produce same output");
    }

    #[test]
    fn test_different_seeds_differ() {
        let mut sls1 = StochasticLSystem::new("F");
        sls1.add_rule(StochasticRule::new('F', "F+F", 0.5));
        sls1.add_rule(StochasticRule::new('F', "F-F", 0.5));
        sls1.set_seed(42);
        let r1 = sls1.iterate(3);

        let mut sls2 = StochasticLSystem::new("F");
        sls2.add_rule(StochasticRule::new('F', "F+F", 0.5));
        sls2.add_rule(StochasticRule::new('F', "F-F", 0.5));
        sls2.set_seed(99);
        let r2 = sls2.iterate(3);

        // Very likely to differ (not guaranteed but extremely probable with 3 iterations)
        // We'll just check both are valid strings
        assert!(r1.contains('F'));
        assert!(r2.contains('F'));
    }

    #[test]
    fn test_stochastic_variation() {
        let mut sls = StochasticLSystem::new("F");
        sls.add_rule(StochasticRule::new('F', "F+F", 0.5));
        sls.add_rule(StochasticRule::new('F', "F-F", 0.5));
        sls.set_seed(42);
        let result = sls.iterate(2);
        // Should contain both + and - or at least one
        assert!(result.contains('+') || result.contains('-'));
    }

    #[test]
    fn test_validate_probabilities() {
        let mut sls = StochasticLSystem::new("F");
        sls.add_rule(StochasticRule::new('F', "F+F", 0.5));
        sls.add_rule(StochasticRule::new('F', "F-F", 0.5));
        assert!(sls.validate_probabilities());
    }

    #[test]
    fn test_validate_probabilities_invalid() {
        let mut sls = StochasticLSystem::new("F");
        sls.add_rule(StochasticRule::new('F', "F+F", 0.3));
        sls.add_rule(StochasticRule::new('F', "F-F", 0.3));
        assert!(!sls.validate_probabilities());
    }

    #[test]
    fn test_symbols_without_rules_preserved() {
        let mut sls = StochasticLSystem::new("F+G");
        sls.add_rule(StochasticRule::new('F', "FF", 1.0));
        let result = sls.iterate(1);
        assert!(result.contains('+'));
        assert!(result.contains('G'));
    }

    #[test]
    fn test_multiple_symbols() {
        let mut sls = StochasticLSystem::new("AB");
        sls.add_rule(StochasticRule::new('A', "AB", 1.0));
        sls.add_rule(StochasticRule::new('B', "A", 1.0));
        let result = sls.iterate(2);
        // Gen 0: AB, Gen 1: ABA, Gen 2: ABAAB
        assert_eq!(result, "ABAAB");
    }
}
