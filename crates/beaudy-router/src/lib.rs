/*
 * @file lib.rs
 *
 * @version 1.0.0
 * @author BleckWolf25
 * @license MIT
 *
 * @summary Command execution router with PTY process management.
 *
 * @description
 * This module handles command routing between built-in and external commands,
 * implements directory change with OLDPWD tracking, and manages interactive
 * PTY process execution with bidirectional I/O threading.
 *
 * @since 16/07/2026
 * @updated 23/07/2026
 */
// ---------- IMPORTS
use crossterm::terminal::size;
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::io::{Read, Write};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

// ---------- DIRECTORY STATE
static OLDPWD: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();

fn get_oldpwd() -> &'static Mutex<Option<PathBuf>> {
    OLDPWD.get_or_init(|| Mutex::new(None))
}

// ---------- ENVIRONMENT VARIABLE EXPANSION
/// Expand environment variables in string ($VAR or ${VAR}).
///
/// ```
/// use beaudy_router::expand_env_vars;
/// unsafe { std::env::set_var("BEAUDY_TEST_VAR", "world"); }
/// assert_eq!(expand_env_vars("hello $BEAUDY_TEST_VAR"), "hello world");
/// ```
pub fn expand_env_vars(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '$' {
            let mut var_name = String::new();
            if chars.peek() == Some(&'{') {
                chars.next();
                while let Some(&ch) = chars.peek() {
                    if ch == '}' {
                        chars.next();
                        break;
                    }
                    var_name.push(ch);
                    chars.next();
                }
            } else {
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() || ch == '_' {
                        var_name.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
            }
            if !var_name.is_empty() {
                if let Ok(val) = std::env::var(&var_name) {
                    result.push_str(&val);
                }
            } else {
                result.push('$');
            }
        } else {
            result.push(c);
        }
    }
    result
}

