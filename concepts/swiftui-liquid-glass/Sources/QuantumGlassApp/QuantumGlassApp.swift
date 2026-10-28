import AppKit
import QuantumGlassUI
import SwiftUI

@main
struct QuantumGlassApp: App {
    private let assets: Result<ConceptAssets, Error>

    init() {
        assets = Result { try ConceptAssets() }
        NSApplication.shared.setActivationPolicy(.regular)
        NSApplication.shared.activate(ignoringOtherApps: true)
    }

    var body: some Scene {
        Window("Quantum", id: "welcome") {
            switch assets {
            case .success(let assets): WelcomeWindow(assets: assets)
            case .failure(let error): Text(error.localizedDescription).padding(40)
            }
        }.windowStyle(.hiddenTitleBar).windowResizability(.contentSize)
        Window("Autumn Workshop.voxl", id: "editor") {
            switch assets {
            case .success(let assets): EditorWindow(assets: assets)
            case .failure(let error): Text(error.localizedDescription).padding(40)
            }
        }.windowStyle(.hiddenTitleBar).windowResizability(.contentSize)
    }
}

private struct WelcomeWindow: View {
    let assets: ConceptAssets
    @Environment(\.colorScheme) private var scheme
    @Environment(\.openWindow) private var openWindow
    var body: some View {
        QuantumWelcomeView(assets: assets, theme: scheme == .dark ? .dark : .light) {
            openWindow(id: "editor")
        }
    }
}

private struct EditorWindow: View {
    let assets: ConceptAssets
    @Environment(\.colorScheme) private var scheme
    var body: some View {
        QuantumEditorView(assets: assets, theme: scheme == .dark ? .dark : .light)
    }
}
