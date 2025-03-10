// utils.h - Utility functions
#ifndef UTILS_H
#define UTILS_H

#include <stdbool.h>

bool file_exists(const char *path);
char *find_executable(const char *name);
void trim_string(char *str);
char *str_duplicate(const char *src);

#endif /* UTILS_H */