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
 * @updated 23/07/2026
 */
// ---------- IMPORTS
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

// ---------- HELPER FUNCTIONS
pub(crate) fn format_size(bytes: u64, is_dir: bool) -> String {
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

pub(crate) fn format_time(time: SystemTime) -> String {
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

pub(crate) fn format_datetime(time: SystemTime) -> String {
    if let Ok(duration) = time.duration_since(UNIX_EPOCH) {
        let secs = duration.as_secs();
        let days = secs / 86400;
        let day_secs = secs % 86400;
        let hours = day_secs / 3600;
        let mins = (day_secs % 3600) / 60;

        let z = days as i64 + 719_468;
        let era = (if z >= 0 { z } else { z - 146_096 }) / 146_097;
        let doe = (z - era * 146_097) as u64;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
        let y = yoe as i64 + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let year = if m <= 2 { y + 1 } else { y };
        format!("{:04}-{:02}-{:02} {:02}:{:02}", year, m, d, hours, mins)
    } else {
        "unknown".to_string()
    }
}

// ---------- BUILTIN COMMANDS
/// Lists directory contents in a clean, screen-reader friendly table format.
///
/// ```
/// use beaudy_builtins::run_bls;
/// let res = run_bls(&["."]);
/// assert_eq!(res.unwrap(), 0);
/// ```
pub fn run_bls(args: &[&str]) -> Result<i32, Box<dyn std::error::Error>> {
    let mut show_all = false;
    let mut recursive = false;
    let mut paths = Vec::new();

    for arg in args {
        match *arg {
            "-a" | "--all" => show_all = true,
            "-R" | "--recursive" => recursive = true,
            _ => paths.push(*arg),
        }
    }

    let target_dir = if paths.is_empty() {
        ".".to_string()
    } else {
        paths[0].to_string()
    };

    render_bls_directory(Path::new(&target_dir), show_all, recursive)
}

fn render_bls_directory(
    path: &Path,
    show_all: bool,
    recursive: bool,
) -> Result<i32, Box<dyn std::error::Error>> {
    if !path.exists() {
        eprintln!("bls: directory '{}' does not exist", path.display());
        return Ok(1);
    }
    if !path.is_dir() {
        eprintln!("bls: '{}' is not a directory", path.display());
        return Ok(1);
    }

    if recursive {
        println!("\r\n{}:", path.display());
    }

    let entries = fs::read_dir(path)?;
    let mut dir_entries = Vec::new();
    let mut max_name_len = 4; // "Name"
    let mut subdirs = Vec::new();

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !show_all && name.starts_with('.') {
            continue;
        }
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        if is_dir {
            subdirs.push(entry.path());
        }
        max_name_len = max_name_len.max(name.len());
        dir_entries.push((entry, name));
    }

    let name_width = max_name_len.clamp(25, 50);

    // Print Table Header
    println!(
        "{:<width$} {:<8} {:<12} {:<15}",
        "Name",
        "Type",
        "Size",
        "Modified",
        width = name_width
    );
    println!("{}", "-".repeat(name_width + 8 + 12 + 15 + 3));

    let mut count = 0;
    for (entry, name) in dir_entries {
        let metadata = entry.metadata()?;
        let is_dir = metadata.is_dir();
        let is_symlink = metadata.file_type().is_symlink();

        let (file_type, color_code) = if is_dir {
            ("Dir", "\x1b[36m") // Cyan
        } else if is_symlink {
            ("Symlink", "\x1b[35m") // Magenta
        } else {
            let is_exe = if cfg!(windows) {
                entry
                    .path()
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|ext| matches!(ext.to_lowercase().as_str(), "exe" | "cmd" | "bat"))
                    .unwrap_or(false)
            } else {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    (metadata.permissions().mode() & 0o111) != 0
                }
                #[cfg(not(unix))]
                {
                    false
                }
            };
            if is_exe {
                ("Exe", "\x1b[32m") // Green
            } else {
                ("File", "\x1b[37m") // White
            }
        };

        let size_str = format_size(metadata.len(), is_dir);
        let mod_time = metadata.modified().unwrap_or(UNIX_EPOCH);
        let time_str = format_time(mod_time);

        let display_name = if name.len() > name_width {
            format!("{}...", &name[..name_width - 3])
        } else {
            name
        };

        let colored_name = format!("{}{}\x1b[0m", color_code, display_name);
        println!(
            "{:<width$} {:<8} {:<12} {:<15}",
            colored_name,
            file_type,
            size_str,
            time_str,
            width = name_width + color_code.len() + 4
        );
        count += 1;
    }

    println!("\r\nTotal entries: {}", count);

    if recursive {
        for subdir in subdirs {
            let _ = render_bls_directory(&subdir, show_all, true);
        }
    }

    Ok(0)
}

