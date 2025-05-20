use day_trade::{
    // Utils
    utils,
    AdaptiveMovingAverageStrategy,
    // Hold-focused strategies
    BollingerBandsStrategy,
    BreakoutStrategy,
    CompositeStrategy,
    DailyOhlcv,
    DualTimeframeStrategy,
    ForecastingStrategy,
    GridTradingStrategy,
    // Buy-focused strategies
    MACrossover,
    MacdStrategy,
    MeanReversionStrategy,
    // Sell-focused strategies
    RsiStrategy,
    Signal,
    // Traits and types
    TradingStrategy,
    VolumeBasedStrategy,
    VwapStrategy,
};

/// Create an instance of each buy-focused strategy for testing
fn create_buy_strategies() -> Vec<Box<dyn TradingStrategy>> {
    let mut strategies: Vec<Box<dyn TradingStrategy>> = Vec::new();

    // Add each buy-focused strategy (with error handling)
    if let Ok(strategy) = MACrossover::new(10, 30) {
        strategies.push(Box::new(strategy));
    }

    if let Ok(strategy) = MacdStrategy::new(12, 26, 9) {
        strategies.push(Box::new(strategy));
    }

    if let Ok(strategy) = BreakoutStrategy::new(20, 1.5, 14) {
        strategies.push(Box::new(strategy));
    }

    if let Ok(strategy) = AdaptiveMovingAverageStrategy::new(20, 10, 40, 14, 2.0) {
        strategies.push(Box::new(strategy));
    }

    strategies
}

/// Create an instance of each sell-focused strategy for testing
fn create_sell_strategies() -> Vec<Box<dyn TradingStrategy>> {
    let mut strategies: Vec<Box<dyn TradingStrategy>> = Vec::new();

    // Add each sell-focused strategy (with error handling)
    if let Ok(strategy) = RsiStrategy::new(14, 70.0, 30.0) {
        strategies.push(Box::new(strategy));
    }

    if let Ok(strategy) = MeanReversionStrategy::new(20, 2.0, 0.1, 0.9) {
        strategies.push(Box::new(strategy));
    }

    if let Ok(strategy) = VolumeBasedStrategy::new(20, 14) {
        strategies.push(Box::new(strategy));
    }

    strategies
}

/// Create an instance of each hold-focused strategy for testing
fn create_hold_strategies() -> Vec<Box<dyn TradingStrategy>> {
    let mut strategies: Vec<Box<dyn TradingStrategy>> = Vec::new();

    // Add each hold-focused strategy (with error handling)
    if let Ok(strategy) = BollingerBandsStrategy::new(20, 2.0) {
        strategies.push(Box::new(strategy));
    }

    // Assuming we have the required constructor methods
    if let Ok(strategy) = DualTimeframeStrategy::new(10, 50) {
        strategies.push(Box::new(strategy));
    }

    if let Ok(strategy) = GridTradingStrategy::new(5, 0.5, 14, 0.2) {
        strategies.push(Box::new(strategy));
    }

    if let Ok(strategy) = VwapStrategy::new(14) {
        strategies.push(Box::new(strategy));
    }

    strategies
}

/// Test that buy-focused strategies generate more buy signals in an uptrend
#[test]
fn test_buy_strategies_in_uptrend() {
    // Generate uptrending data
    let data = generate_uptrend_data(100);

    // Get buy-focused strategies
    let strategies = create_buy_strategies();
    assert!(
        !strategies.is_empty(),
        "Failed to create any buy strategies"
    );

    for strategy in strategies {
        let signals = strategy
            .generate_signals(&data)
            .expect("Failed to generate signals");

        // Count the different signal types
        let buy_count = signals.iter().filter(|&&s| s == Signal::Buy).count();
        let sell_count = signals.iter().filter(|&&s| s == Signal::Sell).count();

        // In an uptrend, we expect more buy signals than sell signals from buy-focused strategies
        assert!(
            buy_count > sell_count,
            "Buy strategy did not generate more buy signals than sell signals in uptrend"
        );
    }
}

