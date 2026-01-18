//! USB HID keymap for converting between key names and scan codes
//!
//! Based on the USB HID Usage Tables specification
//! See: http://www.freebsddiary.org/APC/usb_hid_usages.php

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Static keymap table with all HID key mappings
static KEYMAP_TABLE: &[(&str, u8)] = &[
    // Control codes
    ("<00>", 0x00),
    ("<01>", 0x01),
    ("<02>", 0x02),
    ("<03>", 0x03),

    // Lowercase letters a-z (0x04-0x1d)
    ("a", 0x04),
    ("b", 0x05),
    ("c", 0x06),
    ("d", 0x07),
    ("e", 0x08),
    ("f", 0x09),
    ("g", 0x0a),
    ("h", 0x0b),
    ("i", 0x0c),
    ("j", 0x0d),
    ("k", 0x0e),
    ("l", 0x0f),
    ("m", 0x10),
    ("n", 0x11),
    ("o", 0x12),
    ("p", 0x13),
    ("q", 0x14),
    ("r", 0x15),
    ("s", 0x16),
    ("t", 0x17),
    ("u", 0x18),
    ("v", 0x19),
    ("w", 0x1a),
    ("x", 0x1b),
    ("y", 0x1c),
    ("z", 0x1d),

    // Numbers 1-9, 0 (0x1e-0x27)
    ("1", 0x1e),
    ("2", 0x1f),
    ("3", 0x20),
    ("4", 0x21),
    ("5", 0x22),
    ("6", 0x23),
    ("7", 0x24),
    ("8", 0x25),
    ("9", 0x26),
    ("0", 0x27),

    // Common keys
    ("enter", 0x28),
    ("return", 0x28),
    ("esc", 0x29),
    ("escape", 0x29),
    ("backspace", 0x2a),
    ("tab", 0x2b),
    (" ", 0x2c),
    ("space", 0x2c),

    // Symbols
    ("-", 0x2d),
    ("=", 0x2e),
    ("[", 0x2f),
    ("]", 0x30),
    ("\\", 0x31),
    (";", 0x33),
    ("'", 0x34),
    ("`", 0x35),
    (",", 0x36),
    (".", 0x37),
    ("/", 0x38),

    // Lock keys
    ("capslock", 0x39),

    // Function keys F1-F12 (0x3a-0x45)
    ("f1", 0x3a),
    ("f2", 0x3b),
    ("f3", 0x3c),
    ("f4", 0x3d),
    ("f5", 0x3e),
    ("f6", 0x3f),
    ("f7", 0x40),
    ("f8", 0x41),
    ("f9", 0x42),
    ("f10", 0x43),
    ("f11", 0x44),
    ("f12", 0x45),

    // System keys
    ("printscreen", 0x46),
    ("scrolllock", 0x47),
    ("pause", 0x48),
    ("insert", 0x49),
    ("home", 0x4a),
    ("pageup", 0x4b),
    ("prior", 0x4b),
    ("delete", 0x4c),
    ("end", 0x4d),
    ("pagedown", 0x4e),
    ("next", 0x4e),

    // Arrow keys
    ("right", 0x4f),
    ("left", 0x50),
    ("down", 0x51),
    ("up", 0x52),

    // Numpad
    ("numlock", 0x53),
    ("kp_divide", 0x54),
    ("kp_multiply", 0x55),
    ("kp_subtract", 0x56),
    ("kp_add", 0x57),
    ("kp_enter", 0x58),
    ("kp_end", 0x59),
    ("kp_down", 0x5a),
    ("kp_next", 0x5b),
    ("kp_left", 0x5c),
    ("kp_begin", 0x5d),
    ("kp_right", 0x5e),
    ("kp_home", 0x5f),
    ("kp_up", 0x60),
    ("kp_prior", 0x61),
    ("kp_insert", 0x62),
    ("kp_delete", 0x63),

    // International keys
    ("less", 0x64),
    ("multi_key", 0x65),
    ("compose", 0x65),

    // Extended function keys F13-F24 (0x68-0x73)
    ("f13", 0x68),
    ("f14", 0x69),
    ("f15", 0x6a),
    ("f16", 0x6b),
    ("f17", 0x6c),
    ("f18", 0x6d),
    ("f19", 0x6e),
    ("f20", 0x6f),
    ("f21", 0x70),
    ("f22", 0x71),
    ("f23", 0x72),
    ("f24", 0x73),

    // Media keys
    ("xf86audiomute", 0x7f),
    ("xf86audioraisevolume", 0x80),
    ("xf86audiolowervolume", 0x81),

    // Uppercase letters A-Z (0x84-0x9d) - shifted versions
    ("A", 0x84),
    ("B", 0x85),
    ("C", 0x86),
    ("D", 0x87),
    ("E", 0x88),
    ("F", 0x89),
    ("G", 0x8a),
    ("H", 0x8b),
    ("I", 0x8c),
    ("J", 0x8d),
    ("K", 0x8e),
    ("L", 0x8f),
    ("M", 0x90),
    ("N", 0x91),
    ("O", 0x92),
    ("P", 0x93),
    ("Q", 0x94),
    ("R", 0x95),
    ("S", 0x96),
    ("T", 0x97),
    ("U", 0x98),
    ("V", 0x99),
    ("W", 0x9a),
    ("X", 0x9b),
    ("Y", 0x9c),
    ("Z", 0x9d),

    // Shifted symbols (0x9e-0xb8)
    ("!", 0x9e),
    ("@", 0x9f),
    ("#", 0xa0),
    ("$", 0xa1),
    ("%", 0xa2),
    ("^", 0xa3),
    ("&", 0xa4),
    ("*", 0xa5),
    ("(", 0xa6),
    (")", 0xa7),
    ("_", 0xad),
    ("+", 0xae),
    ("{", 0xaf),
    ("}", 0xb0),
    ("|", 0xb1),
    (":", 0xb3),
    ("\"", 0xb4),
    ("~", 0xb5),
    ("<", 0xb6),
    (">", 0xb7),
    ("?", 0xb8),

    // Modifier keys (0xe0-0xe7)
    ("control_l", 0xe0),
    ("ctrl_l", 0xe0),
    ("lctrl", 0xe0),
    ("shift_l", 0xe1),
    ("lshift", 0xe1),
    ("alt_l", 0xe2),
    ("lalt", 0xe2),
    ("super_l", 0xe3),
    ("lsuper", 0xe3),
    ("lwin", 0xe3),
    ("lcmd", 0xe3),
    ("control_r", 0xe4),
    ("ctrl_r", 0xe4),
    ("rctrl", 0xe4),
    ("shift_r", 0xe5),
    ("rshift", 0xe5),
    ("meta_r", 0xe6),
    ("alt_r", 0xe6),
    ("ralt", 0xe6),
    ("super_r", 0xe7),
    ("rsuper", 0xe7),
    ("rwin", 0xe7),
    ("rcmd", 0xe7),

    // Additional media keys
    ("xf86audiopause", 0xe8),
    ("xf86eject", 0xe9),
    ("xf86audioprev", 0xea),
    ("xf86audionext", 0xeb),
    ("xf86www", 0xf0),
    ("xf86back", 0xf1),
    ("xf86forward", 0xf2),
    ("xf86sleep", 0xf8),
    ("xf86screensaver", 0xf9),
    ("xf86reload", 0xfa),
    ("xf86calculator", 0xfb),
];

