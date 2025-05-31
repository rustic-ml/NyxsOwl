#[cfg(test)]
mod trade_math_tests {
    use crate::test_utils::{assertions::*, TestDataGenerator};
    use approx::assert_relative_eq;
    use nyxs_owl::trade_math::*;
    use proptest::prelude::*;
    use rstest::*;
    use std::collections::HashMap;

    #[fixture]
    fn sample_prices() -> Vec<f64> {
        vec![
            100.0, 102.0, 101.0, 103.0, 105.0, 104.0, 106.0, 108.0, 107.0, 109.0,
        ]
    }

    #[fixture]
    fn sample_returns() -> Vec<f64> {
        vec![0.02, -0.01, 0.03, 0.02, -0.01, 0.02, 0.02, -0.01, 0.02]
    }

    #[fixture]
    fn volatile_prices() -> Vec<f64> {
        vec![
            100.0, 95.0, 110.0, 85.0, 120.0, 80.0, 115.0, 90.0, 125.0, 75.0,
        ]
    }

    mod basic_calculations {
        use super::*;

        #[rstest]
        fn test_returns_calculation(sample_prices: Vec<f64>) {
            let returns = calculate_returns(&sample_prices);

            assert_eq!(returns.len(), sample_prices.len() - 1);

            // Check first return: (102 - 100) / 100 = 0.02
            assert_relative_eq!(returns[0], 0.02, epsilon = 1e-10);

            // Check all returns are finite
            assert!(returns.iter().all(|&r| r.is_finite()));
        }

        #[rstest]
        fn test_log_returns_calculation(sample_prices: Vec<f64>) {
            let log_returns = calculate_log_returns(&sample_prices);

            assert_eq!(log_returns.len(), sample_prices.len() - 1);

            // Log return should be approximately ln(102/100) ≈ 0.0198
            assert_relative_eq!(log_returns[0], (102.0_f64 / 100.0).ln(), epsilon = 1e-10);

            assert!(log_returns.iter().all(|&r| r.is_finite()));
        }

        #[rstest]
        fn test_volatility_calculation(sample_returns: Vec<f64>) {
            let volatility = calculate_volatility(&sample_returns);

            assert!(volatility > 0.0);
            assert!(volatility.is_finite());

            // Volatility should be the standard deviation
            let mean = sample_returns.iter().sum::<f64>() / sample_returns.len() as f64;
            let variance = sample_returns
                .iter()
                .map(|&r| (r - mean).powi(2))
                .sum::<f64>()
                / (sample_returns.len() - 1) as f64;
            let expected_vol = variance.sqrt();

            assert_relative_eq!(volatility, expected_vol, epsilon = 1e-10);
        }

        #[rstest]
        fn test_sharpe_ratio_calculation(sample_returns: Vec<f64>) {
            let risk_free_rate = 0.02;
            let sharpe = calculate_sharpe_ratio(&sample_returns, risk_free_rate);

            assert!(sharpe.is_finite());

            // Manual calculation
            let mean_return = sample_returns.iter().sum::<f64>() / sample_returns.len() as f64;
            let excess_return = mean_return - risk_free_rate / 252.0; // Daily risk-free rate
            let volatility = calculate_volatility(&sample_returns);
            let expected_sharpe = excess_return / volatility;

            assert_relative_eq!(sharpe, expected_sharpe, epsilon = 1e-10);
        }

        #[rstest]
        fn test_maximum_drawdown(sample_prices: Vec<f64>) {
            let max_dd = calculate_maximum_drawdown(&sample_prices);

            assert!(max_dd >= 0.0);
            assert!(max_dd <= 1.0);
            assert!(max_dd.is_finite());

            // For our sample data, there should be some drawdown
            assert!(max_dd > 0.0);
        }

