use chrono::{Duration, TimeZone, Utc};
use forecast_trade::data::TimeSeriesData;
use forecast_trade::models::exponential_smoothing::ExponentialSmoothing;
use forecast_trade::strategies::mean_reversion::MeanReversionStrategy;
use forecast_trade::strategies::trend_following::TrendFollowingStrategy;
use forecast_trade::strategies::volatility_breakout::VolatilityBreakoutStrategy;
use forecast_trade::strategies::{BacktestResults, ForecastStrategy, TimeGranularity};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Forecast Trade: Backtest Strategy Example");
    println!("=========================================\n");

    // Create sample data
    println!("Creating sample data...");
    let daily_data = create_sample_daily_data();
    let minute_data = create_sample_minute_data();

    println!(
        "Sample data created: {} daily points, {} minute points\n",
        daily_data.len(),
        minute_data.len()
    );

    // Create models
    println!("Creating models...");
    let daily_model = ExponentialSmoothing::new(0.7)?;
    let minute_model = ExponentialSmoothing::new(0.3)?;

    // Create strategies
    println!("Creating strategies...");

    // Create daily strategies
    let daily_mr = MeanReversionStrategy::new_with_granularity(
        daily_model.clone(),
        2.0,
        TimeGranularity::Daily,
    )?;

    let daily_tf = TrendFollowingStrategy::new_with_granularity(
        daily_model.clone(),
        20,
        TimeGranularity::Daily,
    )?;

    let daily_vb = VolatilityBreakoutStrategy::new_with_granularity(
        daily_model.clone(),
        1.5,
        TimeGranularity::Daily,
    )?;

    // Create minute strategies
    let minute_mr = MeanReversionStrategy::new_with_granularity(
        minute_model.clone(),
        1.5,
        TimeGranularity::Minute,
    )?;

    let minute_tf = TrendFollowingStrategy::new_with_granularity(
        minute_model.clone(),
        60,
        TimeGranularity::Minute,
    )?;

    let minute_vb = VolatilityBreakoutStrategy::new_with_granularity(
        minute_model.clone(),
        1.2,
        TimeGranularity::Minute,
    )?;

    // Run backtests
    println!("\nRunning backtests with initial balance of $10,000...");

    // Daily backtests
    println!("\nDaily Backtests:");
    let daily_mr_results = daily_mr.backtest(&daily_data, 10000.0)?;
    let daily_tf_results = daily_tf.backtest(&daily_data, 10000.0)?;
    let daily_vb_results = daily_vb.backtest(&daily_data, 10000.0)?;

    // Minute backtests
    println!("\nMinute Backtests:");
    let minute_mr_results = minute_mr.backtest(&minute_data, 10000.0)?;
    let minute_tf_results = minute_tf.backtest(&minute_data, 10000.0)?;
    let minute_vb_results = minute_vb.backtest(&minute_data, 10000.0)?;

    // Compare results
    println!("\nResults Comparison:");
    println!("\nDaily Strategies:");
    println!("  Mean Reversion:");
    print_backtest_results(&daily_mr_results);
    println!("  Trend Following:");
    print_backtest_results(&daily_tf_results);
    println!("  Volatility Breakout:");
    print_backtest_results(&daily_vb_results);

    println!("\nMinute Strategies:");
    println!("  Mean Reversion:");
    print_backtest_results(&minute_mr_results);
    println!("  Trend Following:");
    print_backtest_results(&minute_tf_results);
    println!("  Volatility Breakout:");
    print_backtest_results(&minute_vb_results);

    // Run custom backtest with different transaction costs
    println!("\nCustom Backtest with Different Transaction Costs:");
    println!(
        "\nDaily Mean Reversion with High Transaction Costs (0.5% commission, 0.2% slippage):"
    );
    let high_cost_results = daily_mr.backtest_with_params(&daily_data, 10000.0, 0.005, 0.002)?;
    print_backtest_results(&high_cost_results);

    println!(
        "\nDaily Mean Reversion with Low Transaction Costs (0.05% commission, 0.01% slippage):"
    );
    let low_cost_results = daily_mr.backtest_with_params(&daily_data, 10000.0, 0.0005, 0.0001)?;
    print_backtest_results(&low_cost_results);

    println!("\nKey Observations:");
    println!("1. Transaction costs significantly impact trading performance");
    println!("2. Different timeframes require different strategy parameters");
    println!("3. Each strategy performs differently depending on market conditions");
    println!("4. Mean reversion tends to perform better in range-bound markets");
    println!("5. Trend following tends to perform better in trending markets");
    println!("6. Volatility breakout strategies work well during market regime changes");

    Ok(())
}

