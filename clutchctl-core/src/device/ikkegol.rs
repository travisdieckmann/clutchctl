//! iKKEGOL USB pedal device implementation

use crate::configuration::{Configuration, Trigger};
use crate::device::{DeviceCapabilities, PedalDevice};
use crate::error::{PedalError, Result};
use crate::protocol::{self, ConfigPacket, TriggerMode};
use crate::usb::{open_device_path, HidDeviceInfo};
use hidapi::HidDevice;
use log::debug;
use std::sync::Mutex;

/// USB pedal device models
#[derive(Debug, Clone)]
pub enum IkkegolModel {
    FS2020U1IR,     // iKKEGOL 3 pedals
    FS2017U1IR,     // iKKEGOL 1 pedal
    PCsensor,       // PCsensor variants (3 pedals)
    Scythe,         // Scythe foot switch (3 pedals)
    Scythe2,        // Scythe v2 foot switch (3 pedals)
    FootSwitch1P,   // Single pedal variant
    Unknown(String),
}

impl IkkegolModel {
    /// Parse model from string
    fn from_str(s: &str) -> Self {
        // Check for known model strings
        if s.contains("FS2020U1IR") {
            Self::FS2020U1IR
        } else if s.contains("FS2017U1IR") {
            Self::FS2017U1IR
        } else {
            Self::Unknown(s.to_string())
        }
    }

    /// Get device capabilities
    fn capabilities(&self) -> DeviceCapabilities {
        match self {
            Self::FS2020U1IR | Self::PCsensor | Self::Scythe | Self::Scythe2 => DeviceCapabilities {
                pedal_count: 3,
                first_pedal_index: 0,
                pedal_names: vec![
                    "left".to_string(),
                    "middle".to_string(),
                    "right".to_string(),
                ],
            },
            Self::FS2017U1IR | Self::FootSwitch1P => DeviceCapabilities {
                pedal_count: 1,
                first_pedal_index: 1, // Note: This model uses index 1, not 0
                pedal_names: vec!["pedal".to_string()],
            },
            Self::Unknown(_) => DeviceCapabilities {
                // Default to 3 pedals for unknown models (likely compatible devices)
                pedal_count: 3,
                first_pedal_index: 0,
                pedal_names: vec![
                    "left".to_string(),
                    "middle".to_string(),
                    "right".to_string(),
                ],
            },
        }
    }
}

/// iKKEGOL pedal device
pub struct IkkegolDevice {
    device: Mutex<HidDevice>,
    id: usize,
    model: IkkegolModel,
    version: String,
    capabilities: DeviceCapabilities,
    configurations: Mutex<Vec<Configuration>>,
    trigger_modes: Mutex<Vec<TriggerMode>>,
    modified_pedals: Mutex<Vec<bool>>,
}

impl IkkegolDevice {
    /// Create a new iKKEGOL device
    pub fn new(info: HidDeviceInfo, id: usize) -> Result<Self> {
        let vendor_id = info.vendor_id;
        let product_id = info.product_id;

        debug!("Opening iKKEGOL device {:04x}:{:04x} at path {:?}",
               vendor_id, product_id, info.path);

        // Open the device by path
        let device = open_device_path(&info.path)?;

        // Set non-blocking mode for reads with timeout
        device.set_blocking_mode(false)?;

        // Determine model based on USB ID
        let model = match (vendor_id, product_id) {
            (0x0c45, 0x7403) | (0x0c45, 0x7404) | (0x413d, 0x2107) | (0x3553, 0xb001) => IkkegolModel::PCsensor,
            (0x0426, 0x3011) => IkkegolModel::Scythe,
            (0x055a, 0x0998) => IkkegolModel::Scythe2,
            (0x5131, 0x2019) => IkkegolModel::FootSwitch1P,
            (0x1a86, 0xe026) => {
                // For iKKEGOL devices, try to read the model from the device
                if let Ok((model_str, _)) = Self::read_model_and_version_static(&device) {
                    IkkegolModel::from_str(&model_str)
                } else {
                    IkkegolModel::FS2020U1IR // Default to 3-pedal model
                }
            },
            _ => IkkegolModel::Unknown(format!("{:04x}:{:04x}", vendor_id, product_id)),
        };

        // Try to read version from device (may not work for all models)
        let version = if let Ok((_, ver)) = Self::read_model_and_version_static(&device) {
            ver
        } else {
            "unknown".to_string()
        };

        let capabilities = model.capabilities();

        // Initialize configuration storage
        let pedal_count = capabilities.pedal_count;
        let configurations = vec![Configuration::Unconfigured; pedal_count];
        let trigger_modes = vec![TriggerMode::Press; pedal_count];
        let modified_pedals = vec![false; pedal_count];

        Ok(Self {
            device: Mutex::new(device),
            id,
            model,
            version,
            capabilities,
            configurations: Mutex::new(configurations),
            trigger_modes: Mutex::new(trigger_modes),
            modified_pedals: Mutex::new(modified_pedals),
        })
    }

