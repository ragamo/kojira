#!/bin/sh
# Generate the Homebrew formula for a published release.
#
# Usage:
#   scripts/gen-formula.sh v0.1.1 > Formula/kojira.rb
#
# Downloads the release tarballs from GitHub, computes their SHA-256, and
# prints a multi-platform binary formula to stdout. Used both locally and by
# the release workflow to keep the tap up to date.

set -eu

VERSION="${1:?usage: gen-formula.sh <vX.Y.Z>}"
ver="${VERSION#v}"
repo="ragamo/kojira"
base="https://github.com/$repo/releases/download/$VERSION"

sha() {
  tmp=$(mktemp)
  curl -fsSL --fail-with-body -o "$tmp" "$base/kojira-$1.tar.gz" \
    || { rm -f "$tmp"; echo "error: failed to download kojira-$1.tar.gz" >&2; exit 1; }
  { sha256sum "$tmp" 2>/dev/null || shasum -a 256 "$tmp"; } | awk '{print $1}'
  rm -f "$tmp"
}

MAC_ARM="$(sha aarch64-apple-darwin)"
MAC_X86="$(sha x86_64-apple-darwin)"
LIN_ARM="$(sha aarch64-unknown-linux-gnu)"
LIN_X86="$(sha x86_64-unknown-linux-gnu)"

cat <<EOF
class Kojira < Formula
  desc "A TUI wrapper for Jira — backlog, boards, and more"
  homepage "https://github.com/$repo"
  version "$ver"
  license "MIT"

  on_macos do
    on_arm do
      url "$base/kojira-aarch64-apple-darwin.tar.gz"
      sha256 "$MAC_ARM"
    end
    on_intel do
      url "$base/kojira-x86_64-apple-darwin.tar.gz"
      sha256 "$MAC_X86"
    end
  end

  on_linux do
    on_arm do
      url "$base/kojira-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "$LIN_ARM"
    end
    on_intel do
      url "$base/kojira-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "$LIN_X86"
    end
  end

  def install
    bin.install "kojira"
  end

  test do
    assert_match "kojira #{version}", shell_output("#{bin}/kojira --version")
  end
end
EOF
