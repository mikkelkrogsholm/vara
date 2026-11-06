// Isolated source-rendered HUD reference. No Vara profile, microphone or shortcut access.
#import <AppKit/AppKit.h>
#import "../../../crates/vara-platform/src/macos_hud.m"
static float level_test(void) { return 0.4f; }
static void pump(void) { for(int i=0;i<8;i++) [[NSRunLoop mainRunLoop] runUntilDate:[NSDate dateWithTimeIntervalSinceNow:0.01]]; }
int main(int argc,const char **argv) { @autoreleasepool {
    if(argc!=3)return 2;
    NSDictionary *catalog=[NSJSONSerialization JSONObjectWithData:[NSData dataWithContentsOfFile:[NSString stringWithUTF8String:argv[1]]] options:0 error:NULL];
    [NSApplication sharedApplication];[NSApp setActivationPolicy:NSApplicationActivationPolicyAccessory];
    NSArray *states=@[@[@"18-hud-recording",@"hud.recording",@"hud.audio.local"],@[@"19-hud-transcribing",@"hud.recognizing",@"hud.audio.local"],@[@"20-hud-processing",@"hud.processing",@"hud.ai.local.general"]];
    for(NSArray *state in states){
        NSString *title=catalog[state[1]][@"en"],*detail=catalog[state[2]][@"en"];
        NSCAssert(title&&detail,@"runtime labels exist");
        vara_hud("com.apple.mail","Mail",title.UTF8String,detail.UTF8String,"Vara dictation","Stop dictation","Cancel dictation",[state[0] isEqual:states[0][0]],NULL,NULL,level_test);pump();
        printf("%s font %s family %s size %.2f ascent %.2f descent %.2f baseline %.2f\n",[state[0] UTF8String],stageLabel.font.fontName.UTF8String,stageLabel.font.familyName.UTF8String,stageLabel.font.pointSize,stageLabel.font.ascender,stageLabel.font.descender,stageLabel.baselineOffsetFromBottom);
        [panel.contentView displayIfNeeded];NSBitmapImageRep *rep=[panel.contentView bitmapImageRepForCachingDisplayInRect:panel.contentView.bounds];[panel.contentView cacheDisplayInRect:panel.contentView.bounds toBitmapImageRep:rep];
        NSString *path=[[NSString stringWithUTF8String:argv[2]] stringByAppendingPathComponent:[state[0] stringByAppendingString:@".png"]];
        NSCAssert([[rep representationUsingType:NSBitmapImageFileTypePNG properties:@{}] writeToFile:path atomically:YES],@"snapshot writes");
    }
    vara_hide_hud();pump();puts("PASS: three source-rendered AppKit HUD references; synthetic public Mail origin; no profile or recording access");
}return 0;}
