//! # Strategy Utilities
//!
//! This module provides utility functions for working with strategies.

use crate::strategy_lib::strategy::{Signal, StrategyError};
use polars::prelude::*;

/// Convert a Series of raw signal values (1 for buy, -1 for sell, 0 for hold) to Signal enum
pub fn raw_values_to_signals(raw_signals: &Series) -> Result<Series, StrategyError> {
    if raw_signals.dtype() != &DataType::Int32 && raw_signals.dtype() != &DataType::Int64 {
        return Err(StrategyError::InvalidParameter(
            "Raw signals must be integer type".to_string(),
        ));
    }

    let signals = raw_signals
        .i32()
        .map_err(StrategyError::PolarsError)?
        .into_iter()
        .map(|opt_val| match opt_val {
            Some(1) => Some(Signal::Buy as i32),
            Some(-1) => Some(Signal::Sell as i32),
            _ => Some(Signal::Hold as i32),
        })
        .collect::<Vec<_>>();

    Ok(Series::new("signal".into(), signals))
}

/// Calculate the crossover points between two series
/// Returns a Series with values:
/// 1 when series1 crosses above series2
/// -1 when series1 crosses below series2
/// 0 when no crossover
pub fn calculate_crossovers(series1: &Series, series2: &Series) -> Result<Series, StrategyError> {
    // Make sure both series are numeric
    let is_numeric_dtype = |dt: &DataType| {
        matches!(
            dt,
            DataType::Int8
                | DataType::Int16
                | DataType::Int32
                | DataType::Int64
                | DataType::UInt8
                | DataType::UInt16
                | DataType::UInt32
                | DataType::UInt64
                | DataType::Float32
                | DataType::Float64
        )
    };

    if !is_numeric_dtype(series1.dtype()) || !is_numeric_dtype(series2.dtype()) {
        return Err(StrategyError::InvalidParameter(
            "Both series must be numeric".to_string(),
        ));
    }

    // Convert to f64 to ensure compatibility
    let s1 = series1
        .cast(&DataType::Float64)
        .map_err(StrategyError::PolarsError)?;
    let s2 = series2
        .cast(&DataType::Float64)
        .map_err(StrategyError::PolarsError)?;

    let s1 = s1.f64().map_err(StrategyError::PolarsError)?;
    let s2 = s2.f64().map_err(StrategyError::PolarsError)?;

    // Calculate current and previous differences
    let mut current_diff = Vec::new();
    let mut prev_diff = Vec::new();

    let mut prev_diff_val = 0.0;

    for (i, (s1_val, s2_val)) in s1.into_iter().zip(s2.into_iter()).enumerate() {
        let s1_val = s1_val.unwrap_or(0.0);
        let s2_val = s2_val.unwrap_or(0.0);

        let diff = s1_val - s2_val;
        current_diff.push(diff);

        if i > 0 {
            prev_diff.push(prev_diff_val);
        } else {
            prev_diff.push(0.0);
        }

        prev_diff_val = diff;
    }

    // Calculate crossovers
    let mut crossovers = Vec::new();
    for i in 0..current_diff.len() {
        if i == 0 {
            crossovers.push(0);
        } else {
            // Crossing above
            if prev_diff[i] < 0.0 && current_diff[i] >= 0.0 {
                crossovers.push(1);
            }
            // Crossing below
            else if prev_diff[i] > 0.0 && current_diff[i] <= 0.0 {
                crossovers.push(-1);
            }
            // No crossover
            else {
                crossovers.push(0);
            }
        }
    }

    Ok(Series::new("crossover".into(), crossovers))
}

