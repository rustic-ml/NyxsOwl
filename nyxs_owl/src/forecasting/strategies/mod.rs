//! Forecasting strategy implementations
//!
//! This module provides various forecasting strategies including ARIMA, GARCH,
//! Kalman filters, and ensemble methods for financial time series prediction.

/// Adaptive ensemble forecasting combining multiple models
pub mod adaptive_ensemble;

/// ARIMA (AutoRegressive Integrated Moving Average) strategy
pub mod arima_strategy;

/// Copula-based forecasting for dependency modeling
pub mod copula_strategy;

/// Ensemble forecasting combining multiple prediction models
pub mod ensemble_strategy;

/// Exponential smoothing forecasting methods
pub mod exponential_smoothing;

/// GARCH (Generalized Autoregressive Conditional Heteroskedasticity) strategy
pub mod garch_strategy;

/// Kalman filter-based forecasting strategy
pub mod kalman_strategy;

/// Neural network-based forecasting strategy
pub mod neural_network_strategy;

/// Regime switching forecasting for market state changes
pub mod regime_switching_strategy;

// Re-export main strategy structs for easier access
pub use adaptive_ensemble::*;
pub use arima_strategy::{ArimaStrategy, ArimaStrategyConfig};
pub use copula_strategy::{CopulaStrategy, CopulaStrategyConfig, CopulaStrategyType, CopulaType};
pub use ensemble_strategy::{EnsembleMethod, EnsembleStrategy, EnsembleStrategyConfig};
pub use exponential_smoothing::{ExponentialSmoothingConfig, ExponentialSmoothingStrategy};
pub use garch_strategy::{GarchStrategy, GarchStrategyConfig, GarchType};
pub use kalman_strategy::{KalmanStrategy, KalmanStrategyConfig};
pub use neural_network_strategy::{NeuralNetworkStrategy, NeuralNetworkConfig};
pub use regime_switching_strategy::{
    MarketRegime, RegimeSwitchingConfig, RegimeSwitchingStrategy, RegimeSwitchingType,
};
