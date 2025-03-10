// job_builtins.c - Job control built-in commands
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <signal.h>
#include "BeaudyShell.h"
#include "jobs.h"

// External job list
extern JobList job_list;

// fg - Bring job to foreground
int builtin_fg(char **args, ShellState *state) {
    // If no argument, use most recent job
    int job_id;
    if (args[1] == NULL) {
        if (job_list.first_job == NULL) {
            fprintf(stderr, "fg: no current job\n");
            return 1;
        }
        // Find highest job ID
        Job *job = job_list.first_job;
        job_id = job->job_id;
        
        while (job != NULL) {
            if (job->job_id > job_id) {
                job_id = job->job_id;
            }
            job = job->next;
        }
    } else {
        // Parse job ID from argument (may be prefixed with %)
        char *id_str = args[1];
        if (id_str[0] == '%') {
            id_str++;
        }
        job_id = atoi(id_str);
    }
    
    // Find the job
    Job *job = find_job(&job_list, job_id);
    if (!job) {
        fprintf(stderr, "fg: job %d not found\n", job_id);
        return 1;
    }
    
    // Continue the process group
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
    
    // Wait for the job to complete or stop
    int status;
    pid_t pid;
    
    printf("%s\n", job->command);
    
    while ((pid = waitpid(-job->pgid, &status, WUNTRACED)) > 0) {
        if (WIFSTOPPED(status)) {
            // Job was stopped
            update_job_status(&job_list, job->pgid, JOB_STOPPED);
            break;
        } else if (WIFEXITED(status) || WIFSIGNALED(status)) {
            // Job terminated
            update_job_status(&job_list, job->pgid, JOB_DONE);
        }
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
    // If no argument, use most recent stopped job
    int job_id;
    if (args[1] == NULL) {
        // Find most recent stopped job
        Job *job = job_list.first_job;
        Job *latest_stopped = NULL;
        
        while (job != NULL) {
            if (job->status == JOB_STOPPED) {
                if (!latest_stopped || job->job_id > latest_stopped->job_id) {
                    latest_stopped = job;
                }
            }
            job = job->next;
        }
        
        if (!latest_stopped) {
            fprintf(stderr, "bg: no stopped jobs\n");
            return 1;
        }
        
        job_id = latest_stopped->job_id;
    } else {
        // Parse job ID from argument
        char *id_str = args[1];
        if (id_str[0] == '%') {
            id_str++;
        }
        job_id = atoi(id_str);
    }
    
    // Find the job
    Job *job = find_job(&job_list, job_id);
    if (!job) {
        fprintf(stderr, "bg: job %d not found\n", job_id);
        return 1;
    }
    
    // Verify job is stopped
    if (job->status != JOB_STOPPED) {
        fprintf(stderr, "bg: job %d is not stopped\n", job_id);
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
    print_jobs(&job_list);
    return 0;
}