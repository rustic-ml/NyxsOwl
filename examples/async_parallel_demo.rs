use nyxs_owl::prelude::*;
use nyxs_owl::async_parallel::{
    AsyncParallelProcessor, ParallelConfig, ForecastTask, 
    AsyncDataPipeline, MarketData, ProcessedMarketData
};
use nyxs_owl::memory_optimized::CacheOptimizedTimeSeries;
use nyxs_owl::forecasting::strategies::arima_strategy::{ArimaStrategy, ArimaStrategyConfig};
use polars::prelude::*;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("🚀 NyxsOwl Async/Parallel Processing Demo");
    println!("=========================================\n");

    // Demo 1: Basic Async/Parallel Processor
    demo_basic_async_processing().await?;
    
    // Demo 2: Parallel Market Data Processing
    demo_parallel_market_data_processing().await?;
    
    // Demo 3: Ensemble Forecasting with Parallel Processing
    demo_ensemble_parallel_forecasting().await?;
    
    // Demo 4: Real-time Data Pipeline
    demo_async_data_pipeline().await?;
    
    // Demo 5: ARIMA Strategy with Async Processing
    demo_arima_async_signals().await?;
    
    // Demo 6: Performance Comparison
    demo_performance_comparison().await?;

    println!("\n✅ All async/parallel processing demos completed successfully!");
    Ok(())
}

/// Demo 1: Basic Async/Parallel Processor functionality
async fn demo_basic_async_processing() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Demo 1: Basic Async/Parallel Processing");
    println!("------------------------------------------");

    // Create parallel configuration
    let config = ParallelConfig {
        max_concurrent_forecasts: 8,
        parallel_chunk_size: 500,
        forecast_timeout: Duration::from_secs(10),
        enable_parallel_ensemble: true,
        worker_threads: num_cpus::get(),
    };

    let processor = AsyncParallelProcessor::new(config);
    
    // Create test data
    let mut time_series = CacheOptimizedTimeSeries::new();
    for i in 0..1000 {
        let price = 100.0 + (i as f64 * 0.1) + (i as f64 * 0.05).sin() * 10.0;
        time_series.add_price(price as f32);
    }
    let data = Arc::new(time_series);

    // Create multiple forecast tasks
    let tasks: Vec<ForecastTask> = (0..10)
        .map(|i| ForecastTask {
            id: format!("task_{}", i),
            symbol: format!("STOCK_{}", i),
            data: data.clone(),
            priority: (i % 3) as u8, // Vary priorities
            created_at: Instant::now(),
        })
        .collect();

    println!("Created {} forecast tasks", tasks.len());

    // Process tasks concurrently
    let start = Instant::now();
    let results = processor.process_forecasts_concurrent(tasks).await;
    let duration = start.elapsed();

    println!("✅ Processed {} forecasts in {:?}", results.len(), duration);
    
    // Display results
    for result in &results[..3] { // Show first 3 results
        println!("  📈 {}: Forecast ${:.2} (confidence: {:.1}%) - Worker {}", 
                result.symbol, 
                result.result.forecast_price, 
                result.result.confidence * 100.0,
                result.worker_id);
    }
    
    // Show statistics
    let stats = processor.get_stats();
    println!("  📊 Processor Stats: {}/{} permits, {} tasks total, {} workers",
             stats.available_permits, stats.max_concurrent, 
             stats.total_tasks_processed, stats.worker_threads);
    
    println!();
    Ok(())
}

