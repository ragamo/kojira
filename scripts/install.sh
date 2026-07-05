#!/bin/sh
# kojira installer
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/ragamo/kojira/master/scripts/install.sh | sh
#
# Environment overrides:
#   VERSION=v0.1.0     install a specific tag (default: latest release)
#   INSTALL_DIR=~/bin  install location (default: ~/.local/bin)
#
# Downloads the prebuilt binary for your platform from GitHub Releases,
# verifies its SHA-256 checksum, and installs it. No sudo required.

set -eu

REPO="ragamo/kojira"
BIN="kojira"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${VERSION:-}"

# ---------------------------------------------------------------------------

info() { printf '\033[1;34m::\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33mwarning:\033[0m %s\n' "$*" >&2; }
err()  { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

case "${1:-}" in
  -h|--help)
    sed -n '2,12p' "$0" | sed 's/^# \{0,1\}//'
    exit 0
    ;;
esac

need() { command -v "$1" >/dev/null 2>&1; }

# HTTP GET to stdout via curl or wget
http_get() {
  if need curl; then
    curl -fsSL "$1"
  elif need wget; then
    wget -qO- "$1"
  else
    err "need 'curl' or 'wget' installed"
  fi
}

# Download a URL to a file
http_download() {
  # $1 url, $2 dest
  if need curl; then
    curl -fsSL -o "$2" "$1"
  elif need wget; then
    wget -qO "$2" "$1"
  else
    err "need 'curl' or 'wget' installed"
  fi
}

sha256_of() {
  if need sha256sum; then
    sha256sum "$1" | awk '{print $1}'
  elif need shasum; then
    shasum -a 256 "$1" | awk '{print $1}'
  else
    return 1
  fi
}

# ---------------------------------------------------------------------------
# Detect platform → Rust target triple (must match the release asset names)

detect_target() {
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux)  os_part="unknown-linux-gnu" ;;
    Darwin) os_part="apple-darwin" ;;
    MINGW*|MSYS*|CYGWIN*)
      err "Windows is not supported by this script — download the .zip from https://github.com/$REPO/releases" ;;
    *) err "unsupported OS: $os" ;;
  esac

  case "$arch" in
    x86_64|amd64)  arch_part="x86_64" ;;
    arm64|aarch64) arch_part="aarch64" ;;
    *) err "unsupported architecture: $arch" ;;
  esac

  echo "${arch_part}-${os_part}"
}

# ---------------------------------------------------------------------------

resolve_version() {
  if [ -n "$VERSION" ]; then
    echo "$VERSION"
    return
  fi
  # Parse tag_name from the latest-release API response
  http_get "https://api.github.com/repos/$REPO/releases/latest" \
    | grep '"tag_name"' \
    | head -n1 \
    | sed -E 's/.*"tag_name"[[:space:]]*:[[:space:]]*"([^"]+)".*/\1/'
}

main() {
  need tar || err "need 'tar' installed"

  target="$(detect_target)"
  version="$(resolve_version)"
  [ -n "$version" ] || err "could not determine the latest version"

  archive="${BIN}-${target}.tar.gz"
  base_url="https://github.com/$REPO/releases/download/$version"

  info "Installing $BIN $version ($target)"

  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT INT TERM

  info "Downloading $archive"
  http_download "$base_url/$archive" "$tmp/$archive"

  # Verify checksum when possible
  if http_download "$base_url/$archive.sha256" "$tmp/$archive.sha256" 2>/dev/null; then
    expected="$(awk '{print $1}' "$tmp/$archive.sha256")"
    if actual="$(sha256_of "$tmp/$archive")"; then
      [ "$expected" = "$actual" ] || err "checksum mismatch (expected $expected, got $actual)"
      info "Checksum verified"
    else
      warn "no sha256 tool found — skipping checksum verification"
    fi
  else
    warn "checksum file not found — skipping verification"
  fi

  info "Extracting"
  tar -xzf "$tmp/$archive" -C "$tmp"

  [ -f "$tmp/$BIN" ] || err "binary '$BIN' not found in archive"

  mkdir -p "$INSTALL_DIR"
  cp "$tmp/$BIN" "$INSTALL_DIR/$BIN"
  chmod 755 "$INSTALL_DIR/$BIN"

  info "Installed to $INSTALL_DIR/$BIN"

  # PATH hint
  case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *)
      warn "$INSTALL_DIR is not in your PATH"
      printf '  Add this to your shell profile:\n    export PATH="%s:$PATH"\n' "$INSTALL_DIR"
      ;;
  esac

  info "Done. Run '$BIN' to get started."
}

main "$@"
