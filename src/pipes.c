// /src/pipes.c - Pipe and redirection handling
#include "pipes.h"
#include "execute.h"
#include "input.h"
#include "jobs.h"
#include "signals.h"
#include "utils.h"
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/wait.h>
#include <unistd.h>

JobList job_list;

// Parse input string into a pipeline structure
Pipeline *parse_pipeline(char *input) {
  if (!input || input[0] == '\0') return NULL;

  Pipeline *pipeline = malloc(sizeof(Pipeline));
  if (!pipeline) {
    perror("malloc");
    return NULL;
  }

  // Initialize pipeline
  *pipeline = (Pipeline){
    .commands = NULL,
    .cmd_count = 0,
    .input_file = NULL,
    .output_file = NULL,
    .append_output = false
  };

  char *input_copy = str_duplicate(input);
  if (!input_copy) {
    free(pipeline);
    return NULL;
  }

  // Split by pipe character
  char *commands[MAX_ARGS] = {0};
  char *cmd_str = strtok(input_copy, "|");
  
  while (cmd_str && pipeline->cmd_count < MAX_ARGS - 1) {
    trim_string(cmd_str);
    commands[pipeline->cmd_count++] = cmd_str;
    cmd_str = strtok(NULL, "|");
  }

  pipeline->commands = malloc(pipeline->cmd_count * sizeof(char **));
  if (!pipeline->commands) {
    perror("malloc");
    free(input_copy);
    free(pipeline);
    return NULL;
  }

  for (int i = 0; i < pipeline->cmd_count; i++) {
    char *cmd = commands[i];
    if (!cmd) continue;

    // Handle input redirection (first command only)
    if (i == 0) {
      char *input_redir = strstr(cmd, "<");
      if (input_redir) {
        *input_redir = '\0';
        char *filename = input_redir + 1;
        trim_string(filename);
        trim_string(cmd); // Trim command after redirection removal
        
        if (*filename) {
          pipeline->input_file = str_duplicate(filename);
        }
      }
    }

    // Handle output redirection (last command only)
    if (i == pipeline->cmd_count - 1) {
      char *output_redir = strstr(cmd, ">>");
      if (output_redir) {
        *output_redir = '\0';
        char *filename = output_redir + 2;
        trim_string(filename);
        trim_string(cmd);
        
        if (*filename) {
          pipeline->output_file = str_duplicate(filename);
          pipeline->append_output = true;
        }
      } else {
        output_redir = strstr(cmd, ">");
        if (output_redir) {
          *output_redir = '\0';
          char *filename = output_redir + 1;
          trim_string(filename);
          trim_string(cmd);
          
          if (*filename) {
            pipeline->output_file = str_duplicate(filename);
            pipeline->append_output = false;
          }
        }
      }
    }

    int arg_count = 0;
    pipeline->commands[i] = parse_input(cmd, &arg_count);
    
    if (!pipeline->commands[i] || arg_count == 0) {
      for (int j = 0; j < i; j++) free_args(pipeline->commands[j]);
      free(pipeline->commands);
      free(input_copy);
      free(pipeline);
      return NULL;
    }
  }

  free(input_copy);
  return pipeline;
}

