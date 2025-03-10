// prompt.c - Shell prompt handling
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <pwd.h>
#include "prompt.h"

void update_prompt_info(ShellState *state) {
    // Update current directory
    if (getcwd(state->cwd, sizeof(state->cwd)) == NULL) {
        // If getcwd fails, keep the old value
        perror("getcwd");
    }
}

void print_prompt(ShellState *state) {
    // Create a fancy colorful prompt
    // Format: username@hostname:directory$
    printf("\033[1;32m%s@%s\033[0m:\033[1;34m%s\033[0m$ ", 
           state->username, state->hostname, state->cwd);
    fflush(stdout);
}