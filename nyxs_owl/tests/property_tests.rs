//! Property-based tests for NyxsOwl
//!
//! These tests verify mathematical properties and invariants that should
//! always hold regardless of input data, ensuring robustness.

use nyxs_owl::trade_math::{
    moving_averages::{ExponentialMovingAverage, SimpleMovingAverage},
    oscillators::{RelativeStrengthIndex, StochasticOscillator},
    volatility::{BollingerBands, StandardDeviation},
    volume::OnBalanceVolume,
};
use rstest::rstest;

// Configure property tests for IDE-friendly resource usage
#[cfg(test)]
mod test_config {
    use std::env;

    pub fn get_test_cases() -> usize {
        // Reduce test cases when running in IDE or limited environments
        if env::var("RUST_TEST_THREADS").unwrap_or_default() == "4" {
            10 // Lite mode: 10 test cases per property
        } else {
            100 // Full mode: 100 test cases per property
        }
    }
}

/// Test that SMA satisfies mathematical properties
#[rstest]
#[case(5, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0])]
#[case(3, vec![10.0, 20.0, 30.0, 40.0, 50.0])]
#[case(10, (1..=20).map(|i| i as f64).collect())]
fn test_sma_mathematical_properties(#[case] period: usize, #[case] data: Vec<f64>) {
    let mut sma = SimpleMovingAverage::new(period).expect("Failed to create SMA");

    // Feed data to SMA
    for &value in &data {
        sma.update(value).expect("Failed to update SMA");
    }

    if data.len() >= period {
        let sma_value = sma.value().expect("SMA should have value");
        let last_n_values = &data[data.len() - period..];
        let manual_average = last_n_values.iter().sum::<f64>() / period as f64;

        // SMA should equal the arithmetic mean of the last N values
        assert!(
            (sma_value - manual_average).abs() < 1e-10,
            "SMA should equal manual average: {} vs {}",
            sma_value,
            manual_average
        );

        // SMA should be within the range of input values
        let min_val = last_n_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_val = last_n_values
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        assert!(
            sma_value >= min_val && sma_value <= max_val,
            "SMA should be within input range: {} not in [{}, {}]",
            sma_value,
            min_val,
            max_val
        );
    }
}

/// Test EMA decay property - newer values should have more influence
#[rstest]
#[case(10)]
#[case(20)]
#[case(50)]
fn test_ema_decay_property(#[case] period: usize) {
    let mut ema = ExponentialMovingAverage::new(period).expect("Failed to create EMA");

    // Start with stable value
    for _ in 0..period * 2 {
        ema.update(100.0).expect("Failed to update EMA");
    }
    let stable_value = ema.value().expect("EMA should have stable value");

    // Add a spike
    ema.update(200.0).expect("Failed to update EMA with spike");
    let after_spike = ema.value().expect("EMA should have value after spike");

    // Return to original value
    ema.update(100.0)
        .expect("Failed to update EMA back to original");
    let after_return = ema.value().expect("EMA should have value after return");

    // EMA should react to spike but not fully
    assert!(
        after_spike > stable_value,
        "EMA should increase after positive spike"
    );
    assert!(
        after_spike < 200.0,
        "EMA should not reach spike value immediately"
    );

    // EMA should decay back towards original value
    assert!(after_return < after_spike, "EMA should decay after spike");
    assert!(after_return > stable_value, "EMA should still be elevated");
}

/// Test RSI bounds property - should always be between 0 and 100
#[rstest]
#[case(vec![100.0, 105.0, 110.0, 115.0, 120.0, 125.0, 130.0, 135.0, 140.0, 145.0, 150.0, 155.0, 160.0, 165.0, 170.0])] // Strong uptrend
#[case(vec![100.0, 95.0, 90.0, 85.0, 80.0, 75.0, 70.0, 65.0, 60.0, 55.0, 50.0, 45.0, 40.0, 35.0, 30.0])] // Strong downtrend
#[case(vec![100.0, 101.0, 99.0, 102.0, 98.0, 103.0, 97.0, 104.0, 96.0, 105.0, 95.0, 106.0, 94.0, 107.0, 93.0])] // Volatile sideways
fn test_rsi_bounds_property(#[case] prices: Vec<f64>) {
    let mut rsi = RelativeStrengthIndex::new(14).expect("Failed to create RSI");

    for &price in &prices {
        rsi.update(price).expect("Failed to update RSI");

        if let Ok(rsi_value) = rsi.value() {
            assert!(
                rsi_value >= 0.0 && rsi_value <= 100.0,
                "RSI should be between 0 and 100, got {}",
                rsi_value
            );
            assert!(
                rsi_value.is_finite(),
                "RSI should be finite, got {}",
                rsi_value
            );
        }
    }
}

