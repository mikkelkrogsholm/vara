#import <AppKit/AppKit.h>
#import <ApplicationServices/ApplicationServices.h>
#import <AVFoundation/AVFoundation.h>
#import <CoreAudio/CoreAudio.h>
#include <unistd.h>
#include "macos_text_input.h"
#include "macos_insertion_policy.h"

static NSMutableDictionary<NSString *, id> *targets;
static char *jsonString(id value) {
    NSData *data=[NSJSONSerialization dataWithJSONObject:value options:0 error:nil];
    return strdup([[NSString alloc] initWithData:data encoding:NSUTF8StringEncoding].UTF8String);
}
char *vara_preferred_languages(void) {
    @autoreleasepool { return jsonString(NSLocale.preferredLanguages); }
}
static NSDictionary *diagnostic(const char *reason, const char *method, AXError error) {
    return @{ @"reason":@(reason), @"method":method ? @(method) : NSNull.null,
        @"os_error":error==kAXErrorSuccess ? NSNull.null : @(error) };
}
static NSDictionary *outcome(const char *delivery, const char *reason, const char *method, AXError error) {
    return @{ @"delivery":@(delivery), @"diagnostic":diagnostic(reason,method,error) };
}
char *vara_capture_target(void) {
    @autoreleasepool {
        NSRunningApplication *app=NSWorkspace.sharedWorkspace.frontmostApplication;
        NSString *identity=app.bundleIdentifier ?: [NSString stringWithFormat:@"pid:%d",app.processIdentifier];
        NSString *name=app.localizedName ?: identity;
        NSString *token=nil;
        NSDictionary *failure=nil;
        if (!AXIsProcessTrusted()) failure=diagnostic("accessibility_missing",NULL,kAXErrorSuccess);
        else if (!app || app.terminated) failure=diagnostic("target_gone",NULL,kAXErrorSuccess);
        else if ([app.bundleIdentifier isEqualToString:NSBundle.mainBundle.bundleIdentifier])
            failure=diagnostic("capture_failed",NULL,kAXErrorSuccess);
        else {
            AXUIElementRef application=AXUIElementCreateApplication(app.processIdentifier);
            CFTypeRef field=NULL;
            AXError error=AXUIElementCopyAttributeValue(application,kAXFocusedUIElementAttribute,&field);
            if(error==kAXErrorSuccess && field) {
                @synchronized(NSApplication.class) {
                    if(!targets) targets=[NSMutableDictionary dictionary];
                    if(targets.count>=32) [targets removeAllObjects];
                    token=NSUUID.UUID.UUIDString;
                    targets[token]=@{@"field":CFBridgingRelease(field),@"pid":@(app.processIdentifier)};
                }
            } else {
                failure=diagnostic("capture_failed",NULL,error);
                if(field) CFRelease(field);
            }
            CFRelease(application);
        }
        return jsonString(@{@"app_id":identity,@"app_name":name,@"target_token":token ?: NSNull.null,
            @"capture_diagnostic":failure ?: NSNull.null});
    }
}
// Return only a closed reason plus a numeric AX code; never read target text.
static const char *focusFailure(pid_t pid, AXUIElementRef field, AXError *error) {
    bool trusted=AXIsProcessTrusted();
    NSRunningApplication *running=[NSRunningApplication runningApplicationWithProcessIdentifier:pid];
    bool alive=running && !running.terminated;
    bool sameApp=NSWorkspace.sharedWorkspace.frontmostApplication.processIdentifier==pid;
    if (!trusted || !alive || !sameApp)
        return vara_insertion_guard(trusted,true,alive,sameApp,true,true,true);
    AXUIElementRef app=AXUIElementCreateApplication(pid); CFTypeRef current=NULL;
    *error=AXUIElementCopyAttributeValue(app,kAXFocusedUIElementAttribute,&current); CFRelease(app);
    bool available=*error==kAXErrorSuccess && current;
    bool same=available && CFEqual(current,field);
    if(current) CFRelease(current);
    return vara_insertion_guard(true,true,true,true,available,same,true);
}
static BOOL roleIs(AXUIElementRef field, CFStringRef expected) {
    CFTypeRef role=NULL;
    BOOL matches=AXUIElementCopyAttributeValue(field,kAXRoleAttribute,&role)==kAXErrorSuccess && role && CFEqual(role,expected);
    if (role) CFRelease(role);
    return matches;
}
static BOOL webTextField(AXUIElementRef field) {
    AXUIElementRef current=(AXUIElementRef)CFRetain(field);
    for (int depth=0; depth<40 && current; depth++) {
        if (roleIs(current,CFSTR("AXWebArea"))) { CFRelease(current); return YES; }
        CFTypeRef parent=NULL;
        AXUIElementCopyAttributeValue(current,kAXParentAttribute,&parent); CFRelease(current);
        current=(AXUIElementRef)parent;
    }
    if (current) CFRelease(current);
    return NO;
}
static BOOL selectedRange(AXUIElementRef field, CFRange *range, AXError *error) {
    CFTypeRef value=NULL;
    AXError result=AXUIElementCopyAttributeValue(field,kAXSelectedTextRangeAttribute,&value);
    *error=result;
    BOOL valid=result==kAXErrorSuccess && value && CFGetTypeID(value)==AXValueGetTypeID()
        && AXValueGetType((AXValueRef)value)==kAXValueCFRangeType
        && AXValueGetValue((AXValueRef)value,kAXValueCFRangeType,range);
    if (value) CFRelease(value);
    return valid;
}
static NSDictionary *typeIntoField(pid_t pid, AXUIElementRef field, NSString *text) {
    const char *failure=vara_keyboard_guard(CGPreflightPostEventAccess(),true);
    if(failure) return outcome("manual_ready",failure,"keyboard",kAXErrorSuccess);
    CFTypeRef role=NULL;
    AXError error=AXUIElementCopyAttributeValue(field,kAXRoleAttribute,&role);
    BOOL supported=error==kAXErrorSuccess && role &&
        (CFEqual(role,kAXTextAreaRole) || CFEqual(role,kAXTextFieldRole) || CFEqual(role,kAXComboBoxRole));
    if(role) CFRelease(role);
    if(error!=kAXErrorSuccess) return outcome("manual_ready","field_unavailable","keyboard",error);
    failure=vara_insertion_guard(true,true,true,true,true,true,supported);
    if(failure) return outcome("manual_ready",failure,"keyboard",kAXErrorSuccess);
    AXError verificationError=kAXErrorSuccess;
    CFRange before; BOOL canVerify=selectedRange(field,&before,&verificationError);
    NSUInteger offset=0, length=text.length;
    while (offset<length) {
        AXError error=kAXErrorSuccess;
        failure=focusFailure(pid,field,&error);
        if(failure) return outcome(vara_interrupted_delivery(offset>0),failure,"keyboard",error);
        failure=vara_keyboard_guard(CGPreflightPostEventAccess(),true);
        if(failure) return outcome(vara_interrupted_delivery(offset>0),failure,"keyboard",kAXErrorSuccess);
        UniChar buffer[20]; NSUInteger count=MIN((NSUInteger)20,length-offset);
        [text getCharacters:buffer range:NSMakeRange(offset,count)];
        count=vara_text_chunk(buffer,length-offset);
        CGEventRef down=CGEventCreateKeyboardEvent(NULL,0,true), up=CGEventCreateKeyboardEvent(NULL,0,false);
        failure=vara_keyboard_guard(true,down && up);
        if(failure) { if(down)CFRelease(down); if(up)CFRelease(up);
            return outcome(vara_interrupted_delivery(offset>0),failure,"keyboard",kAXErrorSuccess); }
        CGEventSetFlags(down,0); CGEventSetFlags(up,0);
        CGEventSetIntegerValueField(down,kCGEventSourceUserData,VARA_TEXT_EVENT_MARKER);
        CGEventSetIntegerValueField(up,kCGEventSourceUserData,VARA_TEXT_EVENT_MARKER);
        CGEventKeyboardSetUnicodeString(down,count,buffer);
        CGEventKeyboardSetUnicodeString(up,count,buffer);
        // Recheck the exact field immediately before each chunk is submitted.
        failure=focusFailure(pid,field,&error);
        if(failure) { CFRelease(down); CFRelease(up);
            return outcome(vara_interrupted_delivery(offset>0),failure,"keyboard",error); }
        CGEventPostToPid(pid,down); CGEventPostToPid(pid,up);
        CFRelease(down); CFRelease(up); offset+=count;
    }
    // Queue submission is not acknowledgement. Only verify caret metadata, never text.
    if (canVerify) {
        for (int attempt=0; attempt<25; attempt++) {
            CFRange after; AXError error=kAXErrorSuccess;
            failure=focusFailure(pid,field,&error);
            if(failure) return outcome("unknown",failure,"keyboard",error);
            if (selectedRange(field,&after,&verificationError) && after.length==0 && after.location==before.location+(CFIndex)length)
                return outcome("inserted","confirmed","keyboard",kAXErrorSuccess);
            usleep(10000);
        }
    }
    return outcome("unknown","handoff_unconfirmed","keyboard",verificationError);
}
// Address only the captured field; never clipboard, activation, or automatic replay.
char *vara_insert(const char *tokenText,const char *text) {
    @autoreleasepool {
        NSString *token=[NSString stringWithUTF8String:tokenText];
        NSDictionary *target; @synchronized(NSApplication.class) {target=targets[token];}
        const char *failure=vara_insertion_guard(AXIsProcessTrusted(),target!=nil,true,true,true,true,true);
        if(failure) return jsonString(outcome("manual_ready",failure,NULL,kAXErrorSuccess));
        pid_t pid=[target[@"pid"] intValue];
        AXUIElementRef field=(__bridge AXUIElementRef)target[@"field"];
        AXError error=kAXErrorSuccess;
        failure=focusFailure(pid,field,&error);
        if(failure) return jsonString(outcome("manual_ready",failure,NULL,error));
        NSString *value=[NSString stringWithUTF8String:text];
        if (!value.length) return jsonString(outcome("manual_ready","dispatch_failed",NULL,kAXErrorSuccess));
        if (webTextField(field)) return jsonString(typeIntoField(pid,field,value));
        Boolean writable=false;
        AXError settable=AXUIElementIsAttributeSettable(field,kAXSelectedTextAttribute,&writable);
        if(settable!=kAXErrorSuccess || !writable) return jsonString(typeIntoField(pid,field,value));
        failure=focusFailure(pid,field,&error);
        if(failure) return jsonString(outcome("manual_ready",failure,"accessibility",error));
        AXError sent=AXUIElementSetAttributeValue(field,kAXSelectedTextAttribute,(__bridge CFTypeRef)value);
        // Once AX was invoked, an error cannot prove that the target was unchanged.
        return jsonString(outcome(sent==kAXErrorSuccess ? "inserted" : "unknown",
            sent==kAXErrorSuccess ? "confirmed" : "handoff_unconfirmed","accessibility",sent));
    }
}

