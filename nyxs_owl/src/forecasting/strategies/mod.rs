//! Forecasting strategies module
//! 
//! This module contains implementations of various forecasting-based trading strategies,
//! including ARIMA, Exponential Smoothing, Ensemble Methods, Kalman Filters, GARCH, 
//! Copula Models, and Regime-Switching Models.

pub mod arima_strategy;
pub mod exponential_smoothing;
pub mod kalman_strategy;
pub mod ensemble_strategy;
pub mod garch_strategy;
pub mod copula_strategy;
pub mod regime_switching_strategy;
pub mod adaptive_ensemble;

// Re-export main strategy structs for easier access
pub use arima_strategy::{ArimaStrategy, ArimaStrategyConfig};
pub use exponential_smoothing::{ExponentialSmoothingStrategy, ExponentialSmoothingConfig};
pub use kalman_strategy::{KalmanStrategy, KalmanStrategyConfig};
pub use ensemble_strategy::{EnsembleStrategy, EnsembleStrategyConfig, EnsembleMethod};
pub use garch_strategy::{GarchStrategy, GarchStrategyConfig, GarchType};
pub use copula_strategy::{CopulaStrategy, CopulaStrategyConfig, CopulaType, CopulaStrategyType};
pub use regime_switching_strategy::{RegimeSwitchingStrategy, RegimeSwitchingConfig, RegimeSwitchingType, MarketRegime};
pub use adaptive_ensemble::*; 