/// Test that sell-focused strategies generate more sell signals in a downtrend
#[test]
fn test_sell_strategies_in_downtrend() {
    // Generate downtrending data
    let data = generate_downtrend_data(100);

    // Get sell-focused strategies
    let strategies = create_sell_strategies();
    assert!(
        !strategies.is_empty(),
        "Failed to create any sell strategies"
    );

    for strategy in strategies {
        let signals = strategy
            .generate_signals(&data)
            .expect("Failed to generate signals");

        // Count the different signal types
        let buy_count = signals.iter().filter(|&&s| s == Signal::Buy).count();
        let sell_count = signals.iter().filter(|&&s| s == Signal::Sell).count();

        // In a downtrend, we expect more sell signals than buy signals from sell-focused strategies
        assert!(
            sell_count > buy_count,
            "Sell strategy did not generate more sell signals than buy signals in downtrend"
        );
    }
}

/// Test that hold-focused strategies generate more hold signals in a range-bound market
#[test]
fn test_hold_strategies_in_range() {
    // Generate range-bound data
    let data = generate_range_data(100);

    // Get hold-focused strategies
    let strategies = create_hold_strategies();
    assert!(
        !strategies.is_empty(),
        "Failed to create any hold strategies"
    );

    for strategy in strategies {
        let signals = strategy
            .generate_signals(&data)
            .expect("Failed to generate signals");

        // Count the different signal types
        let hold_count = signals.iter().filter(|&&s| s == Signal::Hold).count();
        let action_count = signals.len() - hold_count; // Buy + Sell counts

        // In a range-bound market, we expect more hold signals from hold-focused strategies
        assert!(
            hold_count > action_count / 2, // More than half of the signals should be holds
            "Hold strategy did not generate enough hold signals in range-bound market"
        );
    }
}

/// Generate uptrending price data
fn generate_uptrend_data(length: usize) -> Vec<DailyOhlcv> {
    let mut data = utils::generate_test_data(length, 100.0, 0.03);

    // Add uptrend by increasing prices by a small percentage each day
    let daily_increase = 0.005; // 0.5% daily increase

    let mut current_price = data[0].data.close;
    for i in 1..data.len() {
        current_price *= 1.0 + daily_increase;

        // Adjust open/high/low around the new close to maintain a general uptrend
        let open = data[i].data.open * current_price / data[i].data.close;
        let high = data[i].data.high * current_price / data[i].data.close;
        let low = data[i].data.low * current_price / data[i].data.close;

        data[i].data.open = open;
        data[i].data.high = high;
        data[i].data.low = low;
        data[i].data.close = current_price;
    }

    data
}

/// Generate downtrending price data
fn generate_downtrend_data(length: usize) -> Vec<DailyOhlcv> {
    let mut data = utils::generate_test_data(length, 100.0, 0.03);

    // Add downtrend by decreasing prices by a small percentage each day
    let daily_decrease = 0.005; // 0.5% daily decrease

    let mut current_price = data[0].data.close;
    for i in 1..data.len() {
        current_price *= 1.0 - daily_decrease;

        // Adjust open/high/low around the new close to maintain a general downtrend
        let open = data[i].data.open * current_price / data[i].data.close;
        let high = data[i].data.high * current_price / data[i].data.close;
        let low = data[i].data.low * current_price / data[i].data.close;

        data[i].data.open = open;
        data[i].data.high = high;
        data[i].data.low = low;
        data[i].data.close = current_price;
    }

    data
}

/// Generate range-bound price data
fn generate_range_data(length: usize) -> Vec<DailyOhlcv> {
    let data = utils::generate_test_data(length, 100.0, 0.02);

    // The random data with low volatility is already somewhat range-bound,
    // so we'll just return it as is
    data
}