// Execute a pipeline of commands
int execute_pipeline(Pipeline *pipeline, ShellState *state) {
  if (!pipeline || pipeline->cmd_count == 0) return 1;

  // Handle built-in commands (single command, with or without redirection)
  if (pipeline->cmd_count == 1) {
    char **args = pipeline->commands[0];
    if (is_builtin(args[0])) {
      int saved_stdin = dup(STDIN_FILENO);
      int saved_stdout = dup(STDOUT_FILENO);
      int status = 0;

      // Handle input redirection
      if (pipeline->input_file) {
        int fd = open(pipeline->input_file, O_RDONLY);
        if (fd == -1) {
          perror(pipeline->input_file);
          return 1;
        }
        dup2(fd, STDIN_FILENO);
        close(fd);
      }

      // Handle output redirection
      if (pipeline->output_file) {
        int flags = O_WRONLY | O_CREAT | (pipeline->append_output ? O_APPEND : O_TRUNC);
        int fd = open(pipeline->output_file, flags, 0644);
        if (fd == -1) {
          perror(pipeline->output_file);
          return 1;
        }
        dup2(fd, STDOUT_FILENO);
        close(fd);
      }

      // Execute built-in
      int arg_count = 0;
      while (args[arg_count]) arg_count++;
      status = handle_builtin(args, arg_count, state);

      // Restore standard streams
      dup2(saved_stdin, STDIN_FILENO);
      dup2(saved_stdout, STDOUT_FILENO);
      close(saved_stdin);
      close(saved_stdout);

      return status;
    }
  }

  // Handle external commands and pipelines
  int pipes[MAX_ARGS-1][2];
  for (int i = 0; i < pipeline->cmd_count - 1; i++) {
    if (pipe(pipes[i]) == -1) {
      perror("pipe");
      return 1;
    }
  }

  for (int i = 0; i < pipeline->cmd_count; i++) {
    pid_t pid = fork();
    if (pid == -1) {
      perror("fork");
      return 1;
    }

    // Create a process group for the pipeline
    if (i == 0) {
      // Set process group ID for the first process
      setpgid(pid, pid);
      
      // Add job to job list if in background
      if (background) {
          add_job(&job_list, pid, input); // Store original command
      } else {
          // Give terminal control to foreground process group
          tcsetpgrp(STDIN_FILENO, pid);
      }
    }

    if (pid == 0) { // Child process
      reset_signals();

      // Input redirection for first command
      if (i == 0 && pipeline->input_file) {
        int fd = open(pipeline->input_file, O_RDONLY);
        if (fd == -1) {
          perror(pipeline->input_file);
          exit(EXIT_FAILURE);
        }
        dup2(fd, STDIN_FILENO);
        close(fd);
      }

      // Output redirection for last command
      if (i == pipeline->cmd_count - 1 && pipeline->output_file) {
        int flags = O_WRONLY | O_CREAT | (pipeline->append_output ? O_APPEND : O_TRUNC);
        int fd = open(pipeline->output_file, flags, 0644);
        if (fd == -1) {
          perror(pipeline->output_file);
          exit(EXIT_FAILURE);
        }
        dup2(fd, STDOUT_FILENO);
        close(fd);
      }

      // Connect pipes
      if (i > 0) dup2(pipes[i-1][0], STDIN_FILENO);
      if (i < pipeline->cmd_count - 1) dup2(pipes[i][1], STDOUT_FILENO);

      // Close all pipe ends
      for (int j = 0; j < pipeline->cmd_count - 1; j++) {
        close(pipes[j][0]);
        close(pipes[j][1]);
      }

      execvp(pipeline->commands[i][0], pipeline->commands[i]);
      fprintf(stderr, "%s: command not found\n", pipeline->commands[i][0]);
      exit(EXIT_FAILURE);
    }
  }

  // Parent process
  for (int i = 0; i < pipeline->cmd_count - 1; i++) {
    close(pipes[i][0]);
    close(pipes[i][1]);
  }

  int status = 0;
  for (int i = 0; i < pipeline->cmd_count; i++) {
    wait(&status);
  }

  return WIFEXITED(status) ? WEXITSTATUS(status) : 1;
}

// Free resources allocated for pipeline
void free_pipeline(Pipeline *pipeline) {
  if (!pipeline) return;

  if (pipeline->commands) {
    for (int i = 0; i < pipeline->cmd_count; i++) {
      if (pipeline->commands[i]) free_args(pipeline->commands[i]);
    }
    free(pipeline->commands);
  }

  free(pipeline->input_file);
  free(pipeline->output_file);
  free(pipeline);
}
