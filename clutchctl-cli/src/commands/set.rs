//! Set command implementation

use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use clutchctl_core::configuration::{
    Configuration, GamepadConfiguration, KeyboardConfiguration, MediaConfiguration,
    MouseConfiguration, TextConfiguration, Trigger, BaseConfiguration,
    keyboard::KeyMode,
};
use clutchctl_core::device::discover_devices;

use crate::cli::{MouseMode, SetConfig};

/// Execute the set command
pub fn execute(device_id: usize, pedal_str: String, config: SetConfig) -> Result<()> {
    // Find the device
    let devices = discover_devices()
        .context("Failed to discover USB devices")?;

    let device = devices
        .into_iter()
        .find(|d| d.id() == device_id)
        .ok_or_else(|| anyhow!("Device with ID {} not found", device_id))?;

    // Get mutable device reference
    let mut device = device;
    let device_mut = std::sync::Arc::get_mut(&mut device)
        .ok_or_else(|| anyhow!("Failed to get mutable device reference"))?;

    // Load current configuration
    device_mut.load_configuration()
        .context("Failed to load device configuration")?;

    // Parse pedal index (get capabilities, parse, then drop the borrow)
    let (pedal_index, pedal_name) = {
        let capabilities = device_mut.capabilities();

        let pedal_index = if let Ok(num) = pedal_str.parse::<usize>() {
            // 1-based index from user
            if num == 0 || num > capabilities.pedal_count {
                return Err(anyhow!(
                    "Invalid pedal index {}. Device has {} pedal(s)",
                    num,
                    capabilities.pedal_count
                ));
            }
            num - 1 // Convert to 0-based
        } else {
            // Try to find by name
            capabilities.find_pedal_by_name(&pedal_str)
                .ok_or_else(|| {
                    let names = capabilities.pedal_names.join(", ");
                    anyhow!(
                        "Unknown pedal '{}'. Available pedals: {}",
                        pedal_str,
                        names
                    )
                })?
        };

        let pedal_name = capabilities.get_pedal_name(pedal_index)
            .unwrap_or(&format!("pedal{}", pedal_index + 1))
            .to_string();

        (pedal_index, pedal_name)
    };

    // Create configuration based on the command
    let new_config = match config {
        SetConfig::None => Configuration::Unconfigured,

        SetConfig::Keyboard { keys, once, invert } => {
            let mode = if once { KeyMode::OneShot } else { KeyMode::Standard };
            let (modifiers, main_key) = KeyboardConfiguration::parse_modifiers(&keys);

            let key_list = if let Some(key) = main_key {
                vec![key]
            } else {
                return Err(anyhow!("No main key specified"));
            };

            let mut kbd_config = KeyboardConfiguration::with_modifiers(mode, key_list, modifiers);
            if invert {
                kbd_config.set_trigger(Trigger::OnRelease);
            }
            Configuration::Keyboard(kbd_config)
        }

        SetConfig::Mouse { mode, invert } => {
            let mut mouse_config = match mode {
                MouseMode::Buttons { buttons } => {
                    let button_set = MouseConfiguration::parse_buttons(&buttons)
                        .ok_or_else(|| anyhow!("Invalid mouse button: {}", buttons))?;
                    MouseConfiguration::buttons(button_set)
                }
                MouseMode::Axis { x, y, wheel } => {
                    MouseConfiguration::axis(x, y, wheel)
                }
            };

            if invert {
                mouse_config.set_trigger(Trigger::OnRelease);
            }
            Configuration::Mouse(mouse_config)
        }

        SetConfig::Text { text, invert } => {
            if text.len() > 38 {
                return Err(anyhow!("Text too long (max 38 characters)"));
            }
            let mut text_config = TextConfiguration::new(text);
            if invert {
                text_config.set_trigger(Trigger::OnRelease);
            }
            Configuration::Text(text_config)
        }

        SetConfig::Media { button, invert } => {
            let media_button = MediaConfiguration::parse_button(&button)
                .ok_or_else(|| anyhow!("Unknown media button: {}", button))?;
            let mut media_config = MediaConfiguration::new(media_button);
            if invert {
                media_config.set_trigger(Trigger::OnRelease);
            }
            Configuration::Media(media_config)
        }

        SetConfig::Game { button, invert } => {
            let game_button = GamepadConfiguration::parse_button(&button)
                .ok_or_else(|| anyhow!("Unknown game button: {}", button))?;
            let mut game_config = GamepadConfiguration::new(game_button);
            if invert {
                game_config.set_trigger(Trigger::OnRelease);
            }
            Configuration::Gamepad(game_config)
        }
    };

    // Set the configuration
    device_mut.set_pedal_configuration(pedal_index, new_config.clone())
        .context("Failed to set pedal configuration")?;

    // Save to device
    device_mut.save_configuration()
        .context("Failed to save configuration to device")?;

    // Display success message
    println!("\n{} Configuration updated for {} {} on device {}",
             "✓".green().bold(),
             pedal_name.yellow().bold(),
             format!("[{}]", pedal_index + 1).cyan(),
             format!("[{}]", device_id).cyan().bold());

    match &new_config {
        Configuration::Unconfigured => {
            println!("  Set to: {}", "Unconfigured".red());
        }
        config => {
            println!("  Set to: {}", config.to_string().green());
        }
    }

    Ok(())
}