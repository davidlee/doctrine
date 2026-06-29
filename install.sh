#!/usr/bin/env sh
# install.sh — no-compile installer for prebuilt doctrine binaries (macOS + Linux).
#
#   curl -fsSL https://raw.githubusercontent.com/davidlee/doctrine/main/install.sh | sh
#
# Downloads the release asset for this machine's OS+arch, verifies its sha256,
# strips the macOS Gatekeeper quarantine (no-op elsewhere), and installs
# `doctrine` to a bin dir. No Rust toolchain required — sidesteps the `cargo
# install` `-liconv` link failure (SL-174). Linux binaries are static musl, so
# they run on any distro regardless of glibc version. Inspect this script before
# piping it to a shell (R6).
#
# Env:
#   DOCTRINE_VERSION   tag to install (default: latest non-prerelease release)
#   DOCTRINE_BIN_DIR   install dir (default: $HOME/.local/bin)
set -eu

# Single-source identifiers (STD-001 — no magic strings).
REPO="davidlee/doctrine"
BIN="doctrine"

# Pure: map a (`uname -s`, `uname -m`) pair to the release-asset target triple.
# Sourceable and unit-tested (scripts/install-test.sh) without running the
# installer. Linux ships static musl; macOS ships native darwin.
triple_for() {
  os="$1"
  arch="$2"
  case "$os" in
    Darwin)
      case "$arch" in
        arm64) echo "aarch64-apple-darwin" ;;
        x86_64) echo "x86_64-apple-darwin" ;;
        *)
          echo "doctrine: unsupported architecture: $arch" >&2
          return 1
          ;;
      esac
      ;;
    Linux)
      case "$arch" in
        x86_64) echo "x86_64-unknown-linux-musl" ;;
        aarch64 | arm64) echo "aarch64-unknown-linux-musl" ;;
        *)
          echo "doctrine: unsupported architecture: $arch" >&2
          return 1
          ;;
      esac
      ;;
    *)
      echo "doctrine: unsupported OS: $os" >&2
      return 1
      ;;
  esac
}

# Verify a sha256 checksum file against its named asset, from whichever tool the
# host provides — Linux ships `sha256sum`, macOS ships `shasum`. Both emit and
# read the same `<hash>  <name>` format, so a checksum produced on either host
# verifies on the other.
verify_sha256() {
  # verify_sha256 <checksum-file>  (run from the dir holding the named asset)
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 -c "$1"
  elif command -v sha256sum >/dev/null 2>&1; then
    sha256sum -c "$1"
  else
    echo "doctrine: no sha256 tool found (need shasum or sha256sum)" >&2
    return 1
  fi
}

# Resolve the latest non-prerelease tag via the GitHub API (no jq dependency).
latest_release() {
  api="https://api.github.com/repos/${REPO}/releases/latest"
  tag="$(curl -fsSL "$api" | sed -n 's/.*"tag_name"[ ]*:[ ]*"\([^"]*\)".*/\1/p' | head -n 1)"
  if [ -z "$tag" ]; then
    echo "doctrine: could not resolve the latest release from $api" >&2
    return 1
  fi
  echo "$tag"
}

main() {
  triple="$(triple_for "$(uname -s)" "$(uname -m)")" || {
    echo "doctrine: build from source instead: cargo install ${BIN}" >&2
    exit 1
  }
  version="${DOCTRINE_VERSION:-$(latest_release)}"
  asset="${BIN}-${triple}.tar.gz"
  base="https://github.com/${REPO}/releases/download/${version}"

  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT INT TERM

  echo "doctrine: downloading ${asset} (${version})"
  curl -fsSL -o "$tmp/$asset" "$base/$asset"
  curl -fsSL -o "$tmp/$asset.sha256" "$base/$asset.sha256"

  # Verify against the published checksum (the .sha256 names the bare asset).
  ( cd "$tmp" && verify_sha256 "$asset.sha256" >/dev/null ) || {
    echo "doctrine: checksum verification FAILED for $asset" >&2
    exit 1
  }

  tar -xzf "$tmp/$asset" -C "$tmp"
  [ -f "$tmp/$BIN" ] || {
    echo "doctrine: archive did not contain a '$BIN' executable" >&2
    exit 1
  }

  # Best-effort macOS Gatekeeper quarantine strip; no-op where xattr is absent
  # (e.g. Linux).
  xattr -d com.apple.quarantine "$tmp/$BIN" 2>/dev/null || true
  chmod +x "$tmp/$BIN"

  bindir="${DOCTRINE_BIN_DIR:-$HOME/.local/bin}"
  mkdir -p "$bindir"
  mv "$tmp/$BIN" "$bindir/$BIN"
  echo "doctrine: installed to $bindir/$BIN"

  case ":$PATH:" in
    *":$bindir:"*) ;;
    *) echo "doctrine: add it to your PATH:  export PATH=\"$bindir:\$PATH\"" ;;
  esac

  echo "doctrine: done — run '$BIN --version' to verify."
}

# Sourceable seam: define the functions but skip the installer when sourced
# lib-only (the unit test sets this). Otherwise run.
[ "${DOCTRINE_INSTALL_LIB_ONLY:-}" = "1" ] || main "$@"