/// Lazy-initialized lookup maps for efficient key name <-> scan code conversion
pub static HID_KEYMAP: Lazy<HidKeymap> = Lazy::new(|| HidKeymap::new());

/// HID keymap for bidirectional key name <-> scan code conversion
pub struct HidKeymap {
    /// Map from key name (lowercase) to scan code
    name_to_code: HashMap<String, u8>,
    /// Map from scan code to key name
    code_to_name: HashMap<u8, &'static str>,
}

impl HidKeymap {
    /// Create a new keymap from the static table
    fn new() -> Self {
        let mut name_to_code = HashMap::new();
        let mut code_to_name = HashMap::new();

        for &(name, code) in KEYMAP_TABLE {
            // Store both exact name and lowercase version
            // This allows case-sensitive lookup for letters
            // and case-insensitive lookup for other keys

            // Skip storing lowercase version if it would conflict with single letters
            // (we want 'a' to map to 0x04, not be overwritten by 'A' lowercased)
            let is_single_letter = name.len() == 1 && name.chars().next().unwrap().is_alphabetic();

            if !is_single_letter {
                // Store lowercase version for case-insensitive lookup
                name_to_code.insert(name.to_lowercase(), code);
            }

            // Always store exact name
            name_to_code.insert(name.to_string(), code);

            // For code-to-name, prefer shorter/simpler names
            // Only insert if not already present or if this name is shorter
            code_to_name.entry(code)
                .and_modify(|existing: &mut &'static str| {
                    if name.len() < existing.len() && !name.starts_with('<') {
                        *existing = name;
                    }
                })
                .or_insert(name);
        }

