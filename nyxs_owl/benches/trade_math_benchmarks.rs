use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use nyxs_owl::trade_math::{
    forecasting::{DoubleExponentialSmoothing, ExponentialSmoothing, LinearRegression},
    moving_averages::{ExponentialMovingAverage, SimpleMovingAverage, VolumeWeightedMovingAverage},
    oscillators::{Macd, RelativeStrengthIndex, StochasticOscillator},
    volatility::{AverageTrueRange, BollingerBands, StandardDeviation},
    volume::{OnBalanceVolume, VolumePriceTrend, VolumeRateOfChange},
};

/// Generate realistic market data for benchmarking
fn generate_market_data(size: usize) -> (Vec<f64>, Vec<f64>) {
    let mut prices = Vec::with_capacity(size);
    let mut volumes = Vec::with_capacity(size);

    let mut price = 100.0;
    let mut rng_seed = 12345u64;

    for i in 0..size {
        // Simple pseudo-random number generator
        rng_seed = rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
        let random = (rng_seed >> 16) as f64 / 65536.0;

        // Generate price with some trend and noise
        let trend = (i as f64 * 0.001).sin() * 0.5;
        let noise = (random - 0.5) * 2.0;
        price = price * (1.0 + trend + noise * 0.02);
        prices.push(price);

        // Generate volume correlated with price movement
        let volume = 1000.0 + (noise.abs() * 5000.0) + (i as f64 * 0.1);
        volumes.push(volume);
    }

    (prices, volumes)
}

fn benchmark_moving_averages(c: &mut Criterion) {
    let mut group = c.benchmark_group("moving_averages");

    // Reduced dataset sizes for IDE-friendly benchmarking
    for &size in [100, 1_000, 5_000].iter() {
        // Reduced from 100_000 max
        group.throughput(Throughput::Elements(size as u64));
        let (prices, volumes) = generate_market_data(size);

        // Simple Moving Average
        group.bench_with_input(BenchmarkId::new("SMA_20", size), &prices, |b, prices| {
            b.iter(|| {
                let mut sma = SimpleMovingAverage::new(20).unwrap();
                for &price in prices {
                    sma.update(price).unwrap();
                }
                sma.value()
            })
        });

        // Exponential Moving Average
        group.bench_with_input(BenchmarkId::new("EMA_20", size), &prices, |b, prices| {
            b.iter(|| {
                let mut ema = ExponentialMovingAverage::new(20).unwrap();
                for &price in prices {
                    ema.update(price).unwrap();
                }
                ema.value()
            })
        });

        // Volume Weighted Moving Average
        group.bench_with_input(
            BenchmarkId::new("VWMA_20", size),
            &(&prices, &volumes),
            |b, (prices, volumes)| {
                b.iter(|| {
                    let mut vwma = VolumeWeightedMovingAverage::new(20).unwrap();
                    for (&price, &volume) in prices.iter().zip(volumes.iter()) {
                        vwma.update(price, volume).unwrap();
                    }
                    vwma.value()
                })
            },
        );
    }

    group.finish();
}

fn benchmark_volatility_indicators(c: &mut Criterion) {
    let mut group = c.benchmark_group("volatility_indicators");

    // Reduced dataset sizes for IDE-friendly benchmarking
    for &size in [100, 1_000, 5_000].iter() {
        // Reduced from 100_000 max
        group.throughput(Throughput::Elements(size as u64));
        let (prices, _) = generate_market_data(size);

        // Bollinger Bands
        group.bench_with_input(
            BenchmarkId::new("BollingerBands_20", size),
            &prices,
            |b, prices| {
                b.iter(|| {
                    let mut bb = BollingerBands::new(20, 2.0).unwrap();
                    for &price in prices {
                        bb.update(price).unwrap();
                    }
                    (bb.upper_band(), bb.middle_band(), bb.lower_band())
                })
            },
        );

        // Average True Range
        group.bench_with_input(BenchmarkId::new("ATR_14", size), &prices, |b, prices| {
            b.iter(|| {
                let mut atr = AverageTrueRange::new(14).unwrap();
                for &price in prices {
                    // Simulate high/low around close price
                    let high = price * 1.005;
                    let low = price * 0.995;
                    atr.update(high, low, price).unwrap();
                }
                atr.value()
            })
        });

        // Standard Deviation
        group.bench_with_input(BenchmarkId::new("StdDev_20", size), &prices, |b, prices| {
            b.iter(|| {
                let mut std_dev = StandardDeviation::new(20).unwrap();
                for &price in prices {
                    std_dev.update(price).unwrap();
                }
                std_dev.value()
            })
        });
    }

    group.finish();
}

