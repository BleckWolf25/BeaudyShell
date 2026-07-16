/*
 * @file main.rs
 *
 * @version 1.0.0
 * @author BleckWolf25
 * @license MIT
 *
 * @summary Main shell entry point with interactive REPL and accessibility features.
 *
 * @description
 * This module implements the primary shell interface, handling user input processing,
 * command history navigation, autocomplete suggestions, syntax highlighting, and
 * coordinating with the router and accessibility modules for a complete shell experience.
 *
 * @since 16/07/2026
 * @updated 16/07/2026
 */
// ---------- IMPORTS
use beaudy_a11y::{command_finished, input_start, output_start, prompt_start};
use beaudy_router::execute_interactive_command;
use crossterm::{
    cursor::MoveToColumn,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode},
};
use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use std::sync::OnceLock;

// ---------- MODULE DECLARATIONS
mod config;

// ---------- STATIC DATA
static PATH_COMMANDS: OnceLock<HashSet<String>> = OnceLock::new();

// ---------- PATH COMMAND DISCOVERY
fn get_path_commands() -> &'static HashSet<String> {
    // Cache PATH executable discovery for performance
    PATH_COMMANDS.get_or_init(|| {
        let mut commands: HashSet<String> = HashSet::new();
        let path_env = std::env::var("PATH").unwrap_or_default();
        for p in std::env::split_paths(&path_env) {
            if let Ok(entries) = fs::read_dir(p) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    #[cfg(windows)]
                    {
                        // Windows: check for executable extensions
                        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                            let ext_lower = ext.to_lowercase();
                            if (ext_lower == "exe" || ext_lower == "cmd" || ext_lower == "bat")
                                && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                            {
                                commands.insert(stem.to_string());
                            }
                        }
                    }
                    #[cfg(not(windows))]
                    {
                        // Unix: check for executable permission bit
                        use std::os::unix::fs::PermissionsExt;
                        if let Ok(metadata) = entry.metadata()
                            && metadata.is_file()
                            && (metadata.permissions().mode() & 0o111) != 0
                            && let Some(name) = path.file_name().and_then(|s| s.to_str())
                        {
                            commands.insert(name.to_string());
                        }
                    }
                }
            }
        }
        commands
    })
}

fn command_exists(cmd: &str) -> bool {
    // Check against common commands first for performance
    let common = [
        "git", "cargo", "npm", "node", "python", "pip", "ls", "cd", "pwd", "mkdir", "rm", "cp",
        "mv", "clear",
    ];
    if common.contains(&cmd) {
        return true;
    }
    get_path_commands().contains(cmd)
}

// ---------- SYNTAX HIGHLIGHTING
fn print_highlighted(stdout: &mut io::Stdout, input: &str) -> io::Result<()> {
    if input.is_empty() {
        return Ok(());
    }

    // Extract the first word for coloring
    let mut word_start = 0;
    let mut word_end = 0;
    let mut found_word = false;

    for (i, c) in input.char_indices() {
        if !c.is_whitespace() {
            if !found_word {
                word_start = i;
                found_word = true;
            }
            word_end = i + c.len_utf8();
        } else if found_word {
            break;
        }
    }

    if found_word {
        let first_word = &input[word_start..word_end];

        execute!(stdout, Print(&input[..word_start]))?;

        // Color code: cyan for builtins, green for existing commands, red for unknown
        let is_builtin = ["bls", "bhelp", "exit"].contains(&first_word);
        let exists = is_builtin || command_exists(first_word);

        let color = if is_builtin {
            Color::Cyan
        } else if exists {
            Color::Green
        } else {
            Color::Red
        };

        execute!(
            stdout,
            SetForegroundColor(color),
            Print(first_word),
            ResetColor
        )?;
        execute!(stdout, Print(&input[word_end..]))?;
    } else {
        execute!(stdout, Print(input))?;
    }

    Ok(())
}

// ---------- AUTOCOMPLETION
fn get_suggestion(input: &str) -> Option<String> {
    if input.is_empty() {
        return None;
    }

    let words: Vec<&str> = input.split_whitespace().collect();
    if words.is_empty() {
        return None;
    }

    let last_word_is_arg = input.ends_with(' ') || words.len() > 1;

    if !last_word_is_arg {
        // Suggest completions for the command itself
        let cmd = words[0];
        let builtins = ["bls", "bhelp", "exit"];
        for b in &builtins {
            if b.starts_with(cmd) && *b != cmd {
                return Some(b[cmd.len()..].to_string());
            }
        }

        // Check current directory for file completions
        if let Ok(entries) = fs::read_dir(".") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.starts_with(cmd) && name != cmd {
                    return Some(name[cmd.len()..].to_string());
                }
            }
        }
    } else {
        // Suggest completions for arguments
        let prefix = if input.ends_with(' ') {
            ""
        } else {
            words.last().unwrap_or(&"")
        };
        if let Ok(entries) = fs::read_dir(".") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.starts_with(prefix) && name != *prefix {
                    return Some(name[prefix.len()..].to_string());
                }
            }
        }
    }

    None
}

