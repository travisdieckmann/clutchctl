//! PCsensor USB pedal device implementation using HID protocol

use crate::configuration::Configuration;
use crate::device::{DeviceCapabilities, PedalDevice};
use crate::error::{PedalError, Result};
use crate::protocol::{TriggerMode, ModifierKeys, HID_KEYMAP};
use crate::configuration::keyboard::{KeyboardConfiguration, KeyMode};
use crate::configuration::mouse::{MouseConfiguration, MouseButton, MouseMode};
use crate::configuration::text::TextConfiguration;
use crate::usb::{open_device_path, HidDeviceInfo};
use hidapi::HidDevice;
use log::debug;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

/// PCsensor device models
#[derive(Debug, Clone)]
pub enum PCsensorModel {
    FootSwitch3Pedal,  // Most PCsensor devices have 3 pedals
    FootSwitch1Pedal,  // Single pedal variant (VID: 5131, PID: 2019)
}

/// PCsensor pedal device using HID protocol
pub struct PCsensorDevice {
    device: Mutex<HidDevice>,
    id: usize,
    model: PCsensorModel,
    version: String,
    capabilities: DeviceCapabilities,
    configurations: Mutex<Vec<Configuration>>,
    trigger_modes: Mutex<Vec<TriggerMode>>,
    modified_pedals: Mutex<Vec<bool>>,
}

impl PCsensorDevice {
    /// Create a new PCsensor device
    pub fn new(info: HidDeviceInfo, id: usize) -> Result<Self> {
        debug!("Opening PCsensor device {:04x}:{:04x} at path {:?}",
               info.vendor_id, info.product_id, info.path);

        // Open the device by path
        let device = open_device_path(&info.path)?;

        // Set non-blocking mode for reads with timeout
        device.set_blocking_mode(false)?;

        // Determine model based on product ID
        let model = if info.product_id == 0x2019 {
            PCsensorModel::FootSwitch1Pedal
        } else {
            PCsensorModel::FootSwitch3Pedal
        };

        let capabilities = match model {
            PCsensorModel::FootSwitch3Pedal => DeviceCapabilities {
                pedal_count: 3,
                first_pedal_index: 0,
                pedal_names: vec![
                    "left".to_string(),
                    "middle".to_string(),
                    "right".to_string(),
                ],
            },
            PCsensorModel::FootSwitch1Pedal => DeviceCapabilities {
                pedal_count: 1,
                first_pedal_index: 0,
                pedal_names: vec!["pedal".to_string()],
            },
        };

        let pedal_count = capabilities.pedal_count;
        let configurations = vec![Configuration::Unconfigured; pedal_count];
        let trigger_modes = vec![TriggerMode::Press; pedal_count];
        let modified_pedals = vec![false; pedal_count];

        let mut device_obj = Self {
            device: Mutex::new(device),
            id,
            model,
            version: "V5.7".to_string(), // Default version
            capabilities,
            configurations: Mutex::new(configurations),
            trigger_modes: Mutex::new(trigger_modes),
            modified_pedals: Mutex::new(modified_pedals),
        };

        // Load current configuration
        debug!("Loading initial device configuration");
        device_obj.load_configuration()?;
        debug!("Successfully initialized PCsensor device");

        Ok(device_obj)
    }

    /// Write HID report to device
    fn hid_write(device: &HidDevice, data: &[u8; 8]) -> Result<()> {
        debug!("Writing HID report: {:02x?}", data);

        // hidapi requires a report ID as the first byte
        // For devices without report IDs, use 0x00
        let mut buffer = [0u8; 9];
        buffer[0] = 0x00; // Report ID
        buffer[1..9].copy_from_slice(data);

        device.write(&buffer)?;
        thread::sleep(Duration::from_millis(30));
        Ok(())
    }

    /// Read HID report from device
    fn hid_read(device: &HidDevice) -> Result<[u8; 8]> {
        let mut buffer = [0u8; 8];
        let timeout_ms = 1000;

        let bytes_read = device.read_timeout(&mut buffer, timeout_ms)?;

        if bytes_read == 0 {
            return Err(PedalError::Timeout);
        }

        if bytes_read != 8 {
            return Err(PedalError::Protocol(
                format!("Expected 8 bytes, got {}", bytes_read)
            ));
        }

        debug!("Read HID report: {:02x?}", buffer);
        Ok(buffer)
    }

