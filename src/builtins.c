// builtins.c - Built-in command implementations
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include "builtins.h"

int builtin_cd(char **args, ShellState *state) {
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

int builtin_pwd(char **args, ShellState *state) {
    char cwd[MAX_PATH_LEN];
    
    if (getcwd(cwd, sizeof(cwd)) != NULL) {
        printf("%s\n", cwd);
        return 0;
    } else {
        perror("pwd");
        return 1;
    }
}

int builtin_help(char **args, ShellState *state) {
    printf("BeaudyShell - A fast and modern Unix shell\n");
    printf("Built-in commands:\n");
    printf("  cd [dir]      Change the current directory\n");
    printf("  exit [n]      Exit the shell with status n\n");
    printf("  pwd           Print the current working directory\n");
    printf("  help          Display this help message\n");
    printf("  echo [arg...] Display a line of text\n");
    return 0;
}

int builtin_echo(char **args, ShellState *state) {
    // Start from args[1] to skip the "echo" command itself
    for (int i = 1; args[i] != NULL; i++) {
        printf("%s", args[i]);
        if (args[i+1] != NULL) {
            printf(" ");
        }
    }
    printf("\n");
    return 0;
}