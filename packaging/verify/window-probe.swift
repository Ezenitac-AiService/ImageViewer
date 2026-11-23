// Prints the number of on-screen windows owned by a process and exits 0 only if it is >= 1.
// Usage: swift packaging/verify/window-probe.swift <pid>
// (specs/004-launch-reliability-refactor/contracts/release-verification-contract.md §2)
import CoreGraphics
import Foundation

guard CommandLine.arguments.count == 2, let pid = Int32(CommandLine.arguments[1]) else {
    FileHandle.standardError.write("usage: window-probe.swift <pid>\n".data(using: .utf8)!)
    exit(2)
}
let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly], kCGNullWindowID) as? [[String: Any]] ?? []
let count = windows.filter { ($0[kCGWindowOwnerPID as String] as? Int32) == pid }.count
print("onscreen=\(count)")
exit(count >= 1 ? 0 : 1)
