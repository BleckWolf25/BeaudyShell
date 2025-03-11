// /src/jobs.c - Job control implementation
#include "beaudyshell.h"
#include "jobs.h"
#include "utils.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <termios.h>
#include <unistd.h>

// Initialize job control
void init_job_control(ShellState *state) {
    if (!state) {
        fprintf(stderr, "Error: null state in init_job_control\n");
        return;
    }
    
    // Allocate memory for job list
    JobList *job_list = malloc(sizeof(JobList));
    if (!job_list) {
        perror("malloc");
        return;
    }
    
    // Initialize job list
    job_list->first_job = NULL;
    job_list->next_job_id = 1;
    
    // Store reference to job list in shell state
    state->jobs = job_list;

    // Put shell in its own process group
    if (setpgid(0, 0) < 0) {
        perror("setpgid");
        return;
    }

    // Take control of the terminal if interactive
    if (state->interactive) {
        if (tcsetpgrp(STDIN_FILENO, getpgrp()) < 0) {
            perror("tcsetpgrp");
            return;
        }
    }
}

// Add a new job to the job list
Job *add_job(ShellState *state, pid_t pgid, const char *cmd) {
    if (!state || !state->jobs || !cmd) {
        fprintf(stderr, "Error: invalid parameters in add_job\n");
        return NULL;
    }

    Job *job = malloc(sizeof(Job));
    if (!job) {
        perror("malloc");
        return NULL;
    }

    job->job_id = state->jobs->next_job_id++;
    job->pgid = pgid;
    job->command = str_duplicate(cmd);
    if (!job->command) {
        perror("str_duplicate");
        free(job);
        return NULL;
    }
    job->status = JOB_RUNNING;
    job->next = NULL;

    // Add to the end of the list
    if (state->jobs->first_job == NULL) {
        state->jobs->first_job = job;
    } else {
        Job *j = state->jobs->first_job;
        while (j->next != NULL) {
            j = j->next;
        }
        j->next = job;
    }

    if (state->interactive) {
        printf("[%d] %d Running                 %s\n", job->job_id, (int)job->pgid,
             job->command);
    }
    
    return job;
}

// Update job status based on process status change
Job *update_job_status(ShellState *state, pid_t pid, JobStatus status) {
    if (!state || !state->jobs) {
        return NULL;
    }
    
    Job *job = find_job_by_pgid(state, pid);

    if (job) {
        job->status = status;

        if (state->interactive) {
            if (status == JOB_STOPPED) {
                printf("[%d] %d Stopped                 %s\n", job->job_id,
                    (int)job->pgid, job->command);
            } else if (status == JOB_DONE) {
                printf("[%d] %d Done                    %s\n", job->job_id,
                    (int)job->pgid, job->command);
            }
        }
    }
    
    return job;
}

// Remove completed jobs from the list
void remove_done_jobs(ShellState *state) {
    if (!state || !state->jobs) {
        return;
    }
    
    Job *prev = NULL;
    Job *curr = state->jobs->first_job;

    while (curr != NULL) {
        if (curr->status == JOB_DONE) {
            Job *temp = curr;

            if (prev == NULL) {
                // First job in list
                state->jobs->first_job = curr->next;
                curr = state->jobs->first_job;
            } else {
                // Middle or end of list
                prev->next = curr->next;
                curr = curr->next;
            }

            // Free job resources
            free(temp->command);
            free(temp);
        } else {
            prev = curr;
            curr = curr->next;
        }
    }
}

// Print all jobs
void print_jobs(ShellState *state) {
    if (!state || !state->jobs) {
        return;
    }
    
    Job *job = state->jobs->first_job;

    if (!job) {
        printf("No jobs\n");
        return;
    }

    printf("JOB  PID   STATUS   COMMAND\n");
    while (job != NULL) {
        const char *status_str = job->status == JOB_RUNNING ? "Running" :
                               job->status == JOB_STOPPED ? "Stopped" : "Done";

        printf("[%d]  %d  %s  %s\n", job->job_id, (int)job->pgid, status_str,
             job->command);

        job = job->next;
    }
}

// Find a job by job ID
Job *find_job(ShellState *state, int job_id) {
    if (!state || !state->jobs) {
        return NULL;
    }
    
    Job *job = state->jobs->first_job;

    while (job != NULL) {
        if (job->job_id == job_id) {
            return job;
        }
        job = job->next;
    }

    return NULL;
}

// Find a job by process group ID
Job *find_job_by_pgid(ShellState *state, pid_t pgid) {
    if (!state || !state->jobs) {
        return NULL;
    }
    
    Job *job = state->jobs->first_job;

    while (job != NULL) {
        if (job->pgid == pgid) {
            return job;
        }
        job = job->next;
    }

    return NULL;
}

// Find the most recent job with given status
Job *find_recent_job_by_status(ShellState *state, JobStatus status) {
    if (!state || !state->jobs) {
        return NULL;
    }
    
    Job *job = state->jobs->first_job;
    Job *latest = NULL;
    
    while (job != NULL) {
        if (job->status == status) {
            if (!latest || job->job_id > latest->job_id) {
                latest = job;
            }
        }
        job = job->next;
    }
    
    return latest;
}

// Clean up all jobs in the job list
void cleanup_jobs(ShellState *state) {
    if (!state || !state->jobs) {
        return;
    }
    
    Job *curr = state->jobs->first_job;

    while (curr != NULL) {
        Job *temp = curr;
        curr = curr->next;

        if (temp->command) {
            free(temp->command);
        }
        free(temp);
    }

    // Free the job list structure itself
    free(state->jobs);
    state->jobs = NULL;
}
