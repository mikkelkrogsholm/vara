#import <AppKit/AppKit.h>
#import <ApplicationServices/ApplicationServices.h>
#import <AVFoundation/AVFoundation.h>
#import <CoreAudio/CoreAudio.h>
#include <unistd.h>
#include "macos_text_input.h"

static NSMutableDictionary<NSString *, id> *targets;
static char *jsonString(id value) {
    NSData *data=[NSJSONSerialization dataWithJSONObject:value options:0 error:nil];
    return strdup([[NSString alloc] initWithData:data encoding:NSUTF8StringEncoding].UTF8String);
}
char *vara_capture_target(void) {
    @autoreleasepool {
        NSRunningApplication *app=NSWorkspace.sharedWorkspace.frontmostApplication;
        NSString *identity=app.bundleIdentifier ?: [NSString stringWithFormat:@"pid:%d",app.processIdentifier];
        NSString *name=app.localizedName ?: @"Ukendt app";
        NSString *token=nil;
        if(AXIsProcessTrusted() && ![app.bundleIdentifier isEqualToString:NSBundle.mainBundle.bundleIdentifier]) {
            AXUIElementRef application=AXUIElementCreateApplication(app.processIdentifier);
            CFTypeRef field=NULL;
            if(AXUIElementCopyAttributeValue(application,kAXFocusedUIElementAttribute,&field)==kAXErrorSuccess && field) {
                @synchronized(NSApplication.class) {
                    if(!targets) targets=[NSMutableDictionary dictionary];
                    if(targets.count>=32) [targets removeAllObjects];
                    token=NSUUID.UUID.UUIDString;
                    targets[token]=@{@"field":CFBridgingRelease(field),@"pid":@(app.processIdentifier)};
                }
            }
            CFRelease(application);
        }
        return jsonString(@{@"app_id":identity,@"app_name":name,@"target_token":token ?: NSNull.null});
    }
}
static BOOL sameFocusedField(pid_t pid, AXUIElementRef field) {
    if (NSWorkspace.sharedWorkspace.frontmostApplication.processIdentifier != pid) return NO;
    AXUIElementRef app=AXUIElementCreateApplication(pid); CFTypeRef current=NULL;
    AXError result=AXUIElementCopyAttributeValue(app,kAXFocusedUIElementAttribute,&current); CFRelease(app);
    BOOL same=result==kAXErrorSuccess && current && CFEqual(current,field);
    if (current) CFRelease(current);
    return same;
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
static BOOL selectedRange(AXUIElementRef field, CFRange *range) {
    CFTypeRef value=NULL;
    AXError result=AXUIElementCopyAttributeValue(field,kAXSelectedTextRangeAttribute,&value);
    BOOL valid=result==kAXErrorSuccess && value && CFGetTypeID(value)==AXValueGetTypeID()
        && AXValueGetType((AXValueRef)value)==kAXValueCFRangeType
        && AXValueGetValue((AXValueRef)value,kAXValueCFRangeType,range);
    if (value) CFRelease(value);
    return valid;
}
static int typeIntoField(pid_t pid, AXUIElementRef field, NSString *text) {
    if (!CGPreflightPostEventAccess()) return 0;
    // A generic web area or button is never an insertion destination.
    if (!roleIs(field,kAXTextAreaRole) && !roleIs(field,kAXTextFieldRole) && !roleIs(field,kAXComboBoxRole)) return 0;
    CFRange before; BOOL canVerify=selectedRange(field,&before);
    NSUInteger offset=0, length=text.length;
    while (offset<length) {
        if (!sameFocusedField(pid,field)) return offset ? -1 : 0;
        UniChar buffer[20]; NSUInteger count=MIN((NSUInteger)20,length-offset);
        [text getCharacters:buffer range:NSMakeRange(offset,count)];
        count=vara_text_chunk(buffer,length-offset);
        CGEventRef down=CGEventCreateKeyboardEvent(NULL,0,true), up=CGEventCreateKeyboardEvent(NULL,0,false);
        if (!down || !up) { if(down)CFRelease(down); if(up)CFRelease(up); return offset ? -1 : 0; }
        CGEventSetFlags(down,0); CGEventSetFlags(up,0);
        CGEventSetIntegerValueField(down,kCGEventSourceUserData,VARA_TEXT_EVENT_MARKER);
        CGEventSetIntegerValueField(up,kCGEventSourceUserData,VARA_TEXT_EVENT_MARKER);
        CGEventKeyboardSetUnicodeString(down,count,buffer);
        CGEventKeyboardSetUnicodeString(up,count,buffer);
        CGEventPostToPid(pid,down); CGEventPostToPid(pid,up);
        CFRelease(down); CFRelease(up); offset+=count;
    }
    // Queue submission is not delivery acknowledgement. Confirm caret movement,
    // without reading or retaining the destination field's text.
    if (canVerify) {
        for (int attempt=0; attempt<25; attempt++) {
            CFRange after;
            if (!sameFocusedField(pid,field)) return -1;
            if (selectedRange(field,&after) && after.length==0 && after.location==before.location+(CFIndex)length) return 1;
            usleep(10000);
        }
    }
    return -1;
}
// Address the captured field; keyboard fallback is targeted and never uses the clipboard.
int vara_insert(const char *tokenText,const char *text) {
    @autoreleasepool {
        NSString *token=[NSString stringWithUTF8String:tokenText];
        NSDictionary *target; @synchronized(NSApplication.class) {target=targets[token];}
        if(!target) return 0;
        pid_t pid=[target[@"pid"] intValue];
        AXUIElementRef field=(__bridge AXUIElementRef)target[@"field"];
        if (!sameFocusedField(pid,field)) return 0;
        NSString *value=[NSString stringWithUTF8String:text];
        if (!value.length) return 0;
        // Some browser editors accept AXSelectedText but do not update their editor.
        // Use their normal text-input path from the outset, avoiding duplicate fallback.
        if (webTextField(field)) return typeIntoField(pid,field,value);
        Boolean writable=false;
        if(AXUIElementIsAttributeSettable(field,kAXSelectedTextAttribute,&writable)!=kAXErrorSuccess || !writable)
            return typeIntoField(pid,field,value);
        AXError sent=AXUIElementSetAttributeValue(field,kAXSelectedTextAttribute,(__bridge CFTypeRef)value);
        return sent==kAXErrorSuccess ? 1 : -1;
    }
}

void vara_release_target(const char *token) {@autoreleasepool{@synchronized(NSApplication.class){[targets removeObjectForKey:[NSString stringWithUTF8String:token]];}}}
bool vara_input_monitoring(void) { return CGPreflightListenEventAccess(); }
void vara_request_input_monitoring(void) {
    dispatch_async(dispatch_get_main_queue(), ^{
        CGRequestListenEventAccess();
        [NSWorkspace.sharedWorkspace openURL:[NSURL URLWithString:@"x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent"]];
    });
}
int vara_accessibility(void) {return AXIsProcessTrusted()?1:0;}
void vara_request_accessibility(void) {dispatch_async(dispatch_get_main_queue(),^{AXIsProcessTrustedWithOptions((__bridge CFDictionaryRef)@{(__bridge NSString *)kAXTrustedCheckOptionPrompt:@YES});});}
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
