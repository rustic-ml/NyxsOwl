//! Pattern recognition indicators and calculations module
//!
//! This module provides implementations of various chart pattern recognition
//! and geometric analysis tools used in technical analysis.

/// Fibonacci retracement and extension patterns
pub mod fibonacci;

pub use fibonacci::{
    calculate_fibonacci_retracements,
    calculate_fibonacci_extensions,
    detect_fibonacci_retracements,
    FIBONACCI_LEVELS,
}; 