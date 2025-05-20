//! Utility functions for intraday trading strategies
//!
//! This module provides helper functions for data loading, strategy evaluation,
//! common calculations, and validation functions.

use crate::{MinuteOhlcv, OhlcvData, PerformanceMetrics, Signal, Trade, TradeError};
use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveTime, Timelike, Utc};
use std::cmp;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Load minute-level OHLCV data from a CSV file
///
/// The expected CSV format is:
/// timestamp,open,high,low,close,volume
/// 2023-01-01T09:30:00Z,100.5,101.2,100.4,100.8,5000
///
/// # Arguments
/// * `file_path` - Path to the CSV file
///
/// # Returns
/// * `Result<Vec<MinuteOhlcv>, TradeError>` - Loaded data or error
pub fn load_minute_data<P: AsRef<Path>>(file_path: P) -> Result<Vec<MinuteOhlcv>, TradeError> {
    let file = File::open(file_path)
        .map_err(|e| TradeError::DataLoadError(format!("Failed to open file: {}", e)))?;

    let reader = BufReader::new(file);
    let mut data = Vec::new();
    let mut lines = reader.lines();

    // Skip header row
    let _ = lines.next();

    for (i, line) in lines.enumerate() {
        let line = line.map_err(|e| {
            TradeError::DataLoadError(format!("Error reading line {}: {}", i + 2, e))
        })?;

        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() != 6 {
            return Err(TradeError::DataLoadError(format!(
                "Invalid CSV format at line {}, expected 6 fields",
                i + 2
            )));
        }

        let timestamp = fields[0].parse::<DateTime<Utc>>().map_err(|e| {
            TradeError::DataLoadError(format!("Invalid timestamp at line {}: {}", i + 2, e))
        })?;

        let open = fields[1].parse::<f64>().map_err(|e| {
            TradeError::DataLoadError(format!("Invalid open price at line {}: {}", i + 2, e))
        })?;

        let high = fields[2].parse::<f64>().map_err(|e| {
            TradeError::DataLoadError(format!("Invalid high price at line {}: {}", i + 2, e))
        })?;

        let low = fields[3].parse::<f64>().map_err(|e| {
            TradeError::DataLoadError(format!("Invalid low price at line {}: {}", i + 2, e))
        })?;

        let close = fields[4].parse::<f64>().map_err(|e| {
            TradeError::DataLoadError(format!("Invalid close price at line {}: {}", i + 2, e))
        })?;

        let volume = fields[5].parse::<f64>().map_err(|e| {
            TradeError::DataLoadError(format!("Invalid volume at line {}: {}", i + 2, e))
        })?;

        data.push(MinuteOhlcv {
            timestamp,
            data: OhlcvData {
                open,
                high,
                low,
                close,
                volume,
            },
        });
    }

    if data.is_empty() {
        return Err(TradeError::DataLoadError(
            "No data found in file".to_string(),
        ));
    }

    // Sort data by timestamp
    data.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    Ok(data)
}

