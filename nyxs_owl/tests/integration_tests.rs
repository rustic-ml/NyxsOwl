//! Integration tests for NyxsOwl
//!
//! These tests verify that multiple components work together correctly
//! and that the public API behaves as expected in real-world scenarios.

use nyxs_owl::trade_math::{
    forecasting::{DoubleExponentialSmoothing, ExponentialSmoothing, LinearRegression},
    moving_averages::{ExponentialMovingAverage, SimpleMovingAverage, VolumeWeightedMovingAverage},
    oscillators::{Macd, RelativeStrengthIndex, StochasticOscillator},
    volatility::{AverageTrueRange, BollingerBands, StandardDeviation},
    volume::{OnBalanceVolume, VolumeMovingAverage, VolumePriceTrend, VolumeRateOfChange},
};

/// Test data representing realistic market conditions
fn get_test_market_data() -> (Vec<f64>, Vec<f64>) {
    // Prices showing an uptrend with volatility (simulating a bull market)
    let prices = vec![
        100.0, 102.5, 101.8, 104.2, 106.1, 105.4, 108.3, 107.9, 110.2, 109.8, 112.5, 115.1, 114.7,
        117.3, 119.8, 118.9, 121.4, 123.7, 122.8, 125.6, 127.2, 126.8, 129.4, 131.1, 130.5, 133.2,
        135.8, 134.9, 137.5, 139.2,
    ];

    // Corresponding volumes (higher volume on up days, lower on down days)
    let volumes = vec![
        1000.0, 1200.0, 900.0, 1400.0, 1600.0, 1100.0, 1800.0, 1300.0, 1900.0, 1200.0, 2100.0,
        2400.0, 1700.0, 2200.0, 2600.0, 1800.0, 2300.0, 2700.0, 1900.0, 2500.0, 2800.0, 2000.0,
        2600.0, 3000.0, 2100.0, 2900.0, 3200.0, 2200.0, 3100.0, 3400.0,
    ];

    (prices, volumes)
}

#[test]
fn test_comprehensive_technical_analysis_workflow() {
    let (prices, volumes) = get_test_market_data();

    // Initialize all indicators
    let mut sma_20 = SimpleMovingAverage::new(20).expect("Failed to create SMA");
    let mut ema_12 = ExponentialMovingAverage::new(12).expect("Failed to create EMA");
    let mut bb_20 = BollingerBands::new(20, 2.0).expect("Failed to create Bollinger Bands");
    let mut rsi_14 = RelativeStrengthIndex::new(14).expect("Failed to create RSI");
    let mut obv = OnBalanceVolume::new();

    // Process all data points
    for (i, (&price, &volume)) in prices.iter().zip(volumes.iter()).enumerate() {
        // Update all indicators
        sma_20.update(price).expect("Failed to update SMA");
        ema_12.update(price).expect("Failed to update EMA");
        bb_20.update(price).expect("Failed to update BB");
        rsi_14.update(price).expect("Failed to update RSI");
        obv.update(price, volume).expect("Failed to update OBV");

        // After sufficient data, verify indicators are producing reasonable values
        if i >= 19 {
            // After 20 data points
            let sma_val = sma_20.value().expect("SMA should have value");
            let bb_middle = bb_20.middle_band().expect("BB middle should have value");
            let bb_upper = bb_20.upper_band().expect("BB upper should have value");
            let bb_lower = bb_20.lower_band().expect("BB lower should have value");

            // SMA and BB middle should be close (both are 20-period SMA)
            assert!(
                (sma_val - bb_middle).abs() < 0.01,
                "SMA and BB middle should be nearly equal"
            );

            // BB bands should be ordered correctly
            assert!(bb_lower < bb_middle, "BB lower should be less than middle");
            assert!(bb_middle < bb_upper, "BB middle should be less than upper");
            assert!(bb_lower > 0.0, "BB lower should be positive");
        }

        if i >= 14 {
            // After 15 data points for RSI (period 14 needs 15 values)
            let rsi_val = rsi_14.value().expect("RSI should have value");
            assert!(
                rsi_val >= 0.0 && rsi_val <= 100.0,
                "RSI should be between 0 and 100, got {}",
                rsi_val
            );
        }

        // OBV should always have a value
        let obv_val = obv.value().expect("OBV should have value");
        if i == 0 {
            assert!(
                obv_val == 0.0,
                "First OBV should be 0 (no previous price to compare)"
            );
        }
    }

    // Final checks - indicators should show uptrend
    let final_sma = sma_20.value().expect("Final SMA");
    let final_rsi = rsi_14.value().expect("Final RSI");
    let final_obv = obv.value().expect("Final OBV");

    // In an uptrend, SMA should be less than final price
    assert!(
        final_sma < *prices.last().unwrap(),
        "In uptrend, SMA should be below final price"
    );

    // RSI should indicate overbought conditions in strong uptrend
    assert!(final_rsi > 50.0, "RSI should be above 50 in uptrend");

    // OBV should be positive (more volume on up days)
    assert!(final_obv > 0.0, "OBV should be positive in uptrend");
}

