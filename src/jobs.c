// jobs.c - Job control implementation
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <signal.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <termios.h>
#include "jobs.h"
#include "utils.h"

// Initialize job control
void init_job_control(JobList *jobs) {
    jobs->first_job = NULL;
    jobs->next_job_id = 1;
    
    // Put shell in its own process group
    setpgid(0, 0);
    
    // Take control of the terminal
    tcsetpgrp(STDIN_FILENO, getpgrp());
}

// Add a new job to the job list
void add_job(JobList *jobs, pid_t pgid, const char *cmd) {
    Job *job = malloc(sizeof(Job));
    if (!job) {
        perror("malloc");
        return;
    }
    
    job->job_id = jobs->next_job_id++;
    job->pgid = pgid;
    job->command = str_duplicate(cmd);
    job->status = JOB_RUNNING;
    job->next = NULL;
    
    // Add to the end of the list
    if (jobs->first_job == NULL) {
        jobs->first_job = job;
    } else {
        Job *j = jobs->first_job;
        while (j->next != NULL) {
            j = j->next;
        }
        j->next = job;
    }
    
    printf("[%d] %d Running                 %s\n", 
           job->job_id, (int)job->pgid, job->command);
}

// Update job status based on process status change
void update_job_status(JobList *jobs, pid_t pid, JobStatus status) {
    Job *job = find_job_by_pid(jobs, pid);
    
    if (job) {
        job->status = status;
        
        if (status == JOB_STOPPED) {
            printf("[%d] %d Stopped                 %s\n", 
                   job->job_id, (int)job->pgid, job->command);
        } else if (status == JOB_DONE) {
            printf("[%d] %d Done                    %s\n", 
                   job->job_id, (int)job->pgid, job->command);
        }
    }
}

// Remove completed jobs from the list
void remove_done_jobs(JobList *jobs) {
    Job *prev = NULL;
    Job *curr = jobs->first_job;
    
    while (curr != NULL) {
        if (curr->status == JOB_DONE) {
            Job *temp = curr;
            
            if (prev == NULL) {
                // First job in list
                jobs->first_job = curr->next;
                curr = jobs->first_job;
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
void print_jobs(JobList *jobs) {
    Job *job = jobs->first_job;
    
    if (!job) {
        printf("No jobs\n");
        return;
    }
    
    printf("JOB  PID   STATUS   COMMAND\n");
    while (job != NULL) {
        const char *status_str = 
            job->status == JOB_RUNNING ? "Running" :
            job->status == JOB_STOPPED ? "Stopped" : "Done";
        
        printf("[%d]  %d  %s  %s\n", 
               job->job_id, (int)job->pgid, status_str, job->command);
        
        job = job->next;
    }
}

// Find a job by job ID
Job *find_job(JobList *jobs, int job_id) {
    Job *job = jobs->first_job;
    
    while (job != NULL) {
        if (job->job_id == job_id) {
            return job;
        }
        job = job->next;
    }
    
    return NULL;
}

// Find a job by process group ID
Job *find_job_by_pid(JobList *jobs, pid_t pid) {
    Job *job = jobs->first_job;
    
    while (job != NULL) {
        if (job->pgid == pid) {
            return job;
        }
        job = job->next;
    }
    
    return NULL;
}

// Clean up all jobs
void cleanup_jobs(JobList *jobs) {
    Job *curr = jobs->first_job;
    
    while (curr != NULL) {
        Job *temp = curr;
        curr = curr->next;
        
        if (temp->command) {
            free(temp->command);
        }
        free(temp);
    }
    
    jobs->first_job = NULL;
}