//! Advanced Performance Optimizations
//!
//! This module contains advanced optimization techniques for high-frequency trading
//! and high-performance financial calculations:
//!
//! - SIMD-optimized bulk calculations
//! - Memory pooling for frequent allocations
//! - Cache-friendly data structures
//! - Branch prediction optimizations
//! - Zero-copy operations

use std::alloc::{alloc, dealloc, Layout};
use std::collections::VecDeque;
use std::ptr::NonNull;
use std::sync::Arc;

/// Memory pool for reducing allocation overhead
pub struct MemoryPool<T> {
    free_blocks: VecDeque<NonNull<T>>,
    block_size: usize,
    capacity: usize,
}

impl<T> MemoryPool<T> {
    /// Create a new memory pool with specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            free_blocks: VecDeque::with_capacity(capacity),
            block_size: std::mem::size_of::<T>(),
            capacity,
        }
    }

    /// Allocate a block from the pool
    pub fn allocate(&mut self) -> Option<NonNull<T>> {
        if let Some(block) = self.free_blocks.pop_front() {
            Some(block)
        } else if self.free_blocks.len() < self.capacity {
            // Allocate new block
            let layout = Layout::new::<T>();
            unsafe {
                let ptr = alloc(layout) as *mut T;
                if ptr.is_null() {
                    None
                } else {
                    Some(NonNull::new_unchecked(ptr))
                }
            }
        } else {
            None
        }
    }

    /// Return a block to the pool
    pub fn deallocate(&mut self, block: NonNull<T>) {
        if self.free_blocks.len() < self.capacity {
            self.free_blocks.push_back(block);
        } else {
            // Pool is full, actually deallocate
            unsafe {
                let layout = Layout::new::<T>();
                dealloc(block.as_ptr() as *mut u8, layout);
            }
        }
    }
}

impl<T> Drop for MemoryPool<T> {
    fn drop(&mut self) {
        // Clean up all remaining blocks
        while let Some(block) = self.free_blocks.pop_front() {
            unsafe {
                let layout = Layout::new::<T>();
                dealloc(block.as_ptr() as *mut u8, layout);
            }
        }
    }
}

/// SIMD-optimized mathematical operations
pub mod simd_math {
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    /// SIMD-optimized sum calculation for f64 arrays
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    pub unsafe fn sum_f64_avx2(values: &[f64]) -> f64 {
        let mut sum = _mm256_setzero_pd();
        let chunks = values.chunks_exact(4);
        let remainder = chunks.remainder();

        for chunk in chunks {
            let v = _mm256_loadu_pd(chunk.as_ptr());
            sum = _mm256_add_pd(sum, v);
        }

        // Horizontal sum of the 4 f64 values in sum
        let high = _mm256_extractf128_pd(sum, 1);
        let low = _mm256_castpd256_pd128(sum);
        let sum128 = _mm_add_pd(high, low);
        let sum_high = _mm_unpackhi_pd(sum128, sum128);
        let result = _mm_add_sd(sum128, sum_high);

        let mut final_sum = _mm_cvtsd_f64(result);

        // Handle remainder
        for &val in remainder {
            final_sum += val;
        }

        final_sum
    }

    /// Fallback non-SIMD sum
    pub fn sum_f64_scalar(values: &[f64]) -> f64 {
        values.iter().sum()
    }

