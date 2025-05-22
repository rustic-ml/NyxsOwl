use chrono::{Duration, TimeZone, Utc};
use forecast_trade::data::TimeSeriesData;
use forecast_trade::models::oxidiviner::{Autoregressive, ExponentialSmoothing, Garch, MovingAverage};
use forecast_trade::strategies::arima_strategy::ArimaStrategy;
use forecast_trade::strategies::mean_reversion::MeanReversionStrategy;
use forecast_trade::strategies::trend_following::TrendFollowingStrategy;
use forecast_trade::strategies::volatility_breakout::VolatilityBreakoutStrategy;
use forecast_trade::strategies::volatility_strategy::VolatilityStrategy;
use forecast_trade::strategies::{ForecastStrategy, TimeGranularity, TradingSignal};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Forecast Trade: Complete OxiDiviner Strategies Example");
    println!("===================================================\n");

    // Create sample data for both daily and minute granularities
    let daily_data = create_sample_daily_data();
    let minute_data = create_sample_minute_data();

    println!(
        "Created sample data: {} daily records, {} minute records\n",
        daily_data.len(),
        minute_data.len()
    );

    // ======================================================
    // 1. EXPONENTIAL SMOOTHING WITH MEAN REVERSION STRATEGY
    // ======================================================
    println!("1. EXPONENTIAL SMOOTHING WITH MEAN REVERSION STRATEGY");
    println!("--------------------------------------------------");

    // Create models with different granularities
    let daily_es = ExponentialSmoothing::new(0.2)?;
    let minute_es = ExponentialSmoothing::new_minute(0.4)?;

    // Create strategies
    let daily_mr_strategy = MeanReversionStrategy::new(daily_es, 2.0)?;
    let minute_mr_strategy = MeanReversionStrategy::new_with_granularity(
        minute_es,
        1.5,
        TimeGranularity::Minute,
    )?;

    // Generate signals and run backtest
    let daily_mr_signals = daily_mr_strategy.generate_signals(&daily_data)?;
    let minute_mr_signals = minute_mr_strategy.generate_signals(&minute_data)?;

    println!("Mean Reversion Strategy:");
    print_signals_summary("Daily", &daily_mr_signals);
    print_signals_summary("Minute", &minute_mr_signals);

    let initial_capital = 10000.0;
    
    let daily_mr_backtest = daily_mr_strategy.backtest(&daily_data, initial_capital)?;
    let minute_mr_backtest = minute_mr_strategy.backtest(&minute_data, initial_capital)?;

    println!("\nBacktest Results:");
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

    // ======================================================
    // 2. MOVING AVERAGE WITH TREND FOLLOWING STRATEGY
    // ======================================================
    println!("\n2. MOVING AVERAGE WITH TREND FOLLOWING STRATEGY");
    println!("--------------------------------------------");

    // Create models with different granularities
    let daily_ma = MovingAverage::new(20)?;
    let minute_ma = MovingAverage::new(60)?;

    // Create strategies
    let daily_tf_strategy = TrendFollowingStrategy::new(daily_ma, 0.5)?;
    let minute_tf_strategy = TrendFollowingStrategy::new_with_granularity(
        minute_ma,
        0.2,
        TimeGranularity::Minute,
    )?;

    // Generate signals and run backtest
    let daily_tf_signals = daily_tf_strategy.generate_signals(&daily_data)?;
    let minute_tf_signals = minute_tf_strategy.generate_signals(&minute_data)?;

    println!("Trend Following Strategy:");
    print_signals_summary("Daily", &daily_tf_signals);
    print_signals_summary("Minute", &minute_tf_signals);

    let daily_tf_backtest = daily_tf_strategy.backtest(&daily_data, initial_capital)?;
    let minute_tf_backtest = minute_tf_strategy.backtest(&minute_data, initial_capital)?;

    println!("\nBacktest Results:");
    println!(
        "Daily: ${:.2} final balance, {:.1}% max drawdown, {:.1}% win rate",
        daily_tf_backtest.final_balance,
        daily_tf_backtest.max_drawdown * 100.0,
        daily_tf_backtest.win_rate * 100.0
    );
    println!(
        "Minute: ${:.2} final balance, {:.1}% max drawdown, {:.1}% win rate",
        minute_tf_backtest.final_balance,
        minute_tf_backtest.max_drawdown * 100.0,
        minute_tf_backtest.win_rate * 100.0
    );

    // ======================================================
    // 3. ARIMA MODEL WITH ARIMA STRATEGY
    // ======================================================
    println!("\n3. ARIMA MODEL WITH ARIMA STRATEGY");
    println!("--------------------------------");

    // Create models with different granularities
    let daily_arima = Autoregressive::new(1, 1, 1)?;
    let minute_arima = Autoregressive::new_minute(2, 1, 2)?;

    // Create strategies
    let daily_arima_strategy = ArimaStrategy::new(daily_arima, 1.0)?;
    let minute_arima_strategy = ArimaStrategy::new_with_granularity(
        minute_arima,
        0.5,
        TimeGranularity::Minute,
    )?;

    // Generate signals and run backtest
    let daily_arima_signals = daily_arima_strategy.generate_signals(&daily_data)?;
    let minute_arima_signals = minute_arima_strategy.generate_signals(&minute_data)?;

    println!("ARIMA Strategy:");
    print_signals_summary("Daily", &daily_arima_signals);
    print_signals_summary("Minute", &minute_arima_signals);

    let daily_arima_backtest = daily_arima_strategy.backtest(&daily_data, initial_capital)?;
    let minute_arima_backtest = minute_arima_strategy.backtest(&minute_data, initial_capital)?;

    println!("\nBacktest Results:");
    println!(
        "Daily: ${:.2} final balance, {:.1}% max drawdown, {:.1}% win rate",
        daily_arima_backtest.final_balance,
        daily_arima_backtest.max_drawdown * 100.0,
        daily_arima_backtest.win_rate * 100.0
    );
    println!(
        "Minute: ${:.2} final balance, {:.1}% max drawdown, {:.1}% win rate",
        minute_arima_backtest.final_balance,
        minute_arima_backtest.max_drawdown * 100.0,
        minute_arima_backtest.win_rate * 100.0
    );

    // ======================================================
    // 4. GARCH MODEL WITH VOLATILITY STRATEGY
    // ======================================================
    println!("\n4. GARCH MODEL WITH VOLATILITY STRATEGY");
    println!("------------------------------------");

    // Create models with different granularities
    let daily_garch = Garch::new(1, 1)?;
    let minute_garch = Garch::new_minute(2, 1)?;

    // Create strategies
    let daily_vol_strategy = VolatilityStrategy::new(daily_garch, 1.5)?;
    let minute_vol_strategy = VolatilityStrategy::new_with_granularity(
        minute_garch,
        1.2,
        TimeGranularity::Minute,
    )?;

    // Generate signals and run backtest
    let daily_vol_signals = daily_vol_strategy.generate_signals(&daily_data)?;
    let minute_vol_signals = minute_vol_strategy.generate_signals(&minute_data)?;

    println!("Volatility Strategy:");
    print_signals_summary("Daily", &daily_vol_signals);
    print_signals_summary("Minute", &minute_vol_signals);

    let daily_vol_backtest = daily_vol_strategy.backtest(&daily_data, initial_capital)?;
    let minute_vol_backtest = minute_vol_strategy.backtest(&minute_data, initial_capital)?;

    println!("\nBacktest Results:");
    println!(
        "Daily: ${:.2} final balance, {:.1}% max drawdown, {:.1}% win rate",
        daily_vol_backtest.final_balance,
        daily_vol_backtest.max_drawdown * 100.0,
        daily_vol_backtest.win_rate * 100.0
    );
    println!(
        "Minute: ${:.2} final balance, {:.1}% max drawdown, {:.1}% win rate",
        minute_vol_backtest.final_balance,
        minute_vol_backtest.max_drawdown * 100.0,
        minute_vol_backtest.win_rate * 100.0
    );

    // ======================================================
    // 5. STRATEGY PERFORMANCE COMPARISON
    // ======================================================
    println!("\n5. STRATEGY PERFORMANCE COMPARISON");
    println!("-------------------------------");

    println!("\nDaily Strategies Performance:");
    println!("-----------------------------");
    println!(
        "Mean Reversion (ES):  ${:.2} ({:.1}%), Drawdown: {:.1}%, Win Rate: {:.1}%",
        daily_mr_backtest.final_balance,
        (daily_mr_backtest.final_balance - initial_capital) / initial_capital * 100.0,
        daily_mr_backtest.max_drawdown * 100.0,
        daily_mr_backtest.win_rate * 100.0
    );
    println!(
        "Trend Following (MA): ${:.2} ({:.1}%), Drawdown: {:.1}%, Win Rate: {:.1}%",
        daily_tf_backtest.final_balance,
        (daily_tf_backtest.final_balance - initial_capital) / initial_capital * 100.0,
        daily_tf_backtest.max_drawdown * 100.0,
        daily_tf_backtest.win_rate * 100.0
    );
    println!(
        "ARIMA Strategy:       ${:.2} ({:.1}%), Drawdown: {:.1}%, Win Rate: {:.1}%",
        daily_arima_backtest.final_balance,
        (daily_arima_backtest.final_balance - initial_capital) / initial_capital * 100.0,
        daily_arima_backtest.max_drawdown * 100.0,
        daily_arima_backtest.win_rate * 100.0
    );
    println!(
        "Volatility (GARCH):   ${:.2} ({:.1}%), Drawdown: {:.1}%, Win Rate: {:.1}%",
        daily_vol_backtest.final_balance,
        (daily_vol_backtest.final_balance - initial_capital) / initial_capital * 100.0,
        daily_vol_backtest.max_drawdown * 100.0,
        daily_vol_backtest.win_rate * 100.0
    );

    println!("\nMinute Strategies Performance:");
    println!("-----------------------------");
    println!(
        "Mean Reversion (ES):  ${:.2} ({:.1}%), Drawdown: {:.1}%, Win Rate: {:.1}%",
        minute_mr_backtest.final_balance,
        (minute_mr_backtest.final_balance - initial_capital) / initial_capital * 100.0,
        minute_mr_backtest.max_drawdown * 100.0,
        minute_mr_backtest.win_rate * 100.0
    );
    println!(
        "Trend Following (MA): ${:.2} ({:.1}%), Drawdown: {:.1}%, Win Rate: {:.1}%",
        minute_tf_backtest.final_balance,
        (minute_tf_backtest.final_balance - initial_capital) / initial_capital * 100.0,
        minute_tf_backtest.max_drawdown * 100.0,
        minute_tf_backtest.win_rate * 100.0
    );
    println!(
        "ARIMA Strategy:       ${:.2} ({:.1}%), Drawdown: {:.1}%, Win Rate: {:.1}%",
        minute_arima_backtest.final_balance,
        (minute_arima_backtest.final_balance - initial_capital) / initial_capital * 100.0,
        minute_arima_backtest.max_drawdown * 100.0,
        minute_arima_backtest.win_rate * 100.0
    );
    println!(
        "Volatility (GARCH):   ${:.2} ({:.1}%), Drawdown: {:.1}%, Win Rate: {:.1}%",
        minute_vol_backtest.final_balance,
        (minute_vol_backtest.final_balance - initial_capital) / initial_capital * 100.0,
        minute_vol_backtest.max_drawdown * 100.0,
        minute_vol_backtest.win_rate * 100.0
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
    let mut dates = Vec::with_capacity(200);
    let mut prices = Vec::with_capacity(200);

    let start_date = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap();

    // Create 200 days of data with trend, cycle and some seasonality
    let mut price = 100.0;
    let trend = 0.05; // 0.05 points per day upward trend

    for i in 0..200 {
        let current_date = start_date + Duration::days(i);
        dates.push(current_date);

        // Add some weekly seasonality
        let day_of_week = current_date.weekday().num_days_from_monday() as f64;
        let seasonality = (day_of_week * std::f64::consts::PI / 7.0).sin() * 2.0;
        
        // Add longer cycle (40 day cycle)
        let cycle = (i as f64 * std::f64::consts::PI / 20.0).sin() * 10.0;
        
        // Add noise
        let noise = (i as f64 * 0.1).sin() * 1.0 + rand_normal(0.0, 0.5);

        price = price + trend + seasonality + cycle * 0.1 + noise;
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

        // Add higher frequency cycles
        let fast_cycle = (i as f64 * std::f64::consts::PI / 30.0).sin() * 0.3;
        let medium_cycle = (i as f64 * std::f64::consts::PI / 60.0).sin() * 0.5;
        
        // Add noise
        let noise = rand_normal(0.0, 0.1);

        price = price + trend + intraday + fast_cycle + medium_cycle + noise;
        prices.push(price);
    }

    TimeSeriesData::new(dates, prices).unwrap()
}

// Helper function to generate normally distributed random numbers
fn rand_normal(mean: f64, std_dev: f64) -> f64 {
    use std::f64::consts::PI;
    
    // Box-Muller transform
    let u1 = rand();
    let u2 = rand();
    
    let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos();
    
    mean + std_dev * z0
}

// Simple random number generator between 0 and 1
fn rand() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() as f64;
    
    (now % 1000.0) / 1000.0
} 