/// Prints shell guide and builtin documentation.
///
/// ```
/// use beaudy_builtins::run_bhelp;
/// let res = run_bhelp();
/// assert_eq!(res.unwrap(), 0);
/// ```
pub fn run_bhelp() -> Result<i32, Box<dyn std::error::Error>> {
    println!("=== BeaudyShell Help Guide ===");
    println!("An Accessible, Modern, and Elegant Shell Frontend.\r\n");
    println!("Available Builtin Commands:");
    println!("  bls [path]   - Lists directory contents in a structured table.");
    println!("  bconfig      - Manage shell configuration.");
    println!("  bsetup       - View setup instructions.");
    println!("  pwd          - Print current working directory.");
    println!("  bmemo <text> - Quick scratchpad note-taker.");
    println!("  bcalc <expr> - Evaluate math expressions.");
    println!("  btrash <file>- Safe delete.");
    println!("  bhash <text> - Generate MD5/SHA256 hashes.");
    println!("  export [K=V] - Set or view environment variables.");
    println!("  alias [K=V]  - Set or view command aliases.");
    println!("  unalias <K>  - Remove a command alias.");
    println!("  clear / cls  - Clears the terminal screen.");
    println!("  bhelp        - Displays this help screen.");
    println!("  exit         - Exits the shell.\r\n");
    println!("Keyboard Shortcuts:");
    println!("  Tab          - Cycles through autocomplete suggestions.");
    println!("  Right Arrow  - Accepts active inline auto-suggestion.");
    println!("  Up/Down Arrow- Navigates command history.");
    println!("  Ctrl+R       - Interactive reverse history search.");
    println!("  Ctrl+C       - Clears active prompt line.");
    println!("  Ctrl+D       - Exits the shell (on empty prompt).\r\n");
    println!("Accessibility Info:");
    println!("  BeaudyShell outputs OSC 133 semantic markers to assist screen readers");
    println!("  in jumping between command prompt, input, and output zones.");
    Ok(0)
}

/// Configuration utility
pub fn run_bconfig(args: &[&str]) -> Result<i32, Box<dyn std::error::Error>> {
    let home_dir = if cfg!(windows) {
        std::env::var("USERPROFILE").ok()
    } else {
        std::env::var("HOME").ok()
    };

    let Some(home) = home_dir else {
        eprintln!("bconfig: Could not determine home directory");
        return Ok(1);
    };

    let config_path = std::path::PathBuf::from(home).join(".beaudy.toml");

    if args.is_empty() {
        if !config_path.exists() {
            println!("Configuration file does not exist. It will be created on next restart.");
            return Ok(0);
        }

        println!("\r\n=== BeaudyShell Interactive Settings ===");
        println!("1. Change Prompt Style (path, compact, static)");
        println!("2. Change Prompt Color (cyan, green, blue, magenta, yellow, white, red)");
        println!("3. Exit Menu");

        print!("Select an option: ");
        let _ = std::io::stdout().flush();

        #[allow(clippy::collapsible_if)]
        let choice = loop {
            if let crossterm::event::Event::Key(key_event) = crossterm::event::read()? {
                if key_event.kind == crossterm::event::KeyEventKind::Press {
                    match key_event.code {
                        crossterm::event::KeyCode::Char('1') => break "1",
                        crossterm::event::KeyCode::Char('2') => break "2",
                        crossterm::event::KeyCode::Char('3') | crossterm::event::KeyCode::Esc => {
                            break "3";
                        }
                        _ => {}
                    }
                }
            }
        };
        println!("{}", choice);

        if choice == "3" {
            return Ok(0);
        }

        let (key, valid_values) = match choice {
            "1" => ("prompt_style", vec!["path", "compact", "static"]),
            "2" => (
                "prompt_color",
                vec!["cyan", "green", "blue", "magenta", "yellow", "white", "red"],
            ),
            _ => unreachable!(),
        };

        println!("\r\nAvailable values for {}:", key);
        for (i, val) in valid_values.iter().enumerate() {
            println!("{}. {}", i + 1, val);
        }
        print!("Select a value (1-{}): ", valid_values.len());
        let _ = std::io::stdout().flush();

        #[allow(clippy::collapsible_if)]
        let new_value = loop {
            if let crossterm::event::Event::Key(key_event) = crossterm::event::read()? {
                if key_event.kind == crossterm::event::KeyEventKind::Press {
                    if let crossterm::event::KeyCode::Char(c) = key_event.code {
                        if let Some(digit) = c.to_digit(10) {
                            let idx = (digit as usize).saturating_sub(1);
                            if idx < valid_values.len() {
                                break valid_values[idx];
                            }
                        }
                    }
                }
            }
        };
        println!("{}", new_value);

        // Update config file
        let content = fs::read_to_string(&config_path)?;
        let mut found = false;
        let mut new_content = String::new();

        for line in content.lines() {
            if line.trim().starts_with(key) && line.contains('=') {
                new_content.push_str(&format!("{} = \"{}\"\n", key, new_value));
                found = true;
            } else {
                new_content.push_str(line);
                new_content.push('\n');
            }
        }

        if !found {
            new_content.push_str(&format!("{} = \"{}\"\n", key, new_value));
        }

        fs::write(&config_path, new_content)?;
        println!("Configuration updated successfully.");
        return Ok(0);
    }

    if args[0] == "set" && args.len() == 3 {
        let key = args[1];
        let value = args[2];

        let content = if config_path.exists() {
            fs::read_to_string(&config_path)?
        } else {
            String::new()
        };

        let mut found = false;
        let mut new_content = String::new();

        for line in content.lines() {
            if line.trim().starts_with(key) && line.contains('=') {
                new_content.push_str(&format!("{} = \"{}\"\n", key, value));
                found = true;
            } else {
                new_content.push_str(line);
                new_content.push('\n');
            }
        }

        if !found {
            new_content.push_str(&format!("{} = \"{}\"\n", key, value));
        }

        fs::write(&config_path, new_content)?;
        println!("Configuration updated successfully.");
        return Ok(0);
    }

    println!("Usage:");
    println!("  bconfig                    - View configuration");
    println!("  bconfig set <key> <value>  - Update a configuration value");
    Ok(1)
}