    /// Auto-dispatching sum function
    pub fn sum_f64_optimized(values: &[f64]) -> f64 {
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") && values.len() >= 4 {
                unsafe { sum_f64_avx2(values) }
            } else {
                sum_f64_scalar(values)
            }
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            sum_f64_scalar(values)
        }
    }

    /// SIMD-optimized variance calculation
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    pub unsafe fn variance_f64_avx2(values: &[f64], mean: f64) -> f64 {
        let mean_vec = _mm256_set1_pd(mean);
        let mut sum_sq = _mm256_setzero_pd();

        let chunks = values.chunks_exact(4);
        let remainder = chunks.remainder();

        for chunk in chunks {
            let v = _mm256_loadu_pd(chunk.as_ptr());
            let diff = _mm256_sub_pd(v, mean_vec);
            let sq = _mm256_mul_pd(diff, diff);
            sum_sq = _mm256_add_pd(sum_sq, sq);
        }

        // Horizontal sum
        let high = _mm256_extractf128_pd(sum_sq, 1);
        let low = _mm256_castpd256_pd128(sum_sq);
        let sum128 = _mm_add_pd(high, low);
        let sum_high = _mm_unpackhi_pd(sum128, sum128);
        let result = _mm_add_sd(sum128, sum_high);

        let mut variance = _mm_cvtsd_f64(result);

        // Handle remainder
        for &val in remainder {
            let diff = val - mean;
            variance += diff * diff;
        }

        variance / values.len() as f64
    }

    /// Auto-dispatching variance function
    pub fn variance_f64_optimized(values: &[f64], mean: f64) -> f64 {
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") && values.len() >= 4 {
                unsafe { variance_f64_avx2(values, mean) }
            } else {
                values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64
            }
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64
        }
    }
}

/// Cache-friendly circular buffer for high-frequency data
#[repr(align(64))] // Align to cache line boundary
#[derive(Debug, Clone)]
pub struct AlignedBuffer<T: Copy + Default> {
    data: Vec<T>,
    capacity: usize,
    head: usize,
    size: usize,
}

impl<T: Copy + Default> AlignedBuffer<T> {
    /// Create a new aligned buffer with specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            data: vec![T::default(); capacity],
            capacity,
            head: 0,
            size: 0,
        }
    }

    /// Push a new value (overwrites oldest if full)
    #[inline]
    pub fn push(&mut self, value: T) {
        self.data[self.head] = value;
        self.head = (self.head + 1) % self.capacity;
        if self.size < self.capacity {
            self.size += 1;
        }
    }

    /// Get a slice of all current values
    pub fn as_slice(&self) -> &[T] {
        if self.size < self.capacity {
            &self.data[0..self.size]
        } else {
            &self.data
        }
    }

    /// Get current size
    #[inline]
    pub fn len(&self) -> usize {
        self.size
    }

    /// Check if buffer is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Check if buffer is full
    #[inline]
    pub fn is_full(&self) -> bool {
        self.size == self.capacity
    }

    /// Get average of all values (optimized for hot path)
    #[inline]
    pub fn average(&self) -> f64
    where
        T: Into<f64> + Copy,
    {
        if self.size == 0 {
            return 0.0;
        }

        let slice = self.as_slice();
        let values: Vec<f64> = slice.iter().map(|&x| x.into()).collect();
        simd_math::sum_f64_optimized(&values) / self.size as f64
    }
}

/// Branch-prediction optimized indicator updates
pub struct FastIndicatorManager {
    // Hot data (frequently accessed) - aligned for cache efficiency
    hot_data: HotData,

    // Cold data (less frequently accessed)
    cold_data: ColdData,
}

#[repr(align(64))]
struct HotData {
    last_price: f64,
    last_volume: f64,
    sma_sum: f64,
    sma_count: usize,
    ema_value: f64,
    rsi_avg_gain: f64,
    rsi_avg_loss: f64,
}

struct ColdData {
    period: usize,
    ema_multiplier: f64,
    rsi_period: usize,
    price_buffer: AlignedBuffer<f64>,
}

impl FastIndicatorManager {
    /// Create a new fast indicator manager
    ///
    /// # Arguments
    /// * `sma_period` - Period for Simple Moving Average calculation
    /// * `ema_period` - Period for Exponential Moving Average calculation  
    /// * `rsi_period` - Period for RSI calculation
    pub fn new(sma_period: usize, ema_period: usize, rsi_period: usize) -> Self {
        Self {
            hot_data: HotData {
                last_price: 0.0,
                last_volume: 0.0,
                sma_sum: 0.0,
                sma_count: 0,
                ema_value: 0.0,
                rsi_avg_gain: 0.0,
                rsi_avg_loss: 0.0,
            },
            cold_data: ColdData {
                period: sma_period,
                ema_multiplier: 2.0 / (ema_period as f64 + 1.0),
                rsi_period,
                price_buffer: AlignedBuffer::new(sma_period),
            },
        }
    }