// ---------- STAGE EXECUTION WITH PIPED INPUT / OUTPUT
fn execute_stage_with_input_output(
    input: &str,
    default_shell: &str,
    aliases: &std::collections::HashMap<String, String>,
    pipe_input: &[u8],
    writer: &mut dyn Write,
) -> Result<i32, Box<dyn std::error::Error>> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(0);
    }

    // Expand alias if applicable
    let mut parts: Vec<&str> = trimmed.split_whitespace().collect();
    let alias_expanded;
    if let Some(aliased) = aliases.get(parts[0]) {
        alias_expanded = format!("{} {}", aliased, parts[1..].join(" "));
        parts = alias_expanded.split_whitespace().collect();
    }

    let cmd_name = parts[0];
    let args = &parts[1..];

    // Builtins handling
    if cmd_name == "export" {
        return beaudy_builtins::run_export(args);
    } else if cmd_name == "alias" {
        return beaudy_builtins::run_alias(args, aliases);
    } else if cmd_name == "bls" {
        return beaudy_builtins::run_bls(args);
    } else if cmd_name == "bhelp" {
        return beaudy_builtins::run_bhelp();
    } else if cmd_name == "bconfig" {
        return beaudy_builtins::run_bconfig(args);
    } else if cmd_name == "pwd" {
        return beaudy_builtins::run_pwd();
    } else if cmd_name == "bsetup" {
        return beaudy_builtins::run_bsetup();
    } else if cmd_name == "bmemo" {
        return beaudy_builtins::run_bmemo(args);
    } else if cmd_name == "bcalc" {
        return beaudy_builtins::run_bcalc(args);
    } else if cmd_name == "btrash" {
        return beaudy_builtins::run_btrash(args);
    } else if cmd_name == "bhash" {
        return beaudy_builtins::run_bhash(args);
    } else if cmd_name == "cd" {
        return execute_cd(args);
    } else if cmd_name == "pushd" {
        return execute_pushd(args);
    } else if cmd_name == "popd" {
        return execute_popd();
    } else if cmd_name == "dirs" {
        return execute_dirs();
    } else if cmd_name == "clear" || cmd_name == "cls" {
        return beaudy_builtins::run_clear();
    }

    // External command execution via PTY / subprocess
    let pty_system = native_pty_system();
    let (cols, rows) = size().unwrap_or((80, 24));
    let pair = pty_system.openpty(PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    let shell = if default_shell.trim().is_empty() {
        if cfg!(windows) {
            "powershell.exe"
        } else {
            "sh"
        }
    } else {
        default_shell.trim()
    };

    let mut cmd = CommandBuilder::new(shell);
    if let Ok(cwd) = std::env::current_dir() {
        cmd.cwd(cwd);
    }
    if cfg!(windows) && (shell.contains("powershell") || shell.contains("pwsh")) {
        cmd.args(["-Command", input]);
    } else if cfg!(windows) && shell.contains("cmd") {
        cmd.args(["/C", input]);
    } else {
        cmd.args(["-c", input]);
    }

    let mut child = match pair.slave.spawn_command(cmd) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("beaudy: command not found or shell error: {e}");
            return Ok(127);
        }
    };
    drop(pair.slave);
    let master = pair.master;

    let mut pty_reader = match master.try_clone_reader() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("beaudy: PTY reader error: {e}");
            return Ok(1);
        }
    };

    #[allow(clippy::collapsible_if)]
    if !pipe_input.is_empty() {
        if let Ok(mut pty_writer) = master.take_writer() {
            let _ = pty_writer.write_all(pipe_input);
            let _ = pty_writer.flush();
        }
    }

    let is_running = Arc::new(AtomicBool::new(true));

    // Output thread
    let is_running_reader = is_running.clone();
    let output_buf = Arc::new(Mutex::new(Vec::new()));
    let output_buf_clone = output_buf.clone();
    let reader_thread = thread::spawn(move || {
        let mut buf = [0u8; 1024];
        while let Ok(n) = pty_reader.read(&mut buf) {
            if n == 0 {
                break;
            }
            if let Ok(mut guard) = output_buf_clone.lock() {
                guard.extend_from_slice(&buf[..n]);
            }
        }
        is_running_reader.store(false, Ordering::SeqCst);
    });

    let exit_status = loop {
        if let Ok(Some(status)) = child.try_wait() {
            break status;
        }
        thread::sleep(std::time::Duration::from_millis(10));
    };

    // Close master PTY handle so pty_reader receives EOF and reader_thread exits cleanly
    drop(master);

    is_running.store(false, Ordering::SeqCst);
    let _ = reader_thread.join();

    if let Ok(guard) = output_buf.lock() {
        // Normalize isolated \n to \r\n for raw mode compatibility
        let mut normalized = Vec::with_capacity(guard.len());
        let mut prev_was_cr = false;
        for &b in guard.iter() {
            if b == b'\n' && !prev_was_cr {
                normalized.push(b'\r');
            }
            normalized.push(b);
            prev_was_cr = b == b'\r';
        }
        let _ = writer.write_all(&normalized);
        let _ = writer.flush();
    }

    let code = if exit_status.success() {
        0
    } else {
        let c = exit_status.exit_code();
        if c == 0 { 1 } else { c as i32 }
    };
    Ok(code)
}

fn execute_cd(args: &[&str]) -> Result<i32, Box<dyn std::error::Error>> {
    let home_dir = if cfg!(windows) {
        std::env::var("USERPROFILE").ok()
    } else {
        std::env::var("HOME").ok()
    };

    let target = if args.is_empty() {
        home_dir.clone().unwrap_or_else(|| ".".to_string())
    } else {
        args[0].to_string()
    };

    let current_dir = std::env::current_dir()?;
    let target_path = if target == "~" {
        home_dir.unwrap_or_else(|| ".".to_string())
    } else if target == "-" {
        let oldpwd_guard = get_oldpwd().lock().unwrap();
        if let Some(ref path) = *oldpwd_guard {
            path.to_string_lossy().into_owned()
        } else {
            eprintln!("cd: OLDPWD not set");
            return Ok(1);
        }
    } else if target.starts_with("~/") {
        if let Some(home) = home_dir {
            target.replace('~', &home)
        } else {
            target
        }
    } else {
        target
    };

    let target_path_buf = std::path::PathBuf::from(target_path);
    match std::env::set_current_dir(&target_path_buf) {
        Ok(_) => {
            let mut oldpwd_guard = get_oldpwd().lock().unwrap();
            *oldpwd_guard = Some(current_dir);
            if args.first().copied() == Some("-") {
                println!("{}", std::env::current_dir()?.display());
            }
            Ok(0)
        }
        Err(e) => {
            eprintln!("cd: {}", e);
            Ok(1)
        }
    }
}

