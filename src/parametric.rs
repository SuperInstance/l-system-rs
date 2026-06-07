//! Parametric L-systems with numerical parameters and conditional rules.
//!
//! Parametric L-systems extend basic L-systems by associating numerical parameters
//! with modules (symbols). Rules can check conditions on parameters and use
//! arithmetic expressions in successor strings.

use crate::rule::ParametricRule;

/// A module in a parametric L-system: a symbol with associated parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    /// The symbol.
    pub symbol: char,
    /// Associated numerical parameters.
    pub params: Vec<f64>,
}

impl Module {
    /// Create a new module with parameters.
    pub fn new(symbol: char, params: Vec<f64>) -> Self {
        Self { symbol, params }
    }

    /// Create a module with no parameters.
    pub fn simple(symbol: char) -> Self {
        Self {
            symbol,
            params: Vec::new(),
        }
    }

    /// Format as a string, e.g., `F(3.5)`.
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        if self.params.is_empty() {
            self.symbol.to_string()
        } else {
            let param_strs: Vec<String> = self.params.iter().map(|p| format!("{p:.2}")).collect();
            format!("{}({})", self.symbol, param_strs.join(","))
        }
    }
}

/// A parametric L-system.
///
/// # Examples
///
/// ```
/// use l_system_rs::parametric::{ParametricLSystem, Module};
/// use l_system_rs::rule::ParametricRule;
///
/// let rules = vec![
///     ParametricRule::new('F', 1, |p| p[0] > 1.0, "F($0/2)+F($0/2)"),
/// ];
/// let mut pls = ParametricLSystem::new(
///     vec![Module::new('F', vec![4.0])],
///     rules,
/// );
/// let result = pls.iterate(1);
/// assert_eq!(result.len(), 3); // F(2)+F(2) -> 3 modules: F, +, F
/// ```
#[derive(Debug, Clone)]
pub struct ParametricLSystem {
    /// The axiom as a sequence of modules.
    pub axiom: Vec<Module>,
    /// Parametric production rules.
    pub rules: Vec<ParametricRule>,
}

impl ParametricLSystem {
    /// Create a new parametric L-system.
    pub fn new(axiom: Vec<Module>, rules: Vec<ParametricRule>) -> Self {
        Self { axiom, rules }
    }

    /// Find a matching rule for a module.
    fn find_rule(&self, module: &Module) -> Option<&ParametricRule> {
        self.rules
            .iter()
            .find(|r| r.matches_with_params(module.symbol, &module.params))
    }

    /// Apply one iteration of parametric rewriting.
    ///
    /// Modules without matching rules are kept unchanged.
    pub fn rewrite(&self, modules: &[Module]) -> Vec<Module> {
        let mut result = Vec::new();
        for module in modules {
            match self.find_rule(module) {
                Some(rule) => {
                    let successor_str = rule.generate_successor(&module.params);
                    let new_modules = parse_modules(&successor_str, &module.params);
                    result.extend(new_modules);
                }
                None => result.push(module.clone()),
            }
        }
        result
    }

    /// Iterate `n` times starting from the axiom.
    pub fn iterate(&mut self, n: usize) -> Vec<Module> {
        let mut current = self.axiom.clone();
        for _ in 0..n {
            current = self.rewrite(&current);
        }
        current
    }

    /// Format the module sequence as a string.
    pub fn format_modules(modules: &[Module]) -> String {
        modules.iter().map(|m| m.to_string()).collect()
    }

    /// Get the total number of modules after `n` iterations.
    pub fn count_after(&mut self, n: usize) -> usize {
        self.iterate(n).len()
    }
}

