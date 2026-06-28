#!/usr/bin/env sh
# install.sh — no-compile installer for prebuilt doctrine macOS binaries.
#
#   curl -fsSL https://raw.githubusercontent.com/davidlee/doctrine/main/install.sh | sh
#
# Downloads the release asset for this machine's arch, verifies its sha256,
# strips the Gatekeeper quarantine, and installs `doctrine` to a bin dir. No
# Rust toolchain required — sidesteps the `cargo install` `-liconv` link failure
# (SL-174). Inspect this script before piping it to a shell (R6).
#
# Env:
#   DOCTRINE_VERSION   tag to install (default: latest non-prerelease release)
#   DOCTRINE_BIN_DIR   install dir (default: $HOME/.local/bin)
set -eu

# Single-source identifiers (STD-001 — no magic strings).
REPO="davidlee/doctrine"
BIN="doctrine"

# Pure: map a `uname -m` value to the release-asset target triple. Sourceable and
# unit-tested (scripts/install-test.sh) without running the installer.
triple_for_arch() {
  case "$1" in
    arm64) echo "aarch64-apple-darwin" ;;
    x86_64) echo "x86_64-apple-darwin" ;;
    *)
      echo "doctrine: unsupported architecture: $1" >&2
      return 1
      ;;
  esac
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
  if [ "$(uname -s)" != "Darwin" ]; then
    echo "doctrine: this installer supports macOS only (Linux is a follow-up)." >&2
    echo "doctrine: on other platforms, build from source: cargo install ${BIN}" >&2
    exit 1
  fi

  triple="$(triple_for_arch "$(uname -m)")"
  version="${DOCTRINE_VERSION:-$(latest_release)}"
  asset="${BIN}-${triple}.tar.gz"
  base="https://github.com/${REPO}/releases/download/${version}"

  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT INT TERM

  echo "doctrine: downloading ${asset} (${version})"
  curl -fsSL -o "$tmp/$asset" "$base/$asset"
  curl -fsSL -o "$tmp/$asset.sha256" "$base/$asset.sha256"

  # Verify against the published checksum (the .sha256 names the bare asset).
  ( cd "$tmp" && shasum -a 256 -c "$asset.sha256" >/dev/null ) || {
    echo "doctrine: checksum verification FAILED for $asset" >&2
    exit 1
  }

  tar -xzf "$tmp/$asset" -C "$tmp"
  [ -f "$tmp/$BIN" ] || {
    echo "doctrine: archive did not contain a '$BIN' executable" >&2
    exit 1
  }

  # Best-effort Gatekeeper quarantine strip (absent xattr/attr is non-fatal).
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
