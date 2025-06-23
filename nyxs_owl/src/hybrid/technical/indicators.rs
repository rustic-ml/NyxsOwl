use crate::hybrid::{error::HybridError, types::*};
use crate::trade_math::{
    momentum::{cci, mfi, roc},
    moving_averages::{ema, sma, vwap},
    volatility::{atr, bollinger_bands},
    volume::vwap as volume_vwap,
};
use polars::prelude::*;
use std::collections::HashMap;

/// Technical indicators for hybrid forecasting strategies
pub struct TechnicalIndicators {
    config: TechnicalConfig,
    cache: HashMap<String, Series>,
}

impl TechnicalIndicators {
    /// Create a new technical indicators instance
    pub fn new(config: TechnicalConfig) -> Self {
        Self {
            config,
            cache: HashMap::new(),
        }
    }

    /// Calculate momentum indicators
    pub fn calculate_momentum_indicators(
        &mut self,
        df: &DataFrame,
    ) -> Result<MomentumIndicators, HybridError> {
        let high = df.column("high")?;
        let low = df.column("low")?;
        let close = df.column("close")?;
        let volume = df.column("volume")?;

        let rsi_period = self.config.rsi_period;
        let cci_period = self.config.cci_period;
        let mfi_period = self.config.mfi_period;
        let roc_period = self.config.roc_period;

        // Calculate RSI
        let rsi = self.calculate_rsi(close, rsi_period)?;
        
        // Calculate CCI
        let cci = cci::calculate_cci(high, low, close, cci_period)?;
        
        // Calculate MFI
        let mfi = mfi::calculate_mfi(high, low, close, volume, mfi_period)?;
        
        // Calculate ROC
        let roc = roc::calculate_roc(close, roc_period)?;

        Ok(MomentumIndicators {
            rsi,
            cci,
            mfi,
            roc,
        })
    }

    /// Calculate trend indicators
    pub fn calculate_trend_indicators(
        &mut self,
        df: &DataFrame,
    ) -> Result<TrendIndicators, HybridError> {
        let high = df.column("high")?;
        let low = df.column("low")?;
        let close = df.column("close")?;

        let sma_short = self.config.sma_short_period;
        let sma_long = self.config.sma_long_period;
        let ema_short = self.config.ema_short_period;
        let ema_long = self.config.ema_long_period;

        // Calculate SMAs
        let sma_short_series = sma::calculate_sma(close, sma_short)?;
        let sma_long_series = sma::calculate_sma(close, sma_long)?;
        
        // Calculate EMAs
        let ema_short_series = ema::calculate_ema(close, ema_short)?;
        let ema_long_series = ema::calculate_ema(close, ema_long)?;

        // Calculate crossover signals
        let sma_crossover = self.calculate_crossover_signal(&sma_short_series, &sma_long_series)?;
        let ema_crossover = self.calculate_crossover_signal(&ema_short_series, &ema_long_series)?;

        Ok(TrendIndicators {
            sma_short: sma_short_series,
            sma_long: sma_long_series,
            ema_short: ema_short_series,
            ema_long: ema_long_series,
            sma_crossover,
            ema_crossover,
        })
    }

    /// Calculate volatility indicators
    pub fn calculate_volatility_indicators(
        &mut self,
        df: &DataFrame,
    ) -> Result<VolatilityIndicators, HybridError> {
        let high = df.column("high")?;
        let low = df.column("low")?;
        let close = df.column("close")?;

        let atr_period = self.config.atr_period;
        let bb_period = self.config.bollinger_period;
        let bb_std = self.config.bollinger_std_dev;

        // Calculate ATR
        let atr_series = atr::calculate_atr(high, low, close, atr_period)?;
        
        // Calculate Bollinger Bands
        let bb_result = bollinger_bands::calculate_bollinger_bands(close, bb_period, bb_std)?;

        Ok(VolatilityIndicators {
            atr: atr_series,
            bollinger_upper: bb_result.upper,
            bollinger_middle: bb_result.middle,
            bollinger_lower: bb_result.lower,
        })
    }

    /// Calculate volume indicators
    pub fn calculate_volume_indicators(
        &mut self,
        df: &DataFrame,
    ) -> Result<VolumeIndicators, HybridError> {
        let high = df.column("high")?;
        let low = df.column("low")?;
        let close = df.column("close")?;
        let volume = df.column("volume")?;

        let vwap_period = self.config.vwap_period;

        // Calculate VWAP
        let vwap_series = vwap::calculate_vwap(high, low, close, volume, vwap_period)?;
        
        // Calculate volume VWAP
        let volume_vwap_series = volume_vwap::calculate_vwap(high, low, close, volume, vwap_period)?;

        Ok(VolumeIndicators {
            vwap: vwap_series,
            volume_vwap: volume_vwap_series,
        })
    }

