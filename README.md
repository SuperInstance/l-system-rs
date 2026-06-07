# l-system-rs

Research-grade Lindenmayer systems library in pure Rust.

## Features

- **String rewriting**: Deterministic and stochastic production rules
- **Turtle interpretation**: Convert L-system strings to geometric line segments
- **Parametric L-systems**: Rules with numeric parameters and conditions
- **Classic fractals**: Koch curve, Sierpinski triangle, dragon curve, plant branching

## Usage

```rust
use l_system_rs::system::LSystem;
use l_system_rs::rule::ProductionRule;

let mut ls = LSystem::new("F");
ls.add_rule(ProductionRule::new('F', "F+F--F+F"));
let result = ls.iterate(3);
```

## License

MIT OR Apache-2.0
