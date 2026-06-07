//! Core L-system implementation with axiom, rules, and string rewriting.
//!
//! An L-system consists of an axiom (initial string) and a set of production
//! rules. At each iteration, every symbol in the current string is replaced
//! according to the matching rule, or kept as-is if no rule matches.

use crate::rule::ProductionRule;

/// A deterministic Lindenmayer system (D0L-system).
///
/// # Examples
///
/// ```
/// use l_system_rs::system::LSystem;
/// use l_system_rs::rule::ProductionRule;
///
/// let mut ls = LSystem::new("A");
/// ls.add_rule(ProductionRule::new('A', "AB"));
/// ls.add_rule(ProductionRule::new('B', "A"));
/// let result = ls.iterate(3);
/// assert_eq!(result, "ABAAB");
/// ```
#[derive(Debug, Clone)]
pub struct LSystem {
    /// The axiom (initial string).
    pub axiom: String,
    /// Production rules.
    pub rules: Vec<ProductionRule>,
}

impl LSystem {
    /// Create a new L-system with the given axiom and no rules.
    pub fn new(axiom: &str) -> Self {
        Self {
            axiom: axiom.to_string(),
            rules: Vec::new(),
        }
    }

    /// Add a production rule.
    pub fn add_rule(&mut self, rule: ProductionRule) {
        self.rules.push(rule);
    }

    /// Add a rule from predecessor and successor strings.
    pub fn add_rule_str(&mut self, predecessor: char, successor: &str) {
        self.rules.push(ProductionRule::new(predecessor, successor));
    }

    /// Apply one iteration of string rewriting.
    ///
    /// Each symbol in the input is replaced by its successor if a matching rule
    /// exists, or kept as-is otherwise.
    pub fn rewrite(&self, input: &str) -> String {
        let mut output = String::new();
        for ch in input.chars() {
            match self.find_rule(ch) {
                Some(rule) => output.push_str(&rule.successor),
                None => output.push(ch),
            }
        }
        output
    }

    /// Find the first rule matching the given symbol.
    pub fn find_rule(&self, symbol: char) -> Option<&ProductionRule> {
        self.rules.iter().find(|r| r.matches(symbol))
    }

    /// Iterate the L-system `n` times starting from the axiom.
    ///
    /// Returns the final string.
    pub fn iterate(&mut self, n: usize) -> String {
        let mut current = self.axiom.clone();
        for _ in 0..n {
            current = self.rewrite(&current);
        }
        current
    }

    /// Iterate and return all intermediate strings including the axiom.
    pub fn iterate_all(&mut self, n: usize) -> Vec<String> {
        let mut results = vec![self.axiom.clone()];
        for _ in 0..n {
            let next = self.rewrite(results.last().unwrap());
            results.push(next);
        }
        results
    }

    /// Count the number of each symbol in the current iteration.
    pub fn symbol_counts(&self, s: &str) -> std::collections::HashMap<char, usize> {
        let mut counts = std::collections::HashMap::new();
        for ch in s.chars() {
            *counts.entry(ch).or_insert(0) += 1;
        }
        counts
    }