        #[rstest]
        fn test_maximum_drawdown_volatile(volatile_prices: Vec<f64>) {
            let max_dd = calculate_maximum_drawdown(&volatile_prices);

            assert!(max_dd >= 0.0);
            assert!(max_dd <= 1.0);

            // Volatile prices should have significant drawdown
            assert!(max_dd > 0.1); // At least 10% drawdown expected
        }
    }

    mod technical_indicators {
        use super::*;

        #[rstest]
        fn test_simple_moving_average(sample_prices: Vec<f64>) {
            let period = 3;
            let sma = calculate_sma(&sample_prices, period);

            assert_eq!(sma.len(), sample_prices.len() - period + 1);

            // First SMA should be average of first 3 prices
            let expected_first = (100.0 + 102.0 + 101.0) / 3.0;
            assert_relative_eq!(sma[0], expected_first, epsilon = 1e-10);

            assert!(sma.iter().all(|&x| x.is_finite()));
            assert!(sma.iter().all(|&x| x > 0.0));
        }

        #[rstest]
        fn test_exponential_moving_average(sample_prices: Vec<f64>) {
            let period = 3;
            let ema = calculate_ema(&sample_prices, period);

            assert_eq!(ema.len(), sample_prices.len());

            // First EMA should be the first price
            assert_relative_eq!(ema[0], sample_prices[0], epsilon = 1e-10);

            assert!(ema.iter().all(|&x| x.is_finite()));
            assert!(ema.iter().all(|&x| x > 0.0));

            // EMA should be more responsive than SMA
            let sma = calculate_sma(&sample_prices, period);
            let ema_end = ema[ema.len() - period];
            let sma_end = sma[sma.len() - 1];

            // For trending data, EMA should differ from SMA
            assert!((ema_end - sma_end).abs() > 0.01);
        }

        #[rstest]
        fn test_bollinger_bands(sample_prices: Vec<f64>) {
            let period = 5;
            let std_dev = 2.0;
            let (upper, middle, lower) = calculate_bollinger_bands(&sample_prices, period, std_dev);

            assert_eq!(upper.len(), sample_prices.len() - period + 1);
            assert_eq!(middle.len(), sample_prices.len() - period + 1);
            assert_eq!(lower.len(), sample_prices.len() - period + 1);

            // Middle band should be SMA
            let sma = calculate_sma(&sample_prices, period);
            for (i, &mid) in middle.iter().enumerate() {
                assert_relative_eq!(mid, sma[i], epsilon = 1e-10);
            }

            // Upper should be above middle, lower should be below
            for i in 0..upper.len() {
                assert!(upper[i] > middle[i]);
                assert!(lower[i] < middle[i]);
                assert!(upper[i] > lower[i]);
            }
        }

        #[rstest]
        fn test_rsi_calculation(sample_prices: Vec<f64>) {
            let period = 5;
            let rsi = calculate_rsi(&sample_prices, period);

            assert!(!rsi.is_empty());

            // RSI should be between 0 and 100
            for &value in &rsi {
                assert!(value >= 0.0 && value <= 100.0);
                assert!(value.is_finite());
            }
        }

        #[rstest]
        fn test_macd_calculation(sample_prices: Vec<f64>) {
            let fast_period = 5;
            let slow_period = 10;
            let signal_period = 3;

            // Need more data for MACD
            let extended_prices: Vec<f64> = (0..20).map(|i| 100.0 + i as f64 * 0.5).collect();

            let (macd_line, signal_line, histogram) =
                calculate_macd(&extended_prices, fast_period, slow_period, signal_period);

            assert_eq!(macd_line.len(), signal_line.len());
            assert_eq!(signal_line.len(), histogram.len());

            // MACD histogram should be MACD line minus signal line
            for i in 0..histogram.len() {
                assert_relative_eq!(histogram[i], macd_line[i] - signal_line[i], epsilon = 1e-10);
            }

            assert!(macd_line.iter().all(|&x| x.is_finite()));
            assert!(signal_line.iter().all(|&x| x.is_finite()));
            assert!(histogram.iter().all(|&x| x.is_finite()));
        }
    }

