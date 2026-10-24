// UTF-16 event payloads may contain at most 20 code units. Never split a surrogate pair.
#define VARA_TEXT_EVENT_MARKER 0x56415241545854LL
static inline size_t vara_text_chunk(const uint16_t *text, size_t remaining) {
    size_t count = remaining < 20 ? remaining : 20;
    if (count < remaining && text[count-1] >= 0xD800 && text[count-1] <= 0xDBFF) count--;
    return count;
}
