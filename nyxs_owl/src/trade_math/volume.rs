use polars::prelude::*;
use crate::trade_math::moving_averages::calculate_vwap;

/// Calculate On-Balance Volume (OBV)
///
/// OBV is a technical indicator that uses volume flow to predict changes in stock price.
/// The absolute value is not important; the indicator's slope is what matters.
///
/// # Arguments
/// * `close` - Series of closing prices
/// * `volume` - Series of volume data
///
/// # Returns
/// * `PolarsResult<Series>` - Series containing OBV values
pub fn calculate_obv(close: &Series, volume: &Series) -> PolarsResult<Series> {
    let close_values = close.f64()?;
    let volume_values = volume.f64()?;

    if close_values.len() != volume_values.len() {
        return Err(PolarsError::InvalidOperation(
            "Close and volume series must have the same length".into(),
        ));
    }

    let mut obv_values = Vec::with_capacity(close_values.len());
    let mut current_obv = 0.0;
    let mut prev_close = None;

    for i in 0..close_values.len() {
        let close_val = close_values.get(i).unwrap_or(0.0);
        let volume_val = volume_values.get(i).unwrap_or(0.0);

        if let Some(prev) = prev_close {
            if close_val > prev {
                current_obv += volume_val;
            } else if close_val < prev {
                current_obv -= volume_val;
            }
            // If close_val == prev, OBV remains unchanged
        }

        obv_values.push(current_obv);
        prev_close = Some(close_val);
    }

    Ok(Series::new("obv".into(), obv_values))
}

/// Calculate Volume Rate of Change (VROC)
///
/// VROC measures the speed and magnitude of volume changes over a specified period.
/// It helps identify volume trends and potential price reversals.
///
/// # Arguments
/// * `volume` - Series of volume data
/// * `period` - The period for VROC calculation (typically 25)
///
/// # Returns
/// * `PolarsResult<Series>` - Series containing VROC values
pub fn calculate_vroc(volume: &Series, period: usize) -> PolarsResult<Series> {
    if period == 0 {
        return Err(PolarsError::InvalidOperation(
            "VROC period must be greater than 0".into(),
        ));
    }

    let volume_values = volume.f64()?;
    let mut vroc_values = vec![None; volume_values.len()];

    // Calculate VROC for each period
    for i in period..volume_values.len() {
        let current_volume = volume_values.get(i).unwrap_or(0.0);
        let period_ago_volume = volume_values.get(i - period).unwrap_or(0.0);

        if period_ago_volume != 0.0 {
            vroc_values[i] = Some(((current_volume - period_ago_volume) / period_ago_volume) * 100.0);
        } else {
            vroc_values[i] = Some(0.0);
        }
    }

    Ok(Series::new("vroc".into(), vroc_values))
}

/// Calculate VWAP (Volume Weighted Average Price) with bands
///
/// VWAP is the average price weighted by volume. VWAP bands provide support and resistance levels.
///
/// # Arguments
/// * `data` - DataFrame containing OHLCV data
/// * `period` - The period for VWAP calculation (typically 20)
/// * `std_dev` - Number of standard deviations for bands (typically 2.0)
///
/// # Returns
/// * `PolarsResult<(Series, Series, Series)>` - (VWAP, Upper Band, Lower Band)
pub fn calculate_vwap_with_bands(
    data: &DataFrame,
    period: usize,
    std_dev: f64,
) -> PolarsResult<(Series, Series, Series)> {
    let vwap = calculate_vwap(data)?;
    
    // Calculate VWAP standard deviation
    let high = data.column("high")?.f64()?;
    let low = data.column("low")?.f64()?;
    let close = data.column("close")?.f64()?;
    let volume = data.column("volume")?.f64()?;

    let mut vwap_std = vec![None; vwap.len()];

    // Calculate VWAP standard deviation for each period
    for i in (period - 1)..vwap.len() {
        let mut sum_squared_diff = 0.0;
        let mut total_volume = 0.0;

        for j in i.saturating_sub(period - 1)..=i {
            if let (Some(h), Some(l), Some(c), Some(v)) = 
                (high.get(j), low.get(j), close.get(j), volume.get(j)) {
                let typical_price = (h + l + c) / 3.0;
                let vwap_val = vwap.get(j).unwrap().try_extract::<f64>().unwrap_or(0.0);
                let diff = typical_price - vwap_val;
                sum_squared_diff += diff * diff * v;
                total_volume += v;
            }
        }

        if total_volume > 0.0 {
            let variance = sum_squared_diff / total_volume;
            vwap_std[i] = Some(variance.sqrt());
        }
    }

    let vwap_std_series = Series::new("vwap_std".into(), vwap_std);
    
    // Calculate bands
    let upper_band = (&vwap + &(&vwap_std_series * std_dev))?;
    let lower_band = (&vwap - &(&vwap_std_series * std_dev))?;

    Ok((
        vwap.with_name("vwap".into()),
        upper_band.with_name("vwap_upper".into()),
        lower_band.with_name("vwap_lower".into()),
    ))
}

