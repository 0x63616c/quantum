import AppKit
import SwiftUI

/// Shared, offscreen-rendered scene images. Nothing here mutates a voxel document.
@MainActor
public final class ConceptAssets {
    let scenes: [String: NSImage]
    let icon: NSImage
    let palette: [UInt32]
    private var backdrops: [String: NSImage] = [:]

    public init() throws {
        var images: [String: NSImage] = [:]
        for name in ["autumn-workshop", "lighthouse", "cherry-blossom", "snow-cabin"] {
            let document = try VoxelScene.load(name)
            images[name] = try VoxelScene.render(document,
                size: name == "autumn-workshop" ? CGSize(width: 1800, height: 1600) : CGSize(width: 720, height: 620))
        }
        scenes = images
        let document = try VoxelScene.load("autumn-workshop")
        palette = document.palette.slots.map {
            UInt32($0.rgba[0]) << 16 | UInt32($0.rgba[1]) << 8 | UInt32($0.rgba[2])
        }
        guard let url = Bundle.module.url(forResource: "quantum-icon", withExtension: "png"),
              let image = NSImage(contentsOf: url) else { throw ConceptError.resource("quantum-icon") }
        icon = image
        for theme in ConceptTheme.allCases {
            for welcome in [false, true] {
                let size = welcome ? CGSize(width: 1000, height: 680) : CGSize(width: 1440, height: 900)
                let content = SceneBackdrop(scene: images["autumn-workshop"]!, theme: theme, welcome: welcome)
                    .frame(width: size.width, height: size.height)
                let renderer = ImageRenderer(content: content)
                renderer.scale = 2
                guard let image = renderer.nsImage else { throw ConceptError.image("backdrop") }
                backdrops["\(theme.rawValue)-\(welcome)"] = image
            }
        }
    }

    func context(theme: ConceptTheme, welcome: Bool, presentation: GlassPresentation) -> GlassContext {
        GlassContext(backdrop: backdrops["\(theme.rawValue)-\(welcome)"]!,
            size: welcome ? CGSize(width: 1000, height: 680) : CGSize(width: 1440, height: 900),
            theme: theme, presentation: presentation)
    }
}

private struct SceneBackdrop: View {
    let scene: NSImage
    let theme: ConceptTheme
    let welcome: Bool

    var body: some View {
        ZStack {
            theme.base
            RadialGradient(colors: [Color(hex: theme.isDark ? 0x343A40 : 0xFAFCFD), theme.base],
                           center: .init(x: 0.53, y: 0.4), startRadius: 40, endRadius: 850)
            if welcome {
                Image(nsImage: scene).resizable().scaledToFit()
                    .frame(width: 1050, height: 940).blur(radius: 65)
                    .opacity(theme.isDark ? 0.19 : 0.14).offset(x: 180, y: 40)
            } else {
                FloorGrid(theme: theme)
                Ellipse().fill(.black.opacity(theme.isDark ? 0.28 : 0.17))
                    .frame(width: 520, height: 160).blur(radius: 28).position(x: 734, y: 687)
                Image(nsImage: scene).resizable().scaledToFit()
                    .frame(width: 800, height: 710).position(x: 739, y: 464)
            }
        }
        .clipped()
    }
}

private struct FloorGrid: View {
    let theme: ConceptTheme
    var body: some View {
        Canvas { context, _ in
            var minor = Path()
            for i in -24...24 {
                let offset = CGFloat(i) * 32
                minor.move(to: CGPoint(x: 740 + offset - 1200, y: 604 - 600))
                minor.addLine(to: CGPoint(x: 740 + offset + 1200, y: 604 + 600))
                minor.move(to: CGPoint(x: 740 + offset - 1200, y: 604 + 600))
                minor.addLine(to: CGPoint(x: 740 + offset + 1200, y: 604 - 600))
            }
            context.stroke(minor, with: .color(theme.ink.opacity(theme.isDark ? 0.035 : 0.045)), lineWidth: 0.6)
        }
        .mask(RadialGradient(colors: [.white, .white.opacity(0)],
            center: .init(x: 0.51, y: 0.65), startRadius: 100, endRadius: 700))
    }
}
