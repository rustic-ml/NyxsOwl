//! Time series data handling for forecasting

use crate::error::{ForecastError, Result};
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use polars::prelude::*;
use std::fs::File;
use std::path::Path;

/// Time series data structure for forecasting
#[derive(Debug, Clone)]
pub struct TimeSeriesData {
    /// Data frame containing the time series data
    df: DataFrame,
    /// Name of the time column
    time_column: String,
    /// Names of the price columns (open, high, low, close)
    price_columns: Vec<String>,
    /// Name of the volume column
    volume_column: Option<String>,
}

/// Data loader for time series data
#[derive(Debug)]
pub struct DataLoader;

impl DataLoader {
    /// Load time series data from a CSV file
    pub fn from_csv<P: AsRef<Path>>(path: P) -> Result<TimeSeriesData> {
        let file = File::open(path)?;
        // Use polars DataFrame reader directly
        let df = CsvReader::new(file)
            .infer_schema(None)
            .has_header(true)
            .finish()?;

        Self::detect_and_create_time_series(df)
    }

    /// Create time series data from an existing DataFrame
    pub fn from_dataframe(df: DataFrame) -> Result<TimeSeriesData> {
        Self::detect_and_create_time_series(df)
    }

    /// Detect time and price columns in a DataFrame and create TimeSeriesData
    fn detect_and_create_time_series(df: DataFrame) -> Result<TimeSeriesData> {
        // Try to find date/time column
        let time_column = Self::detect_time_column(&df)?;

        // Try to find price columns
        let price_columns = Self::detect_price_columns(&df)?;

        // Try to find volume column
        let volume_column = Self::detect_volume_column(&df);

        Ok(TimeSeriesData {
            df,
            time_column,
            price_columns,
            volume_column,
        })
    }

    /// Detect the time column in a DataFrame
    fn detect_time_column(df: &DataFrame) -> Result<String> {
        let column_names = df.get_column_names();

        // Look for common time column names
        for name in &column_names {
            let lower_name = name.to_lowercase();
            if lower_name.contains("time")
                || lower_name.contains("date")
                || lower_name.contains("timestamp")
            {
                return Ok(name.to_string());
            }
        }

        // If not found, use the first column if it looks like a date/time
        if let Some(first_col) = df.get_columns().first() {
            if first_col.dtype().is_temporal() {
                return Ok(first_col.name().to_string());
            }
        }

        Err(ForecastError::DataError(
            "No time column found in data".to_string(),
        ))
    }

    /// Detect price columns in a DataFrame
    fn detect_price_columns(df: &DataFrame) -> Result<Vec<String>> {
        let column_names = df.get_column_names();
        let mut price_columns = Vec::new();

        // Check for OHLC columns
        let required_columns = ["open", "high", "low", "close"];
        for required in &required_columns {
            let mut found = false;
            for name in &column_names {
                if name.to_lowercase().contains(required) {
                    price_columns.push(name.to_string());
                    found = true;
                    break;
                }
            }

            if !found {
                // If we can't find one of the OHLC columns, check if there's a price column
                if required == &"close" {
                    for name in &column_names {
                        if name.to_lowercase().contains("price") {
                            price_columns.push(name.to_string());
                            found = true;
                            break;
                        }
                    }
                }
            }
        }

        if price_columns.is_empty() {
            return Err(ForecastError::DataError(
                "No price columns found in data".to_string(),
            ));
        }

        Ok(price_columns)
    }

    /// Detect volume column in a DataFrame
    fn detect_volume_column(df: &DataFrame) -> Option<String> {
        let column_names = df.get_column_names();

        for name in &column_names {
            if name.to_lowercase().contains("volume") || name.to_lowercase().contains("vol") {
                return Some(name.to_string());
            }
        }

        None
    }
}

impl TimeSeriesData {
    /// Create a new TimeSeriesData instance
    pub fn create_new(
        df: DataFrame,
        time_column: String,
        price_columns: Vec<String>,
        volume_column: Option<String>,
    ) -> Self {
        Self {
            df,
            time_column,
            price_columns,
            volume_column,
        }
    }

    /// Create a new TimeSeriesData from dates and values (for testing)
    pub fn new(dates: Vec<DateTime<Utc>>, values: Vec<f64>) -> Result<Self> {
        // Create a polars Series for dates and values
        let date_series = Series::new(
            "date",
            dates
                .iter()
                .map(|d| d.timestamp_millis())
                .collect::<Vec<i64>>(),
        );
        let values_series = Series::new("close", values);

        // Create a dataframe with the dates and values
        let df = DataFrame::new(vec![date_series, values_series])?;

        Ok(Self {
            df,
            time_column: "date".to_string(),
            price_columns: vec!["close".to_string()],
            volume_column: None,
        })
    }