/// Calculate Accumulation/Distribution Line (ADL)
///
/// ADL measures the cumulative flow of money into and out of a security.
/// It combines price and volume to show the relationship between supply and demand.
///
/// # Arguments
/// * `high` - Series of high prices
/// * `low` - Series of low prices
/// * `close` - Series of closing prices
/// * `volume` - Series of volume data
///
/// # Returns
/// * `PolarsResult<Series>` - Series containing ADL values
pub fn calculate_adl(
    high: &Series,
    low: &Series,
    close: &Series,
    volume: &Series,
) -> PolarsResult<Series> {
    let high_values = high.f64()?;
    let low_values = low.f64()?;
    let close_values = close.f64()?;
    let volume_values = volume.f64()?;

    if high_values.len() != low_values.len() 
        || high_values.len() != close_values.len()
        || high_values.len() != volume_values.len() {
        return Err(PolarsError::InvalidOperation(
            "All input series must have the same length".into(),
        ));
    }

    let mut adl_values = Vec::with_capacity(high_values.len());
    let mut current_adl = 0.0;

    for i in 0..high_values.len() {
        let high_val = high_values.get(i).unwrap_or(0.0);
        let low_val = low_values.get(i).unwrap_or(0.0);
        let close_val = close_values.get(i).unwrap_or(0.0);
        let volume_val = volume_values.get(i).unwrap_or(0.0);

        let range = high_val - low_val;
        if range > 0.0 {
            let money_flow_multiplier = ((close_val - low_val) - (high_val - close_val)) / range;
            let money_flow_volume = money_flow_multiplier * volume_val;
            current_adl += money_flow_volume;
        }

        adl_values.push(current_adl);
    }

    Ok(Series::new("adl".into(), adl_values))
}

/// Calculate Chaikin Money Flow (CMF)
///
/// CMF measures the amount of money flow volume over a specific period.
/// It helps identify buying and selling pressure.
///
/// # Arguments
/// * `high` - Series of high prices
/// * `low` - Series of low prices
/// * `close` - Series of closing prices
/// * `volume` - Series of volume data
/// * `period` - The period for CMF calculation (typically 20)
///
/// # Returns
/// * `PolarsResult<Series>` - Series containing CMF values
pub fn calculate_cmf(
    high: &Series,
    low: &Series,
    close: &Series,
    volume: &Series,
    period: usize,
) -> PolarsResult<Series> {
    if period == 0 {
        return Err(PolarsError::InvalidOperation(
            "CMF period must be greater than 0".into(),
        ));
    }

    let high_values = high.f64()?;
    let low_values = low.f64()?;
    let close_values = close.f64()?;
    let volume_values = volume.f64()?;

    if high_values.len() != low_values.len() 
        || high_values.len() != close_values.len()
        || high_values.len() != volume_values.len() {
        return Err(PolarsError::InvalidOperation(
            "All input series must have the same length".into(),
        ));
    }

    let mut cmf_values = vec![None; high_values.len()];

    // Calculate CMF for each period
    for i in (period - 1)..high_values.len() {
        let mut sum_money_flow_volume = 0.0;
        let mut sum_volume = 0.0;

        for j in i.saturating_sub(period - 1)..=i {
            let high_val = high_values.get(j).unwrap_or(0.0);
            let low_val = low_values.get(j).unwrap_or(0.0);
            let close_val = close_values.get(j).unwrap_or(0.0);
            let volume_val = volume_values.get(j).unwrap_or(0.0);

            let range = high_val - low_val;
            if range > 0.0 {
                let money_flow_multiplier = ((close_val - low_val) - (high_val - close_val)) / range;
                let money_flow_volume = money_flow_multiplier * volume_val;
                sum_money_flow_volume += money_flow_volume;
                sum_volume += volume_val;
            }
        }

        if sum_volume > 0.0 {
            cmf_values[i] = Some(sum_money_flow_volume / sum_volume);
        } else {
            cmf_values[i] = Some(0.0);
        }
    }

    Ok(Series::new("cmf".into(), cmf_values))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_obv() {
        let close = Series::new(
            "close".into(),
            vec![10.0, 10.5, 10.2, 10.8, 10.3, 10.6, 10.4, 10.7],
        );
        let volume = Series::new("volume".into(), vec![100.0, 150.0, 80.0, 200.0, 170.0, 150.0, 90.0, 180.0]);

        let obv = calculate_obv(&close, &volume).unwrap();
        let obv_values = obv.f64().unwrap();

        // Test first value is zero
        assert_eq!(obv_values.get(0).unwrap(), 0.0);

        // Test OBV increases when price increases
        assert!(obv_values.get(1).unwrap() > obv_values.get(0).unwrap());

        // Test OBV decreases when price decreases
        let mut has_decrease = false;
        for i in 1..obv_values.len() {
            if obv_values.get(i).unwrap() < obv_values.get(i - 1).unwrap() {
                has_decrease = true;
                break;
            }
        }
        assert!(has_decrease);

        // Test error case: mismatched lengths
        let short_volume = Series::new("volume".into(), vec![100.0, 150.0]);
        assert!(calculate_obv(&close, &short_volume).is_err());
    }
} 