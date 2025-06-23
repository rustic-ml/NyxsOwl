use nyxs_owl::prelude::*;
use polars::prelude::*;

/// Simple example demonstrating the hybrid strategy framework
/// 
/// This example shows how to:
/// 1. Load and prepare market data
/// 2. Calculate simple technical indicators
/// 3. Generate basic signals
/// 4. Combine signals using a simple hybrid approach
fn main() -> Result<()> {
    println!("=== NyxsOwl Hybrid Strategy Framework Example ===\n");

    // Step 1: Load and prepare market data
    println!("1. Loading market data...");
    let df = load_sample_data()?;
    println!("   Loaded {} rows of OHLCV data\n", df.height());

    // Step 2: Calculate simple technical indicators
    println!("2. Calculating simple technical indicators...");
    let indicators = calculate_simple_indicators(&df)?;
    println!("   Calculated {} indicators\n", indicators.len());

    // Step 3: Generate basic signals
    println!("3. Generating basic signals...");
    let signals = generate_basic_signals(&df, &indicators)?;
    println!("   Generated {} signals\n", signals.len());

    // Step 4: Analyze signal distribution
    println!("4. Analyzing signal distribution...");
    analyze_signal_distribution(&signals);

    // Step 5: Calculate performance metrics
    println!("5. Calculating performance metrics...");
    let metrics = calculate_performance_metrics(&df, &signals)?;
    print_performance_metrics(&metrics);

    // Step 6: Demonstrate simple indicators
    println!("6. Demonstrating simple indicators...");
    demonstrate_simple_indicators(&df)?;

    println!("\n=== Example completed successfully! ===");
    Ok(())
}

/// Load sample market data for demonstration
fn load_sample_data() -> Result<DataFrame> {
    // Create sample OHLCV data with realistic patterns
    let n_points = 1000;
    let mut prices = Vec::with_capacity(n_points);
    let mut volumes = Vec::with_capacity(n_points);
    
    // Generate trending price data with some volatility
    let mut price = 100.0;
    for i in 0..n_points {
        // Add trend component
        let trend = 0.001 * (i as f64);
        
        // Add volatility component
        let volatility = 0.02 * (i as f64 / 100.0).sin();
        
        // Add random component
        let random = (rand::random::<f64>() - 0.5) * 0.01;
        
        price += trend + volatility + random;
        prices.push(price);
        
        // Generate correlated volume
        let base_volume = 1000000.0;
        let volume_variation = 1.0 + 0.5 * (i as f64 / 50.0).sin();
        volumes.push(base_volume * volume_variation);
    }

    // Create OHLCV data
    let mut opens = Vec::with_capacity(n_points);
    let mut highs = Vec::with_capacity(n_points);
    let mut lows = Vec::with_capacity(n_points);
    let mut closes = Vec::with_capacity(n_points);

    for i in 0..n_points {
        let close = prices[i];
        let open = if i == 0 { close } else { closes[i - 1] };
        let high = open.max(close) * (1.0 + rand::random::<f64>() * 0.01);
        let low = open.min(close) * (1.0 - rand::random::<f64>() * 0.01);
        
        opens.push(open);
        highs.push(high);
        lows.push(low);
        closes.push(close);
    }

    // Create DataFrame
    let df = DataFrame::new(vec![
        Series::new("open".into(), opens).into(),
        Series::new("high".into(), highs).into(),
        Series::new("low".into(), lows).into(),
        Series::new("close".into(), closes).into(),
        Series::new("volume".into(), volumes).into(),
    ])?;

    Ok(df)
}

/// Calculate simple technical indicators
fn calculate_simple_indicators(df: &DataFrame) -> Result<Vec<SimpleIndicator>> {
    let mut indicators = Vec::new();
    let close_series = df.column("close")?.as_series().expect("close column missing");

    // Calculate Simple Moving Average
    let sma_values = calculate_sma(&close_series, 20)?;
    indicators.push(SimpleIndicator {
        name: "SMA_20".to_string(),
        values: sma_values,
        signal_type: SignalType::Trend,
    });

    // Calculate Exponential Moving Average
    let ema_values = calculate_ema(&close_series, 20)?;
    indicators.push(SimpleIndicator {
        name: "EMA_20".to_string(),
        values: ema_values,
        signal_type: SignalType::Trend,
    });

    // Calculate Price Momentum (simple rate of change)
    let momentum_values = calculate_momentum(&close_series, 10)?;
    indicators.push(SimpleIndicator {
        name: "Momentum_10".to_string(),
        values: momentum_values,
        signal_type: SignalType::Momentum,
    });

    // Calculate Volatility (simple rolling standard deviation)
    let volatility_values = calculate_volatility(&close_series, 20)?;
    indicators.push(SimpleIndicator {
        name: "Volatility_20".to_string(),
        values: volatility_values,
        signal_type: SignalType::Volatility,
    });

    Ok(indicators)
}