    /// Create a new TimeSeriesData from OHLC data (for testing)
    pub fn new_ohlc(
        dates: Vec<DateTime<Utc>>,
        ohlc_data: Vec<(f64, f64, f64, f64)>,
    ) -> Result<Self> {
        // Extract OHLC components
        let opens: Vec<f64> = ohlc_data.iter().map(|(o, _, _, _)| *o).collect();
        let highs: Vec<f64> = ohlc_data.iter().map(|(_, h, _, _)| *h).collect();
        let lows: Vec<f64> = ohlc_data.iter().map(|(_, _, l, _)| *l).collect();
        let closes: Vec<f64> = ohlc_data.iter().map(|(_, _, _, c)| *c).collect();

        // Create series
        let date_series = Series::new(
            "date",
            dates
                .iter()
                .map(|d| d.timestamp_millis())
                .collect::<Vec<i64>>(),
        );
        let open_series = Series::new("open", opens);
        let high_series = Series::new("high", highs);
        let low_series = Series::new("low", lows);
        let close_series = Series::new("close", closes);

        // Create dataframe
        let df = DataFrame::new(vec![
            date_series,
            open_series,
            high_series,
            low_series,
            close_series,
        ])?;

        Ok(Self {
            df,
            time_column: "date".to_string(),
            price_columns: vec![
                "open".to_string(),
                "high".to_string(),
                "low".to_string(),
                "close".to_string(),
            ],
            volume_column: None,
        })
    }

    /// Get the DataFrame
    pub fn dataframe(&self) -> &DataFrame {
        &self.df
    }

    /// Get the time column name
    pub fn time_column(&self) -> &str {
        &self.time_column
    }

    /// Get the price column names
    pub fn price_columns(&self) -> &[String] {
        &self.price_columns
    }

    /// Get the volume column name
    pub fn volume_column(&self) -> Option<&String> {
        self.volume_column.as_ref()
    }

    /// Get the close prices as a vector
    pub fn close_prices(&self) -> Vec<f64> {
        let close_idx = self
            .price_columns
            .iter()
            .position(|c| c.to_lowercase().contains("close"))
            .unwrap_or(self.price_columns.len() - 1);

        let col = self.df.column(&self.price_columns[close_idx]).unwrap();
        match col.dtype() {
            DataType::Float64 => col.f64().unwrap().into_iter().flatten().collect(),
            DataType::Float32 => col
                .f32()
                .unwrap()
                .into_iter()
                .flatten()
                .map(|v| v as f64)
                .collect(),
            DataType::Int64 => col
                .i64()
                .unwrap()
                .into_iter()
                .flatten()
                .map(|v| v as f64)
                .collect(),
            DataType::Int32 => col
                .i32()
                .unwrap()
                .into_iter()
                .flatten()
                .map(|v| v as f64)
                .collect(),
            _ => Vec::new(),
        }
    }

    /// Get the open prices as a vector
    pub fn open_prices(&self) -> Vec<f64> {
        let open_idx = self
            .price_columns
            .iter()
            .position(|c| c.to_lowercase().contains("open"))
            .unwrap_or(0);

        let col = self.df.column(&self.price_columns[open_idx]).unwrap();
        match col.dtype() {
            DataType::Float64 => col.f64().unwrap().into_iter().flatten().collect(),
            DataType::Float32 => col
                .f32()
                .unwrap()
                .into_iter()
                .flatten()
                .map(|v| v as f64)
                .collect(),
            DataType::Int64 => col
                .i64()
                .unwrap()
                .into_iter()
                .flatten()
                .map(|v| v as f64)
                .collect(),
            DataType::Int32 => col
                .i32()
                .unwrap()
                .into_iter()
                .flatten()
                .map(|v| v as f64)
                .collect(),
            _ => Vec::new(),
        }
    }