fn get_candidates(input: &str) -> Vec<String> {
    // Get all possible completion candidates for tab cycling
    let mut candidates = Vec::new();
    if input.is_empty() {
        return candidates;
    }

    let words: Vec<&str> = input.split_whitespace().collect();
    if words.is_empty() {
        return candidates;
    }

    let last_word_is_arg = input.ends_with(' ') || words.len() > 1;

    if !last_word_is_arg {
        let cmd = words[0];
        let builtins = ["bls", "bhelp", "exit"];
        for b in &builtins {
            if b.starts_with(cmd) {
                candidates.push((*b).to_string());
            }
        }

        if let Ok(entries) = fs::read_dir(".") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.starts_with(cmd) {
                    candidates.push(name);
                }
            }
        }
    } else {
        let prefix = if input.ends_with(' ') {
            ""
        } else {
            words.last().unwrap_or(&"")
        };
        if let Ok(entries) = fs::read_dir(".") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.starts_with(prefix) {
                    candidates.push(name);
                }
            }
        }
    }

    candidates.sort();
    candidates.dedup();
    candidates
}

// ---------- UI RENDERING
fn render_line(
    stdout: &mut io::Stdout,
    prompt: &str,
    prompt_color: Color,
    input: &str,
    cursor_pos: usize,
    suggestion: Option<&str>,
) -> io::Result<()> {
    // Clear line and redraw with current state
    execute!(
        stdout,
        MoveToColumn(0),
        Clear(ClearType::UntilNewLine),
        SetForegroundColor(prompt_color),
        Print(prompt),
        ResetColor
    )?;

    print_highlighted(stdout, input)?;

    // Show suggestion in gray if cursor is at end of input
    if let Some(sug) = suggestion {
        #[allow(clippy::collapsible_if)]
        if cursor_pos == input.len() {
            execute!(
                stdout,
                SetForegroundColor(Color::DarkGrey),
                Print(sug),
                ResetColor
            )?;
        }
    }

    let col = (prompt.len() + cursor_pos) as u16;
    execute!(stdout, MoveToColumn(col))?;
    stdout.flush()?;

    Ok(())
}

// ---------- PROMPT GENERATION
fn get_prompt_string(config: &config::Config) -> String {
    let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

    match config.prompt_style.as_str() {
        "static" => "beaudy> ".to_string(),
        "compact" => {
            // Show only the current folder name
            let folder_name = current_dir
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "beaudy".to_string());
            format!("{} beaudy> ", folder_name)
        }
        _ => {
            // "path" default: show full path with home shorthand
            let home_dir = if cfg!(windows) {
                std::env::var("USERPROFILE").ok()
            } else {
                std::env::var("HOME").ok()
            };

            let path_str = current_dir.to_string_lossy().into_owned();
            let mut display_path = if let Some(home) = home_dir {
                if path_str.starts_with(&home) {
                    path_str.replace(&home, "~")
                } else {
                    path_str
                }
            } else {
                path_str
            };

            // Normalize path separators to forward slashes for cleaner UI
            if !cfg!(windows) {
                display_path = display_path.replace('\\', "/");
            }

            format!("{} beaudy> ", display_path)
        }
    }
}

