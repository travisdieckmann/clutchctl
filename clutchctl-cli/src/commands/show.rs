//! Show command implementation

use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use clutchctl_core::configuration::{Configuration, Trigger};
use clutchctl_core::device::discover_devices;

/// Execute the show command
pub fn execute(device_id: usize) -> Result<()> {
    // Find the device
    let devices = discover_devices()
        .context("Failed to discover USB devices")?;

    let device = devices
        .into_iter()
        .find(|d| d.id() == device_id)
        .ok_or_else(|| anyhow!("Device with ID {} not found", device_id))?;

    // Load configuration
    let mut device = device;
    {
        let device_mut = std::sync::Arc::get_mut(&mut device)
            .ok_or_else(|| anyhow!("Failed to get mutable device reference"))?;
        device_mut.load_configuration()
            .context("Failed to load device configuration")?;
    }

    // Display device information
    println!("\n{} {} {}",
             "Device".bold(),
             format!("[{}]", device_id).cyan().bold(),
             device.model().green());
    println!("Version: {}", device.version());
    println!();

    let capabilities = device.capabilities();
    println!("Pedals: {}\n", capabilities.pedal_count);

    // Display each pedal configuration
    for i in 0..capabilities.pedal_count {
        let default_name = format!("pedal{}", i + 1);
        let pedal_name = capabilities.get_pedal_name(i)
            .unwrap_or(&default_name);

        let config = device.get_pedal_configuration(i)
            .context("Failed to get pedal configuration")?;

        print!("  {} {} ",
               format!("[{}]", i + 1).cyan(),
               pedal_name.yellow().bold());

        // Display trigger mode
        if let Some(trigger) = config.trigger() {
            let trigger_str = match trigger {
                Trigger::OnPress => "(on press)",
                Trigger::OnRelease => "(on release)",
            };
            print!("{} ", trigger_str.dimmed());
        }

        // Display configuration
        match &config {
            Configuration::Unconfigured => {
                println!("{}", "Unconfigured".red());
            }
            config => {
                println!("{}", config.to_string().green());
            }
        }
    }

    println!("\n{}",
             "Use 'clutchctl set <ID> <PEDAL> <CONFIG>' to change configuration.".dimmed());

    Ok(())
}