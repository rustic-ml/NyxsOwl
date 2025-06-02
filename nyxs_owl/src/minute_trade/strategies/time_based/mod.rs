//! Time-based intraday trading strategies
//!
//! This module will contain strategies that trade based on specific times of the day.

// Re-export strategies
pub use self::session_transition_strategy::SessionTransitionStrategy;
pub use self::time_of_day_strategy::TimeOfDayStrategy;

// These will be implemented in the future
mod time_of_day_strategy {
    use crate::minute_trade::utils::calculate_basic_performance;
    use crate::minute_trade::{IntradayStrategy, MinuteOhlcv, Signal, TradeError};
    use chrono::{DateTime, Timelike, Utc};

    /// TimeOfDay strategy for trading specific times of the trading day
    #[derive(Debug, Clone)]
    pub struct TimeOfDayStrategy {
        /// Entry time (hour of day, 0-23)
        entry_hour: u32,
        /// Entry minute (0-59)
        entry_minute: u32,
        /// Exit time (hour of day, 0-23)
        exit_hour: u32,
        /// Exit minute (0-59)
        exit_minute: u32,
        /// Direction to trade (true for long, false for short)
        go_long: bool,
        /// Strategy name
        name: String,
    }

    impl TimeOfDayStrategy {
        /// Create a new Time of Day strategy
        ///
        /// # Arguments
        ///
        /// * `entry_hour` - Hour to enter trade (0-23)
        /// * `entry_minute` - Minute to enter trade (0-59)
        /// * `exit_hour` - Hour to exit trade (0-23)
        /// * `exit_minute` - Minute to exit trade (0-59)
        /// * `go_long` - Direction to trade (true for long, false for short)
        ///
        /// # Returns
        ///
        /// * `Result<Self, String>` - New strategy instance or error message
        pub fn new(
            entry_hour: u32,
            entry_minute: u32,
            exit_hour: u32,
            exit_minute: u32,
            go_long: bool,
        ) -> Result<Self, String> {
            // Validate times
            if entry_hour > 23 {
                return Err("Entry hour must be between 0-23".to_string());
            }
            if exit_hour > 23 {
                return Err("Exit hour must be between 0-23".to_string());
            }
            if entry_minute > 59 {
                return Err("Entry minute must be between 0-59".to_string());
            }
            if exit_minute > 59 {
                return Err("Exit minute must be between 0-59".to_string());
            }

            // Ensure entry time is different from exit time
            if entry_hour == exit_hour && entry_minute == exit_minute {
                return Err("Entry and exit times must be different".to_string());
            }

            let direction = if go_long { "Long" } else { "Short" };

            Ok(Self {
                entry_hour,
                entry_minute,
                exit_hour,
                exit_minute,
                go_long,
                name: format!(
                    "Time of Day {} ({}:{:02} to {}:{:02})",
                    direction, entry_hour, entry_minute, exit_hour, exit_minute
                ),
            })
        }

        /// Get entry hour
        pub fn entry_hour(&self) -> u32 {
            self.entry_hour
        }

        /// Get entry minute
        pub fn entry_minute(&self) -> u32 {
            self.entry_minute
        }

        /// Get exit hour
        pub fn exit_hour(&self) -> u32 {
            self.exit_hour
        }

        /// Get exit minute
        pub fn exit_minute(&self) -> u32 {
            self.exit_minute
        }

        /// Check if time is at or after entry time
        fn is_entry_time(&self, hour: u32, minute: u32) -> bool {
            (hour > self.entry_hour) || (hour == self.entry_hour && minute >= self.entry_minute)
        }

        /// Check if time is at or after exit time
        fn is_exit_time(&self, hour: u32, minute: u32) -> bool {
            (hour > self.exit_hour) || (hour == self.exit_hour && minute >= self.exit_minute)
        }

        /// Convert timestamp to hour and minute
        fn get_hour_minute(&self, timestamp: DateTime<Utc>) -> (u32, u32) {
            (timestamp.hour(), timestamp.minute())
        }
    }

    impl IntradayStrategy for TimeOfDayStrategy {
        fn name(&self) -> &str {
            &self.name
        }

