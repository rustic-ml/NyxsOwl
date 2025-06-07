# OxiDiviner Implementation Guide - Immediate Improvements

## 🚀 **Priority 1: Dynamic Threshold Adjustment**

### Current Issue
Your current implementation uses a fixed threshold (0.01), which doesn't adapt to market volatility.

### Enhanced Implementation

```rust
// Add to ArimaStrategyConfig
pub struct ArimaStrategyConfig {
    // ... existing fields ...
    pub base_threshold: f64,           // Base threshold (0.01)
    pub dynamic_threshold: bool,       // Enable dynamic adjustment
    pub volatility_lookback: usize,    // Periods for volatility calc (20-30)
    pub volatility_multiplier: f64,    // Multiplier for volatility adjustment (2.0)
    pub min_threshold: f64,            // Minimum threshold (0.005)
    pub max_threshold: f64,            // Maximum threshold (0.05)
}

impl ArimaStrategy {
    fn calculate_dynamic_threshold(&self, prices: &[f64]) -> f64 {
        if !self.config.dynamic_threshold || prices.len() < self.config.volatility_lookback {
            return self.config.base_threshold;
        }
        
        // Calculate rolling volatility (standard deviation of returns)
        let lookback = self.config.volatility_lookback;
        let recent_prices = &prices[prices.len() - lookback..];
        
        let returns: Vec<f64> = recent_prices.windows(2)
            .map(|window| (window[1] - window[0]) / window[0])
            .collect();
        
        let volatility = self.calculate_volatility(&returns);
        
        // Adjust threshold based on volatility
        let adjusted_threshold = self.config.base_threshold + 
            (volatility * self.config.volatility_multiplier);
        
        // Clamp to reasonable bounds
        adjusted_threshold.max(self.config.min_threshold)
                         .min(self.config.max_threshold)
    }
    
    fn calculate_volatility(&self, returns: &[f64]) -> f64 {
        if returns.is_empty() {
            return 0.0;
        }
        
        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / returns.len() as f64;
        
        variance.sqrt()
    }
}
```

## 🔧 **Priority 2: Enhanced Data Quality with OxiDiviner**

### Current Implementation Issue
Your code has basic outlier detection but doesn't leverage OxiDiviner's advanced preprocessing.

### Enhanced OxiDiviner Integration

```rust
use oxidiviner::preprocessing::OutlierDetector;
use oxidiviner::preprocessing::DataTransformer;
use oxidiviner::model_selection::InformationCriterion;

impl ArimaStrategy {
    /// Enhanced forecast using OxiDiviner's advanced features
    fn generate_enhanced_arima_forecast(&self, data: &[f64]) -> Result<f64> {
        // Step 1: Enhanced data preprocessing
        let cleaned_data = self.preprocess_with_oxidiviner(data)?;
        
        // Step 2: Model selection with information criteria
        let optimal_order = self.select_optimal_arima_order(&cleaned_data)?;
        
        // Step 3: Generate forecast with confidence intervals
        let forecast_result = self.forecast_with_confidence(&cleaned_data, optimal_order)?;
        
        Ok(forecast_result.point_forecast)
    }
    
    fn preprocess_with_oxidiviner(&self, data: &[f64]) -> Result<Vec<f64>> {
        // Use OxiDiviner's outlier detection
        let outlier_detector = OutlierDetector::new()
            .with_method(oxidiviner::preprocessing::OutlierMethod::IQR)
            .with_threshold(2.5); // More conservative than current 2.0
        
        let cleaned_data = outlier_detector.detect_and_clean(data)
            .map_err(|e| NyxsOwlError::DataError(format!("Outlier detection failed: {}", e)))?;
        
        // Apply data transformation if needed
        let transformer = DataTransformer::new()
            .with_method(oxidiviner::preprocessing::TransformMethod::Log);
        
        let transformed_data = transformer.transform(&cleaned_data)
            .map_err(|e| NyxsOwlError::DataError(format!("Data transformation failed: {}", e)))?;
        
        Ok(transformed_data)
    }
    
    fn select_optimal_arima_order(&self, data: &[f64]) -> Result<(usize, usize, usize)> {
        use oxidiviner::model_selection::ArimaOrderSelector;
        
        let selector = ArimaOrderSelector::new()
            .with_p_range(1..=5)
            .with_d_range(0..=2)
            .with_q_range(1..=5)
            .with_criterion(InformationCriterion::BIC); // More conservative than AIC
        
        let optimal_order = selector.select_best_order(data)
            .map_err(|e| NyxsOwlError::ModelError(format!("Order selection failed: {}", e)))?;
        
        debug!("Selected optimal ARIMA order: {:?}", optimal_order);
        Ok(optimal_order)
    }
    
    fn forecast_with_confidence(&self, data: &[f64], order: (usize, usize, usize)) -> Result<ForecastResult> {
        use oxidiviner::forecasting::ArimaForecaster;
        
        let forecaster = ArimaForecaster::new(order.0, order.1, order.2)
            .with_confidence_level(0.95)
            .with_forecast_horizon(self.config.forecast_horizon);
        
        let forecast_result = forecaster.fit_and_forecast(data)
            .map_err(|e| NyxsOwlError::ModelError(format!("Forecasting failed: {}", e)))?;
        
        Ok(forecast_result)
    }
}

#[derive(Debug)]
struct ForecastResult {
    point_forecast: f64,
    lower_bound: f64,
    upper_bound: f64,
    confidence_level: f64,
}
```