/// Print working directory
///
/// ```
/// use beaudy_builtins::run_pwd;
/// let res = run_pwd();
/// assert_eq!(res.unwrap(), 0);
/// ```
pub fn run_pwd() -> Result<i32, Box<dyn std::error::Error>> {
    if let Ok(dir) = std::env::current_dir() {
        println!("{}", dir.display());
        Ok(0)
    } else {
        eprintln!("pwd: Could not get current directory");
        Ok(1)
    }
}

/// Setup utility to help set BeaudyShell as default
///
/// ```
/// use beaudy_builtins::run_bsetup;
/// let res = run_bsetup();
/// assert_eq!(res.unwrap(), 0);
/// ```
pub fn run_bsetup() -> Result<i32, Box<dyn std::error::Error>> {
    println!("=== BeaudyShell Setup Instructions ===");

    if cfg!(windows) {
        println!(
            "On Windows, you can set BeaudyShell as the default terminal profile in Windows Terminal:"
        );
        println!("1. Open Windows Terminal settings.");
        println!("2. Add a new profile with the command line pointing to beaudy-shell.exe.");
        println!("3. Set it as your default profile.");
    } else {
        println!(
            "To set BeaudyShell as your default shell on Linux/macOS, run the following commands:"
        );
        println!("  echo \"$(which beaudy-shell)\" | sudo tee -a /etc/shells");
        println!("  chsh -s \"$(which beaudy-shell)\"");
    }

    println!(
        "\r\nTo add it to your PATH, ensure the executable is located in a directory listed in your PATH environment variable, such as ~/.local/bin or /usr/local/bin."
    );
    Ok(0)
}

/// Save a note to the local memo pad or view notes
pub fn run_bmemo(args: &[&str]) -> Result<i32, Box<dyn std::error::Error>> {
    let home_dir = if cfg!(windows) {
        std::env::var("USERPROFILE").ok()
    } else {
        std::env::var("HOME").ok()
    };

    let Some(home) = home_dir else {
        eprintln!("bmemo: Could not determine home directory");
        return Ok(1);
    };

    let memo_path = std::path::PathBuf::from(home).join(".beaudy_notes.md");

    if !args.is_empty() && (args[0] == "list" || args[0] == "--list") {
        if !memo_path.exists() {
            println!("No notes saved yet.");
            return Ok(0);
        }
        let content = fs::read_to_string(memo_path)?;
        println!("=== Beaudy Notes ===");
        print!("{}", content);
        return Ok(0);
    }

    if args.is_empty() {
        eprintln!("Usage:");
        eprintln!("  bmemo <text to save>");
        eprintln!("  bmemo list");
        return Ok(1);
    }

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(memo_path)?;

    let now_str = format_datetime(SystemTime::now());
    let text = args.join(" ");
    writeln!(file, "- [{}]: {}", now_str, text)?;
    println!("Note saved to ~/.beaudy_notes.md");
    Ok(0)
}

