//! Error handling for the Hybrid Strategy Framework
//!
//! This module defines error types and handling specific to hybrid strategy operations,
//! ensuring comprehensive error reporting and recovery mechanisms.

use thiserror::Error;

/// Error type for hybrid strategy operations
#[derive(Error, Debug)]
pub enum HybridError {
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Data processing errors
    #[error("Data error: {0}")]
    DataError(String),

    /// Technical indicator errors
    #[error("Technical indicator error: {0}")]
    TechnicalIndicator(String),

    /// Forecasting model errors
    #[error("Forecasting model error: {0}")]
    ForecastingModel(String),

    /// Feature engineering errors
    #[error("Feature engineering error: {0}")]
    FeatureEngineering(String),

    /// Signal confirmation errors
    #[error("Signal confirmation error: {0}")]
    SignalConfirmation(String),

    /// Integration errors
    #[error("Integration error: {0}")]
    Integration(String),

    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    /// Performance errors
    #[error("Performance error: {0}")]
    Performance(String),

    /// Memory errors
    #[error("Memory error: {0}")]
    Memory(String),

    /// Async operation errors
    #[error("Async operation error: {0}")]
    Async(String),

    /// External dependency errors
    #[error("External dependency error: {0}")]
    ExternalDependency(String),

    /// Unknown or unexpected errors
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl HybridError {
    /// Create a configuration error
    pub fn configuration<T: Into<String>>(message: T) -> Self {
        Self::Configuration(message.into())
    }

    /// Create a data error
    pub fn data<T: Into<String>>(message: T) -> Self {
        Self::DataError(message.into())
    }

    /// Create a technical indicator error
    pub fn technical_indicator<T: Into<String>>(message: T) -> Self {
        Self::TechnicalIndicator(message.into())
    }

    /// Create a forecasting model error
    pub fn forecasting_model<T: Into<String>>(message: T) -> Self {
        Self::ForecastingModel(message.into())
    }

    /// Create a feature engineering error
    pub fn feature_engineering<T: Into<String>>(message: T) -> Self {
        Self::FeatureEngineering(message.into())
    }

    /// Create a signal confirmation error
    pub fn signal_confirmation<T: Into<String>>(message: T) -> Self {
        Self::SignalConfirmation(message.into())
    }

    /// Create an integration error
    pub fn integration<T: Into<String>>(message: T) -> Self {
        Self::Integration(message.into())
    }

    /// Create a validation error
    pub fn validation<T: Into<String>>(message: T) -> Self {
        Self::Validation(message.into())
    }

    /// Create a performance error
    pub fn performance<T: Into<String>>(message: T) -> Self {
        Self::Performance(message.into())
    }

    /// Create a memory error
    pub fn memory<T: Into<String>>(message: T) -> Self {
        Self::Memory(message.into())
    }

    /// Create an async operation error
    pub fn async_op<T: Into<String>>(message: T) -> Self {
        Self::Async(message.into())
    }

    /// Create an external dependency error
    pub fn external_dependency<T: Into<String>>(message: T) -> Self {
        Self::ExternalDependency(message.into())
    }

    /// Create an unknown error
    pub fn unknown<T: Into<String>>(message: T) -> Self {
        Self::Unknown(message.into())
    }

    /// Check if the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::DataError(_) | Self::Validation(_) | Self::Performance(_)
        )
    }

    /// Check if the error is critical
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            Self::Configuration(_) | Self::Memory(_) | Self::ExternalDependency(_)
        )
    }

    /// Get error context for debugging
    pub fn context(&self) -> &'static str {
        match self {
            Self::Configuration(_) => "Configuration validation or setup",
            Self::DataError(_) => "Data processing or validation",
            Self::TechnicalIndicator(_) => "Technical indicator calculation",
            Self::ForecastingModel(_) => "Forecasting model operation",
            Self::FeatureEngineering(_) => "Feature extraction or processing",
            Self::SignalConfirmation(_) => "Signal confirmation or validation",
            Self::Integration(_) => "Signal integration or combination",
            Self::Validation(_) => "Input validation or parameter checking",
            Self::Performance(_) => "Performance optimization or timing",
            Self::Memory(_) => "Memory allocation or management",
            Self::Async(_) => "Asynchronous operation",
            Self::ExternalDependency(_) => "External library or service",
            Self::Unknown(_) => "Unknown or unexpected error",
        }
    }
}

impl From<std::io::Error> for HybridError {
    fn from(err: std::io::Error) -> Self {
        Self::DataError(format!("IO error: {}", err))
    }
}

impl From<serde_json::Error> for HybridError {
    fn from(err: serde_json::Error) -> Self {
        Self::DataError(format!("JSON serialization error: {}", err))
    }
}

impl From<chrono::ParseError> for HybridError {
    fn from(err: chrono::ParseError) -> Self {
        Self::DataError(format!("Date/time parsing error: {}", err))
    }
}

impl From<tokio::task::JoinError> for HybridError {
    fn from(err: tokio::task::JoinError) -> Self {
        Self::Async(format!("Task join error: {}", err))
    }
}