/// Demo 2: Parallel Market Data Processing
async fn demo_parallel_market_data_processing() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Demo 2: Parallel Market Data Processing");
    println!("------------------------------------------");

    let config = ParallelConfig::default();
    let processor = AsyncParallelProcessor::new(config);

    // Generate large dataset of market data
    let market_data: Vec<MarketData> = (0..5000)
        .map(|i| {
            let base_price = 100.0 + (i as f64 * 0.1);
            MarketData {
                symbol: format!("STOCK_{}", i % 100), // 100 different stocks
                open: base_price,
                high: base_price * 1.05,
                low: base_price * 0.95,
                close: base_price + (i as f64 * 0.01).sin() * 2.0,
                volume: 1000.0 + (i as f64 * 10.0),
                timestamp: SystemTime::now(),
            }
        })
        .collect();

    println!("Generated {} market data points", market_data.len());

    // Process in parallel
    let start = Instant::now();
    let processed = processor.process_market_data_parallel(&market_data);
    let duration = start.elapsed();

    println!("✅ Processed {} market data points in {:?}", processed.len(), duration);
    println!("  ⚡ Processing rate: {:.0} items/sec", 
             processed.len() as f64 / duration.as_secs_f64());

    // Show sample results
    for item in &processed[..3] {
        println!("  📊 {}: ${:.2} (vol: {:.2}%, momentum: {:.2}%)",
                item.symbol, item.price, 
                item.volatility * 100.0, item.momentum * 100.0);
    }

    println!();
    Ok(())
}

/// Demo 3: Ensemble Forecasting with Parallel Processing
async fn demo_ensemble_parallel_forecasting() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Demo 3: Ensemble Parallel Forecasting");
    println!("----------------------------------------");

    let config = ParallelConfig {
        max_concurrent_forecasts: 12,
        enable_parallel_ensemble: true,
        ..Default::default()
    };
    let processor = AsyncParallelProcessor::new(config);

    // Create complex time series with trend and seasonality
    let mut time_series = CacheOptimizedTimeSeries::new();
    for i in 0..500 {
        let trend = i as f64 * 0.2;
        let seasonal = (i as f64 * 2.0 * std::f64::consts::PI / 50.0).sin() * 5.0;
        let noise = (i as f64 * 0.7).sin() * 2.0;
        let price = 100.0 + trend + seasonal + noise;
        time_series.add_price(price as f32);
    }
    let data = Arc::new(time_series);

    println!("Created complex time series with {} data points", data.len());

    // Process ensemble forecasts
    let ensemble_sizes = vec![3, 5, 7, 10];
    
    for &ensemble_size in &ensemble_sizes {
        let start = Instant::now();
        let results = processor.process_ensemble_parallel(
            data.clone(),
            ensemble_size,
            format!("ENSEMBLE_{}", ensemble_size)
        ).await;
        let duration = start.elapsed();

        println!("✅ Ensemble size {}: {} forecasts in {:?}", 
                ensemble_size, results.len(), duration);
        
        if let Some(first_result) = results.first() {
            println!("  📈 Average forecast: ${:.2} (confidence: {:.1}%)",
                    first_result.forecast_price, first_result.confidence * 100.0);
        }
    }

    println!();
    Ok(())
}

/// Demo 4: Real-time Data Pipeline
async fn demo_async_data_pipeline() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Demo 4: Async Data Pipeline");
    println!("------------------------------");

    let config = ParallelConfig {
        max_concurrent_forecasts: 6,
        parallel_chunk_size: 100,
        ..Default::default()
    };
    let processor = AsyncParallelProcessor::new(config);
    
    // Create data pipeline with 100ms processing interval
    let pipeline = AsyncDataPipeline::new(processor, Duration::from_millis(100));
    
    println!("Starting real-time data pipeline...");
    
    // Start the pipeline
    let pipeline_handle = pipeline.start_pipeline().await;
    
    // Simulate real-time data ingestion
    for i in 0..20 {
        let data = MarketData {
            symbol: format!("REALTIME_{}", i % 5),
            open: 100.0 + i as f64,
            high: 105.0 + i as f64,
            low: 95.0 + i as f64,
            close: 102.0 + i as f64 + (i as f64 * 0.1).sin(),
            volume: 1000.0 + i as f64 * 50.0,
            timestamp: SystemTime::now(),
        };
        
        pipeline.add_market_data(data).await;
        
        if i % 5 == 0 {
            println!("  📡 Ingested {} data points...", i + 1);
        }
        
        // Simulate real-time delay
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    
    // Let pipeline process remaining data
    tokio::time::sleep(Duration::from_millis(300)).await;
    
    println!("✅ Real-time data pipeline demonstration completed");
    
    // Note: In a real application, you would handle pipeline_handle properly
    pipeline_handle.abort();
    
    println!();
    Ok(())
}

