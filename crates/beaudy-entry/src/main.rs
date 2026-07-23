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
 * @updated 23/07/2026
 */
// ---------- IMPORTS
use beaudy_a11y::{command_finished, input_start, output_start, prompt_start};
use crossterm::{
    cursor::MoveToColumn,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode},
};
use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

// ---------- MODULE DECLARATIONS
mod config;

// ---------- STATIC DATA
static PATH_COMMANDS: OnceLock<HashSet<String>> = OnceLock::new();

// ---------- PERSISTENT HISTORY MANAGEMENT
fn get_history_file_path() -> Option<PathBuf> {
    let home_dir = if cfg!(windows) {
        std::env::var("USERPROFILE").ok()
    } else {
        std::env::var("HOME").ok()
    };
    home_dir.map(|h| PathBuf::from(h).join(".beaudy_history"))
}

#[allow(clippy::collapsible_if)]
fn load_history() -> Vec<String> {
    let mut history = Vec::new();
    if let Some(path) = get_history_file_path() {
        if let Ok(content) = fs::read_to_string(path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    history.push(trimmed.to_string());
                }
            }
        }
    }
    history
}

#[allow(clippy::collapsible_if)]
fn append_history_entry(cmd: &str) {
    if let Some(path) = get_history_file_path() {
        if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(file, "{}", cmd);
        }
    }
}

// ---------- PATH COMPLETION HELPER
fn resolve_path_completion(prefix: &str) -> Vec<String> {
    let mut completions = Vec::new();

    let (dir_part, file_prefix) = if let Some(idx) = prefix.rfind(['/', '\\']) {
        (&prefix[..=idx], &prefix[idx + 1..])
    } else {
        ("", prefix)
    };

    let expanded_dir = if dir_part.starts_with('~') {
        let home_dir = if cfg!(windows) {
            std::env::var("USERPROFILE").unwrap_or_default()
        } else {
            std::env::var("HOME").unwrap_or_default()
        };
        dir_part.replacen('~', &home_dir, 1)
    } else if dir_part.is_empty() {
        ".".to_string()
    } else {
        dir_part.to_string()
    };

    let target_path = std::path::Path::new(&expanded_dir);
    if let Ok(entries) = fs::read_dir(target_path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with(file_prefix) {
                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                let trailing = if is_dir { "/" } else { "" };
                let item = format!("{}{}{}", dir_part, name, trailing);
                completions.push(item);
            }
        }
    }

    completions
}

// ---------- GIT INTEGRATION
fn get_git_info() -> Option<String> {
    // Attempt to get branch name via `git rev-parse`
    let branch_output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;

    if !branch_output.status.success() {
        return None;
    }

    let branch = String::from_utf8_lossy(&branch_output.stdout)
        .trim()
        .to_string();
    if branch.is_empty() {
        return None;
    }

    // Check for uncommitted changes (porcelain output)
    let dirty_output = Command::new("git")
        .args(["status", "--porcelain"])
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;

    let is_dirty = !dirty_output.stdout.is_empty();
    let indicator = if is_dirty {
        format!("[{}*]", branch)
    } else {
        format!("[{}]", branch)
    };
    Some(indicator)
}

// ---------- WORD NAVIGATION HELPERS
fn prev_word_boundary(input: &str, cursor: usize) -> usize {
    let chars: Vec<char> = input.chars().collect();
    let mut pos = cursor;
    // Skip trailing whitespace
    while pos > 0 && chars[pos - 1].is_whitespace() {
        pos -= 1;
    }
    // Move back over word characters
    while pos > 0 && !chars[pos - 1].is_whitespace() {
        pos -= 1;
    }
    pos
}

