//! Media control configuration type

use super::{BaseConfiguration, ConfigurationType, Trigger};
use crate::protocol::MediaButton;

/// Media configuration
#[derive(Debug, Clone)]
pub struct MediaConfiguration {
    /// Media button
    pub button: MediaButton,
    /// Trigger mode
    trigger: Trigger,
}

impl MediaConfiguration {
    /// Create a new media configuration
    pub fn new(button: MediaButton) -> Self {
        Self {
            button,
            trigger: Trigger::OnPress,
        }
    }

    /// Parse media button from string
    pub fn parse_button(s: &str) -> Option<MediaButton> {
        match s.to_lowercase().replace('_', "-").as_str() {
            "volume-down" | "volume-minus" => Some(MediaButton::VolumeMinus),
            "volume-up" | "volume-plus" => Some(MediaButton::VolumePlus),
            "mute" => Some(MediaButton::Mute),
            "play" | "play-pause" => Some(MediaButton::Play),
            "forward" | "fast-forward" => Some(MediaButton::Forward),
            "next" | "skip" => Some(MediaButton::Next),
            "stop" => Some(MediaButton::Stop),
            "open-player" | "player" => Some(MediaButton::OpenPlayer),
            "open-homepage" | "homepage" | "home" => Some(MediaButton::OpenHomepage),
            "stop-webpage" | "stop-page" => Some(MediaButton::StopWebpage),
            "back-browse" | "browser-back" => Some(MediaButton::BackBrowse),
            "forward-browse" | "browser-forward" => Some(MediaButton::ForwardBrowse),
            "refresh" | "reload" => Some(MediaButton::Refresh),
            "open-my-computer" | "my-computer" | "computer" => Some(MediaButton::OpenMyComputer),
            "open-mail" | "mail" | "email" => Some(MediaButton::OpenMail),
            "open-calc" | "calculator" | "calc" => Some(MediaButton::OpenCalc),
            "open-search" | "search" => Some(MediaButton::OpenSearch),
            "shutdown" | "power-off" => Some(MediaButton::Shutdown),
            "sleep" | "suspend" => Some(MediaButton::Sleep),
            _ => None,
        }
    }

    /// Get display name for media button
    pub fn button_name(&self) -> &'static str {
        match self.button {
            MediaButton::VolumeMinus => "Volume Down",
            MediaButton::VolumePlus => "Volume Up",
            MediaButton::Mute => "Mute",
            MediaButton::Play => "Play/Pause",
            MediaButton::Forward => "Fast Forward",
            MediaButton::Next => "Next Track",
            MediaButton::Stop => "Stop",
            MediaButton::OpenPlayer => "Open Player",
            MediaButton::OpenHomepage => "Open Homepage",
            MediaButton::StopWebpage => "Stop Webpage",
            MediaButton::BackBrowse => "Browser Back",
            MediaButton::ForwardBrowse => "Browser Forward",
            MediaButton::Refresh => "Refresh",
            MediaButton::OpenMyComputer => "Open My Computer",
            MediaButton::OpenMail => "Open Mail",
            MediaButton::OpenCalc => "Open Calculator",
            MediaButton::OpenSearch => "Open Search",
            MediaButton::Shutdown => "Shutdown",
            MediaButton::Sleep => "Sleep",
        }
    }
}

impl BaseConfiguration for MediaConfiguration {
    fn configuration_type(&self) -> ConfigurationType {
        ConfigurationType::Media
    }

    fn trigger(&self) -> Trigger {
        self.trigger
    }

    fn set_trigger(&mut self, trigger: Trigger) {
        self.trigger = trigger;
    }

    fn to_string(&self) -> String {
        format!("Media: {}", self.button_name())
    }
}