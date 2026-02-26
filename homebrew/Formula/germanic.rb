class Germanic < Formula
  desc "Schema-validated binary data for AI agents"
  homepage "https://github.com/germanicdev/germanic"
  version "0.2.1"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/germanicdev/germanic/releases/download/v#{version}/germanic-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_AARCH64_APPLE_DARWIN"
    end
    on_intel do
      url "https://github.com/germanicdev/germanic/releases/download/v#{version}/germanic-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_X86_64_APPLE_DARWIN"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/germanicdev/germanic/releases/download/v#{version}/germanic-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_AARCH64_UNKNOWN_LINUX_GNU"
    end
    on_intel do
      url "https://github.com/germanicdev/germanic/releases/download/v#{version}/germanic-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_X86_64_UNKNOWN_LINUX_GNU"
    end
  end

  def install
    bin.install "germanic"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/germanic --version")
  end
end
