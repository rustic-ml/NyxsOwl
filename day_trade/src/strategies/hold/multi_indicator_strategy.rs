//! Multi-indicator trading strategy combining RSI, MACD and Moving Averages

use crate::mock_indicators::{Macd, RelativeStrengthIndex, SimpleMovingAverage};
use crate::{DailyOhlcv, Signal, TradeError, TradingStrategy};
use chrono;

/// Strength of a trading signal
#[derive(Debug, Clone, Copy)]
enum SignalStrength {
    Strong,
    Moderate,
    Weak,
    Neutral,
}

/// Signal with associated strength
#[derive(Debug, Clone, Copy)]
struct WeightedSignal {
    signal: Signal,
    strength: SignalStrength,
}

/// Composite strategy that combines multiple technical indicators
pub struct CompositeStrategy {
    // RSI configuration
    rsi_period: usize,
    rsi_overbought: f64,
    rsi_oversold: f64,

    // MACD configuration
    macd_fast_period: usize,
    macd_slow_period: usize,
    macd_signal_period: usize,

    // Moving averages configuration
    short_ma_period: usize,
    medium_ma_period: usize,
    long_ma_period: usize,

    // Signal weights (0.0 to 1.0)
    rsi_weight: f64,
    macd_weight: f64,
    ma_weight: f64,
}

impl CompositeStrategy {
    /// Create a new composite strategy with custom parameters
    pub fn new(
        rsi_period: usize,
        rsi_overbought: f64,
        rsi_oversold: f64,
        macd_fast_period: usize,
        macd_slow_period: usize,
        macd_signal_period: usize,
        short_ma_period: usize,
        medium_ma_period: usize,
        long_ma_period: usize,
        rsi_weight: f64,
        macd_weight: f64,
        ma_weight: f64,
    ) -> Self {
        Self {
            rsi_period,
            rsi_overbought,
            rsi_oversold,
            macd_fast_period,
            macd_slow_period,
            macd_signal_period,
            short_ma_period,
            medium_ma_period,
            long_ma_period,
            rsi_weight,
            macd_weight,
            ma_weight,
        }
    }

    /// Create a composite strategy with default parameters
    pub fn default() -> Self {
        Self {
            // RSI configuration - standard parameters
            rsi_period: 14,
            rsi_overbought: 70.0,
            rsi_oversold: 30.0,

            // MACD configuration - standard parameters
            macd_fast_period: 12,
            macd_slow_period: 26,
            macd_signal_period: 9,

            // Moving averages - 20, 50, 200 day are common
            short_ma_period: 20,
            medium_ma_period: 50,
            long_ma_period: 200,

            // Equal weighting by default
            rsi_weight: 0.33,
            macd_weight: 0.33,
            ma_weight: 0.34,
        }
    }

    /// Analyze RSI to generate a weighted signal
    fn analyze_rsi(
        &self,
        prices: &[f64],
        index: usize,
        rsi: &mut RelativeStrengthIndex,
    ) -> Result<WeightedSignal, TradeError> {
        if index < self.rsi_period {
            return Ok(WeightedSignal {
                signal: Signal::Hold,
                strength: SignalStrength::Neutral,
            });
        }

        let rsi_value = rsi
            .value()
            .map_err(|e| TradeError::CalculationError(format!("Failed to get RSI value: {}", e)))?;

        // Basic RSI interpretation
        let signal = if rsi_value < self.rsi_oversold {
            // Oversold - potential buy signal
            Signal::Buy
        } else if rsi_value > self.rsi_overbought {
            // Overbought - potential sell signal
            Signal::Sell
        } else {
            Signal::Hold
        };

        // Determine signal strength based on distance from thresholds
        let strength = match signal {
            Signal::Buy => {
                if rsi_value < self.rsi_oversold - 10.0 {
                    SignalStrength::Strong
                } else if rsi_value < self.rsi_oversold - 5.0 {
                    SignalStrength::Moderate
                } else {
                    SignalStrength::Weak
                }
            }
            Signal::Sell => {
                if rsi_value > self.rsi_overbought + 10.0 {
                    SignalStrength::Strong
                } else if rsi_value > self.rsi_overbought + 5.0 {
                    SignalStrength::Moderate
                } else {
                    SignalStrength::Weak
                }
            }
            Signal::Hold => SignalStrength::Neutral,
        };

        Ok(WeightedSignal { signal, strength })
    }

