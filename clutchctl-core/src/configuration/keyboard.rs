//! Keyboard configuration type

use super::{BaseConfiguration, ConfigurationType, Trigger};
use crate::protocol::ModifierKeys;

/// Keyboard activation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyMode {
    /// Standard mode - key press and release
    Standard,
    /// One-shot mode - single key press
    OneShot,
}

/// Keyboard configuration
#[derive(Debug, Clone)]
pub struct KeyboardConfiguration {
    /// Activation mode
    pub mode: KeyMode,
    /// Keys to press (USB scan codes or key names)
    pub keys: Vec<String>,
    /// Modifier keys
    pub modifiers: ModifierKeys,
    /// Trigger mode
    trigger: Trigger,
}

impl KeyboardConfiguration {
    /// Create a new keyboard configuration
    pub fn new(mode: KeyMode, keys: Vec<String>) -> Self {
        Self {
            mode,
            keys,
            modifiers: ModifierKeys::empty(),
            trigger: Trigger::OnPress,
        }
    }

    /// Create with modifiers
    pub fn with_modifiers(mode: KeyMode, keys: Vec<String>, modifiers: ModifierKeys) -> Self {
        Self {
            mode,
            keys,
            modifiers,
            trigger: Trigger::OnPress,
        }
    }

    /// Parse modifier keys from a key string
    pub fn parse_modifiers(key: &str) -> (ModifierKeys, Option<String>) {
        let mut modifiers = ModifierKeys::empty();
        let parts: Vec<&str> = key.split('+').collect();

        if parts.len() == 1 {
            return (modifiers, Some(key.to_string()));
        }

        let mut main_key = None;
        for part in parts {
            match part.to_lowercase().as_str() {
                "lcontrol" | "lctrl" => modifiers |= ModifierKeys::LEFT_CONTROL,
                "rcontrol" | "rctrl" => modifiers |= ModifierKeys::RIGHT_CONTROL,
                "control" | "ctrl" => modifiers |= ModifierKeys::LEFT_CONTROL,
                "lshift" => modifiers |= ModifierKeys::LEFT_SHIFT,
                "rshift" => modifiers |= ModifierKeys::RIGHT_SHIFT,
                "shift" => modifiers |= ModifierKeys::LEFT_SHIFT,
                "lalt" => modifiers |= ModifierKeys::LEFT_ALT,
                "ralt" => modifiers |= ModifierKeys::RIGHT_ALT,
                "alt" => modifiers |= ModifierKeys::LEFT_ALT,
                "lsuper" | "lwin" | "lcmd" => modifiers |= ModifierKeys::LEFT_SUPER,
                "rsuper" | "rwin" | "rcmd" => modifiers |= ModifierKeys::RIGHT_SUPER,
                "super" | "win" | "cmd" => modifiers |= ModifierKeys::LEFT_SUPER,
                _ => main_key = Some(part.to_string()),
            }
        }

        (modifiers, main_key)
    }

    /// Format modifiers and keys for display
    pub fn format_keys(&self) -> String {
        let mut parts = Vec::new();

        // Add modifiers
        if self.modifiers.contains(ModifierKeys::LEFT_CONTROL) {
            parts.push("LCtrl");
        }
        if self.modifiers.contains(ModifierKeys::RIGHT_CONTROL) {
            parts.push("RCtrl");
        }
        if self.modifiers.contains(ModifierKeys::LEFT_SHIFT) {
            parts.push("LShift");
        }
        if self.modifiers.contains(ModifierKeys::RIGHT_SHIFT) {
            parts.push("RShift");
        }
        if self.modifiers.contains(ModifierKeys::LEFT_ALT) {
            parts.push("LAlt");
        }
        if self.modifiers.contains(ModifierKeys::RIGHT_ALT) {
            parts.push("RAlt");
        }
        if self.modifiers.contains(ModifierKeys::LEFT_SUPER) {
            parts.push("LSuper");
        }
        if self.modifiers.contains(ModifierKeys::RIGHT_SUPER) {
            parts.push("RSuper");
        }

        // Add main keys
        for key in &self.keys {
            parts.push(key);
        }

        parts.join("+")
    }
}

impl BaseConfiguration for KeyboardConfiguration {
    fn configuration_type(&self) -> ConfigurationType {
        ConfigurationType::Keyboard
    }

    fn trigger(&self) -> Trigger {
        self.trigger
    }

    fn set_trigger(&mut self, trigger: Trigger) {
        self.trigger = trigger;
    }

    fn to_string(&self) -> String {
        let mode_str = match self.mode {
            KeyMode::Standard => "Keyboard",
            KeyMode::OneShot => "Keyboard (One-shot)",
        };
        format!("{}: {}", mode_str, self.format_keys())
    }
}