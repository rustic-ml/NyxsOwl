//! Test to check available OxiDiviner imports

use chrono::{DateTime, Duration, Utc};
use oxidiviner::TimeSeriesData;

fn main() {
    // Test basic imports from oxidiviner
    println!("Testing OxiDiviner imports...");

    // Create sample data
    let mut dates = Vec::new();
    let mut values = Vec::new();
    let start_date = Utc::now() - Duration::days(30);

    for i in 0..30 {
        dates.push(start_date + Duration::days(i));
        values.push(100.0 + i as f64 + (i as f64 * 0.1).sin() * 5.0);
    }

    // Test TimeSeriesData creation
    match TimeSeriesData::new(dates, values, "test_data") {
        Ok(ts_data) => {
            println!("✅ TimeSeriesData created successfully");

            // Try the quick API
            match oxidiviner::quick::auto_select(ts_data, 5) {
                Ok((forecast, model)) => {
                    println!(
                        "✅ Quick API works: {} - {:?}",
                        model,
                        &forecast[..3.min(forecast.len())]
                    );
                }
                Err(e) => {
                    println!("❌ Quick API error: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ TimeSeriesData creation error: {}", e);
        }
    }

    println!("OxiDiviner import test completed.");
}