/// Generate synthetic minute-level data for testing strategies
///
/// # Arguments
/// * `days` - Number of trading days to generate
/// * `points_per_day` - Number of data points per day (e.g., 390 for a 6.5 hour trading day)
/// * `base_price` - Starting price
/// * `volatility` - Price volatility factor (0.0-1.0)
/// * `trend` - Daily trend factor (-0.01 to 0.01 for reasonable values)
///
/// # Returns
/// * `Vec<MinuteOhlcv>` - Generated data
pub fn generate_minute_data(
    days: usize,
    points_per_day: usize,
    base_price: f64,
    volatility: f64,
    trend: f64,
) -> Vec<MinuteOhlcv> {
    use rand::{thread_rng, Rng};

    let mut random = thread_rng();
    let mut data = Vec::with_capacity(days * points_per_day);
    let mut current_price = base_price;

    // Create base date
    let base_date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
    let market_open = NaiveTime::from_hms_opt(9, 30, 0).unwrap();

    for day in 0..days {
        let current_date = base_date + chrono::Days::new(day as u64);

        // Skip weekends
        if current_date.weekday().num_days_from_monday() > 4 {
            continue;
        }

        for minute in 0..points_per_day {
            // Calculate timestamp (9:30 AM + minute)
            let time = market_open + Duration::minutes(minute as i64);
            let datetime = current_date.and_time(time);
            let timestamp = DateTime::<Utc>::from_naive_utc_and_offset(datetime.into(), Utc);

            // Add intraday volatility pattern (more at open and close)
            let minute_factor = minute as f64 / points_per_day as f64;
            let intraday_volatility = 1.0 + 0.5 * (-4.0 * (minute_factor - 0.5).powi(2) + 1.0);

            // Create a random price movement with volatility that varies throughout the day
            let price_change =
                current_price * volatility * intraday_volatility * (random.gen::<f64>() - 0.5);
            let daily_trend = current_price * trend;

            // Set prices
            let open = current_price;
            current_price = open + price_change + daily_trend;
            let close = current_price;

            // High and low based on open/close with some randomness
            let high = open.max(close) + random.gen::<f64>() * volatility * open * 0.2;
            let low = open.min(close) - random.gen::<f64>() * volatility * open * 0.2;

            // Volume with U-shape pattern (higher at open and close)
            let volume_base = 1000.0 + 5000.0 * intraday_volatility;
            let volume = volume_base * (0.5 + random.gen::<f64>());

            data.push(MinuteOhlcv {
                timestamp,
                data: OhlcvData {
                    open,
                    high,
                    low,
                    close,
                    volume,
                },
            });
        }
    }

    data
}

/// Calculate basic performance of a strategy based on signals
///
/// # Arguments
/// * `data` - OHLCV data points
/// * `signals` - Trading signals corresponding to each data point
/// * `initial_cash` - Initial cash amount
/// * `commission` - Commission per trade (as percentage)
///
/// # Returns
/// * `Result<f64, TradeError>` - Performance as percentage return
pub fn calculate_basic_performance(
    data: &[MinuteOhlcv],
    signals: &[Signal],
    initial_cash: f64,
    commission: f64,
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
            Signal::Buy if shares == 0.0 => {
                // Buy shares with all available cash
                let price = data[i].data.open;
                shares = cash / price * (1.0 - commission / 100.0);
                cash = 0.0;
            }
            Signal::Sell if shares > 0.0 => {
                // Sell all shares
                let price = data[i].data.open;
                cash += shares * price * (1.0 - commission / 100.0);
                shares = 0.0;
            }
            _ => {} // Do nothing for hold or repeated signals
        }
    }

    // Calculate final portfolio value
    let final_value = cash + shares * data.last().unwrap().data.close * (1.0 - commission / 100.0);

    // Calculate performance as percent return
    let performance = (final_value / initial_cash - 1.0) * 100.0;

    Ok(performance)
}

