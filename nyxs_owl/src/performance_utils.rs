//! High-Performance SIMD-Accelerated Mathematical Operations
//!
//! This module provides vectorized implementations of mathematical operations
//! commonly used in financial forecasting and time series analysis.

#![allow(unused_imports)]

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// SIMD-accelerated arithmetic operations for financial computations
pub struct SimdMath;

impl SimdMath {
    /// Vectorized addition of two f64 arrays using AVX2
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    pub unsafe fn add_f64_avx2(a: &[f64], b: &[f64], result: &mut [f64]) {
        assert_eq!(a.len(), b.len());
        assert_eq!(a.len(), result.len());
        assert!(
            a.len() % 4 == 0,
            "Array length must be multiple of 4 for AVX2"
        );

        let chunks = a.len() / 4;

        for i in 0..chunks {
            let idx = i * 4;

            // Load 4 f64 values into AVX2 registers
            let va = _mm256_loadu_pd(a.as_ptr().add(idx));
            let vb = _mm256_loadu_pd(b.as_ptr().add(idx));

            // Perform vectorized addition
            let vresult = _mm256_add_pd(va, vb);

            // Store result
            _mm256_storeu_pd(result.as_mut_ptr().add(idx), vresult);
        }
    }

    /// Vectorized multiplication of two f64 arrays using AVX2
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    pub unsafe fn mul_f64_avx2(a: &[f64], b: &[f64], result: &mut [f64]) {
        assert_eq!(a.len(), b.len());
        assert_eq!(a.len(), result.len());
        assert!(
            a.len() % 4 == 0,
            "Array length must be multiple of 4 for AVX2"
        );

        let chunks = a.len() / 4;

        for i in 0..chunks {
            let idx = i * 4;

            let va = _mm256_loadu_pd(a.as_ptr().add(idx));
            let vb = _mm256_loadu_pd(b.as_ptr().add(idx));

            let vresult = _mm256_mul_pd(va, vb);

            _mm256_storeu_pd(result.as_mut_ptr().add(idx), vresult);
        }
    }

    /// Vectorized dot product using AVX2
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    pub unsafe fn dot_product_avx2(a: &[f64], b: &[f64]) -> f64 {
        assert_eq!(a.len(), b.len());
        assert!(
            a.len() % 4 == 0,
            "Array length must be multiple of 4 for AVX2"
        );

        let chunks = a.len() / 4;
        let mut acc = _mm256_setzero_pd();

        for i in 0..chunks {
            let idx = i * 4;

            let va = _mm256_loadu_pd(a.as_ptr().add(idx));
            let vb = _mm256_loadu_pd(b.as_ptr().add(idx));

            // Multiply and accumulate
            let vmul = _mm256_mul_pd(va, vb);
            acc = _mm256_add_pd(acc, vmul);
        }

        // Horizontal sum of accumulator
        let acc_high = _mm256_extractf128_pd(acc, 1);
        let acc_low = _mm256_castpd256_pd128(acc);
        let acc_128 = _mm_add_pd(acc_high, acc_low);
        let acc_64 = _mm_add_pd(acc_128, _mm_shuffle_pd(acc_128, acc_128, 1));

        _mm_cvtsd_f64(acc_64)
    }

