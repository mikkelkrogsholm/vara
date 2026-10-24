#include <assert.h>
#include <stdint.h>
#include <stddef.h>
#include <stdio.h>
#include "../../crates/vara-platform/src/macos_text_input.h"
int main(void) {
    uint16_t text[42]={0};
    assert(vara_text_chunk(text,5)==5);
    assert(vara_text_chunk(text,42)==20);
    text[19]=0xD83D; text[20]=0xDE00;
    assert(vara_text_chunk(text,42)==19);
    assert(vara_text_chunk(text+19,23)==20);
    assert(vara_text_chunk(text,20)==20);
    puts("PASS: UTF-16 payload size and emoji boundary");
}
