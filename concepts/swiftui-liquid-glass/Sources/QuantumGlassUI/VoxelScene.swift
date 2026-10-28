import AppKit
import Metal
import SceneKit
import simd

/// Render-only projections of the repository's sample documents, with voxel data intact.
struct VoxelDocument: Decodable {
    struct Document: Decodable { let name: String }
    struct Layer: Decodable {
        let name: String
        let visible: Bool
        let voxels: [[Int]]
    }
    struct Palette: Decodable {
        struct Slot: Decodable { let index: Int; let rgba: [Int] }
        let order: [Int]
        let slots: [Slot]
    }
    let document: Document
    let layers: [Layer]
    let palette: Palette
}

public enum ConceptError: Error, LocalizedError {
    case resource(String), noMetal, image(String), usage
    public var errorDescription: String? {
        switch self {
        case .resource(let name): "Missing concept resource: \(name)"
        case .noMetal: "SceneKit rendering requires a Metal device."
        case .image(let name): "Could not render or encode \(name)."
        case .usage: "Usage: QuantumGlassRender <output-directory>"
        }
    }
}

/// One merged geometry per scene. Interior faces are culled across layer boundaries.
@MainActor
enum VoxelScene {
    struct Face {
        let normal: SIMD3<Int>
        let u: SIMD3<Int>
        let v: SIMD3<Int>
    }
    static let faces = [
        Face(normal: [1,0,0], u: [0,1,0], v: [0,0,1]),
        Face(normal: [-1,0,0], u: [0,0,1], v: [0,1,0]),
        Face(normal: [0,1,0], u: [0,0,1], v: [1,0,0]),
        Face(normal: [0,-1,0], u: [1,0,0], v: [0,0,1]),
        Face(normal: [0,0,1], u: [1,0,0], v: [0,1,0]),
        Face(normal: [0,0,-1], u: [0,1,0], v: [1,0,0]),
    ]

    static func load(_ name: String) throws -> VoxelDocument {
        guard let url = Bundle.module.url(forResource: name, withExtension: "json") else {
            throw ConceptError.resource(name)
        }
        return try JSONDecoder().decode(VoxelDocument.self, from: Data(contentsOf: url))
    }

    static func render(_ document: VoxelDocument, size: CGSize) throws -> NSImage {
        guard let device = MTLCreateSystemDefaultDevice() else { throw ConceptError.noMetal }
        var occupied: [SIMD3<Int>: Int] = [:]
        for layer in document.layers where layer.visible {
            for voxel in layer.voxels where voxel.count == 4 {
                occupied[SIMD3(voxel[0], voxel[1], voxel[2])] = voxel[3]
            }
        }
        let palette = Dictionary(uniqueKeysWithValues: document.palette.slots.map { ($0.index, $0.rgba) })
        let geometry = mesh(occupied, palette: palette)
        let scene = SCNScene()
        scene.background.contents = NSColor.clear
        scene.rootNode.addChildNode(SCNNode(geometry: geometry))
        let cameraNode = camera(for: occupied, aspect: Float(size.width / size.height))
        scene.rootNode.addChildNode(cameraNode)
        let renderer = SCNRenderer(device: device, options: nil)
        renderer.scene = scene
        renderer.pointOfView = cameraNode
        renderer.autoenablesDefaultLighting = false
        let image = renderer.snapshot(atTime: 0, with: size, antialiasingMode: .multisampling4X)
        guard image.isValid else { throw ConceptError.image(document.document.name) }
        return image
    }

