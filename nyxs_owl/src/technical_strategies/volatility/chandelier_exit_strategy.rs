use crate::simple_types::{NyxsOwlError, Signal};
use crate::trade_math::volatility::calculate_chandelier_exit;
use polars::prelude::*;

/// ChandelierExitStrategy implements a trading strategy based on the Chandelier Exit indicator.
///
/// The strategy generates trading signals based on price movements relative to the Chandelier Exit levels:
/// - Buy when price closes above the Short Exit level (bullish)
/// - Sell when price closes below the Long Exit level (bearish)
/// - Hold when price is between the exit levels (neutral)
pub struct ChandelierExitStrategy {
    period: usize,
    atr_period: usize,
    multiplier: f64,
}

impl ChandelierExitStrategy {
    /// Creates a new ChandelierExitStrategy instance
    ///
    /// # Arguments
    /// * `period` - The lookback period for highest high/lowest low (default: 22)
    /// * `atr_period` - The period for ATR calculation (default: 22)
    /// * `multiplier` - The ATR multiplier (default: 3.0)
    pub fn new(period: usize, atr_period: usize, multiplier: f64) -> Self {
        Self {
            period,
            atr_period,
            multiplier,
        }
    }
}

impl Default for ChandelierExitStrategy {
    /// Creates a new ChandelierExitStrategy instance with default parameters:
    /// - period: 22
    /// - atr_period: 22
    /// - multiplier: 3.0
    fn default() -> Self {
        Self::new(22, 22, 3.0)
    }
}

impl ChandelierExitStrategy {
    /// Generates trading signals based on the Chandelier Exit indicator
    ///
    /// # Arguments
    /// * `high` - Series of high prices
    /// * `low` - Series of low prices
    /// * `close` - Series of closing prices
    ///
    /// # Returns
    /// * `Result<Vec<Signal>, NyxsOwlError>` - Vector of trading signals
    pub fn generate_signals(
        &self,
        high: &Series,
        low: &Series,
        close: &Series,
    ) -> Result<Vec<Signal>, NyxsOwlError> {
        // Calculate Chandelier Exit levels
        let (long_exit, short_exit) = calculate_chandelier_exit(
            high,
            low,
            close,
            self.period,
            self.atr_period,
            self.multiplier,
        )
        .map_err(|e| {
            NyxsOwlError::ValidationError(format!("Failed to calculate Chandelier Exit: {}", e))
        })?;

        let close_values: Vec<Option<f64>> = close
            .f64()
            .map_err(|e| NyxsOwlError::DataError(e.to_string()))?
            .into_iter()
            .collect();
        let long_values: Vec<Option<f64>> = long_exit
            .f64()
            .map_err(|e| NyxsOwlError::DataError(e.to_string()))?
            .into_iter()
            .collect();
        let short_values: Vec<Option<f64>> = short_exit
            .f64()
            .map_err(|e| NyxsOwlError::DataError(e.to_string()))?
            .into_iter()
            .collect();

        let mut signals = vec![Signal::Hold; close_values.len()];

        // Generate signals starting from the warmup period
        for i in (self.period - 1)..close_values.len() {
            if let (Some(close_val), Some(long_val), Some(short_val)) =
                (close_values[i], long_values[i], short_values[i])
            {
                signals[i] = if close_val > short_val {
                    Signal::Buy
                } else if close_val < long_val {
                    Signal::Sell
                } else {
                    Signal::Hold
                };
            }
        }

        Ok(signals)
    }