    /// Get the timestamps as a vector
    pub fn timestamps(&self) -> Vec<DateTime<Utc>> {
        let col = self.df.column(&self.time_column).unwrap();

        match col.dtype() {
            DataType::Datetime(_, _) => col
                .datetime()
                .unwrap()
                .into_iter()
                .map(|opt_ts| {
                    opt_ts.map(|ts| {
                        DateTime::<Utc>::from_naive_utc_and_offset(
                            NaiveDateTime::from_timestamp_opt(
                                ts / 1_000_000_000,
                                (ts % 1_000_000_000) as u32,
                            )
                            .unwrap(),
                            Utc,
                        )
                    })
                })
                .flatten()
                .collect(),
            DataType::Date => col
                .date()
                .unwrap()
                .into_iter()
                .map(|opt_date| {
                    opt_date.map(|date| {
                        let naive_date = NaiveDate::from_ymd_opt(1970, 1, 1)
                            .unwrap()
                            .checked_add_days(chrono::Days::new(date as u64))
                            .unwrap();
                        let naive = NaiveDateTime::new(naive_date, chrono::NaiveTime::default());
                        DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc)
                    })
                })
                .flatten()
                .collect(),
            _ => Vec::new(),
        }
    }

    /// Get a slice of the data from start to end index
    pub fn slice(&self, start: usize, end: Option<usize>) -> Result<Self> {
        let end = end.unwrap_or(self.df.height());
        let sliced_df = self.df.slice(start as i64, end - start);

        Ok(TimeSeriesData {
            df: sliced_df,
            time_column: self.time_column.clone(),
            price_columns: self.price_columns.clone(),
            volume_column: self.volume_column.clone(),
        })
    }

    /// Convert to day_trade DailyOhlcv format
    pub fn to_daily_ohlcv(&self) -> Result<Vec<day_trade::DailyOhlcv>> {
        let open_idx = self
            .price_columns
            .iter()
            .position(|c| c.to_lowercase().contains("open"))
            .unwrap_or(0);
        let high_idx = self
            .price_columns
            .iter()
            .position(|c| c.to_lowercase().contains("high"))
            .unwrap_or(1);
        let low_idx = self
            .price_columns
            .iter()
            .position(|c| c.to_lowercase().contains("low"))
            .unwrap_or(2);
        let close_idx = self
            .price_columns
            .iter()
            .position(|c| c.to_lowercase().contains("close"))
            .unwrap_or(3);

        let dates = self.timestamps();
        let opens = self.column_as_f64(&self.price_columns[open_idx])?;
        let highs = self.column_as_f64(&self.price_columns[high_idx])?;
        let lows = self.column_as_f64(&self.price_columns[low_idx])?;
        let closes = self.column_as_f64(&self.price_columns[close_idx])?;

        let volumes = if let Some(vol_col) = &self.volume_column {
            self.column_as_u64(vol_col)?
        } else {
            vec![1000; dates.len()]
        };

        let mut result = Vec::with_capacity(dates.len());
        for i in 0..dates.len() {
            result.push(day_trade::DailyOhlcv {
                date: dates[i].date_naive(),
                data: day_trade::OhlcvData {
                    open: opens[i],
                    high: highs[i],
                    low: lows[i],
                    close: closes[i],
                    volume: volumes[i],
                },
            });
        }

        Ok(result)
    }

    /// Convert to minute_trade MinuteOhlcv format
    pub fn to_minute_ohlcv(&self) -> Result<Vec<minute_trade::MinuteOhlcv>> {
        let open_idx = self
            .price_columns
            .iter()
            .position(|c| c.to_lowercase().contains("open"))
            .unwrap_or(0);
        let high_idx = self
            .price_columns
            .iter()
            .position(|c| c.to_lowercase().contains("high"))
            .unwrap_or(1);
        let low_idx = self
            .price_columns
            .iter()
            .position(|c| c.to_lowercase().contains("low"))
            .unwrap_or(2);
        let close_idx = self
            .price_columns
            .iter()
            .position(|c| c.to_lowercase().contains("close"))
            .unwrap_or(3);

        let timestamps = self.timestamps();
        let opens = self.column_as_f64(&self.price_columns[open_idx])?;
        let highs = self.column_as_f64(&self.price_columns[high_idx])?;
        let lows = self.column_as_f64(&self.price_columns[low_idx])?;
        let closes = self.column_as_f64(&self.price_columns[close_idx])?;

        let volumes = if let Some(vol_col) = &self.volume_column {
            self.column_as_f64(vol_col)?
        } else {
            vec![1000.0; timestamps.len()]
        };

        let mut result = Vec::with_capacity(timestamps.len());
        for i in 0..timestamps.len() {
            result.push(minute_trade::MinuteOhlcv {
                timestamp: timestamps[i],
                data: minute_trade::OhlcvData {
                    open: opens[i],
                    high: highs[i],
                    low: lows[i],
                    close: closes[i],
                    volume: volumes[i],
                },
            });
        }

        Ok(result)
    }

    /// Helper method to get a column as f64 values
    fn column_as_f64(&self, column_name: &str) -> Result<Vec<f64>> {
        let col = self.df.column(column_name).map_err(|e| {
            ForecastError::DataError(format!("Column '{}' not found: {}", column_name, e))
        })?;

        match col.dtype() {
            DataType::Float64 => Ok(col.f64().unwrap().into_iter().flatten().collect()),
            DataType::Float32 => Ok(col
                .f32()
                .unwrap()
                .into_iter()
                .flatten()
                .map(|v| v as f64)
                .collect()),
            DataType::Int64 => Ok(col
                .i64()
                .unwrap()
                .into_iter()
                .flatten()
                .map(|v| v as f64)
                .collect()),
            DataType::Int32 => Ok(col
                .i32()
                .unwrap()
                .into_iter()
                .flatten()
                .map(|v| v as f64)
                .collect()),
            DataType::UInt64 => Ok(col
                .u64()
                .unwrap()
                .into_iter()
                .flatten()
                .map(|v| v as f64)
                .collect()),
            DataType::UInt32 => Ok(col
                .u32()
                .unwrap()
                .into_iter()
                .flatten()
                .map(|v| v as f64)
                .collect()),
            _ => Err(ForecastError::DataError(format!(
                "Column '{}' cannot be converted to f64",
                column_name
            ))),
        }
    }

    /// Helper method to get a column as u64 values
    fn column_as_u64(&self, column_name: &str) -> Result<Vec<u64>> {
        let col = self.df.column(column_name).map_err(|e| {
            ForecastError::DataError(format!("Column '{}' not found: {}", column_name, e))
        })?;

        match col.dtype() {
            DataType::UInt64 => Ok(col.u64().unwrap().into_iter().flatten().collect()),
            DataType::UInt32 => Ok(col
                .u32()
                .unwrap()
                .into_iter()
                .flatten()
                .map(|v| v as u64)
                .collect()),
            DataType::Int64 => Ok(col
                .i64()
                .unwrap()
                .into_iter()
                .flatten()
                .filter_map(|v| if v >= 0 { Some(v as u64) } else { None })
                .collect()),
            DataType::Int32 => Ok(col
                .i32()
                .unwrap()
                .into_iter()
                .flatten()
                .filter_map(|v| if v >= 0 { Some(v as u64) } else { None })
                .collect()),
            DataType::Float64 => Ok(col
                .f64()
                .unwrap()
                .into_iter()
                .flatten()
                .filter_map(|v| if v >= 0.0 { Some(v as u64) } else { None })
                .collect()),
            DataType::Float32 => Ok(col
                .f32()
                .unwrap()
                .into_iter()
                .flatten()
                .filter_map(|v| if v >= 0.0 { Some(v as u64) } else { None })
                .collect()),
            _ => Err(ForecastError::DataError(format!(
                "Column '{}' cannot be converted to u64",
                column_name
            ))),
        }
    }

    /// Check if the time series is empty
    pub fn is_empty(&self) -> bool {
        self.df.height() == 0
    }

    /// Get the length of the time series
    pub fn len(&self) -> usize {
        self.df.height()
    }

    /// Calculate the mean of the close prices
    pub fn mean(&self) -> Result<f64> {
        let close_prices = self.close_prices();
        if close_prices.is_empty() {
            return Err(ForecastError::DataError(
                "No close prices available".to_string(),
            ));
        }

        let sum: f64 = close_prices.iter().sum();
        Ok(sum / close_prices.len() as f64)
    }

    /// Calculate the standard deviation of the close prices
    pub fn std_dev(&self) -> Result<f64> {
        let close_prices = self.close_prices();
        if close_prices.is_empty() {
            return Err(ForecastError::DataError(
                "No close prices available".to_string(),
            ));
        }

        let mean = self.mean()?;
        let variance: f64 = close_prices
            .iter()
            .map(|price| (price - mean).powi(2))
            .sum::<f64>()
            / close_prices.len() as f64;

        Ok(variance.sqrt())
    }

    /// Calculate mean absolute error between this time series and another
    pub fn mean_absolute_error(&self, other: &Self) -> Result<f64> {
        let prices1 = self.close_prices();
        let prices2 = other.close_prices();

        if prices1.len() != prices2.len() {
            return Err(ForecastError::DataError(
                "Time series have different lengths".to_string(),
            ));
        }

        let sum: f64 = prices1
            .iter()
            .zip(prices2.iter())
            .map(|(p1, p2)| (p1 - p2).abs())
            .sum();

        Ok(sum / prices1.len() as f64)
    }

    /// Calculate mean squared error between this time series and another
    pub fn mean_squared_error(&self, other: &Self) -> Result<f64> {
        let prices1 = self.close_prices();
        let prices2 = other.close_prices();

        if prices1.len() != prices2.len() {
            return Err(ForecastError::DataError(
                "Time series have different lengths".to_string(),
            ));
        }

        let sum: f64 = prices1
            .iter()
            .zip(prices2.iter())
            .map(|(p1, p2)| (p1 - p2).powi(2))
            .sum();

        Ok(sum / prices1.len() as f64)
    }
}
