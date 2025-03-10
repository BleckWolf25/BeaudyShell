// pipes.h - Pipe handling
#ifndef PIPES_H
#define PIPES_H

#include "beaudyShell.h"

typedef struct {
  char ***commands;   // Array of command argument arrays
  int cmd_count;      // Number of commands in the pipeline
  char *input_file;   // Input redirection file
  char *output_file;  // Output redirection file
  bool append_output; // Whether to append to output file
} Pipeline;

Pipeline *parse_pipeline(char *input);
int execute_pipeline(Pipeline *pipeline, ShellState *state);
void free_pipeline(Pipeline *pipeline);

#endif /* PIPES_H */

