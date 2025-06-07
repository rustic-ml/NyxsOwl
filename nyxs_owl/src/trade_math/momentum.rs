#![allow(clippy::too_many_arguments)] // Common for TA functions

use polars::prelude::*;

/// Calculate RSI (Relative Strength Index)
/// 
/// RSI is a momentum oscillator that measures the speed and magnitude of price changes.
/// It oscillates between 0 and 100, with values above 70 typically indicating overbought conditions
/// and values below 30 indicating oversold conditions.
///
/// # Arguments
/// * `prices` - Series of price data (typically closing prices)
/// * `period` - The period for RSI calculation (typically 14)
///
/// # Returns
/// * `PolarsResult<Series>` - Series containing RSI values (first `period` values will be null)
pub fn calculate_rsi(prices: &Series, period: usize) -> PolarsResult<Series> {
    if period == 0 {
        return Err(PolarsError::InvalidOperation(
            "RSI period must be greater than 0".into()
        ));
    }

    let price_values: Vec<Option<f64>> = prices.f64()?.into_iter().collect();
    let mut rsi_values = vec![None; price_values.len()];

    if price_values.len() < period + 1 {
        return Ok(Series::new("rsi".into(), rsi_values));
    }

    // Calculate price changes
    let mut gains = Vec::new();
    let mut losses = Vec::new();

    for i in 1..price_values.len() {
        if let (Some(current), Some(previous)) = (price_values[i], price_values[i-1]) {
            let change = current - previous;
            if change > 0.0 {
                gains.push(change);
                losses.push(0.0);
            } else {
                gains.push(0.0);
                losses.push(-change);
            }
        } else {
            gains.push(0.0);
            losses.push(0.0);
        }
    }

    if gains.len() < period {
        return Ok(Series::new("rsi".into(), rsi_values));
    }

    // Calculate initial average gain and loss (SMA for first calculation)
    let mut avg_gain = gains[..period].iter().sum::<f64>() / period as f64;
    let mut avg_loss = losses[..period].iter().sum::<f64>() / period as f64;

    // Calculate RSI for the first valid period
    let rs = if avg_loss != 0.0 { avg_gain / avg_loss } else { 100.0 };
    rsi_values[period] = Some(100.0 - (100.0 / (1.0 + rs)));

    // Calculate RSI using EMA-style smoothing for subsequent periods
    let alpha = 1.0 / period as f64;
    for i in (period + 1)..price_values.len() {
        let gain_idx = i - 1;
        if gain_idx < gains.len() {
            // Exponential moving average for gains and losses
            avg_gain = alpha * gains[gain_idx] + (1.0 - alpha) * avg_gain;
            avg_loss = alpha * losses[gain_idx] + (1.0 - alpha) * avg_loss;

            let rs = if avg_loss != 0.0 { avg_gain / avg_loss } else { 100.0 };
            rsi_values[i] = Some(100.0 - (100.0 / (1.0 + rs)));
        }
    }

    Ok(Series::new("rsi".into(), rsi_values))
}

/// Calculate MACD (Moving Average Convergence Divergence)
///
/// MACD is a trend-following momentum indicator that shows the relationship
/// between two moving averages of a security's price.
///
/// # Arguments
/// * `prices` - Series of price data (typically closing prices)
/// * `fast_period` - Period for fast EMA (typically 12)
/// * `slow_period` - Period for slow EMA (typically 26)
/// * `signal_period` - Period for signal line EMA (typically 9)
///
/// # Returns
/// * `PolarsResult<(Series, Series, Series)>` - (MACD line, Signal line, Histogram)
pub fn calculate_macd(
    prices: &Series, 
    fast_period: usize, 
    slow_period: usize, 
    signal_period: usize
) -> PolarsResult<(Series, Series, Series)> {
    if fast_period == 0 || slow_period == 0 || signal_period == 0 {
        return Err(PolarsError::InvalidOperation(
            "MACD periods must be greater than 0".into()
        ));
    }

    if fast_period >= slow_period {
        return Err(PolarsError::InvalidOperation(
            "Fast period must be less than slow period".into()
        ));
    }

    // Calculate fast and slow EMAs
    let fast_ema = calculate_ema_internal(prices, fast_period)?;
    let slow_ema = calculate_ema_internal(prices, slow_period)?;

    // Calculate MACD line (fast EMA - slow EMA)
    let macd_line = (&fast_ema - &slow_ema)?;

    // Calculate signal line (EMA of MACD line)
    let signal_line = calculate_ema_internal(&macd_line, signal_period)?;

    // Calculate histogram (MACD line - signal line)
    let histogram = (&macd_line - &signal_line)?;

    Ok((
        macd_line.with_name("macd".into()),
        signal_line.with_name("signal".into()),
        histogram.with_name("histogram".into())
    ))
}

/// Internal EMA calculation helper
fn calculate_ema_internal(series: &Series, period: usize) -> PolarsResult<Series> {
    let values: Vec<Option<f64>> = series.f64()?.into_iter().collect();
    let mut ema_values = vec![None; values.len()];

    if values.len() < period {
        return Ok(Series::new("ema".into(), ema_values));
    }

    // Calculate SMA for the first EMA value
    let mut sum = 0.0;
    let mut count = 0;
    for i in 0..period {
        if let Some(val) = values[i] {
            sum += val;
            count += 1;
        }
    }

    if count == 0 {
        return Ok(Series::new("ema".into(), ema_values));
    }

    let mut ema = sum / count as f64;
    ema_values[period - 1] = Some(ema);

    // Calculate subsequent EMA values
    let multiplier = 2.0 / (period as f64 + 1.0);
    for i in period..values.len() {
        if let Some(current) = values[i] {
            ema = (current * multiplier) + (ema * (1.0 - multiplier));
            ema_values[i] = Some(ema);
        }
    }

    Ok(Series::new("ema".into(), ema_values))
}