/// Parse a successor string into modules.
///
/// Supports simple symbol-only strings like "F+G-F" and parametric forms
/// like "F($0/2)" where `$0` references the parent's first parameter.
///
/// For expressions like `$0/2`, this does basic evaluation.
fn parse_modules(s: &str, parent_params: &[f64]) -> Vec<Module> {
    let mut modules = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];
        if ch.is_alphabetic() {
            // Check if followed by (
            if i + 1 < chars.len() && chars[i + 1] == '(' {
                // Find matching )
                let mut depth = 1;
                let start = i + 2;
                let mut end = start;
                while end < chars.len() && depth > 0 {
                    if chars[end] == '(' {
                        depth += 1;
                    } else if chars[end] == ')' {
                        depth -= 1;
                    }
                    if depth > 0 {
                        end += 1;
                    }
                }
                let expr: String = chars[start..end].iter().collect();
                let val = eval_simple_expr(&expr, parent_params);
                modules.push(Module::new(ch, vec![val]));
                i = end + 1;
            } else {
                modules.push(Module::simple(ch));
                i += 1;
            }
        } else {
            // Non-alphabetic characters become simple modules (like +, -)
            modules.push(Module::simple(ch));
            i += 1;
        }
    }

    modules
}

/// Evaluate a simple arithmetic expression with parameter substitution.
///
/// Supports: `$N`, basic `+`, `-`, `*`, `/` operations on numbers and `$N` refs.
fn eval_simple_expr(expr: &str, params: &[f64]) -> f64 {
    let expr = expr.trim();

    // Check for addition/subtraction
    // Find the last + or - (not at start) for left-to-right evaluation
    let mut paren_depth = 0;
    let chars: Vec<char> = expr.chars().collect();
    let byte_pos: Vec<usize> = {
        let mut v = Vec::new();
        let mut pos = 0;
        for ch in &chars {
            v.push(pos);
            pos += ch.len_utf8();
        }
        v
    };

    for idx in (1..chars.len()).rev() {
        let ch = chars[idx];
        if ch == ')' {
            paren_depth += 1;
        } else if ch == '(' {
            paren_depth -= 1;
        } else if paren_depth == 0 {
            let byte_end = byte_pos[idx];
            if ch == '+' {
                let left = eval_simple_expr(&expr[..byte_end], params);
                let right = eval_simple_expr(&expr[byte_end + 1..], params);
                return left + right;
            } else if ch == '-' {
                let left = eval_simple_expr(&expr[..byte_end], params);
                let right = eval_simple_expr(&expr[byte_end + 1..], params);
                return left - right;
            }
        }
    }

    // Check for multiplication/division
    paren_depth = 0;
    for idx in (1..chars.len()).rev() {
        let ch = chars[idx];
        if ch == ')' {
            paren_depth += 1;
        } else if ch == '(' {
            paren_depth -= 1;
        } else if paren_depth == 0 {
            let byte_end = byte_pos[idx];
            if ch == '*' {
                let left = eval_simple_expr(&expr[..byte_end], params);
                let right = eval_simple_expr(&expr[byte_end + 1..], params);
                return left * right;
            } else if ch == '/' {
                let left = eval_simple_expr(&expr[..byte_end], params);
                let right = eval_simple_expr(&expr[byte_end + 1..], params);
                return left / right;
            }
        }
    }

    // Check for parentheses
    if expr.starts_with('(') && expr.ends_with(')') {
        return eval_simple_expr(&expr[1..expr.len() - 1], params);
    }

    // Parameter reference: $N
    if let Some(rest) = expr.strip_prefix('$') {
        if let Ok(idx) = rest.parse::<usize>() {
            if idx < params.len() {
                return params[idx];
            }
        }
        return 0.0;
    }

    // Plain number
    expr.parse::<f64>().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::ParametricRule;

    #[test]
    fn test_module_new() {
        let m = Module::new('F', vec![3.0]);
        assert_eq!(m.symbol, 'F');
        assert_eq!(m.params, vec![3.0]);
    }

    #[test]
    fn test_module_simple() {
        let m = Module::simple('+');
        assert_eq!(m.symbol, '+');
        assert!(m.params.is_empty());
    }

    #[test]
    fn test_module_to_string() {
        assert_eq!(Module::simple('F').to_string(), "F");
        assert_eq!(Module::new('F', vec![3.0]).to_string(), "F(3.00)");
    }

    #[test]
    fn test_module_equality() {
        let m1 = Module::new('F', vec![1.0, 2.0]);
        let m2 = Module::new('F', vec![1.0, 2.0]);
        let m3 = Module::new('G', vec![1.0]);
        assert_eq!(m1, m2);
        assert_ne!(m1, m3);
    }

    #[test]
    fn test_simple_parametric_iteration() {
        let rules = vec![ParametricRule::new('F', 1, |_| true, "F(2)")];
        let mut pls = ParametricLSystem::new(vec![Module::new('F', vec![1.0])], rules);
        let result = pls.iterate(1);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].params[0], 2.0);
    }

    #[test]
    fn test_parametric_with_condition() {
        let rules = vec![ParametricRule::new(
            'F',
            1,
            |p| p[0] > 2.0,
            "F($0-1)+F($0-1)",
        )];
        let mut pls = ParametricLSystem::new(vec![Module::new('F', vec![4.0])], rules);
        let result = pls.iterate(1);
        // F(3)+F(3) -> 3 modules: F(3), +, F(3)
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_parametric_condition_blocks() {
        let rules = vec![ParametricRule::new('F', 1, |p| p[0] > 5.0, "F($0-1)")];
        let mut pls = ParametricLSystem::new(vec![Module::new('F', vec![3.0])], rules);
        let result = pls.iterate(1);
        // Condition fails, module kept as-is
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].params[0], 3.0);
    }

    #[test]
    fn test_parametric_growth() {
        let rules = vec![ParametricRule::new(
            'F',
            1,
            |_| true,
            "F($0+1)",
        )];
        let mut pls = ParametricLSystem::new(vec![Module::new('F', vec![0.0])], rules);
        let result = pls.iterate(3);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].params[0], 3.0);
    }

    #[test]
    fn test_parse_modules_simple() {
        let modules = parse_modules("F+G", &[]);
        assert_eq!(modules.len(), 3);
        assert_eq!(modules[0].symbol, 'F');
        assert_eq!(modules[1].symbol, '+');
        assert_eq!(modules[2].symbol, 'G');
    }

    #[test]
    fn test_parse_modules_parametric() {
        let modules = parse_modules("F(3.5)", &[]);
        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].symbol, 'F');
        assert!((modules[0].params[0] - 3.5).abs() < 1e-10);
    }

    #[test]
    fn test_parse_modules_expr() {
        let modules = parse_modules("F($0+1)", &[2.0]);
        assert_eq!(modules.len(), 1);
        assert!((modules[0].params[0] - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_eval_simple_number() {
        assert!((eval_simple_expr("3.5", &[]) - 3.5).abs() < 1e-10);
    }

    #[test]
    fn test_eval_param_ref() {
        assert!((eval_simple_expr("$0", &[7.0]) - 7.0).abs() < 1e-10);
        assert!((eval_simple_expr("$1", &[1.0, 2.0]) - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_eval_addition() {
        assert!((eval_simple_expr("$0+1", &[2.0]) - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_eval_division() {
        assert!((eval_simple_expr("$0/2", &[4.0]) - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_format_modules() {
        let modules = vec![Module::new('F', vec![3.0]), Module::simple('+')];
        let s = ParametricLSystem::format_modules(&modules);
        assert!(s.contains("F(3.00)"));
        assert!(s.contains('+'));
    }

    #[test]
    fn test_count_after() {
        let rules = vec![ParametricRule::new('F', 1, |_| true, "F($0)F($0)")];
        let mut pls = ParametricLSystem::new(vec![Module::new('F', vec![1.0])], rules);
        assert_eq!(pls.count_after(0), 1);
        assert_eq!(pls.count_after(1), 2);
        assert_eq!(pls.count_after(2), 4);
    }

    #[test]
    fn test_no_matching_rule_preserves() {
        let rules = vec![ParametricRule::new('G', 1, |_| true, "G(2)")];
        let mut pls = ParametricLSystem::new(vec![Module::new('F', vec![1.0])], rules);
        let result = pls.iterate(1);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].symbol, 'F');
    }
}
