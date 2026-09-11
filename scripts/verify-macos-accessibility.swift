import AppKit
import ApplicationServices
import Foundation

func fail(_ message: String, code: Int32) -> Never {
    fputs("\(message)\n", stderr)
    exit(code)
}

func attribute(_ element: AXUIElement, _ name: String) -> CFTypeRef? {
    var value: CFTypeRef?
    guard AXUIElementCopyAttributeValue(element, name as CFString, &value) == .success else {
        return nil
    }
    return value
}

func stringAttribute(_ element: AXUIElement, _ name: String) -> String {
    attribute(element, name) as? String ?? ""
}

func elements(from root: AXUIElement) -> [AXUIElement] {
    var queue = [root]
    var result: [AXUIElement] = []
    while !queue.isEmpty && result.count < 2_000 {
        let element = queue.removeFirst()
        result.append(element)
        if let children = attribute(element, kAXChildrenAttribute as String) as? [AXUIElement] {
            queue.append(contentsOf: children)
        }
    }
    return result
}

func label(_ element: AXUIElement) -> String {
    let title = stringAttribute(element, kAXTitleAttribute as String)
    return title.isEmpty ? stringAttribute(element, kAXDescriptionAttribute as String) : title
}

func find(
    in application: AXUIElement,
    role: String,
    label expectedLabel: String
) -> AXUIElement? {
    for _ in 0..<25 {
        if let windows = attribute(application, kAXWindowsAttribute as String) as? [AXUIElement],
           let window = windows.first(where: {
               stringAttribute($0, kAXTitleAttribute as String) == "Luma — Learn. Code. Run."
           }),
           let match = elements(from: window).first(where: {
               stringAttribute($0, kAXRoleAttribute as String) == role && label($0) == expectedLabel
           }) {
            return match
        }
        Thread.sleep(forTimeInterval: 0.2)
    }
    return nil
}

guard CommandLine.arguments.count == 2,
      let pid = pid_t(CommandLine.arguments[1]) else {
    fail("usage: verify-macos-accessibility.swift <luma-pid>", code: 64)
}

guard AXIsProcessTrusted() else {
    fail("the verification process needs macOS Accessibility permission", code: 77)
}

guard NSRunningApplication(processIdentifier: pid) != nil else {
    fail("Luma process not found", code: 1)
}
let application = AXUIElementCreateApplication(pid)
_ = attribute(application, kAXChildrenAttribute as String)
guard AXUIElementSetAttributeValue(
    application,
    kAXFrontmostAttribute as CFString,
    kCFBooleanTrue
) == .success else {
    fail("failed to make Luma frontmost through Accessibility", code: 1)
}

if find(in: application, role: kAXTextAreaRole as String, label: "Source editor") == nil {
    guard let startCoding = find(
        in: application,
        role: kAXButtonRole as String,
        label: "Start coding"
    ) else {
        fail("Start coding AXButton not found", code: 2)
    }
    guard AXUIElementPerformAction(startCoding, kAXPressAction as CFString) == .success else {
        fail("failed to press Start coding", code: 2)
    }
}

guard let editor = find(
    in: application,
    role: kAXTextAreaRole as String,
    label: "Source editor"
) else {
    fail("Source editor AXTextArea not found", code: 3)
}
_ = AXUIElementSetAttributeValue(application, kAXFrontmostAttribute as CFString, kCFBooleanTrue)
guard AXUIElementSetAttributeValue(
    editor,
    kAXFocusedAttribute as CFString,
    kCFBooleanTrue
) == .success else {
    fail("failed to focus Source editor through Accessibility", code: 4)
}
var editorFocused: Bool?
var focusResult = AXError.cannotComplete
var focusedRole = ""
var focusedLabel = ""
for _ in 0..<10 {
    _ = AXUIElementSetAttributeValue(application, kAXFrontmostAttribute as CFString, kCFBooleanTrue)
    if let currentEditor = find(
        in: application,
        role: kAXTextAreaRole as String,
        label: "Source editor"
    ) {
        editorFocused = attribute(currentEditor, kAXFocusedAttribute as String) as? Bool
    }
    var focusedValue: CFTypeRef?
    focusResult = AXUIElementCopyAttributeValue(
        application,
        kAXFocusedUIElementAttribute as CFString,
        &focusedValue
    )
    let focusedElement = focusedValue.map { unsafeBitCast($0, to: AXUIElement.self) }
    focusedRole = focusedElement.map { stringAttribute($0, kAXRoleAttribute as String) } ?? ""
    focusedLabel = focusedElement.map(label) ?? ""
    if editorFocused == true
        && focusedRole == kAXTextAreaRole as String
        && focusedLabel == "Source editor"
    {
        break
    }
    Thread.sleep(forTimeInterval: 0.2)
}

print(
    "editor_focused=\(editorFocused.map(String.init) ?? "nil") "
        + "application_result=\(focusResult.rawValue) "
        + "application_role=\(focusedRole) application_label=\(focusedLabel)"
)

if editorFocused != true
    || focusedRole != kAXTextAreaRole as String
    || focusedLabel != "Source editor"
{
    exit(5)
}
