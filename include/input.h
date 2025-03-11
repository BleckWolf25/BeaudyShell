// /include/input.h - Input handling
#pragma once

/**
 * Read a line of input from standard input
 * 
 * @return Dynamically allocated string containing the input line with newline removed,
 *         or NULL on EOF or error. Caller must free this memory.
 */
char *read_input(void);

/**
 * Parse an input string into an array of arguments
 * 
 * @param input Input string to parse
 * @param arg_count Pointer to an integer where the number of arguments will be stored
 * @return Dynamically allocated array of argument strings, or NULL on error.
 *         The array itself must be freed by the caller, but not the individual strings
 *         as they point into the original input string.
 */
char **parse_input(char *input, int *arg_count);

/**
 * Free the memory allocated for an argument array
 * 
 * @param args Argument array to free
 */
void free_args(char **args);

/* INPUT_H */
