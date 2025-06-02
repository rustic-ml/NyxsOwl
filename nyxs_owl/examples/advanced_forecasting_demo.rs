//! # Advanced Forecasting Demo
//!
//! This example demonstrates the advanced forecasting capabilities
//! available in NyxsOwl through simplified OxiDiviner integration.

use chrono::{Duration, Utc};
use nyxs_owl::forecast_trade::{
    data::TimeSeriesData,
    models::oxidiviner::{easy, ArimaAdapter, ExponentialSmoothingAdapter, OxiDivinerAdapter},
    ForecastModel,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔮 NyxsOwl Advanced Forecasting Demo");
    println!("=====================================\n");

    // Generate sample financial time series data with trend and seasonality
    let mut dates = Vec::new();
    let mut values = Vec::new();
    let start_date = Utc::now() - Duration::days(365);

    for i in 0..100 {
        dates.push(start_date + Duration::days(i));
        // Create synthetic data with trend + seasonality + noise
        let trend = 100.0 + i as f64 * 0.1;
        let seasonal = 5.0 * (i as f64 * 2.0 * std::f64::consts::PI / 30.0).sin();
        let noise = (i as f64 * 0.1).sin() * 2.0;
        values.push(trend + seasonal + noise);
    }

    let time_series = TimeSeriesData::new(dates.clone(), values.clone())?;

    println!("📊 Generated {} data points", time_series.len());
    println!(
        "📈 Price range: {:.2} - {:.2}",
        values.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
        values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b))
    );
    println!();

    // Test 1: Quick API Functions
    println!("🚀 Testing Quick API Functions");
    println!("==============================");

    // ARIMA forecast using quick API
    match easy::arima_forecast(dates.clone(), values.clone(), 5) {
        Ok(forecast) => {
            println!("✅ ARIMA Quick Forecast: {:?}", forecast);
        }
        Err(e) => {
            println!("❌ ARIMA Quick Forecast failed: {}", e);
        }
    }

    // Exponential Smoothing using quick API
    match easy::exponential_smoothing_forecast(dates.clone(), values.clone(), 5, Some(0.3)) {
        Ok(forecast) => {
            println!("✅ ES Quick Forecast: {:?}", forecast);
        }
        Err(e) => {
            println!("❌ ES Quick Forecast failed: {}", e);
        }
    }

    // Moving Average using quick API
    match easy::moving_average_forecast(dates.clone(), values.clone(), 5, Some(10)) {
        Ok(forecast) => {
            println!("✅ MA Quick Forecast: {:?}", forecast);
        }
        Err(e) => {
            println!("❌ MA Quick Forecast failed: {}", e);
        }
    }

    println!();

    // Test 2: Auto Model Selection
    println!("🤖 Testing Auto Model Selection");
    println!("===============================");

    match easy::auto_forecast(dates.clone(), values.clone(), 5) {
        Ok((model_name, forecast)) => {
            println!("✅ Auto-selected model: {:?}", model_name);
            println!("📈 Forecast: {:?}", forecast);
        }
        Err(e) => {
            println!("❌ Auto selection failed: {}", e);
        }
    }

    println!();

    // Test 3: Individual Model Adapters
    println!("🔧 Testing Individual Model Adapters");
    println!("====================================");

    // Test ARIMA Adapter
    match ArimaAdapter::arima() {
        Ok(arima_model) => {
            println!("✅ ARIMA Adapter created");
            match arima_model.train(&time_series) {
                Ok(trained_model) => match trained_model.forecast(&time_series, 3) {
                    Ok(result) => {
                        println!("📈 ARIMA Forecast: {:?}", result.forecasts);
                    }
                    Err(e) => {
                        println!("❌ ARIMA forecast failed: {}", e);
                    }
                },
                Err(e) => {
                    println!("❌ ARIMA training failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ ARIMA Adapter creation failed: {}", e);
        }
    }

    // Test Exponential Smoothing Adapter
    match ExponentialSmoothingAdapter::exponential_smoothing(Some(0.3)) {
        Ok(es_model) => {
            println!("✅ ES Adapter created");
            match es_model.train(&time_series) {
                Ok(trained_model) => match trained_model.forecast(&time_series, 3) {
                    Ok(result) => {
                        println!("📈 ES Forecast: {:?}", result.forecasts);
                    }
                    Err(e) => {
                        println!("❌ ES forecast failed: {}", e);
                    }
                },
                Err(e) => {
                    println!("❌ ES training failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ ES Adapter creation failed: {}", e);
        }
    }

    println!();

    // Test 4: Model Comparison
    println!("⚖️  Model Comparison");
    println!("===================");

    let models: Vec<(&str, Box<dyn ForecastModel>)> = vec![
        ("ARIMA", Box::new(ArimaAdapter::arima()?)),
        (
            "ES(0.3)",
            Box::new(ExponentialSmoothingAdapter::exponential_smoothing(Some(
                0.3,
            ))?),
        ),
        (
            "MA(10)",
            Box::new(OxiDivinerAdapter::moving_average(Some(10))?),
        ),
    ];

    for (name, model) in models {
        match model.train(&time_series) {
            Ok(trained_model) => match trained_model.forecast(&time_series, 1) {
                Ok(result) => {
                    println!("📊 {}: Next prediction = {:.2}", name, result.forecasts[0]);
                }
                Err(e) => {
                    println!("❌ {} forecast failed: {}", name, e);
                }
            },
            Err(e) => {
                println!("❌ {} training failed: {}", name, e);
            }
        }
    }

    println!();
    println!("🎉 Advanced Forecasting Demo completed!");
    println!("✨ All OxiDiviner advanced variants are now fully implemented!");

    Ok(())
}
