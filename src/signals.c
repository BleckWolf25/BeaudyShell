// /src/signals.c - Signal handling
#include "signals.h"
#include "jobs.h"
#include <signal.h>
#include <stdio.h>
#include <sys/wait.h>

// External job list reference
extern JobList job_list;

void handle_sigint(int sig __attribute__((unused))) {
  // Handle Ctrl+C
  printf("\n");
  // We don't need to do anything else as the main shell
  // process should ignore this signal and only child processes
  // should be affected
}

void setup_signals(void) {
  // Ignore SIGINT (Ctrl+C) in the shell
  signal(SIGINT, SIG_IGN);

  // Ignore SIGTSTP (Ctrl+Z) in the shell
  signal(SIGTSTP, SIG_IGN);

  // Ignore SIGQUIT (Ctrl+\) in the shell
  signal(SIGQUIT, SIG_IGN);

  // Set handler for SIGCHLD for job control in the shell
  signal(SIGCHLD, handle_sigchld);
}

void handle_sigchld(int sig __attribute__((unused))) {
  int status;
  pid_t pid;
  
  while ((pid = waitpid(-1, &status, WNOHANG | WUNTRACED)) > 0) {
      if (WIFSTOPPED(status)) {
          update_job_status(&job_list, pid, JOB_STOPPED);
      } else if (WIFEXITED(status) || WIFSIGNALED(status)) {
          update_job_status(&job_list, pid, JOB_DONE);
      }
  }
  
  // Clean up completed jobs
  remove_done_jobs(&job_list);
}

void reset_signals(void) {
  // Reset signals to default behavior (for child processes)
  signal(SIGINT, SIG_DFL);
  signal(SIGTSTP, SIG_DFL);
  signal(SIGQUIT, SIG_DFL);
  signal(SIGCHLD, SIG_DFL);  // Also reset SIGCHLD
}