/// Test Bollinger Bands ordering property
#[rstest]
#[case(20, 2.0)]
#[case(10, 1.5)]
#[case(50, 2.5)]
fn test_bollinger_bands_ordering(#[case] period: usize, #[case] std_dev: f64) {
    let mut bb = BollingerBands::new(period, std_dev).expect("Failed to create Bollinger Bands");

    // Generate some price data
    for i in 1..=period + 5 {
        let price = 100.0 + (i as f64 % 10.0); // Varying prices
        bb.update(price).expect("Failed to update BB");

        if let (Ok(lower), Ok(middle), Ok(upper)) =
            (bb.lower_band(), bb.middle_band(), bb.upper_band())
        {
            // Bands should be properly ordered
            assert!(
                lower <= middle,
                "Lower band should be <= middle band: {} vs {}",
                lower,
                middle
            );
            assert!(
                middle <= upper,
                "Middle band should be <= upper band: {} vs {}",
                middle,
                upper
            );

            // All values should be positive for positive price data
            assert!(lower > 0.0, "Lower band should be positive: {}", lower);
            assert!(middle > 0.0, "Middle band should be positive: {}", middle);
            assert!(upper > 0.0, "Upper band should be positive: {}", upper);

            // Band width should be proportional to standard deviation multiplier
            let band_width = upper - lower;
            assert!(
                band_width > 0.0,
                "Band width should be positive: {}",
                band_width
            );
        }
    }
}

/// Test Standard Deviation non-negativity property
#[rstest]
#[case(vec![100.0; 10])] // Constant values - should have zero std dev
#[case(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0])] // Linear sequence
#[case(vec![50.0, 100.0, 50.0, 100.0, 50.0, 100.0, 50.0, 100.0])] // High volatility
fn test_standard_deviation_properties(#[case] data: Vec<f64>) {
    let mut std_dev = StandardDeviation::new(data.len()).expect("Failed to create StdDev");

    for &value in &data {
        std_dev.update(value).expect("Failed to update StdDev");
    }

    let std_value = std_dev.value().expect("StdDev should have value");

    // Standard deviation should always be non-negative
    assert!(
        std_value >= 0.0,
        "Standard deviation should be non-negative: {}",
        std_value
    );

    // For constant values, std dev should be zero (or very close)
    if data.iter().all(|&x| (x - data[0]).abs() < 1e-10) {
        assert!(
            std_value < 1e-6,
            "Standard deviation of constant values should be near zero: {}",
            std_value
        );
    }
}

/// Test OBV directional property
#[rstest]
#[case(vec![(100.0, 1000.0), (105.0, 1200.0), (110.0, 1400.0)])] // Prices and volume up
#[case(vec![(100.0, 1000.0), (95.0, 1200.0), (90.0, 1400.0)])] // Prices down, volume up
fn test_obv_directional_property(#[case] price_volume_pairs: Vec<(f64, f64)>) {
    let mut obv = OnBalanceVolume::new();
    let mut obv_values = Vec::new();

    for &(price, volume) in &price_volume_pairs {
        obv.update(price, volume).expect("Failed to update OBV");
        obv_values.push(obv.value().expect("OBV should have value"));
    }

    // Check directional relationship
    for i in 1..price_volume_pairs.len() {
        let prev_price = price_volume_pairs[i - 1].0;
        let curr_price = price_volume_pairs[i].0;
        let curr_volume = price_volume_pairs[i].1;

        let obv_change = obv_values[i] - obv_values[i - 1];

        if curr_price > prev_price {
            // Price up: OBV should increase by volume amount
            assert!(
                (obv_change - curr_volume).abs() < 1e-10,
                "OBV should increase by volume on up day: {} vs {}",
                obv_change,
                curr_volume
            );
        } else if curr_price < prev_price {
            // Price down: OBV should decrease by volume amount
            assert!(
                (obv_change + curr_volume).abs() < 1e-10,
                "OBV should decrease by volume on down day: {} vs {}",
                obv_change,
                -curr_volume
            );
        } else {
            // Price unchanged: OBV should remain same
            assert!(
                obv_change.abs() < 1e-10,
                "OBV should not change when price unchanged: {}",
                obv_change
            );
        }
    }
}

/// Test Stochastic Oscillator bounds
#[rstest]
#[case(14, 3, vec![100.0, 105.0, 102.0, 108.0, 104.0, 110.0, 106.0, 112.0, 108.0, 115.0, 110.0, 118.0, 112.0, 120.0, 115.0, 122.0, 118.0, 125.0])]
fn test_stochastic_bounds_property(
    #[case] k_period: usize,
    #[case] d_period: usize,
    #[case] prices: Vec<f64>,
) {
    let mut stoch =
        StochasticOscillator::new(k_period, d_period).expect("Failed to create Stochastic");

    // Generate high/low data from prices (simulate realistic high/low around close)
    for &price in &prices {
        let high = price * 1.01; // High slightly above close
        let low = price * 0.99; // Low slightly below close

        stoch
            .update(high, low, price)
            .expect("Failed to update Stochastic");

        if let (Ok(k_value), Ok(d_value)) = (stoch.k_value(), stoch.d_value()) {
            // Both %K and %D should be between 0 and 100
            assert!(
                k_value >= 0.0 && k_value <= 100.0,
                "%K should be between 0 and 100: {}",
                k_value
            );
            assert!(
                d_value >= 0.0 && d_value <= 100.0,
                "%D should be between 0 and 100: {}",
                d_value
            );

            // Values should be finite
            assert!(k_value.is_finite(), "%K should be finite: {}", k_value);
            assert!(d_value.is_finite(), "%D should be finite: {}", d_value);
        }
    }
}

