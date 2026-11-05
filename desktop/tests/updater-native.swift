// macOS Accessibility driver for the installed, real Vara app. No webview hooks.
// Build: DEVELOPER_DIR=/Library/Developer/CommandLineTools swiftc desktop/tests/updater-native.swift -o /tmp/vara-updater-native
// Read: /tmp/vara-updater-native --pid PID tree
// Actions: settings | check | download | install --allow-install
// Inspect again after each action; this helper never chains update operations.
import AppKit
import ApplicationServices
import Foundation

func fail(_ message: String) -> Never {
    fputs("ERROR: \(message)\n", stderr)
    exit(1)
}
let args = Array(CommandLine.arguments.dropFirst())
if args.contains("--help") {
    print("Usage: vara-updater-native [--pid PID] tree|status|settings|check|download|install [--allow-install]")
    print("Without --pid, selects the sole running dk.vara.desktop app. No permission prompts are requested.")
    exit(0)
}
let commands = Set(["tree", "status", "settings", "check", "download", "install"])
var command: String?
var pid: pid_t?
var allowInstall = false
var index = 0
while index < args.count {
    let arg = args[index]
    if arg == "--pid" {
        index += 1
        guard index < args.count, let value = Int32(args[index]), value > 0 else { fail("--pid requires a positive PID") }
        pid = value
    } else if arg == "--allow-install" { allowInstall = true
    } else if commands.contains(arg), command == nil { command = arg
    } else { fail("Unknown or duplicate argument: \(arg); use --help") }
    index += 1
}
guard let command else { fail("Choose a command; use --help") }
if command == "install" && !allowInstall { fail("Install/restart requires the explicit --allow-install argument") }
guard AXIsProcessTrusted() else { fail("Accessibility access is unavailable. This helper does not request or grant permission.") }
let running = NSWorkspace.shared.runningApplications.filter { $0.bundleIdentifier == "dk.vara.desktop" }
if pid == nil {
    guard running.count == 1 else { fail("Expected one running Vara; found \(running.count). Supply --pid.") }
    pid = running[0].processIdentifier
}
guard let app = NSRunningApplication(processIdentifier: pid!), app.bundleIdentifier == "dk.vara.desktop" else {
    fail("Target PID is not the running dk.vara.desktop app; refusing to address another app or OS dialog")
}
let root = AXUIElementCreateApplication(pid!)
AXUIElementSetMessagingTimeout(root, 0.5)
func attribute(_ element: AXUIElement, _ name: String) -> CFTypeRef? {
    var value: CFTypeRef?
    return AXUIElementCopyAttributeValue(element, name as CFString, &value) == .success ? value : nil
}
func string(_ element: AXUIElement, _ name: String) -> String {
    attribute(element, name) as? String ?? ""
}
struct Node {
    let element: AXUIElement
    let path: String
    let role: String
    let title: String
    let description: String
    let value: String
    let enabled: Bool
    var labels: [String] { [title, description, value].filter { !$0.isEmpty } }
}
var nodes: [Node] = []
var seen = Set<CFHashCode>()
let inspectionDeadline = Date().addingTimeInterval(25)
var truncated = false
func visit(_ element: AXUIElement, _ path: String, _ depth: Int) {
    guard depth < 60, nodes.count < 2000, Date() < inspectionDeadline else { truncated = true; return }
    let hash = CFHash(element)
    guard seen.insert(hash).inserted else { return }
    let role = string(element, kAXRoleAttribute)
    // Exclude application menus and any app-owned sheets/dialogs from action selection.
    if ["AXMenuBar", "AXMenu", "AXSheet", "AXDialog"].contains(role) { return }
    nodes.append(Node(element: element, path: path, role: role,
                      title: string(element, kAXTitleAttribute),
                      description: string(element, kAXDescriptionAttribute),
                      value: string(element, kAXValueAttribute),
                      enabled: (attribute(element, kAXEnabledAttribute) as? Bool) ?? true))
    let children = attribute(element, kAXChildrenAttribute) as? [AXUIElement] ?? []
    for (i, child) in children.enumerated() { visit(child, "\(path).\(i)", depth + 1) }
}
visit(root, "app", 0)
if truncated { fputs("WARNING: AX inspection reached its time/depth/node bound; output may be incomplete.\n", stderr) }
func emit(_ fields: [String: Any]) {
    let data = try! JSONSerialization.data(withJSONObject: fields, options: [.sortedKeys])
    print(String(data: data, encoding: .utf8)!)
}
emit(["pid": pid!, "bundle": app.bundleURL?.path ?? "", "command": command, "nodes": nodes.count])
if command == "tree" || command == "status" {
    for node in nodes where command == "tree" || !node.labels.isEmpty {
        emit(["path": node.path, "role": node.role, "title": node.title,
              "description": node.description, "value": node.value, "enabled": node.enabled])
    }
    if !nodes.contains(where: { $0.role == "AXWebArea" }) {
        fail("No AXWebArea exposed by the real WKWebView. No UI action was performed.")
    }
    exit(0)
}
guard nodes.contains(where: { $0.role == "AXWebArea" }) else {
    fail("WKWebView accessibility hierarchy unavailable; no UI action performed")
}
let labels: [String: Set<String>] = [
    "settings": ["Settings", "Indstillinger"],
    "check": ["Check for updates", "Søg efter opdateringer"],
    "download": ["Download update", "Hent opdatering"],
    "install": ["Install and restart", "Installer og genstart"]
]
let matches = nodes.filter { node in
    node.role == "AXButton" && node.labels.contains { labels[command]!.contains($0) }
}
// A global update notice and Settings can expose the same download/install button.
// Choose the first enabled exact-label match, and report it before invoking AXPress.
guard let target = matches.first(where: { $0.enabled }) else {
    fail(matches.isEmpty ? "No visible \(command) button. Inspect tree/status." : "\(command) button is disabled")
}
var actions: CFArray?
guard AXUIElementCopyActionNames(target.element, &actions) == .success,
      (actions as? [String] ?? []).contains(kAXPressAction) else { fail("Matched button does not support AXPress") }
var targetPID: pid_t = 0
AXUIElementGetPid(target.element, &targetPID)
guard targetPID == pid else { fail("Matched control belongs to another process") }
emit(["action": "AXPress", "path": target.path, "labels": target.labels, "matchingButtons": matches.count])
fflush(stdout)
let result = AXUIElementPerformAction(target.element, kAXPressAction as CFString)
guard result == .success else { fail("AXPress failed with AXError \(result.rawValue)") }
emit(["result": "pressed", "command": command, "note": "Inspect status separately; completion is not assumed."])
