//! Command-line interface definition

use anyhow::Result;
use clap::{Parser, Subcommand};

/// USB HID pedal device configuration tool
#[derive(Parser, Debug)]
#[command(name = "clutchctl")]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Disable colored output
    #[arg(long = "no-color", global = true)]
    pub no_color: bool,

    /// Command to execute
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// List all connected pedal devices
    List,

    /// Show configuration of a device
    Show {
        /// Device ID (from list command)
        device: usize,
    },

    /// Set pedal configuration
    Set {
        /// Device ID
        device: usize,

        /// Pedal to configure (name or index)
        pedal: String,

        /// Configuration subcommand
        #[command(subcommand)]
        config: SetConfig,
    },
}

#[derive(Subcommand, Debug)]
pub enum SetConfig {
    /// Configure keyboard input
    Keyboard {
        /// Key combination (e.g., "ctrl+c", "f1")
        keys: String,

        /// One-shot mode (key press only once)
        #[arg(long)]
        once: bool,

        /// Trigger on release instead of press
        #[arg(long)]
        invert: bool,
    },

    /// Configure mouse input
    Mouse {
        /// Mouse configuration arguments
        #[command(subcommand)]
        mode: MouseMode,

        /// Trigger on release instead of press
        #[arg(long)]
        invert: bool,
    },

    /// Configure text input
    Text {
        /// Text to type
        text: String,

        /// Trigger on release instead of press
        #[arg(long)]
        invert: bool,
    },

    /// Configure media control
    Media {
        /// Media button (e.g., "play", "volume-up", "mute")
        button: String,

        /// Trigger on release instead of press
        #[arg(long)]
        invert: bool,
    },

    /// Configure gamepad input
    Game {
        /// Game button (e.g., "up", "button1")
        button: String,

        /// Trigger on release instead of press
        #[arg(long)]
        invert: bool,
    },

    /// Unconfigure pedal
    None,
}

#[derive(Subcommand, Debug)]
pub enum MouseMode {
    /// Mouse buttons
    Buttons {
        /// Button combination (e.g., "left", "left+right")
        buttons: String,
    },

    /// Mouse axis movement
    Axis {
        /// X movement (-100 to 100)
        x: i8,

        /// Y movement (-100 to 100)
        y: i8,

        /// Wheel movement (-100 to 100)
        #[arg(default_value = "0")]
        wheel: i8,
    },
}

impl Cli {
    /// Execute the CLI command
    pub fn execute(self) -> Result<()> {
        // Set log level based on verbose flag
        if self.verbose {
            log::set_max_level(log::LevelFilter::Debug);
        }

        match self.command {
            Command::List => crate::commands::list::execute(),
            Command::Show { device } => crate::commands::show::execute(device),
            Command::Set { device, pedal, config } => {
                crate::commands::set::execute(device, pedal, config)
            }
        }
    }
}