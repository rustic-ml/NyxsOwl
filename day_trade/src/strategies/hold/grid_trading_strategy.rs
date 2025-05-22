//! Grid Trading Strategy
//!
//! Places buy orders at predefined intervals below the current price
//! and sell orders at intervals above the current price
//! Profits from price oscillations in a range-bound market

use crate::{DailyOhlcv, Signal, TradeError, TradingStrategy};
use trade_math::volatility::AverageTrueRange;

/// Price level with status for grid trading
#[derive(Debug, Clone, Copy)]
struct GridLevel {
    /// Price level
    price: f64,
    /// Whether this level has been triggered
    triggered: bool,
    /// Whether this is a buy level (true) or sell level (false)
    is_buy_level: bool,
}

/// Grid trading strategy parameters
pub struct GridTradingStrategy {
    /// Number of grid levels above and below the reference price
    grid_levels: usize,
    /// Grid spacing as ATR multiplier (dynamic grid)
    grid_spacing_atr_multiplier: f64,
    /// ATR period for dynamic grid calculation
    atr_period: usize,
    /// Maximum position size as percentage of portfolio (0.0-1.0)
    max_position_size: f64,
}

impl GridTradingStrategy {
    /// Create a new grid trading strategy with given parameters
    pub fn new(
        grid_levels: usize,
        grid_spacing_atr_multiplier: f64,
        atr_period: usize,
        max_position_size: f64,
    ) -> Result<Self, String> {
        if grid_levels == 0 {
            return Err("Grid levels must be greater than zero".to_string());
        }

        if grid_spacing_atr_multiplier <= 0.0 {
            return Err("Grid spacing ATR multiplier must be positive".to_string());
        }

        if atr_period < 5 {
            return Err("ATR period must be at least 5".to_string());
        }

        if max_position_size <= 0.0 || max_position_size > 1.0 {
            return Err("Maximum position size must be between 0 and 1".to_string());
        }

        Ok(Self {
            grid_levels,
            grid_spacing_atr_multiplier,
            atr_period,
            max_position_size,
        })
    }

    /// Create a default grid trading strategy
    /// Uses grid_levels=5, grid_spacing_atr_multiplier=0.5, atr_period=14, max_position_size=0.2
    pub fn default() -> Self {
        Self {
            grid_levels: 5,
            grid_spacing_atr_multiplier: 0.5,
            atr_period: 14,
            max_position_size: 0.2,
        }
    }
}