/// Calculate Simple Moving Average
fn calculate_sma(prices: &Series, period: usize) -> Result<Vec<f64>> {
    let values: Vec<f64> = prices.f64()?.into_iter().filter_map(|x| x).collect();
    let mut sma = Vec::with_capacity(values.len());
    for i in 0..values.len() {
        if i + 1 >= period {
            let sum: f64 = values[i + 1 - period..=i].iter().sum();
            sma.push(sum / period as f64);
        } else {
            sma.push(f64::NAN);
        }
    }
    Ok(sma)
}

/// Calculate Exponential Moving Average
fn calculate_ema(prices: &Series, period: usize) -> Result<Vec<f64>> {
    let mut ema_values = Vec::new();
    let price_values: Vec<f64> = prices.f64()?.into_iter().filter_map(|x| x).collect();
    let multiplier = 2.0 / (period as f64 + 1.0);
    
    for i in 0..price_values.len() {
        if i == 0 {
            ema_values.push(price_values[i]);
        } else {
            let ema = (price_values[i] * multiplier) + (ema_values[i - 1] * (1.0 - multiplier));
            ema_values.push(ema);
        }
    }
    
    Ok(ema_values)
}

/// Calculate simple momentum (rate of change)
fn calculate_momentum(prices: &Series, period: usize) -> Result<Vec<f64>> {
    let mut momentum_values = Vec::new();
    let price_values: Vec<f64> = prices.f64()?.into_iter().filter_map(|x| x).collect();
    
    for i in 0..price_values.len() {
        if i < period {
            momentum_values.push(0.0);
        } else {
            let current_price = price_values[i];
            let past_price = price_values[i - period];
            let momentum = if past_price > 0.0 {
                (current_price - past_price) / past_price
            } else {
                0.0
            };
            momentum_values.push(momentum);
        }
    }
    
    Ok(momentum_values)
}

/// Calculate simple volatility (rolling standard deviation)
fn calculate_volatility(prices: &Series, period: usize) -> Result<Vec<f64>> {
    let values: Vec<f64> = prices.f64()?.into_iter().filter_map(|x| x).collect();
    let mut volatility = Vec::with_capacity(values.len());
    for i in 0..values.len() {
        if i + 1 >= period {
            let window = &values[i + 1 - period..=i];
            let mean = window.iter().sum::<f64>() / window.len() as f64;
            let var = window.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / window.len() as f64;
            volatility.push(var.sqrt());
        } else {
            volatility.push(f64::NAN);
        }
    }
    Ok(volatility)
}

/// Generate basic signals based on indicators
fn generate_basic_signals(df: &DataFrame, indicators: &[SimpleIndicator]) -> Result<Vec<BasicSignal>> {
    let mut signals = Vec::new();
    let close_series = df.column("close")?.as_series().expect("close column missing");
    let price_values: Vec<f64> = close_series.f64()?.into_iter().filter_map(|x| x).collect();

    for i in 0..price_values.len() {
        let mut buy_signals = 0;
        let mut sell_signals = 0;
        let mut total_confidence = 0.0;

        // Check each indicator for signals
        for indicator in indicators {
            if i < indicator.values.len() {
                let value = indicator.values[i];
                let (signal, confidence) = generate_indicator_signal(indicator, value, i, &price_values);
                match signal {
                    Signal::Buy => buy_signals += 1,
                    Signal::Sell => sell_signals += 1,
                    Signal::Hold => {}
                }
                total_confidence += confidence;
            }
        }

        // Combine signals
        let final_signal = if buy_signals > sell_signals {
            Signal::Buy
        } else if sell_signals > buy_signals {
            Signal::Sell
        } else {
            Signal::Hold
        };

        let avg_confidence = if indicators.len() > 0 {
            total_confidence / indicators.len() as f64
        } else {
            0.5
        };

        signals.push(BasicSignal {
            signal: final_signal,
            confidence: avg_confidence,
            buy_count: buy_signals,
            sell_count: sell_signals,
            timestamp: None,
        });
    }

    Ok(signals)
}