fn next_word_boundary(input: &str, cursor: usize) -> usize {
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut pos = cursor;
    // Skip over word characters
    while pos < len && !chars[pos].is_whitespace() {
        pos += 1;
    }
    // Skip trailing whitespace
    while pos < len && chars[pos].is_whitespace() {
        pos += 1;
    }
    pos
}

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
    let common = [
        "git", "cargo", "npm", "node", "python", "pip", "ls", "cd", "pwd", "mkdir", "rm", "cp",
        "mv", "clear",
    ];
    if common.contains(&cmd) {
        return true;
    }
    if let Some(cmds) = PATH_COMMANDS.get() {
        cmds.contains(cmd)
    } else {
        false
    }
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

        queue!(stdout, Print(&input[..word_start]))?;

        // Color code: cyan for builtins, green for existing commands, red for unknown
        let is_builtin = [
            "bls", "bhelp", "exit", "bconfig", "pwd", "bsetup", "cd", "bmemo", "bcalc", "btrash",
            "bhash",
        ]
        .contains(&first_word);
        let exists = is_builtin || command_exists(first_word);

        let color = if is_builtin {
            Color::Cyan
        } else if exists {
            Color::Green
        } else {
            Color::Red
        };

        queue!(
            stdout,
            SetForegroundColor(color),
            Print(first_word),
            ResetColor
        )?;
        queue!(stdout, Print(&input[word_end..]))?;
    } else {
        queue!(stdout, Print(input))?;
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
        let cmd = words[0];
        if cmd.contains(['/', '\\']) || cmd.starts_with('.') || cmd.starts_with('~') {
            let candidates = resolve_path_completion(cmd);
            for c in candidates {
                if c.starts_with(cmd) && c != cmd {
                    return Some(c[cmd.len()..].to_string());
                }
            }
        } else {
            let builtins = [
                "bls", "bhelp", "exit", "bconfig", "pwd", "bsetup", "cd", "bmemo", "bcalc",
                "btrash", "bhash", "pushd", "popd", "dirs", "export", "alias", "unalias",
            ];
            for b in &builtins {
                if b.starts_with(cmd) && *b != cmd {
                    return Some(b[cmd.len()..].to_string());
                }
            }

            let candidates = resolve_path_completion(cmd);
            for c in candidates {
                if c.starts_with(cmd) && c != cmd {
                    return Some(c[cmd.len()..].to_string());
                }
            }
        }
    } else {
        let prefix = if input.ends_with(' ') {
            ""
        } else {
            words.last().copied().unwrap_or("")
        };
        let candidates = resolve_path_completion(prefix);
        for c in candidates {
            if c.starts_with(prefix) && c != prefix {
                return Some(c[prefix.len()..].to_string());
            }
        }
    }

    None
}

fn get_candidates(input: &str) -> Vec<String> {
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
        if cmd.contains(['/', '\\']) || cmd.starts_with('.') || cmd.starts_with('~') {
            candidates.extend(resolve_path_completion(cmd));
        } else {
            let builtins = [
                "bls", "bhelp", "exit", "bconfig", "pwd", "bsetup", "cd", "bmemo", "bcalc",
                "btrash", "bhash", "pushd", "popd", "dirs", "export", "alias", "unalias",
            ];
            for b in &builtins {
                if b.starts_with(cmd) {
                    candidates.push((*b).to_string());
                }
            }
            candidates.extend(resolve_path_completion(cmd));
        }
    } else {
        let prefix = if input.ends_with(' ') {
            ""
        } else {
            words.last().copied().unwrap_or("")
        };
        candidates.extend(resolve_path_completion(prefix));
    }

    candidates.sort();
    candidates.dedup();
    candidates
}

// ---------- UI RENDERING
#[allow(clippy::too_many_arguments)]
fn render_line(
    stdout: &mut io::Stdout,
    prompt: &str,
    prompt_color: Color,
    input: &str,
    cursor_pos: usize,
    suggestion: Option<&str>,
    search_state: Option<(&str, &str)>,
    last_exit_code: i32,
) -> io::Result<()> {
    if let Some((query, matched)) = search_state {
        queue!(
            stdout,
            MoveToColumn(0),
            Clear(ClearType::UntilNewLine),
            SetForegroundColor(Color::Yellow),
            Print(format!("(reverse-i-search)'{}': ", query)),
            SetForegroundColor(Color::White),
            Print(matched),
            ResetColor
        )?;
        let col = (19 + query.len()) as u16;
        queue!(stdout, MoveToColumn(col))?;
        stdout.flush()?;
        return Ok(());
    }

    // Exit status indicator: green arrow on success, red arrow+code on failure
    let (arrow_color, arrow_str) = if last_exit_code == 0 {
        (Color::Green, "➜ ".to_string())
    } else {
        (Color::Red, format!("➜ [{}] ", last_exit_code))
    };

    // Clear line and redraw with current state
    queue!(
        stdout,
        MoveToColumn(0),
        Clear(ClearType::UntilNewLine),
        SetForegroundColor(arrow_color),
        Print(&arrow_str),
        SetForegroundColor(prompt_color),
        Print(prompt),
        ResetColor
    )?;

    print_highlighted(stdout, input)?;

    // Show suggestion in gray if cursor is at end of input
    if let Some(sug) = suggestion {
        #[allow(clippy::collapsible_if)]
        if cursor_pos == input.len() {
            queue!(
                stdout,
                SetForegroundColor(Color::DarkGrey),
                Print(sug),
                ResetColor
            )?;
        }
    }

    let col = (arrow_str.chars().count() + prompt.chars().count() + cursor_pos) as u16;
    queue!(stdout, MoveToColumn(col))?;
    stdout.flush()?;

    Ok(())
}