    /// Analyze MACD to generate a weighted signal
    fn analyze_macd(
        &self,
        prices: &[f64],
        index: usize,
        macd: &mut Macd,
        prev_histogram: &mut Option<f64>,
    ) -> Result<WeightedSignal, TradeError> {
        let min_periods = self.macd_slow_period + self.macd_signal_period - 1;
        if index < min_periods {
            return Ok(WeightedSignal {
                signal: Signal::Hold,
                strength: SignalStrength::Neutral,
            });
        }

        let macd_line = macd
            .macd_value()
            .map_err(|e| TradeError::CalculationError(format!("Failed to get MACD line: {}", e)))?;

        let signal_line = macd.signal_value().map_err(|e| {
            TradeError::CalculationError(format!("Failed to get signal line: {}", e))
        })?;

        let histogram = macd_line - signal_line;

        let mut signal = Signal::Hold;
        let mut strength = SignalStrength::Neutral;

        // Check for crossovers
        if let Some(prev) = *prev_histogram {
            // MACD line crosses above signal line (histogram goes from negative to positive)
            if histogram > 0.0 && prev <= 0.0 {
                signal = Signal::Buy;

                // Determine strength based on the steepness of the crossover
                let crossover_strength = (histogram - prev).abs();
                strength = if crossover_strength > 0.2 {
                    SignalStrength::Strong
                } else if crossover_strength > 0.1 {
                    SignalStrength::Moderate
                } else {
                    SignalStrength::Weak
                };
            }
            // MACD line crosses below signal line (histogram goes from positive to negative)
            else if histogram < 0.0 && prev >= 0.0 {
                signal = Signal::Sell;

                // Determine strength based on the steepness of the crossover
                let crossover_strength = (histogram - prev).abs();
                strength = if crossover_strength > 0.2 {
                    SignalStrength::Strong
                } else if crossover_strength > 0.1 {
                    SignalStrength::Moderate
                } else {
                    SignalStrength::Weak
                };
            }
        }

        *prev_histogram = Some(histogram);

        Ok(WeightedSignal { signal, strength })
    }

    /// Analyze moving averages to generate a weighted signal
    fn analyze_moving_averages(
        &self,
        prices: &[f64],
        index: usize,
        short_ma: &mut SimpleMovingAverage,
        medium_ma: &mut SimpleMovingAverage,
        long_ma: &mut SimpleMovingAverage,
    ) -> Result<WeightedSignal, TradeError> {
        let min_periods = self.long_ma_period;
        if index < min_periods {
            return Ok(WeightedSignal {
                signal: Signal::Hold,
                strength: SignalStrength::Neutral,
            });
        }

        let short_value = short_ma
            .value()
            .map_err(|e| TradeError::CalculationError(format!("Failed to get short MA: {}", e)))?;

        let medium_value = medium_ma
            .value()
            .map_err(|e| TradeError::CalculationError(format!("Failed to get medium MA: {}", e)))?;

        let long_value = long_ma
            .value()
            .map_err(|e| TradeError::CalculationError(format!("Failed to get long MA: {}", e)))?;

        // Count bullish and bearish signals
        let mut bullish_count = 0;
        let mut bearish_count = 0;

        // Short MA above medium MA is bullish
        if short_value > medium_value {
            bullish_count += 1;
        } else {
            bearish_count += 1;
        }

        // Medium MA above long MA is bullish
        if medium_value > long_value {
            bullish_count += 1;
        } else {
            bearish_count += 1;
        }

        // Price above short MA is bullish
        if prices[index] > short_value {
            bullish_count += 1;
        } else {
            bearish_count += 1;
        }

        // Generate signal based on bullish/bearish count
        let signal = if bullish_count >= 3 {
            Signal::Buy
        } else if bearish_count >= 3 {
            Signal::Sell
        } else {
            Signal::Hold
        };

        // Determine strength based on agreement level
        let strength = match signal {
            Signal::Buy => {
                if bullish_count == 3 {
                    SignalStrength::Strong
                } else if bullish_count == 2 {
                    SignalStrength::Moderate
                } else {
                    SignalStrength::Weak
                }
            }
            Signal::Sell => {
                if bearish_count == 3 {
                    SignalStrength::Strong
                } else if bearish_count == 2 {
                    SignalStrength::Moderate
                } else {
                    SignalStrength::Weak
                }
            }
            Signal::Hold => SignalStrength::Neutral,
        };

        Ok(WeightedSignal { signal, strength })
    }

