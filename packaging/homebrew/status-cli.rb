# frozen_string_literal: true

# Status page CLI - Homebrew Formula
# Install: brew install amkisko/tap/status-cli
class StatusCli < Formula
  desc "Status page CLI — search catalog and check live status"
  homepage "https://github.com/amkisko/status-cli.rs"
  url "https://github.com/amkisko/status-cli.rs/archive/refs/tags/v0.1.0.tar.gz"
  # Fill before release: shasum -a 256 <(curl -sL https://github.com/amkisko/status-cli.rs/archive/refs/tags/vX.Y.Z.tar.gz)
  sha256 ""
  license "MIT"
  head "https://github.com/amkisko/status-cli.rs.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: "status")
    bash_completion.install shell_output("#{bin}/status completions bash"), "status"
    zsh_completion.install shell_output("#{bin}/status completions zsh"), "_status"
    fish_completion.install shell_output("#{bin}/status completions fish"), "status.fish"
    man1.install shell_output("#{bin}/status man"), "status.1"
  end

  test do
    assert_match "status #{version}", shell_output("#{bin}/status version")
  end
end