// ---------- PROMPT GENERATION
fn get_prompt_string(config: &config::Config) -> String {
    let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let git_info = get_git_info();
    let git_suffix = git_info
        .as_deref()
        .map(|g| format!(" {}", g))
        .unwrap_or_default();

    match config.prompt_style.as_str() {
        "static" => format!("beaudy>{} ", git_suffix),
        "compact" => {
            // Show only the current folder name
            let folder_name = current_dir
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "beaudy".to_string());
            format!("{}{} beaudy> ", folder_name, git_suffix)
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

            format!("{}{} beaudy> ", display_path, git_suffix)
        }
    }
}

// ---------- MAIN ENTRY POINT
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Spawn background thread to populate PATH commands without blocking startup UI
    std::thread::spawn(|| {
        let _ = get_path_commands();
    });

    enable_raw_mode()?;
    let mut stdout = io::stdout();

    let mut history = load_history();
    let mut last_exit_code: i32 = 0;

    // Main REPL loop
    loop {
        let config = config::load_config();
        let prompt_color = config.get_crossterm_color();
        let prompt = get_prompt_string(&config);

        // 1. OSC 133 A: Start of Prompt
        queue!(stdout, Print(prompt_start()))?;

        // Print the initial prompt (arrow + path/git) with exit indicator
        let (arrow_color, arrow_str) = if last_exit_code == 0 {
            (Color::Green, "➜ ".to_string())
        } else {
            (Color::Red, format!("➜ [{}] ", last_exit_code))
        };
        queue!(
            stdout,
            SetForegroundColor(arrow_color),
            Print(&arrow_str),
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

        let mut search_mode = false;
        let mut search_query = String::new();
        let mut search_match = String::new();

        // Keystroke capture loop
        loop {
            let sug = get_suggestion(&input);
            let search_state = if search_mode {
                Some((search_query.as_str(), search_match.as_str()))
            } else {
                None
            };

            render_line(
                &mut stdout,
                &prompt,
                prompt_color,
                &input,
                cursor_pos,
                sug.as_deref(),
                search_state,
                last_exit_code,
            )?;

            if search_mode {
                if let Event::Key(key_event) = event::read()? {
                    if key_event.kind == KeyEventKind::Release {
                        continue;
                    }
                    let ctrl_pressed = key_event.modifiers.contains(KeyModifiers::CONTROL);

                    match key_event.code {
                        KeyCode::Char('r') if ctrl_pressed => {
                            if !history.is_empty() {
                                let current_idx = history
                                    .iter()
                                    .rposition(|h| h == &search_match)
                                    .unwrap_or(history.len());
                                #[allow(clippy::collapsible_if)]
                                if current_idx > 0 {
                                    if let Some(found) = history[..current_idx]
                                        .iter()
                                        .rfind(|h| h.contains(&search_query))
                                    {
                                        search_match = found.clone();
                                    }
                                }
                            }
                        }
                        KeyCode::Char('c') if ctrl_pressed => {
                            search_mode = false;
                            search_query.clear();
                            search_match.clear();
                        }
                        KeyCode::Esc => {
                            search_mode = false;
                            search_query.clear();
                            search_match.clear();
                        }
                        KeyCode::Enter | KeyCode::Right => {
                            if !search_match.is_empty() {
                                input = search_match.clone();
                                cursor_pos = input.len();
                            }
                            search_mode = false;
                            search_query.clear();
                            search_match.clear();
                        }
                        KeyCode::Backspace => {
                            search_query.pop();
                            search_match = history
                                .iter()
                                .rfind(|h| h.contains(&search_query))
                                .cloned()
                                .unwrap_or_default();
                        }
                        KeyCode::Char(c) if !ctrl_pressed => {
                            search_query.push(c);
                            search_match = history
                                .iter()
                                .rfind(|h| h.contains(&search_query))
                                .cloned()
                                .unwrap_or_default();
                        }
                        _ => {}
                    }
                }
                continue;
            }

            if let Event::Key(key_event) = event::read()? {
                // In raw mode, only handle KeyPress events
                if key_event.kind == KeyEventKind::Release {
                    continue;
                }

                let ctrl_pressed = key_event.modifiers.contains(KeyModifiers::CONTROL);
                let alt_pressed = key_event.modifiers.contains(KeyModifiers::ALT);

                match key_event.code {
                    // ---------- REVERSE HISTORY SEARCH
                    KeyCode::Char('r') if ctrl_pressed => {
                        search_mode = true;
                        search_query.clear();
                        search_match.clear();
                    }
                    // ---------- SIGNAL & EXIT HANDLING
                    KeyCode::Char('c') if ctrl_pressed => {
                        execute!(stdout, Print("^C\r\n"))?;
                        stdout.flush()?;
                        input.clear();
                        break;
                    }
                    KeyCode::Char('d') if ctrl_pressed => {
                        if input.is_empty() {
                            disable_raw_mode()?;
                            println!("\r\nGoodbye!");
                            return Ok(());
                        }
                    }
                    // ---------- SCREEN CLEARING
                    KeyCode::Char('l') if ctrl_pressed => {
                        tab_candidates = None;
                        execute!(
                            stdout,
                            crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
                            crossterm::cursor::MoveTo(0, 0)
                        )?;
                        // Re-render the current prompt line after clearing
                        let (ac, as_) = if last_exit_code == 0 {
                            (Color::Green, "➜ ".to_string())
                        } else {
                            (Color::Red, format!("➜ [{}] ", last_exit_code))
                        };
                        queue!(
                            stdout,
                            SetForegroundColor(ac),
                            Print(&as_),
                            SetForegroundColor(prompt_color),
                            Print(&prompt),
                            ResetColor
                        )?;
                        stdout.flush()?;
                    }
                    // ---------- LINE EDITING SHORTCUTS
                    // Ctrl+W: delete word before cursor
                    KeyCode::Char('w') if ctrl_pressed => {
                        tab_candidates = None;
                        let new_pos = prev_word_boundary(&input, cursor_pos);
                        input.drain(new_pos..cursor_pos);
                        cursor_pos = new_pos;
                    }
                    // Ctrl+U: delete from start to cursor
                    KeyCode::Char('u') if ctrl_pressed => {
                        tab_candidates = None;
                        input.drain(..cursor_pos);
                        cursor_pos = 0;
                    }
                    // Ctrl+K: delete from cursor to end
                    KeyCode::Char('k') if ctrl_pressed => {
                        tab_candidates = None;
                        input.truncate(cursor_pos);
                    }
                    // ---------- WORD NAVIGATION
                    // Alt+B or Ctrl+Left: move back one word
                    KeyCode::Char('b') if alt_pressed => {
                        tab_candidates = None;
                        cursor_pos = prev_word_boundary(&input, cursor_pos);
                    }
                    KeyCode::Left if ctrl_pressed => {
                        tab_candidates = None;
                        cursor_pos = prev_word_boundary(&input, cursor_pos);
                    }
                    // Alt+F or Ctrl+Right: move forward one word
                    KeyCode::Char('f') if alt_pressed => {
                        tab_candidates = None;
                        cursor_pos = next_word_boundary(&input, cursor_pos);
                    }
                    KeyCode::Right if ctrl_pressed => {
                        tab_candidates = None;
                        cursor_pos = next_word_boundary(&input, cursor_pos);
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
                    KeyCode::Left if !ctrl_pressed => {
                        tab_candidates = None;
                        cursor_pos = cursor_pos.saturating_sub(1);
                    }
                    KeyCode::Right if !ctrl_pressed => {
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
                    KeyCode::Char(c) if !ctrl_pressed && !alt_pressed => {
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

            // Save alias updates to ~/.beaudy.toml
            if let Some(alias_arg) = trimmed.strip_prefix("alias ") {
                let parts: Vec<&str> = alias_arg.trim().splitn(2, '=').collect();
                if parts.len() == 2 {
                    let k = parts[0].trim();
                    let v = parts[1].trim().trim_matches('"').trim_matches('\'');
                    config::save_alias(k, v);
                }
            } else if let Some(name) = trimmed.strip_prefix("unalias ") {
                config::remove_alias(name.trim());
            }

            // Append to history if it's different from the last executed command
            if history.is_empty() || history.last().unwrap() != trimmed {
                history.push(trimmed.to_string());
                append_history_entry(trimmed);
            }

            // 3. OSC 133 C: Start of Command Output
            execute!(stdout, Print(output_start()))?;
            stdout.flush()?;

            let exit_code = match beaudy_router::execute_pipeline(
                trimmed,
                &config.default_shell,
                &config.aliases,
            ) {
                Ok(code) => code,
                Err(e) => {
                    execute!(stdout, Print(format!("Error executing command: {e}\r\n")))?;
                    1
                }
            };

            last_exit_code = exit_code;

            // 4. OSC 133 D: Command Finished
            execute!(stdout, Print(command_finished(exit_code)))?;
            stdout.flush()?;
        }
    }

    disable_raw_mode()?;
    Ok(())
}

// ---------- TESTS
#[cfg(test)]
mod tests {
    use super::*;

    // ── Path Completion ───────────────────────────────────────────────────
    #[test]
    fn test_resolve_path_completion_subdirs() {
        let completions = resolve_path_completion("src/m");
        assert!(!completions.is_empty());
        assert!(completions.contains(&"src/main.rs".to_string()));
    }

    #[test]
    fn test_resolve_path_completion_root() {
        let completions = resolve_path_completion("Cargo");
        assert!(!completions.is_empty());
        assert!(completions.contains(&"Cargo.toml".to_string()));
    }

    #[test]
    fn test_resolve_path_completion_empty_prefix() {
        // Empty prefix should list all items in the current dir
        let completions = resolve_path_completion("");
        assert!(!completions.is_empty());
    }

    #[test]
    fn test_resolve_path_completion_nonexistent_dir() {
        let completions = resolve_path_completion("/no/such/path/xyz/");
        assert!(completions.is_empty());
    }

    #[test]
    fn test_resolve_path_completion_home_tilde() {
        // A "~/" prefix should be expanded; we just verify it doesn't panic
        let _completions = resolve_path_completion("~/");
    }

    // ── Autocompletion suggestions ─────────────────────────────────────────
    #[test]
    fn test_get_suggestion_builtin_prefix() {
        // Typing "bl" should suggest "s" (completing "bls")
        let sug = get_suggestion("bl");
        assert!(sug.is_some(), "Expected a suggestion for 'bl'");
    }

    #[test]
    fn test_get_suggestion_no_match() {
        // A gibberish prefix that matches nothing
        let sug = get_suggestion("zzzzbeaudyshellzzz");
        assert!(sug.is_none());
    }

    #[test]
    fn test_get_suggestion_empty_input() {
        let sug = get_suggestion("");
        assert!(sug.is_none());
    }

    #[test]
    fn test_get_candidates_builtin() {
        let candidates = get_candidates("bl");
        assert!(candidates.contains(&"bls".to_string()));
    }

    #[test]
    fn test_get_candidates_empty() {
        let candidates = get_candidates("");
        assert!(candidates.is_empty());
    }

    #[test]
    fn test_get_candidates_arg_completion() {
        // After a command + space, should complete paths
        let candidates = get_candidates("bls src/");
        // The crate's src directory should be found
        assert!(!candidates.is_empty());
    }

    // ── Prompt generation ─────────────────────────────────────────────────
    #[test]
    fn test_get_prompt_string_static() {
        let cfg = config::Config {
            prompt_style: "static".to_string(),
            ..Default::default()
        };
        let p = get_prompt_string(&cfg);
        assert!(
            p.contains("beaudy>"),
            "static prompt must contain 'beaudy>'"
        );
    }

    #[test]
    fn test_get_prompt_string_compact() {
        let cfg = config::Config {
            prompt_style: "compact".to_string(),
            ..Default::default()
        };
        let p = get_prompt_string(&cfg);
        assert!(
            p.contains("beaudy>"),
            "compact prompt must contain 'beaudy>'"
        );
    }

    #[test]
    fn test_get_prompt_string_path() {
        let cfg = config::Config {
            prompt_style: "path".to_string(),
            ..Default::default()
        };
        let p = get_prompt_string(&cfg);
        assert!(p.contains("beaudy>"), "path prompt must contain 'beaudy>'");
    }

    #[test]
    fn test_get_prompt_string_unknown_style_falls_back_to_path() {
        let cfg = config::Config {
            prompt_style: "unknown_style_xyz".to_string(),
            ..Default::default()
        };
        let p = get_prompt_string(&cfg);
        // Falls through to the _ arm which renders path-style
        assert!(p.contains("beaudy>"));
    }

    // ── Config color mapping ──────────────────────────────────────────────
    #[test]
    fn test_config_color_cyan_default() {
        use crossterm::style::Color;
        let cfg = config::Config::default();
        assert_eq!(cfg.get_crossterm_color(), Color::Cyan);
    }

    #[test]
    fn test_config_color_all_variants() {
        use crossterm::style::Color;
        let pairs = [
            ("green", Color::Green),
            ("blue", Color::Blue),
            ("magenta", Color::Magenta),
            ("yellow", Color::Yellow),
            ("white", Color::White),
            ("red", Color::Red),
            ("unknown", Color::Cyan), // fallback
        ];
        for (name, expected) in pairs {
            let cfg = config::Config {
                prompt_color: name.to_string(),
                ..Default::default()
            };
            assert_eq!(
                cfg.get_crossterm_color(),
                expected,
                "Color mismatch for '{}'",
                name
            );
        }
    }

    // ── History helpers ───────────────────────────────────────────────────
    #[test]
    fn test_history_file_path_is_some() {
        // As long as HOME / USERPROFILE is set the path should be deterministic
        let path = get_history_file_path();
        if std::env::var("HOME").is_ok() || std::env::var("USERPROFILE").is_ok() {
            assert!(path.is_some());
            let p = path.unwrap();
            assert!(p.to_string_lossy().contains(".beaudy_history"));
        }
    }

    #[test]
    fn test_load_history_is_vec() {
        // load_history should always return a Vec (even if empty)
        let h = load_history();
        // Just verify it doesn't panic and returns something
        let _ = h.len();
    }

    // ── Word navigation helpers ───────────────────────────────────────────
    #[test]
    fn test_prev_word_boundary_mid_word() {
        // "hello world|" -> back to "hello |" (pos 6)
        assert_eq!(prev_word_boundary("hello world", 11), 6);
    }

    #[test]
    fn test_prev_word_boundary_at_space() {
        // "hello |" -> all the way back past first word -> 0
        assert_eq!(prev_word_boundary("hello world", 6), 0);
    }

    #[test]
    fn test_prev_word_boundary_at_start() {
        assert_eq!(prev_word_boundary("hello", 0), 0);
    }

    #[test]
    fn test_prev_word_boundary_trailing_spaces() {
        assert_eq!(prev_word_boundary("hello   ", 8), 0);
    }

    #[test]
    fn test_next_word_boundary_at_start() {
        // "|hello world" -> forward to "hello |" (pos 6)
        assert_eq!(next_word_boundary("hello world", 0), 6);
    }

    #[test]
    fn test_next_word_boundary_at_space() {
        // "hello |world" -> forward to end (pos 11)
        assert_eq!(next_word_boundary("hello world", 6), 11);
    }

    #[test]
    fn test_next_word_boundary_at_end() {
        assert_eq!(next_word_boundary("hello", 5), 5);
    }

    #[test]
    fn test_next_word_boundary_multiple_spaces() {
        // "hello   |world" skips spaces to start of next word (pos 8)
        let s = "hello   world";
        assert_eq!(next_word_boundary(s, 5), 8);
    }
}