/// Evaluate math expressions inline
///
/// ```
/// use beaudy_builtins::run_bcalc;
/// let res = run_bcalc(&["2", "+", "2"]);
/// assert_eq!(res.unwrap(), 0);
/// ```
pub fn run_bcalc(args: &[&str]) -> Result<i32, Box<dyn std::error::Error>> {
    if args.is_empty() {
        eprintln!("Usage: bcalc <math expression>");
        return Ok(1);
    }
    let expr = args.join(" ");
    match meval::eval_str(&expr) {
        Ok(result) => {
            println!("{}", result);
            Ok(0)
        }
        Err(e) => {
            eprintln!("bcalc error: {}", e);
            Ok(1)
        }
    }
}

/// Safe delete utility moving files to ~/.btrash with list, restore, and empty options
pub fn run_btrash(args: &[&str]) -> Result<i32, Box<dyn std::error::Error>> {
    let home_dir = if cfg!(windows) {
        std::env::var("USERPROFILE").ok()
    } else {
        std::env::var("HOME").ok()
    };

    let Some(home) = home_dir else {
        eprintln!("btrash: Could not determine home directory");
        return Ok(1);
    };

    let trash_dir = std::path::PathBuf::from(home).join(".btrash");
    #[allow(clippy::collapsible_if)]
    if !trash_dir.exists() {
        if let Err(e) = fs::create_dir_all(&trash_dir) {
            eprintln!("btrash: Could not create trash directory: {}", e);
            return Ok(1);
        }
    }

    if args.is_empty() {
        eprintln!("Usage:");
        eprintln!("  btrash <file1> [file2...]");
        eprintln!("  btrash list");
        eprintln!("  btrash restore <filename>");
        eprintln!("  btrash empty");
        return Ok(1);
    }

    match args[0] {
        "list" | "--list" => {
            let entries = match fs::read_dir(&trash_dir) {
                Ok(e) => e,
                Err(err) => {
                    eprintln!("btrash: Failed to read trash directory: {}", err);
                    return Ok(1);
                }
            };
            println!("=== Trash Contents (~/.btrash) ===");
            let mut count = 0;
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                let Ok(metadata) = entry
                    .metadata()
                    .or_else(|_| entry.path().symlink_metadata())
                else {
                    continue;
                };
                let is_dir = metadata.is_dir();
                let size_str = format_size(metadata.len(), is_dir);
                let mod_time = metadata.modified().unwrap_or(UNIX_EPOCH);
                let time_str = format_datetime(mod_time);
                println!("{:<30} {:<10} {:<20}", name, size_str, time_str);
                count += 1;
            }
            if count == 0 {
                println!("Trash is empty.");
            } else {
                println!("\r\nTotal items: {}", count);
            }
            Ok(0)
        }
        "restore" | "--restore" => {
            if args.len() < 2 {
                eprintln!("Usage: btrash restore <filename>");
                return Ok(1);
            }
            let filename = args[1];
            let source = trash_dir.join(filename);
            if !source.exists() {
                eprintln!("btrash: '{}' is not in trash", filename);
                return Ok(1);
            }
            let dest = Path::new(filename);
            if let Err(e) = fs::rename(&source, dest) {
                if source.is_dir() {
                    eprintln!("btrash: failed to restore directory '{}': {}", filename, e);
                    return Ok(1);
                } else if let Err(copy_err) = fs::copy(&source, dest) {
                    eprintln!(
                        "btrash: failed to restore file '{}': {}",
                        filename, copy_err
                    );
                    return Ok(1);
                } else {
                    let _ = fs::remove_file(&source);
                }
            }
            println!("Restored '{}' to current directory.", filename);
            Ok(0)
        }
        "empty" | "--empty" => {
            let entries = match fs::read_dir(&trash_dir) {
                Ok(e) => e,
                Err(err) => {
                    eprintln!("btrash: Failed to read trash directory: {}", err);
                    return Ok(1);
                }
            };
            let mut count = 0;
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let _ = fs::remove_dir_all(&path);
                } else {
                    let _ = fs::remove_file(&path);
                }
                count += 1;
            }
            println!("Emptied trash (removed {} items).", count);
            Ok(0)
        }
        _ => {
            for arg in args {
                let path = Path::new(arg);
                if !path.exists() {
                    eprintln!("btrash: '{}' not found", arg);
                    continue;
                }
                if let Some(file_name) = path.file_name() {
                    let dest = trash_dir.join(file_name);
                    if let Err(e) = fs::rename(path, &dest) {
                        if path.is_file() && fs::copy(path, &dest).is_ok() {
                            let _ = fs::remove_file(path);
                            println!("Moved '{}' to trash.", arg);
                        } else {
                            eprintln!("btrash: failed to move '{}': {}", arg, e);
                        }
                    } else {
                        println!("Moved '{}' to trash.", arg);
                    }
                }
            }
            Ok(0)
        }
    }
}

