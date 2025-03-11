// /include/prompt.h - Shell prompt
#pragma once

#include "beaudyshell.h"

/**
 * Print the shell prompt to standard output
 * 
 * @param state Current shell state containing prompt information
 */
void print_prompt(ShellState *state);

/**
 * Update prompt information in the shell state
 * 
 * @param state Current shell state to update
 */
void update_prompt_info(ShellState *state);

/* PROMPT_H */
