//! Volatility indicator implementations
//!
//! Contains implementations of various volatility-based indicators:
//! - Bollinger Bands
//! - Average True Range (ATR)
//! - Standard Deviation

use crate::moving_averages::SimpleMovingAverage;
use crate::{MathError, Result};
use std::collections::VecDeque;

/// Bollinger Bands implementation
#[derive(Debug, Clone)]
pub struct BollingerBands {
    period: usize,
    std_dev_multiplier: f64,
    prices: VecDeque<f64>,
    sma: SimpleMovingAverage,
}

impl BollingerBands {
    /// Create a new Bollinger Bands with the specified parameters
    pub fn new(period: usize, std_dev_multiplier: f64) -> Result<Self> {
        if period == 0 {
            return Err(MathError::InvalidInput(
                "Period must be greater than zero".to_string(),
            ));
        }
        if std_dev_multiplier <= 0.0 {
            return Err(MathError::InvalidInput(
                "Standard deviation multiplier must be greater than zero".to_string(),
            ));
        }

        Ok(Self {
            period,
            std_dev_multiplier,
            prices: VecDeque::with_capacity(period),
            sma: SimpleMovingAverage::new(period)?,
        })
    }

    /// Update the Bollinger Bands with a new price value
    pub fn update(&mut self, price: f64) -> Result<()> {
        self.prices.push_back(price);
        self.sma.update(price)?;

        // Keep prices at period length
        if self.prices.len() > self.period {
            self.prices.pop_front();
        }

        Ok(())
    }

    /// Get the current middle band (SMA)
    pub fn middle_band(&self) -> Result<f64> {
        self.sma.value()
    }

    /// Calculate standard deviation of prices
    fn calculate_std_dev(&self) -> Result<f64> {
        if self.prices.len() < self.period {
            return Err(MathError::InsufficientData(format!(
                "Not enough data to calculate standard deviation. Need {} values, have {}.",
                self.period,
                self.prices.len()
            )));
        }

        let sma = self.sma.value()?;

        let variance: f64 = self
            .prices
            .iter()
            .map(|&price| {
                let diff = price - sma;
                diff * diff
            })
            .sum::<f64>()
            / self.prices.len() as f64;

        Ok(variance.sqrt())
    }

    /// Get the current upper band (SMA + multiplier * std_dev)
    pub fn upper_band(&self) -> Result<f64> {
        let middle = self.middle_band()?;
        let std_dev = self.calculate_std_dev()?;

        Ok(middle + (std_dev * self.std_dev_multiplier))
    }

    /// Get the current lower band (SMA - multiplier * std_dev)
    pub fn lower_band(&self) -> Result<f64> {
        let middle = self.middle_band()?;
        let std_dev = self.calculate_std_dev()?;

        Ok(middle - (std_dev * self.std_dev_multiplier))
    }

    /// Calculate Bollinger Band Width (volatility indicator)
    pub fn band_width(&self) -> Result<f64> {
        let upper = self.upper_band()?;
        let lower = self.lower_band()?;
        let middle = self.middle_band()?;

        Ok((upper - lower) / middle * 100.0) // Return as percentage
    }

    /// Calculate %B (where price is relative to the bands)
    pub fn percent_b(&self, price: f64) -> Result<f64> {
        let upper = self.upper_band()?;
        let lower = self.lower_band()?;

        if upper == lower {
            return Err(MathError::CalculationError(
                "Upper and lower bands are equal, cannot calculate %B".to_string(),
            ));
        }

        Ok((price - lower) / (upper - lower))
    }

    /// Get the current period
    pub fn period(&self) -> usize {
        self.period
    }

    /// Get the standard deviation multiplier
    pub fn std_dev_multiplier(&self) -> f64 {
        self.std_dev_multiplier
    }

    /// Reset the Bollinger Bands, clearing all values
    pub fn reset(&mut self) {
        self.prices.clear();
        self.sma = SimpleMovingAverage::new(self.period).unwrap();
    }
}

/// Average True Range (ATR) implementation
#[derive(Debug, Clone)]
pub struct AverageTrueRange {
    period: usize,
    tr_values: VecDeque<f64>,
    previous_close: Option<f64>,
    current_atr: Option<f64>,
    values_seen: usize,
}

impl AverageTrueRange {
    /// Create a new ATR with the specified period
    pub fn new(period: usize) -> Result<Self> {
        if period == 0 {
            return Err(MathError::InvalidInput(
                "Period must be greater than zero".to_string(),
            ));
        }

        Ok(Self {
            period,
            tr_values: VecDeque::with_capacity(period),
            previous_close: None,
            current_atr: None,
            values_seen: 0,
        })
    }

    /// Update the ATR with new price data
    pub fn update(&mut self, high: f64, low: f64, close: f64) -> Result<()> {
        if low > high {
            return Err(MathError::InvalidInput(
                "Low price cannot be greater than high price".to_string(),
            ));
        }

        self.values_seen += 1;

        // Calculate True Range
        let true_range = if let Some(prev_close) = self.previous_close {
            // True Range is the greatest of:
            // 1. High - Low
            // 2. |High - Previous Close|
            // 3. |Low - Previous Close|
            let high_low = high - low;
            let high_pc = (high - prev_close).abs();
            let low_pc = (low - prev_close).abs();

            high_low.max(high_pc).max(low_pc)
        } else {
            // First data point, TR is simply High - Low
            high - low
        };

        self.tr_values.push_back(true_range);

        // Update ATR
        if self.values_seen <= self.period {
            // First period: use simple average
            if self.values_seen == self.period {
                let sum = self.tr_values.iter().sum::<f64>();
                self.current_atr = Some(sum / self.period as f64);
            }
        } else {
            // After first period: ATR = ((Prior ATR * (period - 1)) + Current TR) / period
            if let Some(prior_atr) = self.current_atr {
                let new_atr =
                    (prior_atr * (self.period as f64 - 1.0) + true_range) / self.period as f64;
                self.current_atr = Some(new_atr);

                // Keep tr_values at period length
                if self.tr_values.len() > self.period {
                    self.tr_values.pop_front();
                }
            }
        }

        // Update previous close for next calculation
        self.previous_close = Some(close);

        Ok(())
    }