fn benchmark_oscillators(c: &mut Criterion) {
    let mut group = c.benchmark_group("oscillators");

    // Reduced dataset sizes for IDE-friendly benchmarking
    for &size in [100, 1_000, 5_000].iter() {
        // Reduced from 100_000 max
        group.throughput(Throughput::Elements(size as u64));
        let (prices, _) = generate_market_data(size);

        // RSI
        group.bench_with_input(BenchmarkId::new("RSI_14", size), &prices, |b, prices| {
            b.iter(|| {
                let mut rsi = RelativeStrengthIndex::new(14).unwrap();
                for &price in prices {
                    rsi.update(price).unwrap();
                }
                rsi.value()
            })
        });

        // MACD
        group.bench_with_input(
            BenchmarkId::new("MACD_12_26_9", size),
            &prices,
            |b, prices| {
                b.iter(|| {
                    let mut macd = Macd::new(12, 26, 9).unwrap();
                    for &price in prices {
                        macd.update(price).unwrap();
                    }
                    (macd.macd_value(), macd.signal_value(), macd.histogram())
                })
            },
        );

        // Stochastic Oscillator
        group.bench_with_input(
            BenchmarkId::new("Stochastic_14_3", size),
            &prices,
            |b, prices| {
                b.iter(|| {
                    let mut stoch = StochasticOscillator::new(14, 3).unwrap();
                    for &price in prices {
                        let high = price * 1.005;
                        let low = price * 0.995;
                        stoch.update(high, low, price).unwrap();
                    }
                    (stoch.k_value(), stoch.d_value())
                })
            },
        );
    }

    group.finish();
}

fn benchmark_volume_indicators(c: &mut Criterion) {
    let mut group = c.benchmark_group("volume_indicators");

    for &size in [100, 1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(size as u64));
        let (prices, volumes) = generate_market_data(size);

        // On-Balance Volume
        group.bench_with_input(
            BenchmarkId::new("OBV", size),
            &(&prices, &volumes),
            |b, (prices, volumes)| {
                b.iter(|| {
                    let mut obv = OnBalanceVolume::new();
                    for (&price, &volume) in prices.iter().zip(volumes.iter()) {
                        obv.update(price, volume).unwrap();
                    }
                    obv.value()
                })
            },
        );

        // Volume Price Trend
        group.bench_with_input(
            BenchmarkId::new("VPT", size),
            &(&prices, &volumes),
            |b, (prices, volumes)| {
                b.iter(|| {
                    let mut vpt = VolumePriceTrend::new();
                    for (&price, &volume) in prices.iter().zip(volumes.iter()) {
                        vpt.update(price, volume).unwrap();
                    }
                    vpt.value()
                })
            },
        );

        // Volume Rate of Change
        group.bench_with_input(BenchmarkId::new("VROC_10", size), &volumes, |b, volumes| {
            b.iter(|| {
                let mut vroc = VolumeRateOfChange::new(10).unwrap();
                for &volume in volumes {
                    vroc.update(volume).unwrap();
                }
                vroc.value()
            })
        });
    }

    group.finish();
}

fn benchmark_forecasting(c: &mut Criterion) {
    let mut group = c.benchmark_group("forecasting");

    for &size in [100, 1_000, 10_000].iter() {
        // Smaller sizes for forecasting as they're more complex
        group.throughput(Throughput::Elements(size as u64));
        let (prices, _) = generate_market_data(size);

        // Linear Regression
        group.bench_with_input(
            BenchmarkId::new("LinearRegression_20", size),
            &prices,
            |b, prices| {
                b.iter(|| {
                    let mut lr = LinearRegression::new(20).unwrap();
                    for &price in prices {
                        lr.update(price).unwrap();
                    }
                    lr.forecast(1)
                })
            },
        );

        // Exponential Smoothing
        group.bench_with_input(
            BenchmarkId::new("ExponentialSmoothing", size),
            &prices,
            |b, prices| {
                b.iter(|| {
                    let mut es = ExponentialSmoothing::new(0.3).unwrap();
                    for &price in prices {
                        es.update(price).unwrap();
                    }
                    es.value()
                })
            },
        );

        // Double Exponential Smoothing
        group.bench_with_input(
            BenchmarkId::new("DoubleExponentialSmoothing", size),
            &prices,
            |b, prices| {
                b.iter(|| {
                    let mut des = DoubleExponentialSmoothing::new(0.3, 0.3).unwrap();
                    for &price in prices {
                        des.update(price).unwrap();
                    }
                    des.value()
                })
            },
        );
    }

    group.finish();
}

