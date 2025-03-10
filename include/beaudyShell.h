// beaudyShell.h - Main header file
#ifndef BeaudyShell_H
#define BeaudyShell_H

#include <stdbool.h>

#define MAX_INPUT_SIZE 1024
#define MAX_ARGS 64
#define MAX_PATH_LEN 256
#define MAX_HOSTNAME_LEN 64
#define MAX_USERNAME_LEN 32
#define MAX_PROMPT_LEN 512

// Shell state structure
typedef struct {
  char cwd[MAX_PATH_LEN];          // Current working directory
  bool running;                    // Shell running state
  char username[MAX_USERNAME_LEN]; // Current username
  char hostname[MAX_HOSTNAME_LEN]; // Machine hostname
  int last_exit_code;              // Exit code of last command
  char **environ;                  // Environment variables
  bool interactive;                // Running in interactive mode?
} ShellState;

// Function prototypes for main shell operations
void initialize_shell(ShellState *state, char **environ);
void cleanup_shell(ShellState *state);
void run_shell(ShellState *state);
void handle_signal(int signo);

#endif /* BEAUDYSHELL_H */