/// Generate signal for a specific indicator
fn generate_indicator_signal(
    indicator: &SimpleIndicator,
    value: f64,
    index: usize,
    prices: &[f64],
) -> (Signal, f64) {
    match indicator.name.as_str() {
        "SMA_20" => {
            if index < prices.len() {
                let current_price = prices[index];
                if current_price > value {
                    (Signal::Buy, 0.6)
                } else {
                    (Signal::Sell, 0.6)
                }
            } else {
                (Signal::Hold, 0.5)
            }
        }
        "EMA_20" => {
            if index < prices.len() {
                let current_price = prices[index];
                if current_price > value {
                    (Signal::Buy, 0.7)
                } else {
                    (Signal::Sell, 0.7)
                }
            } else {
                (Signal::Hold, 0.5)
            }
        }
        "Momentum_10" => {
            if value > 0.02 {
                (Signal::Buy, 0.8)
            } else if value < -0.02 {
                (Signal::Sell, 0.8)
            } else {
                (Signal::Hold, 0.3)
            }
        }
        "Volatility_20" => {
            if value > 2.0 {
                (Signal::Hold, 0.9) // High volatility, hold position
            } else if value < 0.5 {
                (Signal::Buy, 0.6) // Low volatility, good for trend following
            } else {
                (Signal::Hold, 0.5)
            }
        }
        _ => (Signal::Hold, 0.5),
    }
}

/// Analyze the distribution of signals
fn analyze_signal_distribution(signals: &[BasicSignal]) {
    let mut buy_count = 0;
    let mut sell_count = 0;
    let mut hold_count = 0;
    let mut total_confidence = 0.0;

    for signal in signals {
        match signal.signal {
            Signal::Buy => buy_count += 1,
            Signal::Sell => sell_count += 1,
            Signal::Hold => hold_count += 1,
        }
        total_confidence += signal.confidence;
    }

    let total = signals.len() as f64;
    println!("   Signal Distribution:");
    println!("     Buy:  {} ({:.1}%)", buy_count, (buy_count as f64 / total) * 100.0);
    println!("     Sell: {} ({:.1}%)", sell_count, (sell_count as f64 / total) * 100.0);
    println!("     Hold: {} ({:.1}%)", hold_count, (hold_count as f64 / total) * 100.0);
    println!("   Average Confidence: {:.3}", total_confidence / total);
}

/// Calculate performance metrics
fn calculate_performance_metrics(
    df: &DataFrame,
    signals: &[BasicSignal],
) -> Result<PerformanceMetrics> {
    let close_series = df.column("close")?.as_series().expect("close column missing");
    let price_values: Vec<f64> = close_series.f64()?.into_iter().filter_map(|x| x).collect();
    let mut returns = Vec::new();
    let mut signal_returns = Vec::new();

    // Calculate price returns
    for i in 1..price_values.len() {
        let current_price = price_values[i];
        let previous_price = price_values[i - 1];
        if previous_price > 0.0 {
            returns.push((current_price - previous_price) / previous_price);
        }
    }

    // Calculate signal-based returns
    for (i, signal) in signals.iter().enumerate() {
        if i < returns.len() {
            let return_value = returns[i];
            let signal_return = match signal.signal {
                Signal::Buy => return_value,
                Signal::Sell => -return_value,
                Signal::Hold => 0.0,
            };
            signal_returns.push(signal_return * signal.confidence);
        }
    }

    // Calculate metrics
    let total_return: f64 = signal_returns.iter().sum();
    let avg_return = if !signal_returns.is_empty() {
        signal_returns.iter().sum::<f64>() / signal_returns.len() as f64
    } else {
        0.0
    };

    let volatility = if signal_returns.len() > 1 {
        let mean = avg_return;
        let variance: f64 = signal_returns.iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / (signal_returns.len() - 1) as f64;
        variance.sqrt()
    } else {
        0.0
    };

    let sharpe_ratio = if volatility > 0.0 {
        avg_return / volatility
    } else {
        0.0
    };

    let max_drawdown = calculate_max_drawdown(&signal_returns);

    Ok(PerformanceMetrics {
        total_return,
        avg_return,
        volatility,
        sharpe_ratio,
        max_drawdown,
        total_signals: signals.len(),
        win_rate: calculate_win_rate(&signal_returns),
    })
}

/// Calculate maximum drawdown
fn calculate_max_drawdown(returns: &[f64]) -> f64 {
    let mut peak = 0.0;
    let mut max_dd = 0.0;
    let mut cumulative = 0.0;

    for &return_value in returns {
        cumulative += return_value;
        if cumulative > peak {
            peak = cumulative;
        }
        let drawdown = peak - cumulative;
        if drawdown > max_dd {
            max_dd = drawdown;
        }
    }

    max_dd
}