fn benchmark_multiple_indicators(c: &mut Criterion) {
    let mut group = c.benchmark_group("multiple_indicators");

    for &size in [1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(size as u64));
        let (prices, volumes) = generate_market_data(size);

        // Comprehensive Technical Analysis Suite
        group.bench_with_input(
            BenchmarkId::new("comprehensive_analysis", size),
            &(&prices, &volumes),
            |b, (prices, volumes)| {
                b.iter(|| {
                    // Initialize all indicators
                    let mut sma_20 = SimpleMovingAverage::new(20).unwrap();
                    let mut ema_12 = ExponentialMovingAverage::new(12).unwrap();
                    let mut bb_20 = BollingerBands::new(20, 2.0).unwrap();
                    let mut rsi_14 = RelativeStrengthIndex::new(14).unwrap();
                    let mut macd = Macd::new(12, 26, 9).unwrap();
                    let mut obv = OnBalanceVolume::new();
                    let mut atr = AverageTrueRange::new(14).unwrap();

                    // Process all data
                    for (&price, &volume) in prices.iter().zip(volumes.iter()) {
                        let high = price * 1.005;
                        let low = price * 0.995;

                        sma_20.update(price).unwrap();
                        ema_12.update(price).unwrap();
                        bb_20.update(price).unwrap();
                        rsi_14.update(price).unwrap();
                        macd.update(price).unwrap();
                        obv.update(price, volume).unwrap();
                        atr.update(high, low, price).unwrap();
                    }

                    // Collect all final values
                    (
                        sma_20.value(),
                        ema_12.value(),
                        bb_20.upper_band(),
                        rsi_14.value(),
                        macd.macd_value(),
                        obv.value(),
                        atr.value(),
                    )
                })
            },
        );
    }

    group.finish();
}

fn benchmark_streaming_vs_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming_vs_batch");
    let size = 10_000;
    let (prices, _) = generate_market_data(size);

    // Streaming processing (what we do normally)
    group.bench_function("streaming_sma", |b| {
        b.iter(|| {
            let mut sma = SimpleMovingAverage::new(20).unwrap();
            for &price in &prices {
                sma.update(price).unwrap();
            }
            sma.value()
        })
    });

    // Simulated batch processing (processing chunks at once)
    group.bench_function("batch_sma", |b| {
        b.iter(|| {
            let mut results = Vec::new();
            for chunk in prices.chunks(1000) {
                let mut sma = SimpleMovingAverage::new(20).unwrap();
                for &price in chunk {
                    sma.update(price).unwrap();
                }
                if let Ok(value) = sma.value() {
                    results.push(value);
                }
            }
            results
        })
    });

    group.finish();
}

fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");

    // Test memory allocation patterns
    group.bench_function("indicator_creation", |b| {
        b.iter(|| {
            // Creating indicators should be lightweight
            let _sma = SimpleMovingAverage::new(20).unwrap();
            let _ema = ExponentialMovingAverage::new(20).unwrap();
            let _bb = BollingerBands::new(20, 2.0).unwrap();
            let _rsi = RelativeStrengthIndex::new(14).unwrap();
            let _obv = OnBalanceVolume::new();
        })
    });

    group.bench_function("indicator_updates", |b| {
        let mut sma = SimpleMovingAverage::new(20).unwrap();
        let mut ema = ExponentialMovingAverage::new(20).unwrap();
        let mut bb = BollingerBands::new(20, 2.0).unwrap();
        let mut rsi = RelativeStrengthIndex::new(14).unwrap();
        let mut obv = OnBalanceVolume::new();

        b.iter(|| {
            // Single update should be very fast
            let price = 100.0;
            let volume = 1000.0;

            sma.update(price).unwrap();
            ema.update(price).unwrap();
            bb.update(price).unwrap();
            rsi.update(price).unwrap();
            obv.update(price, volume).unwrap();
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_moving_averages,
    benchmark_volatility_indicators,
    benchmark_oscillators,
    benchmark_volume_indicators,
    benchmark_forecasting,
    benchmark_multiple_indicators,
    benchmark_streaming_vs_batch,
    benchmark_memory_usage
);

criterion_main!(benches);
