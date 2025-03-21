// /src/jobs.c - Job control implementation
#include "jobs.h"
#include "beaudyshell.h"
#include "utils.h"
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <termios.h>
#include <unistd.h>
#include <pthread.h>

static pthread_mutex_t job_mutex = PTHREAD_MUTEX_INITIALIZER;

static const char *job_status_str(JobStatus status) {
  switch (status) {
  case JOB_RUNNING:
    return "Running";
  case JOB_STOPPED:
    return "Stopped";
  case JOB_DONE:
    return "Done";
  default:
    return "Unknown";
  }
}

// Initialize job control
void init_job_control(ShellState *state) {
  if (!state) return;

  pthread_mutex_lock(&job_mutex);
  state->jobs = malloc(sizeof(JobList));
  if (!state->jobs) {
    perror("malloc");
    exit(EXIT_FAILURE);
  }

  state->jobs->first_job = NULL;
  state->jobs->next_job_id = 1;
  pthread_mutex_unlock(&job_mutex);

  // Set process group and terminal control only in interactive mode
  if (state->interactive) {
    pid_t shell_pgid = getpid();
    
    // Put shell in its own process group
    if (setpgid(shell_pgid, shell_pgid) < 0) {
      perror("setpgid");
      exit(EXIT_FAILURE);
    }

    // Take control of the terminal
    if (tcsetpgrp(STDIN_FILENO, shell_pgid) < 0) {
      perror("tcsetpgrp");
      exit(EXIT_FAILURE);
    }
  }
}

// Add a new job to the job list
Job *add_job(ShellState *state, pid_t pgid, const char *cmd) {
  if (!state || !state->jobs || !cmd) {
    return NULL;
  }

  pthread_mutex_lock(&job_mutex);
  Job *job = calloc(1, sizeof(Job));
  if (!job) {
    pthread_mutex_unlock(&job_mutex);
    perror("calloc");
    return NULL;
  }

  job->job_id = state->jobs->next_job_id++;
  job->pgid = pgid;
  job->command = str_duplicate(cmd);
  if (!job->command) {
    free(job);
    pthread_mutex_unlock(&job_mutex);
    return NULL;
  }
  job->status = JOB_RUNNING;

  // Insert at head of list
  job->next = state->jobs->first_job;
  state->jobs->first_job = job;
  pthread_mutex_unlock(&job_mutex);

  if (state->interactive) {
    printf("[%d] %d %s\t%s\n", job->job_id, (int)job->pgid,
           job_status_str(job->status), job->command);
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
    const char *status_str = job->status == JOB_RUNNING   ? "Running"
                             : job->status == JOB_STOPPED ? "Stopped"
                                                          : "Done";

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
  if (!state || !state->jobs) return;

  pthread_mutex_lock(&job_mutex);
  Job *curr = state->jobs->first_job;
  while (curr) {
    // Send SIGHUP to any remaining job process groups
    if (curr->status != JOB_DONE) {
      killpg(curr->pgid, SIGHUP);
    }
    
    Job *next = curr->next;
    free(curr->command);
    free(curr);
    curr = next;
  }
  pthread_mutex_unlock(&job_mutex);

  free(state->jobs);
  state->jobs = NULL;
}