/// Calculate detailed performance metrics for a strategy
///
/// # Arguments
/// * `data` - OHLCV data points
/// * `signals` - Trading signals corresponding to each data point
/// * `initial_cash` - Initial cash amount
/// * `commission` - Commission per trade (as percentage)
///
/// # Returns
/// * `Result<PerformanceMetrics, TradeError>` - Detailed performance metrics
pub fn calculate_detailed_performance(
    data: &[MinuteOhlcv],
    signals: &[Signal],
    initial_cash: f64,
    commission: f64,
) -> Result<PerformanceMetrics, TradeError> {
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
    let mut trades: Vec<Trade> = Vec::new();
    let mut current_trade: Option<Trade> = None;
    let mut daily_returns = Vec::new();
    let mut portfolio_values = Vec::with_capacity(data.len());

    portfolio_values.push(initial_cash);

    // Calculate trades and daily returns
    for i in 1..data.len() {
        match signals[i - 1] {
            Signal::Buy if shares == 0.0 => {
                // Buy shares with all available cash
                let price = data[i].data.open;
                shares = cash / price * (1.0 - commission / 100.0);
                cash = 0.0;

                current_trade = Some(Trade {
                    entry_time: data[i].timestamp,
                    exit_time: None,
                    entry_price: price,
                    exit_price: None,
                    size: shares,
                    is_long: true,
                    pnl: None,
                });
            }
            Signal::Sell if shares > 0.0 => {
                // Sell all shares
                let price = data[i].data.open;
                let sale_value = shares * price * (1.0 - commission / 100.0);

                // Complete the current trade
                if let Some(mut trade) = current_trade.take() {
                    trade.exit_time = Some(data[i].timestamp);
                    trade.exit_price = Some(price);
                    let entry_value = trade.size * trade.entry_price;
                    trade.pnl = Some(sale_value - entry_value);
                    trades.push(trade);
                }

                cash += sale_value;
                shares = 0.0;
            }
            _ => {} // Do nothing for hold or repeated signals
        }

        // Calculate portfolio value at this point
        let portfolio_value = cash + shares * data[i].data.close;
        portfolio_values.push(portfolio_value);

        // Check if this is a new day
        if i > 1 && data[i].timestamp.date_naive() != data[i - 1].timestamp.date_naive() {
            let prev_day_value = portfolio_values[i - 1];
            let today_value = portfolio_value;
            let daily_return = (today_value / prev_day_value) - 1.0;
            daily_returns.push(daily_return);
        }
    }

    // Ensure any open trade is closed for the calculation
    if let Some(mut trade) = current_trade {
        let last_price = data.last().unwrap().data.close;
        trade.exit_time = Some(data.last().unwrap().timestamp);
        trade.exit_price = Some(last_price);
        let entry_value = trade.size * trade.entry_price;
        let exit_value = trade.size * last_price * (1.0 - commission / 100.0);
        trade.pnl = Some(exit_value - entry_value);
        trades.push(trade);
    }

    // Calculate strategy metrics
    let final_value = portfolio_values.last().unwrap_or(&initial_cash);
    let total_return = (final_value / initial_cash - 1.0) * 100.0;

    // Calculate maximum drawdown
    let mut max_drawdown: f64 = 0.0;
    let mut peak = initial_cash;

    for &value in &portfolio_values {
        if value > peak {
            peak = value;
        } else {
            let drawdown = (peak - value) / peak;
            max_drawdown = max_drawdown.max(drawdown);
        }
    }

    // Calculate win rate and profit factor
    let (wins, losses): (Vec<&Trade>, Vec<&Trade>) = trades
        .iter()
        .filter(|t| t.pnl.is_some())
        .partition(|t| t.pnl.unwrap() > 0.0);

    let win_rate = if trades.is_empty() {
        0.0
    } else {
        wins.len() as f64 / trades.len() as f64 * 100.0
    };

    let gross_profit: f64 = wins.iter().fold(0.0, |sum, t| sum + t.pnl.unwrap_or(0.0));
    let gross_loss: f64 = losses
        .iter()
        .fold(0.0, |sum, t| sum + t.pnl.unwrap_or(0.0).abs());

    let profit_factor = if gross_loss.abs() < f64::EPSILON {
        if gross_profit > 0.0 {
            f64::INFINITY
        } else {
            0.0
        }
    } else {
        gross_profit / gross_loss.abs()
    };

    // Calculate annualized return
    let days = if daily_returns.is_empty() {
        1.0
    } else {
        daily_returns.len() as f64
    };

    let annualized_return = ((1.0 + total_return / 100.0).powf(252.0 / days) - 1.0) * 100.0;

    // Calculate Sharpe ratio
    let avg_daily_return = daily_returns.iter().sum::<f64>() / days;
    let std_dev = if daily_returns.len() <= 1 {
        1.0 // Default to 1.0 if we don't have enough data
    } else {
        let variance = daily_returns
            .iter()
            .map(|r| (r - avg_daily_return).powi(2))
            .sum::<f64>()
            / (daily_returns.len() as f64 - 1.0);
        variance.sqrt()
    };

    let sharpe_ratio = if std_dev.abs() < f64::EPSILON {
        0.0
    } else {
        (avg_daily_return / std_dev) * (252.0_f64).sqrt()
    };

    Ok(PerformanceMetrics {
        total_return,
        annualized_return,
        sharpe_ratio,
        max_drawdown: max_drawdown * 100.0,
        win_rate,
        profit_factor,
        total_trades: trades.len(),
    })
}

