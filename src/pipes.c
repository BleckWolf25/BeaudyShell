// pipes.c - Pipe and redirection handling
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <sys/wait.h>
#include "pipes.h"
#include "execute.h"
#include "signals.h"
#include "utils.h"

// Parse input string into a pipeline structure
Pipeline *parse_pipeline(char *input) {
    if (!input || input[0] == '\0') {
        return NULL;
    }
    
    Pipeline *pipeline = malloc(sizeof(Pipeline));
    if (!pipeline) {
        perror("malloc");
        return NULL;
    }
    
    // Initialize pipeline
    pipeline->commands = NULL;
    pipeline->cmd_count = 0;
    pipeline->input_file = NULL;
    pipeline->output_file = NULL;
    pipeline->append_output = false;
    
    // Make a copy of the input since we'll be modifying it
    char *input_copy = str_duplicate(input);
    if (!input_copy) {
        free(pipeline);
        return NULL;
    }
    
    // Split by pipe character
    char *commands[MAX_ARGS] = {0};
    char *cmd_str = strtok(input_copy, "|");
    
    while (cmd_str != NULL && pipeline->cmd_count < MAX_ARGS - 1) {
        trim_string(cmd_str);
        commands[pipeline->cmd_count++] = cmd_str;
        cmd_str = strtok(NULL, "|");
    }
    
    // Allocate command array
    pipeline->commands = malloc(pipeline->cmd_count * sizeof(char **));
    if (!pipeline->commands) {
        perror("malloc");
        free(input_copy);
        free(pipeline);
        return NULL;
    }
    
    // Parse each command
    for (int i = 0; i < pipeline->cmd_count; i++) {
        char *cmd = commands[i];
        
        // Check for input redirection (only valid for first command)
        if (i == 0) {
            char *input_redir = strstr(cmd, "<");
            if (input_redir) {
                // Null-terminate the command at redirection symbol
                *input_redir = '\0';
                
                // Extract filename and trim whitespace
                char *filename = input_redir + 1;
                trim_string(filename);
                
                if (strlen(filename) > 0) {
                    pipeline->input_file = str_duplicate(filename);
                }
            }
        }
        
        // Check for output redirection (only valid for last command)
        if (i == pipeline->cmd_count - 1) {
            char *output_redir = strstr(cmd, ">>");
            if (output_redir) {
                // Handle append redirection
                *output_redir = '\0';
                
                char *filename = output_redir + 2;
                trim_string(filename);
                
                if (strlen(filename) > 0) {
                    pipeline->output_file = str_duplicate(filename);
                    pipeline->append_output = true;
                }
            } else {
                output_redir = strstr(cmd, ">");
                if (output_redir) {
                    // Handle normal output redirection
                    *output_redir = '\0';
                    
                    char *filename = output_redir + 1;
                    trim_string(filename);
                    
                    if (strlen(filename) > 0) {
                        pipeline->output_file = str_duplicate(filename);
                        pipeline->append_output = false;
                    }
                }
            }
        }
        
        // Parse command into arguments
        int arg_count = 0;
        pipeline->commands[i] = parse_input(cmd, &arg_count);
        
        // If parsing failed, clean up and return NULL
        if (!pipeline->commands[i] || arg_count == 0) {
            // Clean up previously allocated commands
            for (int j = 0; j < i; j++) {
                free_args(pipeline->commands[j]);
            }
            free(pipeline->commands);
            free(input_copy);
            free(pipeline);
            return NULL;
        }
    }
    
    // Free the input copy since we've tokenized it and stored the values
    free(input_copy);
    
    return pipeline;
}

// Execute a pipeline of commands
int execute_pipeline(Pipeline *pipeline, ShellState *state) {
    if (!pipeline || pipeline->cmd_count == 0) {
        return 1;
    }
    
    // Handle built-in commands (only if single command with no redirection)
    if (pipeline->cmd_count == 1 && !pipeline->input_file && !pipeline->output_file) {
        char **args = pipeline->commands[0];
        if (is_builtin(args[0])) {
            int arg_count = 0;
            while (args[arg_count]) arg_count++;
            return handle_builtin(args, arg_count, state);
        }
    }
    
    int status = 0;
    pid_t pid;
    int pipes[MAX_ARGS-1][2]; // Pipes for n-1 connections between n commands
    
    // Create pipes
    for (int i = 0; i < pipeline->cmd_count - 1; i++) {
        if (pipe(pipes[i]) < 0) {
            perror("pipe");
            return 1;
        }
    }
    
    // Fork and execute each command
    for (int i = 0; i < pipeline->cmd_count; i++) {
        pid = fork();
        
        if (pid < 0) {
            perror("fork");
            return 1;
        } else if (pid == 0) {
            // Child process
            
            // Reset signal handlers
            reset_signals();
            
            // Set up input redirection for first command
            if (i == 0 && pipeline->input_file) {
                int fd = open(pipeline->input_file, O_RDONLY);
                if (fd < 0) {
                    perror(pipeline->input_file);
                    exit(EXIT_FAILURE);
                }
                dup2(fd, STDIN_FILENO);
                close(fd);
            }
            
            // Set up output redirection for last command
            if (i == pipeline->cmd_count - 1 && pipeline->output_file) {
                int flags = O_WRONLY | O_CREAT;
                if (pipeline->append_output) {
                    flags |= O_APPEND;
                } else {
                    flags |= O_TRUNC;
                }
                
                int fd = open(pipeline->output_file, flags, 0644);
                if (fd < 0) {
                    perror(pipeline->output_file);
                    exit(EXIT_FAILURE);
                }
                dup2(fd, STDOUT_FILENO);
                close(fd);
            }
            
            // Connect pipes
            if (i > 0) {
                // Connect input from previous pipe
                dup2(pipes[i-1][0], STDIN_FILENO);
            }
            
            if (i < pipeline->cmd_count - 1) {
                // Connect output to next pipe
                dup2(pipes[i][1], STDOUT_FILENO);
            }
            
            // Close all pipe fds
            for (int j = 0; j < pipeline->cmd_count - 1; j++) {
                close(pipes[j][0]);
                close(pipes[j][1]);
            }
            
            // Execute the command
            execvp(pipeline->commands[i][0], pipeline->commands[i]);
            
            // If we get here, execvp failed
            fprintf(stderr, "%s: command not found\n", pipeline->commands[i][0]);
            exit(EXIT_FAILURE);
        }
    }
    
    // Parent process - close all pipe fds
    for (int i = 0; i < pipeline->cmd_count - 1; i++) {
        close(pipes[i][0]);
        close(pipes[i][1]);
    }
    
    // Wait for all children
    for (int i = 0; i < pipeline->cmd_count; i++) {
        wait(&status);
    }
    
    // Return the exit status of the last command
    if (WIFEXITED(status)) {
        return WEXITSTATUS(status);
    } else {
        return 1;
    }
}

// Free resources allocated for pipeline
void free_pipeline(Pipeline *pipeline) {
    if (!pipeline) return;
    
    if (pipeline->commands) {
        for (int i = 0; i < pipeline->cmd_count; i++) {
            if (pipeline->commands[i]) {
                free_args(pipeline->commands[i]);
            }
        }
        free(pipeline->commands);
    }
    
    if (pipeline->input_file) {
        free(pipeline->input_file);
    }
    
    if (pipeline->output_file) {
        free(pipeline->output_file);
    }
    
    free(pipeline);
}