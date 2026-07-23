/*
 * @file config.rs
 *
 * @version 1.0.0
 * @author BleckWolf25
 * @license MIT
 *
 * @summary Shell configuration management and prompt styling.
 *
 * @description
 * This module handles loading and managing shell configuration from ~/.beaudy.toml,
 * providing prompt style and color customization with sensible defaults and
 * automatic configuration file generation.
 *
 * @since 16/07/2026
 * @updated 23/07/2026
 */
// ---------- IMPORTS
use crossterm::style::Color;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// ---------- CONFIGURATION STRUCTURE
#[derive(Clone)]
pub struct Config {
    pub prompt_style: String,
    pub prompt_color: String,
    pub default_shell: String,
    pub aliases: HashMap<String, String>,
}

// ---------- CONFIGURATION IMPLEMENTATION
impl Default for Config {
    fn default() -> Self {
        let default_shell = if cfg!(windows) {
            "powershell.exe".to_string()
        } else {
            "sh".to_string()
        };
        Config {
            prompt_style: "path".to_string(),
            prompt_color: "cyan".to_string(),
            default_shell,
            aliases: HashMap::new(),
        }
    }
}

impl Config {
    // Convert color string to crossterm Color enum
    pub fn get_crossterm_color(&self) -> Color {
        match self.prompt_color.to_lowercase().as_str() {
            "green" => Color::Green,
            "blue" => Color::Blue,
            "magenta" => Color::Magenta,
            "yellow" => Color::Yellow,
            "white" => Color::White,
            "red" => Color::Red,
            _ => Color::Cyan,
        }
    }
}

// ---------- CONFIGURATION LOADING
pub fn load_config() -> Config {
    // Determine home directory based on operating system
    let home_dir = if cfg!(windows) {
        std::env::var("USERPROFILE").ok()
    } else {
        std::env::var("HOME").ok()
    };

    let Some(home) = home_dir else {
        return Config::default();
    };

    let config_path = PathBuf::from(home).join(".beaudy.toml");

    if !config_path.exists() {
        // Write default configuration file
        let default_toml = r#"# BeaudyShell Configuration File
# Settings will take effect upon restarting the shell or running built-in commands.

# Style of the prompt:
#   "path"    - Displays the current path with home shorthand (e.g. ~/projects)
#   "compact" - Displays only the current folder name (e.g. BeaudyShell)
#   "static"  - Displays a static "beaudy>" prompt
prompt_style = "path"

# Color of the prompt:
#   Available: "cyan", "green", "blue", "magenta", "yellow", "white", "red"
prompt_color = "cyan"

# Default subshell for external commands (e.g., "pwsh", "powershell.exe", "bash", "zsh", "sh")
default_shell = "powershell.exe"

[aliases]
# Custom command aliases:
# ll = "bls"
"#;
        let _ = fs::write(&config_path, default_toml);
        return Config::default();
    }

    let mut config = Config::default();
    if let Ok(content) = fs::read_to_string(&config_path) {
        let mut current_section = "";
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                current_section = &line[1..line.len() - 1];
                continue;
            }
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() == 2 {
                let key = parts[0].trim();
                let val = parts[1].trim().trim_matches('"').trim_matches('\'').trim();
                if current_section == "aliases" {
                    config.aliases.insert(key.to_string(), val.to_string());
                } else {
                    match key {
                        "prompt_style" => config.prompt_style = val.to_string(),
                        "prompt_color" => config.prompt_color = val.to_string(),
                        "default_shell" => config.default_shell = val.to_string(),
                        _ => {}
                    }
                }
            }
        }
    }

    config
}

pub fn save_alias(name: &str, command: &str) {
    let home_dir = if cfg!(windows) {
        std::env::var("USERPROFILE").ok()
    } else {
        std::env::var("HOME").ok()
    };
    if let Some(home) = home_dir {
        let config_path = PathBuf::from(home).join(".beaudy.toml");
        if let Ok(content) = fs::read_to_string(&config_path) {
            let mut new_lines = Vec::new();
            let mut in_aliases = false;
            let mut found = false;
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('[') && trimmed.ends_with(']') {
                    in_aliases = trimmed == "[aliases]";
                }
                if in_aliases && trimmed.starts_with(name) && trimmed.contains('=') {
                    new_lines.push(format!("{} = \"{}\"", name, command));
                    found = true;
                } else {
                    new_lines.push(line.to_string());
                }
            }
            if !found {
                if !content.contains("[aliases]") {
                    new_lines.push("\n[aliases]".to_string());
                }
                new_lines.push(format!("{} = \"{}\"", name, command));
            }
            let _ = fs::write(&config_path, new_lines.join("\n"));
        }
    }
}

pub fn remove_alias(name: &str) {
    let home_dir = if cfg!(windows) {
        std::env::var("USERPROFILE").ok()
    } else {
        std::env::var("HOME").ok()
    };
    if let Some(home) = home_dir {
        let config_path = PathBuf::from(home).join(".beaudy.toml");
        if let Ok(content) = fs::read_to_string(&config_path) {
            let mut new_lines = Vec::new();
            let mut in_aliases = false;
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('[') && trimmed.ends_with(']') {
                    in_aliases = trimmed == "[aliases]";
                }
                if in_aliases && trimmed.starts_with(name) && trimmed.contains('=') {
                    continue;
                }
                new_lines.push(line.to_string());
            }
            let _ = fs::write(&config_path, new_lines.join("\n"));
        }
    }
}
