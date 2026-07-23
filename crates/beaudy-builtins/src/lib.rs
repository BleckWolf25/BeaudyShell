/*
 * @file lib.rs
 *
 * @version 1.0.0
 * @author BleckWolf25
 * @license MIT
 *
 * @summary Public API for the beaudy-builtins crate.
 *
 * @description
 * This module exports the built-in command functions (run_bls and run_bhelp)
 * and provides unit tests to verify their functionality for the BeaudyShell project.
 *
 * @since 16/07/2026
 * @updated 23/07/2026
 */
// ---------- MODULE EXPORTS
pub mod builtins;

pub use builtins::{
    run_alias, run_bcalc, run_bconfig, run_bhash, run_bhelp, run_bls, run_bmemo, run_bsetup,
    run_btrash, run_export, run_pwd,
};

// ---------- TESTS
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    // ── bhelp ─────────────────────────────────────────────────────────────
    #[test]
    fn test_bhelp_returns_zero() {
        let result = run_bhelp();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    // ── bls ───────────────────────────────────────────────────────────────
    #[test]
    fn test_bls_current_dir() {
        let result = run_bls(&[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_bls_invalid_dir() {
        let result = run_bls(&["/non/existent/path/for/beaudyshell/test"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_bls_not_a_dir() {
        // Passing a file path where a directory is expected should return exit code 1
        let result = run_bls(&["Cargo.toml"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_bls_show_all_flag() {
        // -a / --all should succeed without error
        let result = run_bls(&["-a"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        let result2 = run_bls(&["--all"]);
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), 0);
    }

    #[test]
    fn test_bls_recursive_flag() {
        // -R / --recursive should succeed without error
        let result = run_bls(&["-R"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        let result2 = run_bls(&["--recursive"]);
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), 0);
    }

    #[test]
    fn test_bls_combined_flags() {
        let result = run_bls(&["-a", "-R"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    // ── format_size (internal helper) ─────────────────────────────────────
    #[test]
    fn test_format_size_directory() {
        assert_eq!(builtins::format_size(0, true), "-");
        assert_eq!(builtins::format_size(1_000_000, true), "-");
    }

    #[test]
    fn test_format_size_bytes() {
        assert_eq!(builtins::format_size(0, false), "0 B");
        assert_eq!(builtins::format_size(512, false), "512 B");
        assert_eq!(builtins::format_size(1023, false), "1023 B");
    }

    #[test]
    fn test_format_size_kilobytes() {
        // 1024 bytes = 1.0 KB
        assert_eq!(builtins::format_size(1024, false), "1.0 KB");
        // 2048 bytes = 2.0 KB
        assert_eq!(builtins::format_size(2048, false), "2.0 KB");
    }

    #[test]
    fn test_format_size_megabytes() {
        // 1 MB = 1048576 bytes
        assert_eq!(builtins::format_size(1024 * 1024, false), "1.0 MB");
    }

    // ── format_time (relative time helper) ────────────────────────────────
    #[test]
    fn test_format_time_just_now() {
        let t = SystemTime::now() - Duration::from_secs(10);
        assert_eq!(builtins::format_time(t), "just now");
    }

    #[test]
    fn test_format_time_minutes_ago() {
        let t = SystemTime::now() - Duration::from_secs(120);
        assert_eq!(builtins::format_time(t), "2m ago");
    }

    #[test]
    fn test_format_time_hours_ago() {
        let t = SystemTime::now() - Duration::from_secs(7200);
        assert_eq!(builtins::format_time(t), "2h ago");
    }

    #[test]
    fn test_format_time_days_ago() {
        let t = SystemTime::now() - Duration::from_secs(86400 * 3);
        assert_eq!(builtins::format_time(t), "3d ago");
    }

    // ── format_datetime (absolute date helper) ────────────────────────────
    #[test]
    fn test_format_datetime_epoch() {
        // UNIX epoch = 1970-01-01 00:00
        let t = UNIX_EPOCH;
        let s = builtins::format_datetime(t);
        assert_eq!(s, "1970-01-01 00:00");
    }

    #[test]
    fn test_format_datetime_known_date() {
        // 1753191300 seconds = 2025-07-22 13:35 UTC
        let secs: u64 = 1753191300;
        let t = UNIX_EPOCH + Duration::from_secs(secs);
        let s = builtins::format_datetime(t);
        assert_eq!(s, "2025-07-22 13:35");
    }

    #[test]
    fn test_format_datetime_matches_pattern() {
        // Whatever the current time, the output must be "YYYY-MM-DD HH:MM"
        let t = SystemTime::now();
        let s = builtins::format_datetime(t);
        assert_eq!(s.len(), 16, "Expected 16 chars, got: {}", s);
        assert_eq!(&s[4..5], "-");
        assert_eq!(&s[7..8], "-");
        assert_eq!(&s[10..11], " ");
        assert_eq!(&s[13..14], ":");
    }

    // ── bcalc ─────────────────────────────────────────────────────────────
    #[test]
    fn test_bcalc_valid_expression() {
        let result = run_bcalc(&["2", "+", "2"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_bcalc_complex_expression() {
        let result = run_bcalc(&["(3", "*", "4)", "/", "2"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_bcalc_no_args() {
        let result = run_bcalc(&[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_bcalc_invalid_expression() {
        let result = run_bcalc(&["not_a_number"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    // ── bhash ─────────────────────────────────────────────────────────────
    #[test]
    fn test_bhash_md5() {
        let result = run_bhash(&["--md5", "hello"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_bhash_sha256() {
        let result = run_bhash(&["--sha256", "hello"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_bhash_default_md5() {
        // Without a flag, defaults to MD5
        let result = run_bhash(&["hello"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_bhash_no_args() {
        let result = run_bhash(&[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_bhash_flag_without_target() {
        let result = run_bhash(&["--sha256"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    // ── bmemo ─────────────────────────────────────────────────────────────
    #[test]
    fn test_bmemo_save_and_list() {
        let save_res = run_bmemo(&["unit", "test", "note"]);
        assert!(save_res.is_ok());
        assert_eq!(save_res.unwrap(), 0);

        let list_res = run_bmemo(&["list"]);
        assert!(list_res.is_ok());
        assert_eq!(list_res.unwrap(), 0);

        // --list variant
        let list_res2 = run_bmemo(&["--list"]);
        assert!(list_res2.is_ok());
        assert_eq!(list_res2.unwrap(), 0);
    }

    #[test]
    fn test_bmemo_no_args_returns_error() {
        let result = run_bmemo(&[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    // ── btrash ────────────────────────────────────────────────────────────
    #[test]
    fn test_btrash_list() {
        let list_res = run_btrash(&["list"]);
        assert!(list_res.is_ok());
        assert_eq!(list_res.unwrap(), 0);
    }

    #[test]
    fn test_btrash_list_long_flag() {
        let list_res = run_btrash(&["--list"]);
        assert!(list_res.is_ok());
        assert_eq!(list_res.unwrap(), 0);
    }

    #[test]
    fn test_btrash_restore_nonexistent_returns_error() {
        let result = run_btrash(&["restore", "does_not_exist_beaudyshell.txt"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_btrash_restore_no_filename_returns_error() {
        let result = run_btrash(&["restore"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_btrash_no_args_returns_error() {
        let result = run_btrash(&[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_btrash_nonexistent_file_move() {
        // Moving a file that doesn't exist should complete (skipping the file) but return 0
        let result = run_btrash(&["does_not_exist_beaudyshell.txt"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    // ── bsetup ────────────────────────────────────────────────────────────
    #[test]
    fn test_bsetup_returns_zero() {
        let result = run_bsetup();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    // ── pwd ───────────────────────────────────────────────────────────────
    #[test]
    fn test_pwd_returns_zero() {
        let result = run_pwd();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    // ── export ────────────────────────────────────────────────────────────
    #[test]
    #[allow(unsafe_code)]
    fn test_export_set_and_read() {
        unsafe {
            std::env::set_var("BEAUDY_EXPORT_TEST", "initial");
        }
        let result = run_export(&["BEAUDY_EXPORT_TEST=new_value"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        assert_eq!(std::env::var("BEAUDY_EXPORT_TEST").unwrap(), "new_value");
    }

    #[test]
    fn test_export_no_args_lists_env() {
        let result = run_export(&[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }
}