/// Calculate simple moving average
///
/// # Arguments
/// * `data` - Price data
/// * `period` - Moving average period
///
/// # Returns
/// * `Vec<Option<f64>>` - Moving average values (None for the first period-1 points)
pub fn calculate_sma(data: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = vec![None; data.len()];
    if data.len() < period {
        return result;
    }

    let mut sum = data.iter().take(period).sum::<f64>();
    result[period - 1] = Some(sum / period as f64);

    for i in period..data.len() {
        sum = sum - data[i - period] + data[i];
        result[i] = Some(sum / period as f64);
    }

    result
}

/// Calculate exponential moving average
///
/// # Arguments
/// * `data` - Price data
/// * `period` - EMA period
///
/// # Returns
/// * `Vec<Option<f64>>` - EMA values (None for the first period-1 points)
pub fn calculate_ema(data: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = vec![None; data.len()];
    if data.len() < period {
        return result;
    }

    // Start with SMA
    let sma = data.iter().take(period).sum::<f64>() / period as f64;
    result[period - 1] = Some(sma);

    // Calculate multiplier: (2 / (period + 1))
    let multiplier = 2.0 / (period as f64 + 1.0);

    // Calculate EMA: EMA = Closing price x multiplier + EMA(previous day) x (1 - multiplier)
    for i in period..data.len() {
        let prev_ema = result[i - 1].unwrap();
        let ema = data[i] * multiplier + prev_ema * (1.0 - multiplier);
        result[i] = Some(ema);
    }

    result
}

/// Calculate Bollinger Bands
///
/// # Arguments
/// * `data` - Price data
/// * `period` - Moving average period
/// * `std_dev_multiplier` - Standard deviation multiplier
///
/// # Returns
/// * `(Vec<Option<f64>>, Vec<Option<f64>>, Vec<Option<f64>>)` - (Middle Band, Upper Band, Lower Band)
pub fn calculate_bollinger_bands(
    data: &[f64],
    period: usize,
    std_dev_multiplier: f64,
) -> (Vec<Option<f64>>, Vec<Option<f64>>, Vec<Option<f64>>) {
    let mut middle_band = vec![None; data.len()];
    let mut upper_band = vec![None; data.len()];
    let mut lower_band = vec![None; data.len()];

    if data.len() < period {
        return (middle_band, upper_band, lower_band);
    }

    for i in (period - 1)..data.len() {
        let slice = &data[(i - period + 1)..=i];
        let mean = slice.iter().sum::<f64>() / period as f64;

        let variance = slice.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / period as f64;

        let std_dev = variance.sqrt();

        middle_band[i] = Some(mean);
        upper_band[i] = Some(mean + std_dev_multiplier * std_dev);
        lower_band[i] = Some(mean - std_dev_multiplier * std_dev);
    }

    (middle_band, upper_band, lower_band)
}

