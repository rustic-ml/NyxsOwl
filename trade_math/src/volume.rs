//! Volume indicator implementations
//!
//! Contains implementations of various volume-based indicators:
//! - Volume Moving Average
//! - On-Balance Volume (OBV)
//! - Volume Rate of Change

use crate::{MathError, Result};
use std::collections::VecDeque;

/// Volume Moving Average (VMA) implementation
#[derive(Debug, Clone)]
pub struct VolumeMovingAverage {
    period: usize,
    volumes: VecDeque<f64>,
    sum: f64,
}

impl VolumeMovingAverage {
    /// Create a new Volume Moving Average with the specified period
    pub fn new(period: usize) -> Result<Self> {
        if period == 0 {
            return Err(MathError::InvalidInput(
                "Period must be greater than zero".to_string(),
            ));
        }

        Ok(Self {
            period,
            volumes: VecDeque::with_capacity(period),
            sum: 0.0,
        })
    }

    /// Update the VMA with a new volume value
    pub fn update(&mut self, volume: f64) -> Result<()> {
        if volume < 0.0 {
            return Err(MathError::InvalidInput(
                "Volume cannot be negative".to_string(),
            ));
        }

        // Add new volume
        self.volumes.push_back(volume);
        self.sum += volume;

        // Remove oldest volume if we have more than period values
        if self.volumes.len() > self.period {
            if let Some(old_volume) = self.volumes.pop_front() {
                self.sum -= old_volume;
            }
        }

        Ok(())
    }

    /// Get the current VMA value
    pub fn value(&self) -> Result<f64> {
        if self.volumes.len() < self.period {
            return Err(MathError::InsufficientData(format!(
                "Not enough data for VMA calculation. Need {} values, have {}.",
                self.period,
                self.volumes.len()
            )));
        }

        Ok(self.sum / self.period as f64)
    }

    /// Get the current period
    pub fn period(&self) -> usize {
        self.period
    }

    /// Reset the VMA, clearing all values
    pub fn reset(&mut self) {
        self.volumes.clear();
        self.sum = 0.0;
    }
}

/// On-Balance Volume (OBV) implementation
#[derive(Debug, Clone)]
pub struct OnBalanceVolume {
    obv: f64,
    previous_close: Option<f64>,
}

impl Default for OnBalanceVolume {
    fn default() -> Self {
        Self::new()
    }
}

impl OnBalanceVolume {
    /// Create a new On-Balance Volume indicator
    pub fn new() -> Self {
        Self {
            obv: 0.0,
            previous_close: None,
        }
    }

    /// Update the OBV with new price and volume data
    pub fn update(&mut self, close: f64, volume: f64) -> Result<()> {
        if volume < 0.0 {
            return Err(MathError::InvalidInput(
                "Volume cannot be negative".to_string(),
            ));
        }

        // If we have a previous close price, update OBV
        if let Some(prev_close) = self.previous_close {
            // OBV calculation rules:
            // - If current close > previous close, add volume to OBV
            // - If current close < previous close, subtract volume from OBV
            // - If current close = previous close, OBV remains unchanged
            if close > prev_close {
                self.obv += volume;
            } else if close < prev_close {
                self.obv -= volume;
            }
            // If close == prev_close, OBV doesn't change
        }

        // Update previous close for next calculation
        self.previous_close = Some(close);

        Ok(())
    }

    /// Get the current OBV value
    pub fn value(&self) -> Result<f64> {
        if self.previous_close.is_none() {
            return Err(MathError::InsufficientData(
                "Not enough data for OBV calculation. Need at least one price-volume update."
                    .to_string(),
            ));
        }

        Ok(self.obv)
    }

    /// Reset the OBV, clearing all values
    pub fn reset(&mut self) {
        self.obv = 0.0;
        self.previous_close = None;
    }
}

/// Volume Rate of Change (VROC) implementation
#[derive(Debug, Clone)]
pub struct VolumeRateOfChange {
    period: usize,
    volumes: VecDeque<f64>,
}

impl VolumeRateOfChange {
    /// Create a new Volume Rate of Change with the specified period
    pub fn new(period: usize) -> Result<Self> {
        if period == 0 {
            return Err(MathError::InvalidInput(
                "Period must be greater than zero".to_string(),
            ));
        }

        Ok(Self {
            period,
            volumes: VecDeque::with_capacity(period + 1),
        })
    }

    /// Update the VROC with a new volume value
    pub fn update(&mut self, volume: f64) -> Result<()> {
        if volume < 0.0 {
            return Err(MathError::InvalidInput(
                "Volume cannot be negative".to_string(),
            ));
        }

        self.volumes.push_back(volume);

        // Keep only necessary volumes (current + n periods ago)
        if self.volumes.len() > self.period + 1 {
            self.volumes.pop_front();
        }

        Ok(())
    }

