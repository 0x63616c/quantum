import SwiftUI

/// All panels sample the same full-window image in a common coordinate space.
struct GlassContext {
    let backdrop: NSImage
    let size: CGSize
    let theme: ConceptTheme
    let presentation: GlassPresentation
}

/// Native Liquid Glass in live macOS 26+ windows; deterministic painted glass elsewhere.
struct LiquidGlass: ViewModifier {
    let context: GlassContext
    var radius: CGFloat = 26

    @ViewBuilder
    func body(content: Content) -> some View {
        if #available(macOS 26, *), context.presentation == .live {
            content.glassEffect(.regular, in: RoundedRectangle(cornerRadius: radius, style: .continuous))
        } else {
            content.background {
                GeometryReader { proxy in
                    PaintedGlass(context: context, rect: proxy.frame(in: .named("glassCanvas")), radius: radius)
                }
            }
        }
    }
}

private struct PaintedGlass: View {
    let context: GlassContext
    let rect: CGRect
    let radius: CGFloat
    private var shape: RoundedRectangle { RoundedRectangle(cornerRadius: radius, style: .continuous) }

    private func sample(magnification: CGFloat = 1, blur: CGFloat) -> some View {
        Image(nsImage: context.backdrop).resizable()
            .frame(width: context.size.width, height: context.size.height)
            .offset(x: -rect.minX, y: -rect.minY)
            .frame(width: rect.width, height: rect.height, alignment: .topLeading)
            .scaleEffect(magnification)
            .blur(radius: blur)
    }

    var body: some View {
        ZStack {
            sample(blur: 16)
            // A narrow magnified boundary approximates the optical lens at a glass edge.
            sample(magnification: 1.025, blur: 2)
                .mask(shape.strokeBorder(lineWidth: 3))
            (context.theme.isDark ? Color(hex: 0x424A55).opacity(0.31) : Color.white.opacity(0.28))
            LinearGradient(colors: [.white.opacity(context.theme.isDark ? 0.07 : 0.16), .clear,
                                     .black.opacity(context.theme.isDark ? 0.10 : 0.025)],
                           startPoint: .topLeading, endPoint: .bottomTrailing)
        }
        .clipShape(shape)
        .overlay {
            shape.strokeBorder(LinearGradient(stops: [
                .init(color: .white.opacity(context.theme.isDark ? 0.50 : 0.85), location: 0),
                .init(color: .white.opacity(0.10), location: 0.35),
                .init(color: .black.opacity(0.08), location: 0.65),
                .init(color: .white.opacity(context.theme.isDark ? 0.24 : 0.68), location: 1),
            ], startPoint: .topLeading, endPoint: .bottomTrailing), lineWidth: 0.85)
        }
        .overlay {
            shape.inset(by: 1.5).stroke(.black.opacity(context.theme.isDark ? 0.17 : 0.035), lineWidth: 1)
                .blur(radius: 1).clipShape(shape)
        }
        .shadow(color: .black.opacity(context.theme.isDark ? 0.22 : 0.07), radius: 18, x: 0, y: 10)
        .shadow(color: .black.opacity(0.035), radius: 2, x: 0, y: 1)
        .allowsHitTesting(false)
    }
}

extension View {
    func liquidGlass(_ context: GlassContext, radius: CGFloat = 26) -> some View {
        modifier(LiquidGlass(context: context, radius: radius))
    }
}

/// Group only adjacent controls; large panels remain independent optical surfaces.
struct GlassGroup<Content: View>: View {
    let presentation: GlassPresentation
    @ViewBuilder var content: () -> Content
    @ViewBuilder var body: some View {
        if #available(macOS 26, *), presentation == .live {
            GlassEffectContainer(spacing: 12, content: content)
        } else {
            content()
        }
    }
}

struct GlassButton<Label: View>: View {
    let context: GlassContext
    var radius: CGFloat = 16
    var action: () -> Void = {}
    @ViewBuilder var label: () -> Label
    @ViewBuilder var body: some View {
        if #available(macOS 26, *), context.presentation == .live {
            Button(action: action, label: label).buttonStyle(.glass)
                .buttonBorderShape(.roundedRectangle(radius: radius))
        } else {
            Button(action: action) { label().liquidGlass(context, radius: radius) }
                .buttonStyle(.plain)
        }
    }
}