/// Demo 5: ARIMA Strategy with Async Processing
async fn demo_arima_async_signals() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Demo 5: ARIMA Strategy Async Signal Generation");
    println!("------------------------------------------------");

    // Create ARIMA strategy with async processing enabled
    let config = ArimaStrategyConfig {
        p: 2,
        d: 1,
        q: 2,
        min_data_points: 50,
        enable_parallel_processing: true,
        max_concurrent_forecasts: 8,
        parallel_ensemble: true,
        forecast_timeout_secs: 15,
        ..Default::default()
    };

    let mut strategy = ArimaStrategy::new(config);
    
    // Create test DataFrame
    let timestamps: Vec<String> = (0..200)
        .map(|i| format!("2024-01-{:02} 09:{:02}:00", (i / 60) + 1, i % 60))
        .collect();
    
    let prices: Vec<f64> = (0..200)
        .map(|i| {
            let trend = i as f64 * 0.5;
            let seasonal = (i as f64 * 2.0 * std::f64::consts::PI / 20.0).sin() * 8.0;
            let noise = (i as f64 * 1.3).sin() * 3.0;
            100.0 + trend + seasonal + noise
        })
        .collect();

    let df = df! {
        "timestamp" => timestamps,
        "close" => prices,
    }?;

    println!("Created DataFrame with {} rows", df.height());
    println!("Parallel processing enabled: {}", strategy.is_parallel_enabled());

    // Generate signals using async processing
    let start = Instant::now();
    let signals = strategy.generate_signals_async(&df, "close", "timestamp").await?;
    let async_duration = start.elapsed();

    println!("✅ Generated {} signals using async processing in {:?}", 
             signals.len(), async_duration);

    // Count signal types
    let buy_count = signals.iter().filter(|&&s| s == Signal::Buy).count();
    let sell_count = signals.iter().filter(|&&s| s == Signal::Sell).count();
    let hold_count = signals.iter().filter(|&&s| s == Signal::Hold).count();

    println!("  📊 Signal Distribution:");
    println!("    🟢 Buy: {} ({:.1}%)", buy_count, buy_count as f64 / signals.len() as f64 * 100.0);
    println!("    🔴 Sell: {} ({:.1}%)", sell_count, sell_count as f64 / signals.len() as f64 * 100.0);
    println!("    ⚪ Hold: {} ({:.1}%)", hold_count, hold_count as f64 / signals.len() as f64 * 100.0);

    // Show async processor stats
    if let Some(stats) = strategy.get_async_stats() {
        println!("  ⚙️  {}", stats);
    }

    // Test ensemble async processing
    println!("\n  🎯 Testing Ensemble Async Processing...");
    let start = Instant::now();
    let ensemble_signals = strategy.generate_ensemble_signals_async(
        &df, "close", "timestamp", 5
    ).await?;
    let ensemble_duration = start.elapsed();

    println!("  ✅ Generated {} ensemble signals in {:?}", 
             ensemble_signals.len(), ensemble_duration);

    println!();
    Ok(())
}

