//! Production rules for L-system rewriting.
//!
//! A production rule maps a predecessor symbol to a successor string.
//! In deterministic L-systems, each symbol has at most one matching rule.
//! In stochastic/parametric systems, multiple rules may apply to the same symbol.

/// A deterministic production rule: `predecessor → successor`.
///
/// # Examples
///
/// ```
/// use l_system_rs::rule::ProductionRule;
///
/// let rule = ProductionRule::new('F', "F+F--F+F");
/// assert_eq!(rule.predecessor, 'F');
/// assert_eq!(rule.successor, "F+F--F+F");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionRule {
    /// The symbol that this rule replaces.
    pub predecessor: char,
    /// The string that replaces the predecessor.
    pub successor: String,
}

impl ProductionRule {
    /// Create a new production rule.
    pub fn new(predecessor: char, successor: &str) -> Self {
        Self {
            predecessor,
            successor: successor.to_string(),
        }
    }

    /// Check if this rule applies to the given symbol.
    pub fn matches(&self, symbol: char) -> bool {
        self.predecessor == symbol
    }
}

/// A stochastic production rule with an associated probability.
///
/// For a given predecessor, the sum of probabilities across all matching
/// stochastic rules should be 1.0.
#[derive(Debug, Clone)]
pub struct StochasticRule {
    /// The symbol that this rule replaces.
    pub predecessor: char,
    /// The string that replaces the predecessor.
    pub successor: String,
    /// Probability of this rule being selected (0.0 to 1.0).
    pub probability: f64,
}

impl StochasticRule {
    /// Create a new stochastic rule with the given probability.
    pub fn new(predecessor: char, successor: &str, probability: f64) -> Self {
        assert!(
            (0.0..=1.0).contains(&probability),
            "Probability must be in [0, 1]"
        );
        Self {
            predecessor,
            successor: successor.to_string(),
            probability,
        }
    }
}

/// A parametric production rule with a condition and parameter expressions.
///
/// Parametric L-systems operate on modules with numerical parameters.
/// A rule matches when the predecessor symbol and condition match,
/// and the successor uses expressions involving the parameters.
#[derive(Debug, Clone)]
pub struct ParametricRule {
    /// The predecessor symbol.
    pub predecessor: char,
    /// Number of parameters expected.
    pub param_count: usize,
    /// Condition expression: a function that takes parameter values and returns whether the rule applies.
    pub condition: fn(&[f64]) -> bool,
    /// Successor generator: takes parameters and produces the successor string.
    /// Parameters are referenced as `$0`, `$1`, etc. in the template.
    pub successor_template: String,
}

impl ParametricRule {
    /// Create a new parametric rule.
    pub fn new(
        predecessor: char,
        param_count: usize,
        condition: fn(&[f64]) -> bool,
        successor_template: &str,
    ) -> Self {
        Self {
            predecessor,
            param_count,
            condition,
            successor_template: successor_template.to_string(),
        }
    }

    /// Check if this rule applies to the given symbol with given parameters.
    pub fn matches_with_params(&self, symbol: char, params: &[f64]) -> bool {
        symbol == self.predecessor
            && params.len() == self.param_count
            && (self.condition)(params)
    }

    /// Generate the successor string by substituting parameter values.
    ///
    /// `$0` is replaced by params[0], `$1` by params[1], etc.
    pub fn generate_successor(&self, params: &[f64]) -> String {
        let mut result = self.successor_template.clone();
        for (i, &val) in params.iter().enumerate() {
            let placeholder = format!("${i}");
            // Round to avoid floating-point noise
            let rounded = if val.fract() == 0.0 {
                format!("{}", val as i64)
            } else {
                format!("{val:.4}")
            };
            result = result.replace(&placeholder, &rounded);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_production_rule_new() {
        let rule = ProductionRule::new('A', "AB");
        assert_eq!(rule.predecessor, 'A');
        assert_eq!(rule.successor, "AB");
    }

    #[test]
    fn test_production_rule_matches() {
        let rule = ProductionRule::new('F', "F+F");
        assert!(rule.matches('F'));
        assert!(!rule.matches('G'));
    }

    #[test]
    fn test_production_rule_equality() {
        let r1 = ProductionRule::new('A', "AB");
        let r2 = ProductionRule::new('A', "AB");
        let r3 = ProductionRule::new('A', "BA");
        assert_eq!(r1, r2);
        assert_ne!(r1, r3);
    }

    #[test]
    fn test_stochastic_rule_new() {
        let rule = StochasticRule::new('F', "F+F", 0.7);
        assert_eq!(rule.predecessor, 'F');
        assert!((rule.probability - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    #[should_panic(expected = "Probability must be in [0, 1]")]
    fn test_stochastic_rule_invalid_prob() {
        StochasticRule::new('F', "F+F", 1.5);
    }

    #[test]
    fn test_parametric_rule_basic() {
        let rule = ParametricRule::new('F', 1, |_| true, "F($0+1)");
        assert_eq!(rule.predecessor, 'F');
        assert!(rule.matches_with_params('F', &[1.0]));
        assert!(!rule.matches_with_params('G', &[1.0]));
    }

    #[test]
    fn test_parametric_rule_condition() {
        let rule = ParametricRule::new('F', 1, |p| p[0] > 2.0, "F($0-1)");
        assert!(!rule.matches_with_params('F', &[1.0]));
        assert!(rule.matches_with_params('F', &[3.0]));
    }

    #[test]
    fn test_parametric_rule_generate_successor() {
        let rule = ParametricRule::new('F', 2, |_| true, "F($0)F($1)");
        let result = rule.generate_successor(&[3.0, 5.0]);
        assert_eq!(result, "F(3)F(5)");
    }

    #[test]
    fn test_parametric_rule_generate_fractional() {
        let rule = ParametricRule::new('F', 1, |_| true, "F($0)");
        let result = rule.generate_successor(&[1.5]);
        assert_eq!(result, "F(1.5000)");
    }

    #[test]
    fn test_parametric_rule_wrong_param_count() {
        let rule = ParametricRule::new('F', 2, |_| true, "F($0)");
        assert!(!rule.matches_with_params('F', &[1.0])); // only 1 param, expects 2
    }
}