void vara_release_target(const char *token) {@autoreleasepool{@synchronized(NSApplication.class){[targets removeObjectForKey:[NSString stringWithUTF8String:token]];}}}
bool vara_post_event_access(void) { return CGPreflightPostEventAccess(); }
bool vara_input_monitoring(void) { return CGPreflightListenEventAccess(); }
void vara_request_input_monitoring(void) {
    dispatch_async(dispatch_get_main_queue(), ^{
        CGRequestListenEventAccess();
        [NSWorkspace.sharedWorkspace openURL:[NSURL URLWithString:@"x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent"]];
    });
}
int vara_accessibility(void) {return AXIsProcessTrusted()?1:0;}
void vara_request_accessibility(void) {dispatch_async(dispatch_get_main_queue(),^{
    AXIsProcessTrustedWithOptions((__bridge CFDictionaryRef)@{(__bridge NSString *)kAXTrustedCheckOptionPrompt:@YES});
    [NSWorkspace.sharedWorkspace openURL:[NSURL URLWithString:@"x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"]];
});}
void vara_request_post_event(void) {dispatch_async(dispatch_get_main_queue(),^{
    CGRequestPostEventAccess();
    [NSWorkspace.sharedWorkspace openURL:[NSURL URLWithString:@"x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"]];
});}
int vara_microphone(void) {switch([AVCaptureDevice authorizationStatusForMediaType:AVMediaTypeAudio]){case AVAuthorizationStatusAuthorized:return 1;case AVAuthorizationStatusNotDetermined:return 0;default:return -1;}}
void vara_request_microphone(void) {
    if([AVCaptureDevice authorizationStatusForMediaType:AVMediaTypeAudio]==AVAuthorizationStatusDenied){dispatch_async(dispatch_get_main_queue(),^{[NSWorkspace.sharedWorkspace openURL:[NSURL URLWithString:@"x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone"]];});}
    else [AVCaptureDevice requestAccessForMediaType:AVMediaTypeAudio completionHandler:^(BOOL granted){(void)granted;}];
}
void vara_free(char *value){free(value);}
char *vara_applications(void) {
    @autoreleasepool {
        NSMutableArray *result=[NSMutableArray array];NSMutableSet *seen=[NSMutableSet set];
        for(NSString *directory in @[@"/Applications",@"/System/Applications",@"/System/Applications/Utilities",[NSHomeDirectory() stringByAppendingPathComponent:@"Applications"]]){
            for(NSString *file in [NSFileManager.defaultManager contentsOfDirectoryAtPath:directory error:nil]){
                if(![file.pathExtension isEqualToString:@"app"])continue;
                NSBundle *bundle=[NSBundle bundleWithPath:[directory stringByAppendingPathComponent:file]];NSString *identifier=bundle.bundleIdentifier;
                if(!identifier || [seen containsObject:identifier])continue;[seen addObject:identifier];
                NSString *name=[bundle objectForInfoDictionaryKey:@"CFBundleDisplayName"] ?: [bundle objectForInfoDictionaryKey:@"CFBundleName"] ?: file.stringByDeletingPathExtension;
                [result addObject:@{@"id":identifier,@"label":name}];
            }
        }
        return jsonString(result);
    }
}