    /// Calculate all technical indicators
    pub fn calculate_all_indicators(
        &mut self,
        df: &DataFrame,
    ) -> Result<TechnicalIndicatorsResult, HybridError> {
        let momentum = self.calculate_momentum_indicators(df)?;
        let trend = self.calculate_trend_indicators(df)?;
        let volatility = self.calculate_volatility_indicators(df)?;
        let volume = self.calculate_volume_indicators(df)?;

        Ok(TechnicalIndicatorsResult {
            momentum,
            trend,
            volatility,
            volume,
        })
    }

    /// Generate feature matrix for forecasting
    pub fn generate_feature_matrix(
        &mut self,
        df: &DataFrame,
    ) -> Result<DataFrame, HybridError> {
        let indicators = self.calculate_all_indicators(df)?;
        
        let mut columns = Vec::new();
        
        // Add original price data
        columns.push(df.column("open")?.clone());
        columns.push(df.column("high")?.clone());
        columns.push(df.column("low")?.clone());
        columns.push(df.column("close")?.clone());
        columns.push(df.column("volume")?.clone());

        // Add momentum indicators
        columns.push(indicators.momentum.rsi.clone());
        columns.push(indicators.momentum.cci.clone());
        columns.push(indicators.momentum.mfi.clone());
        columns.push(indicators.momentum.roc.clone());

        // Add trend indicators
        columns.push(indicators.trend.sma_short.clone());
        columns.push(indicators.trend.sma_long.clone());
        columns.push(indicators.trend.ema_short.clone());
        columns.push(indicators.trend.ema_long.clone());
        columns.push(indicators.trend.sma_crossover.clone());
        columns.push(indicators.trend.ema_crossover.clone());

        // Add volatility indicators
        columns.push(indicators.volatility.atr.clone());
        columns.push(indicators.volatility.bollinger_upper.clone());
        columns.push(indicators.volatility.bollinger_middle.clone());
        columns.push(indicators.volatility.bollinger_lower.clone());

        // Add volume indicators
        columns.push(indicators.volume.vwap.clone());
        columns.push(indicators.volume.volume_vwap.clone());

        let feature_df = DataFrame::new(columns)?;
        
        // Cache the result
        self.cache.insert("feature_matrix".to_string(), feature_df.clone().into());
        
        Ok(feature_df)
    }

    /// Calculate RSI with caching
    fn calculate_rsi(&mut self, close: &Series, period: usize) -> Result<Series, HybridError> {
        let cache_key = format!("rsi_{}", period);
        
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        // Simple RSI calculation
        let mut rsi = Vec::with_capacity(close.len());
        
        for i in 0..close.len() {
            if i < period {
                rsi.push(f64::NAN);
                continue;
            }

            let mut gains = 0.0;
            let mut losses = 0.0;

            for j in (i - period + 1)..=i {
                if j > 0 {
                    let change = close.f64()?.get(j).unwrap_or(0.0) - close.f64()?.get(j - 1).unwrap_or(0.0);
                    if change > 0.0 {
                        gains += change;
                    } else {
                        losses += change.abs();
                    }
                }
            }

            let avg_gain = gains / period as f64;
            let avg_loss = losses / period as f64;

            if avg_loss == 0.0 {
                rsi.push(100.0);
            } else {
                let rs = avg_gain / avg_loss;
                let rsi_value = 100.0 - (100.0 / (1.0 + rs));
                rsi.push(rsi_value);
            }
        }

        let rsi_series = Series::new("rsi", rsi);
        self.cache.insert(cache_key, rsi_series.clone());
        
        Ok(rsi_series)
    }

    /// Calculate crossover signal
    fn calculate_crossover_signal(
        &self,
        short: &Series,
        long: &Series,
    ) -> Result<Series, HybridError> {
        let mut signal = Vec::with_capacity(short.len());
        
        for i in 0..short.len() {
            if i == 0 {
                signal.push(0.0);
                continue;
            }

            let short_prev = short.f64()?.get(i - 1).unwrap_or(f64::NAN);
            let long_prev = long.f64()?.get(i - 1).unwrap_or(f64::NAN);
            let short_curr = short.f64()?.get(i).unwrap_or(f64::NAN);
            let long_curr = long.f64()?.get(i).unwrap_or(f64::NAN);

            if short_prev.is_nan() || long_prev.is_nan() || short_curr.is_nan() || long_curr.is_nan() {
                signal.push(0.0);
                continue;
            }

            // Bullish crossover
            if short_prev <= long_prev && short_curr > long_curr {
                signal.push(1.0);
            }
            // Bearish crossover
            else if short_prev >= long_prev && short_curr < long_curr {
                signal.push(-1.0);
            }
            // No crossover
            else {
                signal.push(0.0);
            }
        }

        Ok(Series::new("crossover", signal))
    }

