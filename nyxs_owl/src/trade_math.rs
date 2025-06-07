// Placeholder for the trade_math module. 

pub mod momentum;
pub mod volatility;
pub mod trend;
pub mod moving_averages;

pub use momentum::{calculate_macd, calculate_stochastic, calculate_rsi};
pub use volatility::{calculate_bollinger_bands, calculate_atr, calculate_ease_of_movement, calculate_volume_price_trend, calculate_obv};
pub use trend::{calculate_aroon, calculate_adx_di, calculate_aroon_oscillator, calculate_directional_movement_components, calculate_adxr, calculate_ichimoku_cloud, calculate_psar};
pub use moving_averages::{calculate_sma, calculate_ema, calculate_wma, calculate_vwap};

// TODO: Add other categories of math/indicator functions as modules
// e.g.:
// pub mod volume;
// pub mod oscillators; // (if not part of momentum, e.g. RSI might go here or its own module) 