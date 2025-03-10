// utils.c - Utility functions
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <ctype.h>
#include "utils.h"

bool file_exists(const char *path) {
    return access(path, F_OK) == 0;
}

char *find_executable(const char *name) {
    if (name == NULL) return NULL;
    
    // If the name contains a slash, it's a path
    if (strchr(name, '/') != NULL) {
        if (file_exists(name)) {
            return str_duplicate(name);
        }
        return NULL;
    }
    
    // Check in PATH
    char *path_env = getenv("PATH");
    if (path_env == NULL) return NULL;
    
    char *path_copy = str_duplicate(path_env);
    char *dir = strtok(path_copy, ":");
    
    while (dir != NULL) {
        char full_path[MAX_PATH_LEN];
        snprintf(full_path, sizeof(full_path), "%s/%s", dir, name);
        
        if (file_exists(full_path)) {
            free(path_copy);
            return str_duplicate(full_path);
        }
        
        dir = strtok(NULL, ":");
    }
    
    free(path_copy);
    return NULL;
}

void trim_string(char *str) {
    if (!str) return;
    
    // Trim leading whitespace
    char *start = str;
    while (isspace((unsigned char)*start)) start++;
    
    if (start != str) {
        memmove(str, start, strlen(start) + 1);
    }
    
    // Trim trailing whitespace
    char *end = str + strlen(str) - 1;
    while (end > str && isspace((unsigned char)*end)) end--;
    *(end + 1) = '\0';
}

char *str_duplicate(const char *src) {
    if (!src) return NULL;
    
    char *dup = malloc(strlen(src) + 1);
    if (dup) {
        strcpy(dup, src);
    }
    return dup;
}