// ---------- DIRECTORY STACK MANAGEMENT
static DIR_STACK: OnceLock<Mutex<Vec<PathBuf>>> = OnceLock::new();

fn get_dir_stack() -> &'static Mutex<Vec<PathBuf>> {
    DIR_STACK.get_or_init(|| Mutex::new(Vec::new()))
}

fn execute_pushd(args: &[&str]) -> Result<i32, Box<dyn std::error::Error>> {
    let current = std::env::current_dir()?;
    let target = if args.is_empty() {
        if let Some(home) = if cfg!(windows) {
            std::env::var("USERPROFILE").ok()
        } else {
            std::env::var("HOME").ok()
        } {
            home
        } else {
            ".".to_string()
        }
    } else {
        args[0].to_string()
    };

    let target_path = PathBuf::from(target);
    match std::env::set_current_dir(&target_path) {
        Ok(_) => {
            let mut stack = get_dir_stack().lock().unwrap();
            stack.push(current);
            print_dirs(&stack)?;
            Ok(0)
        }
        Err(e) => {
            eprintln!("pushd: {}", e);
            Ok(1)
        }
    }
}

fn execute_popd() -> Result<i32, Box<dyn std::error::Error>> {
    let mut stack = get_dir_stack().lock().unwrap();
    if let Some(prev) = stack.pop() {
        match std::env::set_current_dir(&prev) {
            Ok(_) => {
                print_dirs(&stack)?;
                Ok(0)
            }
            Err(e) => {
                eprintln!("popd: {}", e);
                Ok(1)
            }
        }
    } else {
        eprintln!("popd: directory stack empty");
        Ok(1)
    }
}

fn execute_dirs() -> Result<i32, Box<dyn std::error::Error>> {
    let stack = get_dir_stack().lock().unwrap();
    print_dirs(&stack)?;
    Ok(0)
}

fn print_dirs(stack: &[PathBuf]) -> Result<(), Box<dyn std::error::Error>> {
    let current = std::env::current_dir()?;
    let mut dirs_str = current.display().to_string();
    for dir in stack.iter().rev() {
        dirs_str.push(' ');
        dirs_str.push_str(&dir.display().to_string());
    }
    println!("{}", dirs_str);
    Ok(())
}

// ---------- PIPELINE & REDIRECTION ROUTER
/// Execute pipeline command string.
///
/// ```
/// use beaudy_router::execute_pipeline;
/// use std::collections::HashMap;
/// let aliases = HashMap::new();
/// let res = execute_pipeline("pwd", "sh", &aliases);
/// assert!(res.is_ok());
/// ```
pub fn execute_pipeline(
    input: &str,
    default_shell: &str,
    aliases: &std::collections::HashMap<String, String>,
) -> Result<i32, Box<dyn std::error::Error>> {
    let expanded = expand_env_vars(input.trim());
    let trimmed = expanded.trim();
    if trimmed.is_empty() {
        return Ok(0);
    }

    // Check redirection: > or >>
    let (cmd_str, redirect_target, append_mode) = if let Some(idx) = trimmed.rfind(">>") {
        (trimmed[..idx].trim(), Some(trimmed[idx + 2..].trim()), true)
    } else if let Some(idx) = trimmed.rfind('>') {
        (
            trimmed[..idx].trim(),
            Some(trimmed[idx + 1..].trim()),
            false,
        )
    } else {
        (trimmed, None, false)
    };

    let stages: Vec<&str> = cmd_str.split('|').map(|s| s.trim()).collect();
    let mut current_input: Vec<u8> = Vec::new();
    let mut last_code = 0;

    for (i, stage) in stages.iter().enumerate() {
        let is_last = i == stages.len() - 1;
        let mut output_buf = Vec::new();

        if is_last && let Some(target) = redirect_target {
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(append_mode)
                .truncate(!append_mode)
                .open(target)?;
            last_code = execute_stage_with_input_output(
                stage,
                default_shell,
                aliases,
                &current_input,
                &mut file,
            )?;
        } else if is_last {
            last_code = execute_stage_with_input_output(
                stage,
                default_shell,
                aliases,
                &current_input,
                &mut std::io::stdout(),
            )?;
        } else {
            last_code = execute_stage_with_input_output(
                stage,
                default_shell,
                aliases,
                &current_input,
                &mut output_buf,
            )?;
            current_input = output_buf;
        }
    }

    Ok(last_code)
}

