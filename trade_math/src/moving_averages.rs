//! Moving average calculation implementations
//!
//! Contains implementations of various moving average types:
//! - Simple Moving Average (SMA)
//! - Exponential Moving Average (EMA)
//! - Volume-Weighted Moving Average (VWMA)

use crate::{MathError, Result};
use std::collections::VecDeque;

/// Simple Moving Average (SMA) implementation
#[derive(Debug, Clone)]
pub struct SimpleMovingAverage {
    period: usize,
    values: VecDeque<f64>,
    sum: f64,
}

impl SimpleMovingAverage {
    /// Create a new Simple Moving Average with the specified period
    pub fn new(period: usize) -> Result<Self> {
        if period == 0 {
            return Err(MathError::InvalidInput(
                "Period must be greater than zero".to_string(),
            ));
        }

        Ok(Self {
            period,
            values: VecDeque::with_capacity(period),
            sum: 0.0,
        })
    }

    /// Update the SMA with a new value
    pub fn update(&mut self, value: f64) -> Result<()> {
        // Add new value
        self.values.push_back(value);
        self.sum += value;

        // Remove oldest value if we have more than period values
        if self.values.len() > self.period {
            if let Some(old_value) = self.values.pop_front() {
                self.sum -= old_value;
            }
        }

        Ok(())
    }

    /// Get the current SMA value
    pub fn value(&self) -> Result<f64> {
        if self.values.len() < self.period {
            return Err(MathError::InsufficientData(format!(
                "Not enough data for SMA calculation. Need {} values, have {}.",
                self.period,
                self.values.len()
            )));
        }

        Ok(self.sum / self.period as f64)
    }

    /// Get the current period
    pub fn period(&self) -> usize {
        self.period
    }

    /// Reset the SMA, clearing all values
    pub fn reset(&mut self) {
        self.values.clear();
        self.sum = 0.0;
    }
}

/// Exponential Moving Average (EMA) implementation
#[derive(Debug, Clone)]
pub struct ExponentialMovingAverage {
    period: usize,
    multiplier: f64,
    current_ema: Option<f64>,
    values_seen: usize,
}

impl ExponentialMovingAverage {
    /// Create a new Exponential Moving Average with the specified period
    pub fn new(period: usize) -> Result<Self> {
        if period == 0 {
            return Err(MathError::InvalidInput(
                "Period must be greater than zero".to_string(),
            ));
        }

        let multiplier = 2.0 / (period as f64 + 1.0);

        Ok(Self {
            period,
            multiplier,
            current_ema: None,
            values_seen: 0,
        })
    }

    /// Update the EMA with a new value
    pub fn update(&mut self, value: f64) -> Result<()> {
        self.values_seen += 1;

        match self.current_ema {
            // If this is the first value, or we haven't seen enough values for a full period,
            // we're in the initialization phase
            None if self.values_seen < self.period => {
                // For the first period values, we'll use SMA as the initial EMA
                if self.values_seen == self.period {
                    // Initialize with this value (equivalent to SMA of first period values)
                    self.current_ema = Some(value);
                }
            }
            // Otherwise, we calculate the EMA normally
            None => {
                self.current_ema = Some(value);
            }
            Some(current) => {
                // EMA = (Close - EMA(previous)) * multiplier + EMA(previous)
                let new_ema = (value - current) * self.multiplier + current;
                self.current_ema = Some(new_ema);
            }
        }

        Ok(())
    }

    /// Get the current EMA value
    pub fn value(&self) -> Result<f64> {
        match self.current_ema {
            Some(ema) => Ok(ema),
            None => Err(MathError::InsufficientData(format!(
                "Not enough data for EMA calculation. Need at least {} values.",
                self.period
            ))),
        }
    }

    /// Get the current period
    pub fn period(&self) -> usize {
        self.period
    }

    /// Reset the EMA, clearing all values
    pub fn reset(&mut self) {
        self.current_ema = None;
        self.values_seen = 0;
    }
}

/// Volume-Weighted Moving Average (VWMA) implementation
#[derive(Debug, Clone)]
pub struct VolumeWeightedMovingAverage {
    period: usize,
    price_volume_products: VecDeque<f64>,
    volumes: VecDeque<f64>,
    sum_price_volume: f64,
    sum_volume: f64,
}

