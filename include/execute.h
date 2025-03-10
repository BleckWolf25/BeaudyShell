// execute.h - Command execution
#ifndef EXECUTE_H
#define EXECUTE_H

#include "beaudyShell.h"

int execute_command(char **args, int arg_count, ShellState *state);
int handle_builtin(char **args, int arg_count, ShellState *state);
bool is_builtin(char *cmd);

#endif /* EXECUTE_H */
