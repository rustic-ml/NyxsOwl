use day_trade::strategies::hold::moving_average::MovingAverageCrossoverStrategy;
use day_trade::{DailyOhlcv, Signal};
use forecast_trade::models::arima::ArimaModel;
use forecast_trade::strategies::ForecastStrategy;
use forecast_trade::strategies::TrendFollowingStrategy;
use forecast_trade::{DataLoader, ForecastModel};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load data from CSV
    let csv_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("csv")
        .join("daily_data.csv");

    println!("Loading data from: {}", csv_path.display());
    let data = DataLoader::from_csv(csv_path)?;

    // Convert to DailyOhlcv format for trading strategy
    let daily_data = data.to_daily_ohlcv()?;

    // Split data into historical and test periods
    let history_len = daily_data.len() * 8 / 10; // Use 80% for history
    let (historical_data, test_data) = daily_data.split_at(history_len);

    println!("Historical data: {} days", historical_data.len());
    println!("Test data: {} days", test_data.len());

    // Create and fit a forecasting model
    let mut forecast_model = ArimaModel::new(2, 1, 2);
    forecast_model.fit(&data.close_prices()[..history_len])?;

    // Create a forecast-based trading strategy
    let forecast_strategy = TrendFollowingStrategy::new(1.5);

    // Generate forecasts for the future
    let forecast_horizon = 5; // Forecast 5 days ahead
    let forecast = forecast_model.forecast(forecast_horizon)?;

    // Generate trading signals based on the forecast
    let signals = forecast_strategy.generate_signals(&forecast)?;

    println!("Forecast values for the next {} days:", forecast_horizon);
    for (i, value) in forecast.values.iter().enumerate() {
        println!("Day {}: {:.2} -> {:?}", i + 1, value, signals[i]);
    }

    // Now compare with a traditional moving average strategy
    let ma_strategy = MovingAverageCrossoverStrategy::new(10, 30);
    let ma_signals = ma_strategy.generate_signals(&historical_data)?;

    println!("\nLast 5 MA strategy signals:");
    for i in ma_signals.len().saturating_sub(5)..ma_signals.len() {
        println!("Day {}: {:?}", historical_data[i].date, ma_signals[i]);
    }

    Ok(())
}