/// Calculate Stochastic Oscillator
///
/// The Stochastic Oscillator is a momentum indicator that uses support and resistance levels.
/// It gives the location of the close relative to the high-low range over a set number of periods.
///
/// # Arguments
/// * `high` - Series of high prices
/// * `low` - Series of low prices  
/// * `close` - Series of closing prices
/// * `k_period` - Period for %K calculation (typically 14)
/// * `d_period` - Period for %D smoothing (typically 3)
///
/// # Returns
/// * `PolarsResult<(Series, Series)>` - (%K line, %D line)
pub fn calculate_stochastic(
    high: &Series,
    low: &Series,
    close: &Series,
    k_period: usize,
    d_period: usize,
) -> PolarsResult<(Series, Series)> {
    if k_period == 0 || d_period == 0 {
        return Err(PolarsError::InvalidOperation(
            "Stochastic periods must be greater than 0".into()
        ));
    }

    let high_values: Vec<Option<f64>> = high.f64()?.into_iter().collect();
    let low_values: Vec<Option<f64>> = low.f64()?.into_iter().collect();
    let close_values: Vec<Option<f64>> = close.f64()?.into_iter().collect();

    let len = high_values.len().min(low_values.len()).min(close_values.len());
    let mut k_values = vec![None; len];

    // Calculate %K
    for i in k_period - 1..len {
        let start_idx = i + 1 - k_period;
        
        let mut highest_high = f64::NEG_INFINITY;
        let mut lowest_low = f64::INFINITY;
        let mut valid_data = true;

        // Find highest high and lowest low in the period
        for j in start_idx..=i {
            if let (Some(h), Some(l)) = (high_values[j], low_values[j]) {
                highest_high = highest_high.max(h);
                lowest_low = lowest_low.min(l);
            } else {
                valid_data = false;
                break;
            }
        }

        if valid_data {
            if let Some(current_close) = close_values[i] {
                let range = highest_high - lowest_low;
                if range != 0.0 {
                    let k = ((current_close - lowest_low) / range) * 100.0;
                    k_values[i] = Some(k);
                }
            }
        }
    }

    // Calculate %D (SMA of %K)
    let mut d_values = vec![None; len];
    for i in (k_period - 1 + d_period - 1)..len {
        let start_idx = i + 1 - d_period;
        let mut sum = 0.0;
        let mut count = 0;

        for j in start_idx..=i {
            if let Some(k_val) = k_values[j] {
                sum += k_val;
                count += 1;
            }
        }

        if count == d_period {
            d_values[i] = Some(sum / count as f64);
        }
    }

    Ok((
        Series::new("stoch_k".into(), k_values),
        Series::new("stoch_d".into(), d_values)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    fn create_test_series() -> Series {
        Series::new("close".into(), vec![
            44.0, 44.25, 44.5, 43.75, 44.5, 44.75, 44.0, 44.25,
            44.75, 44.5, 44.0, 44.25, 45.0, 45.25, 45.5
        ])
    }

    #[test]
    fn test_rsi_calculation() {
        let prices = create_test_series();
        let rsi = calculate_rsi(&prices, 14).unwrap();
        
        // Verify length
        assert_eq!(rsi.len(), prices.len());
        
        // Verify first 14 values are null
        let rsi_values: Vec<Option<f64>> = rsi.f64().unwrap().into_iter().collect();
        for i in 0..14 {
            assert!(rsi_values[i].is_none());
        }
        
        // Verify RSI is between 0 and 100
        for i in 14..rsi_values.len() {
            if let Some(rsi_val) = rsi_values[i] {
                assert!(rsi_val >= 0.0 && rsi_val <= 100.0);
            }
        }
    }

    #[test]
    fn test_macd_calculation() {
        let prices = create_test_series();
        let (macd, signal, histogram) = calculate_macd(&prices, 5, 10, 3).unwrap();
        
        // Verify lengths
        assert_eq!(macd.len(), prices.len());
        assert_eq!(signal.len(), prices.len());
        assert_eq!(histogram.len(), prices.len());
        
        // Verify names
        assert_eq!(macd.name(), "macd");
        assert_eq!(signal.name(), "signal");
        assert_eq!(histogram.name(), "histogram");
    }

    #[test]
    fn test_stochastic_calculation() {
        let high = Series::new("high".into(), vec![45.0, 45.5, 46.0, 45.75, 46.25]);
        let low = Series::new("low".into(), vec![44.0, 44.25, 44.5, 44.0, 44.75]);
        let close = Series::new("close".into(), vec![44.5, 45.0, 45.5, 44.5, 45.5]);
        
        let (k, d) = calculate_stochastic(&high, &low, &close, 3, 2).unwrap();
        
        // Verify lengths
        assert_eq!(k.len(), 5);
        assert_eq!(d.len(), 5);
        
        // Verify names
        assert_eq!(k.name(), "stoch_k");
        assert_eq!(d.name(), "stoch_d");
    }

    #[test]
    fn test_rsi_invalid_period() {
        let prices = create_test_series();
        let result = calculate_rsi(&prices, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_macd_invalid_periods() {
        let prices = create_test_series();
        
        // Fast period >= slow period
        let result = calculate_macd(&prices, 12, 12, 9);
        assert!(result.is_err());
        
        // Zero periods
        let result = calculate_macd(&prices, 0, 26, 9);
        assert!(result.is_err());
    }
} 