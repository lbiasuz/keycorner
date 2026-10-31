#pragma once
#include <stdbool.h>
#include <stdint.h>
typedef uint32_t layer_state_t;
extern layer_state_t layer_state, default_layer_state;
uint32_t timer_read32(void);
uint32_t timer_elapsed32(uint32_t since);
