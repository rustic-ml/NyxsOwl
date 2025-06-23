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
            "RSI period must be greater than 0".into(),
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

    for (current, previous) in price_values.iter().zip(price_values.iter().skip(1)) {
        if let (Some(current), Some(previous)) = (current, previous) {
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
    let rs = if avg_loss != 0.0 {
        avg_gain / avg_loss
    } else {
        100.0
    };
    rsi_values[period] = Some(100.0 - (100.0 / (1.0 + rs)));

    // Calculate RSI using EMA-style smoothing for subsequent periods
    let alpha = 1.0 / period as f64;
    for (i, _) in price_values
        .iter()
        .enumerate()
        .take(price_values.len())
        .skip(period + 1)
    {
        let gain_idx = i - 1;
        if gain_idx < gains.len() {
            // Exponential moving average for gains and losses
            avg_gain = alpha * gains[gain_idx] + (1.0 - alpha) * avg_gain;
            avg_loss = alpha * losses[gain_idx] + (1.0 - alpha) * avg_loss;

            let rs = if avg_loss != 0.0 {
                avg_gain / avg_loss
            } else {
                100.0
            };
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
    signal_period: usize,
) -> PolarsResult<(Series, Series, Series)> {
    if fast_period == 0 || slow_period == 0 || signal_period == 0 {
        return Err(PolarsError::InvalidOperation(
            "MACD periods must be greater than 0".into(),
        ));
    }

    if fast_period >= slow_period {
        return Err(PolarsError::InvalidOperation(
            "Fast period must be less than slow period".into(),
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
        histogram.with_name("histogram".into()),
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
    for val in values.iter().take(period).flatten() {
        sum += val;
        count += 1;
    }

    if count == 0 {
        return Ok(Series::new("ema".into(), ema_values));
    }

    let mut ema = sum / count as f64;
    ema_values[period - 1] = Some(ema);

    // Calculate subsequent EMA values
    let multiplier = 2.0 / (period as f64 + 1.0);
    for (i, val) in values.iter().enumerate().take(values.len()).skip(period) {
        if let Some(current) = val {
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
            "Stochastic periods must be greater than 0".into(),
        ));
    }

    let high_values: Vec<Option<f64>> = high.f64()?.into_iter().collect();
    let low_values: Vec<Option<f64>> = low.f64()?.into_iter().collect();
    let close_values: Vec<Option<f64>> = close.f64()?.into_iter().collect();

    if high_values.len() != low_values.len() || high_values.len() != close_values.len() {
        return Err(PolarsError::InvalidOperation(
            "All input series must have the same length".into(),
        ));
    }

    let mut k_values = vec![None; high_values.len()];
    let mut d_values = vec![None; high_values.len()];

    if high_values.len() < k_period {
        return Ok((
            Series::new("k".into(), k_values),
            Series::new("d".into(), d_values),
        ));
    }

    // Calculate %K values
    for i in k_period - 1..high_values.len() {
        let mut highest_high = f64::NEG_INFINITY;
        let mut lowest_low = f64::INFINITY;

        // Find highest high and lowest low in the period
        for j in i.saturating_sub(k_period - 1)..=i {
            if let Some(high_val) = high_values[j] {
                highest_high = highest_high.max(high_val);
            }
            if let Some(low_val) = low_values[j] {
                lowest_low = lowest_low.min(low_val);
            }
        }

        if let Some(close_val) = close_values[i] {
            if highest_high != lowest_low {
                let k_value = ((close_val - lowest_low) / (highest_high - lowest_low)) * 100.0;
                k_values[i] = Some(k_value);
            } else {
                k_values[i] = Some(50.0); // Default when high == low
            }
        }
    }

    // Calculate %D values (SMA of %K)
    if k_values.len() >= d_period {
        for (i, d_value) in d_values
            .iter_mut()
            .enumerate()
            .skip(d_period - 1)
            .take(k_values.len() - d_period + 1)
        {
            let mut sum = 0.0;
            let mut count = 0;

            for k_val in k_values[i.saturating_sub(d_period - 1)..=i]
                .iter()
                .flatten()
            {
                sum += k_val;
                count += 1;
            }

            if count > 0 {
                *d_value = Some(sum / count as f64);
            }
        }
    }

    Ok((
        Series::new("k".into(), k_values),
        Series::new("d".into(), d_values),
    ))
}

/// Calculate Commodity Channel Index (CCI)
///
/// CCI measures the current price level relative to an average price level over a given period.
/// It's used to identify cyclical trends in commodities, equities, and currencies.
///
/// # Arguments
/// * `high` - Series of high prices
/// * `low` - Series of low prices  
/// * `close` - Series of closing prices
/// * `period` - The period for CCI calculation (typically 20)
///
/// # Returns
/// * `PolarsResult<Series>` - Series containing CCI values
pub fn calculate_cci(
    high: &Series,
    low: &Series,
    close: &Series,
    period: usize,
) -> PolarsResult<Series> {
    if period == 0 {
        return Err(PolarsError::InvalidOperation(
            "CCI period must be greater than 0".into(),
        ));
    }

    let high_values: Vec<Option<f64>> = high.f64()?.into_iter().collect();
    let low_values: Vec<Option<f64>> = low.f64()?.into_iter().collect();
    let close_values: Vec<Option<f64>> = close.f64()?.into_iter().collect();

    if high_values.len() != low_values.len() || high_values.len() != close_values.len() {
        return Err(PolarsError::InvalidOperation(
            "All input series must have the same length".into(),
        ));
    }

    let mut cci_values = vec![None; high_values.len()];

    // Calculate CCI for each period
    for i in (period - 1)..high_values.len() {
        let mut typical_prices = Vec::new();
        let mut sum_tp = 0.0;
        let mut count = 0;

        // Calculate typical prices for the period
        for j in i.saturating_sub(period - 1)..=i {
            if let (Some(h), Some(l), Some(c)) = (high_values[j], low_values[j], close_values[j]) {
                let tp = (h + l + c) / 3.0;
                typical_prices.push(tp);
                sum_tp += tp;
                count += 1;
            }
        }

        if count == 0 {
            continue;
        }

        let sma_tp = sum_tp / count as f64;
        let mut sum_deviation = 0.0;

        // Calculate mean deviation
        for tp in &typical_prices {
            sum_deviation += (tp - sma_tp).abs();
        }

        let mean_deviation = sum_deviation / count as f64;

        // Calculate CCI
        if let (Some(h), Some(l), Some(c)) = (high_values[i], low_values[i], close_values[i]) {
            let current_tp = (h + l + c) / 3.0;
            if mean_deviation > 0.0 {
                cci_values[i] = Some((current_tp - sma_tp) / (0.015 * mean_deviation));
            } else {
                cci_values[i] = Some(0.0);
            }
        }
    }

    Ok(Series::new("cci".into(), cci_values))
}

/// Calculate Money Flow Index (MFI)
///
/// MFI is a momentum indicator that measures the inflow and outflow of money into a security.
/// It combines price and volume data to identify overbought or oversold conditions.
///
/// # Arguments
/// * `high` - Series of high prices
/// * `low` - Series of low prices
/// * `close` - Series of closing prices
/// * `volume` - Series of volume data
/// * `period` - The period for MFI calculation (typically 14)
///
/// # Returns
/// * `PolarsResult<Series>` - Series containing MFI values
pub fn calculate_mfi(
    high: &Series,
    low: &Series,
    close: &Series,
    volume: &Series,
    period: usize,
) -> PolarsResult<Series> {
    if period == 0 {
        return Err(PolarsError::InvalidOperation(
            "MFI period must be greater than 0".into(),
        ));
    }

    let high_values: Vec<Option<f64>> = high.f64()?.into_iter().collect();
    let low_values: Vec<Option<f64>> = low.f64()?.into_iter().collect();
    let close_values: Vec<Option<f64>> = close.f64()?.into_iter().collect();
    let volume_values: Vec<Option<f64>> = volume.f64()?.into_iter().collect();

    if high_values.len() != low_values.len()
        || high_values.len() != close_values.len()
        || high_values.len() != volume_values.len()
    {
        return Err(PolarsError::InvalidOperation(
            "All input series must have the same length".into(),
        ));
    }

    let mut mfi_values = vec![None; high_values.len()];

    // Calculate MFI for each period
    for (i, mfi_value) in mfi_values.iter_mut().enumerate().skip(period - 1) {
        let mut positive_money_flow = 0.0;
        let mut negative_money_flow = 0.0;

        // Calculate money flow for the period
        for j in i.saturating_sub(period - 1)..=i {
            if j == 0 {
                continue; // Skip first element as we need previous close
            }

            if let (Some(h), Some(l), Some(c), Some(v), Some(_prev_c)) = (
                high_values[j],
                low_values[j],
                close_values[j],
                volume_values[j],
                close_values[j - 1],
            ) {
                let typical_price = (h + l + c) / 3.0;
                let prev_typical_price = if j > 0 {
                    if let (Some(prev_h), Some(prev_l), Some(prev_close)) =
                        (high_values[j - 1], low_values[j - 1], close_values[j - 1])
                    {
                        (prev_h + prev_l + prev_close) / 3.0
                    } else {
                        continue;
                    }
                } else {
                    continue;
                };

                let raw_money_flow = typical_price * v;

                if typical_price > prev_typical_price {
                    positive_money_flow += raw_money_flow;
                } else if typical_price < prev_typical_price {
                    negative_money_flow += raw_money_flow;
                }
            }
        }

        // Calculate MFI
        let total_money_flow = positive_money_flow + negative_money_flow;
        if total_money_flow > 0.0 {
            let money_ratio = positive_money_flow / negative_money_flow;
            *mfi_value = Some(100.0 - (100.0 / (1.0 + money_ratio)));
        } else {
            *mfi_value = Some(50.0); // Neutral when no money flow
        }
    }

    Ok(Series::new("mfi".into(), mfi_values))
}

/// Calculate Rate of Change (ROC)
///
/// ROC measures the percentage change in price over a specified period.
/// It's used to identify momentum and potential reversal points.
///
/// # Arguments
/// * `series` - Series of price data
/// * `period` - The period for ROC calculation (typically 10 or 14)
///
/// # Returns
/// * `PolarsResult<Series>` - Series containing ROC values
pub fn calculate_roc(series: &Series, period: usize) -> PolarsResult<Series> {
    if period == 0 {
        return Err(PolarsError::InvalidOperation(
            "ROC period must be greater than 0".into(),
        ));
    }

    let values: Vec<Option<f64>> = series.f64()?.into_iter().collect();
    let mut roc_values = vec![None; values.len()];

    // Calculate ROC for each period
    for i in period..values.len() {
        if let (Some(current_price), Some(period_ago_price)) = (values[i], values[i - period]) {
            if period_ago_price != 0.0 {
                roc_values[i] =
                    Some(((current_price - period_ago_price) / period_ago_price) * 100.0);
            } else {
                roc_values[i] = Some(0.0);
            }
        }
    }

    Ok(Series::new("roc".into(), roc_values))
}

/// Calculate Ultimate Oscillator
///
/// The Ultimate Oscillator is a momentum oscillator that uses three different timeframes
/// to avoid the pitfalls of using a single timeframe. It incorporates a weighted average
/// of three oscillators, each using a different period.
///
/// # Formula
/// UO = 100 * ((4 * Average7) + (2 * Average14) + Average28) / (4 + 2 + 1)
/// where AverageN = Sum of Buying Pressure for N periods / Sum of True Range for N periods
/// Buying Pressure = Close - Min(Low, Prior Close)
/// True Range = Max(High, Prior Close) - Min(Low, Prior Close)
///
/// # Arguments
/// * `high` - Series of high prices
/// * `low` - Series of low prices
/// * `close` - Series of closing prices
/// * `short_period` - Short period (default 7)
/// * `medium_period` - Medium period (default 14)
/// * `long_period` - Long period (default 28)
///
/// # Returns
/// * `PolarsResult<Series>` - Series containing Ultimate Oscillator values (0-100)
pub fn calculate_ultimate_oscillator(
    high: &Series,
    low: &Series,
    close: &Series,
    short_period: usize,
    medium_period: usize,
    long_period: usize,
) -> PolarsResult<Series> {
    if short_period == 0 || medium_period == 0 || long_period == 0 {
        return Err(PolarsError::InvalidOperation(
            "All periods must be greater than 0".into(),
        ));
    }

    if short_period >= medium_period || medium_period >= long_period {
        return Err(PolarsError::InvalidOperation(
            "Periods must be in ascending order: short < medium < long".into(),
        ));
    }

    let high_values: Vec<Option<f64>> = high.f64()?.into_iter().collect();
    let low_values: Vec<Option<f64>> = low.f64()?.into_iter().collect();
    let close_values: Vec<Option<f64>> = close.f64()?.into_iter().collect();

    if high_values.len() != low_values.len() || high_values.len() != close_values.len() {
        return Err(PolarsError::InvalidOperation(
            "All input series must have the same length".into(),
        ));
    }

    let mut uo_values = vec![None; high_values.len()];
    let mut prev_close = None;

    // Calculate buying pressure and true range for each period
    let mut buying_pressure = Vec::with_capacity(high_values.len());
    let mut true_range = Vec::with_capacity(high_values.len());

    for i in 0..high_values.len() {
        if let (Some(high), Some(low), Some(close)) =
            (high_values[i], low_values[i], close_values[i])
        {
            let min_low_prev = if let Some(prev) = prev_close {
                low.min(prev)
            } else {
                low
            };

            let max_high_prev = if let Some(prev) = prev_close {
                high.max(prev)
            } else {
                high
            };

            let bp = close - min_low_prev;
            let tr = max_high_prev - min_low_prev;

            buying_pressure.push(Some(bp));
            true_range.push(Some(tr));
            prev_close = Some(close);
        } else {
            buying_pressure.push(None);
            true_range.push(None);
        }
    }

    // Calculate averages for each period
    for (i, uo_value) in uo_values
        .iter_mut()
        .enumerate()
        .skip(long_period - 1)
        .take(buying_pressure.len() - long_period + 1)
    {
        let mut short_bp_sum = 0.0;
        let mut short_tr_sum = 0.0;
        let mut medium_bp_sum = 0.0;
        let mut medium_tr_sum = 0.0;
        let mut long_bp_sum = 0.0;
        let mut long_tr_sum = 0.0;
        let mut valid_count = 0;

        // Calculate sums for all periods
        for (bp, tr) in buying_pressure[i.saturating_sub(long_period - 1)..=i]
            .iter()
            .zip(true_range[i.saturating_sub(long_period - 1)..=i].iter())
        {
            if let (Some(bp_val), Some(tr_val)) = (bp, tr) {
                if i.saturating_sub(short_period - 1) <= valid_count {
                    short_bp_sum += bp_val;
                    short_tr_sum += tr_val;
                }
                if i.saturating_sub(medium_period - 1) <= valid_count {
                    medium_bp_sum += bp_val;
                    medium_tr_sum += tr_val;
                }
                long_bp_sum += bp_val;
                long_tr_sum += tr_val;
                valid_count += 1;
            }
        }

        // Calculate Ultimate Oscillator only if we have enough valid data
        if valid_count == long_period {
            let short_avg = if short_tr_sum != 0.0 {
                short_bp_sum / short_tr_sum
            } else {
                0.0
            };

            let medium_avg = if medium_tr_sum != 0.0 {
                medium_bp_sum / medium_tr_sum
            } else {
                0.0
            };

            let long_avg = if long_tr_sum != 0.0 {
                long_bp_sum / long_tr_sum
            } else {
                0.0
            };

            // Calculate weighted sum (4-2-1 weighting)
            let uo = 100.0 * ((4.0 * short_avg) + (2.0 * medium_avg) + long_avg) / 7.0;
            *uo_value = Some(uo);
        }
    }

    Ok(Series::new("ultimate_oscillator".into(), uo_values))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsi() {
        let data = Series::new(
            "test".into(),
            vec![
                1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
            ],
        );
        let rsi = calculate_rsi(&data, 14).unwrap();

        // Test warmup period
        for i in 0..14 {
            assert!(rsi
                .get(i)
                .unwrap()
                .try_extract::<f64>()
                .unwrap_or(f64::NAN)
                .is_nan());
        }

        // Test valid values
        let rsi_val = rsi.get(14).unwrap().try_extract::<f64>().unwrap();
        assert!(rsi_val.is_finite());

        // Test edge cases
        let constant_data = Series::new("constant".into(), vec![100.0; 20]);
        let constant_rsi = calculate_rsi(&constant_data, 14).unwrap();
        let constant_rsi_val = constant_rsi.get(14).unwrap().try_extract::<f64>().unwrap();
        assert!(constant_rsi_val.is_finite());
    }

    #[test]
    fn test_stochastic() {
        let high = Series::new(
            "high".into(),
            vec![
                110.0, 112.0, 115.0, 113.0, 116.0, 118.0, 117.0, 119.0, 121.0, 120.0, 122.0, 123.0,
                124.0, 125.0, 126.0,
            ],
        );
        let low = Series::new(
            "low".into(),
            vec![
                108.0, 109.0, 111.0, 110.0, 112.0, 114.0, 115.0, 116.0, 118.0, 119.0, 120.0, 121.0,
                122.0, 123.0, 124.0,
            ],
        );
        let close = Series::new(
            "close".into(),
            vec![
                109.0, 111.0, 113.0, 112.0, 115.0, 116.0, 116.5, 118.0, 120.0, 119.5, 121.0, 122.0,
                123.0, 124.0, 125.0,
            ],
        );
        let _volume = Series::new("volume".into(), vec![1000.0; 15]);

        let (k, d) = calculate_stochastic(&high, &low, &close, 14, 3).unwrap();

        // Test warmup period
        for i in 0..13 {
            assert!(k
                .get(i)
                .unwrap()
                .try_extract::<f64>()
                .unwrap_or(f64::NAN)
                .is_nan());
            assert!(d
                .get(i)
                .unwrap()
                .try_extract::<f64>()
                .unwrap_or(f64::NAN)
                .is_nan());
        }

        // Test valid values
        for i in 13..k.len() {
            if let Ok(k_val) = k.get(i).unwrap().try_extract::<f64>() {
                assert!(k_val >= 0.0 && k_val <= 100.0);
            }
        }
    }

    #[test]
    fn test_cci() {
        let high = Series::new(
            "high".into(),
            vec![
                110.0, 112.0, 115.0, 113.0, 116.0, 118.0, 117.0, 119.0, 121.0, 120.0, 122.0, 123.0,
                124.0, 125.0, 126.0, 127.0, 128.0, 129.0, 130.0, 131.0, 132.0,
            ],
        );
        let low = Series::new(
            "low".into(),
            vec![
                108.0, 109.0, 111.0, 110.0, 112.0, 114.0, 115.0, 116.0, 118.0, 119.0, 120.0, 121.0,
                122.0, 123.0, 124.0, 125.0, 126.0, 127.0, 128.0, 129.0, 130.0,
            ],
        );
        let close = Series::new(
            "close".into(),
            vec![
                109.0, 111.0, 113.0, 112.0, 115.0, 116.0, 116.5, 118.0, 120.0, 119.5, 121.0, 122.0,
                123.0, 124.0, 125.0, 126.0, 127.0, 128.0, 129.0, 130.0, 131.0,
            ],
        );

        let cci = calculate_cci(&high, &low, &close, 20).unwrap();

        // Test warmup period
        for i in 0..19 {
            assert!(cci
                .get(i)
                .unwrap()
                .try_extract::<f64>()
                .unwrap_or(f64::NAN)
                .is_nan());
        }

        // Test valid values
        for i in 19..cci.len() {
            if let Ok(cci_val) = cci.get(i).unwrap().try_extract::<f64>() {
                assert!(cci_val.is_finite());
            }
        }
    }

    #[test]
    fn test_mfi() {
        let high = Series::new(
            "high".into(),
            vec![
                110.0, 112.0, 115.0, 113.0, 116.0, 118.0, 117.0, 119.0, 121.0, 120.0, 122.0, 123.0,
                124.0, 125.0, 126.0,
            ],
        );
        let low = Series::new(
            "low".into(),
            vec![
                108.0, 109.0, 111.0, 110.0, 112.0, 114.0, 115.0, 116.0, 118.0, 119.0, 120.0, 121.0,
                122.0, 123.0, 124.0,
            ],
        );
        let close = Series::new(
            "close".into(),
            vec![
                109.0, 111.0, 113.0, 112.0, 115.0, 116.0, 116.5, 118.0, 120.0, 119.5, 121.0, 122.0,
                123.0, 124.0, 125.0,
            ],
        );
        let volume = Series::new("volume".into(), vec![1000.0; 15]);

        let mfi = calculate_mfi(&high, &low, &close, &volume, 14).unwrap();

        // Test warmup period
        for i in 0..13 {
            assert!(mfi
                .get(i)
                .unwrap()
                .try_extract::<f64>()
                .unwrap_or(f64::NAN)
                .is_nan());
        }

        // Test valid values
        for i in 13..mfi.len() {
            if let Ok(mfi_val) = mfi.get(i).unwrap().try_extract::<f64>() {
                assert!(mfi_val >= 0.0 && mfi_val <= 100.0);
            }
        }
    }

    #[test]
    fn test_roc() {
        let prices = Series::new(
            "prices".into(),
            vec![
                100.0, 101.0, 99.0, 102.0, 103.0, 101.5, 104.0, 105.0, 103.5, 106.0, 107.0, 108.0,
                109.0, 110.0, 111.0,
            ],
        );
        let roc = calculate_roc(&prices, 10).unwrap();

        // Test warmup period
        for i in 0..10 {
            assert!(roc
                .get(i)
                .unwrap()
                .try_extract::<f64>()
                .unwrap_or(f64::NAN)
                .is_nan());
        }

        // Test valid values
        for i in 10..roc.len() {
            if let Ok(roc_val) = roc.get(i).unwrap().try_extract::<f64>() {
                assert!(roc_val.is_finite());
            }
        }

        // Test edge case with constant prices
        let constant_prices = Series::new("constant".into(), vec![100.0; 15]);
        let constant_roc = calculate_roc(&constant_prices, 10).unwrap();
        assert_eq!(
            constant_roc.get(10).unwrap().try_extract::<f64>().unwrap(),
            0.0
        );
    }

    #[test]
    fn test_ultimate_oscillator() {
        let high = Series::new(
            "high".into(),
            vec![
                110.0, 112.0, 115.0, 113.0, 116.0, 118.0, 117.0, 119.0, 121.0, 120.0, 122.0, 123.0,
                124.0, 125.0, 126.0, 127.0, 128.0, 129.0, 130.0, 131.0, 132.0, 133.0, 134.0, 135.0,
                136.0, 137.0, 138.0, 139.0, 140.0,
            ],
        );
        let low = Series::new(
            "low".into(),
            vec![
                108.0, 109.0, 111.0, 110.0, 112.0, 114.0, 115.0, 116.0, 118.0, 119.0, 120.0, 121.0,
                122.0, 123.0, 124.0, 125.0, 126.0, 127.0, 128.0, 129.0, 130.0, 131.0, 132.0, 133.0,
                134.0, 135.0, 136.0, 137.0, 138.0,
            ],
        );
        let close = Series::new(
            "close".into(),
            vec![
                109.0, 111.0, 113.0, 112.0, 115.0, 116.0, 116.5, 118.0, 120.0, 119.5, 121.0, 122.0,
                123.0, 124.0, 125.0, 126.0, 127.0, 128.0, 129.0, 130.0, 131.0, 132.0, 133.0, 134.0,
                135.0, 136.0, 137.0, 138.0, 139.0,
            ],
        );

        // Test with default periods (7, 14, 28)
        let uo = calculate_ultimate_oscillator(&high, &low, &close, 7, 14, 28).unwrap();

        // Test warmup period
        for i in 0..27 {
            assert!(uo
                .get(i)
                .unwrap()
                .try_extract::<f64>()
                .unwrap_or(f64::NAN)
                .is_nan());
        }

        // Test valid values
        for i in 27..uo.len() {
            if let Ok(uo_val) = uo.get(i).unwrap().try_extract::<f64>() {
                assert!(uo_val >= 0.0 && uo_val <= 100.0);
            }
        }

        // Test invalid periods
        assert!(calculate_ultimate_oscillator(&high, &low, &close, 0, 14, 28).is_err());
        assert!(calculate_ultimate_oscillator(&high, &low, &close, 7, 0, 28).is_err());
        assert!(calculate_ultimate_oscillator(&high, &low, &close, 7, 14, 0).is_err());
        assert!(calculate_ultimate_oscillator(&high, &low, &close, 14, 7, 28).is_err());
        assert!(calculate_ultimate_oscillator(&high, &low, &close, 7, 28, 14).is_err());

        // Test with constant data
        let constant_high = Series::new("high".into(), vec![100.0; 30]);
        let constant_low = Series::new("low".into(), vec![100.0; 30]);
        let constant_close = Series::new("close".into(), vec![100.0; 30]);

        let constant_uo = calculate_ultimate_oscillator(
            &constant_high,
            &constant_low,
            &constant_close,
            7,
            14,
            28,
        )
        .unwrap();
        for i in 27..constant_uo.len() {
            if let Ok(uo_val) = constant_uo.get(i).unwrap().try_extract::<f64>() {
                assert!(uo_val >= 0.0 && uo_val <= 100.0);
            }
        }
    }

    #[test]
    fn test_edge_cases() {
        // Test with insufficient data
        let short_data = Series::new("short".into(), vec![1.0, 2.0, 3.0]);
        assert!(calculate_rsi(&short_data, 14).is_ok());

        // Test with constant prices
        let constant_price = Series::new("close".into(), vec![100.0; 25]);
        let high = Series::new("high".into(), vec![100.0; 25]);
        let low = Series::new("low".into(), vec![100.0; 25]);

        let cci = calculate_cci(&high, &low, &constant_price, 20).unwrap();
        for i in 19..cci.len() {
            if let Ok(cci_val) = cci.get(i).unwrap().try_extract::<f64>() {
                assert!(cci_val.is_finite());
            }
        }

        // Test MFI with constant data
        let constant_price = Series::new("close".into(), vec![100.0; 15]);
        let high = Series::new("high".into(), vec![100.0; 15]);
        let low = Series::new("low".into(), vec![100.0; 15]);
        let volume = Series::new("volume".into(), vec![1000.0; 15]);

        let mfi = calculate_mfi(&high, &low, &constant_price, &volume, 14).unwrap();
        for i in 13..mfi.len() {
            if let Ok(mfi_val) = mfi.get(i).unwrap().try_extract::<f64>() {
                assert!(mfi_val >= 0.0 && mfi_val <= 100.0);
            }
        }

        // Test ROC with constant prices
        let prices = Series::new("prices".into(), vec![0.0, 1.0, 2.0, 3.0, 4.0]);
        let roc = calculate_roc(&prices, 2).unwrap();
        assert_eq!(roc.get(2).unwrap().try_extract::<f64>().unwrap(), 0.0);

        // Test stochastic with constant data
        let high = Series::new("high".into(), vec![100.0; 15]);
        let low = Series::new("low".into(), vec![100.0; 15]);
        let close = Series::new("close".into(), vec![100.0; 15]);

        let (k, _d) = calculate_stochastic(&high, &low, &close, 14, 3).unwrap();
        for i in 13..k.len() {
            if let Ok(k_val) = k.get(i).unwrap().try_extract::<f64>() {
                assert!(k_val >= 0.0 && k_val <= 100.0);
            }
        }
    }
}