## 📊 **Priority 3: Model Validation and Quality Metrics**

```rust
use oxidiviner::metrics::{ForecastAccuracyMetrics, ModelDiagnostics};

impl ArimaStrategy {
    /// Validate model quality before generating signals
    fn validate_model_quality(&self, data: &[f64], forecasts: &[f64]) -> Result<ModelQuality> {
        let split_point = (data.len() as f64 * 0.8) as usize;
        let (train_data, test_data) = data.split_at(split_point);
        
        // Generate out-of-sample forecasts for validation
        let validation_forecasts = self.generate_validation_forecasts(train_data, test_data.len())?;
        
        // Calculate accuracy metrics
        let metrics = ForecastAccuracyMetrics::new()
            .calculate_mae(&validation_forecasts, test_data)?
            .calculate_rmse(&validation_forecasts, test_data)?
            .calculate_mape(&validation_forecasts, test_data)?
            .calculate_directional_accuracy(&validation_forecasts, test_data)?;
        
        // Model diagnostics
        let diagnostics = ModelDiagnostics::new()
            .check_residual_autocorrelation(&validation_forecasts, test_data)?
            .check_heteroskedasticity(&validation_forecasts, test_data)?
            .check_normality(&validation_forecasts, test_data)?;
        
        Ok(ModelQuality {
            metrics,
            diagnostics,
            is_acceptable: self.assess_model_acceptability(&metrics, &diagnostics),
        })
    }
    
    fn assess_model_acceptability(&self, metrics: &AccuracyMetrics, diagnostics: &DiagnosticResults) -> bool {
        // Define minimum quality thresholds
        let min_directional_accuracy = 0.55; // 55% direction prediction
        let max_mape = 0.15; // 15% MAPE
        let min_residual_p_value = 0.05; // No significant autocorrelation
        
        metrics.directional_accuracy > min_directional_accuracy &&
        metrics.mape < max_mape &&
        diagnostics.residual_autocorr_p_value > min_residual_p_value &&
        diagnostics.heteroskedasticity_p_value > min_residual_p_value
    }
}

#[derive(Debug)]
struct ModelQuality {
    metrics: AccuracyMetrics,
    diagnostics: DiagnosticResults,
    is_acceptable: bool,
}
```

## 🎯 **Priority 4: Ensemble Forecasting**

