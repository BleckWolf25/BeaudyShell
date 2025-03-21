#include "execute.h"
#include "jobs.h"
#include "pipes.h"
#include "signals.h"
#include "utils.h"
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/wait.h>
#include <unistd.h>

#define MAX_PIPE_STAGES 10

static int setup_pipe(int pipefd[2]) {
    if (pipe(pipefd) == -1) {
        perror("pipe");
        return -1;
    }
    return 0;
}

static int redirect_io(int fd, int target_fd) {
    if (fd != target_fd) {
        if (dup2(fd, target_fd) == -1) {
            perror("dup2");
            return -1;
        }
        close(fd);
    }
    return 0;
}

static int handle_builtin_with_redirects(Pipeline *pipeline, ShellState *state) {
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
        int flags = O_WRONLY | O_CREAT;
        flags |= pipeline->append_output ? O_APPEND : O_TRUNC;
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
    while (pipeline->commands[0][arg_count]) arg_count++;
    status = handle_builtin(pipeline->commands[0], arg_count, state);

    // Restore standard streams
    dup2(saved_stdin, STDIN_FILENO);
    dup2(saved_stdout, STDOUT_FILENO);
    close(saved_stdin);
    close(saved_stdout);

    return status;
}

static int execute_external_command(char **args, ShellState *state, bool background) {
    char *cmd_path = find_executable(args[0]);
    if (!cmd_path) {
        fprintf(stderr, "%s: command not found\n", args[0]);
        return 127;
    }

    pid_t pid = fork();
    if (pid < 0) {
        perror("fork");
        free(cmd_path);
        return 1;
    }

    if (pid == 0) { // Child process
        reset_signals();

        // Create new process group
        if (setpgid(0, 0) < 0) {
            perror("setpgid");
            exit(EXIT_FAILURE);
        }

        // Give terminal control to foreground process
        if (!background && state->interactive) {
            if (tcsetpgrp(STDIN_FILENO, getpid()) < 0) {
                perror("tcsetpgrp");
                exit(EXIT_FAILURE);
            }
        }

        execv(cmd_path, args);
        perror("execv");
        exit(EXIT_FAILURE);
    }

    // Parent process
    free(cmd_path);

    // Add job to job list
    if (background) {
        char *cmd_str = args[0];
        add_job(state, pid, cmd_str);
        printf("[%d] %d\n", state->jobs->next_job_id - 1, pid);
        return 0;
    }

    // Wait for foreground process
    int status;
    if (waitpid(pid, &status, WUNTRACED) < 0) {
        perror("waitpid");
        return 1;
    }

    // Return terminal control to shell
    if (state->interactive) {
        if (tcsetpgrp(STDIN_FILENO, getpgrp()) < 0) {
            perror("tcsetpgrp");
            return 1;
        }
    }

    if (WIFEXITED(status)) {
        return WEXITSTATUS(status);
    }
    return 1;
}

int execute_pipeline(Pipeline *pipeline, ShellState *state) {
    if (!pipeline || pipeline->cmd_count == 0) return 1;

    // Handle built-in commands in a non-piped context
    if (pipeline->cmd_count == 1 && is_builtin(pipeline->commands[0][0])) {
        return handle_builtin_with_redirects(pipeline, state);
    }

    pid_t pgid = 0;
    int pipes[MAX_PIPE_STAGES][2];

    // Set up pipes
    for (int i = 0; i < pipeline->cmd_count - 1; i++) {
        if (setup_pipe(pipes[i]) == -1) {
            return 1;
        }
    }

    // Execute commands
    for (int i = 0; i < pipeline->cmd_count; i++) {
        pid_t pid = fork();
        if (pid == -1) {
            perror("fork");
            return 1;
        }

        if (pid == 0) { // Child process
            reset_signals();

            // Set process group
            if (pgid == 0) {
                pgid = getpid();
            }
            setpgid(0, pgid);

            // Handle redirections
            if (i == 0 && pipeline->input_file) {
                // Handle input redirection
                // ...existing input redirection code...
            }

            if (i == pipeline->cmd_count - 1 && pipeline->output_file) {
                // Handle output redirection
                // ...existing output redirection code...
            }

            // Set up pipes
            if (i > 0) {
                if (redirect_io(pipes[i-1][0], STDIN_FILENO) == -1) {
                    exit(EXIT_FAILURE);
                }
            }
            if (i < pipeline->cmd_count - 1) {
                if (redirect_io(pipes[i][1], STDOUT_FILENO) == -1) {
                    exit(EXIT_FAILURE);
                }
            }

            // Close unused pipe ends
            // ...existing pipe cleanup code...

            execvp(pipeline->commands[i][0], pipeline->commands[i]);
            perror(pipeline->commands[i][0]);
            exit(EXIT_FAILURE);
        }

        // Parent process
        if (pgid == 0) {
            pgid = pid;
        }
        setpgid(pid, pgid);

        // Close pipe ends in parent
        if (i > 0) {
            close(pipes[i-1][0]);
            close(pipes[i-1][1]);
        }
    }

    // Wait for pipeline completion
    if (!pipeline->background) {
        // Wait for all processes in pipeline
        int status;
        pid_t pid;
        
        while ((pid = waitpid(-pgid, &status, WUNTRACED)) > 0) {
            if (WIFSTOPPED(status)) {
                update_job_status(state, pgid, JOB_STOPPED);
                break;
            }
        }

        // Return terminal control to shell
        if (state->interactive) {
            tcsetpgrp(STDIN_FILENO, getpgrp());
        }

        status = WIFEXITED(status) ? WEXITSTATUS(status) : 1;
        return status;
    } else {
        // Background execution
        add_job(state, pgid, pipeline->original_command);
        printf("[%d] %d\n", state->jobs->next_job_id - 1, pgid);
        return 0;
    }
}

int execute_command(char **args, int arg_count, ShellState *state) {
    if (!args || arg_count == 0 || !args[0]) {
        return 0;
    }

    // Check for background execution
    bool background = false;
    if (arg_count > 0 && strcmp(args[arg_count - 1], "&") == 0) {
        background = true;
        args[--arg_count] = NULL;
    }

    // Handle built-in commands
    if (is_builtin(args[0])) {
        return handle_builtin(args, arg_count, state);
    }

    return execute_external_command(args, state, background);
}
