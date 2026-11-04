// In-process inspection of panels created by the RFD fixture; no external UI automation.
#import <AppKit/AppKit.h>
static NSString *expectedMessage, *expectedExtension, *expectedDirectory;
static int result, attempts;
static void inspect(void) {
    NSSavePanel *panel=nil;
    for (NSWindow *window in NSApp.windows) {
        if (![window isKindOfClass:NSSavePanel.class] || !window.visible) continue;
        NSSavePanel *candidate=(NSSavePanel *)window;
        if (expectedMessage && ![candidate.message isEqualToString:expectedMessage]) continue;
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wdeprecated-declarations"
        if (expectedExtension && ![candidate.allowedFileTypes containsObject:expectedExtension]) continue;
#pragma clang diagnostic pop
        panel=candidate; break;
    }
    if (!panel) {
        if (++attempts < 100) dispatch_after(dispatch_time(DISPATCH_TIME_NOW,100000000),dispatch_get_main_queue(),^{inspect();});
        else { result=-1; [NSApp stopModalWithCode:NSModalResponseCancel]; }
        return;
    }
    BOOL messageOK=!expectedMessage || [panel.message isEqualToString:expectedMessage];
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wdeprecated-declarations"
    NSArray *types=panel.allowedFileTypes;
#pragma clang diagnostic pop
    BOOL extensionOK=!expectedExtension || [types containsObject:expectedExtension];
    NSString *actualDirectory=panel.directoryURL.path.stringByStandardizingPath;
    NSString *fixtureDirectory=expectedDirectory.stringByStandardizingPath;
    BOOL directoryOK=[actualDirectory isEqualToString:fixtureDirectory] ||
        (expectedExtension && [actualDirectory isEqualToString:[fixtureDirectory stringByAppendingPathComponent:@"fixture.txt"]] && [panel.nameFieldStringValue isEqualToString:@"fixture.txt"]);
    result=messageOK && extensionOK && directoryOK ? 1 : -2;
    fprintf(stderr,"panel message=%s types=%s directory=%s result=%d\n",panel.message.UTF8String,types.description.UTF8String,panel.directoryURL.path.UTF8String,result);
    [panel cancel:nil];
}
void vara_fixture_probe_panel(const char *message, const char *extension, const char *directory) {
    expectedMessage=message?[NSString stringWithUTF8String:message]:nil;
    expectedExtension=extension?[NSString stringWithUTF8String:extension]:nil;
    expectedDirectory=[NSString stringWithUTF8String:directory];result=0;attempts=0;
    dispatch_after(dispatch_time(DISPATCH_TIME_NOW,200000000),dispatch_get_main_queue(),^{inspect();});
}
int vara_fixture_panel_result(void) { return result; }
static BOOL tooltip_in_view(NSView *view, NSString *expected) {
    if ([view isKindOfClass:NSStatusBarButton.class] && [view.toolTip isEqualToString:expected]) return YES;
    for (NSView *child in view.subviews) if (tooltip_in_view(child,expected)) return YES;
    return NO;
}
int vara_fixture_tray_tooltip(const char *expected) {
    NSString *value=[NSString stringWithUTF8String:expected];
    for (NSWindow *window in NSApp.windows) if (tooltip_in_view(window.contentView,value)) return 1;
    return 0;
}
