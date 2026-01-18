//! Core library for USB HID pedal device configuration
//!
//! This library provides the core functionality for discovering, configuring,
//! and communicating with USB HID pedal devices, particularly iKKEGOL models.

pub mod configuration;
pub mod device;
pub mod error;
pub mod protocol;
pub mod usb;

// Re-export commonly used types
pub use error::{PedalError, Result};

// USB device constants
// Support multiple device types
pub const SUPPORTED_DEVICES: &[(u16, u16, &str)] = &[
    // iKKEGOL/PCsensor devices using footswitch protocol
    (0x1a86, 0xe026, "iKKEGOL"),       // iKKEGOL devices
    (0x3553, 0xb001, "PCsensor"),      // PCsensor FootSwitch
    (0x0c45, 0x7403, "PCsensor"),      // PCsensor variant 1
    (0x0c45, 0x7404, "PCsensor"),      // PCsensor variant 2
    (0x413d, 0x2107, "PCsensor"),      // PCsensor variant 3
    // Scythe devices
    (0x0426, 0x3011, "Scythe"),        // Scythe USB Foot Switch
    (0x055a, 0x0998, "Scythe2"),       // Scythe USB Foot Switch II
    // Single pedal device
    (0x5131, 0x2019, "FootSwitch1P"),  // Single pedal variant
];

// Legacy constants for compatibility (iKKEGOL)
pub const VENDOR_ID: u16 = 0x1a86;
pub const PRODUCT_ID: u16 = 0xe026;

// USB interface constants (common for all devices)
pub const CONFIG_INTERFACE: u8 = 1;
pub const CONFIG_ENDPOINT: u8 = 0x02;
pub const INTERRUPT_IN_ENDPOINT: u8 = 0x82;
pub const USB_TIMEOUT_MS: u64 = 100;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");