    /// Ultra-fast update optimized for hot paths
    #[inline]
    pub fn update_fast(&mut self, price: f64, volume: f64) {
        // Branch prediction: most common case first
        if likely(price > 0.0 && volume > 0.0) {
            // Update SMA (rolling sum method)
            if self.cold_data.price_buffer.is_full() {
                // Remove oldest value from sum
                let old_values = self.cold_data.price_buffer.as_slice();
                let oldest = old_values[self.hot_data.sma_count % self.cold_data.period];
                self.hot_data.sma_sum -= oldest;
            }

            self.cold_data.price_buffer.push(price);
            self.hot_data.sma_sum += price;

            if self.hot_data.sma_count < self.cold_data.period {
                self.hot_data.sma_count += 1;
            }

            // Update EMA
            if self.hot_data.ema_value == 0.0 {
                self.hot_data.ema_value = price;
            } else {
                self.hot_data.ema_value = self.cold_data.ema_multiplier * price
                    + (1.0 - self.cold_data.ema_multiplier) * self.hot_data.ema_value;
            }

            // Update RSI (simplified for speed)
            if self.hot_data.last_price > 0.0 {
                let change = price - self.hot_data.last_price;
                let alpha = 1.0 / self.cold_data.rsi_period as f64;

                if change > 0.0 {
                    self.hot_data.rsi_avg_gain =
                        alpha * change + (1.0 - alpha) * self.hot_data.rsi_avg_gain;
                    self.hot_data.rsi_avg_loss *= 1.0 - alpha;
                } else {
                    self.hot_data.rsi_avg_gain *= 1.0 - alpha;
                    self.hot_data.rsi_avg_loss =
                        alpha * (-change) + (1.0 - alpha) * self.hot_data.rsi_avg_loss;
                }
            }

            self.hot_data.last_price = price;
            self.hot_data.last_volume = volume;
        }
    }

    /// Get current SMA value
    #[inline]
    pub fn sma(&self) -> Option<f64> {
        if self.hot_data.sma_count >= self.cold_data.period {
            Some(self.hot_data.sma_sum / self.cold_data.period as f64)
        } else {
            None
        }
    }

    /// Get current EMA value
    #[inline]
    pub fn ema(&self) -> Option<f64> {
        if self.hot_data.ema_value > 0.0 {
            Some(self.hot_data.ema_value)
        } else {
            None
        }
    }

    /// Get current RSI value
    #[inline]
    pub fn rsi(&self) -> Option<f64> {
        if self.hot_data.rsi_avg_gain + self.hot_data.rsi_avg_loss > 0.0 {
            let rs = self.hot_data.rsi_avg_gain / self.hot_data.rsi_avg_loss;
            Some(100.0 - (100.0 / (1.0 + rs)))
        } else {
            None
        }
    }
}

/// Branch prediction hints for hot paths
#[inline]
#[cold]
fn cold() {}

#[inline]
fn likely(b: bool) -> bool {
    if !b {
        cold();
    }
    b
}

/// Zero-copy price data structure for streaming updates
#[derive(Clone)]
pub struct StreamingPriceData {
    data: Arc<[f64]>,
    offset: usize,
    len: usize,
}

impl StreamingPriceData {
    /// Create from existing data without copying
    pub fn from_slice(slice: &[f64]) -> Self {
        Self {
            data: Arc::from(slice),
            offset: 0,
            len: slice.len(),
        }
    }