    /// Convert a signal strength to a numeric value for weighting
    fn strength_to_value(&self, strength: SignalStrength) -> f64 {
        match strength {
            SignalStrength::Strong => 1.0,
            SignalStrength::Moderate => 0.7,
            SignalStrength::Weak => 0.3,
            SignalStrength::Neutral => 0.0,
        }
    }

    /// Combine signals from different indicators with their respective weights
    fn combine_signals(
        &self,
        rsi_signal: WeightedSignal,
        macd_signal: WeightedSignal,
        ma_signal: WeightedSignal,
    ) -> Signal {
        let mut buy_score = 0.0;
        let mut sell_score = 0.0;

        // Calculate RSI contribution
        match rsi_signal.signal {
            Signal::Buy => {
                buy_score += self.rsi_weight * self.strength_to_value(rsi_signal.strength)
            }
            Signal::Sell => {
                sell_score += self.rsi_weight * self.strength_to_value(rsi_signal.strength)
            }
            Signal::Hold => {} // No contribution
        }

        // Calculate MACD contribution
        match macd_signal.signal {
            Signal::Buy => {
                buy_score += self.macd_weight * self.strength_to_value(macd_signal.strength)
            }
            Signal::Sell => {
                sell_score += self.macd_weight * self.strength_to_value(macd_signal.strength)
            }
            Signal::Hold => {} // No contribution
        }

        // Calculate MA contribution
        match ma_signal.signal {
            Signal::Buy => buy_score += self.ma_weight * self.strength_to_value(ma_signal.strength),
            Signal::Sell => {
                sell_score += self.ma_weight * self.strength_to_value(ma_signal.strength)
            }
            Signal::Hold => {} // No contribution
        }

        // Determine final signal based on scores
        if buy_score > sell_score && buy_score > 0.5 {
            if buy_score > 0.8 {
                Signal::Buy // Strong buy
            } else {
                Signal::Buy // Regular buy
            }
        } else if sell_score > buy_score && sell_score > 0.5 {
            if sell_score > 0.8 {
                Signal::Sell // Strong sell
            } else {
                Signal::Sell // Regular sell
            }
        } else {
            Signal::Hold
        }
    }
}

