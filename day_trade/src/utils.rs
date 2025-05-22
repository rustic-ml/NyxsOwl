//! Utility functions and helpers for trading strategies
//!
//! Contains common implementations and utilities used across multiple strategies

use crate::{DailyOhlcv, Signal, TradeError};

/// Calculate strategy performance based on signals and data
///
/// This is a common implementation used by most strategies to calculate
/// performance metrics based on a simple buy/sell/hold trading approach.
///
/// # Arguments
/// * `data` - OHLCV data points
/// * `signals` - Trading signals corresponding to each data point
/// * `initial_cash` - Initial cash amount (default 10000.0)
///
/// # Returns
/// * Performance as percentage return
pub fn calculate_basic_performance(
    data: &[DailyOhlcv],
    signals: &[Signal],
    initial_cash: f64,
) -> Result<f64, TradeError> {
    if data.len() != signals.len() {
        return Err(TradeError::InvalidData(
            "Data and signals arrays must be the same length".to_string(),
        ));
    }

    if data.len() <= 1 {
        return Err(TradeError::InsufficientData(
            "Need at least 2 data points to calculate performance".to_string(),
        ));
    }

    let mut cash = initial_cash;
    let mut shares = 0.0;

    for i in 1..data.len() {
        match signals[i - 1] {
            Signal::Buy => {
                // Buy shares with all available cash
                shares = cash / data[i].data.open;
                cash = 0.0;
            }
            Signal::Sell => {
                // Sell all shares
                cash += shares * data[i].data.open;
                shares = 0.0;
            }
            Signal::Hold => {} // Do nothing
        }
    }

    // Calculate final portfolio value
    let final_value = cash + shares * data.last().unwrap().data.close;

    // Calculate performance as percent return
    let performance = (final_value / initial_cash - 1.0) * 100.0;

    Ok(performance)
}

/// Generate dummy OHLCV data for testing purposes
///
/// # Arguments
/// * `num_points` - Number of data points to generate
/// * `starting_price` - Initial price for the first data point
/// * `volatility` - Price volatility factor (0.0-1.0)
///
/// # Returns
/// * Vector of DailyOhlcv data points
pub fn generate_test_data(
    num_points: usize,
    starting_price: f64,
    volatility: f64,
) -> Vec<DailyOhlcv> {
    use chrono::NaiveDate;
    use rand::{thread_rng, Rng};

    let mut rng = thread_rng();
    let mut data = Vec::with_capacity(num_points);
    let mut current_price = starting_price;

    let base_date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();

    for i in 0..num_points {
        // Create a random price movement
        let price_change = current_price * volatility * (rng.random::<f64>() - 0.5);
        let open = current_price;
        let close = open + price_change;

        // High and low based on open/close with some randomness
        let high = open.max(close) + rng.random::<f64>() * volatility * open * 0.5;
        let low = open.min(close) - rng.random::<f64>() * volatility * open * 0.5;

        // Random volume between 1000 and 10000
        let volume = rng.random_range(1000..10000);

        // Create the data point
        let date = base_date
            .checked_add_days(chrono::Days::new(i as u64))
            .unwrap();
        data.push(DailyOhlcv {
            date,
            data: crate::OhlcvData {
                open,
                high,
                low,
                close,
                volume,
            },
        });

        // Update current price for next iteration
        current_price = close;
    }

    data
}

/// Basic validation for strategy parameters
pub fn validate_period(period: usize, min_value: usize) -> Result<(), String> {
    if period < min_value {
        return Err(format!("Period must be at least {}", min_value));
    }
    Ok(())
}

/// Validate a floating-point parameter is positive
pub fn validate_positive(value: f64, name: &str) -> Result<(), String> {
    if value <= 0.0 {
        return Err(format!("{} must be positive", name));
    }
    Ok(())
}

/// Validate a value is within a range
pub fn validate_range(value: f64, min: f64, max: f64, name: &str) -> Result<(), String> {
    if value < min || value > max {
        return Err(format!("{} must be between {} and {}", name, min, max));
    }
    Ok(())
}

pub mod data_generation {
    use crate::DailyOhlcv;
    use chrono::NaiveDate;
    use rand::Rng;

    /// Generate test data with a given trend and volatility
    pub fn generate_daily_data(
        days: usize,
        starting_price: f64,
        volatility: f64,
        trend: f64,
    ) -> Vec<DailyOhlcv> {
        let mut data = Vec::with_capacity(days);
        let mut current_price = starting_price;

        // Use rng instead of thread_rng
        use rand::thread_rng;
        let mut rng = thread_rng();

        for i in 0..days {
            // Add a small random component to the price change, influenced by volatility
            let price_change = current_price * volatility * (rng.random::<f64>() - 0.5);

            // Apply the trend factor (can be positive or negative)
            current_price = current_price * (1.0 + trend) + price_change;

            // Generate open price with small offset from previous close or starting price
            let open = if i == 0 {
                starting_price
            } else {
                current_price * (1.0 + (rng.random::<f64>() - 0.5) * 0.01)
            };
            let close = current_price;

            // Generate high and low with reasonable ranges
            let high = open.max(close) + rng.random::<f64>() * volatility * open * 0.5;
            let low = open.min(close) - rng.random::<f64>() * volatility * open * 0.5;

            // Generate a plausible volume
            let volume = rng.random_range(1000..10000);

            // Create a date for this candle (just for testing)
            let date = NaiveDate::from_ymd_opt(2023, 1, i as u32 % 28 + 1).unwrap();

            data.push(DailyOhlcv {
                date,
                data: crate::OhlcvData {
                    open,
                    high,
                    low,
                    close,
                    volume,
                },
            });
        }

        data
    }
}