    private static func mesh(_ voxels: [SIMD3<Int>: Int], palette: [Int: [Int]]) -> SCNGeometry {
        var positions: [SCNVector3] = []
        var colors: [Float] = []
        var indices: [UInt32] = []
        let light = simd_normalize(SIMD3<Float>(-0.5, 1, 0.75))
        // Stable traversal also keeps the exported imagery repeatable on the same GPU.
        let keys = voxels.keys.sorted { a, b in
            if a.x != b.x { return a.x < b.x }
            if a.y != b.y { return a.y < b.y }
            return a.z < b.z
        }
        for p in keys {
            guard let slot = voxels[p], let rgba = palette[slot] else { continue }
            for face in faces where voxels[p &+ face.normal] == nil {
                let start = UInt32(positions.count)
                let normal = SIMD3<Float>(face.normal)
                let directional = 0.70 + 0.30 * max(0, simd_dot(normal, light))
                for (u, v) in [(0,0), (1,0), (1,1), (0,1)] {
                    let su = face.u &* SIMD3(repeating: u == 0 ? -1 : 1)
                    let sv = face.v &* SIMD3(repeating: v == 0 ? -1 : 1)
                    let outside = p &+ face.normal
                    let a = voxels[outside &+ su] != nil
                    let b = voxels[outside &+ sv] != nil
                    let corner = voxels[outside &+ su &+ sv] != nil
                    let count = a && b ? 3 : (a ? 1 : 0) + (b ? 1 : 0) + (corner ? 1 : 0)
                    let ao = 1 - Float(count) * 0.105
                    let vertex = SIMD3<Float>(p) + SIMD3<Float>(repeating: 0.5)
                        + normal * 0.5 + SIMD3<Float>(su) * 0.5 + SIMD3<Float>(sv) * 0.5
                    positions.append(SCNVector3(vertex.x, vertex.y, vertex.z))
                    // SceneKit vertex colours are linear; the document palette is sRGB.
                    colors += rgba.prefix(3).map {
                        let srgb = Float($0) / 255
                        let linear = srgb <= 0.04045 ? srgb / 12.92 : pow((srgb + 0.055) / 1.055, 2.4)
                        return linear * directional * ao
                    }
                    colors.append(1)
                }
                indices += [start, start+1, start+2, start, start+2, start+3]
            }
        }
        let colorData = colors.withUnsafeBytes { Data($0) }
        let colorSource = SCNGeometrySource(data: colorData, semantic: .color,
            vectorCount: positions.count, usesFloatComponents: true, componentsPerVector: 4,
            bytesPerComponent: MemoryLayout<Float>.size, dataOffset: 0, dataStride: 16)
        let element = SCNGeometryElement(indices: indices, primitiveType: .triangles)
        let geometry = SCNGeometry(sources: [SCNGeometrySource(vertices: positions), colorSource], elements: [element])
        let material = SCNMaterial()
        // Baked directional light + local vertex occlusion, not Quantum's production renderer.
        material.lightingModel = .constant
        material.diffuse.contents = NSColor.white
        geometry.materials = [material]
        return geometry
    }

    private static func camera(for voxels: [SIMD3<Int>: Int], aspect: Float) -> SCNNode {
        let forward = simd_normalize(SIMD3<Float>(-1, -0.82, -1))
        let right = simd_normalize(simd_cross(forward, SIMD3<Float>(0,1,0)))
        let up = simd_cross(right, forward)
        var low = SIMD3<Float>(repeating: .greatestFiniteMagnitude)
        var high = SIMD3<Float>(repeating: -.greatestFiniteMagnitude)
        for p in voxels.keys {
            for corner in [SIMD3<Float>(0,0,0), [1,0,0], [0,1,0], [0,0,1], [1,1,0], [1,0,1], [0,1,1], [1,1,1]] {
                let point = SIMD3<Float>(p) + corner
                let projected = SIMD3(simd_dot(point,right), simd_dot(point,up), simd_dot(point,forward))
                low = simd_min(low, projected); high = simd_max(high, projected)
            }
        }
        let center = (low + high) * 0.5
        let target = right * center.x + up * center.y + forward * center.z
        let node = SCNNode()
        let camera = SCNCamera()
        camera.usesOrthographicProjection = true
        camera.orthographicScale = Double(max(high.y-low.y, (high.x-low.x)/aspect) * 0.55)
        camera.zNear = 0.1; camera.zFar = 1000
        node.camera = camera
        node.simdPosition = target - forward * 200
        node.look(at: SCNVector3(target.x, target.y, target.z))
        return node
    }
}