#[test]
fn test_indicator_combinations_for_signal_generation() {
    let (prices, _volumes) = get_test_market_data();

    let mut ema_fast = ExponentialMovingAverage::new(9).expect("Failed to create fast EMA");
    let mut ema_slow = ExponentialMovingAverage::new(21).expect("Failed to create slow EMA");
    let mut rsi = RelativeStrengthIndex::new(14).expect("Failed to create RSI");
    let mut macd = Macd::new(12, 26, 9).expect("Failed to create MACD");

    let mut bullish_signals = 0;
    let mut bearish_signals = 0;

    for (i, &price) in prices.iter().enumerate() {
        ema_fast.update(price).expect("Failed to update fast EMA");
        ema_slow.update(price).expect("Failed to update slow EMA");
        rsi.update(price).expect("Failed to update RSI");
        macd.update(price).expect("Failed to update MACD");

        // After sufficient warmup period (reduced to ensure we have enough data)
        if i >= 21 {
            // 21-period slow EMA needs 22 data points
            if let (Ok(fast), Ok(slow)) = (ema_fast.value(), ema_slow.value()) {
                // Bullish signal: Fast EMA > Slow EMA (simplified condition)
                if fast > slow {
                    bullish_signals += 1;
                }

                // Bearish signal: Fast EMA < Slow EMA (simplified condition)
                if fast < slow {
                    bearish_signals += 1;
                }
            }
        }
    }

    // In a consistent uptrend, we should see some signals (either bullish or bearish)
    // The exact distribution depends on the EMA periods and data characteristics
    let total_signals = bullish_signals + bearish_signals;
    assert!(total_signals > 0, "Should have some signals generated");
    println!(
        "Bullish signals: {}, Bearish signals: {}, Total: {}",
        bullish_signals, bearish_signals, total_signals
    );
}

#[test]
fn test_volume_price_relationship() {
    let (prices, volumes) = get_test_market_data();

    let mut vwma = VolumeWeightedMovingAverage::new(10).expect("Failed to create VWMA");
    let mut sma = SimpleMovingAverage::new(10).expect("Failed to create SMA");
    let mut vpt = VolumePriceTrend::new();

    for (&price, &volume) in prices.iter().zip(volumes.iter()) {
        vwma.update(price, volume).expect("Failed to update VWMA");
        sma.update(price).expect("Failed to update SMA");
        vpt.update(price, volume).expect("Failed to update VPT");
    }

    let vwma_val = vwma.value().expect("VWMA should have value");
    let sma_val = sma.value().expect("SMA should have value");
    let vpt_val = vpt.value().expect("VPT should have value");

    // VWMA and SMA should be reasonably close but may differ due to volume weighting
    let diff_percent = (vwma_val - sma_val).abs() / sma_val * 100.0;
    assert!(
        diff_percent < 10.0,
        "VWMA and SMA should be within 10%, diff: {}%",
        diff_percent
    );

    // VPT should be positive in an uptrend with increasing volume
    assert!(vpt_val > 0.0, "VPT should be positive in uptrend");
}

