// /include/signals.h - Signal handling
#pragma once

/**
 * Handle SIGCHLD signal (child process status changed)
 * 
 * @param sig Signal number
 */
void handle_sigchld(int sig);

/**
 * Handle SIGINT signal (Ctrl+C)
 * 
 * @param sig Signal number
 */
void handle_sigint(int sig);

/**
 * Set up signal handlers for the shell
 */
void setup_signals(void);

/**
 * Reset signal handlers to default behavior (for child processes)
 */
void reset_signals(void);

/* SIGNALS_H */
