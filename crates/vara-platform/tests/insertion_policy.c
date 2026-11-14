#include "macos_insertion_policy.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>
static void reason(const char *actual, const char *expected) {
    assert(actual && strcmp(actual,expected)==0);
}
int main(void) {
    // Baseline exact field + PID succeeds. Each mutation is an independent refusal.
    assert(!vara_insertion_guard(true,true,true,true,true,true,true));
    reason(vara_insertion_guard(false,true,true,true,true,true,true),"accessibility_missing");
    reason(vara_insertion_guard(true,false,true,true,true,true,true),"target_gone");
    reason(vara_insertion_guard(true,true,false,true,true,true,true),"target_gone");
    reason(vara_insertion_guard(true,true,true,false,true,true,true),"app_changed");
    reason(vara_insertion_guard(true,true,true,true,false,true,true),"field_unavailable");
    reason(vara_insertion_guard(true,true,true,true,true,false,true),"focus_changed");
    reason(vara_insertion_guard(true,true,true,true,true,true,false),"unsupported_role");
    // Revocation takes precedence; no permission is requested by this policy.
    reason(vara_insertion_guard(false,false,false,false,false,false,false),"accessibility_missing");
    assert(!vara_keyboard_guard(true,true));
    reason(vara_keyboard_guard(false,true),"post_event_access_missing");
    reason(vara_keyboard_guard(true,false),"dispatch_failed");
    reason(vara_keyboard_guard(false,false),"post_event_access_missing");
    // Focus loss, event allocation failure or permission revocation after any send
    // cannot be called manual-ready; the same conditions before send are safe refusals.
    reason(vara_interrupted_delivery(false),"manual_ready");
    reason(vara_interrupted_delivery(true),"unknown");
    puts("PASS: native insertion guard, keyboard permission/allocation, before/after-send policy");
}
