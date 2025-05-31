use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use nyxs_owl::trade_math::*;

fn generate_test_data(size: usize) -> Vec<f64> {
    (0..size)
        .map(|i| 100.0 + (i as f64 * 0.01) + (i as f64 * 0.001).sin())
        .collect()
}

fn generate_returns_data(size: usize) -> Vec<f64> {
    (0..size)
        .map(|i| 0.001 * (i as f64 * 0.1).sin() + 0.0005 * (i as f64 * 0.05).cos())
        .collect()
}

fn benchmark_basic_calculations(c: &mut Criterion) {
    let mut group = c.benchmark_group("basic_calculations");

    for size in [100, 1000, 10000].iter() {
        let prices = generate_test_data(*size);
        let returns = generate_returns_data(*size);

        group.bench_with_input(BenchmarkId::new("calculate_returns", size), size, |b, _| {
            b.iter(|| calculate_returns(black_box(&prices)));
        });

        group.bench_with_input(
            BenchmarkId::new("calculate_log_returns", size),
            size,
            |b, _| {
                b.iter(|| calculate_log_returns(black_box(&prices)));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("calculate_volatility", size),
            size,
            |b, _| {
                b.iter(|| calculate_volatility(black_box(&returns)));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("calculate_sharpe_ratio", size),
            size,
            |b, _| {
                b.iter(|| calculate_sharpe_ratio(black_box(&returns), black_box(0.02)));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("calculate_maximum_drawdown", size),
            size,
            |b, _| {
                b.iter(|| calculate_maximum_drawdown(black_box(&prices)));
            },
        );
    }

    group.finish();
}

fn benchmark_technical_indicators(c: &mut Criterion) {
    let mut group = c.benchmark_group("technical_indicators");

    for size in [100, 1000, 10000].iter() {
        let prices = generate_test_data(*size);

        group.bench_with_input(BenchmarkId::new("calculate_sma_20", size), size, |b, _| {
            b.iter(|| calculate_sma(black_box(&prices), black_box(20)));
        });

        group.bench_with_input(BenchmarkId::new("calculate_ema_20", size), size, |b, _| {
            b.iter(|| calculate_ema(black_box(&prices), black_box(20)));
        });

        group.bench_with_input(
            BenchmarkId::new("calculate_bollinger_bands", size),
            size,
            |b, _| {
                b.iter(|| {
                    calculate_bollinger_bands(black_box(&prices), black_box(20), black_box(2.0))
                });
            },
        );

        group.bench_with_input(BenchmarkId::new("calculate_rsi", size), size, |b, _| {
            b.iter(|| calculate_rsi(black_box(&prices), black_box(14)));
        });

        if *size >= 50 {
            // MACD needs more data
            group.bench_with_input(BenchmarkId::new("calculate_macd", size), size, |b, _| {
                b.iter(|| {
                    calculate_macd(
                        black_box(&prices),
                        black_box(12),
                        black_box(26),
                        black_box(9),
                    )
                });
            });
        }
    }

    group.finish();
}

fn benchmark_portfolio_metrics(c: &mut Criterion) {
    let mut group = c.benchmark_group("portfolio_metrics");

    for size in [100, 1000, 5000].iter() {
        let weights = vec![0.25, 0.25, 0.25, 0.25];
        let returns_matrix = vec![
            generate_returns_data(*size),
            generate_returns_data(*size),
            generate_returns_data(*size),
            generate_returns_data(*size),
        ];
        let returns = generate_returns_data(*size);

        group.bench_with_input(
            BenchmarkId::new("calculate_portfolio_returns", size),
            size,
            |b, _| {
                b.iter(|| {
                    calculate_portfolio_returns(black_box(&weights), black_box(&returns_matrix))
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("calculate_correlation_matrix", size),
            size,
            |b, _| {
                b.iter(|| calculate_correlation_matrix(black_box(&returns_matrix)));
            },
        );

        group.bench_with_input(BenchmarkId::new("calculate_var", size), size, |b, _| {
            b.iter(|| calculate_var(black_box(&returns), black_box(0.95)));
        });

        group.bench_with_input(BenchmarkId::new("calculate_cvar", size), size, |b, _| {
            b.iter(|| calculate_cvar(black_box(&returns), black_box(0.95)));
        });
    }

    group.finish();
}

fn benchmark_risk_metrics(c: &mut Criterion) {
    let mut group = c.benchmark_group("risk_metrics");

    for size in [100, 1000, 5000].iter() {
        let asset_returns = generate_returns_data(*size);
        let market_returns = generate_returns_data(*size);
        let benchmark_returns = generate_returns_data(*size);
        let prices = generate_test_data(*size);

        group.bench_with_input(BenchmarkId::new("calculate_beta", size), size, |b, _| {
            b.iter(|| calculate_beta(black_box(&asset_returns), black_box(&market_returns)));
        });

        group.bench_with_input(
            BenchmarkId::new("calculate_information_ratio", size),
            size,
            |b, _| {
                b.iter(|| {
                    calculate_information_ratio(
                        black_box(&asset_returns),
                        black_box(&benchmark_returns),
                    )
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("calculate_sortino_ratio", size),
            size,
            |b, _| {
                b.iter(|| calculate_sortino_ratio(black_box(&asset_returns), black_box(0.02)));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("calculate_calmar_ratio", size),
            size,
            |b, _| {
                b.iter(|| calculate_calmar_ratio(black_box(&asset_returns), black_box(&prices)));
            },
        );
    }

    group.finish();
}

fn benchmark_combined_workflows(c: &mut Criterion) {
    let mut group = c.benchmark_group("combined_workflows");

    for size in [1000, 5000].iter() {
        let prices = generate_test_data(*size);

        group.bench_with_input(
            BenchmarkId::new("complete_technical_analysis", size),
            size,
            |b, _| {
                b.iter(|| {
                    let returns = calculate_returns(black_box(&prices));
                    let volatility = calculate_volatility(&returns);
                    let sharpe = calculate_sharpe_ratio(&returns, 0.02);
                    let max_dd = calculate_maximum_drawdown(&prices);
                    let sma = calculate_sma(&prices, 20);
                    let ema = calculate_ema(&prices, 12);
                    let (bb_upper, bb_middle, bb_lower) =
                        calculate_bollinger_bands(&prices, 20, 2.0);
                    let rsi = calculate_rsi(&prices, 14);

                    // Return something to prevent optimization
                    (
                        volatility,
                        sharpe,
                        max_dd,
                        sma.len(),
                        ema.len(),
                        bb_upper.len(),
                        rsi.len(),
                    )
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("complete_risk_analysis", size),
            size,
            |b, _| {
                b.iter(|| {
                    let returns = calculate_returns(black_box(&prices));
                    let market_returns = generate_returns_data(returns.len());

                    let volatility = calculate_volatility(&returns);
                    let sharpe = calculate_sharpe_ratio(&returns, 0.02);
                    let sortino = calculate_sortino_ratio(&returns, 0.02);
                    let beta = calculate_beta(&returns, &market_returns);
                    let var = calculate_var(&returns, 0.95);
                    let cvar = calculate_cvar(&returns, 0.95);
                    let max_dd = calculate_maximum_drawdown(&prices);
                    let calmar = calculate_calmar_ratio(&returns, &prices);

                    (volatility, sharpe, sortino, beta, var, cvar, max_dd, calmar)
                });
            },
        );
    }

    group.finish();
}

fn benchmark_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_efficiency");

    // Test repeated calculations to check for memory leaks
    let prices = generate_test_data(1000);

    group.bench_function("repeated_calculations", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let returns = calculate_returns(black_box(&prices));
                let _ = calculate_volatility(&returns);
                let _ = calculate_sma(&prices, 20);
                let _ = calculate_ema(&prices, 12);
            }
        });
    });

    group.finish();
}

// Mock implementations for benchmarking if the actual module doesn't exist
#[cfg(not(feature = "trade-math"))]
mod mock_implementations {
    pub fn calculate_returns(prices: &[f64]) -> Vec<f64> {
        prices.windows(2).map(|w| (w[1] - w[0]) / w[0]).collect()
    }

    pub fn calculate_log_returns(prices: &[f64]) -> Vec<f64> {
        prices.windows(2).map(|w| (w[1] / w[0]).ln()).collect()
    }

    pub fn calculate_volatility(returns: &[f64]) -> f64 {
        if returns.is_empty() {
            return 0.0;
        }
        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance =
            returns.iter().map(|&r| (r - mean).powi(2)).sum::<f64>() / (returns.len() - 1) as f64;
        variance.sqrt()
    }

    pub fn calculate_sharpe_ratio(returns: &[f64], risk_free_rate: f64) -> f64 {
        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let excess_return = mean_return - risk_free_rate / 252.0;
        let volatility = calculate_volatility(returns);
        excess_return / volatility
    }

    pub fn calculate_maximum_drawdown(prices: &[f64]) -> f64 {
        let mut max_price = prices[0];
        let mut max_drawdown: f64 = 0.0;

        for &price in prices.iter().skip(1) {
            max_price = max_price.max(price);
            let drawdown = (max_price - price) / max_price;
            max_drawdown = max_drawdown.max(drawdown);
        }

        max_drawdown
    }

    pub fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
        prices
            .windows(period)
            .map(|window| window.iter().sum::<f64>() / window.len() as f64)
            .collect()
    }

    pub fn calculate_ema(prices: &[f64], period: usize) -> Vec<f64> {
        let alpha = 2.0 / (period as f64 + 1.0);
        let mut ema = Vec::with_capacity(prices.len());
        ema.push(prices[0]);

        for &price in prices.iter().skip(1) {
            let prev_ema = ema[ema.len() - 1];
            ema.push(alpha * price + (1.0 - alpha) * prev_ema);
        }

        ema
    }

    // Simplified implementations for other functions...
    pub fn calculate_bollinger_bands(
        prices: &[f64],
        period: usize,
        std_dev: f64,
    ) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        let sma = calculate_sma(prices, period);
        let upper: Vec<f64> = sma.iter().map(|&s| s + std_dev * 0.02).collect();
        let lower: Vec<f64> = sma.iter().map(|&s| s - std_dev * 0.02).collect();
        (upper, sma.clone(), lower)
    }

    pub fn calculate_rsi(prices: &[f64], period: usize) -> Vec<f64> {
        let returns = calculate_returns(prices);
        returns.windows(period).map(|_| 50.0).collect() // Simplified RSI
    }

    pub fn calculate_macd(
        prices: &[f64],
        _fast: usize,
        _slow: usize,
        _signal: usize,
    ) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        let len = prices.len().saturating_sub(26);
        let macd = vec![0.1; len];
        let signal = vec![0.05; len];
        let histogram = vec![0.05; len];
        (macd, signal, histogram)
    }

    pub fn calculate_portfolio_returns(weights: &[f64], returns_matrix: &[Vec<f64>]) -> Vec<f64> {
        let periods = returns_matrix[0].len();
        (0..periods)
            .map(|i| {
                weights
                    .iter()
                    .zip(returns_matrix.iter())
                    .map(|(w, rets)| w * rets[i])
                    .sum()
            })
            .collect()
    }

    pub fn calculate_correlation_matrix(returns_matrix: &[Vec<f64>]) -> Vec<Vec<f64>> {
        let n = returns_matrix.len();
        (0..n)
            .map(|i| (0..n).map(|j| if i == j { 1.0 } else { 0.5 }).collect())
            .collect()
    }

    pub fn calculate_var(returns: &[f64], confidence_level: f64) -> f64 {
        let mut sorted_returns = returns.to_vec();
        sorted_returns.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let index = ((1.0 - confidence_level) * returns.len() as f64) as usize;
        sorted_returns[index.min(sorted_returns.len() - 1)]
    }

    pub fn calculate_cvar(returns: &[f64], confidence_level: f64) -> f64 {
        let var = calculate_var(returns, confidence_level);
        returns.iter().filter(|&&r| r <= var).sum::<f64>() / returns.len() as f64
    }

    pub fn calculate_beta(asset_returns: &[f64], market_returns: &[f64]) -> f64 {
        0.8 // Simplified beta
    }

    pub fn calculate_information_ratio(returns: &[f64], benchmark_returns: &[f64]) -> f64 {
        0.3 // Simplified IR
    }

    pub fn calculate_sortino_ratio(returns: &[f64], risk_free_rate: f64) -> f64 {
        calculate_sharpe_ratio(returns, risk_free_rate) * 1.2
    }

    pub fn calculate_calmar_ratio(returns: &[f64], prices: &[f64]) -> f64 {
        let total_return = returns.iter().sum::<f64>();
        let max_dd = calculate_maximum_drawdown(prices);
        total_return / max_dd.max(0.001)
    }
}

#[cfg(not(feature = "trade-math"))]
use mock_implementations::*;

criterion_group!(
    benches,
    benchmark_basic_calculations,
    benchmark_technical_indicators,
    benchmark_portfolio_metrics,
    benchmark_risk_metrics,
    benchmark_combined_workflows,
    benchmark_memory_efficiency
);

criterion_main!(benches);
