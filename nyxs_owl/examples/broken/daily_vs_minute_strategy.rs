use chrono::{Datelike, Duration, TimeZone, Utc};
use forecast_trade::data::TimeSeriesData;
use forecast_trade::models::exponential_smoothing::ExponentialSmoothing;
use forecast_trade::models::ForecastModel;
use forecast_trade::strategies::mean_reversion::MeanReversionStrategy;
use forecast_trade::strategies::trend_following::TrendFollowingStrategy;
use forecast_trade::strategies::volatility_breakout::VolatilityBreakoutStrategy;
use forecast_trade::strategies::{ForecastStrategy, TradingSignal};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Forecast Trade: Daily vs Minute Strategy Example");
    println!("================================================\n");

    // Load sample data
    // This example uses mock data since real data files might not be available
    println!("Creating mock data for demonstration...");
    let daily_data = create_sample_daily_data();
    let minute_data = create_sample_minute_data();

    println!(
        "Data loaded: {} daily records, {} minute records\n",
        daily_data.len(),
        minute_data.len()
    );

    // Create forecast models for both timeframes
    println!("Creating forecast models...");
    let daily_model = ExponentialSmoothing::new(0.2)?;
    let minute_model = ExponentialSmoothing::new(0.4)?;

    // Train models - but we won't use the trained models directly in this example
    let _trained_daily_model = daily_model.train(&daily_data)?;
    let _trained_minute_model = minute_model.train(&minute_data)?;

    // Create strategies for both timeframes
    println!("Creating strategies...");

    // Mean Reversion Strategy - using models instead of string names
    let daily_mr_strategy = MeanReversionStrategy::new(
        daily_model.clone(),
        2.0, // Standard deviation threshold - higher for daily
    )?;

    let minute_mr_strategy = MeanReversionStrategy::new(
        minute_model.clone(),
        1.5, // Standard deviation threshold - lower for minute
    )?;

    // Trend Following Strategy
    let daily_tf_strategy = TrendFollowingStrategy::new(
        daily_model.clone(),
        10, // Window size in days
    )?;

    let minute_tf_strategy = TrendFollowingStrategy::new(
        minute_model.clone(),
        30, // Window size in minutes
    )?;

    // Volatility Breakout Strategy
    let daily_vb_strategy = VolatilityBreakoutStrategy::new(
        daily_model.clone(),
        1.5, // Volatility multiplier
    )?;

    let minute_vb_strategy = VolatilityBreakoutStrategy::new(
        minute_model.clone(),
        2.0, // Volatility multiplier - higher for minute data due to noise
    )?;

    // Generate signals
    println!("\nGenerating signals...");

    // Mean Reversion signals
    let daily_mr_signals = daily_mr_strategy.generate_signals(&daily_data)?;
    let minute_mr_signals = minute_mr_strategy.generate_signals(&minute_data)?;

    print_signal_stats("Daily Mean Reversion Strategy", &daily_mr_signals);
    print_signal_stats("Minute Mean Reversion Strategy", &minute_mr_signals);

    // Trend Following signals
    let daily_tf_signals = daily_tf_strategy.generate_signals(&daily_data)?;
    let minute_tf_signals = minute_tf_strategy.generate_signals(&minute_data)?;

    print_signal_stats("Daily Trend Following Strategy", &daily_tf_signals);
    print_signal_stats("Minute Trend Following Strategy", &minute_tf_signals);

    // Volatility Breakout signals
    let daily_vb_signals = daily_vb_strategy.generate_signals(&daily_data)?;
    let minute_vb_signals = minute_vb_strategy.generate_signals(&minute_data)?;

    print_signal_stats("Daily Volatility Breakout Strategy", &daily_vb_signals);
    print_signal_stats("Minute Volatility Breakout Strategy", &minute_vb_signals);

    // Run backtests
    println!("\nRunning backtests...\n");

    // Backtest Mean Reversion
    let initial_capital = 10000.0;

    // Daily MR backtest
    let daily_mr_results = daily_mr_strategy.backtest(&daily_data, initial_capital)?;

    println!("Daily Mean Reversion Backtest:");
    println!("  Final balance: ${:.2}", daily_mr_results.final_balance);
    println!("  Total trades: {}", daily_mr_results.total_trades);
    println!("  Win rate: {:.1}%", daily_mr_results.win_rate * 100.0);
    println!(
        "  Max drawdown: {:.1}%",
        daily_mr_results.max_drawdown * 100.0
    );

    // Minute MR backtest
    let minute_mr_results = minute_mr_strategy.backtest(&minute_data, initial_capital)?;

    println!("\nMinute Mean Reversion Backtest:");
    println!("  Final balance: ${:.2}", minute_mr_results.final_balance);
    println!("  Total trades: {}", minute_mr_results.total_trades);
    println!("  Win rate: {:.1}%", minute_mr_results.win_rate * 100.0);
    println!(
        "  Max drawdown: {:.1}%",
        minute_mr_results.max_drawdown * 100.0
    );
    println!(
        "  Sharpe ratio: {:.2}",
        minute_mr_results
            .performance_metrics
            .sharpe_ratio
            .unwrap_or(0.0)
    );

    println!("\nComparing transaction costs between timeframes:");
    println!("  Daily: 0.1% commission, 0.05% slippage");
    println!("  Minute: 0.05% commission, 0.1% slippage");

    // Example of direct conversion between day_trade and minute_trade would go here,
    // but we'll skip it since the formats have changed
    println!("\nDemonstrating timeframe conversions:");
    println!(
        "  Daily data can be converted to OHLCV format using TimeSeriesData::to_daily_ohlcv()"
    );
    println!(
        "  Minute data can be converted to OHLCV format using TimeSeriesData::to_minute_ohlcv()"
    );

    Ok(())
}

