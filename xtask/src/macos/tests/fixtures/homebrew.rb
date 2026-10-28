cask "quantum" do
  version "0.4.0"
  sha256 "abababababababababababababababababababababababababababababababab"

  url "https://github.com/0x63616c/quantum/releases/download/v#{version}/Quantum.dmg"
  name "Quantum"
  desc "3D voxel editor for people and agents"
  homepage "https://quantum.worldwidewebb.co/"

  livecheck do
    url :url
    strategy :github_latest
  end

  auto_updates true
  depends_on arch: :arm64
  depends_on macos: :ventura

  app "Quantum.app"
  binary "#{appdir}/Quantum.app/Contents/MacOS/quantum-mcp"

  zap trash: [
    "~/Library/Application Support/quantum",
    "~/Library/Application Support/voxelite",
    "~/Library/Caches/co.worldwidewebb.voxelite",
    "~/Library/HTTPStorages/co.worldwidewebb.voxelite",
    "~/Library/Preferences/co.worldwidewebb.voxelite.plist",
    "~/Library/Saved Application State/co.worldwidewebb.voxelite.savedState",
  ]
end