    /// Calculates the confidence score for the current signal
    ///
    /// # Arguments
    /// * `high` - Series of high prices
    /// * `low` - Series of low prices
    /// * `close` - Series of closing prices
    /// * `index` - The index for which to calculate the confidence score
    ///
    /// # Returns
    /// * `Result<f64, NyxsOwlError>` - Confidence score between 0.0 and 1.0
    pub fn calculate_confidence(
        &self,
        high: &Series,
        low: &Series,
        close: &Series,
        index: usize,
    ) -> Result<f64, NyxsOwlError> {
        if index < self.period {
            return Ok(0.0);
        }

        let (long_exit, short_exit) = calculate_chandelier_exit(
            high,
            low,
            close,
            self.period,
            self.atr_period,
            self.multiplier,
        )
        .map_err(|e| {
            NyxsOwlError::ValidationError(format!("Failed to calculate Chandelier Exit: {}", e))
        })?;

        let close_val = close
            .f64()
            .map_err(|e| NyxsOwlError::DataError(e.to_string()))?
            .get(index)
            .ok_or_else(|| NyxsOwlError::DataError("Invalid index".into()))?;
        let long_val = long_exit
            .f64()
            .map_err(|e| NyxsOwlError::DataError(e.to_string()))?
            .get(index)
            .ok_or_else(|| NyxsOwlError::DataError("Invalid index".into()))?;
        let short_val = short_exit
            .f64()
            .map_err(|e| NyxsOwlError::DataError(e.to_string()))?
            .get(index)
            .ok_or_else(|| NyxsOwlError::DataError("Invalid index".into()))?;

        if close_val.is_nan() || long_val.is_nan() || short_val.is_nan() {
            return Ok(0.0);
        }

        // Calculate confidence based on distance from exit levels
        let range = short_val - long_val;
        if range <= 0.0 {
            // For constant prices or invalid range, return neutral confidence
            return Ok(0.5);
        }

        let confidence = if close_val > short_val {
            // Bullish confidence
            ((close_val - short_val) / range).min(1.0)
        } else if close_val < long_val {
            // Bearish confidence
            ((long_val - close_val) / range).min(1.0)
        } else {
            // Neutral confidence based on position between exits
            0.5 - (close_val - long_val).abs() / range
        };

        Ok(confidence.clamp(0.0, 1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chandelier_exit_strategy() {
        let high = Series::new(
            "high".into(),
            vec![
                110.0, 112.0, 115.0, 113.0, 116.0, 118.0, 117.0, 119.0, 121.0, 120.0, 122.0, 123.0,
                124.0, 125.0, 126.0, 127.0, 128.0, 129.0, 130.0, 131.0, 132.0, 133.0, 134.0, 135.0,
                136.0,
            ],
        );
        let low = Series::new(
            "low".into(),
            vec![
                108.0, 109.0, 111.0, 110.0, 112.0, 114.0, 115.0, 116.0, 118.0, 119.0, 120.0, 121.0,
                122.0, 123.0, 124.0, 125.0, 126.0, 127.0, 128.0, 129.0, 130.0, 131.0, 132.0, 133.0,
                134.0,
            ],
        );
        let close = Series::new(
            "close".into(),
            vec![
                109.0, 111.0, 113.0, 112.0, 115.0, 116.0, 116.5, 118.0, 120.0, 119.5, 121.0, 122.0,
                123.0, 124.0, 125.0, 126.0, 127.0, 128.0, 129.0, 130.0, 131.0, 132.0, 133.0, 134.0,
                135.0,
            ],
        );

        let strategy = ChandelierExitStrategy::default();
        let signals = strategy.generate_signals(&high, &low, &close).unwrap();

        // Test signal generation
        assert_eq!(signals.len(), close.len());

        // Test warmup period
        for i in 0..21 {
            assert_eq!(signals[i], Signal::Hold);
        }

        // Test confidence calculation
        let confidence = strategy
            .calculate_confidence(&high, &low, &close, 22)
            .unwrap();
        assert!(confidence >= 0.0 && confidence <= 1.0);

        // Test invalid parameters
        let invalid_strategy = ChandelierExitStrategy::new(0, 22, 3.0);
        assert!(invalid_strategy
            .generate_signals(&high, &low, &close)
            .is_err());
    }

    #[test]
    fn test_strategy_with_constant_prices() {
        let constant_high = Series::new("high".into(), vec![100.0; 25]);
        let constant_low = Series::new("low".into(), vec![100.0; 25]);
        let constant_close = Series::new("close".into(), vec![100.0; 25]);

        let strategy = ChandelierExitStrategy::default();
        let signals = strategy
            .generate_signals(&constant_high, &constant_low, &constant_close)
            .unwrap();

        // With constant prices, all signals after warmup should be Hold
        for signal in signals.iter().skip(21) {
            assert_eq!(*signal, Signal::Hold);
        }

        // Confidence should be around 0.5 for constant prices
        let confidence = strategy
            .calculate_confidence(&constant_high, &constant_low, &constant_close, 22)
            .unwrap();
        assert!((confidence - 0.5).abs() < 0.1);
    }
}
