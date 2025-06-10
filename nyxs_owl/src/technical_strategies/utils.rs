use crate::technical_strategies::{TechnicalSignal, CombinationMethod};
use crate::simple_types::Signal;
use polars::prelude::*;
use std::collections::HashMap;

/// Utility functions for technical strategy analysis
pub struct TechnicalUtils;

impl TechnicalUtils {
    /// Validate that a DataFrame contains required columns for technical analysis
    pub fn validate_ohlcv_data(data: &DataFrame) -> PolarsResult<()> {
        let required_columns = ["open", "high", "low", "close", "volume"];
        
        for col in required_columns {
            if data.column(col).is_err() {
                return Err(PolarsError::ComputeError(
                    format!("Missing required column: {}", col).into(),
                ));
            }
        }
        
        Ok(())
    }

    /// Calculate True Range for ATR and volatility calculations
    pub fn calculate_true_range(data: &DataFrame) -> PolarsResult<Series> {
        let high = data.column("high")?.f64()?;
        let low = data.column("low")?.f64()?;
        let close = data.column("close")?.f64()?;
        
        let close_shifted = close.shift(1);
        
        let tr_values: Vec<Option<f64>> = high
            .into_iter()
            .zip(low.into_iter())
            .zip(close.into_iter())
            .zip(close_shifted.f64()?.into_iter())
            .map(|(((h, l), _c), prev_c)| {
                match (h, l, prev_c) {
                    (Some(high_val), Some(low_val), Some(prev_close)) => {
                        let hl = high_val - low_val;
                        let hc = (high_val - prev_close).abs();
                        let lc = (low_val - prev_close).abs();
                        Some(hl.max(hc).max(lc))
                    }
                    (Some(high_val), Some(low_val), None) => Some(high_val - low_val), // First row
                    _ => None,
                }
            })
            .collect();
        
        Ok(Series::new("true_range".into(), tr_values))
    }

    /// Calculate Simple Moving Average
    pub fn calculate_sma(series: &Series, period: usize) -> PolarsResult<Series> {
        let values = series.f64()?;
        let mut sma_values = Vec::with_capacity(values.len());
        
        for i in 0..values.len() {
            if i < period - 1 {
                sma_values.push(None);
            } else {
                let sum: f64 = values.slice(i + 1 - period, period)
                    .into_iter()
                    .filter_map(|x| x)
                    .sum();
                let count = values.slice(i + 1 - period, period)
                    .into_iter()
                    .filter_map(|x| x)
                    .count();
                
                if count == period {
                    sma_values.push(Some(sum / period as f64));
                } else {
                    sma_values.push(None);
                }
            }
        }
        
        Ok(Series::new(format!("sma_{}", period).into(), sma_values))
    }

    /// Calculate Exponential Moving Average
    pub fn calculate_ema(series: &Series, period: usize) -> PolarsResult<Series> {
        let alpha = 2.0 / (period as f64 + 1.0);
        let values = series.f64()?;
        let mut ema_values = Vec::with_capacity(values.len());
        let mut ema = None;
        
        for value in values.into_iter() {
            if let Some(val) = value {
                match ema {
                    None => {
                        ema = Some(val);
                        ema_values.push(Some(val));
                    }
                    Some(prev_ema) => {
                        let new_ema = alpha * val + (1.0 - alpha) * prev_ema;
                        ema = Some(new_ema);
                        ema_values.push(Some(new_ema));
                    }
                }
            } else {
                ema_values.push(None);
            }
        }
        
        Ok(Series::new(format!("ema_{}", period).into(), ema_values))
    }

    /// Calculate Volume Weighted Average Price (VWAP)
    pub fn calculate_vwap(data: &DataFrame) -> PolarsResult<Series> {
        let high = data.column("high")?.f64()?;
        let low = data.column("low")?.f64()?;
        let close = data.column("close")?.f64()?;
        let volume = data.column("volume")?.f64()?;
        
        let mut cumulative_pv = 0.0;
        let mut cumulative_volume = 0.0;
        let mut vwap_values = Vec::new();
        
        for (((h, l), c), v) in high.into_iter()
            .zip(low.into_iter())
            .zip(close.into_iter())
            .zip(volume.into_iter())
        {
            match (h, l, c, v) {
                (Some(high_val), Some(low_val), Some(close_val), Some(vol_val)) => {
                    let typical_price = (high_val + low_val + close_val) / 3.0;
                    cumulative_pv += typical_price * vol_val;
                    cumulative_volume += vol_val;
                    
                    let vwap = if cumulative_volume > 0.0 {
                        cumulative_pv / cumulative_volume
                    } else {
                        typical_price
                    };
                    
                    vwap_values.push(Some(vwap));
                }
                _ => vwap_values.push(None),
            }
        }
        
        Ok(Series::new("vwap".into(), vwap_values))
    }

