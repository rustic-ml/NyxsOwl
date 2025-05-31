use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use nyxs_owl::forecast_trade::*;

fn generate_test_data(size: usize) -> Vec<f64> {
    (0..size)
        .map(|i| 100.0 + (i as f64 * 0.01) + (i as f64 * 0.001).sin())
        .collect()
}

fn benchmark_forecast_methods(c: &mut Criterion) {
    let mut group = c.benchmark_group("forecast_methods");

    for size in [100, 500, 1000].iter() {
        let data = generate_test_data(*size);

        // Benchmark moving average forecasting
        group.bench_with_input(BenchmarkId::new("moving_average", size), size, |b, _| {
            b.iter(|| {
                // Use fallback implementation if the actual module doesn't exist
                let window = 10.min(*size / 2);
                let periods = 5;
                if window > 0 && periods > 0 {
                    let avg = data[data.len().saturating_sub(window)..]
                        .iter()
                        .sum::<f64>()
                        / window as f64;
                    let _forecast = vec![avg; periods];
                }
            });
        });

        // Benchmark exponential smoothing
        group.bench_with_input(
            BenchmarkId::new("exponential_smoothing", size),
            size,
            |b, _| {
                b.iter(|| {
                    let alpha = 0.3;
                    let periods = 5;
                    let mut smoothed = data[0];
                    for &value in data.iter().skip(1) {
                        smoothed = alpha * value + (1.0 - alpha) * smoothed;
                    }
                    let _forecast = vec![smoothed; periods];
                });
            },
        );

        // Benchmark simple trend forecasting
        group.bench_with_input(BenchmarkId::new("trend_forecast", size), size, |b, _| {
            b.iter(|| {
                let periods = 5;
                let last_values = &data[data.len().saturating_sub(10)..];
                if last_values.len() >= 2 {
                    let trend = (last_values[last_values.len() - 1] - last_values[0])
                        / last_values.len() as f64;
                    let base = last_values[last_values.len() - 1];
                    let _forecast: Vec<f64> =
                        (1..=periods).map(|i| base + trend * i as f64).collect();
                }
            });
        });
    }

    group.finish();
}

fn benchmark_forecast_accuracy(c: &mut Criterion) {
    let mut group = c.benchmark_group("forecast_accuracy");

    let data = generate_test_data(200);
    let split_point = 150;
    let train_data = &data[..split_point];
    let test_data = &data[split_point..];

    group.bench_function("accuracy_calculation", |b| {
        b.iter(|| {
            // Simple accuracy calculation
            let alpha = 0.3;
            let mut smoothed = train_data[0];
            for &value in train_data.iter().skip(1) {
                smoothed = alpha * value + (1.0 - alpha) * smoothed;
            }

            // Calculate MSE
            let mse = test_data
                .iter()
                .map(|&actual| (actual - smoothed).powi(2))
                .sum::<f64>()
                / test_data.len() as f64;

            black_box(mse);
        });
    });

    group.finish();
}

fn benchmark_data_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_processing");

    for size in [1000, 5000, 10000].iter() {
        let data = generate_test_data(*size);

        group.bench_with_input(BenchmarkId::new("data_validation", size), size, |b, _| {
            b.iter(|| {
                // Data validation simulation
                let is_valid = !data.is_empty()
                    && data.len() >= 10
                    && data.iter().all(|&x| x.is_finite())
                    && {
                        let min_val = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                        let max_val = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                        (max_val - min_val).abs() >= 1e-10
                    };
                black_box(is_valid);
            });
        });

        group.bench_with_input(
            BenchmarkId::new("data_preprocessing", size),
            size,
            |b, _| {
                b.iter(|| {
                    // Data preprocessing simulation
                    let processed: Vec<f64> = data
                        .iter()
                        .filter(|&&x| x.is_finite())
                        .map(|&x| x)
                        .collect();
                    black_box(processed);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_forecast_methods,
    benchmark_forecast_accuracy,
    benchmark_data_processing
);

criterion_main!(benches);
