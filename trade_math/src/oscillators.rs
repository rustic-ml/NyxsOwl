//! Oscillator indicator implementations
//!
//! Contains implementations of various oscillator indicators:
//! - Relative Strength Index (RSI)
//! - Moving Average Convergence Divergence (MACD)
//! - Stochastic Oscillator

use crate::moving_averages::ExponentialMovingAverage;
use crate::{MathError, Result};
use std::collections::VecDeque;

/// Relative Strength Index (RSI) implementation
#[derive(Debug, Clone)]
pub struct RelativeStrengthIndex {
    period: usize,
    previous_price: Option<f64>,
    gains: VecDeque<f64>,
    losses: VecDeque<f64>,
    avg_gain: Option<f64>,
    avg_loss: Option<f64>,
    values_seen: usize,
}

impl RelativeStrengthIndex {
    /// Create a new RSI with the specified period
    pub fn new(period: usize) -> Result<Self> {
        if period == 0 {
            return Err(MathError::InvalidInput(
                "Period must be greater than zero".to_string(),
            ));
        }

        Ok(Self {
            period,
            previous_price: None,
            gains: VecDeque::with_capacity(period),
            losses: VecDeque::with_capacity(period),
            avg_gain: None,
            avg_loss: None,
            values_seen: 0,
        })
    }

    /// Update the RSI with a new price value
    pub fn update(&mut self, price: f64) -> Result<()> {
        self.values_seen += 1;

        // Calculate change if we have a previous price
        if let Some(prev_price) = self.previous_price {
            let change = price - prev_price;

            // Record gain or loss
            let gain = if change > 0.0 { change } else { 0.0 };
            let loss = if change < 0.0 { -change } else { 0.0 };

            self.gains.push_back(gain);
            self.losses.push_back(loss);

            // If we've accumulated enough data for the initial period
            if self.values_seen > self.period {
                // After first period, we use the smoothed calculation
                if let (Some(avg_gain), Some(avg_loss)) = (self.avg_gain, self.avg_loss) {
                    // Get the new gain and loss values
                    if self.gains.len() > self.period {
                        self.gains.pop_front();
                    }
                    if self.losses.len() > self.period {
                        self.losses.pop_front();
                    }

                    // Calculate smoothed averages
                    // new_avg = (prev_avg * (period - 1) + current_value) / period
                    let new_avg_gain =
                        (avg_gain * (self.period as f64 - 1.0) + gain) / self.period as f64;
                    let new_avg_loss =
                        (avg_loss * (self.period as f64 - 1.0) + loss) / self.period as f64;

                    self.avg_gain = Some(new_avg_gain);
                    self.avg_loss = Some(new_avg_loss);
                }
            } else if self.values_seen == self.period {
                // Initialize the first averages as simple average
                let avg_gain = self.gains.iter().sum::<f64>() / self.period as f64;
                let avg_loss = self.losses.iter().sum::<f64>() / self.period as f64;

                self.avg_gain = Some(avg_gain);
                self.avg_loss = Some(avg_loss);
            }
        }

        // Update previous price for next calculation
        self.previous_price = Some(price);

        Ok(())
    }

    /// Get the current RSI value (0-100)
    pub fn value(&self) -> Result<f64> {
        if self.values_seen <= self.period {
            return Err(MathError::InsufficientData(format!(
                "Not enough data for RSI calculation. Need {} values, have {}.",
                self.period + 1,
                self.values_seen
            )));
        }

        match (self.avg_gain, self.avg_loss) {
            (Some(avg_gain), Some(avg_loss)) => {
                if avg_loss == 0.0 {
                    return Ok(100.0); // If no losses, RSI is 100
                }

                let rs = avg_gain / avg_loss;
                let rsi = 100.0 - (100.0 / (1.0 + rs));

                Ok(rsi)
            }
            _ => Err(MathError::CalculationError(
                "RSI averages not calculated".to_string(),
            )),
        }
    }

    /// Get the current period
    pub fn period(&self) -> usize {
        self.period
    }

    /// Reset the RSI, clearing all values
    pub fn reset(&mut self) {
        self.previous_price = None;
        self.gains.clear();
        self.losses.clear();
        self.avg_gain = None;
        self.avg_loss = None;
        self.values_seen = 0;
    }
}

/// Moving Average Convergence Divergence (MACD) implementation
#[derive(Debug, Clone)]
pub struct Macd {
    fast_ema: ExponentialMovingAverage,
    slow_ema: ExponentialMovingAverage,
    signal_ema: ExponentialMovingAverage,
    macd_values: VecDeque<f64>, // Store MACD line values for signal line calculation
    values_seen: usize,
    signal_period: usize,
}

