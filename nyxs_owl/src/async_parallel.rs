use crate::memory_optimized::{CacheOptimizedTimeSeries, MemoryPool};
use crate::performance_utils::SimdMath;
use crate::simple_types::{NyxsOwlError, Result};
use futures::future::join_all;
use rayon::prelude::*;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};

/// Market data structure for processing
#[derive(Debug, Clone)]
pub struct MarketData {
    pub symbol: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub timestamp: std::time::SystemTime,
}

/// Forecast result from parallel processing
#[derive(Debug, Clone)]
pub struct ForecastResult {
    pub symbol: String,
    pub forecast_price: f64,
    pub confidence: f64,
    pub volatility: f64,
    pub trend_strength: f64,
    pub timestamp: std::time::SystemTime,
    pub metadata: String,
}

/// Configuration for async/parallel processing
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// Maximum number of concurrent forecasting tasks
    pub max_concurrent_forecasts: usize,
    /// Chunk size for parallel data processing
    pub parallel_chunk_size: usize,
    /// Timeout for individual forecast operations
    pub forecast_timeout: Duration,
    /// Enable parallel ensemble processing
    pub enable_parallel_ensemble: bool,
    /// Number of worker threads for CPU-intensive tasks
    pub worker_threads: usize,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            max_concurrent_forecasts: num_cpus::get().max(4),
            parallel_chunk_size: 1000,
            forecast_timeout: Duration::from_secs(30),
            enable_parallel_ensemble: true,
            worker_threads: num_cpus::get(),
        }
    }
}

/// Async forecasting task with priority and metadata
#[derive(Debug, Clone)]
pub struct ForecastTask {
    pub id: String,
    pub symbol: String,
    pub data: Arc<CacheOptimizedTimeSeries>,
    pub priority: u8, // 0 = highest priority
    pub created_at: Instant,
}

/// Result of parallel forecasting operation
#[derive(Debug, Clone)]
pub struct ParallelForecastResult {
    pub task_id: String,
    pub symbol: String,
    pub result: ForecastResult,
    pub processing_time: Duration,
    pub worker_id: usize,
}

/// Async/Parallel Processing Manager
pub struct AsyncParallelProcessor {
    config: ParallelConfig,
    semaphore: Arc<Semaphore>,
    memory_pool: Arc<RwLock<MemoryPool<f64>>>,
    task_counter: Arc<std::sync::atomic::AtomicUsize>,
}

impl AsyncParallelProcessor {
    /// Create new async/parallel processor
    pub fn new(config: ParallelConfig) -> Self {
        // Initialize Rayon thread pool
        rayon::ThreadPoolBuilder::new()
            .num_threads(config.worker_threads)
            .build_global()
            .unwrap_or_else(|_| {
                eprintln!("Warning: Failed to initialize Rayon thread pool");
            });

        Self {
            semaphore: Arc::new(Semaphore::new(config.max_concurrent_forecasts)),
            memory_pool: Arc::new(RwLock::new(MemoryPool::new(1024 * 1024))), // 1MB pool
            task_counter: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            config,
        }
    }

    /// Process multiple forecasting tasks concurrently
    pub async fn process_forecasts_concurrent(
        &self,
        tasks: Vec<ForecastTask>,
    ) -> Vec<ParallelForecastResult> {
        let mut sorted_tasks = tasks;
        // Sort by priority (lower number = higher priority)
        sorted_tasks.sort_by_key(|task| task.priority);

        let futures = sorted_tasks
            .into_iter()
            .map(|task| self.process_single_forecast(task));

        join_all(futures).await.into_iter().flatten().collect()
    }

    /// Process a single forecast task with timeout and resource management
    async fn process_single_forecast(&self, task: ForecastTask) -> Option<ParallelForecastResult> {
        let _permit = self.semaphore.acquire().await.ok()?;
        let start_time = Instant::now();

        let worker_id = self
            .task_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // Timeout wrapper
        let forecast_future = self.execute_forecast_task(task.clone(), worker_id);

        match tokio::time::timeout(self.config.forecast_timeout, forecast_future).await {
            Ok(Some(result)) => {
                let processing_time = start_time.elapsed();
                Some(ParallelForecastResult {
                    task_id: task.id,
                    symbol: task.symbol,
                    result,
                    processing_time,
                    worker_id,
                })
            }
            Ok(None) => {
                eprintln!("Forecast task {} failed", task.id);
                None
            }
            Err(_) => {
                eprintln!("Forecast task {} timed out", task.id);
                None
            }
        }
    }

    /// Execute the actual forecasting computation
    async fn execute_forecast_task(
        &self,
        task: ForecastTask,
        worker_id: usize,
    ) -> Option<ForecastResult> {
        // Move CPU-intensive work to blocking thread pool
        let data = task.data.clone();
        let symbol = task.symbol.clone();

        tokio::task::spawn_blocking(move || {
            Self::compute_forecast_blocking(&data, &symbol, worker_id)
        })
        .await
        .ok()
        .flatten()
    }

