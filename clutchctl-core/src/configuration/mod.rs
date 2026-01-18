//! Configuration types for pedal devices

pub mod keyboard;
pub mod mouse;
pub mod text;
pub mod media;
pub mod gamepad;

pub use keyboard::KeyboardConfiguration;
pub use mouse::MouseConfiguration;
pub use text::TextConfiguration;
pub use media::MediaConfiguration;
pub use gamepad::GamepadConfiguration;

use crate::protocol::TriggerMode;

/// Configuration type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigurationType {
    Keyboard,
    Mouse,
    Text,
    Media,
    Gamepad,
}

/// Trigger type for pedal activation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    OnPress,
    OnRelease,
}

impl From<TriggerMode> for Trigger {
    fn from(mode: TriggerMode) -> Self {
        match mode {
            TriggerMode::Press => Trigger::OnPress,
            TriggerMode::Release => Trigger::OnRelease,
        }
    }
}

impl From<Trigger> for TriggerMode {
    fn from(trigger: Trigger) -> Self {
        match trigger {
            Trigger::OnPress => TriggerMode::Press,
            Trigger::OnRelease => TriggerMode::Release,
        }
    }
}

/// Base configuration trait
pub trait BaseConfiguration {
    /// Get the configuration type
    fn configuration_type(&self) -> ConfigurationType;

    /// Get the trigger mode
    fn trigger(&self) -> Trigger;

    /// Set the trigger mode
    fn set_trigger(&mut self, trigger: Trigger);

    /// Convert to a human-readable string representation
    fn to_string(&self) -> String;
}

/// Main configuration enum that holds all possible configurations
#[derive(Debug, Clone)]
pub enum Configuration {
    Keyboard(KeyboardConfiguration),
    Mouse(MouseConfiguration),
    Text(TextConfiguration),
    Media(MediaConfiguration),
    Gamepad(GamepadConfiguration),
    Unconfigured,
}

impl Configuration {
    /// Check if the configuration is unconfigured
    pub fn is_unconfigured(&self) -> bool {
        matches!(self, Configuration::Unconfigured)
    }

    /// Get the configuration type
    pub fn configuration_type(&self) -> Option<ConfigurationType> {
        match self {
            Configuration::Keyboard(_) => Some(ConfigurationType::Keyboard),
            Configuration::Mouse(_) => Some(ConfigurationType::Mouse),
            Configuration::Text(_) => Some(ConfigurationType::Text),
            Configuration::Media(_) => Some(ConfigurationType::Media),
            Configuration::Gamepad(_) => Some(ConfigurationType::Gamepad),
            Configuration::Unconfigured => None,
        }
    }

    /// Get the trigger mode
    pub fn trigger(&self) -> Option<Trigger> {
        match self {
            Configuration::Keyboard(c) => Some(c.trigger()),
            Configuration::Mouse(c) => Some(c.trigger()),
            Configuration::Text(c) => Some(c.trigger()),
            Configuration::Media(c) => Some(c.trigger()),
            Configuration::Gamepad(c) => Some(c.trigger()),
            Configuration::Unconfigured => None,
        }
    }

    /// Set the trigger mode
    pub fn set_trigger(&mut self, trigger: Trigger) {
        match self {
            Configuration::Keyboard(c) => c.set_trigger(trigger),
            Configuration::Mouse(c) => c.set_trigger(trigger),
            Configuration::Text(c) => c.set_trigger(trigger),
            Configuration::Media(c) => c.set_trigger(trigger),
            Configuration::Gamepad(c) => c.set_trigger(trigger),
            Configuration::Unconfigured => {}
        }
    }
}

impl std::fmt::Display for Configuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Configuration::Keyboard(c) => write!(f, "{}", c.to_string()),
            Configuration::Mouse(c) => write!(f, "{}", c.to_string()),
            Configuration::Text(c) => write!(f, "{}", c.to_string()),
            Configuration::Media(c) => write!(f, "{}", c.to_string()),
            Configuration::Gamepad(c) => write!(f, "{}", c.to_string()),
            Configuration::Unconfigured => write!(f, "Unconfigured"),
        }
    }
}