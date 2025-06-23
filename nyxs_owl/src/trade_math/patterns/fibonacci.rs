use polars::prelude::*;
use std::collections::HashMap;

/// Fibonacci retracement levels
pub const FIBONACCI_LEVELS: [f64; 6] = [0.0, 0.236, 0.382, 0.5, 0.618, 0.786];

/// Calculate Fibonacci retracement levels
///
/// Fibonacci retracements are used to identify potential support and resistance levels
/// based on the Fibonacci sequence ratios.
///
/// # Arguments
/// * `high` - Series of high prices
/// * `low` - Series of low prices
/// * `swing_high` - The swing high point
/// * `swing_low` - The swing low point
///
/// # Returns
/// * `PolarsResult<HashMap<String, f64>>` - Map of Fibonacci levels to price values
pub fn calculate_fibonacci_retracements(
    swing_high: f64,
    swing_low: f64,
) -> PolarsResult<HashMap<String, f64>> {
    if swing_high <= swing_low {
        return Err(PolarsError::InvalidOperation(
            "Swing high must be greater than swing low".into(),
        ));
    }

    let range = swing_high - swing_low;
    let mut levels = HashMap::new();

    for &level in &FIBONACCI_LEVELS {
        let price = swing_high - (range * level);
        levels.insert(format!("fib_{:.0}", level * 1000.0), price);
    }

    Ok(levels)
}

/// Detect Fibonacci retracement levels from price data
///
/// Automatically identifies swing highs and lows to calculate Fibonacci retracements.
///
/// # Arguments
/// * `high` - Series of high prices
/// * `low` - Series of low prices
/// * `window` - Window size for swing detection (typically 20)
///
/// # Returns
/// * `PolarsResult<Vec<HashMap<String, f64>>>` - Vector of Fibonacci levels for each detected swing
pub fn detect_fibonacci_retracements(
    high: &Series,
    low: &Series,
    window: usize,
) -> PolarsResult<Vec<HashMap<String, f64>>> {
    if window == 0 {
        return Err(PolarsError::InvalidOperation(
            "Window size must be greater than 0".into(),
        ));
    }

    let high_values: Vec<Option<f64>> = high.f64()?.into_iter().collect();
    let low_values: Vec<Option<f64>> = low.f64()?.into_iter().collect();

    if high_values.len() != low_values.len() {
        return Err(PolarsError::InvalidOperation(
            "High and low series must have the same length".into(),
        ));
    }

    let mut retracements = Vec::new();

    // Detect swings and calculate Fibonacci levels
    for i in window..high_values.len() - window {
        let mut is_swing_high = true;
        let mut is_swing_low = true;

        let current_high = high_values[i].unwrap_or(0.0);
        let current_low = low_values[i].unwrap_or(0.0);

        // Check if current point is a swing high
        for j in i.saturating_sub(window)..=i + window {
            if j != i && j < high_values.len() {
                if let Some(compare_high) = high_values[j] {
                    if compare_high >= current_high {
                        is_swing_high = false;
                        break;
                    }
                }
            }
        }

        // Check if current point is a swing low
        for j in i.saturating_sub(window)..=i + window {
            if j != i && j < low_values.len() {
                if let Some(compare_low) = low_values[j] {
                    if compare_low <= current_low {
                        is_swing_low = false;
                        break;
                    }
                }
            }
        }

        // Calculate Fibonacci retracements for detected swings
        if is_swing_high || is_swing_low {
            if let Ok(levels) = calculate_fibonacci_retracements(current_high, current_low) {
                retracements.push(levels);
            }
        }
    }

    Ok(retracements)
}

/// Calculate Fibonacci extensions
///
/// Fibonacci extensions project potential price targets beyond the swing high/low.
///
/// # Arguments
/// * `swing_high` - The swing high point
/// * `swing_low` - The swing low point
/// * `extension_levels` - Extension levels (typically [1.272, 1.618, 2.0, 2.618])
///
/// # Returns
/// * `PolarsResult<HashMap<String, f64>>` - Map of extension levels to price values
pub fn calculate_fibonacci_extensions(
    swing_high: f64,
    swing_low: f64,
    extension_levels: &[f64],
) -> PolarsResult<HashMap<String, f64>> {
    if swing_high <= swing_low {
        return Err(PolarsError::InvalidOperation(
            "Swing high must be greater than swing low".into(),
        ));
    }

    let range = swing_high - swing_low;
    let mut extensions = HashMap::new();

    for &level in extension_levels {
        let price = swing_high + (range * level);
        extensions.insert(format!("ext_{:.0}", level * 1000.0), price);
    }

    Ok(extensions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fibonacci_retracements() {
        let swing_high = 100.0;
        let swing_low = 80.0;

        let retracements = calculate_fibonacci_retracements(swing_high, swing_low).unwrap();

        // Test key Fibonacci levels
        assert!((retracements["fib_0"] - 100.0).abs() < 0.001);
        assert!((retracements["fib_236"] - 95.28).abs() < 0.01);
        assert!((retracements["fib_382"] - 92.36).abs() < 0.01);
        assert!((retracements["fib_500"] - 90.0).abs() < 0.001);
        assert!((retracements["fib_618"] - 87.64).abs() < 0.01);
        assert!((retracements["fib_786"] - 84.28).abs() < 0.01);

        // Test invalid input
        assert!(calculate_fibonacci_retracements(80.0, 100.0).is_err());
    }

    #[test]
    fn test_fibonacci_extensions() {
        let swing_high = 100.0;
        let swing_low = 80.0;
        let extension_levels = vec![1.272, 1.618, 2.0];

        let extensions =
            calculate_fibonacci_extensions(swing_high, swing_low, &extension_levels).unwrap();

        // Test extension levels
        assert!((extensions["ext_1272"] - 125.44).abs() < 0.01);
        assert!((extensions["ext_1618"] - 132.36).abs() < 0.01);
        assert!((extensions["ext_2000"] - 140.0).abs() < 0.001);
    }

    #[test]
    fn test_detect_fibonacci_retracements() {
        let high = Series::new(
            "high".into(),
            vec![10.0, 12.0, 11.0, 13.0, 14.0, 13.5, 15.0, 14.0, 13.0, 12.5],
        );
        let low = Series::new(
            "low".into(),
            vec![9.0, 10.0, 9.5, 11.0, 12.0, 11.5, 13.0, 12.0, 11.0, 10.5],
        );

        let window = 2;
        let retracements = detect_fibonacci_retracements(&high, &low, window).unwrap();

        // Should detect some swings
        assert!(!retracements.is_empty());

        // Test invalid window
        assert!(detect_fibonacci_retracements(&high, &low, 0).is_err());
    }
}
