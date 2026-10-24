// Real disposable keychain checks; never addresses a user credential.
#import <Foundation/Foundation.h>
#import <Security/Security.h>
#import <sys/wait.h>
#import <unistd.h>
extern bool vara_credential_parent_is_trusted(void);
extern char *vara_credential(const char *, const char *, const char *);
static NSDictionary *call(const char *op, const char *account) {
    char *raw = vara_credential(op, account, "vara-fictional-test-key");
    NSData *data = [NSData dataWithBytes:raw length:strlen(raw)]; free(raw);
    return [NSJSONSerialization JSONObjectWithData:data options:0 error:NULL];
}
int main(int argc, char **argv) {
    @autoreleasepool {
        if (argc == 4 && !strcmp(argv[1], "--worker")) {
            if (!vara_credential_parent_is_trusted()) return 77;
            NSDictionary *reply = call(argv[2], argv[3]);
            // Only this disposable test value is compared; never print a secret.
            if (!strcmp(argv[2], "get")) return [reply[@"value"] isEqual:@"vara-fictional-test-key"] ? 0 : 1;
            int status = [reply[@"status"] intValue];
            return status == 0 || (!strcmp(argv[2], "status") && status == errSecItemNotFound) ? 0 : 1;
        }
        if (argc == 3 && !strcmp(argv[1], "--untrusted")) {
            NSDictionary *reply = call("get", argv[2]);
            return [reply[@"status"] intValue] == errSecInteractionNotAllowed || [reply[@"status"] intValue] == errSecAuthFailed ? 0 : 1;
        }
        NSString *account = [NSString stringWithFormat:@"credential-test-%d", getpid()];
        bool ok = true;
        for (NSString *op in @[@"status", @"set", @"status", @"get"]) {
            pid_t child = fork();
            if (child == 0) { execl(argv[0], argv[0], "--worker", op.UTF8String, account.UTF8String, NULL); _exit(90); }
            int result; waitpid(child, &result, 0);
            if (!WIFEXITED(result) || WEXITSTATUS(result)) { fprintf(stderr, "Failed: %s\n", op.UTF8String); ok = false; break; }
        }
        if (ok) {
            pid_t child = fork();
            if (child == 0) { execl("/tmp/vara-credentials-untrusted", "/tmp/vara-credentials-untrusted", "--untrusted", account.UTF8String, NULL); _exit(90); }
            int result; waitpid(child, &result, 0);
            ok = WIFEXITED(result) && WEXITSTATUS(result) == 0;
            if (!ok) fprintf(stderr, "Different executable unexpectedly read test key\n");
        }
        NSDictionary *cleanup = call("delete", account.UTF8String);
        ok = ok && [cleanup[@"status"] intValue] == 0;
        NSDictionary *missing = call("status", account.UTF8String);
        ok = ok && [missing[@"status"] intValue] == errSecItemNotFound;
        puts(ok ? "PASS: metadata, explicit ACL, signed parent, read, deletion and cleanup" : "FAIL: credential adapter");
        return ok ? 0 : 1;
    }
}