        fn generate_signals(&self, data: &[MinuteOhlcv]) -> Result<Vec<Signal>, TradeError> {
            let mut signals = Vec::with_capacity(data.len());
            let mut position_entered = false;

            for candle in data {
                let (hour, minute) = self.get_hour_minute(candle.timestamp);
                
                let signal = if !position_entered && self.is_entry_time(hour, minute) {
                    position_entered = true;
                    if self.go_long {
                        Signal::Buy
                    } else {
                        Signal::Sell
                    }
                } else if position_entered && self.is_exit_time(hour, minute) {
                    position_entered = false;
                    if self.go_long {
                        Signal::Sell // Exit long position
                    } else {
                        Signal::Buy // Exit short position
                    }
                } else {
                    Signal::Hold
                };

                signals.push(signal);
            }

            Ok(signals)
        }

        fn calculate_performance(
            &self,
            data: &[MinuteOhlcv],
            signals: &[Signal],
        ) -> Result<f64, TradeError> {
            // Use a low commission for time-based strategies
            let commission = 0.01; // 0.01% per trade
            calculate_basic_performance(data, signals, 10000.0, commission)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::minute_trade::OhlcvData;
        use chrono::{TimeZone, Utc};

        #[test]
        fn test_time_of_day_parameters() {
            // Test valid parameters
            let strategy = TimeOfDayStrategy::new(9, 30, 16, 0, true);
            assert!(strategy.is_ok());

            // Test invalid hour
            let strategy = TimeOfDayStrategy::new(24, 30, 16, 0, true);
            assert!(strategy.is_err());

            // Test invalid minute
            let strategy = TimeOfDayStrategy::new(9, 60, 16, 0, true);
            assert!(strategy.is_err());

            // Test same entry and exit time
            let strategy = TimeOfDayStrategy::new(9, 30, 9, 30, true);
            assert!(strategy.is_err());
        }

        #[test]
        fn test_time_checks() {
            let strategy = TimeOfDayStrategy::new(9, 30, 16, 0, true).unwrap();

            // Test entry time checks
            assert!(strategy.is_entry_time(9, 30));
            assert!(strategy.is_entry_time(9, 45));
            assert!(strategy.is_entry_time(10, 0));
            assert!(!strategy.is_entry_time(9, 29));
            assert!(!strategy.is_entry_time(8, 45));

            // Test exit time checks
            assert!(strategy.is_exit_time(16, 0));
            assert!(strategy.is_exit_time(16, 15));
            assert!(strategy.is_exit_time(17, 0));
            assert!(!strategy.is_exit_time(15, 59));
            assert!(!strategy.is_exit_time(15, 45));
        }

        #[test]
        fn test_signal_generation() {
            // Use test data with predefined timestamps
            let mut data = Vec::new();
            use chrono::{TimeZone, Utc};

            // Create a day's worth of 1-minute data with timestamp
            let day_start = Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap();

            for i in 0..390 {
                // 6.5 hour trading day (390 minutes)
                // Start at 9:30 AM
                let timestamp = day_start
                    + chrono::Duration::hours(9)
                    + chrono::Duration::minutes(30)
                    + chrono::Duration::minutes(i as i64);

                let ohlcv = MinuteOhlcv {
                    timestamp,
                    data: OhlcvData {
                        open: 100.0 + i as f64 * 0.01,
                        high: 100.0 + i as f64 * 0.01 + 0.2,
                        low: 100.0 + i as f64 * 0.01 - 0.2,
                        close: 100.0 + i as f64 * 0.01 + 0.1,
                        volume: 1000.0,
                    },
                };
                data.push(ohlcv);
            }

            // Test a strategy that buys at 10:00 and sells at 15:30
            let strategy = TimeOfDayStrategy::new(10, 0, 15, 30, true).unwrap();
            let signals = strategy.generate_signals(&data).unwrap();

            // Check that we have the correct number of signals
            assert_eq!(signals.len(), data.len());

            // Entry should be at 10:00 AM (30 minutes after market open)
            let entry_idx = 30; // 9:30 + 30 minutes = 10:00

            // Exit should be at 15:30 (6 hours after market open)
            let exit_idx = 360; // 9:30 + 360 minutes = 15:30

            // Check entry signal
            assert_eq!(signals[entry_idx], Signal::Buy);

            // Check exit signal
            assert_eq!(signals[exit_idx], Signal::Sell);

            // Check some hold signals
            assert_eq!(signals[0], Signal::Hold); // Before entry
            assert_eq!(signals[entry_idx + 1], Signal::Hold); // After entry
            assert_eq!(signals[exit_idx - 1], Signal::Hold); // Before exit
        }
    }
}

mod session_transition_strategy {
    use crate::minute_trade::utils::{calculate_basic_performance, validate_positive};
    use crate::minute_trade::{IntradayStrategy, MinuteOhlcv, Signal, TradeError};
    use chrono::{DateTime, Timelike, Utc};

