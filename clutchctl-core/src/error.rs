//! Error types for clutchctl-core

use thiserror::Error;

/// Main error type for clutchctl operations
#[derive(Error, Debug)]
pub enum PedalError {
    /// USB-related errors
    #[error("USB error: {0}")]
    Usb(rusb::Error),

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

    /// Interface already claimed
    #[error("USB interface already claimed")]
    InterfaceClaimed,

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

impl From<rusb::Error> for PedalError {
    fn from(err: rusb::Error) -> Self {
        match err {
            rusb::Error::Access => PedalError::PermissionDenied,
            rusb::Error::Timeout => PedalError::Timeout,
            rusb::Error::Busy => PedalError::InterfaceClaimed,
            _ => PedalError::Usb(err),
        }
    }
}