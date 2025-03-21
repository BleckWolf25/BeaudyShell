// /src/signals.c - Signal handling
#include "signals.h"
#include "jobs.h"
#include <errno.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/wait.h>

static ShellState *shell_state = NULL;
static volatile sig_atomic_t received_sigchld = 0;
static sigset_t saved_mask;

void setup_signals(ShellState *state) {
    shell_state = state;
    struct sigaction sa;
    sigset_t block_mask;

    // Block all signals during handler execution
    sigfillset(&block_mask);
    sa.sa_mask = block_mask;
    sa.sa_flags = SA_RESTART;

    // Save original signal mask
    sigemptyset(&saved_mask);
    sigprocmask(SIG_BLOCK, NULL, &saved_mask);

    // Set up SIGCHLD handler
    sa.sa_handler = handle_sigchld;
    if (sigaction(SIGCHLD, &sa, NULL) == -1) {
        perror("sigaction(SIGCHLD)");
        exit(EXIT_FAILURE);
    }

    // Set up SIGINT handler
    sa.sa_handler = handle_sigint;
    if (sigaction(SIGINT, &sa, NULL) == -1) {
        perror("sigaction(SIGINT)");
        exit(EXIT_FAILURE);
    }

    // Ignore SIGTSTP and SIGQUIT in the shell
    sa.sa_handler = SIG_IGN;
    if (sigaction(SIGTSTP, &sa, NULL) == -1 ||
        sigaction(SIGQUIT, &sa, NULL) == -1) {
        perror("sigaction(SIGTSTP/SIGQUIT)");
        exit(EXIT_FAILURE);
    }
}

static void handle_sigchld_internal(void) {
    int saved_errno = errno;
    pid_t pid;
    int status;
    sigset_t block_mask, orig_mask;

    sigfillset(&block_mask);
    sigprocmask(SIG_BLOCK, &block_mask, &orig_mask);

    while ((pid = waitpid(-1, &status, WNOHANG | WUNTRACED | WCONTINUED)) > 0) {
        if (!shell_state || !shell_state->jobs) continue;

        if (WIFSTOPPED(status)) {
            update_job_status(shell_state, pid, JOB_STOPPED);
        } else if (WIFCONTINUED(status)) {
            update_job_status(shell_state, pid, JOB_RUNNING);
        } else if (WIFEXITED(status) || WIFSIGNALED(status)) {
            update_job_status(shell_state, pid, JOB_DONE);
            remove_done_jobs(shell_state);
        }
    }

    sigprocmask(SIG_SETMASK, &orig_mask, NULL);
    errno = saved_errno;
}

void handle_sigchld(int sig) {
  (void)sig;
  received_sigchld = 1;
}

void handle_sigint(int sig) {
  (void)sig;
  if (shell_state && shell_state->interactive) {
    printf("\n");
    // Reset prompt on next line
  }
}

void check_background_jobs(void) {
    sigset_t mask, prev_mask;
    
    sigemptyset(&mask);
    sigaddset(&mask, SIGCHLD);
    sigprocmask(SIG_BLOCK, &mask, &prev_mask);

    if (received_sigchld) {
        handle_sigchld_internal();
        received_sigchld = 0;
    }

    sigprocmask(SIG_SETMASK, &prev_mask, NULL);
}

void reset_signals(void) {
  struct sigaction sa;
  sa.sa_handler = SIG_DFL;
  sa.sa_flags = 0;
  sigemptyset(&sa.sa_mask);

  // Reset all relevant signals to default behavior
  sigaction(SIGINT, &sa, NULL);
  sigaction(SIGQUIT, &sa, NULL);
  sigaction(SIGTSTP, &sa, NULL);
  sigaction(SIGCHLD, &sa, NULL);
}
