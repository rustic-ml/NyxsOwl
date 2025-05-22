use forecast_trade::models::moving_average::SimpleMA;
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

    // Create and fit a Simple Moving Average model
    let mut model = SimpleMA::new(20)?;
    model.fit(&prices)?;

    // Forecast the next 5 days
    let forecast = model.forecast(5)?;

    // Display the results
    println!("Forecast for the next 5 days using {}:", model.name());
    for (i, value) in forecast.values.iter().enumerate() {
        println!("Day {}: {:.2}", i + 1, value);
    }

    Ok(())
}
