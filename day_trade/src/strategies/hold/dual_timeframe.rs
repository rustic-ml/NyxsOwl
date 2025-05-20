//! Dual Timeframe strategy for combining daily and minute data analysis

use crate::mock_indicators::TimeSeriesPredictor;
use crate::{MinuteOhlcv, Signal, TradingStrategy};

/// Strategy that combines daily and minute data for more accurate signals
pub struct DualTimeframeStrategy {
    daily_strategy: Box<dyn TradingStrategy>,
    confirmation_period: usize, // Number of minutes to confirm the signal
    predictor: TimeSeriesPredictor,
}

impl DualTimeframeStrategy {
    /// Create a new dual timeframe strategy
    pub fn new(daily_strategy: Box<dyn TradingStrategy>, confirmation_period: usize) -> Self {
        // Initialize forecaster with default settings
        let predictor = TimeSeriesPredictor::new(
            confirmation_period, // prediction horizon
            5,                   // embedding dimension
            true,                // use moving average
        )
        .expect("Failed to create TimeSeriesPredictor");

        Self {
            daily_strategy,
            confirmation_period,
            predictor,
        }
    }

    /// Analyze minute data to confirm daily signals using forecasting
    pub fn confirm_signal(&self, daily_signal: Signal, minute_data: &[MinuteOhlcv]) -> Signal {
        if minute_data.len() < self.confirmation_period {
            return Signal::Hold;
        }

        // Extract close prices from minute data
        let close_prices: Vec<f64> = minute_data.iter().map(|m| m.data.close).collect();

        // Use oxidiviner to forecast future price movement
        if let Ok(forecast) = self.predictor.forecast(&close_prices) {
            // Calculate potential future movement based on forecast
            if let Some(last_price) = close_prices.last() {
                if let Some(forecast_price) = forecast.last() {
                    let price_change_pct = (forecast_price - last_price) / last_price * 100.0;

                    // Strong upward forecast confirms buy signal
                    if daily_signal == Signal::Buy && price_change_pct > 0.5 {
                        return Signal::Buy;
                    }
                    // Strong downward forecast confirms sell signal
                    else if daily_signal == Signal::Sell && price_change_pct < -0.5 {
                        return Signal::Sell;
                    }
                    // If forecast contradicts daily signal, stay neutral
                    else if (daily_signal == Signal::Buy && price_change_pct < -0.2)
                        || (daily_signal == Signal::Sell && price_change_pct > 0.2)
                    {
                        return Signal::Hold;
                    }
                }
            }
        }

        // If forecasting couldn't provide clear confirmation, return original signal
        daily_signal
    }
}
