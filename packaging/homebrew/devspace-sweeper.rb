class DevspaceSweeper < Formula
  desc "Tiny cross-platform CLI that finds and safely cleans dev junk"
  homepage "https://github.com/adwaitdeshpande/devspace-sweeper"
  version "0.1.2"

  on_macos do
    url "https://github.com/adwaitdeshpande/devspace-sweeper/releases/download/v0.1.0/devspace-sweeper-macos.tar.gz"
    sha256 "REPLACE_WITH_MACOS_TARBALL_SHA256"
  end

  on_linux do
    url "https://github.com/adwaitdeshpande/devspace-sweeper/releases/download/v0.1.0/devspace-sweeper-linux.tar.gz"
    sha256 "REPLACE_WITH_LINUX_TARBALL_SHA256"
  end

  def install
    bin.install "devspace-sweeper-macos" => "devspace-sweeper" if OS.mac?
    bin.install "devspace-sweeper-linux" => "devspace-sweeper" if OS.linux?
  end

  test do
    assert_match "devspace-sweeper", shell_output("#{bin}/devspace-sweeper --help")
  end
end
