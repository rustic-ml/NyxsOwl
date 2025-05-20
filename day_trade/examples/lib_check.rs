fn main() {
    println!("Checking library modules");

    // Import the mock implementations from our crate
    use day_trade::mock_indicators::{
        Macd, RelativeStrengthIndex, SimpleMovingAverage, TimeSeriesPredictor,
    };

    // Test creating instances of the mock implementations
    let _sma = SimpleMovingAverage::new(14).expect("Failed to create SMA");
    let _rsi = RelativeStrengthIndex::new(14).expect("Failed to create RSI");
    let _macd = Macd::new(12, 26, 9).expect("Failed to create MACD");
    let _predictor =
        TimeSeriesPredictor::new(10, 5, true).expect("Failed to create TimeSeriesPredictor");

    println!("Successfully created mock implementations");
    println!("Done");
}
