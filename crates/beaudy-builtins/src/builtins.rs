/*
 * @file builtins.rs
 *
 * @version 1.0.0
 * @author BleckWolf25
 * @license MIT
 *
 * @summary Built-in shell commands with screen-reader friendly formatting.
 *
 * @description
 * This module implements built-in shell commands including directory listing (bls)
 * and help documentation (bhelp), providing structured output in table format
 * optimized for accessibility and screen reader compatibility.
 *
 * @since 16/07/2026
 * @updated 16/07/2026
 */
// ---------- IMPORTS
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

// ---------- HELPER FUNCTIONS

fn format_size(bytes: u64, is_dir: bool) -> String {
    // Directories show as dash instead of size
    if is_dir {
        return "-".to_string();
    }
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

fn format_time(time: SystemTime) -> String {
    // Calculate relative time from file modification timestamp
    if let Ok(duration) = SystemTime::now().duration_since(time) {
        let secs = duration.as_secs();
        if secs < 60 {
            "just now".to_string()
        } else if secs < 3600 {
            format!("{}m ago", secs / 60)
        } else if secs < 86400 {
            format!("{}h ago", secs / 3600)
        } else {
            format!("{}d ago", secs / 86400)
        }
    } else {
        "unknown".to_string()
    }
}

// ---------- BUILTIN COMMANDS
/// Lists directory contents in a clean, screen-reader friendly table format.
pub fn run_bls(args: &[&str]) -> Result<i32, Box<dyn std::error::Error>> {
    // Default to current directory if no path provided
    let target_dir = if args.is_empty() {
        ".".to_string()
    } else {
        args[0].to_string()
    };

    let path = Path::new(&target_dir);
    if !path.exists() {
        eprintln!("bls: directory '{}' does not exist", target_dir);
        return Ok(1);
    }
    if !path.is_dir() {
        eprintln!("bls: '{}' is not a directory", target_dir);
        return Ok(1);
    }

    let entries = fs::read_dir(path)?;

    // Print Table Header
    println!(
        "{:<25} {:<8} {:<12} {:<15}",
        "Name", "Type", "Size", "Modified"
    );
    println!("{}", "-".repeat(65));

    let mut count = 0;
    for entry in entries {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let metadata = entry.metadata()?;
        let is_dir = metadata.is_dir();
        let file_type = if is_dir {
            "Dir"
        } else if metadata.file_type().is_symlink() {
            "Symlink"
        } else {
            "File"
        };
        let size_str = format_size(metadata.len(), is_dir);
        let mod_time = metadata.modified().unwrap_or(UNIX_EPOCH);
        let time_str = format_time(mod_time);

        println!(
            "{:<25} {:<8} {:<12} {:<15}",
            name, file_type, size_str, time_str
        );
        count += 1;
    }

    println!("\r\nTotal entries: {}", count);
    Ok(0)
}

/// Prints shell guide and builtin documentation.
pub fn run_bhelp() -> Result<i32, Box<dyn std::error::Error>> {
    println!("=== BeaudyShell Help Guide ===");
    println!("An Accessible, Modern, and Elegant Shell Frontend.\r\n");
    println!("Available Builtin Commands:");
    println!("  bls [path]   - Lists directory contents in a structured table.");
    println!("  bhelp        - Displays this help screen.");
    println!("  exit         - Exits the shell.\r\n");
    println!("Keyboard Shortcuts:");
    println!("  Tab          - Cycles through autocomplete suggestions.");
    println!("  Right Arrow  - Accepts the active inline auto-suggestion.");
    println!("  Up/Down Arrow- Navigates the command history.");
    println!("  Ctrl+C       - Exits the shell.\r\n");
    println!("Accessibility Info:");
    println!("  BeaudyShell outputs OSC 133 semantic markers to assist screen readers");
    println!("  in jumping between command prompt, input, and output zones.");
    Ok(0)
}
