# Synced copy of https://github.com/arazgray/homebrew-tap/blob/main/Formula/az.rb
# (the tap is canonical). Install with: brew install arazgray/tap/az
class Az < Formula
  desc "Fast, small & sane terminal text editor"
  homepage "https://github.com/arazgray/az"
  # Release tags are short ("4.0"); the Cargo version is fuller ("4.0.0").
  url "https://github.com/arazgray/az/archive/refs/tags/4.0.tar.gz"
  version "4.0.0"
  sha256 "b6bbf759b180708a4adc40f8812cd631bb10a2f0e5e982e3145dd05f59cbd998"
  license "WTFPL"

  depends_on "rust" => :build

  livecheck do
    url :stable
    strategy :github_tag
    regex(/^(\d+(?:\.\d+)+)$/)
  end

  def install
    # Zero dependencies, so the vendored Cargo.lock builds offline.
    system "cargo", "build", "--release", "--locked"
    bin.install "target/release/az"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/az --version")
  end
end
