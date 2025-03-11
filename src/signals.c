// /src/signals.c - Signal handling
#include "signals.h"
#include "jobs.h"
#include <signal.h>
#include <stdio.h>
#include <sys/wait.h>
#include <stdlib.h>

// Global shell state reference for signal handlers
static ShellState *shell_state = NULL;

void handle_sigint(int sig) {
  (void)sig; // Suppress unused parameter warning
  printf("\n");
  // We don't need to do anything else as the main shell
  // process should ignore this signal and only child processes
  // should be affected
}

void setup_signals(ShellState *state) {
  // Store shell state for signal handlers
  shell_state = state;

  // Set up signal actions with sigaction instead of signal
  struct sigaction sa;
  
  // Initialize sigaction struct
  sa.sa_flags = SA_RESTART;
  sigemptyset(&sa.sa_mask);
  
  // Ignore SIGINT (Ctrl+C) in the shell
  sa.sa_handler = SIG_IGN;
  if (sigaction(SIGINT, &sa, NULL) == -1) {
    perror("sigaction");
    exit(EXIT_FAILURE);
  }

  // Ignore SIGTSTP (Ctrl+Z) in the shell
  if (sigaction(SIGTSTP, &sa, NULL) == -1) {
    perror("sigaction");
    exit(EXIT_FAILURE);
  }

  // Ignore SIGQUIT (Ctrl+\) in the shell
  if (sigaction(SIGQUIT, &sa, NULL) == -1) {
    perror("sigaction");
    exit(EXIT_FAILURE);
  }

  // Set handler for SIGCHLD for job control in the shell
  sa.sa_handler = handle_sigchld;
  if (sigaction(SIGCHLD, &sa, NULL) == -1) {
    perror("sigaction");
    exit(EXIT_FAILURE);
  }
}

void handle_sigchld(int sig) {
  (void)sig; // Suppress unused parameter warning
  
  // Safety check
  if (shell_state == NULL || shell_state->jobs == NULL) {
    return;
  }
  
  int status;
  pid_t pid;
  
  while ((pid = waitpid(-1, &status, WNOHANG | WUNTRACED)) > 0) {
    if (WIFSTOPPED(status)) {
      update_job_status(shell_state, pid, JOB_STOPPED);
    } else if (WIFEXITED(status) || WIFSIGNALED(status)) {
      update_job_status(shell_state, pid, JOB_DONE);
    }
  }
  
  // Clean up completed jobs
  remove_done_jobs(shell_state);
}

void reset_signals(void) {
  // Reset signals to default behavior (for child processes)
  signal(SIGINT, SIG_DFL);
  signal(SIGTSTP, SIG_DFL);
  signal(SIGQUIT, SIG_DFL);
  signal(SIGCHLD, SIG_DFL);  // Also reset SIGCHLD
}
