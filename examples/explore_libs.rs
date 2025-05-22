// This program explores the module structure of NyxsOwl libraries
fn main() {
    println!("Exploring NyxsOwl libraries\n");

    // Day trading strategies examples
    println!("=== Day Trading Strategies ===");
    show_day_trade_examples();

    // Minute trading strategies examples
    println!("\n=== Minute Trading Strategies ===");
    show_minute_trade_examples();

    // Trade math utilities
    println!("\n=== Trade Math Utilities ===");
    show_trade_math_examples();

    println!("\nDone exploring");
}

fn show_day_trade_examples() {
    // Display what day trading functionality is available
    // You can use the commented lines once you have a version() function or similar

    println!("Available day trading strategies:");
    println!("- Forecasting Strategy: Predict future prices using time series analysis");
    println!("- Moving Average Strategy: Trade based on MA crossovers");
    println!("- VWAP Strategy: Trade based on Volume Weighted Average Price");
    println!("- Grid Trading Strategy: Set up a grid of buy/sell orders");

    println!("\nUsage example (see day_trade/examples for more):");
    println!(
        r#"
    use day_trade::strategies::ForecastingStrategy;
    use day_trade::TradingStrategy;
    
    // Create a forecasting strategy
    let strategy = ForecastingStrategy::new(20, 3, 0.3, 0.5)?;
    
    // Generate signals
    let signals = strategy.generate_signals(&data)?;
    
    // Calculate performance
    let performance = strategy.calculate_performance(&data, &signals)?;
    "#
    );
}

fn show_minute_trade_examples() {
    // Display what minute trading functionality is available

    println!("Available minute trading strategies:");
    println!("- Scalping Strategy: Ultra-short term trading");
    println!("- Momentum Breakout: Trade momentum with volume confirmation");
    println!("- Volatility Breakout: Enter after low volatility periods");
    println!("- Mean Reversion: Trade when prices deviate from mean");

    println!("\nUsage example (see minute_trade/examples for more):");
    println!(
        r#"
    use minute_trade::{{ScalpingStrategy, IntradayStrategy}};
    use minute_trade::utils::load_minute_data;
    
    // Load minute-by-minute data
    let data = load_minute_data("AAPL_minute_data.csv")?;
    
    // Create a scalping strategy with 5-minute lookback and 0.1% threshold
    let strategy = ScalpingStrategy::new(5, 0.1)?;
    
    // Generate trading signals
    let signals = strategy.generate_signals(&data)?;
    
    // Calculate performance
    let performance = strategy.calculate_performance(&data, &signals)?;
    "#
    );
}

fn show_trade_math_examples() {
    // Display what trade math utilities are available

    println!("Available mathematical functions:");
    println!("- Statistical Indicators (RSI, MACD, Bollinger Bands)");
    println!("- Performance Metrics (Sharpe Ratio, Sortino Ratio, Max Drawdown)");
    println!("- Time Series Analysis (Moving Averages, Linear Regression)");

    println!("\nUsage example:");
    println!(
        r#"
    use trade_math::indicators::{{calculate_rsi, calculate_macd}};
    use trade_math::performance::{{sharpe_ratio, max_drawdown}};
    
    // Calculate RSI
    let rsi_values = calculate_rsi(&prices, 14)?;
    
    // Calculate MACD
    let (macd_line, signal_line, histogram) = calculate_macd(&prices, 12, 26, 9)?;
    
    // Calculate Sharpe ratio
    let sharpe = sharpe_ratio(&returns, risk_free_rate)?;
    "#
    );
}
