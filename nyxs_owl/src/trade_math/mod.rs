//! # Trade Math
//!
//! Mathematical calculations for trading indicators and strategies.
//! This crate provides implementations of common technical indicators and
//! trading calculations.

use thiserror::Error;

// Indicator modules
pub mod forecasting;
pub mod moving_averages;
pub mod oscillators;
pub mod volatility;
pub mod volume;

pub mod simple_api {
    //! Simplified functions for common technical indicators
    //!
    //! This module provides easy-to-use functions that calculate technical indicators
    //! with a single function call, ideal for beginners and simple use cases.

    use super::{
        moving_averages::{ExponentialMovingAverage, SimpleMovingAverage},
        oscillators::RelativeStrengthIndex,
        volatility::BollingerBands,
    };
    use crate::simple_types::{NyxsOwlError, Price, Result};

    /// Calculate Simple Moving Average for the entire price series
    ///
    /// # Example
    /// ```
    /// use nyxs_owl::prelude::*;
    ///
    /// let prices = vec![100.0, 102.0, 104.0, 103.0, 105.0];
    /// let sma_values = sma(&prices, 3).unwrap();
    /// assert_eq!(sma_values.len(), 3); // 5 prices - 3 period + 1
    /// ```
    pub fn sma(prices: &[Price], period: usize) -> Result<Vec<Price>> {
        if prices.len() < period {
            return Err(NyxsOwlError::InvalidParameter(format!(
                "Need at least {} prices for period {}",
                period, period
            )));
        }

        let mut indicator = SimpleMovingAverage::new(period)
            .map_err(|e| NyxsOwlError::InvalidParameter(e.to_string()))?;

        let mut results = Vec::new();

        for &price in prices {
            indicator
                .update(price)
                .map_err(|e| NyxsOwlError::DataError(e.to_string()))?;

            if let Ok(value) = indicator.value() {
                results.push(value);
            }
        }

        Ok(results)
    }

    /// Calculate Exponential Moving Average for the entire price series
    ///
    /// # Example
    /// ```
    /// use nyxs_owl::prelude::*;
    ///
    /// let prices = vec![100.0, 102.0, 104.0, 103.0, 105.0];
    /// let ema_values = ema(&prices, 3).unwrap();
    /// ```
    pub fn ema(prices: &[Price], period: usize) -> Result<Vec<Price>> {
        if prices.len() < period {
            return Err(NyxsOwlError::InvalidParameter(format!(
                "Need at least {} prices for period {}",
                period, period
            )));
        }

        let mut indicator = ExponentialMovingAverage::new(period)
            .map_err(|e| NyxsOwlError::InvalidParameter(e.to_string()))?;

        let mut results = Vec::new();

        for &price in prices {
            indicator
                .update(price)
                .map_err(|e| NyxsOwlError::DataError(e.to_string()))?;

            if let Ok(value) = indicator.value() {
                results.push(value);
            }
        }

        Ok(results)
    }

    /// Calculate RSI for the entire price series
    ///
    /// # Example
    /// ```
    /// use nyxs_owl::prelude::*;
    ///
    /// let prices = vec![100.0, 102.0, 104.0, 103.0, 105.0, 104.0, 106.0];
    /// let rsi_values = rsi(&prices, 5).unwrap();
    /// ```
    pub fn rsi(prices: &[Price], period: usize) -> Result<Vec<Price>> {
        if prices.len() < period + 1 {
            return Err(NyxsOwlError::InvalidParameter(format!(
                "Need at least {} prices for RSI period {}",
                period + 1,
                period
            )));
        }

        let mut indicator = RelativeStrengthIndex::new(period)
            .map_err(|e| NyxsOwlError::InvalidParameter(e.to_string()))?;

        let mut results = Vec::new();

        for &price in prices {
            indicator
                .update(price)
                .map_err(|e| NyxsOwlError::DataError(e.to_string()))?;

            if let Ok(value) = indicator.value() {
                results.push(value);
            }
        }

        Ok(results)
    }

    /// Bollinger Bands result
    #[derive(Debug, Clone)]
    pub struct BollingerBandsResult {
        /// Upper band values
        pub upper: Vec<Price>,
        /// Middle band (SMA) values  
        pub middle: Vec<Price>,
        /// Lower band values
        pub lower: Vec<Price>,
    }

    /// Calculate Bollinger Bands for the entire price series
    ///
    /// # Example
    /// ```
    /// use nyxs_owl::prelude::*;
    ///
    /// let prices = vec![100.0, 102.0, 104.0, 103.0, 105.0, 104.0, 106.0];
    /// let bb = bollinger_bands(&prices, 5, 2.0).unwrap();
    /// assert!(!bb.upper.is_empty());
    /// assert!(!bb.middle.is_empty());
    /// assert!(!bb.lower.is_empty());
    /// ```
    pub fn bollinger_bands(
        prices: &[Price],
        period: usize,
        std_dev: Price,
    ) -> Result<BollingerBandsResult> {
        if prices.len() < period {
            return Err(NyxsOwlError::InvalidParameter(format!(
                "Need at least {} prices for period {}",
                period, period
            )));
        }

        let mut indicator = BollingerBands::new(period, std_dev)
            .map_err(|e| NyxsOwlError::InvalidParameter(e.to_string()))?;

        let mut upper = Vec::new();
        let mut middle = Vec::new();
        let mut lower = Vec::new();

        for &price in prices {
            indicator
                .update(price)
                .map_err(|e| NyxsOwlError::DataError(e.to_string()))?;

            if let (Ok(u), Ok(m), Ok(l)) = (
                indicator.upper_band(),
                indicator.middle_band(),
                indicator.lower_band(),
            ) {
                upper.push(u);
                middle.push(m);
                lower.push(l);
            }
        }

        Ok(BollingerBandsResult {
            upper,
            middle,
            lower,
        })
    }
}

/// Errors that can occur in trading-related calculations
#[derive(Error, Debug)]
pub enum MathError {
    /// Insufficient data for calculation
    #[error("Insufficient data for calculation: {0}")]
    InsufficientData(String),

    /// Invalid input provided
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Calculation error occurred
    #[error("Calculation error: {0}")]
    CalculationError(String),
}

/// Result type for trading math operations
pub type Result<T> = std::result::Result<T, MathError>;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