    /// Session Transition Strategy - trades based on market session transitions and volatility patterns
    #[derive(Debug, Clone)]
    pub struct SessionTransitionStrategy {
        /// Pre-market session end hour (when regular session starts)
        regular_session_start_hour: u32,
        /// Pre-market session end minute
        regular_session_start_minute: u32,
        /// Regular session end hour (when after-hours starts)
        regular_session_end_hour: u32,
        /// Regular session end minute
        regular_session_end_minute: u32,
        /// Volatility threshold for signal generation (percentage)
        volatility_threshold: f64,
        /// Volume threshold multiplier for confirmation
        volume_multiplier: f64,
        /// Strategy name
        name: String,
    }

    impl Default for SessionTransitionStrategy {
        fn default() -> Self {
            Self::new(9, 30, 16, 0, 1.0, 1.5).unwrap()
        }
    }

    impl SessionTransitionStrategy {
        /// Create a new Session Transition Strategy
        ///
        /// # Arguments
        ///
        /// * `regular_session_start_hour` - Hour when regular session starts (typically 9 for US markets)
        /// * `regular_session_start_minute` - Minute when regular session starts (typically 30 for US markets)
        /// * `regular_session_end_hour` - Hour when regular session ends (typically 16 for US markets)
        /// * `regular_session_end_minute` - Minute when regular session ends (typically 0 for US markets)
        /// * `volatility_threshold` - Minimum volatility percentage for signal generation (typically 0.5-2.0)
        /// * `volume_multiplier` - Volume multiplier for confirmation (typically 1.2-2.0)
        ///
        /// # Returns
        ///
        /// * `Result<Self, String>` - New strategy instance or error message
        pub fn new(
            regular_session_start_hour: u32,
            regular_session_start_minute: u32,
            regular_session_end_hour: u32,
            regular_session_end_minute: u32,
            volatility_threshold: f64,
            volume_multiplier: f64,
        ) -> Result<Self, String> {
            if regular_session_start_hour >= 24 || regular_session_end_hour >= 24 {
                return Err("Hours must be between 0 and 23".to_string());
            }

            if regular_session_start_minute >= 60 || regular_session_end_minute >= 60 {
                return Err("Minutes must be between 0 and 59".to_string());
            }

            validate_positive(volatility_threshold, "Volatility threshold")?;
            validate_positive(volume_multiplier, "Volume multiplier")?;

            if volume_multiplier < 1.1 {
                return Err("Volume multiplier should be at least 1.1 to detect meaningful volume increases.".to_string());
            }

            // Validate that start time is before end time
            let start_minutes = regular_session_start_hour * 60 + regular_session_start_minute;
            let end_minutes = regular_session_end_hour * 60 + regular_session_end_minute;
            
            if start_minutes >= end_minutes {
                return Err("Regular session start time must be before end time".to_string());
            }

            Ok(Self {
                regular_session_start_hour,
                regular_session_start_minute,
                regular_session_end_hour,
                regular_session_end_minute,
                volatility_threshold,
                volume_multiplier,
                name: format!(
                    "Session Transition ({}:{:02}-{}:{:02}, {}% vol, {}x volume)",
                    regular_session_start_hour, regular_session_start_minute,
                    regular_session_end_hour, regular_session_end_minute,
                    volatility_threshold, volume_multiplier
                ),
            })
        }

        /// Get regular session start hour
        pub fn regular_session_start_hour(&self) -> u32 {
            self.regular_session_start_hour
        }

        /// Get regular session start minute
        pub fn regular_session_start_minute(&self) -> u32 {
            self.regular_session_start_minute
        }

        /// Get regular session end hour
        pub fn regular_session_end_hour(&self) -> u32 {
            self.regular_session_end_hour
        }

        /// Get regular session end minute
        pub fn regular_session_end_minute(&self) -> u32 {
            self.regular_session_end_minute
        }

        /// Get volatility threshold
        pub fn volatility_threshold(&self) -> f64 {
            self.volatility_threshold
        }

        /// Get volume multiplier
        pub fn volume_multiplier(&self) -> f64 {
            self.volume_multiplier
        }

