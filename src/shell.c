// /src/shell.c - Main shell operation
#include "beaudyshell.h"
#include "input.h"
#include "pipes.h"
#include "prompt.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

void initialize_shell(ShellState *state, char **environ) {
  state->running = true;
  state->last_exit_code = 0;
  state->environ = environ;

  // Get current working directory
  if (getcwd(state->cwd, sizeof(state->cwd)) == NULL) {
    perror("getcwd");
    exit(EXIT_FAILURE);
  }

  // Get username
  char *user = getenv("USER");
  if (user != NULL) {
    strncpy(state->username, user, MAX_USERNAME_LEN - 1);
    state->username[MAX_USERNAME_LEN - 1] = '\0';
  } else {
    strcpy(state->username, "user");
  }

  // Get hostname
  if (gethostname(state->hostname, MAX_HOSTNAME_LEN) != 0) {
    strcpy(state->hostname, "localhost");
  }
  state->hostname[MAX_HOSTNAME_LEN - 1] = '\0';
}

void cleanup_shell(ShellState *state __attribute__((unused))) {
  // Currently no dynamic memory in ShellState to free
  // This function will be expanded later as we add more features
}

void run_shell(ShellState *state) {
  char *input;

  while (state->running) {
    // Update prompt information (cwd, etc.) before printing
    update_prompt_info(state);

    // Print prompt in interactive mode
    if (state->interactive) {
      print_prompt(state);
    }

    // Read input
    input = read_input();
    if (!input) {
      // EOF (Ctrl+D) - exit shell
      if (state->interactive) {
        printf("\n");
      }
      state->running = false;
      break;
    }

    // Skip empty lines
    if (input[0] == '\0') {
      free(input);
      continue;
    }

    // Parse and handle command with pipes
    Pipeline *pipeline = parse_pipeline(input);
    if (pipeline != NULL) {
      state->last_exit_code = execute_pipeline(pipeline, state);
      free_pipeline(pipeline);
    }

    free(input);
  }
}
