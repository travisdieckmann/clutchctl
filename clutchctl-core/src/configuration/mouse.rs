//! Mouse configuration type

use super::{BaseConfiguration, ConfigurationType, Trigger};
use std::collections::HashSet;

/// Mouse button types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Forward,
    Back,
}

impl MouseButton {
    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "left" => Some(Self::Left),
            "right" => Some(Self::Right),
            "middle" => Some(Self::Middle),
            "forward" => Some(Self::Forward),
            "back" => Some(Self::Back),
            _ => None,
        }
    }

    /// Convert to display string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::Middle => "middle",
            Self::Forward => "forward",
            Self::Back => "back",
        }
    }
}

/// Mouse configuration mode
#[derive(Debug, Clone, PartialEq)]
pub enum MouseMode {
    /// Mouse button clicks
    Buttons(HashSet<MouseButton>),
    /// Mouse axis movement
    Axis {
        x: i8,
        y: i8,
        wheel: i8,
    },
}

/// Mouse configuration
#[derive(Debug, Clone)]
pub struct MouseConfiguration {
    /// Mouse mode
    pub mode: MouseMode,
    /// Trigger mode
    trigger: Trigger,
}

impl MouseConfiguration {
    /// Create a new button configuration
    pub fn buttons(buttons: HashSet<MouseButton>) -> Self {
        Self {
            mode: MouseMode::Buttons(buttons),
            trigger: Trigger::OnPress,
        }
    }

    /// Create a new axis configuration
    pub fn axis(x: i8, y: i8, wheel: i8) -> Self {
        Self {
            mode: MouseMode::Axis { x, y, wheel },
            trigger: Trigger::OnPress,
        }
    }

    /// Parse button string (e.g., "left+right")
    pub fn parse_buttons(s: &str) -> Option<HashSet<MouseButton>> {
        let mut buttons = HashSet::new();
        for part in s.split('+') {
            buttons.insert(MouseButton::from_str(part)?);
        }
        Some(buttons)
    }

    /// Format for display
    pub fn format(&self) -> String {
        match &self.mode {
            MouseMode::Buttons(buttons) => {
                let mut button_strs: Vec<_> = buttons.iter()
                    .map(|b| b.as_str())
                    .collect();
                button_strs.sort();
                button_strs.join("+")
            }
            MouseMode::Axis { x, y, wheel } => {
                if *wheel != 0 {
                    format!("axis({}, {}, {})", x, y, wheel)
                } else {
                    format!("axis({}, {})", x, y)
                }
            }
        }
    }
}

impl BaseConfiguration for MouseConfiguration {
    fn configuration_type(&self) -> ConfigurationType {
        ConfigurationType::Mouse
    }

    fn trigger(&self) -> Trigger {
        self.trigger
    }

    fn set_trigger(&mut self, trigger: Trigger) {
        self.trigger = trigger;
    }

    fn to_string(&self) -> String {
        format!("Mouse: {}", self.format())
    }
}