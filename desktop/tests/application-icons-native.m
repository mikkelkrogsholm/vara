// Local icon lookup only; no application content or user history is read.
#import <AppKit/AppKit.h>
extern char *vara_application_icons(const char *);
int main(void) {
    @autoreleasepool {
        char *raw=vara_application_icons("[\"com.apple.TextEdit\",\"dk.vara.desktop\",\"invalid.vara.nonexistent-app\"]");
        NSDictionary *icons=[NSJSONSerialization JSONObjectWithData:[NSData dataWithBytes:raw length:strlen(raw)] options:0 error:NULL];
        free(raw);
        assert(icons[@"invalid.vara.nonexistent-app"] == nil);
        for(NSString *identifier in @[@"com.apple.TextEdit", @"dk.vara.desktop"]) {
            NSString *src=icons[identifier];
            assert([src hasPrefix:@"data:image/png;base64,"]);
            NSData *png=[[NSData alloc] initWithBase64EncodedString:[src substringFromIndex:22] options:0];
            NSBitmapImageRep *image=[NSBitmapImageRep imageRepWithData:png];
            assert(image.pixelsWide==96 && image.pixelsHigh==96);
            bool visible=false;
            for(int x=0;x<96 && !visible;x++) for(int y=0;y<96 && !visible;y++) visible=[image colorAtX:x y:y].alphaComponent>0;
            assert(visible);
        }
        puts("PASS: real installed/running app icons, non-empty 96px PNG, missing-app fallback");
    }
}
