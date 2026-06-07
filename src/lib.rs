//! # l-system-rs
//!
//! A research-grade Lindenmayer systems library implementing string rewriting,
//! turtle interpretation, stochastic L-systems, and parametric L-systems.
//!
//! # Modules
//!
//! - [`system`] — Core L-system with axiom and iteration
//! - [`rule`] — Production rules (deterministic, stochastic, parametric)
//! - [`turtle`] — Turtle graphics interpretation of L-system strings
//! - [`stochastic`] — Stochastic L-systems with probabilistic rule selection
//! - [`parametric`] — Parametric L-systems with numerical conditions

pub mod parametric;
pub mod rule;
pub mod stochastic;
pub mod system;
pub mod turtle;

pub use rule::ProductionRule;
pub use system::LSystem;
pub use turtle::{Segment, Turtle};