/// Calculate Relative Strength Index (RSI)
///
/// # Arguments
/// * `data` - Price data
/// * `period` - RSI period
///
/// # Returns
/// * `Vec<Option<f64>>` - RSI values (None for the first period points)
pub fn calculate_rsi(data: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = vec![None; data.len()];
    if data.len() <= period {
        return result;
    }

    let mut gains = Vec::with_capacity(data.len() - 1);
    let mut losses = Vec::with_capacity(data.len() - 1);

    // Calculate price changes
    for i in 1..data.len() {
        let change = data[i] - data[i - 1];
        gains.push(change.max(0.0));
        losses.push((-change).max(0.0));
    }

    // Calculate initial averages
    let avg_gain = gains.iter().take(period).sum::<f64>() / period as f64;
    let avg_loss = losses.iter().take(period).sum::<f64>() / period as f64;

    // Calculate first RSI
    let rs = if avg_loss == 0.0 {
        100.0
    } else {
        avg_gain / avg_loss
    };
    let rsi = 100.0 - (100.0 / (1.0 + rs));
    result[period] = Some(rsi);

    // Calculate remaining RSI values
    let mut prev_avg_gain = avg_gain;
    let mut prev_avg_loss = avg_loss;

    for i in (period + 1)..data.len() {
        let current_gain = gains[i - 1];
        let current_loss = losses[i - 1];

        let avg_gain = (prev_avg_gain * (period as f64 - 1.0) + current_gain) / period as f64;
        let avg_loss = (prev_avg_loss * (period as f64 - 1.0) + current_loss) / period as f64;

        prev_avg_gain = avg_gain;
        prev_avg_loss = avg_loss;

        let rs = if avg_loss == 0.0 {
            100.0
        } else {
            avg_gain / avg_loss
        };
        let rsi = 100.0 - (100.0 / (1.0 + rs));
        result[i] = Some(rsi);
    }

    result
}

/// Check if time is within market hours
///
/// # Arguments
/// * `timestamp` - Time to check
///
/// # Returns
/// * `bool` - Whether the time is within market hours (9:30 AM - 4:00 PM ET, weekdays)
pub fn is_market_hours(timestamp: DateTime<Utc>) -> bool {
    // Assume timestamps are in UTC
    // Convert to Eastern Time (UTC-5, ignoring daylight savings for simplicity)
    // This is a simplified approach - in production, you'd want to handle time zones properly
    let et_hour = (timestamp.hour() + 24 - 5) % 24;
    let et_minute = timestamp.minute();

    // Check if it's a weekday
    let weekday = timestamp.weekday().num_days_from_monday();
    if weekday >= 5 {
        return false; // Weekend
    }

    // Check if time is between 9:30 AM and 4:00 PM ET
    if et_hour < 9 || et_hour > 16 {
        return false;
    }

    if et_hour == 9 && et_minute < 30 {
        return false;
    }

    true
}

/// Validate a period parameter
///
/// # Arguments
/// * `period` - Period to validate
/// * `min_value` - Minimum allowed value
///
/// # Returns
/// * `Result<(), String>` - Ok or error message
pub fn validate_period(period: usize, min_value: usize) -> Result<(), String> {
    if period < min_value {
        return Err(format!("Period must be at least {}", min_value));
    }
    Ok(())
}

/// Validate a floating-point parameter is positive
///
/// # Arguments
/// * `value` - Value to validate
/// * `name` - Parameter name for error messages
///
/// # Returns
/// * `Result<(), String>` - Ok or error message
pub fn validate_positive(value: f64, name: &str) -> Result<(), String> {
    if value <= 0.0 {
        return Err(format!("{} must be positive", name));
    }
    Ok(())
}

/// Validate a value is within a range
///
/// # Arguments
/// * `value` - Value to validate
/// * `min` - Minimum allowed value
/// * `max` - Maximum allowed value
/// * `name` - Parameter name for error messages
///
/// # Returns
/// * `Result<(), String>` - Ok or error message
pub fn validate_range(value: f64, min: f64, max: f64, name: &str) -> Result<(), String> {
    if value < min || value > max {
        return Err(format!("{} must be between {} and {}", name, min, max));
    }
    Ok(())
}
