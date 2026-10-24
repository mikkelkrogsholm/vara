// Native state-machine tests. Creates event objects; never posts input or installs a tap.
#import "../../crates/vara-platform/src/macos_shortcut.m"
#include <assert.h>
#include <stdio.h>
static int downs, ups;
static void result(bool down) { if (down) downs++; else ups++; }
static void reset_keys(void) {
    memset(wanted,0,sizeof(wanted)); memset(pressed,0,sizeof(pressed));
    active=NO; downs=ups=0; callback=result;
}
static void flags(uint16_t key, CGEventFlags value) {
    CGEventRef event=CGEventCreateKeyboardEvent(NULL,key,true);
    CGEventSetFlags(event,value); observe(NULL,kCGEventFlagsChanged,event,NULL); CFRelease(event);
}
static void key(uint16_t code, bool down, bool repeat) {
    CGEventRef event=CGEventCreateKeyboardEvent(NULL,code,down);
    CGEventSetIntegerValueField(event,kCGKeyboardEventAutorepeat,repeat);
    observe(NULL,down?kCGEventKeyDown:kCGEventKeyUp,event,NULL); CFRelease(event);
}
int main(void) {
    @autoreleasepool {
        reset_keys(); wanted[58]=wanted[55]=YES;
        flags(58,NX_DEVICELALTKEYMASK|kCGEventFlagMaskAlternate);
        assert(downs==0);
        flags(55,NX_DEVICELALTKEYMASK|NX_DEVICELCMDKEYMASK|kCGEventFlagMaskAlternate|kCGEventFlagMaskCommand);
        assert(downs==1 && ups==0);
        // Repeated flag transitions cannot toggle a held chord twice.
        flags(55,NX_DEVICELALTKEYMASK|NX_DEVICELCMDKEYMASK|kCGEventFlagMaskAlternate|kCGEventFlagMaskCommand);
        assert(downs==1);
        flags(58,NX_DEVICELCMDKEYMASK|kCGEventFlagMaskCommand);
        assert(ups==1);
        flags(55,0); assert(ups==1);
        reset_keys(); wanted[58]=wanted[55]=YES;
        flags(61,NX_DEVICERALTKEYMASK|kCGEventFlagMaskAlternate);
        flags(55,NX_DEVICERALTKEYMASK|NX_DEVICELCMDKEYMASK|kCGEventFlagMaskAlternate|kCGEventFlagMaskCommand);
        assert(downs==0); // Right Option does not match Left Option.
        reset_keys(); wanted[0]=wanted[11]=YES;
        key(0,true,false); assert(downs==0);
        key(11,true,false); assert(downs==1);
        key(11,true,true); assert(downs==1);
        key(0,false,false); assert(ups==1);
        key(11,false,false); assert(ups==1);
        reset_keys(); wanted[0]=YES;
        CGEventRef own=CGEventCreateKeyboardEvent(NULL,0,true);
        CGEventSetIntegerValueField(own,kCGEventSourceUserData,VARA_TEXT_EVENT_MARKER);
        observe(NULL,kCGEventKeyDown,own,NULL); CFRelease(own);
        assert(downs==0 && !pressed[0]); // Inserting text must not trigger a single-A shortcut.
        puts("PASS: left Option+Command, side distinction, arbitrary chord, repeat and release");
    }
    return 0;
}