    /// Write data to the device (8-byte chunks)
    fn hid_write(device: &HidDevice, data: &[u8]) -> Result<()> {
        // hidapi requires a report ID as the first byte
        // For devices without report IDs, use 0x00
        let mut buffer = vec![0x00];
        buffer.extend_from_slice(data);

        debug!("Writing {} bytes: {:02x?}", data.len(), data);
        device.write(&buffer)?;
        Ok(())
    }

    /// Read data from the device (8 bytes)
    fn hid_read(device: &HidDevice, timeout_ms: i32) -> Result<[u8; 8]> {
        let mut buffer = [0u8; 8];

        // hidapi read returns the number of bytes read
        let bytes_read = device.read_timeout(&mut buffer, timeout_ms)?;

        if bytes_read == 0 {
            return Err(PedalError::Timeout);
        }

        debug!("Read {} bytes: {:02x?}", bytes_read, &buffer[..bytes_read]);
        Ok(buffer)
    }

    /// Read model and version from device (static version for use during construction)
    fn read_model_and_version_static(device: &HidDevice) -> Result<(String, String)> {
        // Send read model command
        let cmd = protocol::commands::READ_MODEL;

        // Write with report ID prefix
        let mut buffer = vec![0x00];
        buffer.extend_from_slice(&cmd);
        device.write(&buffer)?;

        // Read response (up to 32 bytes in 8-byte chunks)
        let mut response = Vec::new();
        for _ in 0..4 {
            let mut buf = [0u8; 8];
            match device.read_timeout(&mut buf, 500) {
                Ok(n) if n > 0 => {
                    response.extend_from_slice(&buf[..n]);
                    if n < 8 {
                        break;
                    }
                }
                _ => break,
            }
        }

        // Parse the response
        let response_str = String::from_utf8_lossy(&response);
        let response_str = response_str.trim_end_matches('\0');

        // Split on underscore to get model and version
        if let Some(underscore_pos) = response_str.rfind('_') {
            let model = response_str[..underscore_pos].to_string();
            let version = response_str[underscore_pos + 1..].to_string();
            Ok((model, version))
        } else {
            Ok((response_str.to_string(), "unknown".to_string()))
        }
    }

    /// Get timeout based on model
    fn get_timeout_ms(&self) -> i32 {
        match self.model {
            IkkegolModel::PCsensor | IkkegolModel::Scythe | IkkegolModel::Scythe2 => 500,
            _ => 100,
        }
    }

