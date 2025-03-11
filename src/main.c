// /src/main.c - Entry point for the shell
#include "beaudyshell.h"
#include "signals.h"
#include "jobs.h"
#include <unistd.h>

int main(int argc, char *argv[], char *envp[]) {
  (void)argc; // Suppress unused parameter warning
  (void)argv; // Suppress unused parameter warning
  
  ShellState state;

  // Process command line arguments
  state.interactive = isatty(STDIN_FILENO);

  // Initialize the shell
  initialize_shell(&state, envp);

  // Initialize job control
  init_job_control(&state);

  // Set up signal handlers
  setup_signals(&state);

  // Run the shell
  run_shell(&state);

  // Clean up
  cleanup_jobs(&state);
  cleanup_shell(&state);

  return state.last_exit_code;
}
