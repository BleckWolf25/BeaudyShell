// /src/job_builtins.c - Job control built-in commands
#include "beaudyshell.h"
#include "jobs.h"
#include <limits.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <termios.h>
#include <errno.h>

// fg - Bring job to foreground
int builtin_fg(char **args, ShellState *state) {
    if (!state || !state->jobs) {
        fprintf(stderr, "fg: job control not available\n");
        return 1;
    }
    
    // If no argument, use most recent stopped job, or running background job
    Job *job = NULL;
    if (args[1] == NULL) {
        // First try to find a stopped job
        job = find_recent_job_by_status(state, JOB_STOPPED);
        
        // If no stopped job, try a running job
        if (!job) {
            job = find_recent_job_by_status(state, JOB_RUNNING);
        }
        
        if (!job) {
            fprintf(stderr, "fg: no current job\n");
            return 1;
        }
    } else {
        // Parse job ID from argument (may be prefixed with %)
        char *id_str = args[1];
        if (id_str[0] == '%') {
            id_str++;
        }
        
        // Check if the argument is a valid number
        char *endptr;
        long job_id = strtol(id_str, &endptr, 10);
        if (*endptr != '\0' || job_id <= 0 || job_id > INT_MAX) {
            fprintf(stderr, "fg: invalid job ID: %s\n", args[1]);
            return 1;
        }
        
        // Find the job
        job = find_job(state, (int)job_id);
        if (!job) {
            fprintf(stderr, "fg: job %ld not found\n", job_id);
            return 1;
        }
    }

    // Continue the process group if it was stopped
    if (job->status == JOB_STOPPED) {
        if (killpg(job->pgid, SIGCONT) < 0) {
            perror("killpg");
            return 1;
        }
    }

    job->status = JOB_RUNNING;

    // Put the job in the foreground
    if (tcsetpgrp(STDIN_FILENO, job->pgid) < 0) {
        perror("tcsetpgrp");
        return 1;
    }

    // Print the command
    printf("%s\n", job->command);

    // Wait for the job to complete or stop
    int status;
    pid_t pid;
    
    // Wait for any process in the job's process group
    while ((pid = waitpid(-job->pgid, &status, WUNTRACED)) > 0) {
        if (WIFSTOPPED(status)) {
            // Job was stopped
            update_job_status(state, job->pgid, JOB_STOPPED);
            break;
        } else if (WIFEXITED(status) || WIFSIGNALED(status)) {
            // Job terminated
            update_job_status(state, job->pgid, JOB_DONE);
        }
    }
    
    // Check for errors other than ECHILD (no child processes)
    if (pid < 0 && errno != ECHILD) {
        perror("waitpid");
    }

    // Put the shell back in the foreground
    if (tcsetpgrp(STDIN_FILENO, getpgrp()) < 0) {
        perror("tcsetpgrp");
        return 1;
    }

    return 0;
}

// bg - Continue job in background
int builtin_bg(char **args, ShellState *state) {
    if (!state || !state->jobs) {
        fprintf(stderr, "bg: job control not available\n");
        return 1;
    }
    
    // If no argument, use most recent stopped job
    Job *job = NULL;
    if (args[1] == NULL) {
        // Find most recent stopped job
        job = find_recent_job_by_status(state, JOB_STOPPED);
        
        if (!job) {
            fprintf(stderr, "bg: no stopped jobs\n");
            return 1;
        }
    } else {
        // Parse job ID from argument
        char *id_str = args[1];
        if (id_str[0] == '%') {
            id_str++;
        }
        
        // Check if the argument is a valid number
        char *endptr;
        long job_id = strtol(id_str, &endptr, 10);
        if (*endptr != '\0' || job_id <= 0 || job_id > INT_MAX) {
            fprintf(stderr, "bg: invalid job ID: %s\n", args[1]);
            return 1;
        }
        
        // Find the job
        job = find_job(state, (int)job_id);
        if (!job) {
            fprintf(stderr, "bg: job %ld not found\n", job_id);
            return 1;
        }
    }

    // Verify job is stopped
    if (job->status != JOB_STOPPED) {
        fprintf(stderr, "bg: job %d is not stopped\n", job->job_id);
        return 1;
    }

    // Continue the process group
    if (killpg(job->pgid, SIGCONT) < 0) {
        perror("killpg");
        return 1;
    }

    job->status = JOB_RUNNING;
    printf("[%d] %s &\n", job->job_id, job->command);

    return 0;
}

// jobs - List jobs
int builtin_jobs(char **args, ShellState *state) {
    (void)args; // Suppress unused parameter warning
    
    if (!state || !state->jobs) {
        fprintf(stderr, "jobs: job control not available\n");
        return 1;
    }
    
    print_jobs(state);
    return 0;
}
