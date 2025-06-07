#[derive(Debug, PartialEq, Clone)]
pub enum TrendSignal {
    Uptrend,
    Downtrend,
    NoClearTrend,
}

#[derive(Debug, PartialEq, Clone)]
pub enum CrossoverSignal {
    Bullish,
    Bearish,
    NoSignal,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ActionSignal {
    Buy,
    Sell,
    Hold,
} 