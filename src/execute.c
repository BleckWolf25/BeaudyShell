// execute.c - Command execution
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/wait.h>
#include <errno.h>
#include "execute.h"
#include "builtins.h"
#include "signals.h"

// List of built-in commands
const char *builtin_commands[] = {
    "cd", "exit", "pwd", "help", "echo", NULL
};

// Array of function pointers to built-in command handlers
int (*builtin_functions[])(char **, ShellState *) = {
    &builtin_cd, &builtin_exit, &builtin_pwd, &builtin_help, &builtin_echo
};

bool is_builtin(char *cmd) {
    if (!cmd) return false;
    
    for (int i = 0; builtin_commands[i] != NULL; i++) {
        if (strcmp(cmd, builtin_commands[i]) == 0) {
            return true;
        }
    }
    return false;
}

int handle_builtin(char **args, int arg_count, ShellState *state) {
    if (!args || arg_count == 0 || !args[0]) {
        return 0;
    }
    
    for (int i = 0; builtin_commands[i] != NULL; i++) {
        if (strcmp(args[0], builtin_commands[i]) == 0) {
            return builtin_functions[i](args, state);
        }
    }
    
    return 1; // Command not found
}

int execute_command(char **args, int arg_count, ShellState *state) {
    if (!args || arg_count == 0 || !args[0]) {
        return 0;
    }
    
    // Check if it's a built-in command
    if (is_builtin(args[0])) {
        return handle_builtin(args, arg_count, state);
    }
    
    pid_t pid;
    int status;
    
    pid = fork();
    
    if (pid < 0) {
        perror("fork");
        return -1;
    } else if (pid == 0) {
        // Child process
        
        // Reset signal handlers to default behavior
        reset_signals();
        
        // Execute the command
        execvp(args[0], args);
        
        // If we get here, execvp failed
        fprintf(stderr, "%s: command not found\n", args[0]);
        exit(EXIT_FAILURE);
    } else {
        // Parent process
        
        // Wait for the child to terminate
        waitpid(pid, &status, 0);
        
        if (WIFEXITED(status)) {
            return WEXITSTATUS(status);
        } else {
            return 1; // Default non-zero return for abnormal termination
        }
    }
}