impl Macd {
    /// Create a new MACD with the specified parameters
    pub fn new(fast_period: usize, slow_period: usize, signal_period: usize) -> Result<Self> {
        if fast_period >= slow_period {
            return Err(MathError::InvalidInput(
                "Fast period must be smaller than slow period".to_string(),
            ));
        }

        if signal_period == 0 {
            return Err(MathError::InvalidInput(
                "Signal period must be greater than zero".to_string(),
            ));
        }

        Ok(Self {
            fast_ema: ExponentialMovingAverage::new(fast_period)?,
            slow_ema: ExponentialMovingAverage::new(slow_period)?,
            signal_ema: ExponentialMovingAverage::new(signal_period)?,
            macd_values: VecDeque::with_capacity(signal_period),
            values_seen: 0,
            signal_period,
        })
    }

    /// Update the MACD with a new price value
    pub fn update(&mut self, price: f64) -> Result<()> {
        self.values_seen += 1;

        // Update EMAs
        self.fast_ema.update(price)?;
        self.slow_ema.update(price)?;

        // If we have enough data to calculate both EMAs
        if let (Ok(fast_value), Ok(slow_value)) = (self.fast_ema.value(), self.slow_ema.value()) {
            // Calculate MACD line (fast EMA - slow EMA)
            let macd_value = fast_value - slow_value;

            // Store MACD value for signal line calculation
            self.macd_values.push_back(macd_value);

            // Update signal line EMA with the new MACD value
            self.signal_ema.update(macd_value)?;

            // Keep macd_values at signal_period length
            if self.macd_values.len() > self.signal_period {
                self.macd_values.pop_front();
            }
        }

        Ok(())
    }

    /// Get the current MACD line value (fast EMA - slow EMA)
    pub fn macd_value(&self) -> Result<f64> {
        match (self.fast_ema.value(), self.slow_ema.value()) {
            (Ok(fast), Ok(slow)) => Ok(fast - slow),
            _ => Err(MathError::InsufficientData(
                "Not enough data to calculate MACD line".to_string(),
            )),
        }
    }

    /// Get the current signal line value (EMA of MACD)
    pub fn signal_value(&self) -> Result<f64> {
        self.signal_ema.value().map_err(|_| {
            MathError::InsufficientData("Not enough data to calculate signal line".to_string())
        })
    }

    /// Get the current histogram value (MACD line - signal line)
    pub fn histogram(&self) -> Result<f64> {
        match (self.macd_value(), self.signal_value()) {
            (Ok(macd), Ok(signal)) => Ok(macd - signal),
            _ => Err(MathError::InsufficientData(
                "Not enough data to calculate histogram".to_string(),
            )),
        }
    }

    /// Get the fast period
    pub fn fast_period(&self) -> usize {
        self.fast_ema.period()
    }

    /// Get the slow period
    pub fn slow_period(&self) -> usize {
        self.slow_ema.period()
    }

    /// Get the signal period
    pub fn signal_period(&self) -> usize {
        self.signal_period
    }

    /// Reset the MACD, clearing all values
    pub fn reset(&mut self) {
        self.fast_ema.reset();
        self.slow_ema.reset();
        self.signal_ema.reset();
        self.macd_values.clear();
        self.values_seen = 0;
    }
}

/// Stochastic Oscillator implementation
#[derive(Debug, Clone)]
pub struct StochasticOscillator {
    k_period: usize,                   // Look-back period for %K
    d_period: usize,                   // Smoothing period for %D
    prices: VecDeque<(f64, f64, f64)>, // (high, low, close)
    k_values: VecDeque<f64>,           // Store %K values for %D calculation
    values_seen: usize,
}

impl StochasticOscillator {
    /// Create a new Stochastic Oscillator with the specified parameters
    pub fn new(k_period: usize, d_period: usize) -> Result<Self> {
        if k_period == 0 || d_period == 0 {
            return Err(MathError::InvalidInput(
                "K and D periods must be greater than zero".to_string(),
            ));
        }

        Ok(Self {
            k_period,
            d_period,
            prices: VecDeque::with_capacity(k_period),
            k_values: VecDeque::with_capacity(d_period),
            values_seen: 0,
        })
    }

