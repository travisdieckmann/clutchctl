//! Device trait definitions

use crate::configuration::Configuration;
use crate::error::Result;

/// Device capabilities
#[derive(Debug, Clone)]
pub struct DeviceCapabilities {
    /// Number of pedals
    pub pedal_count: usize,
    /// First pedal index in protocol (some devices start at 1)
    pub first_pedal_index: usize,
    /// Pedal names for display
    pub pedal_names: Vec<String>,
}

impl DeviceCapabilities {
    /// Get the protocol index for a pedal
    pub fn get_protocol_index(&self, pedal_index: usize) -> Option<usize> {
        if pedal_index < self.pedal_count {
            Some(self.first_pedal_index + pedal_index)
        } else {
            None
        }
    }

    /// Get pedal name by index
    pub fn get_pedal_name(&self, pedal_index: usize) -> Option<&str> {
        self.pedal_names.get(pedal_index).map(|s| s.as_str())
    }

    /// Find pedal index by name
    pub fn find_pedal_by_name(&self, name: &str) -> Option<usize> {
        self.pedal_names.iter()
            .position(|n| n.eq_ignore_ascii_case(name))
    }
}

/// Trait for pedal devices
pub trait PedalDevice {
    /// Get device ID
    fn id(&self) -> usize;

    /// Get device model name
    fn model(&self) -> &str;

    /// Get device version
    fn version(&self) -> &str;

    /// Get device capabilities
    fn capabilities(&self) -> &DeviceCapabilities;

    /// Load configuration from device
    fn load_configuration(&mut self) -> Result<()>;

    /// Save configuration to device
    fn save_configuration(&mut self) -> Result<()>;

    /// Get pedal configuration
    fn get_pedal_configuration(&self, pedal_index: usize) -> Result<Configuration>;

    /// Set pedal configuration
    fn set_pedal_configuration(&mut self, pedal_index: usize, config: Configuration) -> Result<()>;

    /// Check if any configuration has been modified
    fn has_modifications(&self) -> bool;

    /// Get last error message if any
    fn last_error(&self) -> Option<&str>;
}