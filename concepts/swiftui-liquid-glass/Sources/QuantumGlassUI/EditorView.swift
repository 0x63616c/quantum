import SwiftUI

/// A fixed artboard for reviewing the design, not an editor implementation.
public struct QuantumEditorView: View {
    let assets: ConceptAssets
    let theme: ConceptTheme
    let presentation: GlassPresentation
    @State private var tool = "pencil.tip"

    public init(assets: ConceptAssets, theme: ConceptTheme, presentation: GlassPresentation = .live) {
        self.assets = assets; self.theme = theme; self.presentation = presentation
    }

    public var body: some View {
        let glass = assets.context(theme: theme, welcome: false, presentation: presentation)
        ZStack(alignment: .topLeading) {
            Image(nsImage: glass.backdrop).resizable().frame(width: 1440, height: 900)
            titlebar(glass).frame(width: 1388, height: 60).position(x: 720, y: 44)
            Sidebar(assets: assets, context: glass).frame(width: 244, height: 666)
                .liquidGlass(glass).position(x: 146, y: 439)
            ToolPalette(context: glass, selected: $tool).position(x: 319, y: 454)
            Inspector(assets: assets, context: glass).frame(width: 302, height: 666)
                .liquidGlass(glass).position(x: 1265, y: 439)
            HStack(spacing: 6) {
                Symbol(name: "cube.transparent", size: 13)
                Text("Isometric").font(.system(size: 12, weight: .medium))
            }.foregroundStyle(theme.secondary).position(x: 426, y: 128)
            AxisGizmo(theme: theme).frame(width: 80, height: 80).position(x: 1056, y: 150)
            cameraBar(glass).position(x: 735, y: 808)
            status(glass).position(x: 735, y: 867)
            HStack(spacing: 7) {
                Symbol(name: "computermouse", size: 12)
                Text("Drag to orbit").font(.system(size: 11))
                Text("·").padding(.horizontal, 3)
                Text("Scroll to zoom").font(.system(size: 11))
            }.foregroundStyle(theme.secondary.opacity(0.8)).position(x: 147, y: 850)
            HStack(spacing: 10) {
                Symbol(name: "keyboard", size: 15)
                Text("100%").font(.system(size: 12, design: .monospaced))
            }.foregroundStyle(theme.secondary).position(x: 1358, y: 850)
        }
        .frame(width: 1440, height: 900)
        .coordinateSpace(name: "glassCanvas")
        .foregroundStyle(theme.ink)
        .environment(\.colorScheme, theme.scheme)
        .clipShape(RoundedRectangle(cornerRadius: 28, style: .continuous))
        .overlay(RoundedRectangle(cornerRadius: 28).strokeBorder(.white.opacity(theme.isDark ? 0.16 : 0.8), lineWidth: 1))
    }

    private func titlebar(_ glass: GlassContext) -> some View {
        HStack(spacing: 18) {
            if presentation == .snapshot { TrafficLights() }
            else { Color.clear.frame(width: 52, height: 12) }
            Rectangle().fill(theme.line).frame(width: 1, height: 22).padding(.horizontal, 3)
            Image(nsImage: assets.icon).resizable().frame(width: 33, height: 33)
            HStack(spacing: 6) {
                Text("Autumn Workshop").font(.system(size: 15, weight: .semibold))
                Text(".voxl").font(.system(size: 14)).foregroundStyle(theme.secondary)
                Symbol(name: "chevron.down", size: 9).foregroundStyle(theme.secondary).padding(.leading, 4)
            }
            Spacer()
            GlassGroup(presentation: presentation) {
                HStack(spacing: 10) {
                    GlassButton(context: glass, radius: 21) {
                        HStack(spacing: 10) {
                            Symbol(name: "magnifyingglass", size: 15)
                            Text("Search commands").font(.system(size: 12))
                            Text("⌘K").font(.system(size: 11, weight: .medium)).foregroundStyle(theme.secondary)
                                .padding(.leading, 14)
                        }.padding(.horizontal, 16).frame(height: 40)
                    }
                    GlassButton(context: glass, radius: 21) {
                        HStack(spacing: 8) { Symbol(name: "square.and.arrow.up", size: 15); Text("Share") }
                            .font(.system(size: 12, weight: .medium)).padding(.horizontal, 16).frame(height: 40)
                    }
                    GlassButton(context: glass, radius: 21) {
                        Text("Save").font(.system(size: 12, weight: .semibold))
                            .frame(width: 73, height: 40)
                    }
                }
            }
        }
    }

