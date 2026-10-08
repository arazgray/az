# Synced copy of https://github.com/arazgray/homebrew-tap/blob/main/Formula/az.rb
# (the tap is canonical). Install with: brew install arazgray/tap/az
class Az < Formula
  desc "Fast, small & sane terminal text editor"
  homepage "https://github.com/arazgray/az"
  # Patch tags carry the full version ("4.2.1"); same for Cargo.
  url "https://github.com/arazgray/az/archive/refs/tags/4.2.1.tar.gz"
  version "4.2.1"
  # TODO(4.1): refresh with: curl -sL <url above> | sha256sum (tag does not exist yet)
  sha256 "0000000000000000000000000000000000000000000000000000000000000000"
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
