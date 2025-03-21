// /src/builtins.c - Built-in command implementations
#include "beaudyshell.h"
#include "builtins.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

// Add external declarations for job control builtins
extern int builtin_jobs(char **args, ShellState *state);
extern int builtin_fg(char **args, ShellState *state);
extern int builtin_bg(char **args, ShellState *state);

// CD command
int builtin_cd(char **args, ShellState *state __attribute__((unused))) {

  if (args[1] == NULL) {
    // If no argument is provided, change to HOME directory
    char *home_dir = getenv("HOME");
    if (home_dir == NULL) {
      fprintf(stderr, "cd: HOME not set\n");
      return 1;
    }
    if (chdir(home_dir) != 0) {
      perror("cd");
      return 1;
    }
  } else {
    if (chdir(args[1]) != 0) {
      perror("cd");
      return 1;
    }
  }
  return 0;
}

int builtin_exit(char **args, ShellState *state) {
  int exit_status = 0;

  if (args[1] != NULL) {
    // If argument provided, use it as exit status
    exit_status = atoi(args[1]);
  }

  state->running = false;
  state->last_exit_code = exit_status;
  return exit_status;
}

// PWD command
int builtin_pwd(char **args __attribute__((unused)),
                ShellState *state __attribute__((unused))) {
  char cwd[MAX_PATH_LEN];

  if (getcwd(cwd, sizeof(cwd)) != NULL) {
    printf("%s\n", cwd);
    return 0;
  } else {
    perror("pwd");
    return 1;
  }
}

// Help command
int builtin_help(char **args __attribute__((unused)),
                 ShellState *state __attribute__((unused))) {
  printf("BeaudyShell - A fast and modern Unix shell\n");
  printf("Built-in commands:\n");
  printf("  cd [dir]      Change the current directory\n");
  printf("  exit [n]      Exit the shell with status n\n");
  printf("  pwd           Print the current working directory\n");
  printf("  help          Display this help message\n");
  printf("  echo [arg...] Display a line of text\n");
  return 0;
}

// Echo command
int builtin_echo(char **args, ShellState *state __attribute__((unused))) {
  // Start from args[1] to skip the "echo" command itself
  for (int i = 1; args[i] != NULL; i++) {
    printf("%s", args[i]);
    if (args[i + 1] != NULL) {
      printf(" ");
    }
  }
  printf("\n");
  return 0;
}

bool is_builtin(const char *cmd) {
    static const char *builtins[] = {
        "cd", "exit", "pwd", "help", "echo", "jobs", "fg", "bg", NULL
    };
    
    for (const char **builtin = builtins; *builtin; builtin++) {
        if (strcmp(cmd, *builtin) == 0) {
            return true;
        }
    }
    return false;
}

int handle_builtin(char **args, int arg_count, ShellState *state) {
    if (!args || !args[0]) return 1;

    struct {
        const char *name;
        int (*func)(char **args, ShellState *state);
    } builtin_commands[] = {
        {"cd", builtin_cd},
        {"exit", builtin_exit},
        {"pwd", builtin_pwd},
        {"help", builtin_help},
        {"echo", builtin_echo},
        {"jobs", builtin_jobs},
        {"fg", builtin_fg},
        {"bg", builtin_bg},
        {NULL, NULL}
    };

    for (int i = 0; builtin_commands[i].name != NULL; i++) {
        if (strcmp(args[0], builtin_commands[i].name) == 0) {
            return builtin_commands[i].func(args, state);
        }
    }
    
    return 1;
}
