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
 * @updated 16/07/2026
 */
// ---------- CONSTANTS
const OSC: &str = "\x1b]";
const ST: &str = "\x1b\\";

// ---------- SEMANTIC MARKER FUNCTIONS
/// Emitted before the prompt is printed.
#[must_use]
pub fn prompt_start() -> String {
    format!("{OSC}133;A{ST}")
}

/// Emitted after the prompt, right before the user starts typing.
#[must_use]
pub fn input_start() -> String {
    format!("{OSC}133;B{ST}")
}

/// Emitted after the user presses Enter, before the command output begins.
#[must_use]
pub fn output_start() -> String {
    format!("{OSC}133;C{ST}")
}

/// Emitted after the command finishes executing.
#[must_use]
pub fn command_finished(exit_code: i32) -> String {
    format!("{OSC}133;D;{exit_code}{ST}")
}
