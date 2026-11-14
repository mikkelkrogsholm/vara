#ifndef VARA_INSERTION_POLICY_H
#define VARA_INSERTION_POLICY_H
#include <stdbool.h>
// Pure pre-dispatch policy, shared with deterministic native tests. No app data.
static inline const char *vara_insertion_guard(bool trusted, bool target_exists,
        bool app_alive, bool same_app, bool field_available, bool same_field, bool supported_role) {
    if (!trusted) return "accessibility_missing";
    if (!target_exists || !app_alive) return "target_gone";
    if (!same_app) return "app_changed";
    if (!field_available) return "field_unavailable";
    if (!same_field) return "focus_changed";
    if (!supported_role) return "unsupported_role";
    return 0;
}
static inline const char *vara_keyboard_guard(bool post_access, bool event_created) {
    if (!post_access) return "post_event_access_missing";
    if (!event_created) return "dispatch_failed";
    return 0;
}
static inline const char *vara_interrupted_delivery(bool any_sent) {
    return any_sent ? "unknown" : "manual_ready";
}
#endif