impl From<std::sync::PoisonError<std::sync::MutexGuard<'_, ()>>> for HybridError {
    fn from(err: std::sync::PoisonError<std::sync::MutexGuard<'_, ()>>) -> Self {
        Self::Async(format!("Mutex poison error: {}", err))
    }
}

impl From<rayon::ThreadPoolBuildError> for HybridError {
    fn from(err: rayon::ThreadPoolBuildError) -> Self {
        Self::Performance(format!("Thread pool build error: {}", err))
    }
}

impl From<HybridError> for crate::NyxsOwlError {
    fn from(err: HybridError) -> Self {
        crate::NyxsOwlError::StrategyError(format!("Hybrid strategy error: {}", err))
    }
}

/// Result type for hybrid strategy operations
pub type HybridResult<T> = Result<T, HybridError>;

/// Error context for better debugging
#[derive(Debug)]
pub struct ErrorContext {
    /// Operation being performed
    pub operation: String,
    /// Component where error occurred
    pub component: String,
    /// Additional context information
    pub context: std::collections::HashMap<String, String>,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new(operation: String, component: String) -> Self {
        Self {
            operation,
            component,
            context: std::collections::HashMap::new(),
        }
    }

    /// Add context information
    pub fn with_context(mut self, key: String, value: String) -> Self {
        self.context.insert(key, value);
        self
    }

    /// Add multiple context items
    pub fn with_contexts(mut self, contexts: Vec<(String, String)>) -> Self {
        for (key, value) in contexts {
            self.context.insert(key, value);
        }
        self
    }
}

/// Enhanced error with context
#[derive(Debug)]
pub struct ContextualError {
    /// The underlying error
    pub error: HybridError,
    /// Error context
    pub context: ErrorContext,
}

impl ContextualError {
    /// Create a new contextual error
    pub fn new(error: HybridError, context: ErrorContext) -> Self {
        Self { error, context }
    }

    /// Get the underlying error
    pub fn inner(&self) -> &HybridError {
        &self.error
    }

    /// Get the error context
    pub fn context(&self) -> &ErrorContext {
        &self.context
    }
}

impl std::fmt::Display for ContextualError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error in {} ({}): {}",
            self.context.operation, self.context.component, self.error
        )
    }
}

impl std::error::Error for ContextualError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

/// Error handling utilities
pub mod utils {
    use super::*;

    /// Wrap an error with context
    pub fn with_context<T>(
        result: HybridResult<T>,
        operation: &str,
        component: &str,
    ) -> Result<T, ContextualError> {
        result.map_err(|error| {
            let context = ErrorContext::new(operation.to_string(), component.to_string());
            ContextualError::new(error, context)
        })
    }

    /// Log error with context
    pub fn log_error(error: &HybridError, context: &ErrorContext) {
        log::error!(
            "Hybrid strategy error in {} ({}): {}",
            context.operation,
            context.component,
            error
        );

        // Log additional context if available
        if !context.context.is_empty() {
            for (key, value) in &context.context {
                log::error!("  {}: {}", key, value);
            }
        }
    }

    /// Handle recoverable errors
    pub fn handle_recoverable_error(error: &HybridError) -> bool {
        if error.is_recoverable() {
            log::warn!("Recoverable error encountered: {}", error);
            true
        } else {
            log::error!("Critical error encountered: {}", error);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let config_error = HybridError::configuration("Invalid configuration");
        assert!(matches!(config_error, HybridError::Configuration(_)));

        let data_error = HybridError::data("Data validation failed");
        assert!(matches!(data_error, HybridError::DataError(_)));

        let tech_error = HybridError::technical_indicator("RSI calculation failed");
        assert!(matches!(tech_error, HybridError::TechnicalIndicator(_)));
    }

    #[test]
    fn test_error_properties() {
        let recoverable_error = HybridError::validation("Parameter out of range");
        assert!(recoverable_error.is_recoverable());
        assert!(!recoverable_error.is_critical());

        let critical_error = HybridError::configuration("Missing required config");
        assert!(!critical_error.is_recoverable());
        assert!(critical_error.is_critical());
    }

    #[test]
    fn test_error_context() {
        let error = HybridError::data("Invalid data format");
        assert_eq!(error.context(), "Data processing or validation");
    }

    #[test]
    fn test_contextual_error() {
        let inner_error = HybridError::technical_indicator("Calculation failed");
        let context = ErrorContext::new("signal_generation".to_string(), "rsi_indicator".to_string());
        let contextual_error = ContextualError::new(inner_error, context);

        assert_eq!(contextual_error.context().operation, "signal_generation");
        assert_eq!(contextual_error.context().component, "rsi_indicator");
    }

    #[test]
    fn test_error_conversion() {
        let hybrid_error = HybridError::data("Test error");
        let nyxs_error: crate::NyxsOwlError = hybrid_error.into();
        
        assert!(matches!(nyxs_error, crate::NyxsOwlError::StrategyError(_)));
    }
} 