/// Demo 6: Performance Comparison (Sync vs Async)
async fn demo_performance_comparison() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Demo 6: Performance Comparison (Sync vs Async)");
    println!("------------------------------------------------");

    // Create identical configurations for comparison
    let sync_config = ArimaStrategyConfig {
        p: 2,
        d: 1,
        q: 2,
        min_data_points: 40,
        enable_parallel_processing: false, // Synchronous
        ..Default::default()
    };

    let async_config = ArimaStrategyConfig {
        p: 2,
        d: 1,
        q: 2,
        min_data_points: 40,
        enable_parallel_processing: true, // Asynchronous
        max_concurrent_forecasts: 8,
        parallel_ensemble: true,
        ..Default::default()
    };

    let mut sync_strategy = ArimaStrategy::new(sync_config);
    let mut async_strategy = ArimaStrategy::new(async_config);

    // Create larger test dataset
    let size = 500;
    let timestamps: Vec<String> = (0..size)
        .map(|i| format!("2024-01-{:02} {:02}:{:02}:00", 
                        (i / 1440) + 1, (i / 60) % 24, i % 60))
        .collect();
    
    let prices: Vec<f64> = (0..size)
        .map(|i| {
            let trend = i as f64 * 0.3;
            let seasonal = (i as f64 * 2.0 * std::f64::consts::PI / 50.0).sin() * 12.0;
            let noise = (i as f64 * 2.1).sin() * 4.0;
            150.0 + trend + seasonal + noise
        })
        .collect();

    let df = df! {
        "timestamp" => timestamps,
        "close" => prices,
    }?;

    println!("Performance test dataset: {} rows", df.height());

    // Test synchronous processing
    println!("\n⏱️  Testing Synchronous Processing...");
    let start = Instant::now();
    let sync_signals = sync_strategy.generate_signals(&df, "close", "timestamp")?;
    let sync_duration = start.elapsed();
    
    println!("  ✅ Sync: {} signals in {:?}", sync_signals.len(), sync_duration);

    // Test asynchronous processing
    println!("\n⏱️  Testing Asynchronous Processing...");
    let start = Instant::now();
    let async_signals = async_strategy.generate_signals_async(&df, "close", "timestamp").await?;
    let async_duration = start.elapsed();
    
    println!("  ✅ Async: {} signals in {:?}", async_signals.len(), async_duration);

    // Performance comparison
    let speedup = sync_duration.as_millis() as f64 / async_duration.as_millis() as f64;
    println!("\n📈 Performance Results:");
    println!("  🐌 Synchronous:  {:?}", sync_duration);
    println!("  🚀 Asynchronous: {:?}", async_duration);
    
    if speedup > 1.0 {
        println!("  ⚡ Speedup: {:.2}x faster with async processing!", speedup);
    } else {
        println!("  📊 Performance ratio: {:.2}x", speedup);
        println!("     (Note: Speedup may vary based on dataset size and complexity)");
    }

    // Memory usage comparison
    let sync_memory = sync_strategy.get_memory_stats();
    let async_memory = async_strategy.get_memory_stats();
    
    println!("\n💾 Memory Usage:");
    println!("  Sync:  {}", sync_memory);
    println!("  Async: {}", async_memory);
    
    if let Some(async_stats) = async_strategy.get_async_stats() {
        println!("  Async Processor: {}", async_stats);
    }

    println!();
    Ok(())
}

/// Helper function to create realistic market data
fn create_realistic_market_data(symbol: &str, days: usize) -> Vec<MarketData> {
    let mut data = Vec::new();
    let mut base_price = 100.0;
    
    for i in 0..days {
        // Simulate random walk with slight upward bias
        let change = (i as f64 * 0.1).sin() * 2.0 + 0.05;
        base_price += change;
        
        let volatility = 0.02; // 2% daily volatility
        let high = base_price * (1.0 + volatility);
        let low = base_price * (1.0 - volatility);
        let open = base_price + (i as f64 * 0.2).sin() * 1.0;
        
        data.push(MarketData {
            symbol: symbol.to_string(),
            open,
            high,
            low,
            close: base_price,
            volume: 10000.0 + (i as f64 * 100.0),
            timestamp: SystemTime::now(),
        });
    }
    
    data
}

/// Demonstrate resource utilization monitoring
async fn monitor_resource_utilization(processor: &AsyncParallelProcessor, duration: Duration) {
    let start = Instant::now();
    
    while start.elapsed() < duration {
        let stats = processor.get_stats();
        println!("  📊 Utilization: {}/{} permits active, {} total processed",
                stats.max_concurrent - stats.available_permits,
                stats.max_concurrent,
                stats.total_tasks_processed);
        
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
} 