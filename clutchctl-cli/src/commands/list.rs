//! List command implementation

use anyhow::{Context, Result};
use colored::Colorize;
use clutchctl_core::device::discover_devices;

/// Execute the list command
pub fn execute() -> Result<()> {
    println!("Discovering USB pedal devices...\n");

    let devices = discover_devices()
        .context("Failed to discover USB devices. Try running with sudo if you see permission errors.")?;

    if devices.is_empty() {
        println!("{}", "No pedal devices found.".yellow());
        println!("\nMake sure your device is connected and you have the necessary permissions.");
        println!("On Linux, you may need to install udev rules or run with sudo.");
        return Ok(());
    }

    println!("Found {} device(s):\n", devices.len());

    for device in devices {
        let id = device.id();
        let model = device.model();
        let version = device.version();
        let capabilities = device.capabilities();

        println!("  {} {}", format!("[{}]", id).cyan().bold(), model.green());
        println!("      Version:  {}", version);
        println!("      Pedals:   {}", capabilities.pedal_count);

        if !capabilities.pedal_names.is_empty() {
            let names = capabilities.pedal_names.join(", ");
            println!("      Names:    {}", names);
        }

        println!();
    }

    println!("{}", "Use 'clutchctl show <ID>' to see device configuration.".dimmed());

    Ok(())
}