    /// Get the current VROC value as percentage
    pub fn value(&self) -> Result<f64> {
        if self.volumes.len() <= self.period {
            return Err(MathError::InsufficientData(format!(
                "Not enough data for VROC calculation. Need at least {} values, have {}.",
                self.period + 1,
                self.volumes.len()
            )));
        }

        let current_volume = *self.volumes.back().unwrap();
        let old_volume = self.volumes[0];

        if old_volume == 0.0 {
            return Err(MathError::CalculationError(
                "Cannot calculate VROC: old volume is zero".to_string(),
            ));
        }

        // VROC = ((Current Volume - Volume n periods ago) / Volume n periods ago) * 100
        Ok((current_volume - old_volume) / old_volume * 100.0)
    }

    /// Get the current period
    pub fn period(&self) -> usize {
        self.period
    }

    /// Reset the VROC, clearing all values
    pub fn reset(&mut self) {
        self.volumes.clear();
    }
}

/// Volume Price Trend (VPT) implementation
#[derive(Debug, Clone)]
pub struct VolumePriceTrend {
    vpt: f64,
    previous_close: Option<f64>,
}

impl Default for VolumePriceTrend {
    fn default() -> Self {
        Self::new()
    }
}

impl VolumePriceTrend {
    /// Create a new Volume Price Trend indicator
    pub fn new() -> Self {
        Self {
            vpt: 0.0,
            previous_close: None,
        }
    }

    /// Update the VPT with new price and volume data
    pub fn update(&mut self, close: f64, volume: f64) -> Result<()> {
        if volume < 0.0 {
            return Err(MathError::InvalidInput(
                "Volume cannot be negative".to_string(),
            ));
        }

        // If we have a previous close price, update VPT
        if let Some(prev_close) = self.previous_close {
            // VPT = Previous VPT + Volume * ((Current Close - Previous Close) / Previous Close)
            let price_change_percent = (close - prev_close) / prev_close;
            self.vpt += volume * price_change_percent;
        }

        // Update previous close for next calculation
        self.previous_close = Some(close);

        Ok(())
    }

    /// Get the current VPT value
    pub fn value(&self) -> Result<f64> {
        if self.previous_close.is_none() {
            return Err(MathError::InsufficientData(
                "Not enough data for VPT calculation. Need at least one price-volume update."
                    .to_string(),
            ));
        }

        Ok(self.vpt)
    }

    /// Reset the VPT, clearing all values
    pub fn reset(&mut self) {
        self.vpt = 0.0;
        self.previous_close = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vma_calculation() {
        let mut vma = VolumeMovingAverage::new(3).unwrap();

        // Add test data
        vma.update(1000.0).unwrap();
        vma.update(1500.0).unwrap();
        vma.update(2000.0).unwrap();

        // Check VMA value: (1000 + 1500 + 2000) / 3 = 1500
        let vma_value = vma.value().unwrap();
        assert!((vma_value - 1500.0).abs() < 0.001);

        // Update with a new value
        vma.update(3000.0).unwrap();

        // Check updated VMA value: (1500 + 2000 + 3000) / 3 = 2166.67
        let updated_vma = vma.value().unwrap();
        assert!((updated_vma - 2166.67).abs() < 0.01);
    }

    #[test]
    fn test_obv_calculation() {
        let mut obv = OnBalanceVolume::new();

        // Initial value should be Error (not enough data)
        assert!(obv.value().is_err());

        // Add test data (Price, Volume)
        obv.update(10.0, 1000.0).unwrap(); // Initial

        // OBV should be 0 after first update (since there's no previous close)
        assert_eq!(obv.value().unwrap(), 0.0);

        // Price increases -> add volume
        obv.update(11.0, 1500.0).unwrap();
        assert_eq!(obv.value().unwrap(), 1500.0);

        // Price decreases -> subtract volume
        obv.update(10.5, 2000.0).unwrap();
        assert_eq!(obv.value().unwrap(), -500.0);

        // Price unchanged -> OBV unchanged
        obv.update(10.5, 1000.0).unwrap();
        assert_eq!(obv.value().unwrap(), -500.0);
    }

    #[test]
    fn test_vroc_calculation() {
        let mut vroc = VolumeRateOfChange::new(2).unwrap();

        // Add test data
        vroc.update(1000.0).unwrap();
        vroc.update(1200.0).unwrap();

        // Not enough data yet
        assert!(vroc.value().is_err());

        vroc.update(1500.0).unwrap();

        // VROC = ((1500 - 1000) / 1000) * 100 = 50%
        let vroc_value = vroc.value().unwrap();
        assert!((vroc_value - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_vpt_calculation() {
        let mut vpt = VolumePriceTrend::new();

        // Initial value should be Error (not enough data)
        assert!(vpt.value().is_err());

        // Add test data (Price, Volume)
        vpt.update(10.0, 1000.0).unwrap(); // Initial

        // VPT should be 0 after first update (since there's no previous close)
        assert_eq!(vpt.value().unwrap(), 0.0);

        // 10% price increase
        vpt.update(11.0, 1500.0).unwrap();
        // VPT = 0 + 1500 * (11 - 10) / 10 = 150
        assert!((vpt.value().unwrap() - 150.0).abs() < 0.001);

        // 5% price decrease
        vpt.update(10.45, 2000.0).unwrap();
        // VPT = 150 + 2000 * (10.45 - 11) / 11 = 150 - 100 = 50
        let vpt_value = vpt.value().unwrap();
        assert!((vpt_value - 50.0).abs() < 0.01);
    }
}