    /// Calculate percentage change between two values
    pub fn percentage_change(from: f64, to: f64) -> f64 {
        if from != 0.0 {
            ((to - from) / from) * 100.0
        } else {
            0.0
        }
    }

    /// Normalize a value to a 0-1 range using min-max normalization
    pub fn normalize_value(value: f64, min: f64, max: f64) -> f64 {
        if max != min {
            (value - min) / (max - min)
        } else {
            0.5 // Default to middle if no range
        }
    }

    /// Clean and filter signals based on criteria
    pub fn filter_signals(
        signals: &[TechnicalSignal],
        min_strength: f64,
        min_confidence: f64,
    ) -> Vec<TechnicalSignal> {
        signals
            .iter()
            .filter(|signal| {
                signal.strength.abs() >= min_strength
                    && signal.confidence >= min_confidence
            })
            .cloned()
            .collect()
    }

    /// Combine multiple signals using different methods
    pub fn combine_signals(
        signals: &[Vec<TechnicalSignal>],
        method: CombinationMethod,
    ) -> Vec<TechnicalSignal> {
        if signals.is_empty() {
            return Vec::new();
        }
        
        let max_len = signals.iter().map(|s| s.len()).max().unwrap_or(0);
        let mut combined = Vec::with_capacity(max_len);
        
        for i in 0..max_len {
            let current_signals: Vec<&TechnicalSignal> = signals
                .iter()
                .filter_map(|s| s.get(i))
                .collect();
            
            if current_signals.is_empty() {
                continue;
            }
            
            let combined_signal = match method {
                CombinationMethod::WeightedAverage => {
                    let total_weight: f64 = current_signals.iter().map(|s| s.confidence).sum();
                    
                    if total_weight > 0.0 {
                        let weighted_strength: f64 = current_signals
                            .iter()
                            .map(|s| s.strength * s.confidence)
                            .sum::<f64>() / total_weight;
                        
                        let avg_confidence: f64 = current_signals
                            .iter()
                            .map(|s| s.confidence)
                            .sum::<f64>() / current_signals.len() as f64;
                        
                        let mut metadata = HashMap::new();
                        metadata.insert("source".to_string(), "combined_weighted".to_string());
                        metadata.insert("signal_count".to_string(), current_signals.len() as f64);
                        
                        TechnicalSignal {
                            signal: if weighted_strength > 0.1 {
                                Signal::Buy
                            } else if weighted_strength < -0.1 {
                                Signal::Sell
                            } else {
                                Signal::Hold
                            },
                            strength: weighted_strength,
                            confidence: avg_confidence,
                            metadata,
                        }
                    } else {
                        continue;
                    }
                }
                CombinationMethod::Consensus => {
                    let positive_signals = current_signals.iter().filter(|s| s.strength > 0.0).count();
                    let negative_signals = current_signals.iter().filter(|s| s.strength < 0.0).count();
                    
                    let consensus_strength = if positive_signals > negative_signals {
                        current_signals
                            .iter()
                            .filter(|s| s.strength > 0.0)
                            .map(|s| s.strength)
                            .sum::<f64>() / positive_signals as f64
                    } else if negative_signals > positive_signals {
                        current_signals
                            .iter()
                            .filter(|s| s.strength < 0.0)
                            .map(|s| s.strength)
                            .sum::<f64>() / negative_signals as f64
                    } else {
                        0.0 // No consensus
                    };
                    
                    let consensus_confidence = if consensus_strength != 0.0 {
                        let majority_count = positive_signals.max(negative_signals);
                        majority_count as f64 / current_signals.len() as f64
                    } else {
                        0.0
                    };
                    
                    let mut metadata = HashMap::new();
                    metadata.insert("source".to_string(), "combined_consensus".to_string());
                    metadata.insert("positive_signals".to_string(), positive_signals as f64);
                    metadata.insert("negative_signals".to_string(), negative_signals as f64);
                    
                    TechnicalSignal {
                        signal: if consensus_strength > 0.1 {
                            Signal::Buy
                        } else if consensus_strength < -0.1 {
                            Signal::Sell
                        } else {
                            Signal::Hold
                        },
                        strength: consensus_strength,
                        confidence: consensus_confidence,
                        metadata,
                    }
                }
            };
            
            combined.push(combined_signal);
        }
        
        combined
    }