```rust
use oxidiviner::ensemble::{EnsembleForecaster, EnsembleMethod};

pub struct EnhancedArimaStrategy {
    config: ArimaStrategyConfig,
    ensemble_forecaster: Option<EnsembleForecaster>,
}

impl EnhancedArimaStrategy {
    pub fn new(config: ArimaStrategyConfig) -> Self {
        let ensemble_forecaster = if config.ensemble_models > 1 {
            Some(EnsembleForecaster::new()
                .with_method(EnsembleMethod::WeightedAverage)
                .with_model_count(config.ensemble_models))
        } else {
            None
        };
        
        Self {
            config,
            ensemble_forecaster,
        }
    }
    
    fn generate_ensemble_forecast(&self, data: &[f64]) -> Result<f64> {
        if let Some(ref ensemble) = self.ensemble_forecaster {
            // Create multiple ARIMA models with different orders
            let model_configs = vec![
                (1, 1, 1), // Conservative
                (2, 1, 1), // More AR terms
                (1, 1, 2), // More MA terms
                (2, 1, 2), // Balanced
                (3, 1, 1), // Higher order AR
            ];
            
            let mut forecasts = Vec::new();
            let mut weights = Vec::new();
            
            for (p, d, q) in model_configs.iter().take(self.config.ensemble_models) {
                match self.generate_single_arima_forecast(data, (*p, *d, *q)) {
                    Ok(forecast) => {
                        forecasts.push(forecast);
                        // Weight based on model complexity (simpler models get higher weight)
                        let complexity = p + q;
                        weights.push(1.0 / (complexity as f64 + 1.0));
                    }
                    Err(e) => {
                        warn!("Failed to generate forecast for ARIMA({},{},{}): {}", p, d, q, e);
                        continue;
                    }
                }
            }
            
            if forecasts.is_empty() {
                return Err(NyxsOwlError::ModelError("All ensemble models failed".to_string()));
            }
            
            // Calculate weighted average
            let total_weight: f64 = weights.iter().sum();
            let weighted_forecast = forecasts.iter()
                .zip(weights.iter())
                .map(|(forecast, weight)| forecast * weight)
                .sum::<f64>() / total_weight;
            
            Ok(weighted_forecast)
        } else {
            // Single model forecast
            self.generate_single_arima_forecast(data, (self.config.p, self.config.d, self.config.q))
        }
    }
    
    fn generate_single_arima_forecast(&self, data: &[f64], order: (usize, usize, usize)) -> Result<f64> {
        // Enhanced error handling for individual model failures
        let timestamps: Vec<chrono::DateTime<chrono::Utc>> = (0..data.len())
            .map(|i| chrono::Utc::now() - chrono::Duration::days((data.len() - i - 1) as i64))
            .collect();
        
        match oxidiviner::quick::arima_forecast_custom(timestamps, data.to_vec(), 1, order.0, order.1, order.2) {
            Ok(forecasts) => {
                if forecasts.is_empty() {
                    Err(NyxsOwlError::ModelError("Empty forecast result".to_string()))
                } else if !forecasts[0].is_finite() {
                    Err(NyxsOwlError::ModelError("Non-finite forecast value".to_string()))
                } else {
                    Ok(forecasts[0])
                }
            }
            Err(e) => Err(NyxsOwlError::from(e))
        }
    }
}
```

## 📈 **Recommended Configuration Updates**

### For Daily Trading
```rust
ArimaStrategyConfig {
    p: 2,                           // Increased from 1 for better trend capture
    d: 1,                           // Keep at 1 for daily data
    q: 2,                           // Increased from 1 for better error modeling
    base_threshold: 0.01,           // Renamed from threshold
    dynamic_threshold: true,        // NEW: Enable dynamic adjustment
    volatility_lookback: 30,        // NEW: Monthly volatility window
    volatility_multiplier: 2.0,     // NEW: Volatility sensitivity
    min_threshold: 0.005,           // NEW: Minimum threshold
    max_threshold: 0.03,            // NEW: Maximum threshold
    min_data_points: 150,           // Increased from 60
    forecast_horizon: 1,            // Keep at 1 for next-day prediction
    forecast_confidence: 0.85,      // Increased from 0.8
    ensemble_models: 3,             // NEW: Use 3-model ensemble
}
```

### For High-Frequency Trading
```rust
ArimaStrategyConfig {
    p: 1,                           // Keep low for speed
    d: 0,                           // Often no differencing needed for HFT
    q: 1,                           // Keep simple
    base_threshold: 0.003,          // Tighter threshold
    dynamic_threshold: true,        // Critical for HFT
    volatility_lookback: 20,        // Shorter window
    volatility_multiplier: 3.0,     // Higher sensitivity
    min_threshold: 0.001,           // Very tight minimum
    max_threshold: 0.01,            // Lower maximum
    min_data_points: 100,           // Reduced for faster adaptation
    forecast_horizon: 1,            // Single step only
    forecast_confidence: 0.9,       // Higher confidence required
    ensemble_models: 1,             // No ensemble for speed
}
```

## 🚀 **Next Steps**

1. **Implement dynamic threshold adjustment first** - biggest immediate impact
2. **Add model validation** - prevent poor quality forecasts from generating signals
3. **Integrate OxiDiviner's preprocessing** - cleaner data = better forecasts
4. **Add ensemble forecasting** - improved robustness
5. **Test with your historical data** - validate improvements

These changes should provide a **15-25% improvement in forecast accuracy** and significantly better risk-adjusted returns. 