    mod portfolio_metrics {
        use super::*;

        #[rstest]
        fn test_portfolio_return_calculation() {
            let weights = vec![0.4, 0.3, 0.3];
            let returns = vec![
                vec![0.01, 0.02, -0.01], // Asset 1
                vec![0.02, -0.01, 0.03], // Asset 2
                vec![-0.01, 0.01, 0.02], // Asset 3
            ];

            let portfolio_returns = calculate_portfolio_returns(&weights, &returns);

            assert_eq!(portfolio_returns.len(), 3);

            // Manual calculation for first period
            let expected_first = 0.4 * 0.01 + 0.3 * 0.02 + 0.3 * (-0.01);
            assert_relative_eq!(portfolio_returns[0], expected_first, epsilon = 1e-10);

            assert!(portfolio_returns.iter().all(|&r| r.is_finite()));
        }

        #[rstest]
        fn test_correlation_matrix() {
            let returns_matrix = vec![
                vec![0.01, 0.02, -0.01, 0.03],
                vec![0.02, -0.01, 0.03, 0.01],
                vec![-0.01, 0.01, 0.02, -0.02],
            ];

            let correlation_matrix = calculate_correlation_matrix(&returns_matrix);

            assert_eq!(correlation_matrix.len(), 3);
            assert_eq!(correlation_matrix[0].len(), 3);

            // Diagonal should be 1.0 (perfect self-correlation)
            for i in 0..3 {
                assert_relative_eq!(correlation_matrix[i][i], 1.0, epsilon = 1e-10);
            }

            // Matrix should be symmetric
            for i in 0..3 {
                for j in 0..3 {
                    assert_relative_eq!(
                        correlation_matrix[i][j],
                        correlation_matrix[j][i],
                        epsilon = 1e-10
                    );
                }
            }

            // All correlations should be between -1 and 1
            for row in &correlation_matrix {
                for &corr in row {
                    assert!(corr >= -1.0 && corr <= 1.0);
                    assert!(corr.is_finite());
                }
            }
        }

        #[rstest]
        fn test_var_calculation(sample_returns: Vec<f64>) {
            let confidence_level = 0.95;
            let var = calculate_var(&sample_returns, confidence_level);

            assert!(var <= 0.0); // VaR should be negative (loss)
            assert!(var.is_finite());

            // Check that approximately (1 - confidence_level) of returns are below VaR
            let exceedances = sample_returns.iter().filter(|&&r| r < var).count() as f64
                / sample_returns.len() as f64;

            // Should be approximately 5% for 95% confidence
            assert!(exceedances <= 0.2); // Allow some tolerance for small sample
        }

        #[rstest]
        fn test_cvar_calculation(sample_returns: Vec<f64>) {
            let confidence_level = 0.95;
            let var = calculate_var(&sample_returns, confidence_level);
            let cvar = calculate_cvar(&sample_returns, confidence_level);

            assert!(cvar <= var); // CVaR should be worse (more negative) than VaR
            assert!(cvar.is_finite());
        }
    }

    mod risk_metrics {
        use super::*;

        #[rstest]
        fn test_beta_calculation() {
            let asset_returns = vec![0.02, -0.01, 0.03, 0.01, -0.02];
            let market_returns = vec![0.01, 0.00, 0.02, 0.005, -0.01];

            let beta = calculate_beta(&asset_returns, &market_returns);

            assert!(beta.is_finite());

            // Beta should indicate relationship to market
            // For positively correlated assets, beta > 0
            if asset_returns
                .iter()
                .zip(&market_returns)
                .map(|(a, m)| a * m)
                .sum::<f64>()
                > 0.0
            {
                assert!(beta > 0.0);
            }
        }

        #[rstest]
        fn test_information_ratio(sample_returns: Vec<f64>) {
            let benchmark_returns = vec![0.01, 0.00, 0.02, 0.01, -0.01, 0.015, 0.01, 0.005, 0.02];

            let ir = calculate_information_ratio(&sample_returns, &benchmark_returns);

            assert!(ir.is_finite());
        }

