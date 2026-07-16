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
 * @updated 16/07/2026
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

// ---------- COMMAND ROUTING
pub fn execute_interactive_command(input: &str) -> Result<i32, Box<dyn std::error::Error>> {
    let trimmed = input.trim();
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(0);
    }

    let cmd_name = parts[0];
    let args = &parts[1..];

    // ---------- BUILTIN COMMANDS
    if cmd_name == "bls" {
        return beaudy_builtins::run_bls(args);
    } else if cmd_name == "bhelp" {
        return beaudy_builtins::run_bhelp();
    } else if cmd_name == "cd" {
        // Determine home directory based on operating system
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

        // Expand path shortcuts: ~, -, ~/prefix
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
                return Ok(0);
            }
            Err(e) => {
                eprintln!("cd: {}", e);
                return Ok(1);
            }
        }
    }

    // ---------- EXTERNAL COMMAND EXECUTION VIA PTY
    let pty_system = native_pty_system();

    // Get the actual terminal size so full-screen apps like vim render correctly
    let (cols, rows) = size().unwrap_or((80, 24));

    let pair = pty_system.openpty(PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    // Determine target shell based on operating system
    let shell = if cfg!(windows) {
        "powershell.exe"
    } else {
        "sh"
    };
    let mut cmd = CommandBuilder::new(shell);
    if let Ok(cwd) = std::env::current_dir() {
        cmd.cwd(cwd);
    }
    if cfg!(windows) {
        cmd.args(["-Command", input]);
    } else {
        cmd.args(["-c", input]);
    }

    let mut child = pair.slave.spawn_command(cmd)?;
    drop(pair.slave);

    let mut reader = pair.master.try_clone_reader()?;
    let mut writer = pair.master.take_writer()?;

    let is_running = Arc::new(AtomicBool::new(true));

    // ---------- OUTPUT THREAD: PTY -> Stdout
    let is_running_reader = is_running.clone();
    let reader_thread = thread::spawn(move || {
        let mut buf = [0u8; 1024];
        let mut stdout = std::io::stdout();

        while let Ok(n) = reader.read(&mut buf) {
            if n == 0 {
                break;
            }
            if stdout.write_all(&buf[..n]).is_err() {
                break;
            }
            if stdout.flush().is_err() {
                break;
            }
        }
        is_running_reader.store(false, Ordering::SeqCst);
    });

    // ---------- INPUT THREAD: Stdin -> PTY
    let is_running_writer = is_running.clone();
    let writer_thread = thread::spawn(move || {
        let mut stdin = std::io::stdin();
        let mut buf = [0u8; 256];

        #[cfg(unix)]
        use std::os::unix::io::AsRawFd;
        #[cfg(unix)]
        let stdin_fd = stdin.as_raw_fd();

        while is_running_writer.load(Ordering::SeqCst) {
            #[cfg(unix)]
            {
                // Use libc::poll to check for raw bytes without blocking
                let mut pfd = libc::pollfd {
                    fd: stdin_fd,
                    events: libc::POLLIN,
                    revents: 0,
                };

                // Poll with a 10ms timeout
                let res = unsafe { libc::poll(&mut pfd, 1, 10) };

                #[allow(clippy::collapsible_if)]
                if res > 0 && (pfd.revents & libc::POLLIN) != 0 {
                    if let Ok(n) = stdin.read(&mut buf) {
                        if n == 0 {
                            break;
                        }
                        if writer.write_all(&buf[..n]).is_err() {
                            break;
                        }
                    }
                }
            }

            #[cfg(not(unix))]
            {
                // Windows fallback
                #[allow(clippy::collapsible_if)]
                if let Ok(true) = crossterm::event::poll(std::time::Duration::from_millis(10)) {
                    if let Ok(n) = stdin.read(&mut buf) {
                        if n == 0 {
                            break;
                        }
                        if writer.write_all(&buf[..n]).is_err() {
                            break;
                        }
                    }
                }
            }
        }
    });

    // Wait for the child process to finish
    let exit_status = child.wait()?;

    // Signal threads to shut down
    is_running.store(false, Ordering::SeqCst);

    // Wait for the I/O threads to finish cleanly
    let _ = reader_thread.join();
    let _ = writer_thread.join();

    // Return 0 for success, 1 for failure (portable-pty's ExitStatus provides `success()`)
    let code = if exit_status.success() { 0 } else { 1 };
    Ok(code)
}

// ---------- TESTS
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cd_traversal() {
        let original_dir = std::env::current_dir().unwrap();

        // Create a temporary directory or just go to a known subfolder
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

        // Restore dir just in case
        let _ = std::env::set_current_dir(original_dir);
    }

    #[test]
    fn test_cd_invalid() {
        // Test that invalid directory returns error code
        let res = execute_interactive_command("cd /invalid/path/that/does/not/exist/beaudyshell");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1);
    }
}
