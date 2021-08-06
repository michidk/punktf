class PunktfBin < Formula
  version '1.0.0-alpha'
  desc "A cross-platform multi-target dotfiles manager"
  homepage "https://github.com/Shemnei/punktf"

  if OS.linux?
    url "https://github.com/Shemnei/punktf/releases/download/#{version}/punktf-#{version}-x86_64-unknown-linux-musl.tar.gz"
    sha256 ""
  end

  conflicts_with "punktf"

  def install
    bin.install "punktf"
  end
end
