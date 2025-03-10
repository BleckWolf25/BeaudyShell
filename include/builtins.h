// builtins.h - Built-in commands
#ifndef BUILTINS_H
#define BUILTINS_H

#include "beaudyShell.h"

int builtin_cd(char **args, ShellState *state);
int builtin_exit(char **args, ShellState *state);
int builtin_pwd(char **args, ShellState *state);
int builtin_help(char **args, ShellState *state);
int builtin_echo(char **args, ShellState *state);

#endif /* BUILTINS_H */
