import SwiftUI

struct Sidebar: View {
    let assets: ConceptAssets
    let context: GlassContext
    @State private var selected = "Workshop details"
    private var theme: ConceptTheme { context.theme }
    private let layers = [("Workshop details", "sparkles", "589", UInt32(0xE8B675)),
                          ("Autumn garden", "leaf", "3,632", UInt32(0xC98B54)),
                          ("Cottage", "house", "6,520", UInt32(0x7195A3)),
                          ("Landscape", "mountain.2", "17,156", UInt32(0x90A36F))]

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack {
                Text("Project").font(.system(size: 13, weight: .semibold))
                Spacer()
                Symbol(name: "sidebar.left", size: 15).foregroundStyle(theme.secondary)
            }.padding(.bottom, 20)
            HStack(spacing: 10) {
                Symbol(name: "cube", size: 20).foregroundStyle(theme.accentText)
                    .frame(width: 38, height: 42).background(theme.selection, in: RoundedRectangle(cornerRadius: 12))
                VStack(alignment: .leading, spacing: 4) {
                    Text("Autumn Workshop").font(.system(size: 13, weight: .semibold))
                    Text("Quantum project").font(.system(size: 11)).foregroundStyle(theme.secondary)
                }
            }
            Rectangle().fill(theme.line).frame(height: 1).padding(.vertical, 23)
            HStack {
                Text("Layers").font(.system(size: 12, weight: .semibold))
                Text("4").font(.system(size: 10, weight: .medium)).foregroundStyle(theme.secondary)
                    .padding(.horizontal, 6).padding(.vertical, 2)
                    .background(theme.ink.opacity(0.06), in: Capsule())
                Spacer()
                Symbol(name: "plus", size: 13).foregroundStyle(theme.secondary)
            }.padding(.bottom, 12)
            VStack(spacing: 6) {
                ForEach(layers, id: \.0) { name, icon, count, color in
                    Button { selected = name } label: {
                        HStack(spacing: 9) {
                            RoundedRectangle(cornerRadius: 2).fill(Color(hex: color)).frame(width: 3, height: 25)
                            VStack(alignment: .leading, spacing: 4) {
                                Text(name).font(.system(size: 12, weight: selected == name ? .semibold : .regular))
                                Text("\(count) voxels").font(.system(size: 10)).foregroundStyle(theme.secondary)
                            }
                            Spacer(minLength: 0)
                            Symbol(name: "eye", size: 11).foregroundStyle(theme.secondary)
                            Symbol(name: name == "Landscape" ? "lock" : "lock.open", size: 10)
                                .foregroundStyle(theme.secondary.opacity(0.65))
                        }.padding(.horizontal, 10).frame(height: 54)
                            .background(selected == name ? theme.selection : .clear,
                                        in: RoundedRectangle(cornerRadius: 13))
                    }.buttonStyle(.plain)
                }
            }.padding(.horizontal, -9)
            HStack(spacing: 6) {
                Symbol(name: "square.3.layers.3d", size: 11)
                Text("All layers visible").font(.system(size: 10))
            }.foregroundStyle(theme.secondary).padding(.top, 16)
            Spacer()
            HStack {
                Symbol(name: "viewfinder", size: 13)
                Text("Scene bounds").font(.system(size: 12, weight: .medium))
                Spacer()
            }.padding(.bottom, 12)
            HStack(spacing: 7) {
                dimension("W", "80"); dimension("H", "64"); dimension("D", "80")
            }
            Rectangle().fill(theme.line).frame(height: 1).padding(.vertical, 19)
            HStack(spacing: 8) {
                Symbol(name: "internaldrive", size: 13)
                Text("On this Mac").font(.system(size: 11))
                Spacer()
                Symbol(name: "ellipsis", size: 14)
            }.foregroundStyle(theme.secondary)
        }.padding(22)
    }

    private func dimension(_ axis: String, _ value: String) -> some View {
        HStack(spacing: 6) {
            Text(axis).font(.system(size: 9)).foregroundStyle(theme.secondary)
            Text(value).font(.system(size: 12, weight: .medium, design: .monospaced))
        }.frame(maxWidth: .infinity).frame(height: 31)
            .background(theme.ink.opacity(0.04), in: RoundedRectangle(cornerRadius: 8))
    }
}
