//! Debug TimeSeriesData to understand the conversion issue

use chrono::{Duration, Utc};
use nyxs_owl::forecast_trade::data::TimeSeriesData;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Debugging TimeSeriesData creation...");

    // Generate sample financial time series data
    let mut dates = Vec::new();
    let mut values = Vec::new();
    let start_date = Utc::now() - Duration::days(10);

    for i in 0..10 {
        dates.push(start_date + Duration::days(i));
        values.push(100.0 + i as f64);
    }

    println!("📊 Created {} dates and {} values", dates.len(), values.len());
    println!("📅 Date range: {} to {}", dates[0], dates[dates.len() - 1]);
    println!("💰 Value range: {} to {}", values[0], values[values.len() - 1]);

    let time_series = TimeSeriesData::new(dates.clone(), values.clone())?;
    
    println!("✅ TimeSeriesData created successfully");
    println!("📈 Length: {}", time_series.len());
    println!("🔄 Is empty: {}", time_series.is_empty());
    
    let retrieved_dates = time_series.timestamps();
    let retrieved_values = time_series.close_prices();
    
    println!("🔍 Retrieved {} dates and {} values", retrieved_dates.len(), retrieved_values.len());
    
    if retrieved_dates.is_empty() || retrieved_values.is_empty() {
        println!("❌ Problem: Retrieved data is empty!");
        
        // Debug the dataframe structure
        println!("🔧 Debugging dataframe:");
        println!("  Column names: {:?}", time_series.dataframe().get_column_names());
        println!("  Time column: {}", time_series.time_column());
        println!("  Price columns: {:?}", time_series.price_columns());
        
        // Try to examine the columns directly
        for col_name in time_series.dataframe().get_column_names() {
            let col = time_series.dataframe().column(col_name).unwrap();
            println!("  Column '{}': dtype={:?}, len={}", col_name, col.dtype(), col.len());
        }
    } else {
        println!("✅ Retrieved data successfully!");
        println!("📅 First retrieved date: {}", retrieved_dates[0]);
        println!("💰 First retrieved value: {}", retrieved_values[0]);
    }

    Ok(())
} 