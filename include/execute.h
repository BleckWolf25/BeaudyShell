// /include/execute.h - Command execution
#pragma once

#include "beaudyshell.h"

/**
 * Execute a command with the given arguments
 * 
 * @param args Array of command arguments (args[0] is the command name)
 * @param arg_count Number of arguments in the args array
 * @param state Current shell state
 * @return Exit status of the command (0 for success, non-zero for failure)
 */
int execute_command(char **args, int arg_count, ShellState *state);

/**
 * Handle execution of a built-in command
 * 
 * @param args Array of command arguments (args[0] is the command name)
 * @param arg_count Number of arguments in the args array
 * @param state Current shell state
 * @return Exit status of the built-in command (0 for success, non-zero for failure)
 */
int handle_builtin(char **args, int arg_count, ShellState *state);

/**
 * Check if a command is a built-in command
 * 
 * @param cmd Command name to check
 * @return true if the command is a built-in command, false otherwise
 */
bool is_builtin(char *cmd);

/* EXECUTE_H */
