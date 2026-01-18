//! iKKEGOL USB pedal device implementation

use crate::configuration::{Configuration, Trigger};
use crate::device::{DeviceCapabilities, PedalDevice};
use crate::error::{PedalError, Result};
use crate::protocol::{self, ConfigPacket, TriggerMode};
use crate::usb::UsbInterfaceLock;
use crate::{CONFIG_ENDPOINT, CONFIG_INTERFACE, INTERRUPT_IN_ENDPOINT, USB_TIMEOUT_MS};
use log::debug;
use rusb::{Device, DeviceHandle, GlobalContext};
use std::time::Duration;

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
    handle: DeviceHandle<GlobalContext>,
    id: usize,
    model: IkkegolModel,
    version: String,
    capabilities: DeviceCapabilities,
    configurations: Vec<Configuration>,
    trigger_modes: Vec<TriggerMode>,
    modified_pedals: Vec<bool>,
    last_error: Option<String>,
}

impl IkkegolDevice {
    /// Create a new iKKEGOL device
    pub fn new(device: Device<GlobalContext>, id: usize) -> Result<Self> {
        // Get device descriptor to check USB IDs
        let desc = device.device_descriptor()?;
        let vendor_id = desc.vendor_id();
        let product_id = desc.product_id();

        let mut handle = device.open()?;

        // Determine model based on USB ID
        let model = match (vendor_id, product_id) {
            (0x0c45, 0x7403) | (0x0c45, 0x7404) | (0x413d, 0x2107) | (0x3553, 0xb001) => IkkegolModel::PCsensor,
            (0x0426, 0x3011) => IkkegolModel::Scythe,
            (0x055a, 0x0998) => IkkegolModel::Scythe2,
            (0x5131, 0x2019) => IkkegolModel::FootSwitch1P,
            (0x1a86, 0xe026) => {
                // For iKKEGOL devices, try to read the model from the device
                if let Ok((model_str, _)) = Self::read_model_and_version(&mut handle) {
                    IkkegolModel::from_str(&model_str)
                } else {
                    IkkegolModel::FS2020U1IR // Default to 3-pedal model
                }
            },
            _ => IkkegolModel::Unknown(format!("{:04x}:{:04x}", vendor_id, product_id)),
        };

        // Try to read version from device (may not work for all models)
        let version = if let Ok((_, ver)) = Self::read_model_and_version(&mut handle) {
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
            handle,
            id,
            model,
            version,
            capabilities,
            configurations,
            trigger_modes,
            modified_pedals,
            last_error: None,
        })
    }

    /// Read model and version from device
    fn read_model_and_version(handle: &mut DeviceHandle<GlobalContext>) -> Result<(String, String)> {
        let mut lock = UsbInterfaceLock::new(handle, CONFIG_INTERFACE)?;

        // Send read model command
        let cmd = protocol::commands::READ_MODEL;
        // Use a longer timeout for model detection to be safe with all devices
        let timeout = Duration::from_millis(500);

        lock.handle().write_interrupt(
            CONFIG_ENDPOINT | rusb::constants::LIBUSB_ENDPOINT_OUT,
            &cmd,
            timeout,
        )?;

        // Read response (up to 32 bytes in 8-byte chunks)
        let mut response = Vec::new();
        for _ in 0..4 {
            let mut buffer = [0u8; 8];
            match lock.handle().read_interrupt(
                INTERRUPT_IN_ENDPOINT,
                &mut buffer,
                timeout,
            ) {
                Ok(n) => {
                    response.extend_from_slice(&buffer[..n]);
                    if n < 8 {
                        break;
                    }
                }
                Err(_) => break,
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

    /// Read configuration for a specific pedal
    fn read_pedal_config(&mut self, pedal_index: usize) -> Result<()> {
        if pedal_index >= self.capabilities.pedal_count {
            return Err(PedalError::InvalidPedalIndex(
                pedal_index,
                self.capabilities.pedal_count,
            ));
        }

        let protocol_index = self.capabilities.get_protocol_index(pedal_index)
            .ok_or_else(|| PedalError::InvalidPedalIndex(pedal_index, self.capabilities.pedal_count))?;

        let mut lock = UsbInterfaceLock::new(&mut self.handle, CONFIG_INTERFACE)?;

        // Send read config command
        let cmd = protocol::commands::read_config(protocol_index as u8);
        // Some devices may need more time to respond
        let timeout_ms = match self.model {
            IkkegolModel::PCsensor | IkkegolModel::Scythe | IkkegolModel::Scythe2 => 500,  // 500ms for these devices
            _ => USB_TIMEOUT_MS,  // 100ms for others
        };
        let timeout = Duration::from_millis(timeout_ms);

        lock.handle().write_interrupt(
            CONFIG_ENDPOINT | rusb::constants::LIBUSB_ENDPOINT_OUT,
            &cmd,
            timeout,
        )?;

        // Read response (40 bytes in 8-byte chunks)
        let mut packet_bytes = [0u8; 40];
        let mut offset = 0;

        while offset < 40 {
            let mut buffer = [0u8; 8];
            let n = lock.handle().read_interrupt(
                INTERRUPT_IN_ENDPOINT,
                &mut buffer,
                timeout,
            )?;

            let copy_len = std::cmp::min(n, 40 - offset);
            packet_bytes[offset..offset + copy_len].copy_from_slice(&buffer[..copy_len]);
            offset += copy_len;

            if n < 8 {
                break;
            }
        }

        // Parse the packet
        let packet = ConfigPacket::from_bytes(&packet_bytes);
        self.configurations[pedal_index] = protocol::ikkegol::parse_config(&packet)?;

        Ok(())
    }

    /// Read trigger modes for all pedals
    fn read_trigger_modes(&mut self) -> Result<()> {
        let mut lock = UsbInterfaceLock::new(&mut self.handle, CONFIG_INTERFACE)?;

        // Send read trigger modes command
        let cmd = protocol::commands::READ_TRIGGER_MODES;
        // Some devices may need more time to respond
        let timeout_ms = match self.model {
            IkkegolModel::PCsensor | IkkegolModel::Scythe | IkkegolModel::Scythe2 => 500,  // 500ms for these devices
            _ => USB_TIMEOUT_MS,  // 100ms for others
        };
        let timeout = Duration::from_millis(timeout_ms);

        lock.handle().write_interrupt(
            CONFIG_ENDPOINT | rusb::constants::LIBUSB_ENDPOINT_OUT,
            &cmd,
            timeout,
        )?;

        // Read response (up to 8 bytes)
        let mut buffer = [0u8; 8];
        let n = lock.handle().read_interrupt(
            INTERRUPT_IN_ENDPOINT,
            &mut buffer,
            timeout,
        )?;

        // Parse trigger modes
        for i in 0..self.capabilities.pedal_count {
            if i < n {
                self.trigger_modes[i] = TriggerMode::from_u8(buffer[i])
                    .unwrap_or(TriggerMode::Press);
            }
        }

        Ok(())
    }

    /// Write configuration for a specific pedal
    fn write_pedal_config(&mut self, pedal_index: usize) -> Result<()> {
        if pedal_index >= self.capabilities.pedal_count {
            return Err(PedalError::InvalidPedalIndex(
                pedal_index,
                self.capabilities.pedal_count,
            ));
        }

        let protocol_index = self.capabilities.get_protocol_index(pedal_index)
            .ok_or_else(|| PedalError::InvalidPedalIndex(pedal_index, self.capabilities.pedal_count))?;

        let mut lock = UsbInterfaceLock::new(&mut self.handle, CONFIG_INTERFACE)?;
        // Some devices may need more time to respond
        let timeout_ms = match self.model {
            IkkegolModel::PCsensor | IkkegolModel::Scythe | IkkegolModel::Scythe2 => 500,  // 500ms for these devices
            _ => USB_TIMEOUT_MS,  // 100ms for others
        };
        let timeout = Duration::from_millis(timeout_ms);

        // Begin write session
        lock.handle().write_interrupt(
            CONFIG_ENDPOINT | rusb::constants::LIBUSB_ENDPOINT_OUT,
            &protocol::commands::BEGIN_WRITE,
            timeout,
        )?;

        // Encode configuration
        let packet = protocol::ikkegol::encode_config(&self.configurations[pedal_index])?;
        let packet_bytes = packet.to_bytes();

        // Send write config header
        let cmd = protocol::commands::write_config_header(packet.size, protocol_index as u8);
        lock.handle().write_interrupt(
            CONFIG_ENDPOINT | rusb::constants::LIBUSB_ENDPOINT_OUT,
            &cmd,
            timeout,
        )?;

        // Write packet data in 8-byte chunks
        for chunk in packet_bytes.chunks(8) {
            let mut buffer = [0u8; 8];
            buffer[..chunk.len()].copy_from_slice(chunk);
            lock.handle().write_interrupt(
                CONFIG_ENDPOINT | rusb::constants::LIBUSB_ENDPOINT_OUT,
                &buffer,
                timeout,
            )?;
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
        for i in 0..self.capabilities.pedal_count {
            let trigger = Trigger::from(self.trigger_modes[i]);
            self.configurations[i].set_trigger(trigger);
        }

        // Clear modification flags
        self.modified_pedals.fill(false);

        Ok(())
    }

    fn save_configuration(&mut self) -> Result<()> {
        debug!("Saving configuration for device {}", self.id);

        // Write modified pedal configurations
        for i in 0..self.capabilities.pedal_count {
            if self.modified_pedals[i] {
                self.write_pedal_config(i)?;
            }
        }

        // TODO: Write trigger modes if modified

        // Clear modification flags
        self.modified_pedals.fill(false);

        Ok(())
    }

    fn get_pedal_configuration(&self, pedal_index: usize) -> Result<Configuration> {
        if pedal_index >= self.capabilities.pedal_count {
            return Err(PedalError::InvalidPedalIndex(
                pedal_index,
                self.capabilities.pedal_count,
            ));
        }

        Ok(self.configurations[pedal_index].clone())
    }

    fn set_pedal_configuration(&mut self, pedal_index: usize, config: Configuration) -> Result<()> {
        if pedal_index >= self.capabilities.pedal_count {
            return Err(PedalError::InvalidPedalIndex(
                pedal_index,
                self.capabilities.pedal_count,
            ));
        }

        self.configurations[pedal_index] = config;
        self.modified_pedals[pedal_index] = true;

        Ok(())
    }

    fn has_modifications(&self) -> bool {
        self.modified_pedals.iter().any(|&m| m)
    }

    fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }
}