    /// Parse configuration from HID report
    fn parse_configuration(data: &[u8; 8]) -> Configuration {
        match data[1] {
            0 => Configuration::Unconfigured,
            1 | 0x81 => {
                // Keyboard configuration
                let mut keys = Vec::new();
                if data[3] != 0 {
                    // Try to decode scan code to key name using HID keymap
                    if let Some(key_name) = HID_KEYMAP.decode_key(data[3]) {
                        keys.push(key_name.to_string());
                    } else {
                        // Fall back to hex representation for unknown codes
                        keys.push(format!("0x{:02x}", data[3]));
                    }
                }

                let modifiers = ModifierKeys::from_bits_truncate(data[2]);
                let mode = if data[1] == 0x81 {
                    KeyMode::OneShot
                } else {
                    KeyMode::Standard
                };

                Configuration::Keyboard(KeyboardConfiguration::with_modifiers(mode, keys, modifiers))
            },
            2 => {
                // Mouse configuration
                let buttons = match data[4] {
                    1 => vec![MouseButton::Left],
                    2 => vec![MouseButton::Right],
                    4 => vec![MouseButton::Middle],
                    _ => vec![],
                };

                let x = data[5] as i8;
                let y = data[6] as i8;
                let wheel = data[7] as i8;

                if !buttons.is_empty() {
                    Configuration::Mouse(MouseConfiguration::buttons(buttons.into_iter().collect()))
                } else {
                    Configuration::Mouse(MouseConfiguration::axis(x, y, wheel))
                }
            },
            3 => {
                // Combined keyboard and mouse - for now, just return keyboard part
                let mut keys = Vec::new();
                if data[3] != 0 {
                    // Try to decode scan code to key name using HID keymap
                    if let Some(key_name) = HID_KEYMAP.decode_key(data[3]) {
                        keys.push(key_name.to_string());
                    } else {
                        // Fall back to hex representation for unknown codes
                        keys.push(format!("0x{:02x}", data[3]));
                    }
                }
                let modifiers = ModifierKeys::from_bits_truncate(data[2]);
                Configuration::Keyboard(KeyboardConfiguration::with_modifiers(KeyMode::Standard, keys, modifiers))
            },
            4 => {
                // String configuration - would need to read more data
                Configuration::Text(TextConfiguration::new(String::new()))
            },
            _ => Configuration::Unconfigured,
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

        let device = self.device.lock()
            .map_err(|_| PedalError::Hid("Failed to lock device".to_string()))?;

        // Send read command for this pedal
        let query: [u8; 8] = [0x01, 0x82, 0x08, (pedal_index + 1) as u8, 0, 0, 0, 0];
        Self::hid_write(&device, &query)?;

        // Read first response packet
        let response = Self::hid_read(&device)?;

        // Check if this is a text configuration that needs more data
        let (config, trigger_mode) = if response[1] == 0x04 {
            // Text configuration - read additional packets
            let text_len = (response[0] as usize).saturating_sub(2).min(38);
            let mut text_data = vec![0u8; 38];

            // The first packet contains the first 6 bytes of text
            if text_len > 0 {
                let first_chunk_len = text_len.min(6);
                text_data[..first_chunk_len].copy_from_slice(&response[2..2 + first_chunk_len]);
            }

            // Read remaining packets if needed
            let mut bytes_read = 6;
            while bytes_read < text_len {
                let packet = Self::hid_read(&device)?;
                let chunk_len = (text_len - bytes_read).min(8);
                text_data[bytes_read..bytes_read + chunk_len].copy_from_slice(&packet[..chunk_len]);
                bytes_read += chunk_len;
            }

            // Decode text from HID scan codes
            let mut text_array: [u8; 38] = [0; 38];
            text_array.copy_from_slice(&text_data);
            let text = TextConfiguration::decode_from_protocol(&text_array);
            (Configuration::Text(TextConfiguration::new(text)), TriggerMode::Press)
        } else {
            // Parse other configuration types normally
            let config = Self::parse_configuration(&response);

            // Trigger mode is in the response type (0x81 means inverted/one-shot)
            let trigger_mode = if response[1] & 0x80 != 0 {
                TriggerMode::Release
            } else {
                TriggerMode::Press
            };

            (config, trigger_mode)
        };

        // Drop device lock before acquiring other locks
        drop(device);

        // Update configurations
        {
            let mut configurations = self.configurations.lock()
                .map_err(|_| PedalError::Hid("Failed to lock configurations".to_string()))?;
            configurations[pedal_index] = config;
        }

        // Update trigger modes
        {
            let mut trigger_modes = self.trigger_modes.lock()
                .map_err(|_| PedalError::Hid("Failed to lock trigger modes".to_string()))?;
            trigger_modes[pedal_index] = trigger_mode;
        }

        Ok(())
    }

    /// Encode configuration to HID format
    fn encode_configuration(config: &Configuration, _trigger: TriggerMode) -> Vec<u8> {
        let mut data = Vec::new();

        match config {
            Configuration::Keyboard(kb) => {
                // Type byte
                let type_byte = if kb.mode == KeyMode::OneShot {
                    0x81
                } else {
                    0x01
                };
                data.push(8); // Length
                data.push(type_byte);
                data.push(kb.modifiers.bits());
                // Parse first key if it exists
                let key_code = if !kb.keys.is_empty() {
                    // First try hex codes for backward compatibility
                    if kb.keys[0].starts_with("0x") {
                        u8::from_str_radix(&kb.keys[0][2..], 16).unwrap_or(0)
                    } else {
                        // Try to encode key name using HID keymap
                        HID_KEYMAP.encode_key(&kb.keys[0]).unwrap_or(0)
                    }
                } else {
                    0
                };
                data.push(key_code);
                data.extend_from_slice(&[0, 0, 0, 0]); // Padding
            },
            Configuration::Mouse(m) => {
                data.push(8); // Length
                data.push(0x02); // Type
                data.push(0); // Reserved
                data.push(0); // Reserved

                // Encode mouse buttons or axis
                match &m.mode {
                    MouseMode::Buttons(buttons) => {
                        let mut button_byte = 0u8;
                        for button in buttons {
                            match button {
                                MouseButton::Left => button_byte |= 1,
                                MouseButton::Right => button_byte |= 2,
                                MouseButton::Middle => button_byte |= 4,
                                _ => {}
                            }
                        }
                        data.push(button_byte);
                        data.push(0);
                        data.push(0);
                        data.push(0);
                    },
                    MouseMode::Axis { x, y, wheel } => {
                        data.push(0); // No buttons
                        data.push(*x as u8);
                        data.push(*y as u8);
                        data.push(*wheel as u8);
                    }
                }
            },
            Configuration::Text(text) => {
                // PCsensor text type is 0x04
                let text_data = text.encode_for_protocol();

                // First 2 bytes: length, type
                // Then the text data (HID scan codes)
                let mut packet = Vec::new();

                // Count non-zero bytes in text_data
                let text_len = text_data.iter().take_while(|&&b| b != 0).count();

                packet.push((text_len + 2).min(40) as u8); // Length (including header)
                packet.push(0x04); // Type: String

                // Add text data (up to 38 bytes to fit in 40-byte packet)
                packet.extend_from_slice(&text_data[..text_len.min(38)]);

                // Pad to 8 bytes minimum
                while packet.len() < 8 {
                    packet.push(0);
                }

                data.extend_from_slice(&packet);
            },
            _ => {
                // Unconfigured
                data.extend_from_slice(&[8, 0, 0, 0, 0, 0, 0, 0]);
            }
        }

        data
    }

    /// Write configuration for a specific pedal
    fn write_pedal_config(&self, pedal_index: usize) -> Result<()> {
        if pedal_index >= self.capabilities.pedal_count {
            return Err(PedalError::InvalidPedalIndex(
                pedal_index,
                self.capabilities.pedal_count,
            ));
        }

        // Get configuration and trigger mode first
        let (config, trigger_mode) = {
            let configurations = self.configurations.lock()
                .map_err(|_| PedalError::Hid("Failed to lock configurations".to_string()))?;
            let trigger_modes = self.trigger_modes.lock()
                .map_err(|_| PedalError::Hid("Failed to lock trigger modes".to_string()))?;
            (configurations[pedal_index].clone(), trigger_modes[pedal_index])
        };

        let device = self.device.lock()
            .map_err(|_| PedalError::Hid("Failed to lock device".to_string()))?;

        // Start write sequence
        let start: [u8; 8] = [0x01, 0x80, 0x08, 0, 0, 0, 0, 0];
        Self::hid_write(&device, &start)?;
        thread::sleep(Duration::from_secs(1));

        // Write pedal header
        let header: [u8; 8] = [0x01, 0x81, 0x08, (pedal_index + 1) as u8, 0, 0, 0, 0];
        Self::hid_write(&device, &header)?;

        // Special handling for text configuration
        if let Configuration::Text(text) = &config {
            // Text configuration requires special multi-packet format
            let text_data = text.encode_for_protocol();

            // Count actual text bytes (non-zero)
            let text_len = text_data.iter().take_while(|&&b| b != 0).count();

            // First packet: length, type, and first 6 bytes of text
            let mut first_packet = [0u8; 8];
            first_packet[0] = (text_len + 2).min(40) as u8; // Length includes 2-byte header
            first_packet[1] = 0x04; // Text type

            let first_chunk_len = text_len.min(6);
            if first_chunk_len > 0 {
                first_packet[2..2 + first_chunk_len].copy_from_slice(&text_data[..first_chunk_len]);
            }
            Self::hid_write(&device, &first_packet)?;

            // Write remaining text in 8-byte packets
            let mut offset = 6;
            while offset < text_len {
                let mut packet = [0u8; 8];
                let chunk_len = (text_len - offset).min(8);
                packet[..chunk_len].copy_from_slice(&text_data[offset..offset + chunk_len]);
                Self::hid_write(&device, &packet)?;
                offset += 8;
            }
        } else {
            // Encode and write other configuration types
            let config_data = Self::encode_configuration(&config, trigger_mode);

            // Write in 8-byte chunks
            for chunk in config_data.chunks(8) {
                let mut packet = [0u8; 8];
                packet[..chunk.len()].copy_from_slice(chunk);
                Self::hid_write(&device, &packet)?;
            }
        }

        Ok(())
    }
}

impl PedalDevice for PCsensorDevice {
    fn id(&self) -> usize {
        self.id
    }