        /// Determine the current market session
        fn get_market_session(&self, timestamp: DateTime<Utc>) -> MarketSession {
            let hour = timestamp.hour();
            let minute = timestamp.minute();
            let current_minutes = hour * 60 + minute;
            
            let start_minutes = self.regular_session_start_hour * 60 + self.regular_session_start_minute;
            let end_minutes = self.regular_session_end_hour * 60 + self.regular_session_end_minute;

            if current_minutes < start_minutes {
                MarketSession::PreMarket
            } else if current_minutes < end_minutes {
                MarketSession::Regular
            } else {
                MarketSession::AfterHours
            }
        }

        /// Check if we're at a session transition point
        fn is_session_transition(&self, current_session: MarketSession, previous_session: MarketSession) -> bool {
            matches!(
                (previous_session, current_session),
                (MarketSession::PreMarket, MarketSession::Regular) |
                (MarketSession::Regular, MarketSession::AfterHours)
            )
        }

        /// Calculate volatility as percentage range (high-low)/close
        fn calculate_volatility(&self, candle: &MinuteOhlcv) -> f64 {
            if candle.data.close <= 0.0 {
                return 0.0;
            }
            ((candle.data.high - candle.data.low) / candle.data.close) * 100.0
        }

        /// Calculate average volume over recent periods
        fn calculate_average_volume(&self, data: &[MinuteOhlcv], end_index: usize, periods: usize) -> Option<f64> {
            if end_index < periods {
                return None;
            }

            let start_index = end_index - periods;
            let total_volume: f64 = data[start_index..end_index]
                .iter()
                .map(|d| d.data.volume)
                .sum();

            Some(total_volume / periods as f64)
        }

