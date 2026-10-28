import AppKit
import QuantumGlassUI
import SwiftUI

@main
@MainActor
struct QuantumGlassRender {
    static func main() {
        do {
            guard CommandLine.arguments.count == 2 else { throw ConceptError.usage }
            _ = NSApplication.shared
            let directory = URL(fileURLWithPath: CommandLine.arguments[1], isDirectory: true)
            try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
            let assets = try ConceptAssets()
            for theme in ConceptTheme.allCases {
                try save(QuantumEditorView(assets: assets, theme: theme, presentation: .snapshot),
                         to: directory.appendingPathComponent("editor-\(theme.rawValue).png"))
                try save(QuantumWelcomeView(assets: assets, theme: theme, presentation: .snapshot),
                         to: directory.appendingPathComponent("welcome-\(theme.rawValue).png"))
            }
        } catch {
            FileHandle.standardError.write(Data("QuantumGlassRender: \(error.localizedDescription)\n".utf8))
            exit(1)
        }
    }

    private static func save<Content: View>(_ content: Content, to url: URL) throws {
        let renderer = ImageRenderer(content: content)
        renderer.scale = 2
        renderer.colorMode = .nonLinear
        guard let image = renderer.cgImage else { throw ConceptError.image(url.lastPathComponent) }
        // Specify 8-bit sRGB explicitly so every output uses the same portable pixel format.
        guard let colorSpace = CGColorSpace(name: CGColorSpace.sRGB),
              let buffer = CGContext(data: nil, width: image.width, height: image.height,
                  bitsPerComponent: 8, bytesPerRow: image.width * 4, space: colorSpace,
                  bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue) else {
            throw ConceptError.image(url.lastPathComponent)
        }
        buffer.draw(image, in: CGRect(x: 0, y: 0, width: image.width, height: image.height))
        guard let output = buffer.makeImage() else { throw ConceptError.image(url.lastPathComponent) }
        let bitmap = NSBitmapImageRep(cgImage: output)
        guard let data = bitmap.representation(using: .png, properties: [:]) else {
            throw ConceptError.image(url.lastPathComponent)
        }
        try data.write(to: url, options: .atomic)
        print("Rendered \(url.lastPathComponent): \(image.width) × \(image.height), \(data.count) bytes")
    }
}
