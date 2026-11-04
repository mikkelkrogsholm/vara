// Exercises the actual HUD implementation in an isolated AppKit process.
// No microphone, permissions, app profile or global shortcuts are accessed.
#import <AppKit/AppKit.h>
#import <assert.h>
#import "../../crates/vara-platform/src/macos_hud.m"
static int cancellations, stops;
static void cancel_test(void) { cancellations++; }
static void stop_test(void) { stops++; }
static float level_test(void) { return 0.4f; }
static void pump(void) { for (int i=0;i<10;i++) [[NSRunLoop mainRunLoop] runUntilDate:[NSDate dateWithTimeIntervalSinceNow:0.01]]; }
static NSString *message(NSDictionary *catalog, NSString *code, NSString *locale) { return catalog[code][locale]; }
static void show(NSDictionary *c, NSString *lang, NSString *stage, NSString *detail, BOOL recording) {
    vara_hud("invalid.vara.fixture", "Fixture", message(c,stage,lang).UTF8String,
             message(c,detail,lang).UTF8String, message(c,@"hud.panel",lang).UTF8String,
             message(c,@"hud.stop",lang).UTF8String, message(c,@"hud.cancel",lang).UTF8String,
             recording, cancel_test, stop_test, level_test);
    pump();
}
static void snapshot(NSString *path) {
    [panel.contentView displayIfNeeded];
    NSBitmapImageRep *rep=[panel.contentView bitmapImageRepForCachingDisplayInRect:panel.contentView.bounds];
    [panel.contentView cacheDisplayInRect:panel.contentView.bounds toBitmapImageRep:rep];
    NSData *png=[rep representationUsingType:NSBitmapImageFileTypePNG properties:@{}];
    assert(png && [png writeToFile:path atomically:YES]);
    assert(rep.pixelsWide > 0 && rep.pixelsHigh > 0);
}
int main(int argc, const char **argv) {
    @autoreleasepool {
        assert(argc == 4);
        NSData *data=[NSData dataWithContentsOfFile:[NSString stringWithUTF8String:argv[1]]];
        NSDictionary *c=[NSJSONSerialization JSONObjectWithData:data options:0 error:NULL]; assert(c);
        [NSApplication sharedApplication]; [NSApp setActivationPolicy:NSApplicationActivationPolicyAccessory];
        BOOL active=NSApp.active; NSWindow *key=NSApp.keyWindow;
        show(c,@"da",@"hud.recording",@"hud.audio.local",YES);
        VaraPanel *identity=panel;
        assert(panel && panel.visible && !panel.canBecomeKeyWindow && !panel.canBecomeMainWindow);
        assert((panel.styleMask & NSWindowStyleMaskNonactivatingPanel) != 0);
        assert(NSApp.active == active && NSApp.keyWindow == key);
        assert([stageLabel.stringValue isEqualToString:@"Lytter"] && [detailLabel.stringValue isEqualToString:@"Lyd: på enheden"]);
        assert([panel.accessibilityLabel isEqualToString:@"Vara diktering"]);
        assert([stopButton.accessibilityLabel isEqualToString:@"Afslut diktering"] && [cancelButton.toolTip isEqualToString:@"Annuller diktering"]);
        assert(!stopButton.hidden); snapshot([NSString stringWithUTF8String:argv[2]]);
        show(c,@"en",@"hud.recording",@"hud.audio.local",YES);
        assert(panel == identity && NSApp.active == active && NSApp.keyWindow == key);
        assert([stageLabel.stringValue isEqualToString:@"Listening"] && [detailLabel.stringValue isEqualToString:@"Audio: on this device"]);
        assert([panel.accessibilityLabel isEqualToString:@"Vara dictation"]);
        assert([stopButton.accessibilityLabel isEqualToString:@"Stop dictation"] && [stopButton.toolTip isEqualToString:@"Stop dictation"]);
        assert([cancelButton.accessibilityLabel isEqualToString:@"Cancel dictation"] && [cancelButton.toolTip isEqualToString:@"Cancel dictation"]);
        assert([stopButton.image.accessibilityDescription isEqualToString:@"Stop dictation"] && [cancelButton.image.accessibilityDescription isEqualToString:@"Cancel dictation"]);
        snapshot([NSString stringWithUTF8String:argv[3]]);
        [control stop:nil]; [control cancel:nil]; assert(stops==1 && cancellations==1);
        show(c,@"en",@"hud.recognizing",@"hud.audio.cloud",NO);
        assert([stageLabel.stringValue isEqualToString:@"Finding your words"] && [detailLabel.stringValue isEqualToString:@"Audio: cloud"]);
        show(c,@"en",@"hud.processing",@"hud.ai.cloud.app",NO);
        assert(panel == identity && stopButton.hidden && [stageLabel.stringValue isEqualToString:@"Refining the text"]);
        assert([detailLabel.stringValue isEqualToString:@"AI: cloud · app prompt"]);
        show(c,@"da",@"hud.processing",@"hud.ai.cloud.app",NO);
        assert([stageLabel.stringValue isEqualToString:@"Tilpasser teksten"] && [detailLabel.stringValue isEqualToString:@"AI: cloud · app-prompt"]);
        vara_hide_hud(); pump(); assert(!panel.visible && stops==1 && cancellations==1);
        puts("PASS: actual AppKit HUD localized labels, accessibility, nonactivation, identity, callbacks, visibility, rendered PNG");
    }
}