impl VolumeWeightedMovingAverage {
    /// Create a new Volume-Weighted Moving Average with the specified period
    pub fn new(period: usize) -> Result<Self> {
        if period == 0 {
            return Err(MathError::InvalidInput(
                "Period must be greater than zero".to_string(),
            ));
        }

        Ok(Self {
            period,
            price_volume_products: VecDeque::with_capacity(period),
            volumes: VecDeque::with_capacity(period),
            sum_price_volume: 0.0,
            sum_volume: 0.0,
        })
    }

    /// Update the VWMA with a new price and volume
    pub fn update(&mut self, price: f64, volume: f64) -> Result<()> {
        if volume < 0.0 {
            return Err(MathError::InvalidInput(
                "Volume cannot be negative".to_string(),
            ));
        }

        let price_volume = price * volume;

        // Add new values
        self.price_volume_products.push_back(price_volume);
        self.volumes.push_back(volume);

        self.sum_price_volume += price_volume;
        self.sum_volume += volume;

        // Remove oldest values if we have more than period values
        if self.price_volume_products.len() > self.period {
            if let Some(old_price_volume) = self.price_volume_products.pop_front() {
                self.sum_price_volume -= old_price_volume;
            }

            if let Some(old_volume) = self.volumes.pop_front() {
                self.sum_volume -= old_volume;
            }
        }

        Ok(())
    }

    /// Get the current VWMA value
    pub fn value(&self) -> Result<f64> {
        if self.price_volume_products.len() < self.period {
            return Err(MathError::InsufficientData(format!(
                "Not enough data for VWMA calculation. Need {} values, have {}.",
                self.period,
                self.price_volume_products.len()
            )));
        }

        if self.sum_volume == 0.0 {
            return Err(MathError::CalculationError(
                "Volume sum is zero, cannot calculate VWMA".to_string(),
            ));
        }

        Ok(self.sum_price_volume / self.sum_volume)
    }

    /// Get the current period
    pub fn period(&self) -> usize {
        self.period
    }

    /// Reset the VWMA, clearing all values
    pub fn reset(&mut self) {
        self.price_volume_products.clear();
        self.volumes.clear();
        self.sum_price_volume = 0.0;
        self.sum_volume = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma_calculation() {
        let mut sma = SimpleMovingAverage::new(3).unwrap();

        // Not enough data yet
        assert!(sma.value().is_err());

        sma.update(2.0).unwrap();
        sma.update(4.0).unwrap();

        // Still not enough data
        assert!(sma.value().is_err());

        sma.update(6.0).unwrap();

        // Now we have enough data
        assert_eq!(sma.value().unwrap(), 4.0); // (2 + 4 + 6) / 3 = 4

        // Add another value
        sma.update(8.0).unwrap();

        // The window slides, dropping the oldest value
        assert_eq!(sma.value().unwrap(), 6.0); // (4 + 6 + 8) / 3 = 6
    }

    #[test]
    fn test_ema_calculation() {
        let mut ema = ExponentialMovingAverage::new(3).unwrap();

        // Not enough data yet
        assert!(ema.value().is_err());

        ema.update(2.0).unwrap();
        ema.update(4.0).unwrap();
        ema.update(6.0).unwrap();

        // Now we have enough data
        let first_ema = ema.value().unwrap();
        assert!((3.9..=4.1).contains(&first_ema)); // Approximately 4.0

        // Add another value
        ema.update(8.0).unwrap();

        // The EMA should be updated
        let second_ema = ema.value().unwrap();
        assert!(second_ema > first_ema); // Should be higher than before
    }

    #[test]
    fn test_vwma_calculation() {
        let mut vwma = VolumeWeightedMovingAverage::new(2).unwrap();

        // Not enough data yet
        assert!(vwma.value().is_err());

        vwma.update(10.0, 100.0).unwrap(); // price = 10, volume = 100
        vwma.update(20.0, 200.0).unwrap(); // price = 20, volume = 200

        // Now we have enough data
        // VWMA = (10*100 + 20*200) / (100 + 200) = 16.67
        let expected = (10.0 * 100.0 + 20.0 * 200.0) / (100.0 + 200.0);
        assert!((vwma.value().unwrap() - expected).abs() < 0.001);

        // Add another value
        vwma.update(15.0, 300.0).unwrap(); // price = 15, volume = 300

        // The window slides, dropping the oldest value
        // VWMA = (20*200 + 15*300) / (200 + 300) = 17.0
        let expected = (20.0 * 200.0 + 15.0 * 300.0) / (200.0 + 300.0);
        assert!((vwma.value().unwrap() - expected).abs() < 0.001);
    }
}
