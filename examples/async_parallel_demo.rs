use polars::prelude::*;
use std::time::Instant;
use tokio::time::Duration;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("🚀 NyxsOwl Async Parallel Processing Demo");
    println!("=========================================");

    // Demo 1: Basic async processing
    demo_basic_async().await?;

    // Demo 2: Parallel data processing
    demo_parallel_processing().await?;

    println!("✅ All async demos completed successfully!");
    Ok(())
}

async fn demo_basic_async() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n⚡ Basic Async Processing");
    println!("========================");

    let start = Instant::now();

    // Simulate async operations
    let tasks = vec![
        simulate_forecast_task("AAPL", 100),
        simulate_forecast_task("MSFT", 150),
        simulate_forecast_task("GOOGL", 200),
    ];

    // Run tasks concurrently
    let results = futures::future::join_all(tasks).await;

    let duration = start.elapsed();

    println!(
        "  📊 Processed {} tasks in {:.2}ms",
        results.len(),
        duration.as_millis()
    );

    for (i, result) in results.iter().enumerate() {
        println!("    Task {}: {:.3}", i + 1, result);
    }

    Ok(())
}

async fn demo_parallel_processing() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 Parallel Data Processing");
    println!("===========================");

    // Create sample datasets
    let datasets = create_sample_datasets();

    let start = Instant::now();

    // Process datasets in parallel
    let processing_tasks: Vec<_> = datasets
        .into_iter()
        .enumerate()
        .map(|(i, df)| async move {
            let result = process_dataset(df, format!("Dataset_{}", i + 1)).await;
            (i + 1, result)
        })
        .collect();

    let results = futures::future::join_all(processing_tasks).await;

    let duration = start.elapsed();

    println!(
        "  📊 Processed {} datasets in {:.2}ms",
        results.len(),
        duration.as_millis()
    );

    for (id, result) in results {
        match result {
            Ok(stats) => println!(
                "    Dataset {}: {} rows processed, avg = {:.2}",
                id, stats.0, stats.1
            ),
            Err(e) => println!("    Dataset {}: Error - {}", id, e),
        }
    }

    Ok(())
}

async fn simulate_forecast_task(symbol: &str, delay_ms: u64) -> f64 {
    // Simulate some processing time
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;

    // Generate a mock forecast value
    let base = symbol.len() as f64;
    base * 123.456 + delay_ms as f64 * 0.001
}

async fn process_dataset(
    df: DataFrame,
    name: String,
) -> std::result::Result<(usize, f64), Box<dyn std::error::Error>> {
    // Simulate async processing
    tokio::time::sleep(Duration::from_millis(50)).await;

    let row_count = df.height();
    let avg_value = df.column("value")?.f64()?.mean().unwrap_or(0.0);

    println!("    🔧 Processing {}: {} rows", name, row_count);

    Ok((row_count, avg_value))
}

fn create_sample_datasets() -> Vec<DataFrame> {
    (0..4)
        .map(|i| {
            let size = if cfg!(test) { 25 + i * 10 } else { 50 + i * 25 }; // Smaller for tests
            let values: Vec<f64> = (0..size)
                .map(|j| 100.0 + (j as f64 * 0.1) + (i as f64 * 10.0))
                .collect();

            df! {
                "id" => (0..size).collect::<Vec<_>>(),
                "value" => values,
            }
            .unwrap()
        })
        .collect()
}
