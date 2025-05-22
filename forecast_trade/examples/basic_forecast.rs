use chrono::{Datelike, Duration, TimeZone, Timelike, Utc};
use forecast_trade::data::TimeSeriesData;
use forecast_trade::models::exponential_smoothing::ExponentialSmoothing;
use forecast_trade::models::{ForecastModel, TrainedForecastModel};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Forecast Trade: Basic Forecasting Example");
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

    // Create models with different parameters for daily and minute data
    println!("Training models...");

    // Daily model - longer smoothing (less reactive)
    let daily_model = ExponentialSmoothing::new(0.2)?;
    let trained_daily_model = daily_model.train(&daily_data)?;

    // Minute model - shorter smoothing (more reactive)
    let minute_model = ExponentialSmoothing::new(0.4)?;
    let trained_minute_model = minute_model.train(&minute_data)?;

    println!("Models trained successfully\n");

    // Generate forecasts
    println!("Generating forecasts...");
    let daily_forecast = trained_daily_model.forecast(5)?;
    println!("Daily forecast (5 days): {:?}", daily_forecast.values());

    // For minute data, forecast 30 minutes ahead
    let minute_forecast = trained_minute_model.forecast(30)?;
    println!(
        "Minute forecast (30 minutes): {:?}",
        minute_forecast.values()
    );

    println!("\nForecasting complete!");

    // Calculate forecast confidence intervals
    println!("\nForecast confidence intervals:");

    // 95% confidence interval for daily forecast
    let daily_ci = daily_forecast.confidence_intervals(0.95)?;
    println!("\nDaily forecast 95% confidence intervals:");
    for (i, (lower, upper)) in daily_ci.iter().enumerate() {
        println!("  Day {}: ({:.2}, {:.2})", i + 1, lower, upper);
    }

    // 95% confidence interval for minute forecast
    let minute_ci = minute_forecast.confidence_intervals(0.95)?;
    println!("\nMinute forecast 95% confidence intervals (every 5th minute):");
    for (i, (lower, upper)) in minute_ci.iter().enumerate() {
        if i % 5 == 0 {
            println!("  Minute {}: ({:.2}, {:.2})", i + 1, lower, upper);
        }
    }

    println!("\nSummary:");
    println!("1. Different alpha values are used for daily vs. minute data");
    println!("2. Daily data requires higher alpha (0.7) due to less noise");
    println!("3. Minute data uses lower alpha (0.3) to smooth out noise");
    println!("4. Confidence intervals help quantify forecast uncertainty");

    Ok(())
}

/// Create sample daily data with a trend and some seasonality
fn create_sample_daily_data() -> TimeSeriesData {
    let mut dates = Vec::with_capacity(100);
    let mut prices = Vec::with_capacity(100);

    let start_date = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap();

    // Create 100 days of data with a trend and some seasonality
    let mut price = 100.0;
    let trend = 0.05; // 0.05 points per day upward trend

    for i in 0..100 {
        let current_date = start_date + Duration::days(i);
        dates.push(current_date);

        // Add some weekly seasonality
        let day_of_week = current_date.weekday().num_days_from_monday() as f64;
        let seasonality = (day_of_week * std::f64::consts::PI / 7.0).sin() * 2.0;

        // Add some noise
        let noise = (i as f64 * 0.1).sin() * 1.0;

        price = price + trend + seasonality + noise;
        prices.push(price);
    }

    TimeSeriesData::new(dates, prices).unwrap()
}

/// Create sample minute data with higher volatility
fn create_sample_minute_data() -> TimeSeriesData {
    let mut dates = Vec::with_capacity(500);
    let mut prices = Vec::with_capacity(500);

    let start_date = Utc.with_ymd_and_hms(2023, 1, 1, 9, 0, 0).unwrap();

    // Create 500 minutes of data (about 8 hours)
    let mut price = 100.0;
    let trend = 0.002; // Smaller trend per minute

    for i in 0..500 {
        let current_date = start_date + Duration::minutes(i);
        dates.push(current_date);

        // Add some intraday pattern (U-shape)
        let minute_of_day = current_date.hour() * 60 + current_date.minute();
        let normalized_time = minute_of_day as f64 / (24.0 * 60.0);
        let intraday = ((normalized_time - 0.5) * 2.0).powi(2) * 1.0;

        // Add higher frequency noise
        let noise = (i as f64 * 0.5).sin() * 0.2 + (i as f64 * 0.3).cos() * 0.3;

        price = price + trend + intraday + noise;
        prices.push(price);
    }

    TimeSeriesData::new(dates, prices).unwrap()
}
