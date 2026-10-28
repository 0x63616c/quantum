import SwiftUI

public struct QuantumWelcomeView: View {
    let assets: ConceptAssets
    let theme: ConceptTheme
    let presentation: GlassPresentation
    let openEditor: () -> Void
    @State private var showAtLaunch = true

    public init(assets: ConceptAssets, theme: ConceptTheme, presentation: GlassPresentation = .live,
                openEditor: @escaping () -> Void = {}) {
        self.assets = assets; self.theme = theme; self.presentation = presentation; self.openEditor = openEditor
    }

    public var body: some View {
        let context = assets.context(theme: theme, welcome: true, presentation: presentation)
        ZStack(alignment: .topLeading) {
            Image(nsImage: context.backdrop).resizable().frame(width: 1000, height: 680)
            if presentation == .snapshot { TrafficLights().position(x: 56, y: 28) }
            VStack(alignment: .leading, spacing: 0) {
                Image(nsImage: assets.icon).resizable().frame(width: 112, height: 112).padding(.leading, -10)
                    .shadow(color: .black.opacity(0.12), radius: 16, y: 12)
                Text("Quantum").font(.system(size: 42, weight: .bold, design: .rounded)).tracking(-1.3).padding(.top, 17)
                Text("Version 0.4.0").font(.system(size: 12)).foregroundStyle(theme.secondary).padding(.top, 8)
                Text("A little world.\nEntirely yours.").font(.system(size: 17, weight: .regular))
                    .lineSpacing(5).foregroundStyle(theme.secondary).padding(.top, 24)
                VStack(spacing: 10) {
                    action("New Project", symbol: "plus", shortcut: "⌘N", context: context, primary: true)
                    action("Open…", symbol: "folder", shortcut: "⌘O", context: context)
                    action("Open Sample…", symbol: "square.3.layers.3d", shortcut: "⇧⌘O", context: context)
                }.padding(.top, 36)
                Spacer()
                Toggle(isOn: $showAtLaunch) {
                    Text("Show at launch").font(.system(size: 11)).foregroundStyle(theme.secondary)
                }.toggleStyle(ConceptCheckboxStyle(theme: theme)).padding(.bottom, 4)
            }.frame(width: 258, height: 542, alignment: .leading).position(x: 176, y: 365)
            Rectangle().fill(theme.line).frame(width: 1, height: 520).position(x: 342, y: 366)
            VStack(alignment: .leading, spacing: 22) {
                HStack {
                    Text("Start with a sample").font(.system(size: 22, weight: .semibold)).tracking(-0.5)
                    Spacer()
                    Symbol(name: "square.grid.2x2", size: 15).foregroundStyle(theme.secondary)
                }
                LazyVGrid(columns: [GridItem(.fixed(276), spacing: 16), GridItem(.fixed(276))], spacing: 16) {
                    sample("Autumn Workshop", key: "autumn-workshop", details: "27,817 voxels · 4 layers", context: context)
                    sample("Lighthouse", key: "lighthouse", details: "3,994 voxels · 1 layer", context: context)
                    sample("Cherry Blossom", key: "cherry-blossom", details: "2,620 voxels · 1 layer", context: context)
                    sample("Snow Cabin", key: "snow-cabin", details: "3,046 voxels · 1 layer", context: context)
                }
            }.frame(width: 568).position(x: 662, y: 354)
        }.frame(width: 1000, height: 680)
            .coordinateSpace(name: "glassCanvas")
            .foregroundStyle(theme.ink)
            .environment(\.colorScheme, theme.scheme)
            .clipShape(RoundedRectangle(cornerRadius: 32, style: .continuous))
            .overlay(RoundedRectangle(cornerRadius: 32).strokeBorder(.white.opacity(theme.isDark ? 0.2 : 0.85), lineWidth: 1))
    }

    private func action(_ title: String, symbol: String, shortcut: String, context: GlassContext, primary: Bool = false) -> some View {
        GlassButton(context: context, radius: 17, action: openEditor) {
            HStack(spacing: 11) {
                Symbol(name: symbol, size: 16).frame(width: 21)
                Text(title).font(.system(size: 13, weight: primary ? .semibold : .medium))
                Spacer()
                Text(shortcut).font(.system(size: 11, weight: .medium)).opacity(0.65)
            }.padding(.horizontal, 15).frame(height: 45)
                .foregroundStyle(primary ? theme.accentText : theme.ink)
                .background(primary ? theme.selection : .clear, in: RoundedRectangle(cornerRadius: 17))
        }
    }

    private func sample(_ title: String, key: String, details: String, context: GlassContext) -> some View {
        Button(action: openEditor) {
            VStack(alignment: .leading, spacing: 0) {
                ZStack {
                    Ellipse().fill(.black.opacity(theme.isDark ? 0.14 : 0.12)).frame(width: 152, height: 37)
                        .blur(radius: 14).offset(y: 55)
                    Image(nsImage: assets.scenes[key]!).resizable().scaledToFit()
                        .frame(width: 236, height: 169).padding(.top, 4)
                }.frame(width: 276, height: 174)
                HStack(alignment: .center) {
                    VStack(alignment: .leading, spacing: 5) {
                        Text(title).font(.system(size: 13, weight: .semibold))
                        Text(details).font(.system(size: 10)).foregroundStyle(theme.secondary)
                    }
                    Spacer()
                    Symbol(name: "arrow.up.right", size: 12).foregroundStyle(theme.secondary.opacity(0.75))
                }.padding(.horizontal, 18).padding(.bottom, 18).padding(.top, 5)
            }.frame(width: 276, height: 232).liquidGlass(context, radius: 24)
        }.buttonStyle(.plain)
    }
}

/// Native checkbox drawing is AppKit-backed and unavailable to ImageRenderer.
private struct ConceptCheckboxStyle: ToggleStyle {
    let theme: ConceptTheme
    func makeBody(configuration: Configuration) -> some View {
        Button { configuration.isOn.toggle() } label: {
            HStack(spacing: 8) {
                RoundedRectangle(cornerRadius: 4).fill(configuration.isOn ? theme.accent : theme.ink.opacity(0.1))
                    .frame(width: 14, height: 14)
                    .overlay {
                        if configuration.isOn {
                            Image(systemName: "checkmark").font(.system(size: 9, weight: .bold)).foregroundStyle(.white)
                        }
                    }
                configuration.label
            }
        }.buttonStyle(.plain).accessibilityValue(configuration.isOn ? "On" : "Off")
    }
}