/// Hashing utility
///
/// ```
/// use beaudy_builtins::run_bhash;
/// let res = run_bhash(&["hello"]);
/// assert_eq!(res.unwrap(), 0);
/// ```
pub fn run_bhash(args: &[&str]) -> Result<i32, Box<dyn std::error::Error>> {
    if args.is_empty() {
        eprintln!("Usage: bhash [--md5 | --sha256] <string or file>");
        return Ok(1);
    }

    let mut use_sha256 = false;
    let target;

    if args[0] == "--sha256" {
        use_sha256 = true;
        if args.len() < 2 {
            eprintln!("Usage: bhash [--md5 | --sha256] <string or file>");
            return Ok(1);
        }
        target = args[1..].join(" ");
    } else if args[0] == "--md5" {
        if args.len() < 2 {
            eprintln!("Usage: bhash [--md5 | --sha256] <string or file>");
            return Ok(1);
        }
        target = args[1..].join(" ");
    } else {
        target = args.join(" "); // default MD5
    }

    let path = Path::new(&target);
    let bytes = if path.exists() && path.is_file() {
        fs::read(path).unwrap_or_else(|_| target.as_bytes().to_vec())
    } else {
        target.as_bytes().to_vec()
    };

    if use_sha256 {
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let result = hasher.finalize();
        println!("{:x}", result);
    } else {
        let digest = md5::compute(&bytes);
        println!("{:x}", digest);
    }

    Ok(0)
}

/// Environment variable export built-in
///
/// ```
/// use beaudy_builtins::run_export;
/// let res = run_export(&["BEAUDY_DOC_TEST=1"]);
/// assert_eq!(res.unwrap(), 0);
/// ```
#[allow(unsafe_code)]
pub fn run_export(args: &[&str]) -> Result<i32, Box<dyn std::error::Error>> {
    if args.is_empty() {
        let mut vars: Vec<(String, String)> = std::env::vars().collect();
        vars.sort_by(|a, b| a.0.cmp(&b.0));
        for (k, v) in vars {
            println!("export {}={}", k, v);
        }
        return Ok(0);
    }

    for arg in args {
        let parts: Vec<&str> = arg.splitn(2, '=').collect();
        if parts.len() == 2 {
            let k = parts[0].trim();
            let v = parts[1].trim().trim_matches('"').trim_matches('\'');
            unsafe {
                std::env::set_var(k, v);
            }
        } else {
            let k = parts[0].trim();
            if std::env::var(k).is_err() {
                unsafe {
                    std::env::set_var(k, "");
                }
            }
        }
    }
    Ok(0)
}

/// Command alias built-in
///
/// ```
/// use beaudy_builtins::run_alias;
/// use std::collections::HashMap;
/// let aliases = HashMap::new();
/// let res = run_alias(&[], &aliases);
/// assert_eq!(res.unwrap(), 0);
/// ```
pub fn run_alias(
    args: &[&str],
    aliases: &std::collections::HashMap<String, String>,
) -> Result<i32, Box<dyn std::error::Error>> {
    if args.is_empty() {
        let mut keys: Vec<&String> = aliases.keys().collect();
        keys.sort();
        for k in keys {
            println!("alias {}='{}'", k, aliases[k]);
        }
        return Ok(0);
    }
    Ok(0)
}

/// Clears the terminal screen.
///
/// ```
/// use beaudy_builtins::run_clear;
/// let res = run_clear();
/// assert_eq!(res.unwrap(), 0);
/// ```
pub fn run_clear() -> Result<i32, Box<dyn std::error::Error>> {
    crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
        crossterm::cursor::MoveTo(0, 0)
    )?;
    Ok(0)
}
