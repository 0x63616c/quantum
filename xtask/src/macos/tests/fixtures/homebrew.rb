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
    "~/Library/Application Support/co.worldwidewebb.quantum",
    "~/Library/Application Support/co.worldwidewebb.voxelite",
    "~/Library/Application Support/quantum",
    "~/Library/Application Support/voxelite",
    "~/Library/Caches/co.worldwidewebb.quantum",
    "~/Library/Caches/co.worldwidewebb.voxelite",
    "~/Library/Cookies/co.worldwidewebb.quantum.binarycookies",
    "~/Library/Cookies/co.worldwidewebb.voxelite.binarycookies",
    "~/Library/HTTPStorages/co.worldwidewebb.quantum",
    "~/Library/HTTPStorages/co.worldwidewebb.voxelite",
    "~/Library/Preferences/co.worldwidewebb.quantum.plist",
    "~/Library/Preferences/co.worldwidewebb.voxelite.plist",
    "~/Library/Saved Application State/co.worldwidewebb.quantum.savedState",
    "~/Library/Saved Application State/co.worldwidewebb.voxelite.savedState",
    "~/Library/WebKit/co.worldwidewebb.quantum",
    "~/Library/WebKit/co.worldwidewebb.voxelite",
  ]
end