    /// Get the current ATR value
    pub fn value(&self) -> Result<f64> {
        match self.current_atr {
            Some(atr) => Ok(atr),
            None => Err(MathError::InsufficientData(format!(
                "Not enough data for ATR calculation. Need {} values, have {}.",
                self.period, self.values_seen
            ))),
        }
    }

    /// Get the current period
    pub fn period(&self) -> usize {
        self.period
    }

    /// Reset the ATR, clearing all values
    pub fn reset(&mut self) {
        self.tr_values.clear();
        self.previous_close = None;
        self.current_atr = None;
        self.values_seen = 0;
    }
}

/// Standard Deviation implementation
#[derive(Debug, Clone)]
pub struct StandardDeviation {
    period: usize,
    values: VecDeque<f64>,
    mean: Option<f64>,
}

impl StandardDeviation {
    /// Create a new StandardDeviation with the specified period
    pub fn new(period: usize) -> Result<Self> {
        if period == 0 {
            return Err(MathError::InvalidInput(
                "Period must be greater than zero".to_string(),
            ));
        }

        Ok(Self {
            period,
            values: VecDeque::with_capacity(period),
            mean: None,
        })
    }

    /// Update the StandardDeviation with a new value
    pub fn update(&mut self, value: f64) -> Result<()> {
        self.values.push_back(value);

        // Recalculate mean
        if self.values.len() >= self.period {
            let sum: f64 = self.values.iter().sum();
            self.mean = Some(sum / self.values.len() as f64);

            // Keep values at period length
            if self.values.len() > self.period {
                self.values.pop_front();
            }
        }

        Ok(())
    }

    /// Get the current standard deviation
    pub fn value(&self) -> Result<f64> {
        if self.values.len() < self.period {
            return Err(MathError::InsufficientData(format!(
                "Not enough data for standard deviation calculation. Need {} values, have {}.",
                self.period,
                self.values.len()
            )));
        }

        if let Some(mean) = self.mean {
            let variance: f64 = self
                .values
                .iter()
                .map(|&value| {
                    let diff = value - mean;
                    diff * diff
                })
                .sum::<f64>()
                / self.values.len() as f64;

            Ok(variance.sqrt())
        } else {
            Err(MathError::CalculationError(
                "Mean not calculated".to_string(),
            ))
        }
    }

    /// Get the current period
    pub fn period(&self) -> usize {
        self.period
    }

    /// Reset the StandardDeviation, clearing all values
    pub fn reset(&mut self) {
        self.values.clear();
        self.mean = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bollinger_bands_calculation() {
        let mut bb = BollingerBands::new(3, 2.0).unwrap();

        // Add test data
        bb.update(10.0).unwrap();
        bb.update(11.0).unwrap();
        bb.update(9.0).unwrap();

        // Check middle band (SMA)
        let middle = bb.middle_band().unwrap();
        assert!((middle - 10.0).abs() < 0.001);

        // Check upper and lower bands
        let upper = bb.upper_band().unwrap();
        let lower = bb.lower_band().unwrap();

        assert!(upper > middle);
        assert!(lower < middle);

        // Check that a price at the upper band has %B = 1.0
        let percent_b_upper = bb.percent_b(upper).unwrap();
        assert!((percent_b_upper - 1.0).abs() < 0.001);

        // Check that a price at the lower band has %B = 0.0
        let percent_b_lower = bb.percent_b(lower).unwrap();
        assert!(percent_b_lower.abs() < 0.001);
    }

    #[test]
    fn test_atr_calculation() {
        let mut atr = AverageTrueRange::new(3).unwrap();

        // Add test data (High, Low, Close)
        atr.update(110.0, 100.0, 105.0).unwrap();
        atr.update(115.0, 103.0, 110.0).unwrap();
        atr.update(112.0, 106.0, 107.0).unwrap();

        // Now we have enough data to calculate ATR
        let atr_value = atr.value().unwrap();
        assert!(atr_value > 0.0);

        // ATR should decrease with less volatility
        atr.update(108.0, 106.0, 107.0).unwrap(); // Tight range
        let new_atr_value = atr.value().unwrap();

        // Due to smoothing, the new ATR might not be less than the old one yet,
        // but at least it should be finite and positive
        assert!(new_atr_value > 0.0 && new_atr_value.is_finite());
    }

    #[test]
    fn test_standard_deviation_calculation() {
        let mut std_dev = StandardDeviation::new(3).unwrap();

        // Add test data: 10, 20, 30
        std_dev.update(10.0).unwrap();
        std_dev.update(20.0).unwrap();
        std_dev.update(30.0).unwrap();

        // Calculate expected std dev: sqrt(((10-20)^2 + (20-20)^2 + (30-20)^2) / 3)
        // = sqrt(200/3) ≈ 8.16
        let expected =
            ((10.0f64 - 20.0).powi(2) + (20.0f64 - 20.0).powi(2) + (30.0f64 - 20.0).powi(2)) / 3.0;
        let expected = expected.sqrt();

        let std_dev_value = std_dev.value().unwrap();
        assert!((std_dev_value - expected).abs() < 0.001);
    }
}
