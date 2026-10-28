import SwiftUI

struct Inspector: View {
    let assets: ConceptAssets
    let context: GlassContext
    @State private var tab = "Session"
    @State private var selectedColor = 0
    private var theme: ConceptTheme { context.theme }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack(spacing: 2) {
                ForEach(["Session", "Comments", "History"], id: \.self) { name in
                    Button { tab = name } label: {
                        Text(name).font(.system(size: 11, weight: tab == name ? .semibold : .regular))
                            .frame(maxWidth: .infinity).frame(height: 30)
                            .background(tab == name ? theme.ink.opacity(theme.isDark ? 0.11 : 0.07) : .clear, in: Capsule())
                    }.buttonStyle(.plain)
                }
            }.padding(3).background(theme.ink.opacity(0.04), in: Capsule())
            sessionContent.padding(.top, 23)
            Rectangle().fill(theme.line).frame(height: 1).padding(.vertical, 20)
            HStack {
                Text("Project colours").font(.system(size: 13, weight: .semibold))
                Spacer()
                Text("36").font(.system(size: 11)).foregroundStyle(theme.secondary)
                Symbol(name: "ellipsis", size: 13).foregroundStyle(theme.secondary).padding(.leading, 4)
            }.padding(.bottom, 14)
            LazyVGrid(columns: Array(repeating: GridItem(.flexible(), spacing: 8), count: 6), spacing: 8) {
                ForEach(Array(assets.palette.enumerated()), id: \.offset) { index, color in
                    Button { selectedColor = index } label: {
                        RoundedRectangle(cornerRadius: 8, style: .continuous).fill(Color(hex: color))
                            .frame(height: 31)
                            .overlay(RoundedRectangle(cornerRadius: 8).strokeBorder(.white.opacity(0.13), lineWidth: 0.7))
                            .overlay {
                                if index == selectedColor {
                                    RoundedRectangle(cornerRadius: 10).strokeBorder(theme.ink, lineWidth: 1.6).padding(-3)
                                    Symbol(name: "checkmark", size: 12).foregroundStyle(swatchInk(color))
                                }
                            }
                    }.buttonStyle(.plain).help(String(format: "#%06X", color))
                }
            }
            HStack(spacing: 9) {
                RoundedRectangle(cornerRadius: 5).fill(Color(hex: assets.palette[selectedColor]))
                    .frame(width: 20, height: 20).overlay(RoundedRectangle(cornerRadius: 5).strokeBorder(theme.line))
                Text(String(format: "#%06X", assets.palette[selectedColor]))
                    .font(.system(size: 11, weight: .medium, design: .monospaced))
                Spacer()
                Text("Brush colour").font(.system(size: 10)).foregroundStyle(theme.secondary)
            }.padding(.top, 18)
            Spacer(minLength: 0)
        }.padding(21)
    }

    private var sessionContent: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack(spacing: 10) {
                Symbol(name: tab == "Comments" ? "bubble.left.and.bubble.right" : "sparkles", size: 18)
                    .foregroundStyle(theme.accentText)
                    .frame(width: 36, height: 36).background(theme.selection, in: RoundedRectangle(cornerRadius: 12))
                VStack(alignment: .leading, spacing: 4) {
                    Text(tab == "Comments" ? "Scene notes" : "Local session").font(.system(size: 13, weight: .semibold))
                    Text(tab == "Comments" ? "Notes from the composition review" : "Editing on this Mac")
                        .font(.system(size: 10)).foregroundStyle(theme.secondary)
                }
            }
            GlassButton(context: context, radius: 14) {
                HStack(spacing: 8) {
                    Symbol(name: "powerplug", size: 13)
                    Text("Connect agent").font(.system(size: 12, weight: .medium))
                    Spacer()
                    Symbol(name: "arrow.up.right", size: 10).foregroundStyle(theme.secondary)
                }.padding(.horizontal, 13).frame(height: 36)
            }.padding(.top, 13)
            HStack {
                Text(tab == "Comments" ? "Pinned to the scene" : "Recent changes").font(.system(size: 12, weight: .semibold))
                Spacer()
                Symbol(name: "arrow.uturn.backward", size: 12)
                Symbol(name: "arrow.uturn.forward", size: 12).opacity(0.4).padding(.leading, 7)
            }.padding(.top, 24).padding(.bottom, 12)
            if tab == "Comments" {
                change("Keep the warm window light", detail: "Composition review · 1 pin", symbol: "bubble.left")
                change("A path through the garden", detail: "Scene note · 1 pin", symbol: "bubble.left")
            } else {
                change("Create named version", detail: "Agent · Just now")
                change("Warm the window & lantern glass", detail: "Agent · 2 minutes ago")
            }
        }.frame(height: 231, alignment: .top)
    }

    private func change(_ title: String, detail: String, symbol: String = "checkmark") -> some View {
        HStack(alignment: .top, spacing: 9) {
            Symbol(name: symbol, size: 10).foregroundStyle(theme.secondary).frame(width: 13).padding(.top, 2)
            VStack(alignment: .leading, spacing: 5) {
                Text(title).font(.system(size: 11, weight: .medium)).lineLimit(1)
                Text(detail).font(.system(size: 10)).foregroundStyle(theme.secondary)
            }
            Spacer(minLength: 0)
        }.frame(height: 44, alignment: .top)
    }

    private func swatchInk(_ color: UInt32) -> Color {
        let brightness = Double((color >> 16) & 255) * 0.299
            + Double((color >> 8) & 255) * 0.587 + Double(color & 255) * 0.114
        return brightness > 155 ? .black : .white
    }
}
