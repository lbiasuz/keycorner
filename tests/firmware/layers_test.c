#include "fake_qmk.h"
#include <assert.h>
#include <string.h>
layer_state_t layer_state, default_layer_state=1;
static uint32_t now, sent;
static uint8_t last[32];
uint32_t timer_read32(void) { return now; }
uint32_t timer_elapsed32(uint32_t since) { return now-since; }
void raw_hid_send(uint8_t *data,uint8_t n) { assert(n==32);memcpy(last,data,n);sent++; }
#include "../../firmware/cstc40/keycorner_layers.c"
static void command(uint8_t op,uint32_t token) {
 uint8_t b[32]={0};memcpy(b,"\xfcKCLY",5);b[5]=1;b[6]=op;put32(b+8,token);
 raw_hid_receive_kb(b,32);assert(b[6]==(0x80|op));assert(get32(b+8)==token);
}
int main(void) {
 command(0,7);layer_state=2;housekeeping_task_user();assert(!sent);
 command(1,7);housekeeping_task_user();assert(!sent);
 layer_state=4;housekeeping_task_user();assert(sent==1&&get32(last+12)==4);
 housekeeping_task_user();assert(sent==1);
 default_layer_state=8;housekeeping_task_user();assert(sent==2&&get32(last+16)==8);
 command(2,8);layer_state=16;housekeeping_task_user();assert(sent==3);
 command(2,7);layer_state=32;housekeeping_task_user();assert(sent==3);
 now=0xfffffff0;command(1,9);now+=2999;layer_state=2;housekeeping_task_user();assert(sent==4);
 now++;layer_state=4;housekeeping_task_user();assert(sent==4);
 uint8_t bad[32]={0};raw_hid_receive_kb(bad,32);assert(bad[0]==0xff);
 return 0;
}
