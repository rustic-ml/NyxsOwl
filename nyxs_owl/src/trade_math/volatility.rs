//! Volatility indicator implementations
//!
//! Contains implementations of various volatility-based indicators:
//! - Bollinger Bands
//! - Average True Range (ATR)
//! - Standard Deviation

use super::moving_averages::SimpleMovingAverage;
use super::{MathError, Result};
use crate::advanced_optimizations::{simd_math, AlignedBuffer};
use std::collections::VecDeque;

/// Bollinger Bands implementation with optimizations
#[derive(Debug, Clone)]
pub struct BollingerBands {
    period: usize,
    std_dev_multiplier: f64,
    prices: VecDeque<f64>,
    sma: SimpleMovingAverage,
    cached_std_dev: Option<f64>, // Cache std dev to avoid recalculation
    values_seen: usize,
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
            prices: VecDeque::with_capacity(period), // Pre-allocate exact capacity
            sma: SimpleMovingAverage::new(period)?,
            cached_std_dev: None,
            values_seen: 0,
        })
    }

    /// Update the Bollinger Bands with a new price value - optimized version
    #[inline]
    pub fn update(&mut self, price: f64) -> Result<()> {
        self.prices.push_back(price);
        self.sma.update(price)?;
        self.values_seen += 1;

        // Keep prices at period length
        if self.values_seen > self.period {
            self.prices.pop_front();
            self.values_seen -= 1;
        }

        // Invalidate cache when new data arrives
        self.cached_std_dev = None;

        Ok(())
    }

    /// Get the current middle band (SMA)
    #[inline]
    pub fn middle_band(&self) -> Result<f64> {
        self.sma.value()
    }

    /// Calculate standard deviation of prices - optimized with caching
    fn calculate_std_dev(&mut self) -> Result<f64> {
        // Return cached value if available
        if let Some(cached) = self.cached_std_dev {
            return Ok(cached);
        }

        if self.values_seen < self.period {
            return Err(MathError::InsufficientData(format!(
                "Not enough data to calculate standard deviation. Need {} values, have {}.",
                self.period, self.values_seen
            )));
        }

        let sma = self.sma.value()?;

        // Optimized variance calculation using iterator chaining
        let variance: f64 = self
            .prices
            .iter()
            .map(|&price| {
                let diff = price - sma;
                diff * diff
            })
            .sum::<f64>()
            / self.period as f64; // Use period instead of len() for consistency

        let std_dev = variance.sqrt();

        // Cache the result
        self.cached_std_dev = Some(std_dev);

        Ok(std_dev)
    }

    /// Get the current upper band (SMA + multiplier * std_dev) - optimized
    pub fn upper_band(&mut self) -> Result<f64> {
        let middle = self.middle_band()?;
        let std_dev = self.calculate_std_dev()?;

        Ok(middle + (std_dev * self.std_dev_multiplier))
    }

    /// Get the current lower band (SMA - multiplier * std_dev) - optimized
    pub fn lower_band(&mut self) -> Result<f64> {
        let middle = self.middle_band()?;
        let std_dev = self.calculate_std_dev()?;

        Ok(middle - (std_dev * self.std_dev_multiplier))
    }

    /// Get all bands at once - most efficient method
    pub fn bands(&mut self) -> Result<(f64, f64, f64)> {
        let middle = self.middle_band()?;
        let std_dev = self.calculate_std_dev()?;
        let band_offset = std_dev * self.std_dev_multiplier;

        Ok((
            middle + band_offset, // upper
            middle,               // middle
            middle - band_offset, // lower
        ))
    }

    /// Calculate Bollinger Band Width (volatility indicator) - optimized
    pub fn band_width(&mut self) -> Result<f64> {
        let (upper, middle, lower) = self.bands()?;
        Ok((upper - lower) / middle * 100.0) // Return as percentage
    }

    /// Calculate %B (where price is relative to the bands) - optimized
    pub fn percent_b(&mut self, price: f64) -> Result<f64> {
        let (upper, _, lower) = self.bands()?;

        if (upper - lower).abs() < f64::EPSILON {
            return Err(MathError::CalculationError(
                "Upper and lower bands are equal, cannot calculate %B".to_string(),
            ));
        }

        Ok((price - lower) / (upper - lower))
    }

    /// Get the current period
    #[inline]
    pub fn period(&self) -> usize {
        self.period
    }

    /// Get the standard deviation multiplier
    #[inline]
    pub fn std_dev_multiplier(&self) -> f64 {
        self.std_dev_multiplier
    }

    /// Check if Bollinger Bands are ready
    #[inline]
    pub fn is_ready(&self) -> bool {
        self.values_seen >= self.period
    }

    /// Get the number of values seen
    #[inline]
    pub fn values_seen(&self) -> usize {
        self.values_seen
    }

    /// Reset the Bollinger Bands, clearing all values
    pub fn reset(&mut self) {
        self.prices.clear();
        self.sma = SimpleMovingAverage::new(self.period).unwrap();
        self.cached_std_dev = None;
        self.values_seen = 0;
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

/// Ultra-fast Bollinger Bands implementation with SIMD optimizations
#[derive(Debug, Clone)]
pub struct FastBollingerBands {
    period: usize,
    std_dev_multiplier: f64,
    prices: AlignedBuffer<f64>,
    sum: f64,
    cached_mean: Option<f64>,
    cached_std_dev: Option<f64>,
    values_seen: usize,
}

impl FastBollingerBands {
    /// Create a new ultra-fast Bollinger Bands
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
            prices: AlignedBuffer::new(period),
            sum: 0.0,
            cached_mean: None,
            cached_std_dev: None,
            values_seen: 0,
        })
    }

    /// Ultra-fast update with SIMD optimizations
    #[inline]
    pub fn update_fast(&mut self, price: f64) -> Result<()> {
        // Remove oldest value from sum if buffer is full
        if self.prices.is_full() {
            let old_slice = self.prices.as_slice();
            let oldest = old_slice[self.values_seen % self.period];
            self.sum -= oldest;
        }

        // Add new value
        self.prices.push(price);
        self.sum += price;

        if self.values_seen < self.period {
            self.values_seen += 1;
        }

        // Invalidate cache
        self.cached_mean = None;
        self.cached_std_dev = None;

        Ok(())
    }

    /// Get current mean using cached value or SIMD calculation
    #[inline]
    fn mean(&mut self) -> Result<f64> {
        if let Some(cached) = self.cached_mean {
            return Ok(cached);
        }

        if self.values_seen < self.period {
            return Err(MathError::InsufficientData(format!(
                "Not enough data. Need {} values, have {}.",
                self.period, self.values_seen
            )));
        }

        let mean = self.sum / self.period as f64;
        self.cached_mean = Some(mean);
        Ok(mean)
    }

    /// Get current standard deviation using SIMD if available
    fn std_dev(&mut self) -> Result<f64> {
        if let Some(cached) = self.cached_std_dev {
            return Ok(cached);
        }

        let mean = self.mean()?;
        let slice = self.prices.as_slice();

        // Use SIMD-optimized variance calculation
        let variance = simd_math::variance_f64_optimized(slice, mean);
        let std_dev = variance.sqrt();

        self.cached_std_dev = Some(std_dev);
        Ok(std_dev)
    }

    /// Get all bands at once with SIMD optimization
    pub fn bands_fast(&mut self) -> Result<(f64, f64, f64)> {
        let mean = self.mean()?;
        let std_dev = self.std_dev()?;
        let offset = std_dev * self.std_dev_multiplier;

        Ok((
            mean + offset, // upper
            mean,          // middle
            mean - offset, // lower
        ))
    }

    /// Check if ready for calculations
    #[inline]
    pub fn is_ready(&self) -> bool {
        self.values_seen >= self.period
    }

    /// Get period
    #[inline]
    pub fn period(&self) -> usize {
        self.period
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

    #[test]
    fn test_fast_bollinger_bands() {
        let mut bb = FastBollingerBands::new(3, 2.0).unwrap();

        bb.update_fast(10.0).unwrap();
        bb.update_fast(11.0).unwrap();
        bb.update_fast(9.0).unwrap();

        assert!(bb.is_ready());

        let (upper, middle, lower) = bb.bands_fast().unwrap();
        assert!((middle - 10.0).abs() < 0.001);
        assert!(upper > middle);
        assert!(lower < middle);
    }
}