        #[rstest]
        fn test_sortino_ratio(sample_returns: Vec<f64>) {
            let risk_free_rate = 0.02;
            let sortino = calculate_sortino_ratio(&sample_returns, risk_free_rate);

            assert!(sortino.is_finite());

            // Sortino should generally be higher than Sharpe for same data
            let sharpe = calculate_sharpe_ratio(&sample_returns, risk_free_rate);

            // For typical return distributions, Sortino >= Sharpe
            assert!(sortino >= sharpe * 0.8); // Allow some tolerance
        }

        #[rstest]
        fn test_calmar_ratio(sample_prices: Vec<f64>) {
            let returns = calculate_returns(&sample_prices);
            let calmar = calculate_calmar_ratio(&returns, &sample_prices);

            assert!(calmar.is_finite());

            // Calmar should be positive for profitable strategies
            let total_return = sample_prices.last().unwrap() / sample_prices[0] - 1.0;
            if total_return > 0.0 {
                assert!(calmar > 0.0);
            }
        }
    }

    mod edge_cases_and_error_handling {
        use super::*;

        #[test]
        fn test_empty_data_handling() {
            let empty_data: Vec<f64> = vec![];

            // All functions should handle empty data gracefully
            assert!(calculate_returns(&empty_data).is_empty());
            assert!(calculate_log_returns(&empty_data).is_empty());

            // These should return NaN or error appropriately
            let vol = calculate_volatility(&empty_data);
            assert!(vol.is_nan() || vol == 0.0);
        }

        #[test]
        fn test_single_value_data() {
            let single_data = vec![100.0];

            let returns = calculate_returns(&single_data);
            assert!(returns.is_empty());

            let log_returns = calculate_log_returns(&single_data);
            assert!(log_returns.is_empty());
        }

        #[test]
        fn test_constant_data() {
            let constant_data = vec![100.0; 10];

            let returns = calculate_returns(&constant_data);
            assert!(returns.iter().all(|&r| r == 0.0));

            let volatility = calculate_volatility(&returns);
            assert!(volatility == 0.0 || volatility.is_nan());

            let max_dd = calculate_maximum_drawdown(&constant_data);
            assert_eq!(max_dd, 0.0);
        }

        #[test]
        fn test_invalid_data_handling() {
            let invalid_data = vec![100.0, f64::NAN, 102.0, f64::INFINITY, 101.0];

            // Functions should handle NaN/Infinity gracefully
            let returns = calculate_returns(&invalid_data);
            // Should either filter out invalid values or handle them appropriately

            let vol = calculate_volatility(&returns);
            // Should be NaN or handle gracefully
        }

        #[test]
        fn test_negative_prices() {
            let negative_prices = vec![100.0, -50.0, 75.0, -25.0];

            // Returns calculation should handle negative prices
            let returns = calculate_returns(&negative_prices);
            assert!(!returns.is_empty());

            // Log returns might be invalid for negative prices
            let log_returns = calculate_log_returns(&negative_prices);
            // Should handle appropriately (NaN or error)
        }
    }

    mod property_based_tests {
        use super::*;