    /// Vectorized autocorrelation calculation for ARIMA models
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    pub unsafe fn autocorrelation_avx2(data: &[f64], lag: usize) -> f64 {
        if lag >= data.len() {
            return 0.0;
        }

        let n = data.len() - lag;
        if n == 0 {
            return 0.0;
        }

        // Calculate means
        let mean1 = Self::mean_avx2(&data[..n]);
        let mean2 = Self::mean_avx2(&data[lag..]);

        // Calculate centered values and their product
        let mut numerator = 0.0;
        let mut var1 = 0.0;
        let mut var2 = 0.0;

        // Process in chunks of 4 for SIMD
        let simd_chunks = n / 4;

        if simd_chunks > 0 {
            let vmean1 = _mm256_set1_pd(mean1);
            let vmean2 = _mm256_set1_pd(mean2);
            let mut vnum_acc = _mm256_setzero_pd();
            let mut vvar1_acc = _mm256_setzero_pd();
            let mut vvar2_acc = _mm256_setzero_pd();

            for i in 0..simd_chunks {
                let idx = i * 4;

                let vdata1 = _mm256_loadu_pd(data.as_ptr().add(idx));
                let vdata2 = _mm256_loadu_pd(data.as_ptr().add(idx + lag));

                // Center the data
                let vcentered1 = _mm256_sub_pd(vdata1, vmean1);
                let vcentered2 = _mm256_sub_pd(vdata2, vmean2);

                // Calculate products for numerator and variances
                let vprod = _mm256_mul_pd(vcentered1, vcentered2);
                let vsq1 = _mm256_mul_pd(vcentered1, vcentered1);
                let vsq2 = _mm256_mul_pd(vcentered2, vcentered2);

                vnum_acc = _mm256_add_pd(vnum_acc, vprod);
                vvar1_acc = _mm256_add_pd(vvar1_acc, vsq1);
                vvar2_acc = _mm256_add_pd(vvar2_acc, vsq2);
            }

            // Horizontal sum of accumulators
            numerator += Self::horizontal_sum_pd(vnum_acc);
            var1 += Self::horizontal_sum_pd(vvar1_acc);
            var2 += Self::horizontal_sum_pd(vvar2_acc);
        }

        // Process remaining elements
        for i in (simd_chunks * 4)..n {
            let centered1 = data[i] - mean1;
            let centered2 = data[i + lag] - mean2;

            numerator += centered1 * centered2;
            var1 += centered1 * centered1;
            var2 += centered2 * centered2;
        }

        // Calculate correlation coefficient
        let denominator = (var1 * var2).sqrt();
        if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        }
    }

    /// Vectorized mean calculation using AVX2
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    pub unsafe fn mean_avx2(data: &[f64]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }

        let chunks = data.len() / 4;
        let mut acc = _mm256_setzero_pd();

        // Process chunks of 4
        for i in 0..chunks {
            let idx = i * 4;
            let vdata = _mm256_loadu_pd(data.as_ptr().add(idx));
            acc = _mm256_add_pd(acc, vdata);
        }

        let mut sum = Self::horizontal_sum_pd(acc);

        // Process remaining elements
        for i in (chunks * 4)..data.len() {
            sum += data[i];
        }

        sum / data.len() as f64
    }

    /// Vectorized variance calculation using AVX2
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    pub unsafe fn variance_avx2(data: &[f64]) -> f64 {
        if data.len() <= 1 {
            return 0.0;
        }

        let mean = Self::mean_avx2(data);
        let chunks = data.len() / 4;
        let vmean = _mm256_set1_pd(mean);
        let mut acc = _mm256_setzero_pd();

        // Process chunks of 4
        for i in 0..chunks {
            let idx = i * 4;
            let vdata = _mm256_loadu_pd(data.as_ptr().add(idx));
            let vdiff = _mm256_sub_pd(vdata, vmean);
            let vsq = _mm256_mul_pd(vdiff, vdiff);
            acc = _mm256_add_pd(acc, vsq);
        }

        let mut sum_sq = Self::horizontal_sum_pd(acc);

        // Process remaining elements
        for i in (chunks * 4)..data.len() {
            let diff = data[i] - mean;
            sum_sq += diff * diff;
        }

        sum_sq / (data.len() - 1) as f64
    }

    /// Helper function to perform horizontal sum of __m256d
    #[cfg(target_arch = "x86_64")]
    #[inline]
    unsafe fn horizontal_sum_pd(v: __m256d) -> f64 {
        let high = _mm256_extractf128_pd(v, 1);
        let low = _mm256_castpd256_pd128(v);
        let sum_128 = _mm_add_pd(high, low);
        let sum_64 = _mm_add_pd(sum_128, _mm_shuffle_pd(sum_128, sum_128, 1));
        _mm_cvtsd_f64(sum_64)
    }

    /// Safe wrapper for SIMD operations with fallback
    pub fn safe_dot_product(a: &[f64], b: &[f64]) -> f64 {
        if a.len() != b.len() {
            return 0.0;
        }

        // Check if we can use SIMD (length multiple of 4)
        #[cfg(target_arch = "x86_64")]
        {
            if a.len() >= 4 && a.len() % 4 == 0 && is_x86_feature_detected!("avx2") {
                return unsafe { Self::dot_product_avx2(a, b) };
            }
        }

        // Fallback to scalar implementation
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }

    /// Safe wrapper for SIMD autocorrelation with fallback
    pub fn safe_autocorrelation(data: &[f64], lag: usize) -> f64 {
        #[cfg(target_arch = "x86_64")]
        {
            if data.len() >= 8 && is_x86_feature_detected!("avx2") {
                return unsafe { Self::autocorrelation_avx2(data, lag) };
            }
        }

        // Fallback to simple implementation
        Self::autocorrelation_scalar(data, lag)
    }

    /// Safe wrapper for SIMD mean calculation
    pub fn safe_mean(data: &[f64]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }

        #[cfg(target_arch = "x86_64")]
        {
            if data.len() >= 4 && data.len() % 4 == 0 && is_x86_feature_detected!("avx2") {
                return unsafe { Self::mean_avx2(data) };
            }
        }

        data.iter().sum::<f64>() / data.len() as f64
    }

    /// Safe wrapper for SIMD variance calculation
    pub fn safe_variance(data: &[f64]) -> f64 {
        if data.len() <= 1 {
            return 0.0;
        }

        #[cfg(target_arch = "x86_64")]
        {
            if data.len() >= 4 && is_x86_feature_detected!("avx2") {
                return unsafe { Self::variance_avx2(data) };
            }
        }

        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let sum_sq: f64 = data.iter().map(|x| (x - mean).powi(2)).sum();
        sum_sq / (data.len() - 1) as f64
    }

    /// Scalar fallback for autocorrelation
    fn autocorrelation_scalar(data: &[f64], lag: usize) -> f64 {
        if lag >= data.len() {
            return 0.0;
        }

        let n = data.len() - lag;
        if n == 0 {
            return 0.0;
        }

        let mean1: f64 = data[..n].iter().sum::<f64>() / n as f64;
        let mean2: f64 = data[lag..].iter().sum::<f64>() / n as f64;

        let mut numerator = 0.0;
        let mut var1 = 0.0;
        let mut var2 = 0.0;

        for i in 0..n {
            let centered1 = data[i] - mean1;
            let centered2 = data[i + lag] - mean2;

            numerator += centered1 * centered2;
            var1 += centered1 * centered1;
            var2 += centered2 * centered2;
        }

        let denominator = (var1 * var2).sqrt();
        if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        }
    }
}