    fn model(&self) -> &str {
        match self.model {
            PCsensorModel::FootSwitch3Pedal => "PCsensor FootSwitch",
            PCsensorModel::FootSwitch1Pedal => "PCsensor FootSwitch (1P)",
        }
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn capabilities(&self) -> &DeviceCapabilities {
        &self.capabilities
    }

    fn load_configuration(&mut self) -> Result<()> {
        for i in 0..self.capabilities.pedal_count {
            self.read_pedal_config(i)?;
        }
        Ok(())
    }

    fn save_configuration(&mut self) -> Result<()> {
        // Write all three pedals (PCsensor protocol requires this)
        for i in 0..3 {
            if i < self.capabilities.pedal_count {
                self.write_pedal_config(i)?;
            } else {
                // Write empty config for non-existent pedals
                let device = self.device.lock()
                    .map_err(|_| PedalError::Hid("Failed to lock device".to_string()))?;
                let header: [u8; 8] = [0x01, 0x81, 0x08, (i + 1) as u8, 0, 0, 0, 0];
                Self::hid_write(&device, &header)?;
                let empty: [u8; 8] = [8, 0, 0, 0, 0, 0, 0, 0];
                Self::hid_write(&device, &empty)?;
            }
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

    fn set_pedal_configuration(
        &mut self,
        pedal_index: usize,
        config: Configuration,
    ) -> Result<()> {
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
