// /src/main.c - Entry point for the shell
#include "beaudyshell.h"
#include "signals.h"
#include "jobs.h"
#include <unistd.h>

int main(int argc __attribute__((unused)), char *argv[] __attribute__((unused)),
         char *envp[]) {
  extern char **environ;
  ShellState state;

  // Process command line arguments
  state.interactive = isatty(STDIN_FILENO);

  // Initialize the shell
  initialize_shell(&state, envp);

  // Initialize job control
  init_job_control(&state);

  // Set up signal handlers
  setup_signals();

  // Run the shell
  run_shell(&state);

  // Clean up
  cleanup_jobs(&state);
  cleanup_shell(&state);

  return state.last_exit_code;
}