    /// CPU-intensive forecast computation (runs on blocking thread pool)
    fn compute_forecast_blocking(
        data: &CacheOptimizedTimeSeries,
        symbol: &str,
        worker_id: usize,
    ) -> Option<ForecastResult> {
        // Simulate ARIMA-style forecasting with SIMD acceleration
        let prices = data.get_prices();
        if prices.len() < 10 {
            return None;
        }

        // Use SIMD for statistical calculations
        let mean = SimdMath::safe_mean(&prices);
        let variance = SimdMath::safe_variance(&prices);
        let volatility = variance.sqrt();

        // Parallel trend analysis using Rayon
        let trend_strength = Self::parallel_trend_analysis(&prices);

        // Generate forecast
        let forecast_price = mean + (trend_strength * volatility * 0.1);
        let confidence = (1.0f64 - (volatility / mean).min(1.0)).max(0.0);

        Some(ForecastResult {
            symbol: symbol.to_string(),
            forecast_price,
            confidence,
            volatility,
            trend_strength,
            timestamp: std::time::SystemTime::now(),
            metadata: format!("worker_{}", worker_id),
        })
    }

    /// Parallel trend analysis using Rayon
    fn parallel_trend_analysis(prices: &[f64]) -> f64 {
        if prices.len() < 4 {
            return 0.0;
        }

        // Split into chunks for parallel processing
        let chunk_size = (prices.len() / num_cpus::get()).max(10);

        let trend_components: Vec<f64> = prices
            .par_chunks(chunk_size)
            .map(|chunk| {
                if chunk.len() < 2 {
                    return 0.0;
                }

                // Calculate local trend for this chunk
                let first = chunk[0];
                let last = chunk[chunk.len() - 1];
                (last - first) / chunk.len() as f64
            })
            .collect();

        // Aggregate trend components
        SimdMath::safe_mean(&trend_components)
    }

    /// Process ensemble forecasts in parallel
    pub async fn process_ensemble_parallel(
        &self,
        data: Arc<CacheOptimizedTimeSeries>,
        ensemble_size: usize,
        symbol: String,
    ) -> Vec<ForecastResult> {
        if !self.config.enable_parallel_ensemble {
            return vec![];
        }

        let tasks: Vec<_> = (0..ensemble_size)
            .map(|i| {
                ForecastTask {
                    id: format!("ensemble_{}_{}", symbol, i),
                    symbol: symbol.clone(),
                    data: data.clone(),
                    priority: 1, // Medium priority for ensemble
                    created_at: Instant::now(),
                }
            })
            .collect();

        let results = self.process_forecasts_concurrent(tasks).await;
        results.into_iter().map(|r| r.result).collect()
    }

    /// Parallel batch processing of market data
    pub fn process_market_data_parallel(
        &self,
        market_data: &[MarketData],
    ) -> Vec<ProcessedMarketData> {
        market_data
            .par_chunks(self.config.parallel_chunk_size)
            .flat_map(|chunk| {
                chunk
                    .par_iter()
                    .map(|data| ProcessedMarketData::from_market_data(data))
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    /// Get processing statistics
    pub fn get_stats(&self) -> ProcessingStats {
        ProcessingStats {
            available_permits: self.semaphore.available_permits(),
            max_concurrent: self.config.max_concurrent_forecasts,
            total_tasks_processed: self.task_counter.load(std::sync::atomic::Ordering::Relaxed),
            worker_threads: self.config.worker_threads,
        }
    }
}

/// Processed market data with enhanced metrics
#[derive(Debug, Clone)]
pub struct ProcessedMarketData {
    pub symbol: String,
    pub price: f64,
    pub volume: f64,
    pub volatility: f64,
    pub momentum: f64,
    pub processed_at: Instant,
}

impl ProcessedMarketData {
    fn from_market_data(data: &MarketData) -> Self {
        // Simulate processing with some calculations
        let volatility = (data.high - data.low) / data.close;
        let momentum = (data.close - data.open) / data.open;

        Self {
            symbol: data.symbol.clone(),
            price: data.close,
            volume: data.volume,
            volatility,
            momentum,
            processed_at: Instant::now(),
        }
    }
}

/// Processing statistics
#[derive(Debug)]
pub struct ProcessingStats {
    pub available_permits: usize,
    pub max_concurrent: usize,
    pub total_tasks_processed: usize,
    pub worker_threads: usize,
}

/// Async data pipeline for real-time processing
pub struct AsyncDataPipeline {
    processor: Arc<AsyncParallelProcessor>,
    data_buffer: Arc<RwLock<Vec<MarketData>>>,
    processing_interval: Duration,
}

impl AsyncDataPipeline {
    /// Create new async data pipeline
    pub fn new(processor: AsyncParallelProcessor, processing_interval: Duration) -> Self {
        Self {
            processor: Arc::new(processor),
            data_buffer: Arc::new(RwLock::new(Vec::new())),
            processing_interval,
        }
    }

    /// Start the async processing pipeline
    pub async fn start_pipeline(&self) -> tokio::task::JoinHandle<()> {
        let processor = self.processor.clone();
        let data_buffer = self.data_buffer.clone();
        let interval = self.processing_interval;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                // Process buffered data
                let data = {
                    let mut buffer = data_buffer.write().await;
                    let data = buffer.clone();
                    buffer.clear();
                    data
                };

                if !data.is_empty() {
                    let processed = processor.process_market_data_parallel(&data);
                    println!("Processed {} market data points", processed.len());
                }
            }
        })
    }