        /// Generate signal based on session transition and market conditions
        fn evaluate_transition_signal(
            &self,
            current_candle: &MinuteOhlcv,
            previous_candle: &MinuteOhlcv,
            avg_volume: Option<f64>,
            is_transition: bool,
        ) -> Signal {
            if !is_transition {
                return Signal::Hold;
            }

            let volatility = self.calculate_volatility(current_candle);
            let current_volume = current_candle.data.volume;
            
            // Check volatility condition
            if volatility < self.volatility_threshold {
                return Signal::Hold;
            }

            // Check volume condition if we have historical data
            if let Some(avg_vol) = avg_volume {
                let volume_ratio = current_volume / avg_vol;
                if volume_ratio < self.volume_multiplier {
                    return Signal::Hold;
                }
            }

            // Determine signal direction based on price movement
            let price_change = current_candle.data.close - previous_candle.data.close;
            
            if price_change > 0.0 {
                // Price increasing with high volatility and volume at transition - buy
                Signal::Buy
            } else if price_change < 0.0 {
                // Price decreasing with high volatility and volume at transition - sell
                Signal::Sell
            } else {
                Signal::Hold
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    enum MarketSession {
        PreMarket,
        Regular,
        AfterHours,
    }

    impl IntradayStrategy for SessionTransitionStrategy {
        fn name(&self) -> &str {
            &self.name
        }

        fn generate_signals(&self, data: &[MinuteOhlcv]) -> Result<Vec<Signal>, TradeError> {
            if data.len() < 2 {
                return Err(TradeError::InsufficientData(
                    "Need at least 2 data points for Session Transition strategy".to_string()
                ));
            }

            let mut signals = Vec::with_capacity(data.len());

            // First signal is always hold
            signals.push(Signal::Hold);

            let mut previous_session = self.get_market_session(data[0].timestamp);
            let mut in_position = false;
            let mut position_is_long = false;

            // Generate signals for the remaining data points
            for i in 1..data.len() {
                let current_candle = &data[i];
                let previous_candle = &data[i - 1];
                let current_session = self.get_market_session(current_candle.timestamp);
                
                let is_transition = self.is_session_transition(current_session, previous_session);
                
                // Calculate average volume over the last 10 periods (if available)
                let avg_volume = self.calculate_average_volume(data, i, 10.min(i));

                let signal = if !in_position {
                    // No position - look for entry signals at transitions
                    let entry_signal = self.evaluate_transition_signal(
                        current_candle,
                        previous_candle,
                        avg_volume,
                        is_transition,
                    );

                    match entry_signal {
                        Signal::Buy => {
                            in_position = true;
                            position_is_long = true;
                            Signal::Buy
                        }
                        Signal::Sell => {
                            in_position = true;
                            position_is_long = false;
                            Signal::Sell
                        }
                        Signal::Hold => Signal::Hold,
                    }
                } else {
                    // In position - look for exit conditions
                    let volatility = self.calculate_volatility(current_candle);
                    
                    // Exit if volatility drops significantly or at session transition
                    let should_exit = volatility < (self.volatility_threshold * 0.5) || is_transition;
                    
                    if should_exit {
                        in_position = false;
                        if position_is_long {
                            Signal::Sell // Close long position
                        } else {
                            Signal::Buy // Close short position
                        }
                    } else {
                        Signal::Hold
                    }
                };

                signals.push(signal);
                previous_session = current_session;
            }

            Ok(signals)
        }

        fn calculate_performance(
            &self,
            data: &[MinuteOhlcv],
            signals: &[Signal],
        ) -> Result<f64, TradeError> {
            // Use moderate commission for session transition strategies
            let commission = 0.02; // 0.02% per trade
            calculate_basic_performance(data, signals, 10000.0, commission)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::minute_trade::{create_test_data, OhlcvData};
        use chrono::{TimeZone, Utc};

        #[test]
        fn test_session_transition_parameters() {
            // Test valid parameters
            let strategy = SessionTransitionStrategy::new(9, 30, 16, 0, 1.0, 1.5);
            assert!(strategy.is_ok());

            // Test invalid hours
            let strategy = SessionTransitionStrategy::new(24, 30, 16, 0, 1.0, 1.5);
            assert!(strategy.is_err());

            // Test invalid minutes
            let strategy = SessionTransitionStrategy::new(9, 60, 16, 0, 1.0, 1.5);
            assert!(strategy.is_err());

            // Test invalid time order
            let strategy = SessionTransitionStrategy::new(16, 0, 9, 30, 1.0, 1.5);
            assert!(strategy.is_err());

            // Test invalid volume multiplier
            let strategy = SessionTransitionStrategy::new(9, 30, 16, 0, 1.0, 1.0);
            assert!(strategy.is_err());
        }

        #[test]
        fn test_market_session_detection() {
            let strategy = SessionTransitionStrategy::new(9, 30, 16, 0, 1.0, 1.5).unwrap();

            // Pre-market time
            let pre_market = Utc.with_ymd_and_hms(2021, 1, 1, 8, 0, 0).unwrap();
            assert_eq!(strategy.get_market_session(pre_market), MarketSession::PreMarket);

            // Regular session time
            let regular = Utc.with_ymd_and_hms(2021, 1, 1, 12, 0, 0).unwrap();
            assert_eq!(strategy.get_market_session(regular), MarketSession::Regular);

            // After-hours time
            let after_hours = Utc.with_ymd_and_hms(2021, 1, 1, 18, 0, 0).unwrap();
            assert_eq!(strategy.get_market_session(after_hours), MarketSession::AfterHours);
        }

        #[test]
        fn test_session_transition_detection() {
            let strategy = SessionTransitionStrategy::new(9, 30, 16, 0, 1.0, 1.5).unwrap();

            // Pre-market to regular session transition
            assert!(strategy.is_session_transition(MarketSession::Regular, MarketSession::PreMarket));

            // Regular to after-hours transition
            assert!(strategy.is_session_transition(MarketSession::AfterHours, MarketSession::Regular));

            // No transition (same session)
            assert!(!strategy.is_session_transition(MarketSession::Regular, MarketSession::Regular));
        }

        #[test]
        fn test_volatility_calculation() {
            let strategy = SessionTransitionStrategy::new(9, 30, 16, 0, 1.0, 1.5).unwrap();
            
            let timestamp = Utc.with_ymd_and_hms(2021, 1, 1, 12, 0, 0).unwrap();
            let candle = MinuteOhlcv {
                timestamp,
                data: OhlcvData {
                    open: 100.0,
                    high: 102.0,
                    low: 98.0,
                    close: 100.0,
                    volume: 1000.0,
                },
            };

            let volatility = strategy.calculate_volatility(&candle);
            assert!((volatility - 4.0).abs() < f64::EPSILON); // (102-98)/100 * 100 = 4%
        }

        #[test]
        fn test_signal_generation() {
            let data = create_test_data(50);
            let strategy = SessionTransitionStrategy::new(9, 30, 16, 0, 1.0, 1.5).unwrap();

            let signals = strategy.generate_signals(&data).unwrap();

            // Check that we have the correct number of signals
            assert_eq!(signals.len(), data.len());

            // First signal should be Hold
            assert_eq!(signals[0], Signal::Hold);
        }
    }
}
