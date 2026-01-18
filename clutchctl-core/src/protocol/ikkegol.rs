//! iKKEGOL protocol encoding and decoding

use crate::configuration::{
    Configuration, GamepadConfiguration, KeyboardConfiguration, MediaConfiguration,
    MouseConfiguration, TextConfiguration,
    keyboard::KeyMode,
    mouse::{MouseButton, MouseMode},
};
use crate::error::{PedalError, Result};
use crate::protocol::{
    ConfigPacket, ConfigType, GameKey, KeyboardData, MediaButton, MediaData,
    ModifierKeys, MouseData, ProtocolMouseButton, HID_KEYMAP,
};
use std::collections::HashSet;

/// Parse a configuration packet into a Configuration
pub fn parse_config(packet: &ConfigPacket) -> Result<Configuration> {
    match packet.get_config_type() {
        Some(ConfigType::Unconfigured) => Ok(Configuration::Unconfigured),

        Some(ConfigType::Keyboard) | Some(ConfigType::KeyboardOnce) |
        Some(ConfigType::KeyboardMulti) | Some(ConfigType::KeyboardMultiOnce) => {
            let data = packet.parse_data();
            if let crate::protocol::ConfigData::Keyboard(kbd) = data {
                let mode = match packet.get_config_type() {
                    Some(ConfigType::KeyboardOnce) | Some(ConfigType::KeyboardMultiOnce) => {
                        KeyMode::OneShot
                    }
                    _ => KeyMode::Standard,
                };

                // Parse keys (non-zero scan codes)
                let mut keys = Vec::new();
                for &scan_code in &kbd.keys {
                    if scan_code != 0 {
                        // Try to decode scan code to key name using HID keymap
                        if let Some(key_name) = HID_KEYMAP.decode_key(scan_code) {
                            keys.push(key_name.to_string());
                        } else {
                            // Fall back to hex representation for unknown codes
                            keys.push(format!("0x{:02x}", scan_code));
                        }
                    }
                }

                let modifiers = ModifierKeys::from_bits_truncate(kbd.modifiers);
                Ok(Configuration::Keyboard(
                    KeyboardConfiguration::with_modifiers(mode, keys, modifiers)
                ))
            } else {
                Err(PedalError::Protocol("Invalid keyboard data".to_string()))
            }
        }

        Some(ConfigType::Mouse) => {
            let data = packet.parse_data();
            if let crate::protocol::ConfigData::Mouse(mouse) = data {
                if mouse.buttons != 0 {
                    // Button mode
                    let mut buttons = HashSet::new();
                    let proto_buttons = ProtocolMouseButton::from_bits_truncate(mouse.buttons);

                    if proto_buttons.contains(ProtocolMouseButton::LEFT) {
                        buttons.insert(MouseButton::Left);
                    }
                    if proto_buttons.contains(ProtocolMouseButton::RIGHT) {
                        buttons.insert(MouseButton::Right);
                    }
                    if proto_buttons.contains(ProtocolMouseButton::MIDDLE) {
                        buttons.insert(MouseButton::Middle);
                    }
                    if proto_buttons.contains(ProtocolMouseButton::BACK) {
                        buttons.insert(MouseButton::Back);
                    }
                    if proto_buttons.contains(ProtocolMouseButton::FORWARD) {
                        buttons.insert(MouseButton::Forward);
                    }

                    Ok(Configuration::Mouse(MouseConfiguration::buttons(buttons)))
                } else {
                    // Axis mode
                    Ok(Configuration::Mouse(MouseConfiguration::axis(
                        mouse.mouse_x,
                        mouse.mouse_y,
                        mouse.mouse_wheel,
                    )))
                }
            } else {
                Err(PedalError::Protocol("Invalid mouse data".to_string()))
            }
        }

        Some(ConfigType::Text) => {
            let data = packet.parse_data();
            if let crate::protocol::ConfigData::Text(text) = data {
                let text_str = TextConfiguration::decode_from_protocol(&text.string);
                Ok(Configuration::Text(TextConfiguration::new(text_str)))
            } else {
                Err(PedalError::Protocol("Invalid text data".to_string()))
            }
        }

        Some(ConfigType::Media) => {
            let data = packet.parse_data();
            if let crate::protocol::ConfigData::Media(media) = data {
                if let Some(button) = MediaButton::from_u8(media.key) {
                    Ok(Configuration::Media(MediaConfiguration::new(button)))
                } else {
                    Err(PedalError::Protocol(format!("Unknown media button: {}", media.key)))
                }
            } else {
                Err(PedalError::Protocol("Invalid media data".to_string()))
            }
        }

        Some(ConfigType::Game) => {
            let data = packet.parse_data();
            if let crate::protocol::ConfigData::Game(game) = data {
                if let Some(key) = GameKey::from_u8(game.key) {
                    Ok(Configuration::Gamepad(GamepadConfiguration::new(key)))
                } else {
                    Err(PedalError::Protocol(format!("Unknown game key: {}", game.key)))
                }
            } else {
                Err(PedalError::Protocol("Invalid game data".to_string()))
            }
        }

        None => Err(PedalError::Protocol(format!("Unknown config type: {}", packet.config_type))),
    }
}