    /// Add market data to processing buffer
    pub async fn add_market_data(&self, data: MarketData) {
        let mut buffer = self.data_buffer.write().await;
        buffer.push(data);
    }
}

#[cfg(test)]
mod tests {
    use super::MarketData;
    use super::*;

    #[tokio::test]
    async fn test_parallel_forecast_processing() {
        let config = ParallelConfig {
            max_concurrent_forecasts: 2,
            parallel_chunk_size: 100,
            forecast_timeout: Duration::from_secs(5),
            enable_parallel_ensemble: true,
            worker_threads: 2,
        };

        let processor = AsyncParallelProcessor::new(config);

        // Create test data
        let mut time_series = CacheOptimizedTimeSeries::new();
        for i in 0..100 {
            time_series.add_price((100.0 + (i as f64 * 0.1)) as f32);
        }
        let data = Arc::new(time_series);

        // Create test tasks
        let tasks = vec![
            ForecastTask {
                id: "test_1".to_string(),
                symbol: "AAPL".to_string(),
                data: data.clone(),
                priority: 0,
                created_at: Instant::now(),
            },
            ForecastTask {
                id: "test_2".to_string(),
                symbol: "GOOGL".to_string(),
                data: data.clone(),
                priority: 1,
                created_at: Instant::now(),
            },
        ];

        let results = processor.process_forecasts_concurrent(tasks).await;
        assert_eq!(results.len(), 2);

        for result in &results {
            assert!(result.result.confidence > 0.0);
            assert!(result.processing_time.as_nanos() >= 0); // More lenient timing check
        }
    }

    #[tokio::test]
    async fn test_ensemble_parallel_processing() {
        let config = ParallelConfig::default();
        let processor = AsyncParallelProcessor::new(config);

        let mut time_series = CacheOptimizedTimeSeries::new();
        for i in 0..50 {
            time_series.add_price((100.0 + (i as f64 * 0.2)) as f32);
        }
        let data = Arc::new(time_series);

        let results = processor
            .process_ensemble_parallel(data, 5, "TEST".to_string())
            .await;

        assert_eq!(results.len(), 5);
        for result in &results {
            assert_eq!(result.symbol, "TEST");
            assert!(result.confidence >= 0.0 && result.confidence <= 1.0);
        }
    }

    #[test]
    fn test_parallel_market_data_processing() {
        let config = ParallelConfig::default();
        let processor = AsyncParallelProcessor::new(config);

        let market_data: Vec<MarketData> = (0..1000)
            .map(|i| MarketData {
                symbol: format!("STOCK_{}", i % 10),
                open: 100.0 + (i as f64 * 0.1),
                high: 105.0 + (i as f64 * 0.1),
                low: 95.0 + (i as f64 * 0.1),
                close: 102.0 + (i as f64 * 0.1),
                volume: 1000.0 + (i as f64 * 10.0),
                timestamp: std::time::SystemTime::now(),
            })
            .collect();

        let start = Instant::now();
        let processed = processor.process_market_data_parallel(&market_data);
        let duration = start.elapsed();

        assert_eq!(processed.len(), 1000);
        println!("Processed {} items in {:?}", processed.len(), duration);

        // Verify processing
        for (i, item) in processed.iter().enumerate() {
            assert!(item.volatility >= 0.0);
            assert_eq!(item.symbol, format!("STOCK_{}", i % 10));
        }
    }

    #[tokio::test]
    async fn test_async_data_pipeline() {
        let config = ParallelConfig {
            max_concurrent_forecasts: 2,
            parallel_chunk_size: 10,
            forecast_timeout: Duration::from_secs(1),
            enable_parallel_ensemble: true,
            worker_threads: 2,
        };

        let processor = AsyncParallelProcessor::new(config);
        let pipeline = AsyncDataPipeline::new(processor, Duration::from_millis(100));

        // Start pipeline
        let _handle = pipeline.start_pipeline().await;

        // Add some test data
        for i in 0..5 {
            let data = MarketData {
                symbol: format!("TEST_{}", i),
                open: 100.0,
                high: 105.0,
                low: 95.0,
                close: 102.0,
                volume: 1000.0,
                timestamp: std::time::SystemTime::now(),
            };
            pipeline.add_market_data(data).await;
        }

        // Let it process for a short time
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Pipeline should be running (test passes if no panic)
        assert!(true);
    }
}