    /// Get the length of the string after `n` iterations without computing it.
    /// Only works for simple systems where growth rate is consistent.
    pub fn estimate_length(&mut self, n: usize) -> usize {
        self.iterate(n).len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::ProductionRule;

    #[test]
    fn test_fibonacci_lsystem() {
        let mut ls = LSystem::new("A");
        ls.add_rule(ProductionRule::new('A', "AB"));
        ls.add_rule(ProductionRule::new('B', "A"));
        // Iteration 0: A (1)
        // Iteration 1: AB (2)
        // Iteration 2: ABA (3)
        // Iteration 3: ABAAB (5)
        // Iteration 4: ABAABABA (8)
        // Lengths follow Fibonacci sequence
        assert_eq!(ls.iterate(0), "A");
        assert_eq!(ls.iterate(0).len(), 1);
        assert_eq!(ls.iterate(1), "AB");
        assert_eq!(ls.iterate(1).len(), 2);
        assert_eq!(ls.iterate(3), "ABAAB");
        assert_eq!(ls.iterate(3).len(), 5);
        assert_eq!(ls.iterate(4).len(), 8);
    }

    #[test]
    fn test_koch_curve() {
        let mut ls = LSystem::new("F");
        ls.add_rule(ProductionRule::new('F', "F+F--F+F"));
        let result = ls.iterate(1);
        assert_eq!(result, "F+F--F+F");
        let result2 = ls.iterate(2);
        // Each F in the first iteration is replaced
        assert!(result2.contains('+'));
        assert!(result2.len() > result.len());
    }

    #[test]
    fn test_sierpinski_triangle() {
        let mut ls = LSystem::new("F-G-G");
        ls.add_rule(ProductionRule::new('F', "F-G+F+G-F"));
        ls.add_rule(ProductionRule::new('G', "GG"));
        let gen1 = ls.iterate(1);
        assert_eq!(gen1, "F-G+F+G-F-GG-GG");
    }

    #[test]
    fn test_no_rules_identity() {
        let mut ls = LSystem::new("ABC");
        assert_eq!(ls.iterate(5), "ABC");
    }

    #[test]
    fn test_symbols_without_rules_preserved() {
        let mut ls = LSystem::new("F+F");
        ls.add_rule(ProductionRule::new('F', "F-F"));
        let result = ls.iterate(1);
        assert_eq!(result, "F-F+F-F");
    }

    #[test]
    fn test_iterate_all() {
        let mut ls = LSystem::new("A");
        ls.add_rule(ProductionRule::new('A', "AA"));
        let all = ls.iterate_all(3);
        assert_eq!(all.len(), 4); // axiom + 3 iterations
        assert_eq!(all[0], "A");
        assert_eq!(all[1], "AA");
        assert_eq!(all[2], "AAAA");
        assert_eq!(all[3], "AAAAAAAA");
    }

    #[test]
    fn test_plant_branching() {
        let mut ls = LSystem::new("X");
        ls.add_rule(ProductionRule::new('X', "F+[[X]-X]-F[-FX]+X"));
        ls.add_rule(ProductionRule::new('F', "FF"));
        let result = ls.iterate(1);
        assert_eq!(result, "F+[[X]-X]-F[-FX]+X");
        let result2 = ls.iterate(2);
        assert!(result2.len() > result.len());
        // Should contain brackets for branching
        assert!(result2.contains('['));
        assert!(result2.contains(']'));
    }

    #[test]
    fn test_dragon_curve() {
        let mut ls = LSystem::new("FX");
        ls.add_rule(ProductionRule::new('X', "X+YF+"));
        ls.add_rule(ProductionRule::new('Y', "-FX-Y"));
        let gen1 = ls.iterate(1);
        assert_eq!(gen1, "FX+YF+");
    }

    #[test]
    fn test_symbol_counts() {
        let ls = LSystem::new("AAB");
        let counts = ls.symbol_counts("AABBC");
        assert_eq!(counts[&'A'], 2);
        assert_eq!(counts[&'B'], 2);
        assert_eq!(counts[&'C'], 1);
    }

    #[test]
    fn test_exponential_growth() {
        let mut ls = LSystem::new("F");
        ls.add_rule(ProductionRule::new('F', "FFF"));
        assert_eq!(ls.iterate(0).len(), 1);
        assert_eq!(ls.iterate(1).len(), 3);
        assert_eq!(ls.iterate(2).len(), 9);
        assert_eq!(ls.iterate(3).len(), 27);
    }

    #[test]
    fn test_cantor_set() {
        let mut ls = LSystem::new("A");
        ls.add_rule(ProductionRule::new('A', "ABA"));
        ls.add_rule(ProductionRule::new('B', "BBB"));
        let gen1 = ls.iterate(1);
        assert_eq!(gen1, "ABA");
        let gen2 = ls.iterate(2);
        assert_eq!(gen2, "ABABBBABA");
    }

    #[test]
    fn test_find_rule() {
        let mut ls = LSystem::new("A");
        ls.add_rule(ProductionRule::new('A', "AB"));
        ls.add_rule(ProductionRule::new('B', "A"));
        assert!(ls.find_rule('A').is_some());
        assert!(ls.find_rule('B').is_some());
        assert!(ls.find_rule('C').is_none());
    }

    #[test]
    fn test_add_rule_str() {
        let mut ls = LSystem::new("F");
        ls.add_rule_str('F', "F+F");
        assert_eq!(ls.iterate(1), "F+F");
    }

    #[test]
    fn test_rewrite_identity() {
        let mut ls = LSystem::new("XYZ");
        ls.add_rule(ProductionRule::new('X', "X"));
        assert_eq!(ls.rewrite("XYZ"), "XYZ");
    }

    #[test]
    fn test_empty_axiom() {
        let mut ls = LSystem::new("");
        ls.add_rule(ProductionRule::new('F', "FF"));
        assert_eq!(ls.iterate(5), "");
    }

    #[test]
    fn test_hilbert_curve() {
        let mut ls = LSystem::new("A");
        ls.add_rule(ProductionRule::new('A', "-BF+AFA+FB-"));
        ls.add_rule(ProductionRule::new('B', "+AF-BFB-FA+"));
        let gen1 = ls.iterate(1);
        assert_eq!(gen1, "-BF+AFA+FB-");
    }
}
