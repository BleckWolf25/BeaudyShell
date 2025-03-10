// input.c - Input handling
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include "input.h"
#include "utils.h"

char *read_input(void) {
    char *input = malloc(MAX_INPUT_SIZE);
    
    if (!input) {
        perror("malloc");
        return NULL;
    }
    
    if (fgets(input, MAX_INPUT_SIZE, stdin) == NULL) {
        free(input);
        return NULL;
    }
    
    // Remove trailing newline
    size_t len = strlen(input);
    if (len > 0 && input[len-1] == '\n') {
        input[len-1] = '\0';
    }
    
    trim_string(input);
    return input;
}

char **parse_input(char *input, int *arg_count) {
    if (!input || input[0] == '\0') {
        *arg_count = 0;
        return NULL;
    }
    
    char **args = malloc(MAX_ARGS * sizeof(char *));
    if (!args) {
        perror("malloc");
        *arg_count = 0;
        return NULL;
    }
    
    // Simple space-based tokenization for now
    // This will be replaced with more advanced parsing later
    int i = 0;
    char *token = strtok(input, " \t\n");
    
    while (token != NULL && i < MAX_ARGS - 1) {
        args[i++] = token;
        token = strtok(NULL, " \t\n");
    }
    
    args[i] = NULL;
    *arg_count = i;
    return args;
}

void free_args(char **args) {
    if (args) {
        free(args);
    }
}