impl TradingStrategy for GridTradingStrategy {
    fn generate_signals(&self, data: &[DailyOhlcv]) -> Result<Vec<Signal>, TradeError> {
        if data.len() < self.atr_period + 1 {
            return Err(TradeError::InsufficientData(format!(
                "Need at least {} data points for grid trading strategy",
                self.atr_period + 1
            )));
        }

        let mut signals = Vec::with_capacity(data.len());
        let mut atr = AverageTrueRange::new(self.atr_period)
            .map_err(|e| TradeError::CalculationError(e.to_string()))?;

        // For tracking grid levels and positions
        let mut grid_levels: Vec<GridLevel> = Vec::new();
        let mut position_size = 0.0;

        // Initialize ATR with initial data points without generating signals
        for i in 0..self.atr_period {
            atr.update(data[i].data.high, data[i].data.low, data[i].data.close)
                .map_err(|e| TradeError::CalculationError(e.to_string()))?;
            signals.push(Signal::Hold);
        }

        // Initialize the grid after we have enough data for ATR
        let mut reference_price = data[self.atr_period].data.close;
        let mut atr_value = atr
            .value()
            .map_err(|e| TradeError::CalculationError(e.to_string()))?;
        let grid_spacing = atr_value * self.grid_spacing_atr_multiplier;

        // Create initial grid levels
        grid_levels.clear();
        for i in 1..=self.grid_levels {
            let buy_level = reference_price - (i as f64 * grid_spacing);
            let sell_level = reference_price + (i as f64 * grid_spacing);

            grid_levels.push(GridLevel {
                price: buy_level,
                triggered: false,
                is_buy_level: true,
            });

            grid_levels.push(GridLevel {
                price: sell_level,
                triggered: false,
                is_buy_level: false,
            });
        }

        // Generate signals for the rest of the data
        for i in self.atr_period..data.len() {
            atr.update(data[i].data.high, data[i].data.low, data[i].data.close)
                .map_err(|e| TradeError::CalculationError(e.to_string()))?;

            // Update ATR value and recalculate grid every 5 periods
            if i % 5 == 0 {
                atr_value = atr
                    .value()
                    .map_err(|e| TradeError::CalculationError(e.to_string()))?;

                // Update reference price and grid spacing
                reference_price = data[i].data.close;
                let new_grid_spacing = atr_value * self.grid_spacing_atr_multiplier;

                // Reset grid levels
                grid_levels.clear();
                for j in 1..=self.grid_levels {
                    let buy_level = reference_price - (j as f64 * new_grid_spacing);
                    let sell_level = reference_price + (j as f64 * new_grid_spacing);

                    grid_levels.push(GridLevel {
                        price: buy_level,
                        triggered: false,
                        is_buy_level: true,
                    });

                    grid_levels.push(GridLevel {
                        price: sell_level,
                        triggered: false,
                        is_buy_level: false,
                    });
                }
            }

            // Check if price hit any grid levels
            let mut current_signal = Signal::Hold;
            let low = data[i].data.low;
            let high = data[i].data.high;

            // Sort grid levels by price for better decision making
            grid_levels.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());

            for level in &mut grid_levels {
                if !level.triggered {
                    if level.is_buy_level
                        && low <= level.price
                        && position_size < self.max_position_size
                    {
                        // Buy signal at buy grid level
                        level.triggered = true;
                        position_size += 1.0 / (self.grid_levels as f64 * 2.0);
                        current_signal = Signal::Buy;
                        break;
                    } else if !level.is_buy_level && high >= level.price && position_size > 0.0 {
                        // Sell signal at sell grid level
                        level.triggered = true;
                        position_size -= 1.0 / (self.grid_levels as f64 * 2.0);
                        current_signal = Signal::Sell;
                        break;
                    }
                }
            }

            signals.push(current_signal);
        }

        Ok(signals)
    }

    fn calculate_performance(
        &self,
        data: &[DailyOhlcv],
        signals: &[Signal],
    ) -> Result<f64, TradeError> {
        if data.len() != signals.len() {
            return Err(TradeError::InvalidData(
                "Data and signals arrays must be the same length".to_string(),
            ));
        }

        if data.len() <= 1 {
            return Err(TradeError::InsufficientData(
                "Need at least 2 data points to calculate performance".to_string(),
            ));
        }

        let initial_cash = 10000.0;
        let mut cash = initial_cash; // Initial cash
        let mut shares = 0.0; // Initial shares

        // For grid trading, we use smaller position sizes per trade
        let position_size = initial_cash * self.max_position_size / (2.0 * self.grid_levels as f64);

        for i in 1..data.len() {
            match signals[i - 1] {
                Signal::Buy => {
                    // Buy shares with fixed position size
                    if cash >= position_size {
                        let new_shares = position_size / data[i].data.open;
                        shares += new_shares;
                        cash -= position_size;
                    }
                }
                Signal::Sell => {
                    // Sell a portion of shares
                    if shares > 0.0 {
                        let shares_to_sell = shares / (2.0 * self.grid_levels as f64);
                        cash += shares_to_sell * data[i].data.open;
                        shares -= shares_to_sell;
                    }
                }
                Signal::Hold => {} // Do nothing
            }
        }

        // Calculate final portfolio value
        let final_value = cash + shares * data.last().unwrap().data.close;

        // Calculate performance as percent return
        let performance = (final_value / initial_cash - 1.0) * 100.0;

        Ok(performance)
    }
}