/// Calculate basic percentage change between consecutive values in a series
pub fn calculate_pct_change(series: &Series) -> Result<Series, StrategyError> {
    let is_numeric_dtype = |dt: &DataType| {
        matches!(
            dt,
            DataType::Int8
                | DataType::Int16
                | DataType::Int32
                | DataType::Int64
                | DataType::UInt8
                | DataType::UInt16
                | DataType::UInt32
                | DataType::UInt64
                | DataType::Float32
                | DataType::Float64
        )
    };

    if !is_numeric_dtype(series.dtype()) {
        return Err(StrategyError::InvalidParameter(
            "Series must be numeric".to_string(),
        ));
    }

    let s = series
        .cast(&DataType::Float64)
        .map_err(StrategyError::PolarsError)?;
    let s = s.f64().map_err(StrategyError::PolarsError)?;

    let mut pct_changes = Vec::new();
    let mut prev_val = None;

    for val in s.into_iter() {
        if let Some(curr_val) = val {
            if let Some(prev) = prev_val {
                if prev != 0.0 {
                    pct_changes.push(Some((curr_val - prev) / prev));
                } else {
                    pct_changes.push(Some(0.0));
                }
            } else {
                pct_changes.push(None);
            }
            prev_val = Some(curr_val);
        } else {
            pct_changes.push(None);
            prev_val = None;
        }
    }

    let name = format!("{}_pct_change", series.name());
    Series::new(name.into(), pct_changes)
        .cast(&DataType::Float64)
        .map_err(StrategyError::PolarsError)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_raw_values_to_signals() {
        let raw_signals = Series::new("raw".into(), &[1, 0, -1, 1, 0, -1]);
        let signals = raw_values_to_signals(&raw_signals).unwrap();

        let signal_vals = signals.i32().unwrap();

        assert_eq!(signal_vals.get(0), Some(Signal::Buy as i32));
        assert_eq!(signal_vals.get(1), Some(Signal::Hold as i32));
        assert_eq!(signal_vals.get(2), Some(Signal::Sell as i32));
        assert_eq!(signal_vals.get(3), Some(Signal::Buy as i32));
        assert_eq!(signal_vals.get(4), Some(Signal::Hold as i32));
        assert_eq!(signal_vals.get(5), Some(Signal::Sell as i32));
    }

    #[test]
    fn test_raw_values_to_signals_invalid_type() {
        let raw_signals = Series::new("raw".into(), &["buy", "hold", "sell"]);
        let result = raw_values_to_signals(&raw_signals);

        assert!(result.is_err());
        match result.unwrap_err() {
            StrategyError::InvalidParameter(msg) => {
                assert!(msg.contains("integer type"));
            }
            _ => panic!("Expected InvalidParameter error"),
        }
    }

    #[test]
    fn test_calculate_crossovers() {
        // Create two series where we know there will be crossovers
        let s1 = Series::new("s1".into(), &[10.0, 11.0, 9.0, 8.0, 12.0, 14.0]);
        let s2 = Series::new("s2".into(), &[11.0, 10.0, 10.0, 9.0, 10.0, 15.0]);

        let crossovers = calculate_crossovers(&s1, &s2).unwrap();
        let crossover_vals = crossovers.i32().unwrap();

        // First point is always 0 (no crossover)
        assert_eq!(crossover_vals.get(0), Some(0));

        // s1 crosses above s2 at index 1
        assert_eq!(crossover_vals.get(1), Some(1));

        // s1 crosses below s2 at index 2
        assert_eq!(crossover_vals.get(2), Some(-1));

        // No crossover at index 3
        assert_eq!(crossover_vals.get(3), Some(0));

        // s1 crosses above s2 at index 4
        assert_eq!(crossover_vals.get(4), Some(1));

        // s1 crosses below s2 at index 5
        assert_eq!(crossover_vals.get(5), Some(-1));
    }

    #[test]
    fn test_calculate_crossovers_invalid_type() {
        let s1 = Series::new("s1".into(), &[10.0, 11.0, 9.0]);
        let s2 = Series::new("s2".into(), &["a", "b", "c"]);

        let result = calculate_crossovers(&s1, &s2);

        assert!(result.is_err());
        match result.unwrap_err() {
            StrategyError::InvalidParameter(msg) => {
                assert!(msg.contains("numeric"));
            }
            _ => panic!("Expected InvalidParameter error"),
        }
    }

    #[test]
    fn test_calculate_pct_change() {
        let values = Series::new("values".into(), &[100.0, 110.0, 99.0, 105.0]);
        let pct_change = calculate_pct_change(&values).unwrap();

        let pct_vals = pct_change.f64().unwrap();

        // First value should be None (no previous value)
        assert!(pct_vals.get(0).is_none());

        // 110.0 / 100.0 - 1 = 0.1 (10% increase)
        assert_relative_eq!(pct_vals.get(1).unwrap(), 0.1, epsilon = 1e-10);

        // 99.0 / 110.0 - 1 = -0.1 (10% decrease)
        assert_relative_eq!(pct_vals.get(2).unwrap(), -0.1, epsilon = 1e-10);

        // 105.0 / 99.0 - 1 = 0.0606... (6.06% increase)
        assert_relative_eq!(
            pct_vals.get(3).unwrap(),
            0.06060606060606055,
            epsilon = 1e-10
        );
    }

    #[test]
    fn test_calculate_pct_change_with_zero() {
        let values = Series::new("values".into(), &[0.0, 10.0, 0.0, 5.0]);
        let pct_change = calculate_pct_change(&values).unwrap();

        let pct_vals = pct_change.f64().unwrap();

        // First value should be None
        assert!(pct_vals.get(0).is_none());

        // Division by zero should return 0.0 by our logic
        assert_eq!(pct_vals.get(1).unwrap(), 0.0);

        // 0.0 / 10.0 = 0.0 - 1 = -1.0 (100% decrease)
        assert_eq!(pct_vals.get(2).unwrap(), -1.0);

        // 5.0 / 0.0 would be division by zero, so we return 0.0
        assert_eq!(pct_vals.get(3).unwrap(), 0.0);
    }

    #[test]
    fn test_calculate_pct_change_invalid_type() {
        let values = Series::new("values".into(), &["a", "b", "c"]);
        let result = calculate_pct_change(&values);

        assert!(result.is_err());
        match result.unwrap_err() {
            StrategyError::InvalidParameter(msg) => {
                assert!(msg.contains("numeric"));
            }
            _ => panic!("Expected InvalidParameter error"),
        }
    }
}
