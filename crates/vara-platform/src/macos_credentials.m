// Login-keychain adapter. Never request secret data for a presence check.
#import <Foundation/Foundation.h>
#import <Security/Security.h>
#import <unistd.h>

// A command-line worker must not turn the signed app into a credential oracle.
// Its actual parent must satisfy this executable's designated signing requirement.
bool vara_credential_parent_is_trusted(void) {
    SecCodeRef self = NULL, parent = NULL;
    SecRequirementRef requirement = NULL;
    OSStatus status = SecCodeCopySelf(kSecCSDefaultFlags, &self);
    if (status == errSecSuccess) status = SecCodeCopyDesignatedRequirement(self, kSecCSDefaultFlags, &requirement);
    NSDictionary *attributes = @{(__bridge id)kSecGuestAttributePid: @(getppid())};
    if (status == errSecSuccess) status = SecCodeCopyGuestWithAttributes(NULL, (__bridge CFDictionaryRef)attributes, kSecCSDefaultFlags, &parent);
    if (status == errSecSuccess) status = SecCodeCheckValidity(parent, kSecCSStrictValidate, requirement);
    if (requirement) CFRelease(requirement);
    if (parent) CFRelease(parent);
    if (self) CFRelease(self);
    return status == errSecSuccess;
}

static OSStatus privateAccess(SecAccessRef *access) {
    SecTrustedApplicationRef app = NULL;
    OSStatus status = SecTrustedApplicationCreateFromPath(NULL, &app);
    if (status == errSecSuccess) {
        const void *apps[] = {app};
        CFArrayRef trusted = CFArrayCreate(NULL, apps, 1, &kCFTypeArrayCallBacks);
        status = SecAccessCreate(CFSTR("Vara API-nøgle"), trusted, access);
        CFRelease(trusted);
        CFRelease(app);
    }
    return status;
}

char *vara_credential(const char *operation, const char *account, const char *secret) {
    @autoreleasepool {
        // No security dialogs can linger in a worker. Unlock/permission problems
        // are returned to the UI; the parent also enforces a hard deadline.
        SecKeychainSetUserInteractionAllowed(false);
        SecKeychainRef keychain = NULL;
        OSStatus status = SecKeychainCopyDefault(&keychain);
        SecKeychainStatus state = 0;
        if (status == errSecSuccess) status = SecKeychainGetStatus(keychain, &state);
        if (status == errSecSuccess && !(state & kSecUnlockStateStatus)) status = errSecInteractionNotAllowed;
        NSString *op = @(operation);
        NSString *identifier = @(account);
        NSString *value = nil;
        bool exists = false;
        SecKeychainItemRef item = NULL;
        if (status == errSecSuccess) {
            NSMutableDictionary *query = [@{
                (__bridge id)kSecClass: (__bridge id)kSecClassGenericPassword,
                (__bridge id)kSecAttrService: @"dk.vara.desktop",
                (__bridge id)kSecAttrAccount: identifier,
                (__bridge id)kSecMatchSearchList: @[(__bridge id)keychain],
                (__bridge id)kSecMatchLimit: (__bridge id)kSecMatchLimitOne,
                (__bridge id)kSecUseAuthenticationUI: (__bridge id)kSecUseAuthenticationUIFail
            } mutableCopy];
            CFTypeRef result = NULL;
            if ([op isEqualToString:@"status"]) {
                query[(__bridge id)kSecReturnAttributes] = @YES;
                status = SecItemCopyMatching((__bridge CFDictionaryRef)query, &result);
                exists = status == errSecSuccess;
            } else {
                query[(__bridge id)kSecReturnRef] = @YES;
                status = SecItemCopyMatching((__bridge CFDictionaryRef)query, &result);
                if (status == errSecSuccess) item = (SecKeychainItemRef)CFRetain(result);
            }
            if (result) CFRelease(result);
        }
        if ([op isEqualToString:@"set"] && (status == errSecSuccess || status == errSecItemNotFound)) {
            SecAccessRef access = NULL;
            status = privateAccess(&access);
            UInt32 length = (UInt32)strlen(secret);
            if (status == errSecSuccess && item) {
                // Also harden an existing entry when the user replaces its key.
                status = SecKeychainItemSetAccess(item, access);
                if (status == errSecSuccess) status = SecKeychainItemModifyAttributesAndData(item, NULL, length, secret);
            } else if (status == errSecSuccess) {
                const char *service = "dk.vara.desktop";
                SecKeychainAttribute attrs[] = {
                    {kSecServiceItemAttr, (UInt32)strlen(service), (void *)service},
                    {kSecAccountItemAttr, (UInt32)strlen(account), (void *)account}
                };
                SecKeychainAttributeList list = {2, attrs};
                status = SecKeychainItemCreateFromContent(kSecGenericPasswordItemClass, &list, length, secret, keychain, access, NULL);
            }
            if (access) CFRelease(access);
        } else if ([op isEqualToString:@"get"] && status == errSecSuccess) {
            UInt32 length = 0; void *bytes = NULL;
            status = SecKeychainItemCopyAttributesAndData(item, NULL, NULL, NULL, &length, &bytes);
            if (status == errSecSuccess) value = [[NSString alloc] initWithBytes:bytes length:length encoding:NSUTF8StringEncoding];
            if (bytes) SecKeychainItemFreeAttributesAndData(NULL, bytes);
            if (status == errSecSuccess && !value) status = errSecDecode;
        } else if ([op isEqualToString:@"delete"] && status == errSecSuccess) {
            status = SecKeychainItemDelete(item);
        }
        if (item) CFRelease(item);
        if (keychain) CFRelease(keychain);
        NSDictionary *reply = @{@"status": @(status), @"exists": @(exists), @"value": value ?: [NSNull null]};
        NSData *json = [NSJSONSerialization dataWithJSONObject:reply options:0 error:NULL];
        return strdup([[NSString alloc] initWithData:json encoding:NSUTF8StringEncoding].UTF8String);
    }
}