/// Calculate win rate
fn calculate_win_rate(returns: &[f64]) -> f64 {
    let wins = returns.iter().filter(|&&r| r > 0.0).count();
    if returns.is_empty() {
        0.0
    } else {
        wins as f64 / returns.len() as f64
    }
}

/// Print performance metrics
fn print_performance_metrics(metrics: &PerformanceMetrics) {
    println!("   Performance Metrics:");
    println!("     Total Return: {:.4} ({:.2}%)", metrics.total_return, metrics.total_return * 100.0);
    println!("     Average Return: {:.4} ({:.2}%)", metrics.avg_return, metrics.avg_return * 100.0);
    println!("     Volatility: {:.4} ({:.2}%)", metrics.volatility, metrics.volatility * 100.0);
    println!("     Sharpe Ratio: {:.4}", metrics.sharpe_ratio);
    println!("     Max Drawdown: {:.4} ({:.2}%)", metrics.max_drawdown, metrics.max_drawdown * 100.0);
    println!("     Win Rate: {:.2}%", metrics.win_rate * 100.0);
    println!("     Total Signals: {}", metrics.total_signals);
}

/// Demonstrate simple indicators
fn demonstrate_simple_indicators(df: &DataFrame) -> Result<()> {
    println!("   Simple Indicators Demo:");
    let close_series = df.column("close")?.as_series().expect("close column missing");
    let price_values: Vec<f64> = close_series.f64()?.into_iter().filter_map(|x| x).collect();
    let sma_values = calculate_sma(&close_series, 20)?;
    let current_price = price_values[price_values.len() - 1];
    let current_sma = sma_values[sma_values.len() - 1];
    println!("     SMA (20): current={:.2}, price_vs_sma={:.2}%", current_sma, ((current_price - current_sma) / current_sma) * 100.0);
    let ema_values = calculate_ema(&close_series, 20)?;
    let current_ema = ema_values[ema_values.len() - 1];
    println!("     EMA (20): current={:.2}, price_vs_ema={:.2}%", current_ema, ((current_price - current_ema) / current_ema) * 100.0);
    let momentum_values = calculate_momentum(&close_series, 10)?;
    let current_momentum = momentum_values[momentum_values.len() - 1];
    println!("     Momentum (10): current={:.4} ({:.2}%)", current_momentum, current_momentum * 100.0);
    let volatility_values = calculate_volatility(&close_series, 20)?;
    let current_volatility = volatility_values[volatility_values.len() - 1];
    println!("     Volatility (20): current={:.4} ({:.2}%)", current_volatility, current_volatility * 100.0);
    Ok(())
}

/// Custom types for the example
#[derive(Debug, Clone)]
struct SimpleIndicator {
    name: String,
    values: Vec<f64>,
    signal_type: SignalType,
}

#[derive(Debug, Clone)]
enum SignalType {
    Momentum,
    Trend,
    Volatility,
    Volume,
}

#[derive(Debug, Clone)]
struct BasicSignal {
    signal: Signal,
    confidence: f64,
    buy_count: usize,
    sell_count: usize,
    timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone)]
struct PerformanceMetrics {
    total_return: f64,
    avg_return: f64,
    volatility: f64,
    sharpe_ratio: f64,
    max_drawdown: f64,
    total_signals: usize,
    win_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_signal_creation() {
        let signal = BasicSignal {
            signal: Signal::Buy,
            confidence: 0.8,
            buy_count: 2,
            sell_count: 1,
            timestamp: None,
        };
        
        assert_eq!(signal.signal, Signal::Buy);
        assert_eq!(signal.confidence, 0.8);
        assert_eq!(signal.buy_count, 2);
        assert_eq!(signal.sell_count, 1);
    }

    #[test]
    fn test_performance_metrics() {
        let returns = vec![0.01, -0.005, 0.02, -0.01, 0.015];
        let max_dd = calculate_max_drawdown(&returns);
        let win_rate = calculate_win_rate(&returns);
        
        assert!(max_dd >= 0.0);
        assert!(win_rate >= 0.0 && win_rate <= 1.0);
    }

    #[test]
    fn test_sma_calculation() {
        let prices = Series::new("close", vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let sma = calculate_sma(&prices, 3).unwrap();
        assert_eq!(sma.len(), 5);
        assert_eq!(sma[4], 4.0); // (3+4+5)/3 = 4.0
    }
} 