fn print_signal_stats(strategy_name: &str, signals: &[TradingSignal]) {
    let total = signals.len();
    let buy_count = signals.iter().filter(|&&s| s == TradingSignal::Buy).count();
    let sell_count = signals
        .iter()
        .filter(|&&s| s == TradingSignal::Sell)
        .count();
    let hold_count = signals
        .iter()
        .filter(|&&s| s == TradingSignal::Hold)
        .count();

    println!("{}:", strategy_name);
    println!("  Total signals: {}", total);
    println!(
        "  Buy signals: {} ({:.1}%)",
        buy_count,
        (buy_count as f64 / total as f64) * 100.0
    );
    println!(
        "  Sell signals: {} ({:.1}%)",
        sell_count,
        (sell_count as f64 / total as f64) * 100.0
    );
    println!(
        "  Hold signals: {} ({:.1}%)",
        hold_count,
        (hold_count as f64 / total as f64) * 100.0
    );
}

// Sample data generation
fn create_sample_daily_data() -> TimeSeriesData {
    let mut dates = Vec::with_capacity(100);
    let mut prices = Vec::with_capacity(100);

    let start_date = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap();

    // Create 100 days of data with trend and some seasonality
    let mut price = 100.0;
    let trend = 0.05; // 0.05 points per day upward trend

    for i in 0..100 {
        let current_date = start_date + Duration::days(i);
        dates.push(current_date);

        // Add some weekly seasonality and noise
        let day_of_week = current_date.weekday().num_days_from_monday() as f64;
        let seasonality = (day_of_week * std::f64::consts::PI / 7.0).sin() * 2.0;
        let noise = (i as f64 * 0.1).sin() * 1.0;

        price = price + trend + seasonality + noise;
        prices.push(price);
    }

    TimeSeriesData::new(dates, prices).unwrap()
}

fn create_sample_minute_data() -> TimeSeriesData {
    let mut dates = Vec::with_capacity(480);
    let mut prices = Vec::with_capacity(480);

    let start_date = Utc.with_ymd_and_hms(2023, 1, 1, 9, 30, 0).unwrap();

    // Create 8 hours of minute data (480 minutes)
    let mut price = 100.0;
    let trend = 0.002; // Small trend per minute

    for i in 0..480 {
        let current_date = start_date + Duration::minutes(i);
        dates.push(current_date);

        // Add intraday pattern (U-shape)
        let minute_of_day = i % 480;
        let normalized_time = minute_of_day as f64 / 480.0;
        let intraday = ((normalized_time - 0.5) * 2.0).powi(2) * 1.0;

        // Add higher frequency noise
        let noise = (i as f64 * 0.5).sin() * 0.2 + (i as f64 * 0.3).cos() * 0.3;

        price = price + trend + intraday + noise;
        prices.push(price);
    }

    TimeSeriesData::new(dates, prices).unwrap()
}
