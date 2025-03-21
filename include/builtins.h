// /include/builtins.h - Built-in commands
#pragma once

#include "beaudyshell.h"

/**
 * Built-in commands supported by the shell.
 * Each returns an exit status (0 for success, non-zero for failure).
 */

/**
 * Change the current working directory
 * 
 * @param args Command arguments (args[0] is "cd", args[1] is the directory)
 * @param state Shell state
 * @return Exit status (0 for success, non-zero for failure)
 */
int builtin_cd(char **args, ShellState *state);

/**
 * Exit the shell
 * 
 * @param args Command arguments (args[0] is "exit", args[1] is optional exit code)
 * @param state Shell state
 * @return Exit status to be returned by the shell
 */
int builtin_exit(char **args, ShellState *state);

/**
 * Print the current working directory
 * 
 * @param args Command arguments (args[0] is "pwd")
 * @param state Shell state
 * @return Exit status (0 for success, non-zero for failure)
 */
int builtin_pwd(char **args, ShellState *state);

/**
 * Display help information about built-in commands
 * 
 * @param args Command arguments (args[0] is "help")
 * @param state Shell state
 * @return Always returns 0
 */
int builtin_help(char **args, ShellState *state);

/**
 * Display a line of text
 * 
 * @param args Command arguments (args[0] is "echo", remaining args are text to echo)
 * @param state Shell state
 * @return Always returns 0
 */
int builtin_echo(char **args, ShellState *state);

/**
 * List current jobs
 * 
 * @param args Command arguments (args[0] is "jobs")
 * @param state Shell state
 * @return Exit status (0 for success, non-zero for failure)
 */
int builtin_jobs(char **args, ShellState *state);

/**
 * Bring job to foreground
 * 
 * @param args Command arguments (args[0] is "fg", args[1] is optional job ID)
 * @param state Shell state
 * @return Exit status (0 for success, non-zero for failure)
 */
int builtin_fg(char **args, ShellState *state);

/**
 * Continue job in background
 * 
 * @param args Command arguments (args[0] is "bg", args[1] is optional job ID)
 * @param state Shell state
 * @return Exit status (0 for success, non-zero for failure)
 */
int builtin_bg(char **args, ShellState *state);

/**
 * Check if a command is a built-in command
 * 
 * @param cmd Command name to check
 * @return true if command is built-in, false otherwise
 */
bool is_builtin(const char *cmd);

/**
 * Execute a built-in command
 * 
 * @param args Array of command arguments
 * @param arg_count Number of arguments
 * @param state Current shell state
 * @return Exit status of command
 */
int handle_builtin(char **args, int arg_count, ShellState *state);

/* BUILTINS_H */
