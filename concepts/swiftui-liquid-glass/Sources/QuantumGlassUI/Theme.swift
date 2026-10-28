import SwiftUI

/// The concept uses Quantum's existing blue accent and SF's native metrics.
public enum ConceptTheme: String, CaseIterable, Sendable {
    case dark, light

    public var scheme: ColorScheme { self == .dark ? .dark : .light }
    var isDark: Bool { self == .dark }
    var ink: Color { Color(hex: isDark ? 0xF0F2F4 : 0x24282E) }
    var secondary: Color { Color(hex: isDark ? 0xADB3BC : 0x626973) }
    var accent: Color { Color(hex: 0x2563EB) }
    var accentText: Color { Color(hex: isDark ? 0x93C5FD : 0x1D4ED8) }
    var base: Color { Color(hex: isDark ? 0x191C20 : 0xE9EDF0) }
    var line: Color { ink.opacity(isDark ? 0.12 : 0.10) }
    var selection: Color { accent.opacity(isDark ? 0.23 : 0.11) }
}

extension Color {
    init(hex: UInt32) {
        self.init(.sRGB, red: Double((hex >> 16) & 255) / 255,
                  green: Double((hex >> 8) & 255) / 255, blue: Double(hex & 255) / 255)
    }
}

/// Snapshot mode deliberately avoids window-server materials, even on newer macOS.
public enum GlassPresentation: Sendable {
    case live, snapshot
}

struct TrafficLights: View {
    var body: some View {
        HStack(spacing: 8) {
            ForEach([UInt32(0xFF6058), 0xFFBD2E, 0x28C840], id: \.self) { color in
                Circle().fill(Color(hex: color))
                    .overlay(Circle().strokeBorder(.black.opacity(0.12), lineWidth: 0.5))
                    .frame(width: 12, height: 12)
            }
        }
        .accessibilityHidden(true)
    }
}

struct Symbol: View {
    let name: String
    var size: CGFloat = 16
    var body: some View {
        Image(systemName: name).font(.system(size: size, weight: .medium))
            .symbolRenderingMode(.hierarchical)
    }
}
