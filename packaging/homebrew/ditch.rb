# Homebrew Formula for ditch
#
# Copy to your tap as Formula/ditch.rb after release.
# Checksums are filled by `sastri release update` (scripts/update-packaging-checksums.sh is a fallback).
class Ditch < Formula
  desc "Delete regenerable build output from development projects."
  homepage "https://github.com/baoulo/ditch"
  version "0.2.0"
  license "MIT OR Apache-2.0"

  on_macos do
    on_arm do
      url "https://github.com/baoulo/ditch/releases/download/v#{version}/ditch-v#{version}-macos-aarch64.tar.gz"
      # sha256 "REPLACE_AFTER_RELEASE"
    end
    on_intel do
      url "https://github.com/baoulo/ditch/releases/download/v#{version}/ditch-v#{version}-macos-x86_64.tar.gz"
      # sha256 "REPLACE_AFTER_RELEASE"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/baoulo/ditch/releases/download/v#{version}/ditch-v#{version}-linux-aarch64.tar.gz"
      # sha256 "REPLACE_AFTER_RELEASE"
    end
    on_intel do
      url "https://github.com/baoulo/ditch/releases/download/v#{version}/ditch-v#{version}-linux-x86_64.tar.gz"
      # sha256 "REPLACE_AFTER_RELEASE"
    end
  end

  def install
    bin.install "ditch"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/ditch --version")
  end
end
