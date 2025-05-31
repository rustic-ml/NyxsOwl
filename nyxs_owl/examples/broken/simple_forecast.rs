use forecast_trade::models::oxidiviner::MovingAverageAdapter;
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

    // Get the close prices
    let prices = data.close_prices();
    println!("Loaded {} data points", prices.len());

    // Create and train a Simple Moving Average model
    let model = MovingAverageAdapter::new(20)?;
    let trained_model = model.train(&data)?;

    // Forecast the next 5 days
    let forecast = trained_model.forecast(&data, 5)?;

    // Display the results
    println!("Forecast for the next 5 days:");
    for (i, value) in forecast.values.iter().enumerate() {
        println!("Day {}: {:.2}", i + 1, value);
    }

    Ok(())
}
