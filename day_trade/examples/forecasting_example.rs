use trade_math::forecasting::{DoubleExponentialSmoothing, ExponentialSmoothing, LinearRegression};

fn main() {
    println!("Forecasting Methods Example");
    println!("==========================\n");

    // Sample price data (simulated stock prices)
    let prices = vec![
        100.0, 102.5, 101.8, 104.3, 107.1, 106.5, 108.2, 110.0, 109.7, 111.5, 113.2, 114.8, 116.4,
        115.9, 117.2, 119.0, 121.5, 122.8, 124.0, 123.5,
    ];

    println!("Using {} price points", prices.len());

    // 1. Linear Regression Example
    println!("\n1. Linear Regression");
    println!("-------------------");

    let mut lr = LinearRegression::new(10).unwrap();

    // Process all prices
    for (i, &price) in prices.iter().enumerate() {
        lr.update(price).unwrap();

        // Once we have enough data, show regression stats
        if i >= 9 {
            let slope = lr.slope().unwrap();
            let forecast_next = lr.forecast(1).unwrap();
            let r_squared = lr.r_squared().unwrap();

            println!(
                "After {} points: Slope = {:.4}, R² = {:.4}, Next forecast = {:.2}",
                i + 1,
                slope,
                r_squared,
                forecast_next
            );
        }
    }

    // 2. Simple Exponential Smoothing
    println!("\n2. Simple Exponential Smoothing");
    println!("-----------------------------");

    let mut es = ExponentialSmoothing::new(0.3).unwrap();

    for (i, &price) in prices.iter().enumerate() {
        es.update(price).unwrap();

        let smoothed = es.value().unwrap();
        println!("Price: {:.2}, Smoothed: {:.2}", price, smoothed);

        if i == prices.len() - 1 {
            let forecast = es.forecast().unwrap();
            println!("Forecast for next period: {:.2}", forecast);
        }
    }

    // 3. Double Exponential Smoothing (Holt's method)
    println!("\n3. Double Exponential Smoothing (Holt's method)");
    println!("-------------------------------------------");

    let mut des = DoubleExponentialSmoothing::new(0.4, 0.3).unwrap();

    for (i, &price) in prices.iter().enumerate() {
        des.update(price).unwrap();

        if i > 0 {
            let level = des.level().unwrap();
            let trend = des.trend().unwrap();

            println!(
                "Price: {:.2}, Level: {:.2}, Trend: {:.4}",
                price, level, trend
            );

            if i == prices.len() - 1 {
                // Forecast 1, 3, and 5 periods ahead
                let forecast1 = des.forecast(1).unwrap();
                let forecast3 = des.forecast(3).unwrap();
                let forecast5 = des.forecast(5).unwrap();

                println!("\nForecasts:");
                println!("1 period ahead: {:.2}", forecast1);
                println!("3 periods ahead: {:.2}", forecast3);
                println!("5 periods ahead: {:.2}", forecast5);
            }
        }
    }
}
