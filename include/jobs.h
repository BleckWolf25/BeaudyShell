// jobs.h - Job control
#ifndef JOBS_H
#define JOBS_H

#include "beaudyShell.h"
#include <sys/types.h>

typedef enum { JOB_RUNNING, JOB_STOPPED, JOB_DONE } JobStatus;

typedef struct Job {
  int job_id;
  pid_t pgid;    // Process group ID
  char *command; // Command string
  JobStatus status;
  struct Job *next; // For linked list
} Job;

typedef struct {
  Job *first_job;
  int next_job_id;
} JobList;

void init_job_control(JobList *jobs);
void add_job(JobList *jobs, pid_t pgid, const char *cmd);
void update_job_status(JobList *jobs, pid_t pid, JobStatus status);
void remove_done_jobs(JobList *jobs);
void print_jobs(JobList *jobs);
Job *find_job(JobList *jobs, int job_id);
Job *find_job_by_pid(JobList *jobs, pid_t pid);
void cleanup_jobs(JobList *jobs);

#endif /* JOBS_H */