/// Batch processing utilities for multiple time series
pub struct SimdBatch;

impl SimdBatch {
    /// Process multiple time series correlations simultaneously
    pub fn batch_correlations(series_data: &[&[f64]], lags: &[usize]) -> Vec<Vec<f64>> {
        series_data
            .iter()
            .map(|series| {
                lags.iter()
                    .map(|&lag| SimdMath::safe_autocorrelation(series, lag))
                    .collect()
            })
            .collect()
    }

    /// Batch mean calculations for portfolio analysis
    pub fn batch_means(series_data: &[&[f64]]) -> Vec<f64> {
        series_data
            .iter()
            .map(|series| SimdMath::safe_mean(series))
            .collect()
    }

    /// Batch variance calculations for risk analysis
    pub fn batch_variances(series_data: &[&[f64]]) -> Vec<f64> {
        series_data
            .iter()
            .map(|series| SimdMath::safe_variance(series))
            .collect()
    }
}

/// Performance benchmarking utilities
pub struct SimdBenchmark;

impl SimdBenchmark {
    /// Run a simple performance comparison between SIMD and scalar implementations
    pub fn run_performance_comparison() {
        use std::time::Instant;

        println!("🚀 SIMD Performance Comparison");
        println!("==============================");

        // Test with different data sizes
        let sizes = vec![1000, 10000, 100000];

        for size in sizes {
            println!("\n📊 Testing with {} data points:", size);

            // Generate test data
            let data1: Vec<f64> = (0..size).map(|i| (i as f64 * 0.1).sin()).collect();
            let data2: Vec<f64> = (0..size).map(|i| (i as f64 * 0.1).cos()).collect();

            // Test dot product
            let start = Instant::now();
            let scalar_result = data1
                .iter()
                .zip(data2.iter())
                .map(|(x, y)| x * y)
                .sum::<f64>();
            let scalar_time = start.elapsed();

            let start = Instant::now();
            let simd_result = SimdMath::safe_dot_product(&data1, &data2);
            let simd_time = start.elapsed();

            let speedup = scalar_time.as_nanos() as f64 / simd_time.as_nanos() as f64;

            println!("  • Dot Product:");
            println!(
                "    - Scalar: {:?} (result: {:.6})",
                scalar_time, scalar_result
            );
            println!("    - SIMD:   {:?} (result: {:.6})", simd_time, simd_result);
            println!("    - Speedup: {:.2}x", speedup);

            // Test variance calculation
            let start = Instant::now();
            let mean = data1.iter().sum::<f64>() / data1.len() as f64;
            let scalar_variance =
                data1.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (data1.len() - 1) as f64;
            let scalar_var_time = start.elapsed();

            let start = Instant::now();
            let simd_variance = SimdMath::safe_variance(&data1);
            let simd_var_time = start.elapsed();

            let var_speedup = scalar_var_time.as_nanos() as f64 / simd_var_time.as_nanos() as f64;

            println!("  • Variance:");
            println!(
                "    - Scalar: {:?} (result: {:.6})",
                scalar_var_time, scalar_variance
            );
            println!(
                "    - SIMD:   {:?} (result: {:.6})",
                simd_var_time, simd_variance
            );
            println!("    - Speedup: {:.2}x", var_speedup);
        }

        println!("\n✅ Performance comparison complete!");

        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                println!("💪 AVX2 support detected - optimal performance enabled!");
            } else {
                println!("⚠️  AVX2 not available - using scalar fallback");
            }
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            println!("ℹ️  Non-x86_64 architecture - using scalar implementation");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_mean() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let mean = SimdMath::safe_mean(&data);
        assert!((mean - 4.5).abs() < 1e-10);
    }

    #[test]
    fn test_simd_variance() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let var = SimdMath::safe_variance(&data);
        let expected = 6.0; // Variance of 1..8
        assert!((var - expected).abs() < 1e-10);
    }

    #[test]
    fn test_simd_autocorrelation() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let autocorr = SimdMath::safe_autocorrelation(&data, 1);
        // Autocorrelation at lag 1 should be high for monotonic sequence
        assert!(autocorr > 0.8);
    }

    #[test]
    fn test_batch_processing() {
        let series1 = vec![1.0, 2.0, 3.0, 4.0];
        let series2 = vec![2.0, 4.0, 6.0, 8.0];
        let series_data = vec![&series1[..], &series2[..]];

        let means = SimdBatch::batch_means(&series_data);
        assert_eq!(means.len(), 2);
        assert!((means[0] - 2.5).abs() < 1e-10);
        assert!((means[1] - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_simd_dot_product() {
        let a = vec![1.0, 2.0, 3.0, 4.0];
        let b = vec![2.0, 3.0, 4.0, 5.0];
        let result = SimdMath::safe_dot_product(&a, &b);
        let expected = 40.0; // 1*2 + 2*3 + 3*4 + 4*5 = 40
        assert!((result - expected).abs() < 1e-10);
    }

    #[test]
    fn test_performance_benchmark() {
        // Just ensure the benchmark doesn't crash
        SimdBenchmark::run_performance_comparison();
    }
}