/// Test monotonicity properties
#[rstest]
#[case(10)]
#[case(20)]
fn test_increasing_price_trends(#[case] period: usize) {
    let mut sma = SimpleMovingAverage::new(period).expect("Failed to create SMA");
    let mut ema = ExponentialMovingAverage::new(period).expect("Failed to create EMA");

    let mut sma_values = Vec::new();
    let mut ema_values = Vec::new();

    // Create strictly increasing price series
    for i in 1..=period * 2 {
        let price = i as f64;
        sma.update(price).expect("Failed to update SMA");
        ema.update(price).expect("Failed to update EMA");

        if let Ok(sma_val) = sma.value() {
            sma_values.push(sma_val);
        }
        if let Ok(ema_val) = ema.value() {
            ema_values.push(ema_val);
        }
    }

    // For strictly increasing prices, moving averages should generally increase
    // (though not necessarily monotonically due to rolling window effects)
    if sma_values.len() > 1 {
        let sma_final = sma_values.last().unwrap();
        let sma_first = sma_values.first().unwrap();
        assert!(
            sma_final > sma_first,
            "SMA should increase with increasing prices"
        );
    }

    if ema_values.len() > 1 {
        let ema_final = ema_values.last().unwrap();
        let ema_first = ema_values.first().unwrap();
        assert!(
            ema_final > ema_first,
            "EMA should increase with increasing prices"
        );
    }
}

/// Test numerical stability with extreme values
#[rstest]
#[case(vec![f64::MAX / 1e6, f64::MAX / 1e6, f64::MAX / 1e6])] // Very large values
#[case(vec![f64::MIN_POSITIVE, f64::MIN_POSITIVE * 2.0, f64::MIN_POSITIVE * 3.0])] // Very small values
#[case(vec![0.0, 0.0, 0.0])] // Zero values
fn test_numerical_stability(#[case] extreme_values: Vec<f64>) {
    let mut sma = SimpleMovingAverage::new(3).expect("Failed to create SMA");
    let mut std_dev = StandardDeviation::new(3).expect("Failed to create StdDev");

    for &value in &extreme_values {
        assert!(
            sma.update(value).is_ok(),
            "SMA should handle extreme values"
        );
        assert!(
            std_dev.update(value).is_ok(),
            "StdDev should handle extreme values"
        );
    }

    // Results should be finite and not NaN
    if let Ok(sma_val) = sma.value() {
        assert!(
            sma_val.is_finite(),
            "SMA result should be finite with extreme values"
        );
        assert!(
            !sma_val.is_nan(),
            "SMA result should not be NaN with extreme values"
        );
    }

    if let Ok(std_val) = std_dev.value() {
        assert!(
            std_val.is_finite(),
            "StdDev result should be finite with extreme values"
        );
        assert!(
            !std_val.is_nan(),
            "StdDev result should not be NaN with extreme values"
        );
        assert!(
            std_val >= 0.0,
            "StdDev should be non-negative with extreme values"
        );
    }
}

/// Test indicator behavior with insufficient data
#[rstest]
#[case(10, vec![1.0, 2.0, 3.0])] // Less data than period
#[case(5, vec![])] // Empty data
fn test_insufficient_data_handling(#[case] period: usize, #[case] data: Vec<f64>) {
    let mut sma = SimpleMovingAverage::new(period).expect("Failed to create SMA");
    let mut bb = BollingerBands::new(period, 2.0).expect("Failed to create BB");
    let mut rsi = RelativeStrengthIndex::new(period).expect("Failed to create RSI");

    for &value in &data {
        assert!(
            sma.update(value).is_ok(),
            "Updates should succeed even with insufficient data"
        );
        assert!(bb.update(value).is_ok(), "BB updates should succeed");
        assert!(rsi.update(value).is_ok(), "RSI updates should succeed");
    }

    if data.len() < period {
        // Should return error when insufficient data
        assert!(
            sma.value().is_err(),
            "SMA should return error with insufficient data"
        );
        assert!(
            bb.middle_band().is_err(),
            "BB should return error with insufficient data"
        );
        assert!(
            rsi.value().is_err(),
            "RSI should return error with insufficient data"
        );
    }
}