    /// Get cached indicator
    pub fn get_cached(&self, key: &str) -> Option<&Series> {
        self.cache.get(key)
    }

    /// Clear cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;

    fn create_test_dataframe() -> DataFrame {
        let dates = (0..100).map(|i| i as i64).collect::<Vec<_>>();
        let opens = (0..100).map(|i| 100.0 + i as f64 * 0.1).collect::<Vec<_>>();
        let highs = (0..100).map(|i| 102.0 + i as f64 * 0.1).collect::<Vec<_>>();
        let lows = (0..100).map(|i| 98.0 + i as f64 * 0.1).collect::<Vec<_>>();
        let closes = (0..100).map(|i| 101.0 + i as f64 * 0.1).collect::<Vec<_>>();
        let volumes = (0..100).map(|i| 1000000 + i * 1000).collect::<Vec<_>>();

        DataFrame::new(vec![
            Series::new("date", dates),
            Series::new("open", opens),
            Series::new("high", highs),
            Series::new("low", lows),
            Series::new("close", closes),
            Series::new("volume", volumes),
        ]).unwrap()
    }

    #[test]
    fn test_technical_indicators_creation() {
        let config = TechnicalConfig::default();
        let indicators = TechnicalIndicators::new(config);
        assert_eq!(indicators.cache.len(), 0);
    }

    #[test]
    fn test_momentum_indicators_calculation() {
        let config = TechnicalConfig::default();
        let mut indicators = TechnicalIndicators::new(config);
        let df = create_test_dataframe();

        let result = indicators.calculate_momentum_indicators(&df);
        assert!(result.is_ok());

        let momentum = result.unwrap();
        assert_eq!(momentum.rsi.len(), 100);
        assert_eq!(momentum.cci.len(), 100);
        assert_eq!(momentum.mfi.len(), 100);
        assert_eq!(momentum.roc.len(), 100);
    }

    #[test]
    fn test_trend_indicators_calculation() {
        let config = TechnicalConfig::default();
        let mut indicators = TechnicalIndicators::new(config);
        let df = create_test_dataframe();

        let result = indicators.calculate_trend_indicators(&df);
        assert!(result.is_ok());

        let trend = result.unwrap();
        assert_eq!(trend.sma_short.len(), 100);
        assert_eq!(trend.sma_long.len(), 100);
        assert_eq!(trend.ema_short.len(), 100);
        assert_eq!(trend.ema_long.len(), 100);
    }

    #[test]
    fn test_volatility_indicators_calculation() {
        let config = TechnicalConfig::default();
        let mut indicators = TechnicalIndicators::new(config);
        let df = create_test_dataframe();

        let result = indicators.calculate_volatility_indicators(&df);
        assert!(result.is_ok());

        let volatility = result.unwrap();
        assert_eq!(volatility.atr.len(), 100);
        assert_eq!(volatility.bollinger_upper.len(), 100);
        assert_eq!(volatility.bollinger_middle.len(), 100);
        assert_eq!(volatility.bollinger_lower.len(), 100);
    }

    #[test]
    fn test_volume_indicators_calculation() {
        let config = TechnicalConfig::default();
        let mut indicators = TechnicalIndicators::new(config);
        let df = create_test_dataframe();

        let result = indicators.calculate_volume_indicators(&df);
        assert!(result.is_ok());

        let volume = result.unwrap();
        assert_eq!(volume.vwap.len(), 100);
        assert_eq!(volume.volume_vwap.len(), 100);
    }

    #[test]
    fn test_feature_matrix_generation() {
        let config = TechnicalConfig::default();
        let mut indicators = TechnicalIndicators::new(config);
        let df = create_test_dataframe();

        let result = indicators.generate_feature_matrix(&df);
        assert!(result.is_ok());

        let feature_df = result.unwrap();
        assert_eq!(feature_df.shape(), (100, 18)); // 5 price + 4 momentum + 6 trend + 4 volatility + 2 volume
    }

    #[test]
    fn test_cache_functionality() {
        let config = TechnicalConfig::default();
        let mut indicators = TechnicalIndicators::new(config);
        let df = create_test_dataframe();

        // First calculation should populate cache
        let _ = indicators.calculate_momentum_indicators(&df);
        assert!(indicators.cache.len() > 0);

        // Second calculation should use cache
        let _ = indicators.calculate_momentum_indicators(&df);
        
        // Clear cache
        indicators.clear_cache();
        assert_eq!(indicators.cache.len(), 0);
    }
} 