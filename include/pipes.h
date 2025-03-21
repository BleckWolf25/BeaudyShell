// /include/pipes.h - Pipe handling
#pragma once

#include "beaudyshell.h"
#include <stdbool.h>

/**
 * Structure representing a command pipeline with redirection
 */
typedef struct Pipeline {
  char ***commands;           // Array of command argument arrays
  int cmd_count;              // Number of commands in the pipeline
  char *input_file;           // Input redirection file
  char *output_file;          // Output redirection file
  bool append_output;         // Whether to append to output file
  char *original_command;     // Original command string
  bool background;            // Background execution flag
} Pipeline;


/**
 * Parse an input string into a pipeline structure
 * 
 * @param input Input string to parse
 * @return Dynamically allocated Pipeline structure, or NULL on error.
 *         Caller must free this memory using free_pipeline().
 */
Pipeline *parse_pipeline(char *input);

/**
 * Execute a pipeline of commands
 * 
 * @param pipeline Pipeline structure to execute
 * @param state Current shell state
 * @return Exit status of the last command in the pipeline
 */
int execute_pipeline(Pipeline *pipeline, ShellState *state);

/**
 * Free the memory allocated for a pipeline structure
 * 
 * @param pipeline Pipeline structure to free
 */
void free_pipeline(Pipeline *pipeline);

/* PIPES_H */