        Self {
            name_to_code,
            code_to_name,
        }
    }

    /// Convert a key name to its USB HID scan code
    ///
    /// # Arguments
    /// * `name` - Key name (case-sensitive for letters, case-insensitive for others)
    ///
    /// # Returns
    /// * `Some(code)` - The USB HID scan code if found
    /// * `None` - If the key name is not recognized
    ///
    /// # Examples
    /// ```
    /// assert_eq!(HID_KEYMAP.encode_key("a"), Some(0x04));
    /// assert_eq!(HID_KEYMAP.encode_key("A"), Some(0x84));
    /// assert_eq!(HID_KEYMAP.encode_key("F5"), Some(0x3e));
    /// assert_eq!(HID_KEYMAP.encode_key("f5"), Some(0x3e));
    /// assert_eq!(HID_KEYMAP.encode_key("enter"), Some(0x28));
    /// ```
    pub fn encode_key(&self, name: &str) -> Option<u8> {
        // First try exact match (for case-sensitive letters)
        if let Some(&code) = self.name_to_code.get(name) {
            return Some(code);
        }
        // Then try case-insensitive match (for function keys, special keys, etc.)
        self.name_to_code.get(&name.to_lowercase()).copied()
    }

    /// Convert a USB HID scan code to its key name
    ///
    /// # Arguments
    /// * `code` - USB HID scan code
    ///
    /// # Returns
    /// * `Some(name)` - The key name if found
    /// * `None` - If the scan code is not recognized
    ///
    /// # Examples
    /// ```
    /// assert_eq!(HID_KEYMAP.decode_key(0x04), Some("a"));
    /// assert_eq!(HID_KEYMAP.decode_key(0x3e), Some("f5"));
    /// assert_eq!(HID_KEYMAP.decode_key(0x28), Some("enter"));
    /// ```
    pub fn decode_key(&self, code: u8) -> Option<&str> {
        self.code_to_name.get(&code).copied()
    }

    /// Convert a character to its USB HID scan code
    ///
    /// This is useful for encoding text strings where each character
    /// needs to be converted to its HID scan code.
    ///
    /// # Arguments
    /// * `ch` - Character to encode
    ///
    /// # Returns
    /// * `Some(code)` - The USB HID scan code if found
    /// * `None` - If the character cannot be encoded
    ///
    /// # Examples
    /// ```
    /// assert_eq!(HID_KEYMAP.encode_char('a'), Some(0x04));
    /// assert_eq!(HID_KEYMAP.encode_char('A'), Some(0x84));
    /// assert_eq!(HID_KEYMAP.encode_char(' '), Some(0x2c));
    /// assert_eq!(HID_KEYMAP.encode_char('!'), Some(0x9e));
    /// ```
    pub fn encode_char(&self, ch: char) -> Option<u8> {
        let key = ch.to_string();
        self.name_to_code.get(&key.to_lowercase()).copied()
    }

    /// Check if a character requires shift modifier
    ///
    /// # Arguments
    /// * `ch` - Character to check
    ///
    /// # Returns
    /// * `true` if the character requires shift modifier
    /// * `false` otherwise
    pub fn requires_shift(&self, ch: char) -> bool {
        matches!(self.encode_char(ch), Some(code) if code >= 0x84)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_key() {
        let keymap = &*HID_KEYMAP;

        // Lowercase letters
        assert_eq!(keymap.encode_key("a"), Some(0x04));
        assert_eq!(keymap.encode_key("z"), Some(0x1d));

        // Case insensitive
        assert_eq!(keymap.encode_key("A"), Some(0x84));
        assert_eq!(keymap.encode_key("F5"), Some(0x3e));
        assert_eq!(keymap.encode_key("f5"), Some(0x3e));

        // Numbers
        assert_eq!(keymap.encode_key("1"), Some(0x1e));
        assert_eq!(keymap.encode_key("0"), Some(0x27));

        // Special keys
        assert_eq!(keymap.encode_key("enter"), Some(0x28));
        assert_eq!(keymap.encode_key("space"), Some(0x2c));
        assert_eq!(keymap.encode_key("tab"), Some(0x2b));

        // Function keys
        assert_eq!(keymap.encode_key("f1"), Some(0x3a));
        assert_eq!(keymap.encode_key("f12"), Some(0x45));

        // Unknown key
        assert_eq!(keymap.encode_key("unknown"), None);
    }

    #[test]
    fn test_decode_key() {
        let keymap = &*HID_KEYMAP;

        assert_eq!(keymap.decode_key(0x04), Some("a"));
        assert_eq!(keymap.decode_key(0x1d), Some("z"));
        assert_eq!(keymap.decode_key(0x28), Some("enter"));
        assert_eq!(keymap.decode_key(0x3e), Some("f5"));

        // Non-existent code
        assert_eq!(keymap.decode_key(0xff), None);
    }

    #[test]
    fn test_encode_char() {
        let keymap = &*HID_KEYMAP;

        assert_eq!(keymap.encode_char('a'), Some(0x04));
        assert_eq!(keymap.encode_char('A'), Some(0x84));
        assert_eq!(keymap.encode_char(' '), Some(0x2c));
        assert_eq!(keymap.encode_char('!'), Some(0x9e));
        assert_eq!(keymap.encode_char('@'), Some(0x9f));
    }

    #[test]
    fn test_round_trip() {
        let keymap = &*HID_KEYMAP;

        // Test that encode -> decode gives back a valid key name
        for key in ["a", "f5", "enter", "space", "tab"] {
            if let Some(code) = keymap.encode_key(key) {
                assert!(keymap.decode_key(code).is_some());
            }
        }
    }
}