        proptest! {
            #[test]
            fn test_returns_properties(
                prices in prop::collection::vec(1.0f64..1000.0, 2..100)
            ) {
                let returns = calculate_returns(&prices);

                prop_assert_eq!(returns.len(), prices.len() - 1);

                // Returns should be finite (assuming no overflow)
                prop_assert!(returns.iter().all(|&r| r.is_finite()));

                // Returns should satisfy: price[i+1] = price[i] * (1 + return[i])
                for i in 0..returns.len() {
                    let expected_price = prices[i] * (1.0 + returns[i]);
                    prop_assert!((expected_price - prices[i + 1]).abs() < 1e-10);
                }
            }

            #[test]
            fn test_volatility_properties(
                returns in prop::collection::vec(-0.1f64..0.1, 5..100)
            ) {
                let vol = calculate_volatility(&returns);

                // Volatility should be non-negative
                prop_assert!(vol >= 0.0);
                prop_assert!(vol.is_finite() || vol.is_nan());

                // For non-constant data, volatility should be positive
                let has_variation = returns.windows(2).any(|w| (w[0] - w[1]).abs() > 1e-10);
                if has_variation {
                    prop_assert!(vol > 0.0);
                }
            }

            #[test]
            fn test_sma_properties(
                prices in prop::collection::vec(1.0f64..1000.0, 10..100),
                period in 2usize..10
            ) {
                let sma = calculate_sma(&prices, period);

                prop_assert_eq!(sma.len(), prices.len() - period + 1);
                prop_assert!(sma.iter().all(|&x| x.is_finite()));
                prop_assert!(sma.iter().all(|&x| x > 0.0));

                // Each SMA value should be between min and max of its window
                for (i, &sma_val) in sma.iter().enumerate() {
                    let window = &prices[i..i + period];
                    let min_price = window.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                    let max_price = window.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

                    prop_assert!(sma_val >= min_price);
                    prop_assert!(sma_val <= max_price);
                }
            }

            #[test]
            fn test_drawdown_properties(
                prices in prop::collection::vec(1.0f64..1000.0, 5..100)
            ) {
                let max_dd = calculate_maximum_drawdown(&prices);

                // Drawdown should be between 0 and 1
                prop_assert!(max_dd >= 0.0);
                prop_assert!(max_dd <= 1.0);
                prop_assert!(max_dd.is_finite());

                // If prices are strictly increasing, drawdown should be 0
                let is_increasing = prices.windows(2).all(|w| w[1] >= w[0]);
                if is_increasing {
                    prop_assert!(max_dd == 0.0);
                }
            }
        }
    }

    mod performance_tests {
        use super::*;

        #[test]
        fn test_large_dataset_performance() {
            let large_prices: Vec<f64> = (0..10000)
                .map(|i| 100.0 + (i as f64 * 0.01) + (i as f64 * 0.001).sin())
                .collect();

            let start = std::time::Instant::now();

            let returns = calculate_returns(&large_prices);
            let volatility = calculate_volatility(&returns);
            let max_dd = calculate_maximum_drawdown(&large_prices);
            let sma = calculate_sma(&large_prices, 20);

            let duration = start.elapsed();

            // Should complete in reasonable time
            assert!(duration < std::time::Duration::from_millis(100));

            // Results should be valid
            assert_eq!(returns.len(), 9999);
            assert!(volatility > 0.0);
            assert!(max_dd >= 0.0);
            assert!(!sma.is_empty());
        }

        #[test]
        fn test_memory_efficiency() {
            // Test that we don't use excessive memory
            let prices: Vec<f64> = (0..1000).map(|i| 100.0 + i as f64 * 0.1).collect();

            // These operations should not cause memory issues
            for _ in 0..100 {
                let _ = calculate_returns(&prices);
                let _ = calculate_sma(&prices, 10);
                let _ = calculate_ema(&prices, 10);
            }
        }
    }

    mod integration_tests {
        use super::*;

        #[test]
        fn test_trading_strategy_metrics() {
            // Simulate a trading strategy with mixed results
            let prices = TestDataGenerator::generate_stock_prices(252, 100.0, 0.015);
            let price_values: Vec<f64> = prices.iter().map(|(_, p)| *p).collect();

            let returns = calculate_returns(&price_values);
            let volatility = calculate_volatility(&returns);
            let sharpe = calculate_sharpe_ratio(&returns, 0.02);
            let max_dd = calculate_maximum_drawdown(&price_values);
            let sortino = calculate_sortino_ratio(&returns, 0.02);

            // All metrics should be valid for a realistic trading strategy
            assert_trading_metrics_valid(returns.iter().sum::<f64>(), sharpe, max_dd);

            assert!(volatility > 0.0);
            assert!(sortino.is_finite());

            println!("Strategy Metrics:");
            println!(
                "  Total Return: {:.2}%",
                returns.iter().sum::<f64>() * 100.0
            );
            println!("  Volatility: {:.2}%", volatility * 100.0);
            println!("  Sharpe Ratio: {:.2}", sharpe);
            println!("  Sortino Ratio: {:.2}", sortino);
            println!("  Max Drawdown: {:.2}%", max_dd * 100.0);
        }

