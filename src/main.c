// main.c - Entry point for the shell
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "BeaudyShell.h"
#include "signals.h"
#include "prompt.h"

int main(int argc, char *argv[], char *envp[]) {
    ShellState state;
    
    // Process command line arguments
    state.interactive = isatty(STDIN_FILENO);
    
    // Initialize the shell
    initialize_shell(&state, envp);
    
    // Set up signal handlers
    setup_signals();
    
    // Run the shell
    run_shell(&state);
    
    // Clean up
    cleanup_shell(&state);
    
    return state.last_exit_code;
}