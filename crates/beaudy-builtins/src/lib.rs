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
 * @updated 16/07/2026
 */
// ---------- MODULE EXPORTS
pub mod builtins;

pub use builtins::{run_bhelp, run_bls};

// ---------- TESTS
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bhelp() {
        // Verify help command executes successfully
        let result = run_bhelp();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_bls_current_dir() {
        // Test that bls on current directory returns success
        let result = run_bls(&[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_bls_invalid_dir() {
        // Test that bls on non-existent directory returns failure code
        let result = run_bls(&["/non/existent/path/for/beaudyshell/test"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }
}
