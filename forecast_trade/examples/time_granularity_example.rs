use chrono::{Duration, TimeZone, Utc};
use forecast_trade::data::TimeSeriesData;
use forecast_trade::models::exponential_smoothing::ExponentialSmoothing;
use forecast_trade::models::ForecastModel;
use forecast_trade::strategies::mean_reversion::MeanReversionStrategy;
use forecast_trade::strategies::trend_following::TrendFollowingStrategy;
use forecast_trade::strategies::volatility_breakout::{VolatilityBreakoutConfig, VolatilityBreakoutStrategy};
use forecast_trade::strategies::{ForecastStrategy, TimeGranularity, TradingSignal};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Forecast Trade: Time Granularity Example");
    println!("=======================================\n");

    // Create sample data for both granularities
    let daily_data = create_sample_daily_data();
    let minute_data = create_sample_minute_data();

    println!(
        "Created sample data: {} daily records, {} minute records\n",
        daily_data.len(),
        minute_data.len()
    );

    // === Test models with different granularities ===
    println!("1. TESTING MODELS WITH DIFFERENT GRANULARITIES");
    println!("---------------------------------------------");

    // Test Exponential Smoothing with different granularities
    let daily_es = ExponentialSmoothing::new(0.2)?;
    let minute_es = ExponentialSmoothing::new_minute(0.4)?;

    println!("Exponential Smoothing - Daily alpha: {}", daily_es.clone().train(&daily_data)?.forecast(&daily_data, 1)?.values[0]);
    println!("Exponential Smoothing - Minute alpha: {}\n", minute_es.clone().train(&minute_data)?.forecast(&minute_data, 1)?.values[0]);

    // === Test strategies with different granularities ===
    println!("\n2. TESTING STRATEGIES WITH DIFFERENT GRANULARITIES");
    println!("------------------------------------------------");

    // Test Mean Reversion Strategy
    println!("Mean Reversion Strategy:");
    let daily_mr = MeanReversionStrategy::new(daily_es.clone(), 2.0)?;

    let minute_mr = MeanReversionStrategy::new_with_granularity(
        minute_es.clone(),
        1.5, 
        TimeGranularity::Minute
    )?;

    let daily_mr_signals = daily_mr.generate_signals(&daily_data)?;
    let minute_mr_signals = minute_mr.generate_signals(&minute_data)?;

    print_signals_summary("Daily", &daily_mr_signals);
    print_signals_summary("Minute", &minute_mr_signals);

    // Test Trend Following Strategy
    println!("\nTrend Following Strategy:");
    let daily_tf = TrendFollowingStrategy::new(daily_es.clone(), 0.5)?;

    let minute_tf = TrendFollowingStrategy::new_with_granularity(
        minute_es.clone(),
        0.2,
        TimeGranularity::Minute
    )?;

    let daily_tf_signals = daily_tf.generate_signals(&daily_data)?;
    let minute_tf_signals = minute_tf.generate_signals(&minute_data)?;

    print_signals_summary("Daily", &daily_tf_signals);
    print_signals_summary("Minute", &minute_tf_signals);

    // Test Volatility Breakout Strategy
    println!("\nVolatility Breakout Strategy:");
    
    let daily_vb = VolatilityBreakoutStrategy::new(
        daily_es.clone(), 
        1.5
    )?;

    let minute_vb = VolatilityBreakoutStrategy::new_with_granularity(
        minute_es.clone(),
        2.0,
        TimeGranularity::Minute
    )?;

    let daily_vb_signals = daily_vb.generate_signals(&daily_data)?;
    let minute_vb_signals = minute_vb.generate_signals(&minute_data)?;

    print_signals_summary("Daily", &daily_vb_signals);
    print_signals_summary("Minute", &minute_vb_signals);

    // === Run backtests with different granularities ===
    println!("\n3. RUNNING BACKTESTS WITH DIFFERENT GRANULARITIES");
    println!("-----------------------------------------------");
    
    let initial_capital = 10000.0;

    // Backtest Mean Reversion Strategy
    println!("Mean Reversion Strategy Backtest:");
    let daily_mr_backtest = daily_mr.backtest(&daily_data, initial_capital)?;
    let minute_mr_backtest = minute_mr.backtest(&minute_data, initial_capital)?;

    println!("Daily: ${:.2} final balance, {:.1}% max drawdown, {:.1}% win rate", 
        daily_mr_backtest.final_balance, 
        daily_mr_backtest.max_drawdown * 100.0,
        daily_mr_backtest.win_rate * 100.0
    );
    println!("Minute: ${:.2} final balance, {:.1}% max drawdown, {:.1}% win rate", 
        minute_mr_backtest.final_balance, 
        minute_mr_backtest.max_drawdown * 100.0,
        minute_mr_backtest.win_rate * 100.0
    );

    // Backtest Volatility Breakout Strategy
    println!("\nVolatility Breakout Strategy Backtest:");
    let daily_vb_backtest = daily_vb.backtest(&daily_data, initial_capital)?;
    let minute_vb_backtest = minute_vb.backtest(&minute_data, initial_capital)?;

    println!("Daily: ${:.2} final balance, {:.1}% max drawdown, {:.1}% win rate", 
        daily_vb_backtest.final_balance, 
        daily_vb_backtest.max_drawdown * 100.0,
        daily_vb_backtest.win_rate * 100.0
    );
    println!("Minute: ${:.2} final balance, {:.1}% max drawdown, {:.1}% win rate", 
        minute_vb_backtest.final_balance, 
        minute_vb_backtest.max_drawdown * 100.0,
        minute_vb_backtest.win_rate * 100.0
    );

    // Compare transaction costs
    println!("\n4. COMPARING TRANSACTION COSTS BETWEEN GRANULARITIES");
    println!("--------------------------------------------------");
    println!("Daily: 0.1% commission, 0.05% slippage");
    println!("Minute: 0.05% commission, 0.1% slippage");

    // Custom transaction costs test
    println!("\nCustom Transaction Costs Test (Mean Reversion):");
    let daily_custom = daily_mr.backtest_with_params(&daily_data, initial_capital, 0.002, 0.001)?;
    let minute_custom = minute_mr.backtest_with_params(&minute_data, initial_capital, 0.001, 0.002)?;

    println!("Daily (high commission): ${:.2} final balance", daily_custom.final_balance);
    println!("Minute (high slippage): ${:.2} final balance", minute_custom.final_balance);

    Ok(())
}

// Helper function to print signals summary
fn print_signals_summary(granularity: &str, signals: &[TradingSignal]) {
    let buy_count = signals.iter().filter(|&&s| s == TradingSignal::Buy).count();
    let sell_count = signals.iter().filter(|&&s| s == TradingSignal::Sell).count();
    let hold_count = signals.iter().filter(|&&s| s == TradingSignal::Hold).count();
    
    println!("  {}: {} signals - {} buy, {} sell, {} hold", 
        granularity, 
        signals.len(), 
        buy_count,
        sell_count,
        hold_count
    );
}

// Helper function to create sample daily data
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

// Helper function to create sample minute data
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