    /// Read configuration for a specific pedal
    fn read_pedal_config(&self, pedal_index: usize) -> Result<()> {
        if pedal_index >= self.capabilities.pedal_count {
            return Err(PedalError::InvalidPedalIndex(
                pedal_index,
                self.capabilities.pedal_count,
            ));
        }

        let protocol_index = self.capabilities.get_protocol_index(pedal_index)
            .ok_or_else(|| PedalError::InvalidPedalIndex(pedal_index, self.capabilities.pedal_count))?;

        let device = self.device.lock()
            .map_err(|_| PedalError::Hid("Failed to lock device".to_string()))?;

        // Send read config command
        let cmd = protocol::commands::read_config(protocol_index as u8);
        let timeout_ms = self.get_timeout_ms();

        Self::hid_write(&device, &cmd)?;

        // Read response (40 bytes in 8-byte chunks)
        let mut packet_bytes = [0u8; 40];
        let mut offset = 0;

        while offset < 40 {
            match Self::hid_read(&device, timeout_ms) {
                Ok(buffer) => {
                    let copy_len = std::cmp::min(8, 40 - offset);
                    packet_bytes[offset..offset + copy_len].copy_from_slice(&buffer[..copy_len]);
                    offset += copy_len;
                }
                Err(PedalError::Timeout) if offset > 0 => break,
                Err(e) => return Err(e),
            }
        }

        // Drop device lock before locking configurations
        drop(device);

        // Parse the packet
        let packet = ConfigPacket::from_bytes(&packet_bytes);
        let config = protocol::ikkegol::parse_config(&packet)?;

        let mut configurations = self.configurations.lock()
            .map_err(|_| PedalError::Hid("Failed to lock configurations".to_string()))?;
        configurations[pedal_index] = config;

        Ok(())
    }

    /// Read trigger modes for all pedals
    fn read_trigger_modes(&self) -> Result<()> {
        let device = self.device.lock()
            .map_err(|_| PedalError::Hid("Failed to lock device".to_string()))?;

        // Send read trigger modes command
        let cmd = protocol::commands::READ_TRIGGER_MODES;
        let timeout_ms = self.get_timeout_ms();

        Self::hid_write(&device, &cmd)?;

        // Read response (up to 8 bytes)
        let buffer = Self::hid_read(&device, timeout_ms)?;

        // Drop device lock before locking trigger_modes
        drop(device);

        // Parse trigger modes
        let mut trigger_modes = self.trigger_modes.lock()
            .map_err(|_| PedalError::Hid("Failed to lock trigger modes".to_string()))?;

        for i in 0..self.capabilities.pedal_count {
            if i < 8 {
                trigger_modes[i] = TriggerMode::from_u8(buffer[i])
                    .unwrap_or(TriggerMode::Press);
            }
        }

        Ok(())
    }

    /// Write configuration for a specific pedal
    fn write_pedal_config(&self, pedal_index: usize) -> Result<()> {
        if pedal_index >= self.capabilities.pedal_count {
            return Err(PedalError::InvalidPedalIndex(
                pedal_index,
                self.capabilities.pedal_count,
            ));
        }

        let protocol_index = self.capabilities.get_protocol_index(pedal_index)
            .ok_or_else(|| PedalError::InvalidPedalIndex(pedal_index, self.capabilities.pedal_count))?;

        // Get configuration first
        let config = {
            let configurations = self.configurations.lock()
                .map_err(|_| PedalError::Hid("Failed to lock configurations".to_string()))?;
            configurations[pedal_index].clone()
        };

        let device = self.device.lock()
            .map_err(|_| PedalError::Hid("Failed to lock device".to_string()))?;

        // Begin write session
        Self::hid_write(&device, &protocol::commands::BEGIN_WRITE)?;

        // Encode configuration
        let packet = protocol::ikkegol::encode_config(&config)?;
        let packet_bytes = packet.to_bytes();

        // Send write config header
        let cmd = protocol::commands::write_config_header(packet.size, protocol_index as u8);
        Self::hid_write(&device, &cmd)?;

        // Write packet data in 8-byte chunks
        for chunk in packet_bytes.chunks(8) {
            let mut buffer = [0u8; 8];
            buffer[..chunk.len()].copy_from_slice(chunk);
            Self::hid_write(&device, &buffer)?;
        }

        Ok(())
    }
}

impl PedalDevice for IkkegolDevice {
    fn id(&self) -> usize {
        self.id
    }