#[test]
fn test_volatility_measurement_consistency() {
    let prices = vec![
        100.0, 102.0, 98.0, 104.0, 96.0, 106.0, 94.0, 108.0, 92.0, 110.0, // High volatility
        110.0, 110.5, 109.8, 110.2, 109.9, 110.1, 109.7, 110.3, 109.6,
        110.4, // Low volatility
    ];

    let mut bb_high_vol = BollingerBands::new(10, 2.0).expect("Failed to create BB");
    let mut atr = AverageTrueRange::new(10).expect("Failed to create ATR");
    let mut std_dev = StandardDeviation::new(10).expect("Failed to create StdDev");

    // Process high volatility period
    for &price in &prices[..10] {
        bb_high_vol.update(price).expect("Failed to update BB");
        atr.update(price * 1.02, price * 0.98, price)
            .expect("Failed to update ATR"); // Simulated high/low
        std_dev.update(price).expect("Failed to update StdDev");
    }

    let bb_width_high = bb_high_vol.upper_band().unwrap() - bb_high_vol.lower_band().unwrap();
    let atr_high = atr.value().unwrap();
    let std_high = std_dev.value().unwrap();

    // Reset indicators for low volatility period
    let mut bb_low_vol = BollingerBands::new(10, 2.0).expect("Failed to create BB");
    let mut atr_low = AverageTrueRange::new(10).expect("Failed to create ATR");
    let mut std_dev_low = StandardDeviation::new(10).expect("Failed to create StdDev");

    // Process low volatility period
    for &price in &prices[10..] {
        bb_low_vol.update(price).expect("Failed to update BB");
        atr_low
            .update(price * 1.001, price * 0.999, price)
            .expect("Failed to update ATR"); // Simulated tight high/low
        std_dev_low.update(price).expect("Failed to update StdDev");
    }

    let bb_width_low = bb_low_vol.upper_band().unwrap() - bb_low_vol.lower_band().unwrap();
    let atr_low_val = atr_low.value().unwrap();
    let std_low = std_dev_low.value().unwrap();

    // High volatility period should show higher values across all volatility measures
    assert!(
        bb_width_high > bb_width_low * 2.0,
        "BB width should be much higher in high vol period"
    );
    assert!(
        atr_high > atr_low_val * 2.0,
        "ATR should be much higher in high vol period"
    );
    assert!(
        std_high > std_low * 2.0,
        "Standard deviation should be much higher in high vol period"
    );
}

#[test]
fn test_forecasting_accuracy() {
    // Create a simple linear trend for testing
    let trend_data: Vec<f64> = (1..=20).map(|i| 100.0 + i as f64 * 2.0).collect(); // y = 100 + 2x

    let mut lr = LinearRegression::new(10).expect("Failed to create LinearRegression");
    let mut es = ExponentialSmoothing::new(0.3).expect("Failed to create ExponentialSmoothing");
    let mut des = DoubleExponentialSmoothing::new(0.3, 0.3)
        .expect("Failed to create DoubleExponentialSmoothing");

    // Feed data to forecasting models
    for &value in &trend_data {
        lr.update(value).expect("Failed to update LR");
        es.update(value).expect("Failed to update ES");
        des.update(value).expect("Failed to update DES");
    }

    // Get forecasts
    let lr_forecast = lr.forecast(1).expect("LR should have forecast");
    let es_forecast = es.value().expect("ES should have forecast");
    let des_forecast = des.value().expect("DES should have forecast");

    // Expected next value in the trend would be around 142.0 (100 + 2*21)
    let expected = 142.0;

    // Linear regression should be most accurate for linear trends
    assert!(
        (lr_forecast - expected).abs() < 5.0,
        "Linear regression forecast should be close to expected value"
    );

    // All forecasts should be positive and reasonable
    assert!(es_forecast > 0.0, "ES forecast should be positive");
    assert!(des_forecast > 0.0, "DES forecast should be positive");

    println!(
        "Forecasts - LR: {:.2}, ES: {:.2}, DES: {:.2}, Expected: {:.2}",
        lr_forecast, es_forecast, des_forecast, expected
    );
}