impl TradingStrategy for CompositeStrategy {
    fn generate_signals(&self, data: &[DailyOhlcv]) -> Result<Vec<Signal>, TradeError> {
        if data.is_empty() {
            return Err(TradeError::InsufficientData(
                "No price data provided".to_string(),
            ));
        }

        let mut signals = Vec::with_capacity(data.len());
        let mut prices = Vec::with_capacity(data.len());

        // Extract closing prices
        for candle in data {
            prices.push(candle.data.close);
        }

        // Initialize indicators
        let mut rsi = RelativeStrengthIndex::new(self.rsi_period);
        let mut macd = Macd::new(
            self.macd_fast_period,
            self.macd_slow_period,
            self.macd_signal_period,
        );
        let mut short_ma = SimpleMovingAverage::new(self.short_ma_period);
        let mut medium_ma = SimpleMovingAverage::new(self.medium_ma_period);
        let mut long_ma = SimpleMovingAverage::new(self.long_ma_period);

        // For tracking the previous MACD histogram value
        let mut prev_histogram: Option<f64> = None;

        // Process each price point
        for (i, price) in prices.iter().enumerate() {
            // Update indicators
            if let Err(e) = rsi
                .as_mut()
                .unwrap_or_else(|e| panic!("{}", e))
                .update(*price)
            {
                return Err(TradeError::CalculationError(e));
            }
            if let Err(e) = macd
                .as_mut()
                .unwrap_or_else(|e| panic!("{}", e))
                .update(*price)
            {
                return Err(TradeError::CalculationError(e));
            }
            if let Err(e) = short_ma
                .as_mut()
                .unwrap_or_else(|e| panic!("{}", e))
                .update(*price)
            {
                return Err(TradeError::CalculationError(e));
            }
            if let Err(e) = medium_ma
                .as_mut()
                .unwrap_or_else(|e| panic!("{}", e))
                .update(*price)
            {
                return Err(TradeError::CalculationError(e));
            }
            if let Err(e) = long_ma
                .as_mut()
                .unwrap_or_else(|e| panic!("{}", e))
                .update(*price)
            {
                return Err(TradeError::CalculationError(e));
            }

            // Generate signals from each indicator
            let rsi_signal = self.analyze_rsi(
                &prices,
                i,
                &mut rsi.as_mut().unwrap_or_else(|e| panic!("{}", e)),
            )?;
            let macd_signal = self.analyze_macd(
                &prices,
                i,
                &mut macd.as_mut().unwrap_or_else(|e| panic!("{}", e)),
                &mut prev_histogram,
            )?;
            let ma_signal = self.analyze_moving_averages(
                &prices,
                i,
                &mut short_ma.as_mut().unwrap_or_else(|e| panic!("{}", e)),
                &mut medium_ma.as_mut().unwrap_or_else(|e| panic!("{}", e)),
                &mut long_ma.as_mut().unwrap_or_else(|e| panic!("{}", e)),
            )?;

            // Combine signals with weights
            let combined_signal = self.combine_signals(rsi_signal, macd_signal, ma_signal);

            // Add to signals list
            signals.push(combined_signal);
        }

        Ok(signals)
    }

