// /include/jobs.h - Job control
#pragma once

#include <sys/types.h>
#include <stdbool.h>
#include "beaudyshell.h"

/**
 * Enumeration of possible job states
 */
typedef enum { 
    JOB_RUNNING,  // Job is running in foreground or background
    JOB_STOPPED,  // Job is stopped (suspended)
    JOB_DONE      // Job has completed
} JobStatus;

/**
 * Structure representing a job (process group)
 */
typedef struct Job {
    int job_id;       // Job ID (for user reference)
    pid_t pgid;       // Process group ID
    char *command;    // Command string
    JobStatus status; // Current job status
    struct Job *next; // For linked list
} Job;

/**
 * Structure representing a list of jobs
 */
typedef struct JobList {
    Job *first_job;   // First job in the list
    int next_job_id;  // Next job ID to assign
} JobList;

/**
 * Initialize job control subsystem
 * 
 * @param state Shell state containing job list
 */
void init_job_control(ShellState *state);

/**
 * Add a new job to the job list
 * 
 * @param state Shell state containing job list
 * @param pgid Process group ID
 * @param cmd Command string
 * @return Pointer to the newly created job, or NULL on failure
 */
Job *add_job(ShellState *state, pid_t pgid, const char *cmd);

/**
 * Update job status based on process status change
 * 
 * @param state Shell state containing job list
 * @param pid Process ID
 * @param status New job status
 * @return Pointer to the updated job, or NULL if not found
 */
Job *update_job_status(ShellState *state, pid_t pid, JobStatus status);

/**
 * Remove completed jobs from the list
 * 
 * @param state Shell state containing job list
 */
void remove_done_jobs(ShellState *state);

/**
 * Print all jobs in the job list
 * 
 * @param state Shell state containing job list
 */
void print_jobs(ShellState *state);

/**
 * Find a job by job ID
 * 
 * @param state Shell state containing job list
 * @param job_id Job ID to find
 * @return Pointer to job or NULL if not found
 */
Job *find_job(ShellState *state, int job_id);

/**
 * Find a job by process group ID
 * 
 * @param state Shell state containing job list
 * @param pgid Process group ID to find
 * @return Pointer to job or NULL if not found
 */
Job *find_job_by_pgid(ShellState *state, pid_t pgid);

/**
 * Find the most recent job with given status
 * 
 * @param state Shell state containing job list
 * @param status Job status to match
 * @return Pointer to the most recent job with matching status, or NULL if not found
 */
Job *find_recent_job_by_status(ShellState *state, JobStatus status);

/**
 * Clean up all jobs in the job list
 * 
 * @param state Shell state containing job list
 */
void cleanup_jobs(ShellState *state);

/* JOBS_H */
