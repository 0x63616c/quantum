// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "QuantumGlassConcept",
    platforms: [.macOS(.v15)],
    products: [
        .library(name: "QuantumGlassUI", targets: ["QuantumGlassUI"]),
        .executable(name: "QuantumGlassApp", targets: ["QuantumGlassApp"]),
        .executable(name: "QuantumGlassRender", targets: ["QuantumGlassRender"]),
    ],
    targets: [
        .target(name: "QuantumGlassUI", resources: [.process("Resources")]),
        .executableTarget(name: "QuantumGlassApp", dependencies: ["QuantumGlassUI"]),
        .executableTarget(name: "QuantumGlassRender", dependencies: ["QuantumGlassUI"]),
    ]
)