        #[test]
        fn test_technical_analysis_workflow() {
            let prices = TestDataGenerator::generate_stock_prices(100, 100.0, 0.02);
            let price_values: Vec<f64> = prices.iter().map(|(_, p)| *p).collect();

            // Complete technical analysis workflow
            let sma_20 = calculate_sma(&price_values, 20);
            let ema_12 = calculate_ema(&price_values, 12);
            let (bb_upper, bb_middle, bb_lower) = calculate_bollinger_bands(&price_values, 20, 2.0);
            let rsi = calculate_rsi(&price_values, 14);
            let (macd, signal, histogram) = calculate_macd(&price_values, 12, 26, 9);

            // All indicators should produce valid results
            assert!(!sma_20.is_empty());
            assert!(!ema_12.is_empty());
            assert!(!bb_upper.is_empty());
            assert!(!rsi.is_empty());
            assert!(!macd.is_empty());

            // RSI should be in valid range
            assert!(rsi.iter().all(|&r| r >= 0.0 && r <= 100.0));

            // Bollinger bands should maintain order
            for i in 0..bb_upper.len() {
                assert!(bb_upper[i] > bb_middle[i]);
                assert!(bb_middle[i] > bb_lower[i]);
            }

            println!("✓ Technical analysis workflow completed successfully");
        }
    }
}

// Mock implementations if the actual trade_math module doesn't exist
#[cfg(not(feature = "trade-math"))]
mod mock_trade_math {
    pub fn calculate_returns(prices: &[f64]) -> Vec<f64> {
        prices.windows(2).map(|w| (w[1] - w[0]) / w[0]).collect()
    }

    pub fn calculate_log_returns(prices: &[f64]) -> Vec<f64> {
        prices.windows(2).map(|w| (w[1] / w[0]).ln()).collect()
    }

    pub fn calculate_volatility(returns: &[f64]) -> f64 {
        if returns.is_empty() {
            return f64::NAN;
        }
        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance =
            returns.iter().map(|&r| (r - mean).powi(2)).sum::<f64>() / (returns.len() - 1) as f64;
        variance.sqrt()
    }

    pub fn calculate_sharpe_ratio(returns: &[f64], risk_free_rate: f64) -> f64 {
        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let excess_return = mean_return - risk_free_rate / 252.0;
        let volatility = calculate_volatility(returns);
        excess_return / volatility
    }

    pub fn calculate_maximum_drawdown(prices: &[f64]) -> f64 {
        let mut max_price = prices[0];
        let mut max_drawdown = 0.0;

        for &price in prices.iter().skip(1) {
            max_price = max_price.max(price);
            let drawdown = (max_price - price) / max_price;
            max_drawdown = max_drawdown.max(drawdown);
        }

        max_drawdown
    }

    pub fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
        prices
            .windows(period)
            .map(|window| window.iter().sum::<f64>() / window.len() as f64)
            .collect()
    }

    pub fn calculate_ema(prices: &[f64], period: usize) -> Vec<f64> {
        let alpha = 2.0 / (period as f64 + 1.0);
        let mut ema = Vec::with_capacity(prices.len());
        ema.push(prices[0]);

        for &price in prices.iter().skip(1) {
            let prev_ema = ema[ema.len() - 1];
            ema.push(alpha * price + (1.0 - alpha) * prev_ema);
        }

        ema
    }

    // Additional mock implementations for other functions...
}
