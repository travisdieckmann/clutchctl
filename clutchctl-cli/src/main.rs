//! clutchctl command-line interface

mod cli;
mod commands;

use anyhow::Result;
use clap::Parser;
use env_logger::Env;

fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();

    // Parse command-line arguments
    let cli = cli::Cli::parse();

    // Configure colored output based on platform and user preference
    configure_colored_output(cli.no_color);

    // Execute the command
    cli.execute()
}

/// Configure colored output based on the platform and terminal capabilities
fn configure_colored_output(no_color: bool) {
    use colored::control;

    // Disable colors if requested via flag or env var
    if no_color || std::env::var("NO_COLOR").is_ok() {
        control::set_override(false);
        return;
    }

    #[cfg(windows)]
    {
        // On Windows, try to enable virtual terminal processing for ANSI codes
        // This enables colors in Windows Terminal, modern PowerShell, and Windows 10+ Command Prompt
        if control::set_virtual_terminal(true).is_err() {
            // If we can't enable ANSI support, check if we're in a known good terminal
            // Otherwise disable colors to avoid garbled output
            let term = std::env::var("TERM").unwrap_or_default();
            let wt_session = std::env::var("WT_SESSION").is_ok(); // Windows Terminal
            let vscode = std::env::var("TERM_PROGRAM").unwrap_or_default() == "vscode";

            if !wt_session && !vscode && !term.contains("xterm") && !term.contains("color") {
                // Disable colors if we can't determine terminal support
                control::set_override(false);
            }
        }
    }

    // On Unix-like systems, colors usually work by default
    // The colored crate handles this automatically
}