    /// Calculate signal accuracy against actual price movements
    pub fn calculate_signal_accuracy(
        signals: &[TechnicalSignal],
        price_returns: &[f64],
        threshold: f64,
    ) -> f64 {
        if signals.len() != price_returns.len() || signals.is_empty() {
            return 0.0;
        }
        
        let correct_predictions = signals
            .iter()
            .zip(price_returns.iter())
            .filter(|(signal, &return_val)| {
                (signal.strength > threshold && return_val > 0.0)
                    || (signal.strength < -threshold && return_val < 0.0)
                    || (signal.strength.abs() <= threshold && return_val.abs() <= threshold)
            })
            .count();
        
        correct_predictions as f64 / signals.len() as f64
    }

    /// Calculate support and resistance levels using pivot points
    pub fn calculate_pivot_levels(data: &DataFrame) -> PolarsResult<HashMap<String, f64>> {
        let len = data.height();
        if len == 0 {
            return Ok(HashMap::new());
        }
        
        let high = data.column("high")?.f64()?.get(len - 1).unwrap_or(0.0);
        let low = data.column("low")?.f64()?.get(len - 1).unwrap_or(0.0);
        let close = data.column("close")?.f64()?.get(len - 1).unwrap_or(0.0);
        
        let pivot = (high + low + close) / 3.0;
        let r1 = 2.0 * pivot - low;
        let s1 = 2.0 * pivot - high;
        let r2 = pivot + (high - low);
        let s2 = pivot - (high - low);
        let r3 = high + 2.0 * (pivot - low);
        let s3 = low - 2.0 * (high - pivot);
        
        let mut levels = HashMap::new();
        levels.insert("pivot".to_string(), pivot);
        levels.insert("r1".to_string(), r1);
        levels.insert("r2".to_string(), r2);
        levels.insert("r3".to_string(), r3);
        levels.insert("s1".to_string(), s1);
        levels.insert("s2".to_string(), s2);
        levels.insert("s3".to_string(), s3);
        
        Ok(levels)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simple_types::Signal;

    fn create_test_data() -> DataFrame {
        df! {
            "date" => ["2023-01-01", "2023-01-02", "2023-01-03", "2023-01-04", "2023-01-05"],
            "open" => [100.0, 102.0, 105.0, 103.0, 107.0],
            "high" => [103.0, 106.0, 108.0, 107.0, 110.0],
            "low" => [99.0, 101.0, 104.0, 102.0, 106.0],
            "close" => [102.0, 105.0, 103.0, 107.0, 109.0],
            "volume" => [1000.0, 1200.0, 800.0, 1500.0, 1100.0],
        }.unwrap()
    }

    #[test]
    fn test_validate_ohlcv_data() {
        let data = create_test_data();
        assert!(TechnicalUtils::validate_ohlcv_data(&data).is_ok());
    }

    #[test]
    fn test_calculate_true_range() {
        let data = create_test_data();
        let tr = TechnicalUtils::calculate_true_range(&data).unwrap();
        assert_eq!(tr.len(), 5);
        // First value should be high - low
        assert!((tr.f64().unwrap().get(0).unwrap() - 4.0).abs() < 0.001); // 103 - 99
    }

    #[test]
    fn test_percentage_change() {
        assert_eq!(TechnicalUtils::percentage_change(100.0, 110.0), 10.0);
        assert_eq!(TechnicalUtils::percentage_change(100.0, 90.0), -10.0);
        assert_eq!(TechnicalUtils::percentage_change(0.0, 100.0), 0.0);
    }

    #[test]
    fn test_normalize_value() {
        assert_eq!(TechnicalUtils::normalize_value(5.0, 0.0, 10.0), 0.5);
        assert_eq!(TechnicalUtils::normalize_value(0.0, 0.0, 10.0), 0.0);
        assert_eq!(TechnicalUtils::normalize_value(10.0, 0.0, 10.0), 1.0);
    }

    #[test]
    fn test_signal_filtering() {
        let signals = vec![
            TechnicalSignal {
                signal: Signal::Buy,
                strength: 0.8,
                confidence: 0.9,
                metadata: HashMap::new(),
            },
            TechnicalSignal {
                signal: Signal::Hold,
                strength: 0.3,
                confidence: 0.4,
                metadata: HashMap::new(),
            },
            TechnicalSignal {
                signal: Signal::Sell,
                strength: -0.7,
                confidence: 0.8,
                metadata: HashMap::new(),
            },
        ];
        
        let filtered = TechnicalUtils::filter_signals(&signals, 0.5, 0.7);
        assert_eq!(filtered.len(), 2); // Only first and third signals should pass
    }
}