    private func cameraBar(_ glass: GlassContext) -> some View {
        HStack(spacing: 3) {
            ForEach([("cube", "Iso"), ("rectangle", "Front"), ("rectangle.righthalf.inset.filled", "Right"),
                     ("rectangle.tophalf.inset.filled", "Top")], id: \.1) { symbol, label in
                HStack(spacing: 7) { Symbol(name: symbol, size: 14); Text(label).font(.system(size: 12, weight: .medium)) }
                    .foregroundStyle(label == "Iso" ? theme.accentText : theme.secondary)
                    .frame(width: 77, height: 38)
                    .background(label == "Iso" ? theme.selection : .clear, in: Capsule())
            }
            Rectangle().fill(theme.line).frame(width: 1, height: 20).padding(.horizontal, 6)
            Symbol(name: "arrow.up.left.and.arrow.down.right", size: 14).frame(width: 36, height: 38)
        }.padding(7).liquidGlass(glass, radius: 28).fixedSize()
    }

    private func status(_ glass: GlassContext) -> some View {
        HStack(spacing: 12) {
            Text("27,817 voxels")
            Circle().fill(theme.secondary.opacity(0.5)).frame(width: 2, height: 2)
            Text("80 × 64 × 80").monospacedDigit()
            Circle().fill(theme.secondary.opacity(0.5)).frame(width: 2, height: 2)
            Circle().fill(Color(hex: theme.isDark ? 0x86EFAC : 0x258B55)).frame(width: 5, height: 5)
            Text("Ready").padding(.leading, -7)
        }.font(.system(size: 11)).foregroundStyle(theme.secondary)
            .padding(.horizontal, 17).frame(height: 30).liquidGlass(glass, radius: 16).fixedSize()
    }
}

private struct ToolPalette: View {
    let context: GlassContext
    @Binding var selected: String
    private let tools = [("pencil.tip", "Brush · B"), ("eraser", "Erase · E"),
                         ("paintbrush.pointed", "Paint · P"), ("drop.halffull", "Fill · F"),
                         ("rectangle.dashed", "Select · M"), ("eyedropper", "Pick colour · I")]
    var body: some View {
        VStack(spacing: 5) {
            ForEach(tools, id: \.0) { name, help in
                Button { selected = name } label: {
                    Symbol(name: name, size: 20).frame(width: 44, height: 43)
                        .foregroundStyle(selected == name ? context.theme.accentText : context.theme.ink.opacity(0.85))
                        .background(selected == name ? context.theme.selection : .clear,
                                    in: RoundedRectangle(cornerRadius: 16))
                        .overlay(RoundedRectangle(cornerRadius: 16)
                            .strokeBorder(selected == name ? context.theme.accentText.opacity(0.25) : .clear, lineWidth: 0.7))
                }.buttonStyle(.plain).help(help)
            }
            Rectangle().fill(context.theme.line).frame(width: 28, height: 1).padding(.vertical, 6)
            ForEach(["X", "Y", "Z"], id: \.self) { axis in
                ZStack(alignment: .bottomTrailing) {
                    Symbol(name: axis == "Y" ? "arrow.up.and.down.righttriangle.up.righttriangle.down" : "arrow.left.and.right.righttriangle.left.righttriangle.right", size: 18)
                        .frame(width: 40, height: 33)
                    Text(axis).font(.system(size: 8, weight: .semibold)).offset(x: 1, y: 0)
                }.foregroundStyle(context.theme.secondary).help("Mirror \(axis)")
            }
        }.padding(9).liquidGlass(context, radius: 31).fixedSize()
    }
}

private struct AxisGizmo: View {
    let theme: ConceptTheme
    var body: some View {
        ZStack {
            Path { p in
                p.move(to: CGPoint(x: 40,y: 43)); p.addLine(to: CGPoint(x: 40,y: 12))
                p.move(to: CGPoint(x: 40,y: 43)); p.addLine(to: CGPoint(x: 69,y: 61))
                p.move(to: CGPoint(x: 40,y: 43)); p.addLine(to: CGPoint(x: 11,y: 61))
            }.stroke(theme.secondary.opacity(0.6), lineWidth: 1)
            axis("Y", color: 0x86EFAC).position(x: 40, y: 12)
            axis("X", color: 0xF87171).position(x: 69, y: 61)
            axis("Z", color: 0x93C5FD).position(x: 11, y: 61)
            Circle().fill(theme.secondary).frame(width: 4, height: 4).position(x: 40, y: 43)
        }
    }
    private func axis(_ label: String, color: UInt32) -> some View {
        Text(label).font(.system(size: 10, weight: .semibold)).foregroundStyle(Color(hex: 0x25303A))
            .frame(width: 22, height: 22).background(Color(hex: color), in: Circle())
            .overlay(Circle().strokeBorder(.white.opacity(0.35), lineWidth: 0.6))
    }
}
