// SPDX-License-Identifier: GPL-2.0-or-later
// Optional, leased layer-state reporting. No key events or EEPROM writes.
#include QMK_KEYBOARD_H
#include "raw_hid.h"
#include <string.h>

static bool subscribed;
static uint32_t renewed_at, session;
static layer_state_t previous_layers, previous_default;

static void put32(uint8_t *p, uint32_t v) {
    for (uint8_t i = 0; i < 4; ++i) p[i] = v >> (8 * i);
}
static uint32_t get32(const uint8_t *p) {
    return (uint32_t)p[0] | (uint32_t)p[1] << 8 | (uint32_t)p[2] << 16 | (uint32_t)p[3] << 24;
}
static void response(uint8_t *data, uint8_t op, uint32_t token) {
    memset(data, 0, 32);
    memcpy(data, "\xfcKCLY", 5);
    data[5] = 1;
    data[6] = op;
    data[7] = 1; // layer masks supported
    put32(data + 8, token);
    put32(data + 12, layer_state);
    put32(data + 16, default_layer_state);
}
void raw_hid_receive_kb(uint8_t *data, uint8_t length) {
    if (length != 32 || memcmp(data, "\xfcKCLY", 5) || data[5] != 1 || data[6] > 2) {
        data[0] = 0xff;
        return;
    }
    uint8_t op = data[6];
    uint32_t token = get32(data + 8);
    if (op == 1) {
        subscribed = true;
        session = token;
        renewed_at = timer_read32();
        previous_layers = layer_state;
        previous_default = default_layer_state;
    } else if (op == 2 && token == session) {
        subscribed = false;
    }
    // VIA sends this reply after returning from the hook.
    response(data, 0x80 | op, token);
}
void housekeeping_task_user(void) {
    if (!subscribed) return;
    if (timer_elapsed32(renewed_at) >= 3000) {
        subscribed = false;
        return;
    }
    if (layer_state != previous_layers || default_layer_state != previous_default) {
        previous_layers = layer_state;
        previous_default = default_layer_state;
        uint8_t data[32];
        response(data, 0x83, session);
        raw_hid_send(data, sizeof(data));
    }
}
