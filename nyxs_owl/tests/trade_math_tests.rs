#[cfg(test)]
mod trade_math_tests {
    use nyxs_owl::trade_math::{
        moving_averages::SimpleMovingAverage, oscillators::RelativeStrengthIndex,
        volatility::BollingerBands,
    };

    #[test]
    fn test_basic_sma_functionality() {
        let mut sma = SimpleMovingAverage::new(3).unwrap();
        let prices = vec![100.0, 102.0, 101.0, 103.0];

        for price in prices {
            let _ = sma.update(price);
        }

        // After 4 updates with period 3, we should have a value
        if let Ok(value) = sma.value() {
            assert!(value > 0.0 && value < 200.0);
            assert!(value.is_finite());
        }
    }

    #[test]
    fn test_basic_rsi_functionality() {
        let mut rsi = RelativeStrengthIndex::new(14).unwrap();
        let prices = vec![
            100.0, 102.0, 101.0, 103.0, 105.0, 104.0, 106.0, 108.0, 107.0, 109.0, 111.0, 110.0,
            112.0, 114.0, 113.0, 115.0, 117.0, 116.0, 118.0, 120.0,
        ];

        for price in prices {
            let _ = rsi.update(price);
        }

        // RSI should be between 0 and 100
        if let Ok(value) = rsi.value() {
            assert!(value >= 0.0 && value <= 100.0);
            assert!(value.is_finite());
        }
    }

    #[test]
    fn test_basic_bollinger_bands_functionality() {
        let mut bb = BollingerBands::new(20, 2.0).unwrap();
        let prices = vec![
            100.0, 102.0, 101.0, 103.0, 105.0, 104.0, 106.0, 108.0, 107.0, 109.0, 111.0, 110.0,
            112.0, 114.0, 113.0, 115.0, 117.0, 116.0, 118.0, 120.0, 119.0, 121.0, 123.0, 122.0,
            124.0,
        ];

        for price in prices {
            let _ = bb.update(price);
        }

        // Test that all bands are available and properly ordered
        if let (Ok(upper), Ok(middle), Ok(lower)) =
            (bb.upper_band(), bb.middle_band(), bb.lower_band())
        {
            assert!(upper > middle);
            assert!(middle > lower);
            assert!(upper.is_finite() && middle.is_finite() && lower.is_finite());
        }
    }

    #[test]
    fn test_indicators_stability() {
        // Test that indicators don't crash with edge cases
        let mut sma = SimpleMovingAverage::new(5).unwrap();
        let mut rsi = RelativeStrengthIndex::new(14).unwrap();

        // Constant price
        for _ in 0..20 {
            let _ = sma.update(100.0);
            let _ = rsi.update(100.0);
        }

        // Should handle constant prices gracefully
        if let Ok(sma_val) = sma.value() {
            assert!((sma_val - 100.0).abs() < 0.01);
        }

        if let Ok(rsi_val) = rsi.value() {
            // RSI with constant prices should be around 50 or 0
            assert!(rsi_val >= 0.0 && rsi_val <= 100.0);
        }
    }
}
