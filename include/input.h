// input.h - Input handling
#ifndef INPUT_H
#define INPUT_H

#include "beaudyShell.h"

char *read_input(void);
char **parse_input(char *input, int *arg_count);
void free_args(char **args);

#endif /* INPUT_H */
