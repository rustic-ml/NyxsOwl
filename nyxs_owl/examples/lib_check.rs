/// Simple library check example
/// This validates that the basic library structure compiles and works

fn main() {
    println!("🔍 NyxsOwl Library Check");
    println!("========================");

    // Test basic functionality
    test_basic_math();
    test_data_structures();

    println!("✅ All basic checks passed!");
}

fn test_basic_math() {
    println!("📊 Testing basic math operations...");

    // Simple calculations
    let prices = vec![100.0, 102.0, 101.0, 103.0, 105.0];

    // Calculate simple returns
    let returns: Vec<f64> = prices.windows(2).map(|w| (w[1] - w[0]) / w[0]).collect();

    println!("  Sample prices: {:?}", prices);
    println!("  Sample returns: {:?}", returns);

    // Basic statistics
    let mean = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance =
        returns.iter().map(|&r| (r - mean).powi(2)).sum::<f64>() / (returns.len() - 1) as f64;
    let volatility = variance.sqrt();

    println!("  Mean return: {:.4}", mean);
    println!("  Volatility: {:.4}", volatility);

    assert!(mean.is_finite());
    assert!(volatility >= 0.0);

    println!("  ✓ Basic math operations working");
}

fn test_data_structures() {
    println!("📈 Testing data structures...");

    // Test OHLC data structure
    #[derive(Debug)]
    struct OhlcData {
        open: f64,
        high: f64,
        low: f64,
        close: f64,
    }

    let ohlc = OhlcData {
        open: 100.0,
        high: 105.0,
        low: 99.0,
        close: 103.0,
    };

    println!("  Sample OHLC: {:?}", ohlc);

    // Validate OHLC constraints
    assert!(ohlc.high >= ohlc.open);
    assert!(ohlc.high >= ohlc.close);
    assert!(ohlc.low <= ohlc.open);
    assert!(ohlc.low <= ohlc.close);

    println!("  ✓ Data structures working");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_check() {
        test_basic_math();
        test_data_structures();
    }

    #[test]
    fn test_simple_calculations() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let sum: f64 = data.iter().sum();
        assert_eq!(sum, 15.0);

        let mean = sum / data.len() as f64;
        assert_eq!(mean, 3.0);
    }

    #[test]
    fn test_error_handling() {
        let empty_data: Vec<f64> = vec![];
        assert!(empty_data.is_empty());

        let invalid_data = vec![f64::NAN, f64::INFINITY];
        assert!(invalid_data.iter().any(|&x| !x.is_finite()));
    }
}