// ---------- MAIN ENTRY POINT
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::load_config();
    let prompt_color = config.get_crossterm_color();

    enable_raw_mode()?;
    let mut stdout = io::stdout();

    let mut history: Vec<String> = Vec::new();

    // Main REPL loop
    loop {
        let prompt = get_prompt_string(&config);

        // 1. OSC 133 A: Start of Prompt
        execute!(stdout, Print(prompt_start()))?;

        // Print the prompt initially
        execute!(
            stdout,
            SetForegroundColor(prompt_color),
            Print(&prompt),
            ResetColor
        )?;
        stdout.flush()?;

        // 2. OSC 133 B: Start of User Input
        execute!(stdout, Print(input_start()))?;
        stdout.flush()?;

        let mut input = String::new();
        let mut cursor_pos = 0;
        let mut history_index: Option<usize> = None;
        let mut temp_input = String::new();

        let mut tab_candidates: Option<Vec<String>> = None;
        let mut tab_index = 0;
        let mut tab_start_idx = 0;

        // Keystroke capture loop
        loop {
            let sug = get_suggestion(&input);
            render_line(
                &mut stdout,
                &prompt,
                prompt_color,
                &input,
                cursor_pos,
                sug.as_deref(),
            )?;

            if let Event::Key(key_event) = event::read()? {
                // In raw mode, only handle KeyPress events
                if key_event.kind == KeyEventKind::Release {
                    continue;
                }

                let ctrl_pressed = key_event.modifiers.contains(KeyModifiers::CONTROL);

                match key_event.code {
                    // ---------- EXIT HANDLING
                    KeyCode::Char('c') if ctrl_pressed => {
                        disable_raw_mode()?;
                        println!("\r\nGoodbye!");
                        return Ok(());
                    }
                    // ---------- COMMAND EXECUTION
                    KeyCode::Enter => {
                        execute!(stdout, Print("\r\n"))?;
                        stdout.flush()?;
                        break;
                    }
                    // ---------- TEXT EDITING
                    KeyCode::Backspace => {
                        tab_candidates = None;
                        if cursor_pos > 0 {
                            let mut chars: Vec<char> = input.chars().collect();
                            chars.remove(cursor_pos - 1);
                            input = chars.into_iter().collect();
                            cursor_pos -= 1;
                        }
                    }
                    KeyCode::Delete => {
                        tab_candidates = None;
                        if cursor_pos < input.len() {
                            let mut chars: Vec<char> = input.chars().collect();
                            chars.remove(cursor_pos);
                            input = chars.into_iter().collect();
                        }
                    }
                    // ---------- CURSOR MOVEMENT
                    KeyCode::Left => {
                        tab_candidates = None;
                        cursor_pos = cursor_pos.saturating_sub(1);
                    }
                    KeyCode::Right => {
                        tab_candidates = None;
                        if cursor_pos < input.len() {
                            cursor_pos += 1;
                        } else if let Some(ref suggestion_str) = sug {
                            // Accept inline suggestion when moving right at end
                            input.push_str(suggestion_str);
                            cursor_pos = input.len();
                        }
                    }
                    KeyCode::Home => {
                        tab_candidates = None;
                        cursor_pos = 0;
                    }
                    KeyCode::End => {
                        tab_candidates = None;
                        #[allow(clippy::collapsible_if)]
                        if cursor_pos == input.len() {
                            if let Some(ref suggestion_str) = sug {
                                input.push_str(suggestion_str);
                            }
                        }
                        cursor_pos = input.len();
                    }
                    // ---------- HISTORY NAVIGATION
                    KeyCode::Up => {
                        tab_candidates = None;
                        if !history.is_empty() {
                            if history_index.is_none() {
                                temp_input = input.clone();
                                history_index = Some(history.len() - 1);
                            } else {
                                let idx = history_index.unwrap();
                                if idx > 0 {
                                    history_index = Some(idx - 1);
                                }
                            }
                            input = history[history_index.unwrap()].clone();
                            cursor_pos = input.len();
                        }
                    }
                    KeyCode::Down => {
                        tab_candidates = None;
                        if let Some(idx) = history_index {
                            if idx + 1 < history.len() {
                                history_index = Some(idx + 1);
                                input = history[idx + 1].clone();
                            } else {
                                history_index = None;
                                input = temp_input.clone();
                            }
                            cursor_pos = input.len();
                        }
                    }
                    // ---------- AUTOCOMPPLETION CYCLING
                    KeyCode::Tab => {
                        if let Some(ref candidates) = tab_candidates {
                            tab_index = (tab_index + 1) % candidates.len();
                        } else {
                            let candidates = get_candidates(&input);
                            if !candidates.is_empty() {
                                let words: Vec<&str> = input.split_whitespace().collect();
                                let prefix = if input.ends_with(' ') {
                                    ""
                                } else {
                                    words.last().unwrap_or(&"")
                                };
                                tab_start_idx = if input.ends_with(' ') {
                                    input.len()
                                } else {
                                    input.rfind(prefix).unwrap_or(input.len())
                                };
                                tab_candidates = Some(candidates);
                                tab_index = 0;
                            }
                        }

                        if let Some(ref candidates) = tab_candidates {
                            let candidate = &candidates[tab_index];
                            let mut new_input = input[..tab_start_idx].to_string();
                            new_input.push_str(candidate);
                            input = new_input;
                            cursor_pos = input.len();
                        }
                    }
                    // ---------- CHARACTER INPUT
                    KeyCode::Char(c) => {
                        tab_candidates = None;
                        let mut chars: Vec<char> = input.chars().collect();
                        chars.insert(cursor_pos, c);
                        input = chars.into_iter().collect();
                        cursor_pos += 1;
                    }
                    _ => {}
                }
            }
        }

        // ---------- COMMAND PROCESSING
        let trimmed = input.trim();
        if !trimmed.is_empty() {
            if trimmed == "exit" {
                break;
            }

            // Append to history if it's different from the last executed command
            if history.is_empty() || history.last().unwrap() != trimmed {
                history.push(trimmed.to_string());
            }

            // 3. OSC 133 C: Start of Command Output
            execute!(stdout, Print(output_start()))?;
            stdout.flush()?;

            let exit_code = match execute_interactive_command(trimmed) {
                Ok(code) => code,
                Err(e) => {
                    execute!(stdout, Print(format!("Error executing command: {e}\r\n")))?;
                    1
                }
            };

            // 4. OSC 133 D: Command Finished
            execute!(stdout, Print(command_finished(exit_code)))?;
            stdout.flush()?;
        }
    }

    disable_raw_mode()?;
    Ok(())
}
