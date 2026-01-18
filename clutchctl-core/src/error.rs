//! Error types for clutchctl-core

use thiserror::Error;

/// Main error type for clutchctl operations
#[derive(Error, Debug)]
pub enum PedalError {
    /// HID-related errors
    #[error("HID error: {0}")]
    Hid(String),

    /// Device not found
    #[error("Device not found with ID {0}")]
    DeviceNotFound(usize),

    /// Invalid device model
    #[error("Unknown device model: {0}")]
    UnknownModel(String),

    /// Protocol errors
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// Invalid pedal index
    #[error("Invalid pedal index {0} for device with {1} pedals")]
    InvalidPedalIndex(usize, usize),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Timeout during USB communication
    #[error("USB communication timeout")]
    Timeout,

    /// Device busy or in use
    #[error("Device busy or in use by another application")]
    DeviceBusy,

    /// Permission denied
    #[error("Permission denied - try running with sudo or check udev rules")]
    PermissionDenied,

    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Unsupported device
    #[error("Unsupported device: {0}")]
    UnsupportedDevice(String),
}

/// Result type alias for PedalError
pub type Result<T> = std::result::Result<T, PedalError>;

impl From<hidapi::HidError> for PedalError {
    fn from(err: hidapi::HidError) -> Self {
        let msg = err.to_string();
        // Try to categorize common errors
        if msg.contains("Permission denied") || msg.contains("access denied") {
            PedalError::PermissionDenied
        } else if msg.contains("timed out") || msg.contains("timeout") {
            PedalError::Timeout
        } else if msg.contains("busy") || msg.contains("in use") {
            PedalError::DeviceBusy
        } else {
            PedalError::Hid(msg)
        }
    }
}