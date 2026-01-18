//! Text configuration type

use super::{BaseConfiguration, ConfigurationType, Trigger};
use crate::protocol::HID_KEYMAP;

/// Text configuration - types a string when pedal is activated
#[derive(Debug, Clone)]
pub struct TextConfiguration {
    /// Text to type
    pub text: String,
    /// Trigger mode
    trigger: Trigger,
}

impl TextConfiguration {
    /// Create a new text configuration
    pub fn new(text: String) -> Self {
        Self {
            text,
            trigger: Trigger::OnPress,
        }
    }

    /// Get the text with characters encoded as USB HID scan codes
    pub fn encode_for_protocol(&self) -> Vec<u8> {
        let mut encoded = Vec::new();

        for ch in self.text.chars() {
            // Convert character to HID scan code
            if let Some(code) = HID_KEYMAP.encode_char(ch) {
                encoded.push(code);
            } else if ch == ' ' {
                // Space character
                encoded.push(0x2c);
            } else {
                // Skip unsupported characters
                continue;
            }

            if encoded.len() >= 38 {
                break; // Maximum text length
            }
        }

        // Pad with zeros
        while encoded.len() < 38 {
            encoded.push(0);
        }

        encoded
    }

    /// Decode text from HID scan code format
    pub fn decode_from_protocol(data: &[u8; 38]) -> String {
        let mut text = String::new();

        for &byte in data {
            if byte == 0 {
                break; // Null terminator
            }

            // Try to decode HID scan code to character
            if let Some(key_name) = HID_KEYMAP.decode_key(byte) {
                // Handle special cases
                if key_name == "space" {
                    text.push(' ');
                } else if key_name.len() == 1 {
                    // Single character key
                    text.push_str(key_name);
                } else {
                    // Special key - represent as <key>
                    text.push('<');
                    text.push_str(key_name);
                    text.push('>');
                }
            } else {
                // Unknown scan code - represent as hex
                text.push_str(&format!("<0x{:02x}>", byte));
            }
        }

        text
    }
}

impl BaseConfiguration for TextConfiguration {
    fn configuration_type(&self) -> ConfigurationType {
        ConfigurationType::Text
    }

    fn trigger(&self) -> Trigger {
        self.trigger
    }

    fn set_trigger(&mut self, trigger: Trigger) {
        self.trigger = trigger;
    }

    fn to_string(&self) -> String {
        format!("Text: \"{}\"", self.text)
    }
}