// Resolve the installed/running application's actual Finder icon by bundle ID.
// The caller cannot supply arbitrary filesystem paths.
char *vara_application_icons(const char *identifiers) {
    @autoreleasepool {
        NSData *input=[NSData dataWithBytes:identifiers length:strlen(identifiers)];
        NSArray *ids=[NSJSONSerialization JSONObjectWithData:input options:0 error:NULL];
        NSMutableDictionary *result=[NSMutableDictionary dictionary];
        for(NSString *identifier in ids) {
            NSURL *url=nil;
            for(NSRunningApplication *app in [NSRunningApplication runningApplicationsWithBundleIdentifier:identifier]) {
                if(app.bundleURL) { url=app.bundleURL; break; }
            }
            if(!url) url=[NSWorkspace.sharedWorkspace URLForApplicationWithBundleIdentifier:identifier];
            if(!url) continue;
            NSImage *image=[[NSWorkspace.sharedWorkspace iconForFile:url.path] copy];
            NSBitmapImageRep *bitmap=[[NSBitmapImageRep alloc] initWithBitmapDataPlanes:NULL pixelsWide:96 pixelsHigh:96 bitsPerSample:8 samplesPerPixel:4 hasAlpha:YES isPlanar:NO colorSpaceName:NSDeviceRGBColorSpace bytesPerRow:0 bitsPerPixel:0];
            NSGraphicsContext *context=[NSGraphicsContext graphicsContextWithBitmapImageRep:bitmap];
            if(!image || !context) continue;
            [NSGraphicsContext saveGraphicsState];
            NSGraphicsContext.currentContext=context;
            [image drawInRect:NSMakeRect(0,0,96,96) fromRect:NSZeroRect operation:NSCompositingOperationCopy fraction:1 respectFlipped:NO hints:nil];
            [NSGraphicsContext restoreGraphicsState];
            NSData *png=[bitmap representationUsingType:NSBitmapImageFileTypePNG properties:@{}];
            if(png) result[identifier]=[@"data:image/png;base64," stringByAppendingString:[png base64EncodedStringWithOptions:0]];
        }
        return jsonString(result);
    }
}

