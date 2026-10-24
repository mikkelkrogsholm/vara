#import <AppKit/AppKit.h>
#import <ApplicationServices/ApplicationServices.h>
#import <IOKit/hidsystem/IOLLEvent.h>
#include "macos_text_input.h"

// Listen-only: never swallow normal typing or system shortcuts.
static CFMachPortRef tap;
static CFRunLoopSourceRef source;
static BOOL wanted[128], pressed[128], active;
static void (*callback)(bool);
static void (*capture)(uint16_t, bool);
static BOOL ensure_tap(void);
void vara_shortcut_clear(void);
static CGEventFlags modifier_mask(CGKeyCode key) {
    switch (key) {
        case 54:return NX_DEVICERCMDKEYMASK; case 55:return NX_DEVICELCMDKEYMASK;
        case 56:return NX_DEVICELSHIFTKEYMASK; case 60:return NX_DEVICERSHIFTKEYMASK;
        case 58:return NX_DEVICELALTKEYMASK; case 61:return NX_DEVICERALTKEYMASK;
        case 59:return NX_DEVICELCTLKEYMASK; case 62:return NX_DEVICERCTLKEYMASK;
        case 57:return kCGEventFlagMaskAlphaShift; case 63:return kCGEventFlagMaskSecondaryFn;
        default:return 0;
    }
}
static CGEventRef observe(CGEventTapProxy proxy, CGEventType type, CGEventRef event, void *info) {
    (void)proxy; (void)info;
    if (type == kCGEventTapDisabledByTimeout || type == kCGEventTapDisabledByUserInput) {
        memset(pressed, 0, sizeof(pressed));
        if (active && callback) callback(false);
        active = NO;
        CGEventTapEnable(tap, true);
        return event;
    }
    if (CGEventGetIntegerValueField(event,kCGEventSourceUserData)==VARA_TEXT_EVENT_MARKER) return event;
    CGKeyCode key = (CGKeyCode)CGEventGetIntegerValueField(event, kCGKeyboardEventKeycode);
    if (key >= 128) return event;
    if (type == kCGEventFlagsChanged) {
        pressed[key] = (CGEventGetFlags(event) & modifier_mask(key)) != 0;
    } else if (type == kCGEventKeyDown) {
        if (CGEventGetIntegerValueField(event, kCGKeyboardEventAutorepeat)) return event;
        pressed[key] = YES;
    } else if (type == kCGEventKeyUp) pressed[key] = NO;
    if (capture) capture(key, pressed[key]);
    BOOL down = YES, any = NO;
    for (int i=0; i<128; i++) if (wanted[i]) { any = YES; if (!pressed[i]) down = NO; }
    down = down && any;
    if (down != active) { active = down; if (callback) callback(down); }
    return event;
}
static BOOL ensure_tap(void) {
    if (!CGPreflightListenEventAccess()) return NO;
    if (tap) { CGEventTapEnable(tap, true); return YES; }
    CGEventMask mask = CGEventMaskBit(kCGEventFlagsChanged) | CGEventMaskBit(kCGEventKeyDown) | CGEventMaskBit(kCGEventKeyUp);
    tap = CGEventTapCreate(kCGSessionEventTap, kCGHeadInsertEventTap, kCGEventTapOptionListenOnly, mask, observe, NULL);
    if (!tap) return NO;
    source = CFMachPortCreateRunLoopSource(kCFAllocatorDefault, tap, 0);
    CFRunLoopAddSource(CFRunLoopGetMain(), source, kCFRunLoopCommonModes);
    CGEventTapEnable(tap, true);
    return YES;
}
int vara_shortcut_capture(void (*handler)(uint16_t, bool)) {
    __block int result = 0;
    void (^work)(void) = ^{ if (ensure_tap()) { capture = handler; memset(pressed,0,sizeof(pressed)); result = 1; } };
    if ([NSThread isMainThread]) work(); else dispatch_sync(dispatch_get_main_queue(), work);
    return result;
}
void vara_shortcut_capture_end(void) {
    void (^work)(void) = ^{ capture = NULL; memset(pressed,0,sizeof(pressed)); active=NO; if (!callback) vara_shortcut_clear(); };
    if ([NSThread isMainThread]) work(); else dispatch_sync(dispatch_get_main_queue(), work);
}
int vara_shortcut_set(const uint16_t *keys, size_t count, void (*handler)(bool)) {
    __block int result = 0;
    void (^work)(void) = ^{
        if (!ensure_tap()) return;
        memset(wanted, 0, sizeof(wanted)); memset(pressed, 0, sizeof(pressed)); active = NO;
        for (size_t i=0; i<count; i++) if (keys[i]<128) wanted[keys[i]]=YES;
        callback = handler; result = 1;
    };
    if ([NSThread isMainThread]) work(); else dispatch_sync(dispatch_get_main_queue(), work);
    return result;
}
void vara_shortcut_clear(void) {
    void (^work)(void) = ^{
        if (source) { CFRunLoopRemoveSource(CFRunLoopGetMain(), source, kCFRunLoopCommonModes); CFRelease(source); source=NULL; }
        if (tap) { CFMachPortInvalidate(tap); CFRelease(tap); tap=NULL; }
        memset(wanted,0,sizeof(wanted)); memset(pressed,0,sizeof(pressed)); active=NO; callback=NULL; capture=NULL;
    };
    if ([NSThread isMainThread]) work(); else dispatch_sync(dispatch_get_main_queue(), work);
}
