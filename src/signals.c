// signals.c - Signal handling
#include <stdio.h>
#include <stdlib.h>
#include <signal.h>
#include "signals.h"

void handle_sigint(int sig) {
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
}

void reset_signals(void) {
    // Reset signals to default behavior (for child processes)
    signal(SIGINT, SIG_DFL);
    signal(SIGTSTP, SIG_DFL);
    signal(SIGQUIT, SIG_DFL);
}