// Helper function to create sample daily data with trends and volatility clusters
fn create_sample_daily_data() -> TimeSeriesData {
    let start_date = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap();
    let mut dates = Vec::with_capacity(200);
    let mut prices = Vec::with_capacity(200);

    let mut price = 100.0;
    for i in 0..200 {
        dates.push(start_date + Duration::days(i));

        // Create different market regimes
        let regime = (i / 50) % 3; // 0 = trending up, 1 = range bound, 2 = trending down

        match regime {
            0 => {
                // Trending up
                let trend = 0.001 * i as f64;
                let cycle = (i as f64 / 10.0).sin() * 2.0;
                let noise = (i as f64).cos() * 0.5;
                price = price + trend + cycle + noise;
            }
            1 => {
                // Range bound
                let mean = 150.0;
                let reversion = (mean - price) * 0.05;
                let noise = (i as f64 * 2.0).sin() * 3.0;
                price = price + reversion + noise;
            }
            2 => {
                // Trending down
                let trend = -0.001 * i as f64;
                let cycle = (i as f64 / 10.0).sin() * 2.0;
                let noise = (i as f64).cos() * 0.5;
                price = price + trend + cycle + noise;
            }
            _ => unreachable!(),
        }

        // Ensure price is positive
        price = price.max(50.0);
        prices.push(price);
    }

    TimeSeriesData::new(dates, prices).unwrap()
}

// Helper function to create sample minute data with more noise
fn create_sample_minute_data() -> TimeSeriesData {
    let start_date = Utc.with_ymd_and_hms(2023, 1, 1, 9, 30, 0).unwrap();
    let mut dates = Vec::with_capacity(480);
    let mut prices = Vec::with_capacity(480);

    let mut price = 100.0;
    for i in 0..480 {
        dates.push(start_date + Duration::minutes(i));

        // Create different market regimes by hour
        let hour = i / 60;
        let regime = hour % 3; // 0 = trending up, 1 = range bound, 2 = trending down

        match regime {
            0 => {
                // Trending up
                let trend = 0.0002 * i as f64;
                let cycle = (i as f64 / 30.0).sin() * 0.2;
                let noise = (i as f64 * 5.0).cos() * 0.1;
                price = price + trend + cycle + noise;
            }
            1 => {
                // Range bound
                let mean = price;
                let reversion = ((mean - price) * 0.1).min(0.1).max(-0.1);
                let noise = (i as f64 * 10.0).sin() * 0.3;
                price = price + reversion + noise;
            }
            2 => {
                // Trending down
                let trend = -0.0002 * i as f64;
                let cycle = (i as f64 / 30.0).sin() * 0.2;
                let noise = (i as f64 * 5.0).cos() * 0.1;
                price = price + trend + cycle + noise;
            }
            _ => unreachable!(),
        }

        // Add volatility clusters
        if (i % 120) >= 90 {
            // Higher volatility period
            price += (i as f64 * 20.0).sin() * 0.3;
        }

        // Ensure price is positive
        price = price.max(50.0);
        prices.push(price);
    }

    TimeSeriesData::new(dates, prices).unwrap()
}

// Helper function to print backtest results
fn print_backtest_results(results: &BacktestResults) {
    println!("    Final balance: ${:.2}", results.final_balance);
    println!(
        "    ROI: {:.2}%",
        (results.final_balance / 10000.0 - 1.0) * 100.0
    );
    println!("    Total trades: {}", results.total_trades);
    println!("    Win rate: {:.1}%", results.win_rate * 100.0);
    println!("    Max drawdown: {:.1}%", results.max_drawdown * 100.0);

    if let Some(sharpe) = results.performance_metrics.sharpe_ratio {
        println!("    Sharpe ratio: {:.2}", sharpe);
    }
}
