use chrono::{Duration, TimeZone, Utc};
use forecast_trade::data::TimeSeriesData;
use forecast_trade::models::oxidiviner::{ExponentialSmoothing, MovingAverage};
use forecast_trade::strategies::mean_reversion::MeanReversionStrategy;
use forecast_trade::strategies::trend_following::TrendFollowingStrategy;
use forecast_trade::strategies::volatility_breakout::VolatilityBreakoutStrategy;
use forecast_trade::strategies::{ForecastStrategy, TimeGranularity, TradingSignal};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Forecast Trade: OxiDiviner Models Example");
    println!("=======================================\n");

    // Create sample data for both daily and minute granularities
    let daily_data = create_sample_daily_data();
    let minute_data = create_sample_minute_data();

    println!(
        "Created sample data: {} daily records, {} minute records\n",
        daily_data.len(),
        minute_data.len()
    );

    // === OxiDiviner Exponential Smoothing ===
    println!("1. EXPONENTIAL SMOOTHING (OxiDiviner)");
    println!("------------------------------------");

    // Create models with different granularities
    let daily_es = ExponentialSmoothing::new(0.2)?;
    let minute_es = ExponentialSmoothing::new_minute(0.4)?;

    // Train models and generate forecasts
    let trained_daily_es = daily_es.train(&daily_data)?;
    let trained_minute_es = minute_es.train(&minute_data)?;

    let daily_forecast = trained_daily_es.forecast(&daily_data, 5)?;
    let minute_forecast = trained_minute_es.forecast(&minute_data, 5)?;

    println!("Daily ES forecast (next 5 days):");
    for (i, value) in daily_forecast.values.iter().enumerate() {
        println!("  Day {}: {:.2}", i + 1, value);
    }

    println!("\nMinute ES forecast (next 5 minutes):");
    for (i, value) in minute_forecast.values.iter().enumerate() {
        println!("  Minute {}: {:.2}", i + 1, value);
    }

    // === OxiDiviner Moving Average ===
    println!("\n2. MOVING AVERAGE (OxiDiviner)");
    println!("-----------------------------");

    // Create models with different granularities
    let daily_ma = MovingAverage::new(20)?;
    let minute_ma = MovingAverage::new(60)?;

    // Train models and generate forecasts
    let trained_daily_ma = daily_ma.train(&daily_data)?;
    let trained_minute_ma = minute_ma.train(&minute_data)?;

    let daily_ma_forecast = trained_daily_ma.forecast(&daily_data, 5)?;
    let minute_ma_forecast = trained_minute_ma.forecast(&minute_data, 5)?;

    println!("Daily MA forecast (next 5 days):");
    for (i, value) in daily_ma_forecast.values.iter().enumerate() {
        println!("  Day {}: {:.2}", i + 1, value);
    }

    println!("\nMinute MA forecast (next 5 minutes):");
    for (i, value) in minute_ma_forecast.values.iter().enumerate() {
        println!("  Minute {}: {:.2}", i + 1, value);
    }

    // === Using OxiDiviner models with strategies ===
    println!("\n3. STRATEGIES WITH OXIDIVINER MODELS");
    println!("----------------------------------");

    // Mean Reversion Strategy with OxiDiviner ES model
    let daily_mr_strategy = MeanReversionStrategy::new(daily_es, 2.0)?;
    let minute_mr_strategy = MeanReversionStrategy::new_with_granularity(
        minute_es,
        1.5,
        TimeGranularity::Minute,
    )?;

    // Generate signals
    let daily_mr_signals = daily_mr_strategy.generate_signals(&daily_data)?;
    let minute_mr_signals = minute_mr_strategy.generate_signals(&minute_data)?;

    println!("Mean Reversion Strategy:");
    print_signals_summary("Daily", &daily_mr_signals);
    print_signals_summary("Minute", &minute_mr_signals);

    // Trend Following Strategy with OxiDiviner MA model
    let daily_tf_strategy = TrendFollowingStrategy::new(daily_ma, 0.5)?;
    let minute_tf_strategy = TrendFollowingStrategy::new_with_granularity(
        minute_ma,
        0.2,
        TimeGranularity::Minute,
    )?;

    // Generate signals
    let daily_tf_signals = daily_tf_strategy.generate_signals(&daily_data)?;
    let minute_tf_signals = minute_tf_strategy.generate_signals(&minute_data)?;

    println!("\nTrend Following Strategy:");
    print_signals_summary("Daily", &daily_tf_signals);
    print_signals_summary("Minute", &minute_tf_signals);

    // Volatility Breakout Strategy with OxiDiviner ES model
    let daily_vb_strategy = VolatilityBreakoutStrategy::new(ExponentialSmoothing::new(0.2)?, 1.5)?;
    let minute_vb_strategy = VolatilityBreakoutStrategy::new_with_granularity(
        ExponentialSmoothing::new(0.4)?,
        2.0,
        TimeGranularity::Minute,
    )?;

    // Generate signals
    let daily_vb_signals = daily_vb_strategy.generate_signals(&daily_data)?;
    let minute_vb_signals = minute_vb_strategy.generate_signals(&minute_data)?;

    println!("\nVolatility Breakout Strategy:");
    print_signals_summary("Daily", &daily_vb_signals);
    print_signals_summary("Minute", &minute_vb_signals);

    // === Backtesting with OxiDiviner models ===
    println!("\n4. BACKTESTING WITH OXIDIVINER MODELS");
    println!("-----------------------------------");

    let initial_capital = 10000.0;

    // Backtest Mean Reversion Strategy
    println!("Mean Reversion Strategy Backtest:");
    let daily_mr_backtest = daily_mr_strategy.backtest(&daily_data, initial_capital)?;
    let minute_mr_backtest = minute_mr_strategy.backtest(&minute_data, initial_capital)?;

    println!(
        "Daily: ${:.2} final balance, {:.1}% max drawdown, {:.1}% win rate",
        daily_mr_backtest.final_balance,
        daily_mr_backtest.max_drawdown * 100.0,
        daily_mr_backtest.win_rate * 100.0
    );
    println!(
        "Minute: ${:.2} final balance, {:.1}% max drawdown, {:.1}% win rate",
        minute_mr_backtest.final_balance,
        minute_mr_backtest.max_drawdown * 100.0,
        minute_mr_backtest.win_rate * 100.0
    );

    // Backtest Volatility Breakout Strategy
    println!("\nVolatility Breakout Strategy Backtest:");
    let daily_vb_backtest = daily_vb_strategy.backtest(&daily_data, initial_capital)?;
    let minute_vb_backtest = minute_vb_strategy.backtest(&minute_data, initial_capital)?;

    println!(
        "Daily: ${:.2} final balance, {:.1}% max drawdown, {:.1}% win rate",
        daily_vb_backtest.final_balance,
        daily_vb_backtest.max_drawdown * 100.0,
        daily_vb_backtest.win_rate * 100.0
    );
    println!(
        "Minute: ${:.2} final balance, {:.1}% max drawdown, {:.1}% win rate",
        minute_vb_backtest.final_balance,
        minute_vb_backtest.max_drawdown * 100.0,
        minute_vb_backtest.win_rate * 100.0
    );

    Ok(())
}

// Helper function to print signals summary
fn print_signals_summary(granularity: &str, signals: &[TradingSignal]) {
    let buy_count = signals.iter().filter(|&&s| s == TradingSignal::Buy).count();
    let sell_count = signals.iter().filter(|&&s| s == TradingSignal::Sell).count();
    let hold_count = signals.iter().filter(|&&s| s == TradingSignal::Hold).count();

    println!(
        "  {}: {} signals - {} buy, {} sell, {} hold",
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