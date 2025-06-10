// Placeholder for the trade_math module.

pub mod momentum;
pub mod moving_averages;
pub mod trend;
pub mod volatility;

pub use momentum::{calculate_macd, calculate_rsi, calculate_stochastic};
pub use moving_averages::{calculate_ema, calculate_sma, calculate_vwap, calculate_wma};
pub use trend::{
    calculate_adx_di, calculate_adxr, calculate_aroon, calculate_aroon_oscillator,
    calculate_directional_movement_components, calculate_ichimoku_cloud, calculate_psar,
};
pub use volatility::{
    calculate_atr, calculate_bollinger_bands, calculate_ease_of_movement, calculate_obv,
    calculate_volume_price_trend,
};

// TODO: Add other categories of math/indicator functions as modules
// e.g.:
// pub mod volume;
// pub mod oscillators; // (if not part of momentum, e.g. RSI might go here or its own module)