    fn calculate_performance(
        &self,
        data: &[DailyOhlcv],
        signals: &[Signal],
    ) -> Result<f64, TradeError> {
        if data.len() != signals.len() {
            return Err(TradeError::CalculationError(
                "Data and signals length mismatch".to_string(),
            ));
        }

        if data.len() < 2 {
            return Err(TradeError::InsufficientData(
                "Need at least 2 data points".to_string(),
            ));
        }

        let mut position = 0.0; // Current position size
        let mut cash = 10000.0; // Starting cash
        let mut equity = cash; // Starting equity

        for i in 1..data.len() {
            let signal = signals[i - 1]; // Signal from the previous day
            let price = data[i].data.open; // Execute at the open of the next day

            if price <= 0.0 {
                return Err(TradeError::CalculationError(format!(
                    "Invalid price data at day {}",
                    i
                )));
            }

            match signal {
                Signal::Buy if position <= 0.0 => {
                    // Close any short position
                    cash += position * price;
                    // Go long with all available cash
                    position = cash / price;
                    cash = 0.0;
                }
                Signal::Sell if position >= 0.0 => {
                    // Close any long position
                    cash += position * price;
                    // Go short with all available cash
                    position = -cash / price;
                    cash = cash * 2.0; // Reserve cash for covering the short
                }
                _ => {} // Hold current position
            }

            // Update equity value at the end of the day
            equity = cash + (position * data[i].data.close);
        }

        // Calculate total return
        let return_pct = (equity - 10000.0) / 10000.0 * 100.0;

        Ok(return_pct)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create test price data
    fn create_test_data() -> Vec<DailyOhlcv> {
        // Generate a sample price series
        let prices = vec![
            100.0, 101.0, 102.0, 103.0, 105.0, 106.0, 107.0, 106.0, 105.0, 104.0, // 0-9
            103.0, 102.0, 101.0, 100.0, 99.0, 98.0, 97.0, 96.0, 95.0, 94.0, // 10-19
            93.0, 92.0, 91.0, 90.0, 91.0, 92.0, 93.0, 94.0, 95.0, 96.0, // 20-29
            97.0, 98.0, 99.0, 100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, // 30-39
            107.0, 108.0, 109.0, 110.0, 111.0, 112.0, 111.0, 110.0, 109.0, 108.0, // 40-49
            107.0, 106.0, 105.0, 104.0, 103.0, 102.0, 101.0, 100.0, 99.0, 98.0, // 50-59
            97.0, 96.0, 95.0, 94.0, 93.0, 92.0, 91.0, 90.0, 89.0, 88.0, // 60-69
            87.0, 86.0, 85.0, 86.0, 87.0, 88.0, 89.0, 90.0, 91.0, 92.0, // 70-79
            93.0, 94.0, 95.0, 96.0, 97.0, 98.0, 99.0, 100.0, 101.0, 102.0, // 80-89
            103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0, 110.0, 111.0, 112.0, // 90-99
        ];

        // Convert to OHLCV format - use close as the base and generate
        // other values with small variations
        let mut ohlcv_data = Vec::with_capacity(prices.len());

        for (i, &close) in prices.iter().enumerate() {
            let open = if i == 0 { close } else { prices[i - 1] };
            let high = close.max(open) + (close * 0.01); // 1% above max of open/close
            let low = close.min(open) - (close * 0.01); // 1% below min of open/close
            let volume = close * 1000.0; // Just a placeholder

            ohlcv_data.push(DailyOhlcv {
                date: chrono::NaiveDate::from_ymd_opt(2023, (i / 30) + 1, (i % 30) + 1)
                    .unwrap_or_default(),
                data: OhlcvData {
                    open,
                    high,
                    low,
                    close,
                    volume: volume as u64,
                },
            });
        }

        ohlcv_data
    }

    #[test]
    fn test_composite_signals() {
        let strategy = CompositeStrategy::default();
        let test_data = create_test_data();

        let signals = strategy.generate_signals(&test_data).unwrap();

        // Just validate signal generation works at all
        assert_eq!(signals.len(), test_data.len());

        // Check that we get some of each signal type (not just all holds)
        let buy_count = signals.iter().filter(|&s| matches!(s, Signal::Buy)).count();
        let sell_count = signals
            .iter()
            .filter(|&s| matches!(s, Signal::Sell))
            .count();
        let hold_count = signals
            .iter()
            .filter(|&s| matches!(s, Signal::Hold))
            .count();

        assert!(buy_count > 0, "Should have some buy signals");
        assert!(sell_count > 0, "Should have some sell signals");
        assert!(hold_count > 0, "Should have some hold signals");
    }

    #[test]
    fn test_performance_calculation() {
        let strategy = CompositeStrategy::default();
        let test_data = create_test_data();

        let signals = strategy.generate_signals(&test_data).unwrap();
        let performance = strategy
            .calculate_performance(&test_data, &signals)
            .unwrap();

        // Verify we get a performance number - not checking the exact value
        // as it depends on the signal generation that might change
        assert!(performance != 0.0, "Performance should not be exactly zero");
    }

    #[test]
    fn test_custom_weights() {
        // Create a strategy that heavily weighs RSI
        let rsi_biased = CompositeStrategy::new(
            14, 70.0, 30.0, // RSI params
            12, 26, 9, // MACD params
            20, 50, 200, // MA params
            0.8, 0.1, 0.1, // Weights strongly biased toward RSI
        );

        // Create a strategy that heavily weighs MACD
        let macd_biased = CompositeStrategy::new(
            14, 70.0, 30.0, // RSI params
            12, 26, 9, // MACD params
            20, 50, 200, // MA params
            0.1, 0.8, 0.1, // Weights strongly biased toward MACD
        );

        let test_data = create_test_data();

        let rsi_signals = rsi_biased.generate_signals(&test_data).unwrap();
        let macd_signals = macd_biased.generate_signals(&test_data).unwrap();

        // The signals should be different due to different weightings
        let mut different_signals = 0;
        for i in 0..rsi_signals.len() {
            if rsi_signals[i] != macd_signals[i] {
                different_signals += 1;
            }
        }

        // At least some signals should differ
        assert!(
            different_signals > 0,
            "Different weightings should produce at least some different signals"
        );
    }
}
