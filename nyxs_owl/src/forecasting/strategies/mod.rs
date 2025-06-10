//! Forecasting strategies module
//!
//! This module contains implementations of various forecasting-based trading strategies,
//! including ARIMA, Exponential Smoothing, Ensemble Methods, Kalman Filters, GARCH,
//! Copula Models, and Regime-Switching Models.

pub mod adaptive_ensemble;
pub mod arima_strategy;
pub mod copula_strategy;
pub mod ensemble_strategy;
pub mod exponential_smoothing;
pub mod garch_strategy;
pub mod kalman_strategy;
pub mod regime_switching_strategy;

// Re-export main strategy structs for easier access
pub use adaptive_ensemble::*;
pub use arima_strategy::{ArimaStrategy, ArimaStrategyConfig};
pub use copula_strategy::{CopulaStrategy, CopulaStrategyConfig, CopulaStrategyType, CopulaType};
pub use ensemble_strategy::{EnsembleMethod, EnsembleStrategy, EnsembleStrategyConfig};
pub use exponential_smoothing::{ExponentialSmoothingConfig, ExponentialSmoothingStrategy};
pub use garch_strategy::{GarchStrategy, GarchStrategyConfig, GarchType};
pub use kalman_strategy::{KalmanStrategy, KalmanStrategyConfig};
pub use regime_switching_strategy::{
    MarketRegime, RegimeSwitchingConfig, RegimeSwitchingStrategy, RegimeSwitchingType,
};