// Enumerate input stream metadata without creating an AudioUnit for every device.
// Some third-party drivers block while a supported-format probe opens their I/O.
char *vara_input_devices(void) {
    @autoreleasepool {
        AudioObjectPropertyAddress all={kAudioHardwarePropertyDevices,kAudioObjectPropertyScopeGlobal,kAudioObjectPropertyElementMain};
        UInt32 bytes=0;
        if(AudioObjectGetPropertyDataSize(kAudioObjectSystemObject,&all,0,NULL,&bytes)!=noErr) return NULL;
        AudioDeviceID *devices=malloc(bytes);
        if(!devices) return NULL;
        if(AudioObjectGetPropertyData(kAudioObjectSystemObject,&all,0,NULL,&bytes,devices)!=noErr) {free(devices);return NULL;}
        NSMutableArray *result=[NSMutableArray array];
        for(UInt32 i=0;i<bytes/sizeof(AudioDeviceID);i++) {
            AudioObjectPropertyAddress streams={kAudioDevicePropertyStreams,kAudioDevicePropertyScopeInput,kAudioObjectPropertyElementMain};
            UInt32 streamBytes=0;
            if(AudioObjectGetPropertyDataSize(devices[i],&streams,0,NULL,&streamBytes)!=noErr || streamBytes==0) continue;
            AudioObjectPropertyAddress title={kAudioObjectPropertyName,kAudioObjectPropertyScopeGlobal,kAudioObjectPropertyElementMain};
            CFStringRef value=NULL; UInt32 size=sizeof(value);
            if(AudioObjectGetPropertyData(devices[i],&title,0,NULL,&size,&value)==noErr && value) {
                NSString *name=CFBridgingRelease(value);
                [result addObject:@{@"id":name,@"label":name}];
            }
        }
        free(devices);
        return jsonString(result);
    }
}