/// Encode a Configuration into a ConfigPacket
pub fn encode_config(config: &Configuration) -> Result<ConfigPacket> {
    let mut packet = ConfigPacket::unconfigured();

    match config {
        Configuration::Unconfigured => {
            packet.config_type = ConfigType::Unconfigured as u8;
            packet.size = 0;
        }

        Configuration::Keyboard(kbd) => {
            // Determine config type based on mode and key count
            let key_count = kbd.keys.len();
            packet.config_type = if key_count > 1 {
                if kbd.mode == KeyMode::OneShot {
                    ConfigType::KeyboardMultiOnce as u8
                } else {
                    ConfigType::KeyboardMulti as u8
                }
            } else {
                if kbd.mode == KeyMode::OneShot {
                    ConfigType::KeyboardOnce as u8
                } else {
                    ConfigType::Keyboard as u8
                }
            };

            // Encode keyboard data
            let mut kbd_data = KeyboardData {
                modifiers: kbd.modifiers.bits(),
                keys: [0; 6],
            };

            // Convert key names to scan codes
            for (i, key) in kbd.keys.iter().enumerate() {
                if i >= 6 {
                    break;
                }

                // First try hex scan codes for backward compatibility
                if let Some(hex) = key.strip_prefix("0x") {
                    if let Ok(code) = u8::from_str_radix(hex, 16) {
                        kbd_data.keys[i] = code;
                        continue;
                    }
                }

                // Try to encode key name using HID keymap
                if let Some(code) = HID_KEYMAP.encode_key(key) {
                    kbd_data.keys[i] = code;
                }
            }

            // Copy keyboard data to packet
            unsafe {
                let kbd_bytes = std::slice::from_raw_parts(
                    &kbd_data as *const _ as *const u8,
                    std::mem::size_of::<KeyboardData>(),
                );
                packet.data[..kbd_bytes.len()].copy_from_slice(kbd_bytes);
            }

            packet.size = 40; // Full packet size
        }

        Configuration::Mouse(mouse) => {
            packet.config_type = ConfigType::Mouse as u8;

            let mut mouse_data = MouseData {
                unknown: [0, 0],
                buttons: 0,
                mouse_x: 0,
                mouse_y: 0,
                mouse_wheel: 0,
            };

            match &mouse.mode {
                MouseMode::Buttons(buttons) => {
                    let mut proto_buttons = ProtocolMouseButton::empty();
                    for button in buttons {
                        match button {
                            MouseButton::Left => proto_buttons |= ProtocolMouseButton::LEFT,
                            MouseButton::Right => proto_buttons |= ProtocolMouseButton::RIGHT,
                            MouseButton::Middle => proto_buttons |= ProtocolMouseButton::MIDDLE,
                            MouseButton::Back => proto_buttons |= ProtocolMouseButton::BACK,
                            MouseButton::Forward => proto_buttons |= ProtocolMouseButton::FORWARD,
                        }
                    }
                    mouse_data.buttons = proto_buttons.bits();
                }
                MouseMode::Axis { x, y, wheel } => {
                    mouse_data.mouse_x = *x;
                    mouse_data.mouse_y = *y;
                    mouse_data.mouse_wheel = *wheel;
                }
            }

            // Copy mouse data to packet
            unsafe {
                let mouse_bytes = std::slice::from_raw_parts(
                    &mouse_data as *const _ as *const u8,
                    std::mem::size_of::<MouseData>(),
                );
                packet.data[..mouse_bytes.len()].copy_from_slice(mouse_bytes);
            }

            packet.size = 40;
        }

        Configuration::Text(text) => {
            packet.config_type = ConfigType::Text as u8;

            let encoded = text.encode_for_protocol();
            packet.data[..38].copy_from_slice(&encoded[..38]);

            packet.size = 40;
        }

        Configuration::Media(media) => {
            packet.config_type = ConfigType::Media as u8;

            let media_data = MediaData {
                key: media.button as u8,
            };

            packet.data[0] = media_data.key;
            packet.size = 40;
        }

        Configuration::Gamepad(gamepad) => {
            packet.config_type = ConfigType::Game as u8;

            packet.data[0] = gamepad.button as u8;
            packet.size = 40;
        }
    }

    Ok(packet)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unconfigured_round_trip() {
        let config = Configuration::Unconfigured;
        let packet = encode_config(&config).unwrap();
        let parsed = parse_config(&packet).unwrap();

        assert!(matches!(parsed, Configuration::Unconfigured));
    }

    #[test]
    fn test_media_round_trip() {
        let config = Configuration::Media(MediaConfiguration::new(MediaButton::Play));
        let packet = encode_config(&config).unwrap();
        let parsed = parse_config(&packet).unwrap();

        if let Configuration::Media(media) = parsed {
            assert_eq!(media.button, MediaButton::Play);
        } else {
            panic!("Expected media configuration");
        }
    }

    #[test]
    fn test_gamepad_round_trip() {
        let config = Configuration::Gamepad(GamepadConfiguration::new(GameKey::Button1));
        let packet = encode_config(&config).unwrap();
        let parsed = parse_config(&packet).unwrap();

        if let Configuration::Gamepad(gamepad) = parsed {
            assert_eq!(gamepad.button, GameKey::Button1);
        } else {
            panic!("Expected gamepad configuration");
        }
    }
}