pub fn execute_interactive_command(input: &str) -> Result<i32, Box<dyn std::error::Error>> {
    execute_pipeline(input, "", &std::collections::HashMap::new())
}

// ---------- TESTS
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static CWD_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_cd_traversal() {
        let _guard = CWD_MUTEX.lock().unwrap();
        let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let _ = std::env::set_current_dir(&crate_dir);
        let original_dir = std::env::current_dir().unwrap();

        // "src" should always exist in crate
        let res = execute_interactive_command("cd src");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);

        let new_dir = std::env::current_dir().unwrap();
        assert!(new_dir.ends_with("src"));

        // Go back using "-"
        let res_back = execute_interactive_command("cd -");
        assert!(res_back.is_ok());
        assert_eq!(res_back.unwrap(), 0);

        let back_dir = std::env::current_dir().unwrap();
        assert_eq!(back_dir, original_dir);

        // Restore dir
        let _ = std::env::set_current_dir(original_dir);
    }

    #[test]
    fn test_cd_invalid() {
        let _guard = CWD_MUTEX.lock().unwrap();
        // Test that invalid directory returns error code
        let res = execute_interactive_command("cd /invalid/path/that/does/not/exist/beaudyshell");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1);
    }

    #[test]
    #[allow(unsafe_code)]
    fn test_expand_env_vars() {
        unsafe {
            std::env::set_var("BEAUDY_TEST_VAR", "BeaudyVal");
        }
        let expanded = expand_env_vars("echo $BEAUDY_TEST_VAR and ${BEAUDY_TEST_VAR}");
        assert_eq!(expanded, "echo BeaudyVal and BeaudyVal");
    }

    #[test]
    fn test_pushd_popd_dirs() {
        let _guard = CWD_MUTEX.lock().unwrap();
        let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let _ = std::env::set_current_dir(&crate_dir);
        let original_dir = std::env::current_dir().unwrap();

        let res_push = execute_interactive_command("pushd src");
        assert!(res_push.is_ok());
        assert_eq!(res_push.unwrap(), 0);
        assert!(std::env::current_dir().unwrap().ends_with("src"));

        let res_dirs = execute_interactive_command("dirs");
        assert!(res_dirs.is_ok());

        let res_pop = execute_interactive_command("popd");
        assert!(res_pop.is_ok());
        assert_eq!(res_pop.unwrap(), 0);
        assert_eq!(std::env::current_dir().unwrap(), original_dir);
    }

    #[test]
    fn test_nonexistent_command() {
        let res = execute_pipeline(
            "nonexistent_command_123456789",
            "sh",
            &std::collections::HashMap::new(),
        );
        assert!(res.is_ok());
        assert_ne!(res.unwrap(), 0);
    }

    #[test]
    fn test_invalid_shell() {
        let res = execute_pipeline(
            "ls",
            "nonexistent_shell_xyz_999",
            &std::collections::HashMap::new(),
        );
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 127);
    }

    #[test]
    fn test_clear_builtin() {
        let res_clear = execute_pipeline("clear", "sh", &std::collections::HashMap::new());
        assert!(res_clear.is_ok());
        assert_eq!(res_clear.unwrap(), 0);

        let res_cls = execute_pipeline("cls", "sh", &std::collections::HashMap::new());
        assert!(res_cls.is_ok());
        assert_eq!(res_cls.unwrap(), 0);
    }
}