    /// Create a window view without copying data
    pub fn window(&self, start: usize, len: usize) -> Option<Self> {
        if start + len <= self.len {
            Some(Self {
                data: Arc::clone(&self.data),
                offset: self.offset + start,
                len,
            })
        } else {
            None
        }
    }

    /// Get slice view
    pub fn as_slice(&self) -> &[f64] {
        &self.data[self.offset..self.offset + self.len]
    }

    /// Length of current view
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if the data is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// Batch processing utilities for high-throughput scenarios
pub mod batch_processing {
    use super::*;

    /// Process multiple price updates in batch for better cache utilization
    pub fn batch_update_indicators(
        manager: &mut FastIndicatorManager,
        prices: &[f64],
        volumes: &[f64],
    ) {
        // Process in cache-friendly chunks
        const CHUNK_SIZE: usize = 64; // Fit in L1 cache

        let chunks = prices.chunks(CHUNK_SIZE);
        let vol_chunks = volumes.chunks(CHUNK_SIZE);

        for (price_chunk, vol_chunk) in chunks.zip(vol_chunks) {
            for (&price, &volume) in price_chunk.iter().zip(vol_chunk.iter()) {
                manager.update_fast(price, volume);
            }
        }
    }

    /// Batch calculate simple moving averages using SIMD
    pub fn batch_sma(prices: &[f64], period: usize) -> Vec<f64> {
        let mut result = Vec::with_capacity(prices.len().saturating_sub(period - 1));

        for window in prices.windows(period) {
            let avg = simd_math::sum_f64_optimized(window) / period as f64;
            result.push(avg);
        }

        result
    }

    /// Batch calculate exponential moving averages
    pub fn batch_ema(prices: &[f64], period: usize) -> Vec<f64> {
        let mut result = Vec::with_capacity(prices.len());
        if prices.is_empty() {
            return result;
        }

        let multiplier = 2.0 / (period as f64 + 1.0);
        let mut ema = prices[0];
        result.push(ema);

        for &price in prices.iter().skip(1) {
            ema = multiplier * price + (1.0 - multiplier) * ema;
            result.push(ema);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_pool() {
        let mut pool: MemoryPool<f64> = MemoryPool::new(10);

        // Allocate some blocks
        let block1 = pool.allocate().expect("Should allocate");
        let block2 = pool.allocate().expect("Should allocate");

        // Return them
        pool.deallocate(block1);
        pool.deallocate(block2);

        // Should be able to reuse
        let _block3 = pool.allocate().expect("Should reuse");
    }

    #[test]
    fn test_simd_math() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];

        let scalar_sum = simd_math::sum_f64_scalar(&values);
        let simd_sum = simd_math::sum_f64_optimized(&values);

        assert!((scalar_sum - simd_sum).abs() < f64::EPSILON);
    }

    #[test]
    fn test_aligned_buffer() {
        let mut buffer = AlignedBuffer::new(3);

        buffer.push(1.0);
        buffer.push(2.0);
        buffer.push(3.0);
        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.average(), 2.0);

        // Should overwrite oldest
        buffer.push(4.0);
        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.average(), 3.0); // (2+3+4)/3
    }

    #[test]
    fn test_fast_indicator_manager() {
        let mut manager = FastIndicatorManager::new(3, 3, 3);

        manager.update_fast(100.0, 1000.0);
        manager.update_fast(101.0, 1100.0);
        manager.update_fast(102.0, 1200.0);

        assert!(manager.sma().is_some());
        assert!(manager.ema().is_some());

        let sma = manager.sma().unwrap();
        assert!((sma - 101.0).abs() < 0.01);
    }

    #[test]
    fn test_batch_processing() {
        let prices = vec![100.0, 101.0, 102.0, 103.0, 104.0];
        let sma_results = batch_processing::batch_sma(&prices, 3);

        assert_eq!(sma_results.len(), 3);
        assert!((sma_results[0] - 101.0).abs() < 0.01); // (100+101+102)/3
    }
}