    /// Update the Stochastic Oscillator with new price data
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<()> {
        if low > high {
            return Err(MathError::InvalidInput(
                "Low price cannot be greater than high price".to_string(),
            ));
        }

        self.values_seen += 1;

        // Add new price data
        self.prices.push_back((high, low, close));

        // Keep prices at k_period length
        if self.prices.len() > self.k_period {
            self.prices.pop_front();
        }

        // If we have enough data to calculate %K
        if self.prices.len() == self.k_period {
            // Find highest high and lowest low in the period
            let highest_high = self
                .prices
                .iter()
                .map(|&(h, _, _)| h)
                .fold(f64::NEG_INFINITY, f64::max);
            let lowest_low = self
                .prices
                .iter()
                .map(|&(_, l, _)| l)
                .fold(f64::INFINITY, f64::min);

            // Calculate %K = (Current Close - Lowest Low) / (Highest High - Lowest Low) * 100
            let k_value = if highest_high == lowest_low {
                50.0 // If flat price action, use middle value
            } else {
                let current_close = close;
                (current_close - lowest_low) / (highest_high - lowest_low) * 100.0
            };

            // Store %K value for %D calculation
            self.k_values.push_back(k_value);

            // Keep k_values at d_period length
            if self.k_values.len() > self.d_period {
                self.k_values.pop_front();
            }
        }

        Ok(())
    }

    /// Get the current %K value (0-100)
    pub fn k_value(&self) -> Result<f64> {
        if self.prices.len() < self.k_period {
            return Err(MathError::InsufficientData(format!(
                "Not enough data for %%K calculation. Need {} values, have {}.",
                self.k_period,
                self.prices.len()
            )));
        }

        if let Some(&k) = self.k_values.back() {
            Ok(k)
        } else {
            Err(MathError::CalculationError(
                "%K value not calculated".to_string(),
            ))
        }
    }

    /// Get the current %D value (0-100, simple moving average of %K)
    pub fn d_value(&self) -> Result<f64> {
        if self.k_values.len() < self.d_period {
            return Err(MathError::InsufficientData(format!(
                "Not enough data for %%D calculation. Need {} %%K values, have {}.",
                self.d_period,
                self.k_values.len()
            )));
        }

        let sum = self.k_values.iter().sum::<f64>();
        Ok(sum / self.d_period as f64)
    }

    /// Get the K period
    pub fn k_period(&self) -> usize {
        self.k_period
    }

    /// Get the D period
    pub fn d_period(&self) -> usize {
        self.d_period
    }

    /// Reset the Stochastic Oscillator, clearing all values
    pub fn reset(&mut self) {
        self.prices.clear();
        self.k_values.clear();
        self.values_seen = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsi_calculation() {
        let mut rsi = RelativeStrengthIndex::new(3).unwrap();

        // Add some test data: 10.0, 10.5, 11.0, 10.5, 10.0
        rsi.update(10.0).unwrap();
        rsi.update(10.5).unwrap();
        rsi.update(11.0).unwrap();
        rsi.update(10.5).unwrap();

        // Now we have enough data to calculate RSI
        let rsi_value = rsi.value().unwrap();
        assert!(rsi_value >= 0.0 && rsi_value <= 100.0);

        // Test downtrend: should produce lower RSI
        rsi.update(10.0).unwrap();
        let new_rsi_value = rsi.value().unwrap();
        assert!(new_rsi_value < rsi_value);
    }

    #[test]
    fn test_macd_calculation() {
        let mut macd = Macd::new(3, 6, 2).unwrap();

        // Add some test data with a clear trend
        for i in 0..10 {
            let price = 100.0 + i as f64 * 2.0;
            macd.update(price).unwrap();
        }

        // Check that we can calculate MACD values
        let macd_value = macd.macd_value().unwrap();
        assert!(macd_value > 0.0); // In an uptrend, MACD should be positive

        // Signal line needs more data points
        if let Ok(signal_value) = macd.signal_value() {
            let histogram = macd.histogram().unwrap();
            assert_eq!(histogram, macd_value - signal_value);
        }
    }

    #[test]
    fn test_stochastic_calculation() {
        let mut stochastic = StochasticOscillator::new(3, 2).unwrap();

        // Add some test data
        // High, Low, Close
        stochastic.update(110.0, 100.0, 105.0).unwrap();
        stochastic.update(115.0, 105.0, 110.0).unwrap();
        stochastic.update(120.0, 110.0, 115.0).unwrap();

        // Now we have enough data for %K
        let k_value = stochastic.k_value().unwrap();
        assert!(k_value >= 0.0 && k_value <= 100.0);

        // Add one more for %D
        stochastic.update(125.0, 115.0, 120.0).unwrap();

        // Now we should have enough for %D
        let d_value = stochastic.d_value().unwrap();
        assert!(d_value >= 0.0 && d_value <= 100.0);
    }
}