    fn model(&self) -> &str {
        match &self.model {
            IkkegolModel::FS2020U1IR => "FS2020U1IR",
            IkkegolModel::FS2017U1IR => "FS2017U1IR",
            IkkegolModel::PCsensor => "PCsensor FootSwitch",
            IkkegolModel::Scythe => "Scythe USB Foot Switch",
            IkkegolModel::Scythe2 => "Scythe USB Foot Switch II",
            IkkegolModel::FootSwitch1P => "FootSwitch (Single Pedal)",
            IkkegolModel::Unknown(s) => s,
        }
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn capabilities(&self) -> &DeviceCapabilities {
        &self.capabilities
    }

    fn load_configuration(&mut self) -> Result<()> {
        debug!("Loading configuration for device {}", self.id);

        // Read configurations for all pedals
        for i in 0..self.capabilities.pedal_count {
            self.read_pedal_config(i)?;
        }

        // Read trigger modes
        self.read_trigger_modes()?;

        // Apply trigger modes to configurations
        {
            let trigger_modes = self.trigger_modes.lock()
                .map_err(|_| PedalError::Hid("Failed to lock trigger modes".to_string()))?;
            let mut configurations = self.configurations.lock()
                .map_err(|_| PedalError::Hid("Failed to lock configurations".to_string()))?;

            for i in 0..self.capabilities.pedal_count {
                let trigger = Trigger::from(trigger_modes[i]);
                configurations[i].set_trigger(trigger);
            }
        }

        // Clear modification flags
        {
            let mut modified_pedals = self.modified_pedals.lock()
                .map_err(|_| PedalError::Hid("Failed to lock modified flags".to_string()))?;
            modified_pedals.fill(false);
        }

        Ok(())
    }

    fn save_configuration(&mut self) -> Result<()> {
        debug!("Saving configuration for device {}", self.id);

        // Get list of modified pedals
        let modified_indices: Vec<usize> = {
            let modified_pedals = self.modified_pedals.lock()
                .map_err(|_| PedalError::Hid("Failed to lock modified flags".to_string()))?;
            (0..self.capabilities.pedal_count)
                .filter(|&i| modified_pedals[i])
                .collect()
        };

        // Write modified pedal configurations
        for i in modified_indices {
            self.write_pedal_config(i)?;
        }

        // Clear modification flags
        {
            let mut modified_pedals = self.modified_pedals.lock()
                .map_err(|_| PedalError::Hid("Failed to lock modified flags".to_string()))?;
            modified_pedals.fill(false);
        }

        Ok(())
    }

    fn get_pedal_configuration(&self, pedal_index: usize) -> Result<Configuration> {
        if pedal_index >= self.capabilities.pedal_count {
            return Err(PedalError::InvalidPedalIndex(
                pedal_index,
                self.capabilities.pedal_count,
            ));
        }

        let configurations = self.configurations.lock()
            .map_err(|_| PedalError::Hid("Failed to lock configurations".to_string()))?;
        Ok(configurations[pedal_index].clone())
    }

    fn set_pedal_configuration(&mut self, pedal_index: usize, config: Configuration) -> Result<()> {
        if pedal_index >= self.capabilities.pedal_count {
            return Err(PedalError::InvalidPedalIndex(
                pedal_index,
                self.capabilities.pedal_count,
            ));
        }

        {
            let mut configurations = self.configurations.lock()
                .map_err(|_| PedalError::Hid("Failed to lock configurations".to_string()))?;
            configurations[pedal_index] = config;
        }

        {
            let mut modified_pedals = self.modified_pedals.lock()
                .map_err(|_| PedalError::Hid("Failed to lock modified flags".to_string()))?;
            modified_pedals[pedal_index] = true;
        }

        Ok(())
    }

    fn has_modifications(&self) -> bool {
        if let Ok(modified_pedals) = self.modified_pedals.lock() {
            modified_pedals.iter().any(|&m| m)
        } else {
            false
        }
    }

    fn last_error(&self) -> Option<&str> {
        None
    }
}