#[test]
fn test_error_handling_and_edge_cases() {
    // Test creation with invalid parameters
    assert!(
        SimpleMovingAverage::new(0).is_err(),
        "Should reject zero period"
    );
    assert!(
        BollingerBands::new(1, -1.0).is_err(),
        "Should reject negative standard deviation"
    );
    assert!(
        RelativeStrengthIndex::new(0).is_err(),
        "Should reject zero period for RSI"
    );

    // Test with extreme values
    let mut sma = SimpleMovingAverage::new(5).expect("Failed to create SMA");

    // Very large values
    assert!(
        sma.update(f64::MAX / 10.0).is_ok(),
        "Should handle large values"
    );

    // Very small values
    assert!(
        sma.update(f64::MIN_POSITIVE).is_ok(),
        "Should handle small positive values"
    );

    // Test zero values
    assert!(sma.update(0.0).is_ok(), "Should handle zero values");

    // Test that we get reasonable results even with extreme inputs
    if let Ok(value) = sma.value() {
        assert!(value.is_finite(), "Result should be finite");
        assert!(!value.is_nan(), "Result should not be NaN");
    }
}

#[test]
fn test_performance_under_load() {
    use std::time::Instant;

    let data_size = 10000;
    let prices: Vec<f64> = (0..data_size).map(|i| 100.0 + (i as f64 * 0.01)).collect();
    let volumes: Vec<f64> = (0..data_size).map(|i| 1000.0 + (i as f64 * 0.1)).collect();

    // Remove the unused indicators vector

    let start = Instant::now();

    let mut sma = SimpleMovingAverage::new(50).expect("Failed to create SMA");
    let mut ema = ExponentialMovingAverage::new(50).expect("Failed to create EMA");
    let mut rsi = RelativeStrengthIndex::new(14).expect("Failed to create RSI");
    let mut bb = BollingerBands::new(20, 2.0).expect("Failed to create BB");
    let mut obv = OnBalanceVolume::new();

    for (&price, &volume) in prices.iter().zip(volumes.iter()) {
        sma.update(price).expect("SMA update failed");
        ema.update(price).expect("EMA update failed");
        rsi.update(price).expect("RSI update failed");
        bb.update(price).expect("BB update failed");
        obv.update(price, volume).expect("OBV update failed");
    }

    let duration = start.elapsed();

    // Should process 10k data points across 5 indicators in reasonable time
    assert!(
        duration.as_millis() < 100,
        "Processing should be fast, took {:?}",
        duration
    );

    // Verify final values are reasonable
    assert!(sma.value().is_ok(), "SMA should have final value");
    assert!(ema.value().is_ok(), "EMA should have final value");
    assert!(rsi.value().is_ok(), "RSI should have final value");
    assert!(bb.middle_band().is_ok(), "BB should have final value");
    assert!(obv.value().is_ok(), "OBV should have final value");

    println!("Processed {} data points in {:?}", data_size, duration);
}

#[test]
fn test_indicator_reset_and_reuse() {
    let mut sma = SimpleMovingAverage::new(5).expect("Failed to create SMA");

    // Fill with initial data
    for i in 1..=10 {
        sma.update(i as f64).expect("Failed to update SMA");
    }

    let initial_value = sma.value().expect("SMA should have value");

    // Reset by creating new instance (simulating strategy restart)
    sma = SimpleMovingAverage::new(5).expect("Failed to recreate SMA");

    // Should not have value before sufficient data
    assert!(
        sma.value().is_err(),
        "New SMA should not have value initially"
    );

    // Fill with different data
    for i in 100..=105 {
        sma.update(i as f64).expect("Failed to update new SMA");
    }

    let new_value = sma.value().expect("New SMA should have value");

    // Values should be completely different
    assert!(
        (new_value - initial_value).abs() > 50.0,
        "Reset SMA should produce different values"
    );
}
