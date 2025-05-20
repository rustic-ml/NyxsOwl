//! Mock implementations of indicators that interface with rustalib and oxidiviner
//! This provides adapters to the actual libraries

use crate::TradeError;

/// SimpleMovingAverage adapter for rustalib
pub struct SimpleMovingAverage {
    period: usize,
    prices: Vec<f64>,
}

impl SimpleMovingAverage {
    /// Create a new SMA with the specified period
    pub fn new(period: usize) -> Result<Self, String> {
        Ok(Self {
            period,
            prices: Vec::new(),
        })
    }

    /// Update the indicator with a new price
    pub fn update(&mut self, price: f64) -> Result<(), String> {
        self.prices.push(price);
        if self.prices.len() > self.period * 2 {
            self.prices.remove(0);
        }
        Ok(())
    }

    /// Get the current SMA value
    pub fn value(&self) -> Result<f64, String> {
        if self.prices.len() < self.period {
            return Err(format!("Not enough data points for SMA calculation"));
        }

        let sum: f64 = self.prices.iter().rev().take(self.period).sum();
        Ok(sum / self.period as f64)
    }
}

/// RelativeStrengthIndex adapter for rustalib
pub struct RelativeStrengthIndex {
    period: usize,
    prices: Vec<f64>,
    gains: Vec<f64>,
    losses: Vec<f64>,
}

impl RelativeStrengthIndex {
    /// Create a new RSI with the specified period
    pub fn new(period: usize) -> Result<Self, String> {
        Ok(Self {
            period,
            prices: Vec::new(),
            gains: Vec::new(),
            losses: Vec::new(),
        })
    }

    /// Update the indicator with a new price
    pub fn update(&mut self, price: f64) -> Result<(), String> {
        if !self.prices.is_empty() {
            let prev_price = *self.prices.last().unwrap();
            let change = price - prev_price;

            if change > 0.0 {
                self.gains.push(change);
                self.losses.push(0.0);
            } else {
                self.gains.push(0.0);
                self.losses.push(change.abs());
            }

            if self.gains.len() > self.period * 2 {
                self.gains.remove(0);
                self.losses.remove(0);
            }
        }

        self.prices.push(price);
        if self.prices.len() > self.period * 2 {
            self.prices.remove(0);
        }

        Ok(())
    }

    /// Get the current RSI value
    pub fn value(&self) -> Result<f64, String> {
        if self.prices.len() <= self.period || self.gains.len() < self.period {
            return Err(format!("Not enough data points for RSI calculation"));
        }

        let avg_gain: f64 =
            self.gains.iter().rev().take(self.period).sum::<f64>() / self.period as f64;
        let avg_loss: f64 =
            self.losses.iter().rev().take(self.period).sum::<f64>() / self.period as f64;

        if avg_loss == 0.0 {
            return Ok(100.0);
        }

        let rs = avg_gain / avg_loss;
        let rsi = 100.0 - (100.0 / (1.0 + rs));

        Ok(rsi)
    }
}

/// MACD adapter for rustalib
pub struct Macd {
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
    prices: Vec<f64>,
    macd_values: Vec<f64>,
}

impl Macd {
    /// Create a new MACD with the specified parameters
    pub fn new(
        fast_period: usize,
        slow_period: usize,
        signal_period: usize,
    ) -> Result<Self, String> {
        Ok(Self {
            fast_period,
            slow_period,
            signal_period,
            prices: Vec::new(),
            macd_values: Vec::new(),
        })
    }

    /// Update the indicator with a new price
    pub fn update(&mut self, price: f64) -> Result<(), String> {
        self.prices.push(price);

        // Calculate new MACD value if we have enough data
        if self.prices.len() > self.slow_period {
            let fast_ema = self.calculate_ema(&self.prices, self.fast_period)?;
            let slow_ema = self.calculate_ema(&self.prices, self.slow_period)?;
            let macd_value = fast_ema - slow_ema;
            self.macd_values.push(macd_value);
        }

        // Trim older values to save memory
        if self.prices.len() > self.slow_period * 2 {
            self.prices.remove(0);
        }

        if self.macd_values.len() > self.signal_period * 2 {
            self.macd_values.remove(0);
        }

        Ok(())
    }

    /// Calculate EMA for a given period
    fn calculate_ema(&self, prices: &[f64], period: usize) -> Result<f64, String> {
        if prices.len() < period {
            return Err(format!("Not enough data for EMA calculation"));
        }

        // Multiplier: (2 / (period + 1))
        let multiplier = 2.0 / (period as f64 + 1.0);

        // Calculate SMA for the initial EMA value
        let mut sum = 0.0;
        for i in 0..period {
            sum += prices[prices.len() - period + i];
        }
        let sma = sum / period as f64;

        // Calculate EMA
        let mut ema = sma;
        for i in prices.len() - period..prices.len() {
            ema = (prices[i] - ema) * multiplier + ema;
        }

        Ok(ema)
    }

    /// Get the current MACD value
    pub fn macd_value(&self) -> Result<f64, String> {
        if self.macd_values.len() < 1 {
            return Err(format!("Not enough data for MACD calculation"));
        }

        Ok(*self.macd_values.last().unwrap())
    }

    /// Get the current signal line value
    pub fn signal_value(&self) -> Result<f64, String> {
        if self.macd_values.len() < self.signal_period {
            return Err(format!("Not enough data for signal line calculation"));
        }

        // Multiplier: (2 / (period + 1))
        let multiplier = 2.0 / (self.signal_period as f64 + 1.0);

        // Calculate SMA for the initial EMA value
        let mut sum = 0.0;
        for i in 0..self.signal_period {
            sum += self.macd_values[self.macd_values.len() - self.signal_period + i];
        }
        let sma = sum / self.signal_period as f64;

        // Calculate EMA
        let mut ema = sma;
        for i in self.macd_values.len() - self.signal_period..self.macd_values.len() {
            ema = (self.macd_values[i] - ema) * multiplier + ema;
        }

        Ok(ema)
    }
}

/// TimeSeriesPredictor adapter for oxidiviner
pub struct TimeSeriesPredictor {
    horizon: usize,
    embedding_dim: usize,
    use_ma: bool,
    data: Vec<f64>,
}

impl TimeSeriesPredictor {
    /// Create a new TimeSeriesPredictor
    pub fn new(horizon: usize, embedding_dim: usize, use_ma: bool) -> Result<Self, String> {
        Ok(Self {
            horizon,
            embedding_dim,
            use_ma,
            data: Vec::new(),
        })
    }

    /// Forecast future prices based on provided data
    pub fn forecast(&self, prices: &[f64]) -> Result<Vec<f64>, TradeError> {
        if prices.len() < self.embedding_dim + self.horizon {
            return Err(TradeError::InsufficientData(format!(
                "Need at least {} data points for forecasting",
                self.embedding_dim + self.horizon
            )));
        }

        // Simple trend-following forecast
        let mut result = Vec::with_capacity(self.horizon);
        let trend = self.calculate_trend(prices);

        let last_price = *prices.last().unwrap();
        for i in 0..self.horizon {
            result.push(last_price + trend * (i + 1) as f64);
        }

        Ok(result)
    }

    /// Calculate the trend in the price data
    fn calculate_trend(&self, prices: &[f64]) -> f64 {
        if prices.len() < 2 {
            return 0.0;
        }

        let window_size = self.embedding_dim.min(prices.len() / 2);
        let recent_prices = &prices[prices.len() - window_size..];

        let first = recent_prices[0];
        let last = recent_prices[recent_prices.len() - 1];

        (last - first) / window_size as f64
    }
}
