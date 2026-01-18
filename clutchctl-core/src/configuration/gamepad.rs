//! Gamepad configuration type

use super::{BaseConfiguration, ConfigurationType, Trigger};
use crate::protocol::GameKey;

/// Gamepad configuration
#[derive(Debug, Clone)]
pub struct GamepadConfiguration {
    /// Game button
    pub button: GameKey,
    /// Trigger mode
    trigger: Trigger,
}

impl GamepadConfiguration {
    /// Create a new gamepad configuration
    pub fn new(button: GameKey) -> Self {
        Self {
            button,
            trigger: Trigger::OnPress,
        }
    }

    /// Parse game button from string
    pub fn parse_button(s: &str) -> Option<GameKey> {
        match s.to_lowercase().as_str() {
            "left" | "dpad-left" => Some(GameKey::Left),
            "right" | "dpad-right" => Some(GameKey::Right),
            "up" | "dpad-up" => Some(GameKey::Up),
            "down" | "dpad-down" => Some(GameKey::Down),
            "button1" | "button-1" | "1" => Some(GameKey::Button1),
            "button2" | "button-2" | "2" => Some(GameKey::Button2),
            "button3" | "button-3" | "3" => Some(GameKey::Button3),
            "button4" | "button-4" | "4" => Some(GameKey::Button4),
            "button5" | "button-5" | "5" => Some(GameKey::Button5),
            "button6" | "button-6" | "6" => Some(GameKey::Button6),
            "button7" | "button-7" | "7" => Some(GameKey::Button7),
            "button8" | "button-8" | "8" => Some(GameKey::Button8),
            _ => None,
        }
    }

    /// Get display name for game button
    pub fn button_name(&self) -> &'static str {
        match self.button {
            GameKey::Left => "D-Pad Left",
            GameKey::Right => "D-Pad Right",
            GameKey::Up => "D-Pad Up",
            GameKey::Down => "D-Pad Down",
            GameKey::Button1 => "Button 1",
            GameKey::Button2 => "Button 2",
            GameKey::Button3 => "Button 3",
            GameKey::Button4 => "Button 4",
            GameKey::Button5 => "Button 5",
            GameKey::Button6 => "Button 6",
            GameKey::Button7 => "Button 7",
            GameKey::Button8 => "Button 8",
        }
    }
}

impl BaseConfiguration for GamepadConfiguration {
    fn configuration_type(&self) -> ConfigurationType {
        ConfigurationType::Gamepad
    }

    fn trigger(&self) -> Trigger {
        self.trigger
    }

    fn set_trigger(&mut self, trigger: Trigger) {
        self.trigger = trigger;
    }

    fn to_string(&self) -> String {
        format!("Gamepad: {}", self.button_name())
    }
}