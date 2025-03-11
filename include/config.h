// /include/config.h - Configuration options for the shell
#pragma once

/**
 * Feature flags - Control which features are enabled in the shell
 */
#define ENABLE_JOB_CONTROL 1        // Enable job control features
#define ENABLE_COMMAND_HISTORY 1    // Enable command history
#define ENABLE_TAB_COMPLETION 0     // Enable tab completion (not yet implemented)
#define ENABLE_GLOB_EXPANSION 1     // Enable glob pattern expansion
#define ENABLE_VARIABLE_EXPANSION 1 // Enable environment variable expansion

/**
 * Customization options
 */
#define MAX_HISTORY_SIZE 1000          // Maximum history entries
#define DEFAULT_PROMPT "\\u@\\h:\\w$ " // Default prompt format

/**
 * Color settings for the prompt
 */
#define COLOR_USERNAME "\033[1;32m" // Bold green
#define COLOR_HOSTNAME "\033[1;32m" // Bold green
#define COLOR_PATH "\033[1;34m"     // Bold blue
#define COLOR_RESET "\033[0m"       // Reset color

/* CONFIG_H */
