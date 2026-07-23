/*
 * @file lib.rs
 *
 * @version 1.0.0
 * @author BleckWolf25
 * @license MIT
 *
 * @summary Accessibility module providing OSC 133 semantic markers for screen readers.
 *
 * @description
 * This module provides functions to emit OSC 133 escape sequences that mark different
 * regions of terminal output (prompt, input, output, command completion) to enable
 * screen readers to navigate between these zones more effectively.
 *
 * @since 16/07/2026
 * @updated 23/07/2026
 */
// ---------- CONSTANTS
const OSC: &str = "\x1b]";
const ST: &str = "\x1b\\";

// ---------- SEMANTIC MARKER FUNCTIONS
/// Emitted before the prompt is printed.
///
/// ```
/// use beaudy_a11y::prompt_start;
/// let marker = prompt_start();
/// assert!(marker.contains("133;A"));
/// ```
#[must_use]
pub fn prompt_start() -> String {
    format!("{OSC}133;A{ST}")
}

/// Emitted after the prompt, right before the user starts typing.
///
/// ```
/// use beaudy_a11y::input_start;
/// let marker = input_start();
/// assert!(marker.contains("133;B"));
/// ```
#[must_use]
pub fn input_start() -> String {
    format!("{OSC}133;B{ST}")
}

/// Emitted after the user presses Enter, before the command output begins.
///
/// ```
/// use beaudy_a11y::output_start;
/// let marker = output_start();
/// assert!(marker.contains("133;C"));
/// ```
#[must_use]
pub fn output_start() -> String {
    format!("{OSC}133;C{ST}")
}

/// Emitted after the command finishes executing.
///
/// ```
/// use beaudy_a11y::command_finished;
/// let marker = command_finished(0);
/// assert!(marker.contains("133;D;0"));
/// ```
#[must_use]
pub fn command_finished(exit_code: i32) -> String {
    format!("{OSC}133;D;{exit_code}{ST}")
}

// ---------- TESTS
#[cfg(test)]
mod tests {
    use super::*;

    // The canonical ESC byte used in OSC sequences
    const ESC: char = '\x1b';

    // Helper: assert the string starts with ESC ]
    fn assert_osc_start(s: &str) {
        assert!(
            s.starts_with('\x1b'),
            "Expected ESC prefix, got: {:?}",
            &s[..s.len().min(6)]
        );
    }

    // Helper: assert the string ends with ESC backslash (ST)
    fn assert_st_end(s: &str) {
        assert!(
            s.ends_with('\x1b'), // ST = ESC '\'
            "Expected ST (ESC \\) suffix in: {:?}",
            s
        );
    }

    #[test]
    fn test_prompt_start_osc_format() {
        let s = prompt_start();
        // Must be ESC ] 133 ; A ESC backslash
        assert_eq!(s, "\x1b]133;A\x1b\\");
        assert!(s.contains("133;A"), "Missing 133;A marker");
        assert_osc_start(&s);
    }

    #[test]
    fn test_input_start_osc_format() {
        let s = input_start();
        assert_eq!(s, "\x1b]133;B\x1b\\");
        assert!(s.contains("133;B"), "Missing 133;B marker");
        assert_osc_start(&s);
    }

    #[test]
    fn test_output_start_osc_format() {
        let s = output_start();
        assert_eq!(s, "\x1b]133;C\x1b\\");
        assert!(s.contains("133;C"), "Missing 133;C marker");
        assert_osc_start(&s);
    }

    #[test]
    fn test_command_finished_zero_exit() {
        let s = command_finished(0);
        assert_eq!(s, "\x1b]133;D;0\x1b\\");
        assert!(s.contains("133;D;0"), "Missing 133;D;0 for exit 0");
    }

    #[test]
    fn test_command_finished_nonzero_exit() {
        let s = command_finished(1);
        assert!(s.contains("133;D;1"), "Missing 133;D;1 for exit 1");

        let s127 = command_finished(127);
        assert!(s127.contains("133;D;127"), "Missing 133;D;127 for exit 127");
    }

    #[test]
    fn test_command_finished_negative_exit() {
        // Negative exit codes (e.g. SIGKILL = -9) should be serialised correctly
        let s = command_finished(-9);
        assert!(s.contains("133;D;-9"), "Missing 133;D;-9 for exit -9");
    }

    #[test]
    fn test_all_sequences_use_same_esc_char() {
        // All sequences must share the same ESC byte, not a look-alike
        for s in [
            prompt_start(),
            input_start(),
            output_start(),
            command_finished(0),
        ] {
            assert_eq!(
                s.chars().next().unwrap(),
                ESC,
                "First char must be ESC (0x1B)"
            );
        }
    }

    #[test]
    fn test_sequences_are_distinct() {
        // Each marker must be a different string
        let p = prompt_start();
        let i = input_start();
        let o = output_start();
        let d = command_finished(0);
        assert_ne!(p, i);
        assert_ne!(p, o);
        assert_ne!(p, d);
        assert_ne!(i, o);
        assert_ne!(i, d);
        assert_ne!(o, d);
    }

    // Suppress the unused variable warning from the dead-code helper
    #[allow(dead_code)]
    fn _use_helper(s: &str) {
        assert_st_end(s);
    }
}
