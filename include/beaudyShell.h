// /include/beaudyshell.h - Main header file for BeaudyShell
#pragma once

#include <stdbool.h>
#include <sys/types.h>

// Constants
#define MAX_INPUT_SIZE 1024      // Maximum size of input line
#define MAX_ARGS 64              // Maximum number of command arguments
#define MAX_PATH_LEN 256         // Maximum path length
#define MAX_HOSTNAME_LEN 64      // Maximum hostname length
#define MAX_USERNAME_LEN 32      // Maximum username length
#define MAX_PROMPT_LEN 512       // Maximum prompt length

// Forward declarations
typedef struct JobList JobList;

/**
 * Structure containing the shell's state
 */
typedef struct {
  char cwd[MAX_PATH_LEN];          // Current working directory
  bool running;                    // Shell running state
  char username[MAX_USERNAME_LEN]; // Current username
  char hostname[MAX_HOSTNAME_LEN]; // Machine hostname
  int last_exit_code;              // Exit code of last command
  char **environ;                  // Environment variables
  bool interactive;                // Running in interactive mode?
  JobList *jobs;                   // Job control list
} ShellState;

/**
 * Initialize the shell state
 * 
 * @param state Pointer to ShellState structure to initialize
 * @param environ Environment variables array
 */
void initialize_shell(ShellState *state, char **environ);

/**
 * Clean up resources used by the shell
 * 
 * @param state Pointer to ShellState structure to clean up
 */
void cleanup_shell(ShellState *state);

/**
 * Run the main shell loop
 * 
 * @param state Pointer to ShellState structure
 */
void run_shell(ShellState *state);

/* BEAUDYSHELL_H */
