// Inspect an isolated fixture bundle; never requests microphone access.
#import <Foundation/Foundation.h>
#import <assert.h>
int main(int argc, const char **argv) {
    @autoreleasepool {
        assert(argc >= 3);
        NSBundle *bundle=[NSBundle bundleWithPath:[NSString stringWithUTF8String:argv[1]]]; assert(bundle);
        NSString *wanted=[NSString stringWithUTF8String:argv[2]];
        NSString *actual=bundle.localizedInfoDictionary[@"NSMicrophoneUsageDescription"];
        if (!actual || ![actual isEqualToString:wanted]) { fprintf(stderr,"Unexpected localization: %s, preferred %s\n",actual.UTF8String, bundle.preferredLocalizations.description.UTF8String); return 1; }
        printf("PASS: NSBundle localized usage description: %s\n", actual.UTF8String);
    }
}
