// Native capture helper. Requires the logged-in runner's Accessibility permission.
import AppKit
import ApplicationServices

func fail(_ message: String) -> Never {
    FileHandle.standardError.write(Data("\(message)\n".utf8))
    exit(1)
}
guard CommandLine.arguments.count == 6,
      let pid = Int32(CommandLine.arguments[1]),
      let width = Double(CommandLine.arguments[3]),
      let height = Double(CommandLine.arguments[4]),
      let scale = Double(CommandLine.arguments[5]) else { fail("invalid capture arguments") }
guard AXIsProcessTrusted() else { fail("Grant Accessibility to the runner session before capturing") }
guard CGPreflightScreenCaptureAccess() else { fail("Grant Screen Recording to the runner session before capturing") }
let pattern = CommandLine.arguments[2]
guard let screen = NSScreen.screens.first(where: {
    $0.backingScaleFactor == scale && $0.visibleFrame.width >= width && $0.visibleFrame.height >= height
}) else { fail("No display at native \(scale)x with room for \(width)x\(height); configure the golden GUI session") }
let app = AXUIElementCreateApplication(pid)
var value: CFTypeRef?
let mainTop = NSScreen.screens[0].frame.maxY
for _ in 0..<150 {
    if AXUIElementCopyAttributeValue(app, kAXWindowsAttribute as CFString, &value) == .success,
       let windows = value as? [AXUIElement] {
        for window in windows {
            var title: CFTypeRef?
            AXUIElementCopyAttributeValue(window, kAXTitleAttribute as CFString, &title)
            guard let text = title as? String, text.range(of: pattern, options: .regularExpression) != nil else { continue }
            var point = CGPoint(x: screen.visibleFrame.minX, y: mainTop - screen.visibleFrame.maxY)
            var size = CGSize(width: width, height: height)
            guard let position = AXValueCreate(.cgPoint, &point), let dimensions = AXValueCreate(.cgSize, &size),
                  AXUIElementSetAttributeValue(window, kAXPositionAttribute as CFString, position) == .success,
                  AXUIElementSetAttributeValue(window, kAXSizeAttribute as CFString, dimensions) == .success else {
                fail("Unable to place/resize capture window")
            }
            NSRunningApplication(processIdentifier: pid)?.activate(options: [.activateIgnoringOtherApps])
            AXUIElementPerformAction(window, kAXRaiseAction as CFString)
            Thread.sleep(forTimeInterval: 6)
            let info = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
            guard let found = info.first(where: {
                ($0[kCGWindowOwnerPID as String] as? Int32) == pid &&
                ($0[kCGWindowName as String] as? String) == text &&
                ($0[kCGWindowLayer as String] as? Int) == 0
            }), let id = found[kCGWindowNumber as String] as? UInt32 else { fail("No on-screen capture window") }
            print(id)
            exit(0)
        }
    }
    Thread.sleep(forTimeInterval: 0.2)
}
fail("Timed out finding /\(pattern)/ for pid \(pid)")
