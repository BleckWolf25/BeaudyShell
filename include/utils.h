// /include/utils.h - Utility functions
#pragma once

#include <stdbool.h>

/**
 * Check if a file exists at the specified path
 * 
 * @param path Path to the file to check
 * @return true if the file exists and is accessible, false otherwise
 */
bool file_exists(const char *path);

/**
 * Find the full path to an executable in the PATH
 * 
 * @param name Name of the executable to find
 * @return Dynamically allocated string containing the full path to the executable,
 *         or NULL if not found. Caller must free this memory.
 */
char *find_executable(const char *name);

/**
 * Trim leading and trailing whitespace from a string in-place
 * 
 * @param str String to trim
 */
void trim_string(char *str);

/**
 * Create a duplicate of a string
 * 
 * @param src Source string to duplicate
 * @return Dynamically allocated copy of the string, or NULL on error.
 *         Caller must free this memory.
 */
char *str_duplicate(const char *src);

/* UTILS_H */
